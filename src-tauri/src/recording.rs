//! Recording a meeting, and stopping cleanly whatever happens.
//!
//! The recorder is a supervised subprocess like the transcription and diarisation
//! runtimes: it is given two paths and writes two files, one line of JSON per second
//! on its output, and nothing here knows what a process tap is.
//!
//! ```text
//! record-meeting --system <path.wav> --microphone <path.wav>
//! ```
//!
//! **Cleanup is the dangerous part and is why this module exists.** During this
//! project's own study, orphaned recorders held Core Audio taps after being killed,
//! `coreaudiod` went to 43 % of a core, and the machine lost its audio entirely until
//! the orphans were found and killed by hand. A recorder that outlives the
//! application is not a leaked process, it is a broken laptop. So a running recorder
//! is registered on disk before it starts, stopped on every exit path this can reach,
//! and swept for at startup.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// What the recorder is called, most preferred first, following the same rule as the
/// other runtimes: the name the application ships under comes before a developer's
/// own checkout.
const RECORDER_NAMES: &[&str] = &["localog-record-meeting", "record-meeting"];

/// Where a running recorder's process id is written, so a later run can find one this
/// one left behind.
const RUNNING_MARKER: &str = "recording.pid";

/// One second of a recording, as the recorder reports it.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Level {
    pub seconds: u64,
    /// Loudest sample in the last second, 0.0 to 1.0.
    pub system_peak: f32,
    pub microphone_peak: f32,
}

/// A recording in progress.
pub(crate) struct Recording {
    child: Child,
    /// The latest second the recorder reported, shared with the thread reading it.
    latest: Arc<Mutex<Level>>,
    /// What the recorder said went wrong, in its own words.
    ///
    /// The recorder writes a line to its error output when it cannot capture one
    /// of the two sources and carries on with the other — an unpermitted tap, a
    /// Core Audio status nobody expected, a microphone another application is
    /// holding. That output was piped and never read, so every one of those
    /// explanations was written into a pipe and dropped. The permission case is
    /// now caught before a meeting starts; the rest are only knowable from here.
    notes: Arc<Mutex<Vec<String>>>,
    stopping: Arc<AtomicBool>,
    pub system_path: PathBuf,
    pub microphone_path: PathBuf,
    marker: PathBuf,
}

/// Whether a recorder is available to run at all.
pub(crate) fn recorder_path() -> Option<PathBuf> {
    crate::runtime::discover_executable(RECORDER_NAMES)
}

/// What this machine will allow, asked before a meeting rather than during one.
///
/// Every value is a string the recorder chose, passed through rather than parsed
/// into an enum here. There are two platforms still to write recorders for, and a
/// closed set defined on this side would have to be widened for each of them by
/// somebody who cannot see what the new recorder actually answers.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Permissions {
    /// "granted", or "not-granted" — the preflight cannot tell never-asked from
    /// refused, and says so rather than guessing.
    pub system_audio: String,
    /// "granted", "denied", "restricted", or "undetermined". Undetermined is a
    /// normal first run: macOS asks the first time, and nothing is wrong.
    pub microphone: String,
    /// Set when the question could not be put at all, which is not the same as an
    /// answer of no. A missing recorder is a broken installation, not a refused
    /// permission, and telling somebody to visit System Settings would send them
    /// to fix something that is not broken.
    pub unavailable: Option<String>,
}

/// Ask the recorder what it will be allowed to capture.
///
/// Cheap enough to call whenever the record screen opens: the recorder answers and
/// exits without creating a tap, a file, or a device.
pub(crate) fn permissions() -> Permissions {
    let Some(recorder) = recorder_path() else {
        return Permissions {
            unavailable: Some("No recorder is installed. LocaLog ships one; this build cannot find it.".into()),
            ..Permissions::default()
        };
    };
    let output = Command::new(&recorder).arg("--check").output();
    match output {
        Ok(output) if output.status.success() => serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|_| Permissions {
                unavailable: Some("The recorder did not say what it is allowed to do.".into()),
                ..Permissions::default()
            }),
        // An older recorder predating --check exits non-zero on the usage guard.
        // Saying nothing is known is honest; claiming a refusal would not be.
        _ => Permissions {
            unavailable: Some("This recorder cannot report what it is allowed to do.".into()),
            ..Permissions::default()
        },
    }
}

