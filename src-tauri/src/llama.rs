//! The generation runtime LocaLog spawns and owns.
//!
//! Transcription bundles whisper.cpp and downloads its models; generation asked
//! people to install Ollama and run a terminal command. This is the other half of
//! closing that gap: `llama-server` ships as a sidecar like every other runtime,
//! and the application starts it, waits for it, and stops it.
//!
//! ## Why owning the process matters more than it looks
//!
//! Ollama is a machine-wide daemon shared with whatever else is running, which is
//! why `PLAN.md` carries all that discipline about checking `ollama ps` and
//! discarding a run that slipped onto the CPU. A server this application starts
//! for itself, with a context size it chose, is not shared and does not have to be
//! guessed at.
//!
//! It also moves one thing: Ollama takes the context width per request, and
//! `llama-server` takes it at launch. So a width the machine cannot afford is now
//! refused before the model is loaded rather than discovered while it swaps.

use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// What the generation runtime is called, most preferred first.
///
/// The shipped name comes first for the reason every other runtime's does: a
/// packaged release that found a system installation before its own signed
/// sidecar would be running something nobody reviewed.
pub(crate) const LLAMA_SERVER_NAMES: &[&str] = &["localog-llama-server", "llama-server"];

/// How long the server may take to load a model and answer that it is ready.
///
/// Generous, because this is reading gigabytes from disk into memory: a 12B model
/// on a cold cache is tens of seconds, and being impatient here would kill a
/// server that was going to work. What this catches is the one that never
/// answers at all.
const READY_WITHIN: Duration = Duration::from_secs(180);

/// How often to ask, while waiting.
const ASK_EVERY: Duration = Duration::from_millis(250);

pub(crate) fn server_path() -> Option<PathBuf> {
    crate::runtime::discover_executable(LLAMA_SERVER_NAMES)
}

/// What to start, and how much room to give it.
#[derive(Clone, Debug)]
pub(crate) struct ServerRequest {
    pub model: PathBuf,
    /// Decided from the machine's memory before the model is loaded, because
    /// after it is loaded is too late.
    pub context_tokens: u32,
    /// 0 lets the runtime choose. Set for a machine where the default is wrong.
    pub gpu_layers: Option<u32>,
}

#[derive(Debug)]
pub(crate) enum StartFailure {
    /// The sidecar is not in the bundle and not on the path.
    Missing,
    /// No port could be claimed to give it.
    NoPort,
    /// The process would not start at all.
    LaunchFailed(String),
    /// It started and never reported itself ready.
    NeverReady,
    /// It started and then stopped on its own.
    Stopped(Option<i32>),
}

impl StartFailure {
    /// The code the interface renders. Words live in the dictionary, as they do
    /// for every other failure this application reports.
    pub(crate) fn code(&self) -> String {
        match self {
            Self::Missing => "generationRuntimeMissing".into(),
            Self::NoPort => "generationRuntimeNoPort".into(),
            Self::LaunchFailed(_) => "generationRuntimeNotStarted".into(),
            Self::NeverReady => "generationRuntimeNeverReady".into(),
            Self::Stopped(_) => "generationRuntimeStopped".into(),
        }
    }
}

/// A running generation server, stopped when this is dropped.
pub(crate) struct Server {
    child: Child,
    base_url: String,
}

impl Server {
    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for Server {
    /// Killed rather than asked politely.
    ///
    /// It holds gigabytes of a model and has nothing to write down, so there is
    /// nothing a graceful shutdown would save — and a generation runtime left
    /// resident after the application quit is exactly the failure this module
    /// exists to stop happening by accident.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A port nothing else is using, given up immediately so the server can take it.
///
/// There is a gap between letting go and the server binding, and something else
/// could take it in between. Asking the operating system is still better than
/// picking a number: a fixed port collides with a second copy of the application
/// and with whatever else on the machine happened to choose the same one.
fn free_port() -> Option<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).ok()?;
    let port = listener.local_addr().ok()?.port();
    drop(listener);
    Some(port)
}

/// Start a server for one model and wait until it will answer.
pub(crate) fn start(request: &ServerRequest) -> Result<Server, StartFailure> {
    let executable = server_path().ok_or(StartFailure::Missing)?;
    start_with(&executable, request)
}

