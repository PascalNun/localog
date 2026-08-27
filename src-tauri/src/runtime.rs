//! Small, explicit boundary for user-provided local runtimes.

use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs::File;
#[cfg(unix)]
use std::io::ErrorKind;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub(crate) struct RuntimeConfig {
    pub executable: PathBuf,
    pub model: PathBuf,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResolvedTranscriptionConfig {
    pub executable_path: PathBuf,
    pub model_path: PathBuf,
    pub runtime_version: String,
    pub model_digest: String,
    pub model_byte_count: u64,
    pub language_code: String,
    pub sample_rate: u32,
    pub channels: u8,
    pub codec: String,
    pub container: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ModelProvenance {
    pub digest: String,
    pub byte_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModelFileIdentity {
    pub byte_count: u64,
    pub modified_at_ns: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProcessLimits {
    pub timeout: Duration,
    pub termination_grace: Duration,
    pub max_stdout_tail_bytes: usize,
    pub max_stderr_tail_bytes: usize,
}

impl ProcessLimits {
    pub(crate) fn with_max_output(max_output: usize) -> Self {
        Self {
            timeout: Duration::from_secs(2 * 60 * 60),
            termination_grace: Duration::from_millis(500),
            max_stdout_tail_bytes: max_output,
            max_stderr_tail_bytes: max_output,
        }
    }

    fn version() -> Self {
        Self {
            timeout: Duration::from_secs(3),
            termination_grace: Duration::from_millis(250),
            max_stdout_tail_bytes: 16 * 1024,
            max_stderr_tail_bytes: 16 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProcessFailure {
    Cancelled,
    TimedOut,
    LaunchFailed,
    WaitFailed,
    OutputReadFailed,
    Exited(Option<i32>),
}

impl Display for ProcessFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Cancelled => "cancelled",
            Self::TimedOut => "runtime timed out",
            Self::LaunchFailed => "runtime could not be started",
            Self::WaitFailed => "runtime status could not be read",
            Self::OutputReadFailed => "runtime output could not be read",
            Self::Exited(Some(code)) => {
                return write!(formatter, "runtime exited with status {code}");
            }
            Self::Exited(None) => "runtime exited without a status code",
        };
        formatter.write_str(message)
    }
}

pub(crate) fn validate_config(executable: &Path, model: &Path) -> Result<RuntimeConfig, String> {
    if !executable.is_absolute() || !model.is_absolute() {
        return Err("runtimePathsMustBeAbsolute".into());
    }
    if !executable.is_file() {
        return Err("whisperExecutableMissing".into());
    }
    if !model.is_file() {
        return Err("whisperModelMissing".into());
    }
    Ok(RuntimeConfig {
        executable: executable.to_path_buf(),
        model: model.to_path_buf(),
    })
}

/// Find a bundled or PATH-provided runtime without asking the person using the
/// app to browse for an executable. The explicit setting still wins, which is
/// useful for development and for a future signed sidecar override.
pub(crate) fn discover_executable(names: &[&str]) -> Option<PathBuf> {
    let beside_the_app = std::env::current_exe()
        .ok()
        .and_then(|current| current.parent().map(Path::to_path_buf));
    let on_path = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default();
    let locations = search_locations(beside_the_app.as_deref(), on_path);
    names.iter().find_map(|name| {
        locations.iter().find_map(|directory| {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
            // Tauri's development layout keeps the target triple in the file
            // name; release bundles use the logical base name. Accept both.
            std::fs::read_dir(directory)
                .ok()?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| {
                    path.is_file()
                        && path
                            .file_name()
                            .and_then(|value| value.to_str())
                            .is_some_and(|value| {
                                value.starts_with(&format!("{name}-"))
                                    || value.starts_with(&format!("{name}."))
                            })
                })
        })
    })
}

/// What the transcription runtime is called, most preferred first.
///
/// The name the application ships under comes first, so a packaged build uses
/// the reviewed sidecar rather than a developer's own checkout. The upstream
/// names follow, because a contributor with whisper.cpp on their PATH should not
/// have to build a sidecar to run the application.
///
/// Named here so the three places that look for it cannot drift apart. They did:
/// the diariser was already found by its shipped name while whisper was not, so
/// a bundled transcription runtime would have been packaged and never used.
pub(crate) const WHISPER_NAMES: &[&str] = &["localog-whisper", "whisper-cli", "whisper-cpp"];

/// What the speaker-separation runtime is called, most preferred first.
pub(crate) const DIARISER_NAMES: &[&str] = &[
    "localog-speaker-diarization",
    "sherpa-onnx-offline-speaker-diarization",
    "sherpa-onnx-speaker-diarization",
];

/// What the media tools are called, most preferred first.
///
/// The shipped names come first for the same reason as the others: a packaged
/// release that found a system installation before its own signed sidecar would
/// be running something nobody reviewed. The upstream names follow, because
/// almost every developer machine already has these two and should not have to
/// build a sidecar to run the application.
pub(crate) const FFMPEG_NAMES: &[&str] = &["localog-ffmpeg", "ffmpeg"];
pub(crate) const FFPROBE_NAMES: &[&str] = &["localog-ffprobe", "ffprobe"];

/// What the speaker-embedding runtime is called.
///
/// Only the shipped name: unlike whisper and the diariser, this executable is
/// ours rather than an upstream tool somebody might already have, so there is no
/// second name that could mean the same thing.
pub(crate) const EMBEDDING_NAMES: &[&str] = &["localog-speaker-embedding"];

/// Where a runtime is looked for, in the order it is looked for.
///
/// What ships with the application comes first. A packaged release that found a
/// system installation before its own signed sidecar would be running something
/// nobody reviewed, on a machine whose owner was told the application brings its
/// own runtimes — so this order is the distribution promise, not a preference.
///
/// It is a separate function because the order was wrong for as long as it was
/// only a comment: the locations were appended to the PATH entries and therefore
/// searched last, directly against what the comment beside them claimed.
fn search_locations(beside_the_app: Option<&Path>, on_path: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut locations = Vec::new();
    if let Some(parent) = beside_the_app {
        // Tauri puts external binaries beside the executable in development and
        // inside the bundle in a release.
        locations.push(parent.to_path_buf());
        locations.push(parent.join("Resources"));
        locations.push(parent.join("..").join("Resources"));
        locations.push(parent.join("binaries"));
    }
    #[cfg(debug_assertions)]
    locations.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries"));
    locations.extend(on_path);
    locations
}

/// A cheap launch probe used for optional runtimes. The probe is deliberately
/// bounded and never runs on the UI thread; it catches a missing executable or
/// an incompatible architecture before a meeting is started.
pub(crate) fn executable_health(path: &Path) -> bool {
    let mut command = Command::new(path);
    command.arg("--help");
    let token = AtomicBool::new(false);
    run_process(command, &token, ProcessLimits::version()).is_ok()
}

pub(crate) fn model_provenance(path: &Path) -> std::io::Result<ModelProvenance> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }
    Ok(ModelProvenance {
        digest: format!("sha256:{:x}", hasher.finalize()),
        byte_count: bytes,
    })
}

pub(crate) fn model_file_identity(path: &Path) -> std::io::Result<ModelFileIdentity> {
    let metadata = std::fs::metadata(path)?;
    let modified_at = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(std::io::Error::other)?;
    Ok(ModelFileIdentity {
        byte_count: metadata.len(),
        modified_at_ns: modified_at.as_nanos().to_string(),
    })
}

#[derive(Debug)]
pub(crate) struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
}