/// Kill any recorder a previous run left behind.
///
/// Called at startup, before anything else touches audio. A recorder that survived a
/// crash is still holding a tap, and on macOS that is what takes the machine's sound
/// away — so this runs even when the application has no intention of recording.
pub(crate) fn sweep_orphans(root: &Path) {
    let marker = root.join(RUNNING_MARKER);
    let Ok(contents) = std::fs::read_to_string(&marker) else {
        return;
    };
    if let Ok(pid) = contents.trim().parse::<i32>() {
        // A polite stop first: the recorder finalises both files on SIGTERM, so an
        // orphan's recording is still playable afterwards.
        stop_process(pid);
    }
    let _ = std::fs::remove_file(&marker);
}

/// Ask a process to stop, and insist if it does not.
///
/// Only ever a real, positive process id. `kill` reads zero as "every process in my
/// own group" and a negative number as "that group", so a marker holding either —
/// truncated, half-written, or from a crash mid-write — would turn this cleanup into
/// the thing it exists to prevent. Writing that guard cost a test run that terminated
/// its own test runner.
#[cfg(unix)]
fn stop_process(pid: i32) {
    if pid <= 0 {
        return;
    }
    // Safety: sending a signal cannot corrupt this process's memory, and a pid that
    // has already exited simply returns an error.
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
    // The recorder finalises within one checkpoint. Beyond that it is not stopping.
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let alive = unsafe { libc::kill(pid, 0) == 0 };
        if !alive {
            return;
        }
    }
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
}

#[cfg(not(unix))]
fn stop_process(_pid: i32) {}

impl Recording {
    /// Start recording to two files.
    ///
    /// The marker is written before the process starts rather than after, because the
    /// failure that matters is a recorder running with nothing recording that it is.
    pub(crate) fn start(
        root: &Path,
        system_path: PathBuf,
        microphone_path: PathBuf,
    ) -> Result<Self, String> {
        let recorder = recorder_path().ok_or_else(|| {
            "No recorder is installed. LocaLog ships one; this build cannot find it.".to_string()
        })?;
        sweep_orphans(root);

        let mut child = Command::new(&recorder)
            .arg("--system")
            .arg(&system_path)
            .arg("--microphone")
            .arg(&microphone_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("The recorder could not be started: {error}"))?;

        let marker = root.join(RUNNING_MARKER);
        let _ = std::fs::write(&marker, child.id().to_string());

        let latest = Arc::new(Mutex::new(Level::default()));
        let notes = Arc::new(Mutex::new(Vec::new()));
        let stopping = Arc::new(AtomicBool::new(false));
        if let Some(errors) = child.stderr.take() {
            let notes = Arc::clone(&notes);
            std::thread::spawn(move || {
                for line in BufReader::new(errors).lines().map_while(Result::ok) {
                    let line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(mut held) = notes.lock() {
                        // A recorder that somehow produced output without end must
                        // not grow this without end either.
                        if held.len() < 20 && !held.contains(&line) {
                            held.push(line);
                        }
                    }
                }
            });
        }
        if let Some(output) = child.stdout.take() {
            let latest = Arc::clone(&latest);
            let stopping = Arc::clone(&stopping);
            std::thread::spawn(move || {
                for line in BufReader::new(output).lines().map_while(Result::ok) {
                    if stopping.load(Ordering::Acquire) {
                        return;
                    }
                    if let (Ok(level), Ok(mut held)) =
                        (serde_json::from_str::<Level>(&line), latest.lock())
                    {
                        *held = level;
                    }
                }
            });
        }

        Ok(Self {
            child,
            latest,
            notes,
            stopping,
            system_path,
            microphone_path,
            marker,
        })
    }