pub(crate) fn start_with(
    executable: &Path,
    request: &ServerRequest,
) -> Result<Server, StartFailure> {
    let port = free_port().ok_or(StartFailure::NoPort)?;
    let mut command = Command::new(executable);
    command
        .arg("--model")
        .arg(&request.model)
        .arg("--ctx-size")
        .arg(request.context_tokens.to_string())
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        // Nothing reads them, and a server writing a progress bar into a pipe
        // nobody drains eventually blocks on the write.
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());
    if let Some(layers) = request.gpu_layers {
        command.arg("--n-gpu-layers").arg(layers.to_string());
    }

    let child = command.spawn().map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            StartFailure::Missing
        } else {
            StartFailure::LaunchFailed(error.to_string())
        }
    })?;

    let base_url = format!("http://127.0.0.1:{port}");
    let mut server = Server { child, base_url };
    wait_until_ready(&mut server)?;
    Ok(server)
}

/// Ask until it answers, it dies, or the deadline passes.
fn wait_until_ready(server: &mut Server) -> Result<(), StartFailure> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .build()
        .into();
    let health = format!("{}/health", server.base_url);
    let deadline = Instant::now() + READY_WITHIN;
    while Instant::now() < deadline {
        // A server that stopped will never become ready, and waiting the full
        // three minutes to say so helps nobody.
        if let Ok(Some(status)) = server.child.try_wait() {
            return Err(StartFailure::Stopped(status.code()));
        }
        if agent.get(&health).call().is_ok() {
            return Ok(());
        }
        std::thread::sleep(ASK_EVERY);
    }
    Err(StartFailure::NeverReady)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_port_is_asked_for_rather_than_chosen() {
        let first = free_port().expect("the operating system has a spare port");
        let second = free_port().expect("and another");
        assert!(first > 1024, "never a privileged port: {first}");
        assert!(second > 1024, "{second}");
    }

    /// The failure a fresh checkout hits, and it must name itself rather than
    /// arriving as a panic or an empty string.
    #[test]
    fn a_missing_runtime_is_named() {
        let absent = Path::new("/nonexistent/localog-llama-server");
        let request = ServerRequest {
            model: PathBuf::from("/nonexistent/model.gguf"),
            context_tokens: 16_384,
            gpu_layers: None,
        };
        let Err(failure) = start_with(absent, &request) else {
            panic!("there is nothing at that path to start");
        };
        assert_eq!(failure.code(), "generationRuntimeMissing");
    }

    /// Every failure carries a code the interface can render. A new variant
    /// without one would reach somebody as a blank.
    #[test]
    fn every_failure_has_a_code() {
        for failure in [
            StartFailure::Missing,
            StartFailure::NoPort,
            StartFailure::LaunchFailed("x".into()),
            StartFailure::NeverReady,
            StartFailure::Stopped(Some(1)),
        ] {
            let code = failure.code();
            assert!(!code.is_empty(), "{failure:?}");
            assert!(code.starts_with("generationRuntime"), "{code}");
        }
    }
}

#[cfg(test)]
mod against_the_real_runtime {
    use super::*;

    /// The bundled server, started with a model that is not there.
    ///
    /// It must be reported as stopped within seconds rather than waited on for
    /// three minutes: the deadline is sized for loading gigabytes, and applying
    /// it to a server that already exited would make every mistyped path feel
    /// like a hang.
    #[test]
    #[ignore = "needs the built sidecar; run after npm run build:sidecar"]
    fn a_server_that_dies_is_noticed_quickly() {
        let Some(executable) = server_path() else {
            return;
        };
        let began = Instant::now();
        let request = ServerRequest {
            model: PathBuf::from("/nonexistent/model.gguf"),
            context_tokens: 16_384,
            gpu_layers: None,
        };
        let Err(failure) = start_with(&executable, &request) else {
            panic!("a server with no model should not become ready");
        };
        assert_eq!(failure.code(), "generationRuntimeStopped", "{failure:?}");
        assert!(
            began.elapsed() < Duration::from_secs(30),
            "noticed after {:?}, which is waiting rather than watching",
            began.elapsed()
        );
    }
}
