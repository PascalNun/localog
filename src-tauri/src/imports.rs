//! Durable source import: a compact managed copy with explicit recovery boundaries.
//!
//! An imported file is the person's file, not ours. The original is never touched.
//! What is kept for transcription is a managed copy: already-compressed audio
//! verbatim, and uncompressed audio or video converted to Opus first. A two-hour
//! WAV is about 1.3 GB; the same meeting at 32 kbps is about 29 MB, and keeping
//! thirty of them is the difference between a workspace and a full disk.

use crate::domain::WorkspaceSnapshot;
use crate::media;
use crate::runtime;
use crate::storage::{CommittedSource, ImportJobRecord, StorageError, WorkspaceRepository};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const COPY_BUFFER_BYTES: usize = 256 * 1024;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// Room left on the disk after an import.
///
/// The copy itself is not the only thing that needs space: the database, the
/// prepared audio for transcription, and the system all write too. An import that
/// exactly fills a disk turns a full workspace into a workspace that cannot open,
/// which is a much worse failure than declining the import.
const IMPORT_HEADROOM_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImportOutcome {
    Completed,
    Cancelled,
    DuplicateConfirmation,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImportBoundary {
    BeforeCopy,
    DuringCopy,
    AfterTemporaryFile,
    AfterFinalRename,
    AfterCommit,
}

#[derive(Debug)]
enum ImportRunError {
    Cancelled,
    UnsupportedMedia,
    EmptySource,
    SourceMissing,
    PermissionDenied,
    InsufficientSpace,
    ConversionFailed,
    Io(io::Error),
    Storage(StorageError),
    #[cfg(test)]
    SimulatedTermination,
}

impl From<StorageError> for ImportRunError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<io::Error> for ImportRunError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

trait ImportHooks: Send + Sync {
    fn checkpoint(&self, _boundary: ImportBoundary) -> Result<(), ImportRunError> {
        Ok(())
    }
}

struct ProductionHooks;
impl ImportHooks for ProductionHooks {}

/// Run one persisted import intent. Progress callbacks are bounded to roughly 10 Hz.
pub(crate) fn run_import(
    root: &Path,
    meeting_id: &str,
    cancellation: Arc<AtomicBool>,
    notify: impl Fn(WorkspaceSnapshot),
) -> Result<ImportOutcome, StorageError> {
    run_import_with_hooks(root, meeting_id, &cancellation, &notify, &ProductionHooks)
}

fn run_import_with_hooks(
    root: &Path,
    meeting_id: &str,
    cancellation: &AtomicBool,
    notify: &impl Fn(WorkspaceSnapshot),
    hooks: &dyn ImportHooks,
) -> Result<ImportOutcome, StorageError> {
    let result = execute_import(root, meeting_id, cancellation, notify, hooks);

    match result {
        Ok(outcome) => Ok(outcome),
        Err(ImportRunError::Cancelled) => {
            let repository = WorkspaceRepository::open(root)?;
            let job = repository.import_job_for_meeting(meeting_id)?;
            remove_staged_copy(root, &job);
            repository.mark_import_cancelled(&job.id)?;
            notify(repository.workspace_snapshot()?);
            Ok(ImportOutcome::Cancelled)
        }
        #[cfg(test)]
        Err(ImportRunError::SimulatedTermination) => {
            Err(StorageError::InvalidData("simulated import termination"))
        }
        Err(error) => {
            let repository = WorkspaceRepository::open(root)?;
            let job = repository.import_job_for_meeting(meeting_id)?;
            remove_staged_copy(root, &job);
            repository.mark_import_failed(&job.id, error_code(&error))?;
            notify(repository.workspace_snapshot()?);
            Ok(ImportOutcome::Failed)
        }
    }
}

pub(crate) fn cancel_unstarted_import(
    root: &Path,
    meeting_id: &str,
) -> Result<WorkspaceSnapshot, StorageError> {
    let repository = WorkspaceRepository::open(root)?;
    let job = repository.import_job_for_meeting(meeting_id)?;
    remove_staged_copy(root, &job);
    repository.mark_import_cancelled(&job.id)?;
    repository.workspace_snapshot()
}

fn execute_import(
    root: &Path,
    meeting_id: &str,
    cancellation: &AtomicBool,
    notify: &impl Fn(WorkspaceSnapshot),
    hooks: &dyn ImportHooks,
) -> Result<ImportOutcome, ImportRunError> {
    let mut repository = WorkspaceRepository::open(root)?;
    let mut job = repository.import_job_for_meeting(meeting_id)?;
    let stored = stored_media(&job.original_name)?;
    let paths = import_paths(root, &job, stored.extension)?;

    // Duplicate confirmation resumes from the already durable temporary copy.
    if job.stage == "temporary_complete" && job.duplicate_allowed {
        let source = committed_source_from_job(&job)?;
        verify_file(&paths.temporary, &source)?;
        return finalize_import(&mut repository, &job, &paths, source, notify, hooks);
    }

    let source_metadata = fs::metadata(&job.source_path).map_err(classify_source_error)?;
    if !source_metadata.is_file() {
        return Err(ImportRunError::SourceMissing);
    }
    let total_bytes = source_metadata.len();
    if total_bytes == 0 {
        return Err(ImportRunError::EmptySource);
    }
    // A conversion needs a length to report progress against and to estimate what
    // it will cost. The WAV header already knows it; anything else is probed once.
    let duration_ms = match stored.form {
        StoredForm::Opus => source_duration_ms(&job.source_path, cancellation),
        StoredForm::Copy => None,
    };
    let expected_stored_bytes = match (stored.form, duration_ms) {
        (StoredForm::Opus, Some(milliseconds)) => opus_size_estimate(milliseconds),
        _ => total_bytes,
    };
    guard_free_space(root, expected_stored_bytes)?;

    repository.mark_import_running(
        &job.id,
        expected_stored_bytes,
        stored.media_type,
        &paths.final_relative,
    )?;
    notify(repository.workspace_snapshot()?);
    hooks.checkpoint(ImportBoundary::BeforeCopy)?;

    fs::create_dir_all(
        paths
            .temporary
            .parent()
            .ok_or_else(|| io::Error::other("temporary import path has no parent"))?,
    )
    .map_err(classify_destination_error)?;
    fs::create_dir_all(
        paths
            .final_path
            .parent()
            .ok_or_else(|| io::Error::other("final import path has no parent"))?,
    )
    .map_err(classify_destination_error)?;
    remove_temporary_files(&paths);

    let (stored_checksum, stored_bytes, original_checksum) = match stored.form {
        StoredForm::Copy => {
            let checksum = copy_verbatim(
                &mut repository,
                &job,
                &paths,
                total_bytes,
                cancellation,
                notify,
                hooks,
            )?;
            (checksum.clone(), total_bytes, checksum)
        }
        StoredForm::Opus => {
            let ffmpeg = runtime::discover_executable(runtime::FFMPEG_NAMES)
                .ok_or(ImportRunError::ConversionFailed)?;
            // The checksum of the chosen file, not of the conversion. Ogg gives
            // every encode a fresh serial number, so the stored bytes of two
            // imports of the same WAV never match; duplicate detection needs the
            // bytes somebody actually chose.
            let (original_checksum, _) = hash_file(&job.source_path, cancellation, true)?;
            if cancellation.load(Ordering::Acquire) {
                return Err(ImportRunError::Cancelled);
            }
            let encode_error = std::cell::RefCell::new(None);
            let mut last_progress = Instant::now();
            let mut last_percent = u8::MAX;
            let encoded = media::encode_to_opus_reporting(
                &ffmpeg,
                &job.source_path,
                &paths.temporary,
                media::OPUS_KBPS,
                duration_ms,
                cancellation,
                |percent| {
                    if percent == last_percent {
                        return;
                    }
                    last_percent = percent;
                    if last_progress.elapsed() < PROGRESS_INTERVAL && percent < 100 {
                        return;
                    }
                    let progress = expected_stored_bytes.saturating_mul(percent as u64) / 100;
                    if let Err(error) = repository.update_import_progress(&job.id, progress) {
                        *encode_error.borrow_mut() = Some(error);
                        return;
                    }
                    match repository.workspace_snapshot() {
                        Ok(snapshot) => notify(snapshot),
                        Err(error) => *encode_error.borrow_mut() = Some(error),
                    }
                    last_progress = Instant::now();
                },
            );
            if let Some(error) = encode_error.into_inner() {
                return Err(error.into());
            }
            if let Err(message) = encoded {
                if cancellation.load(Ordering::Acquire) {
                    return Err(ImportRunError::Cancelled);
                }
                if message.contains("No space left on device") {
                    return Err(ImportRunError::InsufficientSpace);
                }
                return Err(ImportRunError::ConversionFailed);
            }
            let (stored_checksum, stored_bytes) = hash_file(&paths.temporary, cancellation, false)?;
            if stored_bytes == 0 {
                return Err(ImportRunError::EmptySource);
            }
            (stored_checksum, stored_bytes, original_checksum)
        }
    };

    let source = CommittedSource {
        checksum: stored_checksum,
        byte_count: stored_bytes,
        media_type: stored.media_type.to_string(),
        final_relative_path: paths.final_relative.clone(),
    };
    repository.record_import_validation(
        &job.id,
        &source.checksum,
        source.byte_count,
        &source.media_type,
        &source.final_relative_path,
        &original_checksum,
    )?;
    notify(repository.workspace_snapshot()?);
    hooks.checkpoint(ImportBoundary::AfterTemporaryFile)?;

    job = repository.import_job_by_id(&job.id)?;
    if repository.probable_duplicate_exists(&job.recording_id, &original_checksum)?
        && !job.duplicate_allowed
    {
        repository.pause_for_duplicate(&job.id)?;
        notify(repository.workspace_snapshot()?);
        return Ok(ImportOutcome::DuplicateConfirmation);
    }

    finalize_import(&mut repository, &job, &paths, source, notify, hooks)
}

/// Copy an already-compressed source byte for byte.
///
/// Returns the checksum of what was written. The source length is checked at the
/// end because a file edited while it is being read must not be committed as
/// something it is not.
fn copy_verbatim(
    repository: &mut WorkspaceRepository,
    job: &ImportJobRecord,
    paths: &ImportPaths,
    total_bytes: u64,
    cancellation: &AtomicBool,
    notify: &impl Fn(WorkspaceSnapshot),
    hooks: &dyn ImportHooks,
) -> Result<String, ImportRunError> {
    let mut input = File::open(&job.source_path).map_err(classify_source_error)?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&paths.temporary)
        .map_err(classify_destination_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
    let mut copied_bytes = 0_u64;
    let mut last_progress = Instant::now();
    let mut first_chunk = true;

    loop {
        if cancellation.load(Ordering::Acquire) {
            return Err(ImportRunError::Cancelled);
        }
        let read = input.read(&mut buffer).map_err(classify_source_error)?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .map_err(classify_destination_error)?;
        hasher.update(&buffer[..read]);
        copied_bytes += read as u64;

        if first_chunk {
            hooks.checkpoint(ImportBoundary::DuringCopy)?;
            first_chunk = false;
        }
        if last_progress.elapsed() >= PROGRESS_INTERVAL || copied_bytes == total_bytes {
            repository.update_import_progress(&job.id, copied_bytes)?;
            notify(repository.workspace_snapshot()?);
            last_progress = Instant::now();
        }
    }

    output.flush().map_err(classify_destination_error)?;
    output.sync_all().map_err(classify_destination_error)?;
    if copied_bytes != total_bytes {
        return Err(ImportRunError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "source changed while it was being copied",
        )));
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn finalize_import(
    repository: &mut WorkspaceRepository,
    job: &ImportJobRecord,
    paths: &ImportPaths,
    source: CommittedSource,
    notify: &impl Fn(WorkspaceSnapshot),
    hooks: &dyn ImportHooks,
) -> Result<ImportOutcome, ImportRunError> {
    repository.mark_import_finalizing(&job.id)?;
    notify(repository.workspace_snapshot()?);

    if paths.final_path.exists() {
        verify_file(&paths.final_path, &source)?;
        if paths.temporary.exists() {
            let _ = fs::remove_file(&paths.temporary);
        }
    } else {
        fs::rename(&paths.temporary, &paths.final_path).map_err(classify_destination_error)?;
        sync_directory(paths.final_path.parent())?;
    }
    hooks.checkpoint(ImportBoundary::AfterFinalRename)?;

    repository.commit_import(job, &source)?;
    notify(repository.workspace_snapshot()?);
    hooks.checkpoint(ImportBoundary::AfterCommit)?;
    Ok(ImportOutcome::Completed)
}

/// Reconcile only the import states for which this implementation has explicit authority rules.
pub(crate) fn reconcile_imports(root: &Path) -> Result<WorkspaceSnapshot, StorageError> {
    let mut repository = WorkspaceRepository::open(root)?;
    repository.mark_abandoned_imports_interrupted()?;
    let jobs = repository.unfinished_import_jobs()?;

    for job in jobs {
        let Ok(stored) = stored_media(&job.original_name) else {
            repository.mark_import_failed(&job.id, "unsupported_media")?;
            continue;
        };
        // A job recorded by this version names the file it was storing. An older
        // one may have been interrupted mid-way through keeping a WAV verbatim;
        // its recorded extension is still the truthful one, and committing that
        // copy is better than leaving a complete file unreachable.
        let extension = job
            .final_relative_path
            .as_deref()
            .and_then(Path::extension)
            .and_then(|value| value.to_str())
            .filter(|value| {
                !value.is_empty()
                    && value
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric())
            })
            .unwrap_or(stored.extension);
        let Ok(paths) = import_paths(root, &job, extension) else {
            repository.mark_import_failed(&job.id, "invalid_managed_path")?;
            continue;
        };

        if paths.final_path.exists() {
            match committed_source_from_job(&job)
                .and_then(|source| verify_file(&paths.final_path, &source).map(|()| source))
            {
                Ok(source) => {
                    repository.commit_import(&job, &source)?;
                    if paths.temporary.exists() {
                        let _ = fs::remove_file(&paths.temporary);
                    }
                    continue;
                }
                Err(_) => {
                    quarantine_final_copy(&paths, &job);
                    repository.mark_import_failed(&job.id, "recovery_required")?;
                }
            }
        }

        // A duplicate-confirmation copy is complete and intentionally retained for the choice.
        if job.stage != "duplicate_confirmation" {
            remove_temporary_files(&paths);
        }
    }

    repository.workspace_snapshot()
}