fn no_progress(_line: &str) -> Option<u8> {
    None
}

/// Run a bounded-output child process, checking cancellation without blocking the UI thread.
pub(crate) fn run_process(
    command: Command,
    cancellation: &AtomicBool,
    limits: ProcessLimits,
) -> Result<ProcessOutput, ProcessFailure> {
    run_process_with_progress(command, cancellation, limits, no_progress, |_| {})
}

/// As `run_process`, but parses each stderr line through `parse_progress` and
/// reports matched 0..=100 percentages to `on_progress` while the process runs.
/// `on_progress` runs on the calling thread, so it may touch non-`Send` state.
pub(crate) fn run_process_with_progress(
    mut command: Command,
    cancellation: &AtomicBool,
    limits: ProcessLimits,
    parse_progress: fn(&str) -> Option<u8>,
    mut on_progress: impl FnMut(u8),
) -> Result<ProcessOutput, ProcessFailure> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Keep termination scoped to the runtime and any children it starts.
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) == 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::last_os_error())
                }
            });
        }
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|_| ProcessFailure::LaunchFailed)?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate(&mut child, limits.termination_grace);
            return Err(ProcessFailure::OutputReadFailed);
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            terminate(&mut child, limits.termination_grace);
            return Err(ProcessFailure::OutputReadFailed);
        }
    };
    let (progress_sender, progress_receiver) = std::sync::mpsc::channel::<u8>();
    let stdout_thread = thread::spawn(move || read_tail(stdout, limits.max_stdout_tail_bytes));
    let stderr_thread = thread::spawn(move || {
        read_tail_with_progress(
            stderr,
            limits.max_stderr_tail_bytes,
            parse_progress,
            progress_sender,
        )
    });
    let started_at = Instant::now();
    loop {
        while let Ok(value) = progress_receiver.try_recv() {
            on_progress(value);
        }
        if cancellation.load(Ordering::Acquire) {
            terminate(&mut child, limits.termination_grace);
            let _ = stdout_thread.join();
            let _ = stderr_thread.join();
            return Err(ProcessFailure::Cancelled);
        }
        if started_at.elapsed() >= limits.timeout {
            terminate(&mut child, limits.termination_grace);
            let _ = stdout_thread.join();
            let _ = stderr_thread.join();
            return Err(ProcessFailure::TimedOut);
        }
        let finished = match child.try_wait() {
            Ok(status) => status.is_some(),
            Err(_) => {
                terminate(&mut child, limits.termination_grace);
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();
                return Err(ProcessFailure::WaitFailed);
            }
        };
        if finished {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let status = child.wait().map_err(|_| ProcessFailure::WaitFailed)?;
    let stdout = stdout_thread
        .join()
        .map_err(|_| ProcessFailure::OutputReadFailed)?
        .map_err(|_| ProcessFailure::OutputReadFailed)?;
    let stderr = stderr_thread
        .join()
        .map_err(|_| ProcessFailure::OutputReadFailed)?
        .map_err(|_| ProcessFailure::OutputReadFailed)?;
    // Drain any progress produced after the final poll but before the reader closed.
    while let Ok(value) = progress_receiver.try_recv() {
        on_progress(value);
    }
    if !status.success() {
        return Err(ProcessFailure::Exited(status.code()));
    }
    Ok(ProcessOutput {
        stdout: stdout.text,
        stderr: stderr.text,
    })
}

struct TailOutput {
    text: String,
}

fn read_tail(mut reader: impl Read, max: usize) -> std::io::Result<TailOutput> {
    let mut tail = VecDeque::with_capacity(max.min(8192));
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            if tail.len() == max {
                let _ = tail.pop_front();
            }
            if max > 0 {
                tail.push_back(*byte);
            }
        }
    }
    Ok(TailOutput {
        text: String::from_utf8_lossy(&tail.into_iter().collect::<Vec<_>>()).into_owned(),
    })
}

