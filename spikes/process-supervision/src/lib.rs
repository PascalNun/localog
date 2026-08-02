#![cfg(unix)]

use std::collections::VecDeque;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use std::os::unix::process::CommandExt;

const EVENT_CAPACITY: usize = 32;
const MAX_LOG_LINES: usize = 80;
const MAX_LOG_BYTES: usize = 16 * 1024;
const MAX_STAGE_CHARS: usize = 160;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

pub type Result<T> = std::result::Result<T, ProcessError>;

#[derive(Debug)]
pub enum ProcessError {
    Io(std::io::Error),
    Busy,
    TimedOut,
    SignalFailed(std::io::Error),
}

impl Display for ProcessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "process I/O error: {error}"),
            Self::Busy => write!(formatter, "the supervised heavy-job lane is already busy"),
            Self::TimedOut => write!(formatter, "the supervised process did not finish in time"),
            Self::SignalFailed(error) => write!(formatter, "process-group signal failed: {error}"),
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<std::io::Error> for ProcessError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug)]
pub struct ProcessSpec {
    pub program: PathBuf,
    pub arguments: Vec<OsString>,
    pub working_directory: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressEvent {
    pub percent: u8,
    pub stage: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationResult {
    pub forced: bool,
    pub elapsed: Duration,
    pub status: ExitStatus,
}

#[derive(Default)]
struct BoundedLog {
    lines: VecDeque<String>,
    bytes: usize,
}

impl BoundedLog {
    fn push(&mut self, line: String) {
        let line = truncate_chars(&line, 512);
        self.bytes += line.len();
        self.lines.push_back(line);
        while self.lines.len() > MAX_LOG_LINES || self.bytes > MAX_LOG_BYTES {
            if let Some(removed) = self.lines.pop_front() {
                self.bytes = self.bytes.saturating_sub(removed.len());
            } else {
                break;
            }
        }
    }

    fn snapshot(&self) -> Vec<String> {
        self.lines.iter().cloned().collect()
    }

    fn byte_count(&self) -> usize {
        self.bytes
    }
}

pub struct RunningProcess {
    child: Child,
    process_group: i32,
    events: Receiver<ProgressEvent>,
    stdout_tail: Arc<Mutex<BoundedLog>>,
    stderr_tail: Arc<Mutex<BoundedLog>>,
    readers: Vec<JoinHandle<()>>,
    exit_status: Option<ExitStatus>,
}

impl RunningProcess {
    pub fn spawn(spec: &ProcessSpec) -> Result<Self> {
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.arguments)
            .current_dir(&spec.working_directory)
            .env_clear()
            .env("LANG", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);

        let mut child = command.spawn()?;
        let process_group = child.id() as i32;
        let stdout = child.stdout.take().expect("piped stdout is available");
        let stderr = child.stderr.take().expect("piped stderr is available");
        let stdout_tail = Arc::new(Mutex::new(BoundedLog::default()));
        let stderr_tail = Arc::new(Mutex::new(BoundedLog::default()));
        let (event_sender, events) = sync_channel(EVENT_CAPACITY);

        let stdout_reader = spawn_stdout_reader(stdout, stdout_tail.clone(), event_sender);
        let stderr_reader = spawn_log_reader(stderr, stderr_tail.clone());

        Ok(Self {
            child,
            process_group,
            events,
            stdout_tail,
            stderr_tail,
            readers: vec![stdout_reader, stderr_reader],
            exit_status: None,
        })
    }

    pub fn process_group(&self) -> i32 {
        self.process_group
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        if let Some(status) = self.exit_status {
            return Ok(Some(status));
        }
        let status = self.child.try_wait()?;
        if let Some(status) = status {
            self.exit_status = Some(status);
            self.join_readers();
        }
        Ok(status)
    }

    pub fn wait_timeout(&mut self, timeout: Duration) -> Result<ExitStatus> {
        let started = Instant::now();
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            if started.elapsed() >= timeout {
                return Err(ProcessError::TimedOut);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn cancel(&mut self, grace: Duration) -> Result<CancellationResult> {
        let started = Instant::now();
        if let Some(status) = self.try_wait()? {
            return Ok(CancellationResult {
                forced: false,
                elapsed: started.elapsed(),
                status,
            });
        }

        signal_process_group(self.process_group, libc::SIGTERM)?;
        let grace_started = Instant::now();
        while grace_started.elapsed() < grace {
            if let Some(status) = self.try_wait()? {
                return Ok(CancellationResult {
                    forced: false,
                    elapsed: started.elapsed(),
                    status,
                });
            }
            thread::sleep(Duration::from_millis(10));
        }

        signal_process_group(self.process_group, libc::SIGKILL)?;
        let status = self.wait_timeout(Duration::from_secs(2))?;
        Ok(CancellationResult {
            forced: true,
            elapsed: started.elapsed(),
            status,
        })
    }

    pub fn drain_progress(&self) -> Vec<ProgressEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            events.push(event);
        }
        events
    }

    pub fn stdout_tail(&self) -> Vec<String> {
        self.stdout_tail.lock().expect("stdout log lock").snapshot()
    }

    pub fn stderr_tail(&self) -> Vec<String> {
        self.stderr_tail.lock().expect("stderr log lock").snapshot()
    }

    pub fn bounded_log_bytes(&self) -> usize {
        self.stdout_tail
            .lock()
            .expect("stdout log lock")
            .byte_count()
            + self
                .stderr_tail
                .lock()
                .expect("stderr log lock")
                .byte_count()
    }

    fn join_readers(&mut self) {
        for reader in self.readers.drain(..) {
            let _ = reader.join();
        }
    }
}

impl Drop for RunningProcess {
    fn drop(&mut self) {
        if self.exit_status.is_none() {
            let _ = signal_process_group(self.process_group, libc::SIGKILL);
            let _ = self.child.wait();
        }
        self.join_readers();
    }
}

#[derive(Default)]
pub struct HeavyJobLane {
    active: Option<RunningProcess>,
}

impl HeavyJobLane {
    pub fn start(&mut self, spec: &ProcessSpec) -> Result<&mut RunningProcess> {
        if let Some(active) = self.active.as_mut()
            && active.try_wait()?.is_none()
        {
            return Err(ProcessError::Busy);
        }
        self.active = Some(RunningProcess::spawn(spec)?);
        Ok(self.active.as_mut().expect("active process was inserted"))
    }