fn committed_source_from_job(job: &ImportJobRecord) -> Result<CommittedSource, ImportRunError> {
    Ok(CommittedSource {
        checksum: job
            .checksum
            .clone()
            .ok_or_else(|| io::Error::other("validated import checksum is missing"))?,
        byte_count: job
            .byte_count
            .ok_or_else(|| io::Error::other("validated import byte count is missing"))?,
        media_type: job
            .media_type
            .clone()
            .ok_or_else(|| io::Error::other("validated import media type is missing"))?,
        final_relative_path: job
            .final_relative_path
            .clone()
            .ok_or_else(|| io::Error::other("validated final path is missing"))?,
    })
}

struct ImportPaths {
    temporary: PathBuf,
    final_path: PathBuf,
    final_relative: PathBuf,
    recovery_directory: PathBuf,
}

fn import_paths(
    root: &Path,
    job: &ImportJobRecord,
    extension: &str,
) -> Result<ImportPaths, ImportRunError> {
    for component in [&job.project_id, &job.meeting_id, &job.recording_id] {
        if !safe_identifier(component) {
            return Err(ImportRunError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "managed identifier is invalid",
            )));
        }
    }
    let meeting_relative = PathBuf::from("projects")
        .join(&job.project_id)
        .join("meetings")
        .join(&job.meeting_id);
    let final_relative = meeting_relative
        .join("recordings")
        .join(format!("{}.{}", job.recording_id, extension));
    // Only generated identifiers influence managed placement; the original filename contributes
    // a validated extension and can never escape the application-data root.
    Ok(ImportPaths {
        temporary: root.join(
            meeting_relative
                .join("working/imports")
                .join(format!("{}.part", job.recording_id)),
        ),
        final_path: root.join(&final_relative),
        final_relative,
        recovery_directory: root.join(meeting_relative.join("working/recovery")),
    })
}