/// Like `read_tail`, but also accumulates complete lines and forwards any parsed
/// progress percentage over `progress`. Keeps the same bounded diagnostic tail.
fn read_tail_with_progress(
    mut reader: impl Read,
    max: usize,
    parse_progress: fn(&str) -> Option<u8>,
    progress: std::sync::mpsc::Sender<u8>,
) -> std::io::Result<TailOutput> {
    let mut tail = VecDeque::with_capacity(max.min(8192));
    let mut line = String::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            if tail.len() == max {
                let _ = tail.pop_front();
            }
            if max > 0 {
                tail.push_back(*byte);
            }
            if *byte == b'\n' || *byte == b'\r' {
                if let Some(value) = parse_progress(&line) {
                    let _ = progress.send(value);
                }
                line.clear();
            } else if line.len() < 4096 {
                // Bound the line buffer so a stream without newlines cannot grow unbounded.
                line.push(*byte as char);
            }
        }
    }
    if let Some(value) = parse_progress(&line) {
        let _ = progress.send(value);
    }
    Ok(TailOutput {
        text: String::from_utf8_lossy(&tail.into_iter().collect::<Vec<_>>()).into_owned(),
    })
}

fn terminate(child: &mut Child, grace: Duration) {
    #[cfg(unix)]
    {
        let process_group = -(child.id() as i32);
        unsafe {
            let _ = libc::kill(process_group, libc::SIGTERM);
        }
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            let child_finished = child.try_wait().ok().flatten().is_some();
            if child_finished && !process_group_exists(process_group) {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        unsafe {
            // The runtime may have spawned children that ignore SIGTERM.
            let _ = libc::kill(process_group, libc::SIGKILL);
        }
        let _ = child.wait();
    }
    #[cfg(not(unix))]
    {
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            if child.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(unix)]
fn process_group_exists(process_group: i32) -> bool {
    let result = unsafe { libc::kill(process_group, 0) };
    result == 0 || std::io::Error::last_os_error().kind() == ErrorKind::PermissionDenied
}

pub(crate) fn executable_version(path: &Path) -> Option<String> {
    executable_version_with_argument(path, "--version")
}

pub(crate) fn ffmpeg_version(path: &Path) -> Option<String> {
    executable_version_with_argument(path, "-version")
}

fn executable_version_with_argument(path: &Path, argument: &str) -> Option<String> {
    let mut command = Command::new(path);
    command.arg(argument);
    let token = AtomicBool::new(false);
    run_process(command, &token, ProcessLimits::version())
        .ok()
        .and_then(|output| {
            first_non_empty_line(&output.stdout).or_else(|| first_non_empty_line(&output.stderr))
        })
}

fn first_non_empty_line(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The order is the distribution promise: a release must run the runtime it
    /// shipped with, not one it happened to find on the machine. This was wrong
    /// while it was only a comment -- the bundle locations were appended to the
    /// PATH entries and so were searched last.
    #[test]
    fn what_ships_with_the_application_is_searched_before_the_machine() {
        let app = PathBuf::from("/Applications/LocaLog.app/Contents/MacOS");
        let on_path = vec![
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/opt/homebrew/bin"),
        ];
        let locations = search_locations(Some(&app), on_path.clone());

        let first_system = locations
            .iter()
            .position(|location| on_path.contains(location))
            .expect("the machine's own locations are still searched");
        let last_bundled = locations
            .iter()
            .rposition(|location| location.starts_with(&app))
            .expect("the bundle is searched");
        assert!(
            last_bundled < first_system,
            "every bundled location must come before any system one, got {locations:?}"
        );
    }

    /// A packaged build must reach its own runtime first. Both were declared as
    /// sidecars, but only one was looked for by the name it ships under, so the
    /// transcription runtime would have been bundled, signed and never used.
    #[test]
    fn each_runtime_is_looked_for_by_the_name_it_ships_under() {
        assert_eq!(WHISPER_NAMES.first(), Some(&"localog-whisper"));
        assert_eq!(DIARISER_NAMES.first(), Some(&"localog-speaker-diarization"));
        // A contributor with the upstream build on their PATH still needs it to
        // be found, so the shipped name is a preference and not the only option.
        assert!(WHISPER_NAMES.contains(&"whisper-cli"));
        assert!(DIARISER_NAMES.contains(&"sherpa-onnx-offline-speaker-diarization"));
        // The embedding sidecar is ours, so it has exactly one name and no
        // upstream alternative that could mean the same thing.
        assert_eq!(EMBEDDING_NAMES, &["localog-speaker-embedding"]);
        // FFmpeg was found only on PATH for as long as it was looked for by a
        // different function than everything else, so a bundled copy would have
        // been packaged, signed and never used - exactly the fault this list was
        // written to stop happening twice.
        assert_eq!(FFMPEG_NAMES.first(), Some(&"localog-ffmpeg"));
        assert_eq!(FFPROBE_NAMES.first(), Some(&"localog-ffprobe"));
        assert!(FFMPEG_NAMES.contains(&"ffmpeg"));
        assert!(FFPROBE_NAMES.contains(&"ffprobe"));
    }

    /// Without a known application directory there is nothing to prefer, and the
    /// machine's own locations must still be used rather than nothing at all.
    #[test]
    fn the_machine_is_still_searched_when_the_bundle_is_unknown() {
        let on_path = vec![PathBuf::from("/usr/bin")];
        assert!(search_locations(None, on_path.clone()).ends_with(&on_path));
    }

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;
    #[cfg(unix)]
    use tempfile::tempdir;

    #[test]
    fn bounded_reader_drains_and_keeps_only_the_tail() {
        let output = read_tail("0123456789".as_bytes(), 4).unwrap();
        assert_eq!(output.text, "6789");
    }

    #[cfg(unix)]
    #[test]
    fn process_drains_large_output_without_failing() {
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "i=0; while [ $i -lt 100000 ]; do printf x; i=$((i+1)); done",
        ]);
        let output = run_process(
            command,
            &AtomicBool::new(false),
            ProcessLimits {
                timeout: Duration::from_secs(5),
                termination_grace: Duration::from_millis(250),
                max_stdout_tail_bytes: 128,
                max_stderr_tail_bytes: 128,
            },
        )
        .unwrap();
        assert_eq!(output.stdout.len(), 128);
    }

    #[cfg(unix)]
    #[test]
    fn timeout_returns_a_typed_failure() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);
        let result = run_process(
            command,
            &AtomicBool::new(false),
            ProcessLimits {
                timeout: Duration::from_millis(100),
                termination_grace: Duration::from_millis(100),
                max_stdout_tail_bytes: 128,
                max_stderr_tail_bytes: 128,
            },
        );
        assert_eq!(result.unwrap_err(), ProcessFailure::TimedOut);
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_returns_a_typed_failure() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);
        let result = run_process(
            command,
            &AtomicBool::new(true),
            ProcessLimits::with_max_output(128),
        );
        assert_eq!(result.unwrap_err(), ProcessFailure::Cancelled);
    }

    #[cfg(unix)]
    #[test]
    fn non_zero_exit_preserves_the_status_code() {
        let mut command = Command::new("sh");
        command.args(["-c", "exit 7"]);
        let result = run_process(
            command,
            &AtomicBool::new(false),
            ProcessLimits::with_max_output(128),
        );
        assert_eq!(result.unwrap_err(), ProcessFailure::Exited(Some(7)));
    }

    #[cfg(unix)]
    #[test]
    fn ffmpeg_version_uses_the_tool_specific_argument() {
        let temporary = tempdir().unwrap();
        let script = temporary.path().join("version-script");
        std::fs::write(
            &script,
            b"#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffmpeg synthetic 1'; else exit 7; fi\n",
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();

        assert_eq!(
            ffmpeg_version(&script).as_deref(),
            Some("ffmpeg synthetic 1")
        );
        assert_eq!(executable_version(&script), None);
    }

    #[cfg(unix)]
    #[test]
    fn streams_parsed_progress_from_stderr_while_running() {
        fn parse(line: &str) -> Option<u8> {
            line.strip_prefix("progress = ")?
                .strip_suffix('%')?
                .parse()
                .ok()
        }
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "echo 'progress = 10%' 1>&2; echo 'progress = 60%' 1>&2; \
             echo 'progress = 100%' 1>&2; echo done",
        ]);
        let mut seen = Vec::new();
        let output = run_process_with_progress(
            command,
            &AtomicBool::new(false),
            ProcessLimits::with_max_output(4096),
            parse,
            |value| seen.push(value),
        )
        .unwrap();
        assert_eq!(seen, vec![10, 60, 100]);
        assert!(output.stdout.contains("done"));
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_kills_a_process_group_that_ignores_sigterm() {
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "trap '' TERM; (trap '' TERM; while :; do sleep 1; done) & wait",
        ]);
        let result = run_process(
            command,
            &AtomicBool::new(true),
            ProcessLimits {
                timeout: Duration::from_secs(5),
                termination_grace: Duration::from_millis(100),
                max_stdout_tail_bytes: 128,
                max_stderr_tail_bytes: 128,
            },
        );
        assert_eq!(result.unwrap_err(), ProcessFailure::Cancelled);
    }
}
