//! Small, explicit boundary for user-provided local runtimes.

use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub(crate) struct RuntimeConfig {
    pub executable: PathBuf,
    pub model: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct ModelProvenance {
    pub digest: String,
    pub byte_count: u64,
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
        return Err("Choose absolute paths for the whisper.cpp executable and model.".into());
    }
    if !executable.is_file() {
        return Err("The selected whisper.cpp executable was not found.".into());
    }
    if !model.is_file() {
        return Err("The selected whisper.cpp model was not found.".into());
    }
    Ok(RuntimeConfig {
        executable: executable.to_path_buf(),
        model: model.to_path_buf(),
    })
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

#[derive(Debug)]
pub(crate) struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Run a bounded-output child process, checking cancellation without blocking the UI thread.
pub(crate) fn run_process(
    mut command: Command,
    cancellation: &AtomicBool,
    limits: ProcessLimits,
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
    let stdout = child
        .stdout
        .take()
        .ok_or(ProcessFailure::OutputReadFailed)?;
    let stderr = child
        .stderr
        .take()
        .ok_or(ProcessFailure::OutputReadFailed)?;
    let stdout_thread = thread::spawn(move || read_tail(stdout, limits.max_stdout_tail_bytes));
    let stderr_thread = thread::spawn(move || read_tail(stderr, limits.max_stderr_tail_bytes));
    let started_at = Instant::now();
    loop {
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
        if child
            .try_wait()
            .map_err(|_| ProcessFailure::WaitFailed)?
            .is_some()
        {
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

fn terminate(child: &mut Child, grace: Duration) {
    #[cfg(unix)]
    unsafe {
        let _ = libc::kill(-(child.id() as i32), libc::SIGTERM);
    }
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    let _ = child.kill();
}

pub(crate) fn executable_version(path: &Path) -> Option<String> {
    let mut command = Command::new(path);
    command.arg("--version");
    let token = AtomicBool::new(false);
    run_process(command, &token, ProcessLimits::version())
        .ok()
        .map(|output| {
            output
                .stdout
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

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
}
