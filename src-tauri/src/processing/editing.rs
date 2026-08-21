//! The working transcript and the working protocol: what somebody has changed
//! and not yet committed.
//!
//! Everything here is reached from the interface rather than from a job. A person
//! retypes a segment, renames a speaker, edits a paragraph of the protocol — and
//! each of those has to leave the file on disk and the row in the database
//! agreeing with each other, which is why they all go through
//! `persist_transcript_working` or `commit_protocol_bytes` rather than writing
//! anything themselves.
//!
//! Committing is separate from saving on purpose. Autosave keeps the working
//! copy current; a revision is only cut when somebody asks for one, or when a job
//! is about to read the document and the working copy is dirty.

use super::durability::{
    cleanup_working_backup, meeting_root, read_verified, replace_working_file, write_durable_new,
};
use super::{APP_VERSION, VOCABULARY_REVISION, processing_to_storage};
use crate::domain::WorkspaceSnapshot;
use crate::storage::{
    Result as StorageResult, StorageError, TranscriptArtifact, WorkspaceRepository, checksum_bytes,
    managed_relative_path, new_id, unix_time_millis, validate_transcript_artifact,
};
use rusqlite::{OptionalExtension, params};
use std::path::Path;

pub(crate) fn autosave_transcript_segment(
    root: &Path,
    meeting_id: &str,
    segment_id: &str,
    text: &str,
) -> StorageResult<WorkspaceSnapshot> {
    let repository = WorkspaceRepository::open(root)?;
    let (path, mut artifact) = working_transcript(root, &repository, meeting_id)?;
    let segment = artifact
        .segments
        .iter_mut()
        .find(|segment| segment.id == segment_id)
        .ok_or(StorageError::InvalidData(
            "The transcript segment no longer exists.",
        ))?;
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 20_000 {
        return Err(StorageError::InvalidData("Enter valid transcript text."));
    }
    segment.text = trimmed.to_string();
    // The reader has now said what the words are, so the model's doubt is settled.
    segment.needs_review = false;
    segment.uncertain_words.clear();
    persist_transcript_working(&repository, meeting_id, &path, &artifact)?;
    repository.workspace_snapshot()
}
/// Take a segment out of the working transcript.
///
/// For the throat-clearing, the crosstalk, and the thirty seconds of somebody's
/// dog. What is removed is not paraphrased or emptied: it is gone from the working
/// document, so nothing downstream has to decide what an empty segment means.
///
/// The committed revision this was edited from is untouched, as every revision is,
/// so a person who deletes the wrong line can return to the transcript as
/// transcribed. That is the safety net here — there is no per-segment undo, and
/// pretending otherwise would be worse than saying so.
///
/// The last segment cannot be deleted. A transcript of nothing is not a document
/// somebody meant to make, and every screen downstream would have to learn to read
/// one.
pub(crate) fn delete_transcript_segment(
    root: &Path,
    meeting_id: &str,
    segment_id: &str,
) -> StorageResult<WorkspaceSnapshot> {
    let repository = WorkspaceRepository::open(root)?;
    let (path, mut artifact) = working_transcript(root, &repository, meeting_id)?;
    if !artifact
        .segments
        .iter()
        .any(|segment| segment.id == segment_id)
    {
        return Err(StorageError::InvalidData(
            "The transcript segment no longer exists.",
        ));
    }
    if artifact.segments.len() <= 1 {
        return Err(StorageError::InvalidData(
            "A transcript needs at least one segment.",
        ));
    }
    artifact.segments.retain(|segment| segment.id != segment_id);
    persist_transcript_working(&repository, meeting_id, &path, &artifact)?;
    repository.workspace_snapshot()
}
pub(crate) fn rename_speaker(
    root: &Path,
    meeting_id: &str,
    speaker: &str,
    replacement: &str,
) -> StorageResult<WorkspaceSnapshot> {
    let repository = WorkspaceRepository::open(root)?;
    let replacement = replacement.trim();
    if replacement.is_empty() || replacement.chars().count() > 200 {
        return Err(StorageError::InvalidData("Enter a valid speaker label."));
    }
    let (path, mut artifact) = working_transcript(root, &repository, meeting_id)?;
    for segment in &mut artifact.segments {
        if segment.speaker == speaker {
            segment.speaker = replacement.to_string();
        }
    }
    persist_transcript_working(&repository, meeting_id, &path, &artifact)?;
    repository.workspace_snapshot()
}
pub(super) fn persist_transcript_working(
    repository: &WorkspaceRepository,
    meeting_id: &str,
    path: &str,
    artifact: &TranscriptArtifact,
) -> StorageResult<()> {
    validate_transcript_artifact(artifact, meeting_id)?;
    let bytes = serde_json::to_vec_pretty(artifact)
        .map_err(|_| StorageError::InvalidData("The transcript could not be saved."))?;
    let relative = Path::new(path);
    replace_working_file(&repository.root, relative, &bytes, None)
        .map_err(processing_to_storage)?;
    repository.connection.execute(
        "UPDATE transcript_working SET checksum = ?1, byte_count = ?2, updated_at_ms = ?3
         WHERE meeting_id = ?4",
        params![
            checksum_bytes(&bytes),
            bytes.len() as i64,
            unix_time_millis(),
            meeting_id
        ],
    )?;
    cleanup_working_backup(&repository.root, relative);
    Ok(())
}
pub(crate) fn autosave_protocol(
    root: &Path,
    meeting_id: &str,
    markdown: &str,
) -> StorageResult<WorkspaceSnapshot> {
    if markdown.trim().is_empty() || markdown.len() > 5_000_000 {
        return Err(StorageError::InvalidData("Enter valid protocol text."));
    }
    let repository = WorkspaceRepository::open(root)?;
    let path: String = repository.connection.query_row(
        "SELECT artifact_path FROM protocol_working WHERE meeting_id = ?1",
        [meeting_id],
        |row| row.get(0),
    )?;
    let relative = Path::new(&path);
    replace_working_file(&repository.root, relative, markdown.as_bytes(), None)
        .map_err(processing_to_storage)?;
    repository.connection.execute(
        "UPDATE protocol_working SET checksum = ?1, byte_count = ?2, updated_at_ms = ?3
         WHERE meeting_id = ?4",
        params![
            checksum_bytes(markdown.as_bytes()),
            markdown.len() as i64,
            unix_time_millis(),
            meeting_id,
        ],
    )?;
    cleanup_working_backup(&repository.root, relative);
    repository.workspace_snapshot()
}
pub(crate) fn create_protocol_revision(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<WorkspaceSnapshot> {
    let mut repository = WorkspaceRepository::open(root)?;
    force_protocol_revision(&mut repository, meeting_id)?;
    repository.workspace_snapshot()
}
pub(crate) fn mark_protocol_reviewed(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<WorkspaceSnapshot> {
    let mut repository = WorkspaceRepository::open(root)?;
    let revision_id = force_protocol_revision(&mut repository, meeting_id)?;
    let transaction = repository.connection.transaction()?;
    transaction.execute(
        "UPDATE protocol_revisions SET status = 'reviewed' WHERE id = ?1 AND meeting_id = ?2",
        params![revision_id, meeting_id],
    )?;
    transaction.execute(
        "UPDATE protocol_working SET reviewed_revision_id = ?1, updated_at_ms = ?2
         WHERE meeting_id = ?3",
        params![revision_id, unix_time_millis(), meeting_id],
    )?;
    transaction.execute(
        "UPDATE meetings SET lifecycle = 'reviewed', updated_at_ms = ?1 WHERE id = ?2",
        params![unix_time_millis(), meeting_id],
    )?;
    transaction.commit()?;
    repository.workspace_snapshot()
}
pub(super) fn force_protocol_revision(
    repository: &mut WorkspaceRepository,
    meeting_id: &str,
) -> StorageResult<String> {
    let (path, checksum): (String, String) = repository.connection.query_row(
        "SELECT artifact_path, checksum FROM protocol_working WHERE meeting_id = ?1",
        [meeting_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let bytes = read_verified(&repository.root, &path, &checksum).map_err(processing_to_storage)?;
    commit_protocol_bytes(repository, meeting_id, &bytes, None)
}
pub(crate) fn restore_protocol_revision(
    root: &Path,
    meeting_id: &str,
    revision_id: &str,
) -> StorageResult<WorkspaceSnapshot> {
    let mut repository = WorkspaceRepository::open(root)?;
    let (path, checksum): (String, String) = repository
        .connection
        .query_row(
            "SELECT artifact_path, checksum FROM protocol_revisions
             WHERE id = ?1 AND meeting_id = ?2",
            params![revision_id, meeting_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or(StorageError::InvalidData(
            "The selected protocol revision no longer exists.",
        ))?;
    let bytes = read_verified(root, &path, &checksum).map_err(processing_to_storage)?;
    commit_protocol_bytes(&mut repository, meeting_id, &bytes, Some(revision_id))?;
    repository.workspace_snapshot()
}
pub(super) fn commit_transcript_working_if_dirty(
    repository: &mut WorkspaceRepository,
    meeting_id: &str,
) -> StorageResult<String> {
    let (base_id, path, checksum, base_checksum): (String, String, String, String) = repository
        .connection
        .query_row(
            "SELECT w.base_revision_id, w.artifact_path, w.checksum, r.checksum
             FROM transcript_working w JOIN transcript_revisions r ON r.id = w.base_revision_id
             WHERE w.meeting_id = ?1",
            [meeting_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?
        .ok_or(StorageError::InvalidData(
            "Review the transcript before generation.",
        ))?;
    if checksum == base_checksum {
        return Ok(base_id);
    }
    let bytes = read_verified(&repository.root, &path, &checksum).map_err(processing_to_storage)?;
    let mut artifact: TranscriptArtifact = serde_json::from_slice(&bytes)
        .map_err(|_| StorageError::InvalidData("The saved transcript is invalid."))?;
    let revision_id = new_id("transcript");
    artifact.revision_id = revision_id.clone();
    validate_transcript_artifact(&artifact, meeting_id)?;
    let revision_bytes = serde_json::to_vec_pretty(&artifact)
        .map_err(|_| StorageError::InvalidData("The transcript could not be committed."))?;
    let (project_id, recording_id, language, source_checksum): (String, String, String, String) =
        repository.connection.query_row(
            "SELECT m.project_id, r.recording_id, r.language, r.source_checksum
             FROM transcript_revisions r JOIN meetings m ON m.id = r.meeting_id
             WHERE r.id = ?1",
            [&base_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
    let relative = meeting_root(&project_id, meeting_id)
        .join("transcripts/revisions")
        .join(format!("{revision_id}.json"));
    write_durable_new(&repository.root.join(&relative), &revision_bytes)
        .map_err(processing_to_storage)?;
    replace_working_file(&repository.root, Path::new(&path), &revision_bytes, None)
        .map_err(processing_to_storage)?;
    let next_checksum = checksum_bytes(&revision_bytes);
    let transaction = repository.connection.transaction()?;
    let ordinal: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(ordinal), 0) + 1 FROM transcript_revisions WHERE meeting_id = ?1",
        [meeting_id],
        |row| row.get(0),
    )?;
    transaction.execute(
        "INSERT INTO transcript_revisions (
            id, meeting_id, recording_id, ordinal, artifact_path, checksum, byte_count,
            language, provider, runtime_version, model_digest, settings_json,
            source_checksum, app_version, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'local-user-edit', '1',
                   'none', '{}', ?9, ?10, ?11)",
        params![
            revision_id,
            meeting_id,
            recording_id,
            ordinal,
            managed_relative_path(&relative)?,
            next_checksum,
            revision_bytes.len() as i64,
            language,
            source_checksum,
            APP_VERSION,
            unix_time_millis(),
        ],
    )?;
    transaction.execute(
        "UPDATE transcript_working SET base_revision_id = ?1, checksum = ?2,
                byte_count = ?3, updated_at_ms = ?4 WHERE meeting_id = ?5",
        params![
            revision_id,
            next_checksum,
            revision_bytes.len() as i64,
            unix_time_millis(),
            meeting_id,
        ],
    )?;
    transaction.commit()?;
    cleanup_working_backup(&repository.root, Path::new(&path));
    Ok(revision_id)
}
pub(super) fn commit_protocol_bytes(
    repository: &mut WorkspaceRepository,
    meeting_id: &str,
    bytes: &[u8],
    restored_from: Option<&str>,
) -> StorageResult<String> {
    let (project_id, transcript_id, transcript_checksum, style_id, working_path): (
        String,
        String,
        String,
        String,
        String,
    ) = repository.connection.query_row(
        "SELECT m.project_id, p.transcript_revision_id, t.checksum, m.style_id, w.artifact_path
         FROM protocol_working w JOIN protocol_revisions p ON p.id = w.base_revision_id
         JOIN transcript_revisions t ON t.id = p.transcript_revision_id
         JOIN meetings m ON m.id = w.meeting_id WHERE w.meeting_id = ?1",
        [meeting_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )?;
    let revision_id = new_id("protocol");
    let relative = meeting_root(&project_id, meeting_id)
        .join("protocols/revisions")
        .join(format!("{revision_id}.md"));
    write_durable_new(&repository.root.join(&relative), bytes).map_err(processing_to_storage)?;
    replace_working_file(&repository.root, Path::new(&working_path), bytes, None)
        .map_err(processing_to_storage)?;
    let checksum = checksum_bytes(bytes);
    let transaction = repository.connection.transaction()?;
    let ordinal: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(ordinal), 0) + 1 FROM protocol_revisions WHERE meeting_id = ?1",
        [meeting_id],
        |row| row.get(0),
    )?;
    transaction.execute(
        "INSERT INTO protocol_revisions (
            id, meeting_id, transcript_revision_id, ordinal, artifact_path, checksum,
            byte_count, status, provider, runtime_version, model_digest, settings_json,
            style_id, style_revision, vocabulary_revision, transcript_checksum,
            app_version, restored_from_revision_id, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'draft', 'local-user-edit', '1',
                   'none', '{}', ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            revision_id,
            meeting_id,
            transcript_id,
            ordinal,
            managed_relative_path(&relative)?,
            checksum,
            bytes.len() as i64,
            style_id,
            format!("{style_id}@1"),
            VOCABULARY_REVISION,
            transcript_checksum,
            APP_VERSION,
            restored_from,
            unix_time_millis(),
        ],
    )?;
    transaction.execute(
        "UPDATE protocol_working SET base_revision_id = ?1, checksum = ?2,
                byte_count = ?3, updated_at_ms = ?4 WHERE meeting_id = ?5",
        params![
            revision_id,
            checksum,
            bytes.len() as i64,
            unix_time_millis(),
            meeting_id,
        ],
    )?;
    transaction.execute(
        "UPDATE meetings SET lifecycle = 'protocol_draft', updated_at_ms = ?1 WHERE id = ?2",
        params![unix_time_millis(), meeting_id],
    )?;
    transaction.commit()?;
    cleanup_working_backup(&repository.root, Path::new(&working_path));
    Ok(revision_id)
}
/// The working transcript of a meeting, with the path it is stored at.
pub(super) fn working_transcript(
    root: &Path,
    repository: &WorkspaceRepository,
    meeting_id: &str,
) -> StorageResult<(String, TranscriptArtifact)> {
    let (path, checksum): (String, String) = repository.connection.query_row(
        "SELECT artifact_path, checksum FROM transcript_working WHERE meeting_id = ?1",
        [meeting_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let bytes = read_verified(root, &path, &checksum).map_err(processing_to_storage)?;
    let artifact: TranscriptArtifact = serde_json::from_slice(&bytes)
        .map_err(|_| StorageError::InvalidData("The saved transcript is invalid."))?;
    Ok((path, artifact))
}
