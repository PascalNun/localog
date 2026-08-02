//! Durable source import: a deliberately small staged copy with explicit recovery boundaries.

use crate::domain::WorkspaceSnapshot;
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
    SourceMissing,
    PermissionDenied,
    InsufficientSpace,
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
    let (media_type, extension) = media_type(&job.original_name)?;
    let paths = import_paths(root, &job, extension)?;

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
    repository.mark_import_running(&job.id, total_bytes, media_type, &paths.final_relative)?;
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
    if paths.temporary.exists() {
        fs::remove_file(&paths.temporary).map_err(classify_destination_error)?;
    }

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

    let source = CommittedSource {
        checksum: format!("{:x}", hasher.finalize()),
        byte_count: copied_bytes,
        media_type: media_type.to_string(),
        final_relative_path: paths.final_relative.clone(),
    };
    repository.record_import_validation(
        &job.id,
        &source.checksum,
        source.byte_count,
        &source.media_type,
        &source.final_relative_path,
    )?;
    notify(repository.workspace_snapshot()?);
    hooks.checkpoint(ImportBoundary::AfterTemporaryFile)?;

    job = repository.import_job_by_id(&job.id)?;
    if repository.probable_duplicate_exists(&job.recording_id, &source.checksum)?
        && !job.duplicate_allowed
    {
        repository.pause_for_duplicate(&job.id)?;
        notify(repository.workspace_snapshot()?);
        return Ok(ImportOutcome::DuplicateConfirmation);
    }

    finalize_import(&mut repository, &job, &paths, source, notify, hooks)
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
        let Ok((_, extension)) = media_type(&job.original_name) else {
            repository.mark_import_failed(&job.id, "unsupported_media")?;
            continue;
        };
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
        if paths.temporary.exists() && job.stage != "duplicate_confirmation" {
            let _ = fs::remove_file(&paths.temporary);
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

fn media_type(original_name: &str) -> Result<(&'static str, &'static str), ImportRunError> {
    let extension = Path::new(original_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or(ImportRunError::UnsupportedMedia)?;
    match extension.as_str() {
        "wav" => Ok(("audio/wav", "wav")),
        "mp3" => Ok(("audio/mpeg", "mp3")),
        "m4a" => Ok(("audio/mp4", "m4a")),
        "aac" => Ok(("audio/aac", "aac")),
        "flac" => Ok(("audio/flac", "flac")),
        "ogg" => Ok(("audio/ogg", "ogg")),
        "opus" => Ok(("audio/opus", "opus")),
        "mp4" => Ok(("video/mp4", "mp4")),
        "mov" => Ok(("video/quicktime", "mov")),
        "mkv" => Ok(("video/x-matroska", "mkv")),
        "webm" => Ok(("video/webm", "webm")),
        _ => Err(ImportRunError::UnsupportedMedia),
    }
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
    if let Ok((_, extension)) = media_type(&job.original_name)
        && let Ok(paths) = import_paths(root, job, extension)
    {
        let _ = fs::remove_file(paths.temporary);
    }
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
        ImportRunError::SourceMissing => "source_missing",
        ImportRunError::PermissionDenied => "permission_denied",
        ImportRunError::InsufficientSpace => "insufficient_space",
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
            let temporary = tempdir().unwrap();
            let root = temporary.path().join("managed");
            let source = temporary.path().join("synthetic-review.wav");
            fs::write(&source, &source_bytes).unwrap();
            let mut repository = WorkspaceRepository::open(&root).unwrap();
            let project = repository
                .create_project(NewProjectInput {
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
                    source_name: "synthetic-review.wav".to_string(),
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
            Some("audio/wav")
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

        let second_source = fixture.source.with_file_name("synthetic-review-copy.wav");
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
                    source_name: "synthetic-review-copy.wav".to_string(),
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
}