/// How an imported file is kept.
///
/// Already-compressed audio is kept as it is: re-encoding an MP3 would lose
/// quality and save nothing worth having. Anything uncompressed — PCM audio, a
/// lossless codec, a video container — is converted to Opus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StoredForm {
    Copy,
    Opus,
}

struct StoredMedia {
    form: StoredForm,
    media_type: &'static str,
    extension: &'static str,
}

fn stored_media(original_name: &str) -> Result<StoredMedia, ImportRunError> {
    let extension = Path::new(original_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or(ImportRunError::UnsupportedMedia)?;
    let stored = match extension.as_str() {
        "wav" | "flac" | "mp4" | "mov" | "mkv" | "webm" => StoredMedia {
            form: StoredForm::Opus,
            media_type: "audio/opus",
            extension: "opus",
        },
        "mp3" => StoredMedia {
            form: StoredForm::Copy,
            media_type: "audio/mpeg",
            extension: "mp3",
        },
        "m4a" => StoredMedia {
            form: StoredForm::Copy,
            media_type: "audio/mp4",
            extension: "m4a",
        },
        "aac" => StoredMedia {
            form: StoredForm::Copy,
            media_type: "audio/aac",
            extension: "aac",
        },
        "ogg" => StoredMedia {
            form: StoredForm::Copy,
            media_type: "audio/ogg",
            extension: "ogg",
        },
        "opus" => StoredMedia {
            form: StoredForm::Copy,
            media_type: "audio/opus",
            extension: "opus",
        },
        _ => return Err(ImportRunError::UnsupportedMedia),
    };
    Ok(stored)
}

/// What a conversion is expected to occupy, in bytes.
///
/// 32 kbps is 4,000 bytes a second. This is an estimate used for the progress bar
/// and the free-space guard; the committed byte count is measured afterwards.
fn opus_size_estimate(duration_ms: u64) -> u64 {
    duration_ms.saturating_mul(media::OPUS_KBPS as u64) / 8
}

/// Refuse an import that would leave the disk with no room to finish it.
///
/// When free space cannot be read the import proceeds: a guard that fails closed
/// would turn an unsupported platform into an application that cannot import at
/// all, and the write itself still reports a full disk.
fn guard_free_space(root: &Path, needed: u64) -> Result<(), ImportRunError> {
    let Some(available) = crate::models::available_bytes(root) else {
        return Ok(());
    };
    if available < needed.saturating_add(IMPORT_HEADROOM_BYTES) {
        return Err(ImportRunError::InsufficientSpace);
    }
    Ok(())
}

/// A source's length, for progress and the size estimate.
///
/// A PCM WAV says so in its header without running anything. Everything else is
/// asked of FFprobe once; if that is unavailable the estimate falls back to the
/// source size, which is conservative rather than wrong.
fn source_duration_ms(path: &Path, cancellation: &AtomicBool) -> Option<u64> {
    if let Some(milliseconds) = media::wav_duration_ms(path) {
        return Some(milliseconds);
    }
    let ffprobe = runtime::discover_executable(runtime::FFPROBE_NAMES)?;
    let probe = media::probe(&ffprobe, path, cancellation).ok()?;
    let seconds = probe.format?.duration?.parse::<f64>().ok()?;
    (seconds.is_finite() && seconds > 0.0).then_some((seconds * 1000.0) as u64)
}

fn hash_file(
    path: &Path,
    cancellation: &AtomicBool,
    source: bool,
) -> Result<(String, u64), ImportRunError> {
    let classify = if source {
        classify_source_error as fn(io::Error) -> ImportRunError
    } else {
        classify_destination_error
    };
    let mut file = File::open(path).map_err(classify)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
    let mut bytes = 0u64;
    loop {
        if cancellation.load(Ordering::Acquire) {
            return Err(ImportRunError::Cancelled);
        }
        let read = file.read(&mut buffer).map_err(classify)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), bytes))
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn verify_file(path: &Path, expected: &CommittedSource) -> Result<(), ImportRunError> {
    let metadata = fs::metadata(path).map_err(classify_destination_error)?;
    if metadata.len() != expected.byte_count {
        return Err(ImportRunError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "managed copy size does not match validated metadata",
        )));
    }
    let mut file = File::open(path).map_err(classify_destination_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer).map_err(classify_destination_error)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if format!("{:x}", hasher.finalize()) != expected.checksum {
        return Err(ImportRunError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "managed copy checksum does not match validated metadata",
        )));
    }
    Ok(())
}