    /// The most recent second the recorder reported.
    pub(crate) fn level(&self) -> Level {
        self.latest.lock().map(|held| *held).unwrap_or_default()
    }

    /// What the recorder has said went wrong, in its own words.
    pub(crate) fn notes(&self) -> Vec<String> {
        self.notes.lock().map(|held| held.clone()).unwrap_or_default()
    }

    /// Whether the recorder is still running, which is not the same as having been
    /// asked to stop: it can die on its own and somebody has to be told.
    pub(crate) fn still_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Stop cleanly and leave both files finalised.
    pub(crate) fn stop(mut self) -> Result<(PathBuf, PathBuf), String> {
        self.stopping.store(true, Ordering::Release);
        self.terminate();
        let _ = std::fs::remove_file(&self.marker);
        Ok((self.system_path.clone(), self.microphone_path.clone()))
    }

    #[cfg(unix)]
    fn terminate(&mut self) {
        stop_process(self.child.id() as i32);
        let _ = self.child.wait();
    }

    #[cfg(not(unix))]
    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Stopping when the value goes away, whatever took it away.
///
/// A recorder left running holds an audio tap, and on macOS that costs the machine
/// its sound. Every path out of a recording therefore ends here, including the ones
/// nobody wrote: a panic unwinding, an error returning early, the application
/// closing while a recording is in progress.
impl Drop for Recording {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
        self.terminate();
        let _ = std::fs::remove_file(&self.marker);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_level_line_from_the_recorder_is_understood() {
        let line = r#"{"seconds":12,"systemPeak":0.42,"microphonePeak":0.918}"#;
        let level: Level = serde_json::from_str(line).unwrap();
        assert_eq!(level.seconds, 12);
        assert!((level.microphone_peak - 0.918).abs() < f32::EPSILON);
    }

    /// The recorder writes one line per second and the reader must not stall on a
    /// line it does not recognise — a warning on the same stream would otherwise end
    /// the level display for the rest of the meeting.
    #[test]
    fn an_unrecognised_line_is_not_a_level() {
        assert!(serde_json::from_str::<Level>("starting up").is_err());
        assert!(serde_json::from_str::<Level>("").is_err());
    }

    #[test]
    fn sweeping_without_a_marker_does_nothing() {
        let temporary = tempfile::tempdir().unwrap();
        sweep_orphans(temporary.path());
        assert!(!temporary.path().join(RUNNING_MARKER).exists());
    }

    /// A marker naming a process that is long gone must be cleared rather than left
    /// to make every future startup try to kill a stranger.
    /// Zero means "my whole process group" to `kill`, and a negative number means
    /// "that group". A marker holding either must be discarded, not acted on: this
    /// test terminated its own test runner before the guard existed.
    #[test]
    fn a_marker_that_would_signal_a_whole_process_group_is_refused() {
        for dangerous in ["0", "-1", "-4242"] {
            let temporary = tempfile::tempdir().unwrap();
            let marker = temporary.path().join(RUNNING_MARKER);
            std::fs::write(&marker, dangerous).unwrap();
            sweep_orphans(temporary.path());
            assert!(!marker.exists(), "{dangerous} must clear the marker");
        }
    }

    #[test]
    fn sweeping_clears_a_marker_for_a_process_that_has_gone() {
        let temporary = tempfile::tempdir().unwrap();
        let marker = temporary.path().join(RUNNING_MARKER);
        // High enough to be nobody, low enough to be a plausible pid.
        std::fs::write(&marker, "999999").unwrap();
        sweep_orphans(temporary.path());
        assert!(!marker.exists(), "the marker must not survive the sweep");
    }

    #[test]
    fn a_marker_that_is_not_a_number_is_discarded_rather_than_acted_on() {
        let temporary = tempfile::tempdir().unwrap();
        let marker = temporary.path().join(RUNNING_MARKER);
        std::fs::write(&marker, "not a process").unwrap();
        sweep_orphans(temporary.path());
        assert!(!marker.exists());
    }
}
