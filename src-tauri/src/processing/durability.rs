//! Putting bytes on disk in a way that survives losing power halfway.
//!
//! Nothing here knows what a transcript is or what a job does. It writes to a
//! temporary name, fsyncs, renames into place, and keeps the file it replaced
//! under a `.previous` name until the new one is known good — so a crash leaves
//! either the old file or the new one, never half of either.
//!
//! It is a leaf: it calls nothing in the rest of the pipeline, and everything
//! else calls it. The one thing it borrows upwards is `ProcessingError`, because
//! that is the vocabulary its callers already speak.
//!
//! Two neighbours deliberately stayed behind. `record_staged` writes a row and
//! never touches a file, and `normalized_cache_matches` is about the media cache
//! rather than about writing — keeping either here would have dragged the job
//! and cache types into a module that otherwise needs neither.

use super::ProcessingError;
use crate::storage::{ProcessingJobRecord, checksum_bytes, managed_relative_path};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) fn verify_streamed_checksum(
    root: &Path,
    relative: &str,
    expected: &str,
    cancellation: &AtomicBool,
) -> Result<(), ProcessingError> {
    let (checksum, _) = streamed_checksum(root, relative, cancellation)?;
    if checksum != expected {
        return Err(ProcessingError::InvalidOutput);
    }
    Ok(())
}
pub(super) fn streamed_checksum(
    root: &Path,
    relative: &str,
    cancellation: &AtomicBool,
) -> Result<(String, u64), ProcessingError> {
    managed_relative_path(Path::new(relative))?;
    let mut file = File::open(root.join(relative))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 256 * 1024];
    let mut byte_count = 0_u64;
    loop {
        if cancellation.load(Ordering::Acquire) {
            return Err(ProcessingError::Cancelled);
        }
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        byte_count += read as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), byte_count))
}
pub(super) fn read_verified(
    root: &Path,
    relative: &str,
    expected: &str,
) -> Result<Vec<u8>, ProcessingError> {
    managed_relative_path(Path::new(relative))?;
    let bytes = fs::read(root.join(relative))?;
    if checksum_bytes(&bytes) != expected {
        return Err(ProcessingError::InvalidOutput);
    }
    Ok(bytes)
}
pub(super) fn write_durable_new(path: &Path, bytes: &[u8]) -> Result<(), ProcessingError> {
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| std::io::Error::other("missing parent"))?,
    )?;
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    sync_directory(path.parent())?;
    Ok(())
}
pub(super) fn replace_working_file(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    _expected_previous: Option<&str>,
) -> Result<(), ProcessingError> {
    managed_relative_path(relative)?;
    let path = root.join(relative);
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| std::io::Error::other("missing parent"))?,
    )?;
    let next = next_path(&path);
    let previous = backup_path(&path);
    let _ = fs::remove_file(&next);
    let _ = fs::remove_file(&previous);
    write_durable_new(&next, bytes)?;
    if path.exists() {
        fs::rename(&path, &previous)?;
    }
    fs::rename(&next, &path)?;
    sync_directory(path.parent())?;
    Ok(())
}
pub(super) fn cleanup_working_backup(root: &Path, relative: &Path) {
    let path = root.join(relative);
    let _ = fs::remove_file(backup_path(&path));
    let _ = fs::remove_file(next_path(&path));
}
pub(super) fn finalize_staged(staged: &Path, final_path: &Path) -> Result<(), ProcessingError> {
    fs::create_dir_all(
        final_path
            .parent()
            .ok_or_else(|| std::io::Error::other("missing parent"))?,
    )?;
    fs::rename(staged, final_path)?;
    sync_directory(final_path.parent())?;
    Ok(())
}
pub(super) fn staged_path(root: &Path, job: &ProcessingJobRecord, extension: &str) -> PathBuf {
    root.join(meeting_root(&job.project_id, &job.meeting_id))
        .join("working/jobs")
        .join(format!("{}.{}.part", job.id, extension))
}
pub(super) fn remove_staged(root: &Path, job: &ProcessingJobRecord) {
    for extension in ["json", "md"] {
        let _ = fs::remove_file(staged_path(root, job, extension));
    }
}
pub(super) fn quarantine_final(root: &Path, job: &ProcessingJobRecord) {
    let final_path = root.join(&job.final_relative_path);
    let recovery = root
        .join(meeting_root(&job.project_id, &job.meeting_id))
        .join("working/recovery");
    if fs::create_dir_all(&recovery).is_ok() {
        let _ = fs::rename(final_path, recovery.join(format!("{}.orphan", job.id)));
    }
}
pub(super) fn meeting_root(project_id: &str, meeting_id: &str) -> PathBuf {
    PathBuf::from("projects")
        .join(project_id)
        .join("meetings")
        .join(meeting_id)
}
pub(super) fn backup_path(path: &Path) -> PathBuf {
    path.with_extension(format!(
        "{}.previous",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("work")
    ))
}
pub(super) fn next_path(path: &Path) -> PathBuf {
    path.with_extension(format!(
        "{}.next",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("work")
    ))
}
#[cfg(unix)]
pub(super) fn sync_directory(directory: Option<&Path>) -> Result<(), ProcessingError> {
    File::open(directory.ok_or_else(|| std::io::Error::other("missing directory"))?)?.sync_all()?;
    Ok(())
}
#[cfg(not(unix))]
pub(super) fn sync_directory(_directory: Option<&Path>) -> Result<(), ProcessingError> {
    Ok(())
}