fn remove_staged_copy(root: &Path, job: &ImportJobRecord) {
    if let Ok(stored) = stored_media(&job.original_name)
        && let Ok(paths) = import_paths(root, job, stored.extension)
    {
        remove_temporary_files(&paths);
    }
}

/// Take a half-written copy with it, including the encoder's own partial file.
///
/// `encode_to_opus` writes `<name>.opus.part` before renaming it into place. A
/// process killed mid-encode leaves that file behind, and a later retry that only
/// looked for `<name>.part` would keep it forever.
fn remove_temporary_files(paths: &ImportPaths) {
    let _ = fs::remove_file(&paths.temporary);
    let _ = fs::remove_file(paths.temporary.with_extension("opus.part"));
}

fn quarantine_final_copy(paths: &ImportPaths, job: &ImportJobRecord) {
    if fs::create_dir_all(&paths.recovery_directory).is_ok() {
        let destination = paths
            .recovery_directory
            .join(format!("{}.orphan", job.recording_id));
        let _ = fs::rename(&paths.final_path, destination);
    }
}

#[cfg(unix)]
fn sync_directory(directory: Option<&Path>) -> Result<(), ImportRunError> {
    let directory = directory.ok_or_else(|| io::Error::other("final path has no parent"))?;
    File::open(directory)
        .and_then(|file| file.sync_all())
        .map_err(classify_destination_error)
}

