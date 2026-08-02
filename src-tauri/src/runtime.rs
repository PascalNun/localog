//! Small, explicit boundary for user-provided local runtimes.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
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

pub(crate) struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Run a bounded-output child process, checking cancellation without blocking the UI thread.
pub(crate) fn run_process(
    mut command: Command,
    cancellation: &AtomicBool,
    max_output: usize,
) -> Result<ProcessOutput, String> {
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
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or("The local runtime did not expose stdout.")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("The local runtime did not expose stderr.")?;
    let stdout_thread = thread::spawn(move || read_bounded(stdout, max_output));
    let stderr_thread = thread::spawn(move || read_bounded(stderr, max_output));
    loop {
        if cancellation.load(Ordering::Acquire) {
            terminate(&mut child);
            let _ = stdout_thread.join();
            let _ = stderr_thread.join();
            return Err("cancelled".into());
        }
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let status = child.wait().map_err(|error| error.to_string())?;
    let stdout = stdout_thread
        .join()
        .map_err(|_| "The local runtime output could not be read.")?
        .map_err(|error| error.to_string())?;
    let stderr = stderr_thread
        .join()
        .map_err(|_| "The local runtime error output could not be read.")?
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!("runtime exited with status {}", status));
    }
    Ok(ProcessOutput { stdout, stderr })
}

fn read_bounded(reader: impl Read, max: usize) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    reader.take((max + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err(std::io::Error::other(
            "runtime output exceeded the safety limit",
        ));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn terminate(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        let _ = libc::kill(-(child.id() as i32), libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_millis(500);
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
    run_process(command, &token, 16 * 1024)
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

#[allow(dead_code)]
fn _keep_write_import() -> std::io::Result<()> {
    std::io::sink().write_all(&[])
}