    pub fn active_mut(&mut self) -> Option<&mut RunningProcess> {
        self.active.as_mut()
    }
}

fn spawn_stdout_reader(
    stdout: impl std::io::Read + Send + 'static,
    tail: Arc<Mutex<BoundedLog>>,
    events: SyncSender<ProgressEvent>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut last_progress = Instant::now() - PROGRESS_INTERVAL;
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if let Some(progress) = parse_progress(&line) {
                if progress.percent == 100 || last_progress.elapsed() >= PROGRESS_INTERVAL {
                    let _ = events.try_send(progress);
                    last_progress = Instant::now();
                }
            } else {
                tail.lock().expect("stdout log lock").push(line);
            }
        }
    })
}

fn spawn_log_reader(
    stderr: impl std::io::Read + Send + 'static,
    tail: Arc<Mutex<BoundedLog>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let Ok(line) = line else { break };
            tail.lock().expect("stderr log lock").push(line);
        }
    })
}

fn parse_progress(line: &str) -> Option<ProgressEvent> {
    let payload = line.strip_prefix("PROGRESS ")?;
    let (percent, stage) = payload.split_once(' ')?;
    let percent = percent.parse::<u8>().ok()?;
    if percent > 100 || stage.trim().is_empty() {
        return None;
    }
    Some(ProgressEvent {
        percent,
        stage: truncate_chars(stage.trim(), MAX_STAGE_CHARS),
    })
}

fn truncate_chars(value: &str, maximum: usize) -> String {
    value.chars().take(maximum).collect()
}

fn signal_process_group(process_group: i32, signal: i32) -> Result<()> {
    // SAFETY: `process_group` comes from the spawned child PID. A negative PID targets only that
    // dedicated process group, which was created with `CommandExt::process_group(0)`.
    let result = unsafe { libc::kill(-process_group, signal) };
    if result == 0 {
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(ProcessError::SignalFailed(error))
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn progress_parser_rejects_malformed_and_oversized_values() {
        assert_eq!(
            parse_progress("PROGRESS 42 normalizing audio"),
            Some(ProgressEvent {
                percent: 42,
                stage: "normalizing audio".to_string()
            })
        );
        assert_eq!(parse_progress("PROGRESS 101 invalid"), None);
        assert_eq!(parse_progress("PROGRESS nope invalid"), None);
        assert_eq!(parse_progress("ordinary output"), None);
    }
}