#[cfg(not(unix))]
fn sync_directory(_directory: Option<&Path>) -> Result<(), ImportRunError> {
    // Windows durability and replacement semantics remain behind this focused adapter boundary.
    Ok(())
}

fn classify_source_error(error: io::Error) -> ImportRunError {
    match error.kind() {
        io::ErrorKind::NotFound => ImportRunError::SourceMissing,
        io::ErrorKind::PermissionDenied => ImportRunError::PermissionDenied,
        _ => ImportRunError::Io(error),
    }
}

fn classify_destination_error(error: io::Error) -> ImportRunError {
    match error.kind() {
        io::ErrorKind::PermissionDenied => ImportRunError::PermissionDenied,
        _ if error.raw_os_error() == Some(28) => ImportRunError::InsufficientSpace,
        _ => ImportRunError::Io(error),
    }
}

fn error_code(error: &ImportRunError) -> &'static str {
    match error {
        ImportRunError::UnsupportedMedia => "unsupported_media",
        ImportRunError::EmptySource => "empty_source",
        ImportRunError::SourceMissing => "source_missing",
        ImportRunError::PermissionDenied => "permission_denied",
        ImportRunError::InsufficientSpace => "insufficient_space",
        ImportRunError::ConversionFailed => "conversion_failed",
        ImportRunError::Cancelled => "cancelled",
        ImportRunError::Io(error) if error.raw_os_error() == Some(28) => "insufficient_space",
        ImportRunError::Io(_) => "import_failed",
        ImportRunError::Storage(error) => match error {
            StorageError::Io(io_error) if io_error.raw_os_error() == Some(28) => {
                "insufficient_space"
            }
            StorageError::Io(io_error) if io_error.kind() == io::ErrorKind::PermissionDenied => {
                "permission_denied"
            }
            _ => "import_failed",
        },
        #[cfg(test)]
        ImportRunError::SimulatedTermination => "interrupted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{MeetingLifecycle, NewMeetingInput, NewProjectInput};
    use tempfile::{TempDir, tempdir};

    struct ImportFixture {
        _temporary: TempDir,
        root: PathBuf,
        source: PathBuf,
        source_bytes: Vec<u8>,
        meeting_id: String,
    }

    impl ImportFixture {
        fn new(source_bytes: Vec<u8>) -> Self {
            Self::with_name("synthetic-review.mp3", source_bytes)
        }

        /// The stored form follows the name, so a fixture that wants the copy path
        /// uses a compressed name and one that wants conversion says `.wav`.
        fn with_name(source_name: &str, source_bytes: Vec<u8>) -> Self {
            let temporary = tempdir().unwrap();
            let root = temporary.path().join("managed");
            let source = temporary.path().join(source_name);
            fs::write(&source, &source_bytes).unwrap();
            let mut repository = WorkspaceRepository::open(&root).unwrap();
            let project = repository
                .create_project(NewProjectInput {
                    names: Vec::new(),
                    name: "Synthetic public project".to_string(),
                    description: "Crash-boundary fixture".to_string(),
                    default_language: "English".to_string(),
                })
                .unwrap();
            let meeting = repository
                .create_meeting(NewMeetingInput {
                    project_id: project.id,
                    title: "Synthetic review".to_string(),
                    occurred_at: "2026-08-02".to_string(),
                    language: "English".to_string(),
                    source_name: source_name.to_string(),
                    source_path: Some(source.to_string_lossy().into_owned()),
                    style_id: "style-formal".to_string(),
                })
                .unwrap();
            Self {
                _temporary: temporary,
                root,
                source,
                source_bytes,
                meeting_id: meeting.id,
            }
        }

        fn snapshot(&self) -> WorkspaceSnapshot {
            WorkspaceRepository::open(&self.root)
                .unwrap()
                .workspace_snapshot()
                .unwrap()
        }
    }

    struct TerminateAt(ImportBoundary);

    impl ImportHooks for TerminateAt {
        fn checkpoint(&self, boundary: ImportBoundary) -> Result<(), ImportRunError> {
            if boundary == self.0 {
                Err(ImportRunError::SimulatedTermination)
            } else {
                Ok(())
            }
        }
    }

    struct CancelAfterFirstChunk(Arc<AtomicBool>);

    impl ImportHooks for CancelAfterFirstChunk {
        fn checkpoint(&self, boundary: ImportBoundary) -> Result<(), ImportRunError> {
            if boundary == ImportBoundary::DuringCopy {
                self.0.store(true, Ordering::Release);
            }
            Ok(())
        }
    }

    struct FailAt(ImportBoundary, FailureKind);

    enum FailureKind {
        Permission,
        NoSpace,
    }

    impl ImportHooks for FailAt {
        fn checkpoint(&self, boundary: ImportBoundary) -> Result<(), ImportRunError> {
            if boundary != self.0 {
                return Ok(());
            }
            Err(match self.1 {
                FailureKind::Permission => ImportRunError::PermissionDenied,
                FailureKind::NoSpace => ImportRunError::InsufficientSpace,
            })
        }
    }

    #[test]
    fn committed_import_preserves_the_original_and_records_truthful_metadata() {
        let fixture = ImportFixture::new(b"synthetic audio content".repeat(20_000));
        let outcome = run_import_with_hooks(
            &fixture.root,
            &fixture.meeting_id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();
        assert_eq!(outcome, ImportOutcome::Completed);

        let snapshot = fixture.snapshot();
        assert_eq!(
            snapshot.meetings[0].lifecycle,
            MeetingLifecycle::SourceReady
        );
        assert_eq!(
            snapshot.meetings[0].source_byte_count,
            Some(fixture.source_bytes.len() as u64)
        );
        assert_eq!(
            snapshot.meetings[0].source_media_type.as_deref(),
            Some("audio/mpeg")
        );
        assert_eq!(snapshot.jobs[0].state, crate::domain::JobState::Completed);
        assert_eq!(fs::read(&fixture.source).unwrap(), fixture.source_bytes);

        let repository = WorkspaceRepository::open(&fixture.root).unwrap();
        let job = repository
            .import_job_for_meeting(&fixture.meeting_id)
            .unwrap();
        let final_path = fixture.root.join(job.final_relative_path.unwrap());
        assert_eq!(fs::read(final_path).unwrap(), fixture.source_bytes);
    }

    #[test]
    fn reconciliation_never_presents_partial_files_as_complete() {
        for boundary in [
            ImportBoundary::BeforeCopy,
            ImportBoundary::DuringCopy,
            ImportBoundary::AfterTemporaryFile,
            ImportBoundary::AfterFinalRename,
            ImportBoundary::AfterCommit,
        ] {
            let fixture = ImportFixture::new(vec![42; COPY_BUFFER_BYTES * 2 + 17]);
            let result = run_import_with_hooks(
                &fixture.root,
                &fixture.meeting_id,
                &AtomicBool::new(false),
                &|_| {},
                &TerminateAt(boundary),
            );
            assert!(result.is_err());

            let snapshot = reconcile_imports(&fixture.root).unwrap();
            let should_be_committed = matches!(
                boundary,
                ImportBoundary::AfterFinalRename | ImportBoundary::AfterCommit
            );
            assert_eq!(
                snapshot.meetings[0].lifecycle == MeetingLifecycle::SourceReady,
                should_be_committed,
                "unexpected lifecycle after {boundary:?}"
            );
            assert_eq!(
                snapshot.jobs[0].state == crate::domain::JobState::Completed,
                should_be_committed,
                "unexpected job state after {boundary:?}"
            );
            if !should_be_committed {
                assert_eq!(snapshot.jobs[0].state, crate::domain::JobState::Interrupted);
            }
            assert_eq!(fs::read(&fixture.source).unwrap(), fixture.source_bytes);
        }
    }

    #[test]
    fn cancellation_during_copy_retains_the_draft_meeting_and_removes_staging() {
        let fixture = ImportFixture::new(vec![7; COPY_BUFFER_BYTES * 3]);
        let cancellation = Arc::new(AtomicBool::new(false));
        let outcome = run_import_with_hooks(
            &fixture.root,
            &fixture.meeting_id,
            &cancellation,
            &|_| {},
            &CancelAfterFirstChunk(cancellation.clone()),
        )
        .unwrap();
        assert_eq!(outcome, ImportOutcome::Cancelled);
        let snapshot = fixture.snapshot();
        assert_eq!(snapshot.meetings[0].lifecycle, MeetingLifecycle::Draft);
        assert_eq!(snapshot.jobs[0].state, crate::domain::JobState::Cancelled);
        assert_eq!(fs::read(&fixture.source).unwrap(), fixture.source_bytes);
    }

    #[test]
    fn permission_and_disk_failures_are_bounded_and_retryable() {
        for (failure, expected_code) in [
            (FailureKind::Permission, "permission_denied"),
            (FailureKind::NoSpace, "insufficient_space"),
        ] {
            let fixture = ImportFixture::new(vec![5; COPY_BUFFER_BYTES * 2]);
            let outcome = run_import_with_hooks(
                &fixture.root,
                &fixture.meeting_id,
                &AtomicBool::new(false),
                &|_| {},
                &FailAt(ImportBoundary::DuringCopy, failure),
            )
            .unwrap();
            assert_eq!(outcome, ImportOutcome::Failed);
            let snapshot = fixture.snapshot();
            assert_eq!(snapshot.meetings[0].lifecycle, MeetingLifecycle::Draft);
            assert_eq!(
                snapshot.jobs[0]
                    .error
                    .as_ref()
                    .map(|error| error.code.as_str()),
                Some(expected_code)
            );
            assert_eq!(fs::read(&fixture.source).unwrap(), fixture.source_bytes);
        }
    }

    #[test]
    fn empty_media_never_becomes_source_ready() {
        let fixture = ImportFixture::new(Vec::new());
        let outcome = run_import_with_hooks(
            &fixture.root,
            &fixture.meeting_id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();
        assert_eq!(outcome, ImportOutcome::Failed);
        let snapshot = fixture.snapshot();
        assert_eq!(snapshot.meetings[0].lifecycle, MeetingLifecycle::Draft);
        assert_eq!(
            snapshot.jobs[0]
                .error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("empty_source")
        );
    }

    #[test]
    fn duplicate_content_requires_an_explicit_second_copy_choice() {
        let source_bytes = b"probable duplicate fixture".repeat(30_000);
        let fixture = ImportFixture::new(source_bytes.clone());
        run_import_with_hooks(
            &fixture.root,
            &fixture.meeting_id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();

        let second_source = fixture.source.with_file_name("synthetic-review-copy.mp3");
        fs::write(&second_source, &source_bytes).unwrap();
        let second_meeting = {
            let mut repository = WorkspaceRepository::open(&fixture.root).unwrap();
            let project_id = repository.workspace_snapshot().unwrap().projects[0]
                .id
                .clone();
            repository
                .create_meeting(NewMeetingInput {
                    project_id,
                    title: "Synthetic duplicate review".to_string(),
                    occurred_at: "2026-08-03".to_string(),
                    language: "English".to_string(),
                    source_name: "synthetic-review-copy.mp3".to_string(),
                    source_path: Some(second_source.to_string_lossy().into_owned()),
                    style_id: "style-formal".to_string(),
                })
                .unwrap()
        };

        let outcome = run_import_with_hooks(
            &fixture.root,
            &second_meeting.id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();
        assert_eq!(outcome, ImportOutcome::DuplicateConfirmation);
        let paused = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        let duplicate_job = paused
            .jobs
            .iter()
            .find(|job| job.meeting_id == second_meeting.id)
            .unwrap();
        assert!(duplicate_job.requires_duplicate_confirmation);
        assert_eq!(
            paused
                .meetings
                .iter()
                .find(|meeting| meeting.id == second_meeting.id)
                .unwrap()
                .lifecycle,
            MeetingLifecycle::Draft
        );

        WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .retry_import(&second_meeting.id, true)
            .unwrap();
        let outcome = run_import_with_hooks(
            &fixture.root,
            &second_meeting.id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();
        assert_eq!(outcome, ImportOutcome::Completed);
        assert_eq!(
            fixture
                .snapshot()
                .meetings
                .iter()
                .filter(|meeting| meeting.lifecycle == MeetingLifecycle::SourceReady)
                .count(),
            2
        );
    }

    /// Which formats are stored as they arrived and which are converted.
    ///
    /// The distinction is not cosmetic: re-encoding an MP3 spends quality to save
    /// almost nothing, and keeping a two-hour WAV costs more than a gigabyte.
    #[test]
    fn uncompressed_sources_become_opus_and_compressed_ones_are_kept() {
        for name in [
            "meeting.wav",
            "meeting.flac",
            "meeting.mp4",
            "meeting.mov",
            "meeting.mkv",
            "meeting.webm",
            "MEETING.WAV",
        ] {
            let stored = stored_media(name).unwrap();
            assert_eq!(stored.form, StoredForm::Opus, "{name}");
            assert_eq!(stored.media_type, "audio/opus", "{name}");
            assert_eq!(stored.extension, "opus", "{name}");
        }
        for (name, mime, extension) in [
            ("meeting.mp3", "audio/mpeg", "mp3"),
            ("meeting.m4a", "audio/mp4", "m4a"),
            ("meeting.aac", "audio/aac", "aac"),
            ("meeting.ogg", "audio/ogg", "ogg"),
            ("meeting.opus", "audio/opus", "opus"),
        ] {
            let stored = stored_media(name).unwrap();
            assert_eq!(stored.form, StoredForm::Copy, "{name}");
            assert_eq!(stored.media_type, mime, "{name}");
            assert_eq!(stored.extension, extension, "{name}");
        }
        assert!(stored_media("meeting.txt").is_err());
        assert!(stored_media("meeting").is_err());
    }

    /// The estimate is the bitrate and nothing else: 32 kbps is four kilobytes a
    /// second, and an hour is about 14 MB.
    #[test]
    fn the_size_estimate_is_the_bitrate() {
        assert_eq!(opus_size_estimate(1_000), 4_000);
        assert_eq!(opus_size_estimate(60 * 60 * 1_000), 14_400_000);
        assert_eq!(opus_size_estimate(0), 0);
    }

    /// Real PCM, because a conversion cannot be demonstrated on a byte pattern.
    fn pcm_wav(milliseconds: u64) -> Vec<u8> {
        let samples = (16_000 * milliseconds / 1_000) as usize;
        let mut audio = Vec::with_capacity(samples * 2);
        for sample in 0..samples {
            let value = ((sample as f32 * 0.05).sin() * 8_000.0) as i16;
            audio.extend_from_slice(&value.to_le_bytes());
        }
        let mut wav = Vec::with_capacity(44 + audio.len());
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + audio.len() as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&16_000u32.to_le_bytes());
        wav.extend_from_slice(&(16_000u32 * 2).to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(audio.len() as u32).to_le_bytes());
        wav.extend_from_slice(&audio);
        wav
    }

    /// A WAV import keeps the original, stores a much smaller Opus, and describes
    /// the file it actually kept rather than the one it read.
    #[test]
    #[ignore = "requires ffmpeg"]
    fn a_wav_import_is_stored_as_opus() {
        if runtime::discover_executable(runtime::FFMPEG_NAMES).is_none() {
            panic!("ffmpeg is not available to this test");
        }
        let fixture = ImportFixture::with_name("synthetic-review.wav", pcm_wav(10_000));
        let outcome = run_import_with_hooks(
            &fixture.root,
            &fixture.meeting_id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();
        assert_eq!(outcome, ImportOutcome::Completed);

        let snapshot = fixture.snapshot();
        assert_eq!(
            snapshot.meetings[0].lifecycle,
            MeetingLifecycle::SourceReady
        );
        assert_eq!(
            snapshot.meetings[0].source_media_type.as_deref(),
            Some("audio/opus")
        );
        let repository = WorkspaceRepository::open(&fixture.root).unwrap();
        let job = repository
            .import_job_for_meeting(&fixture.meeting_id)
            .unwrap();
        let relative = job.final_relative_path.clone().unwrap();
        assert_eq!(
            relative.extension().and_then(|value| value.to_str()),
            Some("opus")
        );
        let stored = fs::read(fixture.root.join(&relative)).unwrap();
        assert_eq!(&stored[..4], b"OggS");
        assert_eq!(
            snapshot.meetings[0].source_byte_count,
            Some(stored.len() as u64)
        );
        assert!(stored.len() < fixture.source_bytes.len() / 5);
        // The person's file is not touched, and its checksum is recorded separately
        // from the checksum of what was stored.
        assert_eq!(fs::read(&fixture.source).unwrap(), fixture.source_bytes);
        assert!(job.original_checksum.is_some());
        assert_ne!(job.original_checksum.as_deref(), job.checksum.as_deref());
    }

    /// Two imports of the same uncompressed file are the same meeting, even though
    /// Ogg gives every encode a fresh serial number.
    #[test]
    #[ignore = "requires ffmpeg"]
    fn the_same_wav_is_recognised_as_a_duplicate_after_conversion() {
        let source_bytes = pcm_wav(5_000);
        let fixture = ImportFixture::with_name("synthetic-review.wav", source_bytes.clone());
        run_import_with_hooks(
            &fixture.root,
            &fixture.meeting_id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();

        let second_source = fixture.source.with_file_name("synthetic-review-again.wav");
        fs::write(&second_source, &source_bytes).unwrap();
        let second_meeting = {
            let mut repository = WorkspaceRepository::open(&fixture.root).unwrap();
            let project_id = repository.workspace_snapshot().unwrap().projects[0]
                .id
                .clone();
            repository
                .create_meeting(NewMeetingInput {
                    project_id,
                    title: "Synthetic duplicate review".to_string(),
                    occurred_at: "2026-08-03".to_string(),
                    language: "English".to_string(),
                    source_name: "synthetic-review-again.wav".to_string(),
                    source_path: Some(second_source.to_string_lossy().into_owned()),
                    style_id: "style-formal".to_string(),
                })
                .unwrap()
        };
        let outcome = run_import_with_hooks(
            &fixture.root,
            &second_meeting.id,
            &AtomicBool::new(false),
            &|_| {},
            &ProductionHooks,
        )
        .unwrap();
        assert_eq!(outcome, ImportOutcome::DuplicateConfirmation);
    }
}
