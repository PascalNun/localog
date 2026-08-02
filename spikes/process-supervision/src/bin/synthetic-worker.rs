use signal_hook::consts::signal::SIGTERM;
use signal_hook::flag;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.iter().any(|argument| argument == "--child-sleep") {
        child_sleep();
        return;
    }

    let steps = value(&arguments, "--steps").unwrap_or(100);
    let interval_ms = value(&arguments, "--interval-ms").unwrap_or(10);
    let flood_lines = value(&arguments, "--flood-lines").unwrap_or(0);
    let ignore_term = arguments.iter().any(|argument| argument == "--ignore-term");
    let spawn_child = arguments.iter().any(|argument| argument == "--spawn-child");
    let malformed = arguments
        .iter()
        .any(|argument| argument == "--malformed-progress");
    if let Some(argument) = text_value(&arguments, "--echo-arg") {
        emit_stdout(format_args!("ECHO_ARG {argument}"));
    }

    let term_received = Arc::new(AtomicBool::new(false));
    if ignore_term {
        // SAFETY: this synthetic mode intentionally ignores SIGTERM so the supervisor's forced
        // escalation path can be verified. It is never used by production code.
        unsafe { libc::signal(SIGTERM, libc::SIG_IGN) };
    } else {
        flag::register(SIGTERM, term_received.clone()).expect("register SIGTERM handler");
    }
    emit_stdout(format_args!("IGNORE_TERM {ignore_term}"));
    emit_stdout(format_args!("READY"));

    let mut descendant = if spawn_child {
        let child = Command::new(env::current_exe().expect("current executable"))
            .arg("--child-sleep")
            .spawn()
            .expect("spawn synthetic descendant");
        emit_stdout(format_args!("CHILD_PID {}", child.id()));
        Some(child)
    } else {
        None
    };

    for line in 0..flood_lines {
        eprintln!("synthetic diagnostic line {line}: {}", "x".repeat(240));
    }

    if malformed {
        emit_stdout(format_args!("PROGRESS invalid malformed progress"));
        emit_stdout(format_args!("PROGRESS 200 impossible progress"));
    }

    for step in 0..=steps {
        if !ignore_term && term_received.load(Ordering::Relaxed) {
            eprintln!("synthetic worker received cancellation");
            if let Some(child) = descendant.as_mut() {
                let _ = child.wait();
            }
            std::process::exit(42);
        }
        let percent = ((step * 100) / steps.max(1)).min(100);
        emit_stdout(format_args!("PROGRESS {percent} synthetic stage {step}"));
        thread::sleep(Duration::from_millis(interval_ms));
    }

    if let Some(child) = descendant.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn child_sleep() {
    let cancelled = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, cancelled.clone()).expect("register child SIGTERM handler");
    while !cancelled.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(20));
    }
}

fn emit_stdout(arguments: std::fmt::Arguments<'_>) {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{arguments}").expect("write synthetic output");
    stdout.flush().expect("flush synthetic output");
}

fn value(arguments: &[String], name: &str) -> Option<u64> {
    text_value(arguments, name)?.parse().ok()
}

fn text_value<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    let index = arguments.iter().position(|argument| argument == name)?;
    arguments.get(index + 1).map(String::as_str)
}
