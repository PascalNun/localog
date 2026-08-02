use localog_process_supervision_spike::{HeavyJobLane, ProcessError, ProcessSpec, RunningProcess};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

fn helper(arguments: &[&str], working_directory: &Path) -> ProcessSpec {
    ProcessSpec {
        program: PathBuf::from(env!("CARGO_BIN_EXE_synthetic-worker")),
        arguments: arguments.iter().map(OsString::from).collect(),
        working_directory: working_directory.to_path_buf(),
    }
}

#[test]
fn progress_is_typed_throttled_and_logs_remain_bounded() {
    let temporary = tempfile::tempdir().unwrap();
    let spec = helper(
        &[
            "--steps",
            "500",
            "--interval-ms",
            "2",
            "--flood-lines",
            "2000",
            "--malformed-progress",
        ],
        temporary.path(),
    );
    let mut process = RunningProcess::spawn(&spec).unwrap();
    let started = Instant::now();
    let mut events = Vec::new();
    while process.try_wait().unwrap().is_none() {
        events.extend(process.drain_progress());
        thread::sleep(Duration::from_millis(20));
    }
    events.extend(process.drain_progress());

    let elapsed = started.elapsed();
    assert!(elapsed < Duration::from_secs(3));
    assert!(
        events.len() >= 5,
        "received too few progress events: {}",
        events.len()
    );
    let event_limit = (elapsed.as_millis() / 100) as usize + 3;
    assert!(
        events.len() <= event_limit,
        "progress was not throttled: {} events in {elapsed:?}",
        events.len()
    );
    assert!(events.iter().all(|event| event.percent <= 100));
    assert!(events.iter().any(|event| event.percent == 100));
    assert!(process.bounded_log_bytes() <= 32 * 1024);
    assert!(process.stderr_tail().len() <= 80);
    assert!(
        process
            .stdout_tail()
            .iter()
            .any(|line| line.contains("malformed progress"))
    );
}

#[test]
fn cancellation_terminates_the_process_group_and_descendant() {
    let temporary = tempfile::tempdir().unwrap();
    let spec = helper(
        &["--steps", "10000", "--interval-ms", "10", "--spawn-child"],
        temporary.path(),
    );
    let mut process = RunningProcess::spawn(&spec).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let descendant_pid = loop {
        if let Some(line) = process
            .stdout_tail()
            .into_iter()
            .find(|line| line.starts_with("CHILD_PID "))
        {
            break line
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse::<i32>()
                .unwrap();
        }
        assert!(Instant::now() < deadline, "descendant PID was not reported");
        thread::sleep(Duration::from_millis(10));
    };

    let cancellation = process.cancel(Duration::from_millis(500)).unwrap();
    assert!(!cancellation.forced);
    assert!(cancellation.elapsed < Duration::from_secs(1));
    thread::sleep(Duration::from_millis(50));

    // SAFETY: signal 0 only probes whether the recorded synthetic descendant PID still exists.
    let alive = unsafe { libc::kill(descendant_pid, 0) } == 0;
    assert!(
        !alive,
        "synthetic descendant remained alive after group cancellation"
    );
}

#[test]
fn forced_cancellation_escalates_after_the_grace_period() {
    let temporary = tempfile::tempdir().unwrap();
    let spec = helper(
        &["--steps", "10000", "--interval-ms", "10", "--ignore-term"],
        temporary.path(),
    );
    let mut process = RunningProcess::spawn(&spec).unwrap();
    let ready_deadline = Instant::now() + Duration::from_secs(1);
    while !process.stdout_tail().iter().any(|line| line == "READY") {
        assert!(
            Instant::now() < ready_deadline,
            "worker did not become ready"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        process
            .stdout_tail()
            .iter()
            .any(|line| line == "IGNORE_TERM true")
    );
    let cancellation = process.cancel(Duration::from_millis(80)).unwrap();
    assert!(
        cancellation.forced,
        "unexpected cancellation result: {cancellation:?}"
    );
    assert!(cancellation.elapsed < Duration::from_secs(1));
}

#[test]
fn single_heavy_job_lane_rejects_a_concurrent_start() {
    let temporary = tempfile::tempdir().unwrap();
    let spec = helper(
        &["--steps", "10000", "--interval-ms", "10"],
        temporary.path(),
    );
    let mut lane = HeavyJobLane::default();
    lane.start(&spec).unwrap();
    assert!(matches!(lane.start(&spec), Err(ProcessError::Busy)));
    lane.active_mut()
        .unwrap()
        .cancel(Duration::from_millis(500))
        .unwrap();
}

#[test]
fn missing_executable_and_hostile_arguments_are_safe() {
    let temporary = tempfile::tempdir().unwrap();
    let missing = ProcessSpec {
        program: temporary.path().join("does-not-exist"),
        arguments: Vec::new(),
        working_directory: temporary.path().to_path_buf(),
    };
    assert!(matches!(
        RunningProcess::spawn(&missing),
        Err(ProcessError::Io(_))
    ));

    let marker = temporary.path().join("must-not-exist");
    let hostile = format!("$(touch {})", marker.display());
    let spec = ProcessSpec {
        program: PathBuf::from(env!("CARGO_BIN_EXE_synthetic-worker")),
        arguments: vec![OsString::from("--echo-arg"), OsString::from(&hostile)],
        working_directory: temporary.path().to_path_buf(),
    };
    let mut process = RunningProcess::spawn(&spec).unwrap();
    assert!(
        process
            .wait_timeout(Duration::from_secs(3))
            .unwrap()
            .success()
    );
    assert!(!marker.exists());
    assert!(
        process
            .stdout_tail()
            .iter()
            .any(|line| line == &format!("ECHO_ARG {hostile}"))
    );
}
