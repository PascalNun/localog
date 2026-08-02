use localog_process_supervision_spike::{ProcessSpec, RunningProcess};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() {
    let executable = sibling_binary("synthetic-worker");
    let temporary = tempfile::tempdir().expect("temporary working directory");
    let spec = ProcessSpec {
        program: executable,
        arguments: [
            "--steps",
            "10000",
            "--interval-ms",
            "2",
            "--flood-lines",
            "1000",
            "--spawn-child",
        ]
        .into_iter()
        .map(OsString::from)
        .collect(),
        working_directory: temporary.path().to_path_buf(),
    };

    let launch_started = Instant::now();
    let mut process = RunningProcess::spawn(&spec).expect("launch synthetic worker");
    let launch_elapsed = launch_started.elapsed();
    std::thread::sleep(Duration::from_millis(550));
    let events_before_cancel = process.drain_progress().len();
    let cancellation = process
        .cancel(Duration::from_millis(500))
        .expect("cancel synthetic worker");

    println!("launch_ms={:.3}", launch_elapsed.as_secs_f64() * 1000.0);
    println!("progress_events_550ms={events_before_cancel}");
    println!("bounded_log_bytes={}", process.bounded_log_bytes());
    println!(
        "cancel_ms={:.3}",
        cancellation.elapsed.as_secs_f64() * 1000.0
    );
    println!("forced={}", cancellation.forced);
}

fn sibling_binary(name: &str) -> PathBuf {
    let current = std::env::current_exe().expect("current executable");
    let profile_directory = current
        .parent()
        .and_then(|examples| examples.parent())
        .expect("release profile directory");
    profile_directory.join(name)
}
