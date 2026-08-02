use crate::domain::{
    JobErrorSummary, JobState, JobSummary, MeetingLifecycle, MeetingSummary, NewMeetingInput,
    NewProjectInput, ProjectSummary, ProtocolDocument, ProtocolRevisionSummary, TranscriptDocument,
    TranscriptSegment, WorkspaceSnapshot,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const CURRENT_SCHEMA_VERSION: i64 = 4;
const DEFAULT_STYLE_ID: &str = "style-formal";

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Clone, Debug)]
pub(crate) struct ImportJobRecord {
    pub id: String,
    pub meeting_id: String,
    pub project_id: String,
    pub recording_id: String,
    pub original_name: String,
    pub state: String,
    pub stage: String,
    pub source_path: PathBuf,
    pub duplicate_allowed: bool,
    pub checksum: Option<String>,
    pub byte_count: Option<u64>,
    pub media_type: Option<String>,
    pub final_relative_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) struct CommittedSource {
    pub checksum: String,
    pub byte_count: u64,
    pub media_type: String,
    pub final_relative_path: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessingJobRecord {
    pub id: String,
    pub meeting_id: String,
    pub project_id: String,
    pub recording_id: String,
    pub kind: String,
    pub state: String,
    pub input_revision_id: Option<String>,
    pub result_revision_id: String,
    pub final_relative_path: PathBuf,
    pub fail_requested: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranscriptArtifact {
    pub schema_version: u8,
    pub meeting_id: String,
    pub revision_id: String,
    pub language: String,
    pub segments: Vec<TranscriptSegment>,
}

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    InvalidData(&'static str),
    MissingProject,
    MissingMeeting,
    MissingJob,
    ImportBusy,
    UnsupportedSchema(i64),
}

impl StorageError {
    /// Commands return bounded, content-free messages rather than database paths or SQL details.
    pub fn user_message(&self) -> String {
        match self {
            Self::InvalidData(message) => (*message).to_string(),
            Self::MissingProject => "The selected project no longer exists.".to_string(),
            Self::MissingMeeting => "The selected meeting no longer exists.".to_string(),
            Self::MissingJob => "The import job is no longer available.".to_string(),
            Self::ImportBusy => {
                "Another recording is already being imported. Finish or cancel it first."
                    .to_string()
            }
            Self::UnsupportedSchema(_) => {
                "This LocaLog data was created by a newer, unsupported version.".to_string()
            }
            Self::Io(_) | Self::Sql(_) => {
                "LocaLog could not access its local workspace storage.".to_string()
            }
        }
    }
}

impl Display for StorageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Sql(error) => write!(formatter, "SQLite error: {error}"),
            Self::InvalidData(message) => write!(formatter, "invalid workspace data: {message}"),
            Self::MissingProject => write!(formatter, "project does not exist"),
            Self::MissingMeeting => write!(formatter, "meeting does not exist"),
            Self::MissingJob => write!(formatter, "job does not exist"),
            Self::ImportBusy => write!(formatter, "another import is active"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported schema version {version}")
            }
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}

pub struct WorkspaceRepository {
    pub(crate) connection: Connection,
    pub(crate) root: PathBuf,
}

impl WorkspaceRepository {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        fs::create_dir_all(root)?;
        let connection = Connection::open(root.join("localog.sqlite3"))?;
        let version = schema_version(&connection)?;
        if version > CURRENT_SCHEMA_VERSION {
            // Do not change persistent database settings before compatibility is known.
            return Err(StorageError::UnsupportedSchema(version));
        }
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = FULL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            ",
        )?;
        migrate(&connection, version)?;
        Ok(Self {
            connection,
            root: root.to_path_buf(),
        })
    }

    pub fn workspace_snapshot(&self) -> Result<WorkspaceSnapshot> {
        Ok(WorkspaceSnapshot {
            projects: self.list_projects()?,
            meetings: self.list_meetings()?,
            jobs: self.list_jobs()?,
            transcripts: self.load_transcript_documents()?,
            protocols: self.load_protocol_documents()?,
            active_meeting_id: self.workspace_state("active_meeting_id")?,
            active_route: self.workspace_state("active_route")?,
        })
    }

    pub fn create_project(&mut self, input: NewProjectInput) -> Result<ProjectSummary> {
        let name = required_text(&input.name, 200, "Enter a project name.")?;
        let description = optional_text(&input.description, 2_000, "The description is too long.")?;
        let default_language = required_text(
            &input.default_language,
            64,
            "Choose a valid default meeting language.",
        )?;
        let id = new_id("project");
        let now = unix_time_millis();

        self.connection.execute(
            "INSERT INTO projects (
                id, name, description, default_language, default_style_id,
                created_at_ms, updated_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                id,
                name,
                description,
                default_language,
                DEFAULT_STYLE_ID,
                now
            ],
        )?;

        self.project_by_id(&id)?.ok_or(StorageError::MissingProject)
    }

    pub fn create_meeting(&mut self, input: NewMeetingInput) -> Result<MeetingSummary> {
        let project_id = required_text(&input.project_id, 128, "Choose a valid project.")?;
        let source_name = source_name(&input.source_name)?;
        let title = if input.title.trim().is_empty() {
            title_from_source(&source_name)
        } else {
            required_text(&input.title, 240, "The meeting title is too long.")?
        };
        let occurred_at = meeting_date(&input.occurred_at)?;
        let language = required_text(&input.language, 64, "Choose a valid meeting language.")?;
        let style_id = required_text(&input.style_id, 128, "Choose a valid protocol style.")?;
        let source_path = required_source_path(input.source_path.as_deref())?;

        if !self.project_exists(&project_id)? {
            return Err(StorageError::MissingProject);
        }
        if self.import_is_active()? {
            return Err(StorageError::ImportBusy);
        }

        let meeting_id = new_id("meeting");
        let recording_id = new_id("recording");
        let job_id = new_id("job");
        let now = unix_time_millis();
        let transaction = self.connection.transaction()?;

        transaction.execute(
            "INSERT INTO meetings (
                id, project_id, title, occurred_at, lifecycle, language, style_id,
                created_at_ms, updated_at_ms
             ) VALUES (?1, ?2, ?3, ?4, 'draft', ?5, ?6, ?7, ?7)",
            params![
                meeting_id,
                project_id,
                title,
                occurred_at,
                language,
                style_id,
                now
            ],
        )?;
        // Assignment is durable with the meeting; the later import job will add the managed path.
        transaction.execute(
            "INSERT INTO recordings (
                id, meeting_id, kind, original_name, state, created_at_ms
             ) VALUES (?1, ?2, 'imported', ?3, 'pending', ?4)",
            params![recording_id, meeting_id, source_name, now],
        )?;
        transaction.execute(
            "INSERT INTO jobs (
                id, meeting_id, recording_id, kind, state, stage,
                progress_bytes, attempt, source_path, duplicate_allowed,
                created_at_ms, updated_at_ms
             ) VALUES (?1, ?2, ?3, 'import', 'queued', 'ready_to_import',
                       0, 1, ?4, 0, ?5, ?5)",
            params![job_id, meeting_id, recording_id, source_path, now],
        )?;
        transaction.commit()?;

        self.meeting_by_id(&meeting_id)?
            .ok_or(StorageError::MissingMeeting)
    }

    pub fn update_meeting_title(&self, meeting_id: &str, title: &str) -> Result<()> {
        let meeting_id = required_text(meeting_id, 128, "Choose a valid meeting.")?;
        let title = required_text(title, 240, "Enter a meeting title.")?;
        let updated = self.connection.execute(
            "UPDATE meetings SET title = ?1, updated_at_ms = ?2 WHERE id = ?3",
            params![title, unix_time_millis(), meeting_id],
        )?;
        if updated == 0 {
            return Err(StorageError::MissingMeeting);
        }
        Ok(())
    }

    pub(crate) fn import_job_for_meeting(&self, meeting_id: &str) -> Result<ImportJobRecord> {
        self.connection
            .query_row(
                "SELECT
                    j.id, j.meeting_id, m.project_id, j.recording_id, r.original_name,
                    j.state, j.stage, j.source_path, j.duplicate_allowed,
                    j.result_checksum, j.result_byte_count, j.result_media_type,
                    j.final_relative_path
                 FROM jobs j
                 JOIN meetings m ON m.id = j.meeting_id
                 JOIN recordings r ON r.id = j.recording_id
                 WHERE j.meeting_id = ?1 AND j.kind = 'import'
                 ORDER BY j.created_at_ms DESC, j.id DESC
                 LIMIT 1",
                [meeting_id],
                import_job_from_row,
            )
            .optional()?
            .ok_or(StorageError::MissingJob)
    }

    pub(crate) fn import_job_by_id(&self, job_id: &str) -> Result<ImportJobRecord> {
        self.connection
            .query_row(
                "SELECT
                    j.id, j.meeting_id, m.project_id, j.recording_id, r.original_name,
                    j.state, j.stage, j.source_path, j.duplicate_allowed,
                    j.result_checksum, j.result_byte_count, j.result_media_type,
                    j.final_relative_path
                 FROM jobs j
                 JOIN meetings m ON m.id = j.meeting_id
                 JOIN recordings r ON r.id = j.recording_id
                 WHERE j.id = ?1 AND j.kind = 'import'",
                [job_id],
                import_job_from_row,
            )
            .optional()?
            .ok_or(StorageError::MissingJob)
    }

    pub(crate) fn unfinished_import_jobs(&self) -> Result<Vec<ImportJobRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT
                j.id, j.meeting_id, m.project_id, j.recording_id, r.original_name,
                j.state, j.stage, j.source_path, j.duplicate_allowed,
                j.result_checksum, j.result_byte_count, j.result_media_type,
                j.final_relative_path
             FROM jobs j
             JOIN meetings m ON m.id = j.meeting_id
             JOIN recordings r ON r.id = j.recording_id
             WHERE j.kind = 'import' AND j.state != 'completed'
             ORDER BY j.created_at_ms, j.id",
        )?;
        let rows = statement.query_map([], import_job_from_row)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub(crate) fn mark_abandoned_imports_interrupted(&self) -> Result<()> {
        let now = unix_time_millis();
        self.connection.execute(
            "UPDATE jobs
             SET state = 'interrupted', stage = 'interrupted', error_code = 'interrupted',
                 error_message = NULL, updated_at_ms = ?1, finished_at_ms = ?1
             WHERE kind = 'import' AND state IN ('running', 'cancelling')",
            [now],
        )?;
        Ok(())
    }

    pub(crate) fn mark_import_running(
        &self,
        job_id: &str,
        total_bytes: u64,
        media_type: &str,
        final_relative_path: &Path,
    ) -> Result<()> {
        let now = unix_time_millis();
        let updated = self.connection.execute(
            "UPDATE jobs
             SET state = 'running', stage = 'copying', progress_bytes = 0,
                 total_bytes = ?1, result_media_type = ?2, final_relative_path = ?3,
                 error_code = NULL, error_message = NULL,
                 started_at_ms = COALESCE(started_at_ms, ?4), updated_at_ms = ?4,
                 finished_at_ms = NULL
             WHERE id = ?5 AND kind = 'import'
               AND state IN ('queued', 'failed', 'cancelled', 'interrupted')",
            params![
                total_bytes as i64,
                media_type,
                relative_path_text(final_relative_path)?,
                now,
                job_id
            ],
        )?;
        if updated == 0 {
            return Err(StorageError::MissingJob);
        }
        Ok(())
    }

    pub(crate) fn update_import_progress(&self, job_id: &str, copied_bytes: u64) -> Result<()> {
        self.connection.execute(
            "UPDATE jobs
             SET progress_bytes = ?1, stage = 'copying', updated_at_ms = ?2
             WHERE id = ?3 AND state IN ('running', 'cancelling')",
            params![copied_bytes as i64, unix_time_millis(), job_id],
        )?;
        Ok(())
    }

    pub(crate) fn record_import_validation(
        &self,
        job_id: &str,
        checksum: &str,
        byte_count: u64,
        media_type: &str,
        final_relative_path: &Path,
    ) -> Result<()> {
        self.connection.execute(
            "UPDATE jobs
             SET stage = 'temporary_complete', progress_bytes = ?1, total_bytes = ?1,
                 result_checksum = ?2, result_byte_count = ?1, result_media_type = ?3,
                 final_relative_path = ?4, updated_at_ms = ?5
             WHERE id = ?6 AND state = 'running'",
            params![
                byte_count as i64,
                checksum,
                media_type,
                relative_path_text(final_relative_path)?,
                unix_time_millis(),
                job_id
            ],
        )?;
        Ok(())
    }

    pub(crate) fn probable_duplicate_exists(
        &self,
        recording_id: &str,
        checksum: &str,
    ) -> Result<bool> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM recordings
                 WHERE id != ?1 AND state = 'committed' AND checksum = ?2
                 LIMIT 1",
                params![recording_id, checksum],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    pub(crate) fn pause_for_duplicate(&self, job_id: &str) -> Result<()> {
        self.connection.execute(
            "UPDATE jobs
             SET state = 'queued', stage = 'duplicate_confirmation', updated_at_ms = ?1
             WHERE id = ?2 AND state = 'running'",
            params![unix_time_millis(), job_id],
        )?;
        Ok(())
    }

    pub(crate) fn mark_import_finalizing(&self, job_id: &str) -> Result<()> {
        self.connection.execute(
            "UPDATE jobs SET stage = 'finalizing', updated_at_ms = ?1
             WHERE id = ?2 AND state = 'running'",
            params![unix_time_millis(), job_id],
        )?;
        Ok(())
    }

    pub(crate) fn commit_import(
        &mut self,
        job: &ImportJobRecord,
        source: &CommittedSource,
    ) -> Result<()> {
        let now = unix_time_millis();
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "UPDATE recordings
             SET state = 'committed', managed_path = ?1, checksum = ?2,
                 byte_count = ?3, media_type = ?4
             WHERE id = ?5 AND meeting_id = ?6",
            params![
                relative_path_text(&source.final_relative_path)?,
                source.checksum,
                source.byte_count as i64,
                source.media_type,
                job.recording_id,
                job.meeting_id
            ],
        )?;
        transaction.execute(
            "UPDATE meetings
             SET lifecycle = 'source_ready', updated_at_ms = ?1
             WHERE id = ?2 AND lifecycle = 'draft'",
            params![now, job.meeting_id],
        )?;
        transaction.execute(
            "UPDATE jobs
             SET state = 'completed', stage = 'completed', progress_bytes = ?1,
                 total_bytes = ?1, source_path = NULL, error_code = NULL,
                 error_message = NULL, updated_at_ms = ?2, finished_at_ms = ?2
             WHERE id = ?3",
            params![source.byte_count as i64, now, job.id],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn request_import_cancellation(&self, meeting_id: &str) -> Result<ImportJobRecord> {
        let job = self.import_job_for_meeting(meeting_id)?;
        if matches!(job.state.as_str(), "completed" | "cancelled" | "failed") {
            return Ok(job);
        }
        self.connection.execute(
            "UPDATE jobs SET state = 'cancelling', updated_at_ms = ?1
             WHERE id = ?2 AND state IN ('queued', 'running', 'interrupted')",
            params![unix_time_millis(), job.id],
        )?;
        self.import_job_by_id(&job.id)
    }

    pub(crate) fn mark_import_cancelled(&self, job_id: &str) -> Result<()> {
        let now = unix_time_millis();
        self.connection.execute(
            "UPDATE jobs
             SET state = 'cancelled', stage = 'cancelled', error_code = NULL,
                 error_message = NULL, updated_at_ms = ?1, finished_at_ms = ?1
             WHERE id = ?2 AND state != 'completed'",
            params![now, job_id],
        )?;
        Ok(())
    }

    pub(crate) fn mark_import_failed(&self, job_id: &str, code: &str) -> Result<()> {
        let now = unix_time_millis();
        self.connection.execute(
            "UPDATE jobs
             SET state = 'failed', stage = 'failed', error_code = ?1,
                 error_message = NULL, updated_at_ms = ?2, finished_at_ms = ?2
             WHERE id = ?3 AND state != 'completed'",
            params![code, now, job_id],
        )?;
        Ok(())
    }

    pub(crate) fn retry_import(&self, meeting_id: &str, allow_duplicate: bool) -> Result<String> {
        let job = self.import_job_for_meeting(meeting_id)?;
        let duplicate_resume = job.stage == "duplicate_confirmation" && allow_duplicate;
        if job.source_path.as_os_str().is_empty() {
            return Err(StorageError::InvalidData(
                "The original source must be selected again.",
            ));
        }
        let next_stage = if duplicate_resume {
            "temporary_complete"
        } else {
            "ready_to_import"
        };
        let updated = self.connection.execute(
            "UPDATE jobs
             SET state = 'queued', stage = ?1, duplicate_allowed = ?2,
                 error_code = NULL, error_message = NULL, attempt = attempt + 1,
                 updated_at_ms = ?3, finished_at_ms = NULL
             WHERE id = ?4 AND state IN ('queued', 'failed', 'cancelled', 'interrupted')",
            params![
                next_stage,
                i64::from(allow_duplicate),
                unix_time_millis(),
                job.id
            ],
        )?;
        if updated == 0 {
            return Err(StorageError::MissingJob);
        }
        Ok(job.id)
    }

    pub(crate) fn replace_import_source(
        &mut self,
        meeting_id: &str,
        source_name_value: &str,
        source_path_value: &str,
    ) -> Result<()> {
        let meeting_id = required_text(meeting_id, 128, "Choose a valid meeting.")?;
        let source_name = source_name(source_name_value)?;
        let source_path = required_source_path(Some(source_path_value))?;
        if self.import_is_active()? {
            return Err(StorageError::ImportBusy);
        }
        let recording_id: String = self
            .connection
            .query_row(
                "SELECT id FROM recordings
                 WHERE meeting_id = ?1 AND state != 'committed'
                 ORDER BY created_at_ms, id LIMIT 1",
                [&meeting_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StorageError::MissingMeeting)?;
        let now = unix_time_millis();
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "UPDATE recordings
             SET original_name = ?1, state = 'pending', managed_path = NULL,
                 checksum = NULL, byte_count = NULL, media_type = NULL
             WHERE id = ?2",
            params![source_name, recording_id],
        )?;
        let updated = transaction.execute(
            "UPDATE jobs
             SET state = 'queued', stage = 'ready_to_import', progress_bytes = 0,
                 total_bytes = NULL, source_path = ?1, error_code = NULL,
                 error_message = NULL, duplicate_allowed = 0,
                 result_checksum = NULL, result_byte_count = NULL,
                 result_media_type = NULL, final_relative_path = NULL,
                 attempt = attempt + 1, updated_at_ms = ?2, finished_at_ms = NULL
             WHERE id = (
                 SELECT id FROM jobs WHERE meeting_id = ?3 AND kind = 'import'
                 ORDER BY created_at_ms DESC, id DESC LIMIT 1
             )",
            params![source_path, now, meeting_id],
        )?;
        if updated == 0 {
            transaction.execute(
                "INSERT INTO jobs (
                    id, meeting_id, recording_id, kind, state, stage,
                    progress_bytes, attempt, source_path, duplicate_allowed,
                    created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, 'import', 'queued', 'ready_to_import',
                           0, 1, ?4, 0, ?5, ?5)",
                params![new_id("job"), meeting_id, recording_id, source_path, now],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn list_projects(&self) -> Result<Vec<ProjectSummary>> {
        let mut statement = self.connection.prepare(
            "SELECT
                p.id, p.name, p.description, p.default_language, p.default_style_id,
                COUNT(m.id)
             FROM projects p
             LEFT JOIN meetings m ON m.project_id = p.id AND m.archived_at_ms IS NULL
             WHERE p.archived_at_ms IS NULL
             GROUP BY p.id
             ORDER BY p.created_at_ms, p.id",
        )?;
        let rows = statement.query_map([], project_from_row)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    fn list_meetings(&self) -> Result<Vec<MeetingSummary>> {
        let mut statement = self.connection.prepare(
            "SELECT
                m.id, m.project_id, m.title, m.occurred_at, m.duration_label,
                m.lifecycle, m.language,
                (
                    SELECT r.original_name FROM recordings r
                    WHERE r.meeting_id = m.id
                    ORDER BY r.created_at_ms, r.id LIMIT 1
                ),
                (
                    SELECT r.byte_count FROM recordings r
                    WHERE r.meeting_id = m.id
                    ORDER BY r.created_at_ms, r.id LIMIT 1
                ),
                (
                    SELECT r.media_type FROM recordings r
                    WHERE r.meeting_id = m.id
                    ORDER BY r.created_at_ms, r.id LIMIT 1
                ),
                m.style_id
             FROM meetings m
             WHERE m.archived_at_ms IS NULL
             ORDER BY m.occurred_at DESC, m.created_at_ms DESC, m.id",
        )?;
        let rows = statement.query_map([], meeting_from_row)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    fn project_by_id(&self, id: &str) -> Result<Option<ProjectSummary>> {
        Ok(self
            .connection
            .query_row(
                "SELECT
                    p.id, p.name, p.description, p.default_language, p.default_style_id,
                    COUNT(m.id)
                 FROM projects p
                 LEFT JOIN meetings m ON m.project_id = p.id AND m.archived_at_ms IS NULL
                 WHERE p.id = ?1 AND p.archived_at_ms IS NULL
                 GROUP BY p.id",
                [id],
                project_from_row,
            )
            .optional()?)
    }

    fn meeting_by_id(&self, id: &str) -> Result<Option<MeetingSummary>> {
        Ok(self
            .connection
            .query_row(
                "SELECT
                    m.id, m.project_id, m.title, m.occurred_at, m.duration_label,
                    m.lifecycle, m.language,
                    (
                        SELECT r.original_name FROM recordings r
                        WHERE r.meeting_id = m.id
                        ORDER BY r.created_at_ms, r.id LIMIT 1
                    ),
                    (
                        SELECT r.byte_count FROM recordings r
                        WHERE r.meeting_id = m.id
                        ORDER BY r.created_at_ms, r.id LIMIT 1
                    ),
                    (
                        SELECT r.media_type FROM recordings r
                        WHERE r.meeting_id = m.id
                        ORDER BY r.created_at_ms, r.id LIMIT 1
                    ),
                    m.style_id
                 FROM meetings m
                 WHERE m.id = ?1 AND m.archived_at_ms IS NULL",
                [id],
                meeting_from_row,
            )
            .optional()?)
    }

    fn project_exists(&self, id: &str) -> Result<bool> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM projects WHERE id = ?1 AND archived_at_ms IS NULL",
                [id],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    fn import_is_active(&self) -> Result<bool> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM jobs
                 WHERE kind = 'import' AND state IN ('running', 'cancelling')
                 LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    fn list_jobs(&self) -> Result<Vec<JobSummary>> {
        let mut statement = self.connection.prepare(
            "SELECT
                id, meeting_id, kind, state, stage, progress_bytes, total_bytes,
                attempt, error_code, error_message
             FROM jobs
             ORDER BY updated_at_ms DESC, id DESC",
        )?;
        let rows = statement.query_map([], job_from_row)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    fn workspace_state(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT value FROM workspace_state WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn save_workspace_location(&self, meeting_id: &str, route: &str) -> Result<()> {
        if !matches!(route, "meeting" | "transcript" | "protocol") {
            return Err(StorageError::InvalidData("Choose a valid workspace view."));
        }
        if !self
            .connection
            .query_row(
                "SELECT 1 FROM meetings WHERE id = ?1 AND archived_at_ms IS NULL",
                [meeting_id],
                |_| Ok(()),
            )
            .optional()?
            .is_some()
        {
            return Err(StorageError::MissingMeeting);
        }
        let transaction = self.connection.unchecked_transaction()?;
        for (key, value) in [("active_meeting_id", meeting_id), ("active_route", route)] {
            transaction.execute(
                "INSERT INTO workspace_state (key, value, updated_at_ms) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value,
                     updated_at_ms = excluded.updated_at_ms",
                params![key, value, unix_time_millis()],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn load_transcript_documents(&self) -> Result<HashMap<String, TranscriptDocument>> {
        let mut statement = self.connection.prepare(
            "SELECT w.meeting_id, w.base_revision_id, w.artifact_path, w.checksum,
                    w.updated_at_ms, r.checksum
             FROM transcript_working w
             JOIN transcript_revisions r ON r.id = w.base_revision_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        let mut documents = HashMap::new();
        for row in rows {
            let (meeting_id, base_revision_id, path, working_checksum, saved_at_ms, base_checksum) =
                row?;
            let bytes = read_verified_artifact(&self.root, &path, &working_checksum)?;
            let artifact: TranscriptArtifact = serde_json::from_slice(&bytes)
                .map_err(|_| StorageError::InvalidData("The saved transcript is invalid."))?;
            validate_transcript_artifact(&artifact, &meeting_id)?;
            documents.insert(
                meeting_id,
                TranscriptDocument {
                    schema_version: artifact.schema_version,
                    meeting_id: artifact.meeting_id,
                    revision_id: artifact.revision_id,
                    language: artifact.language,
                    segments: artifact.segments,
                    base_revision_id,
                    is_dirty: working_checksum != base_checksum,
                    save_state: "saved".to_string(),
                    saved_at_ms,
                },
            );
        }
        Ok(documents)
    }

    fn load_protocol_documents(&self) -> Result<HashMap<String, ProtocolDocument>> {
        let mut statement = self.connection.prepare(
            "SELECT w.meeting_id, w.base_revision_id, w.reviewed_revision_id,
                    w.artifact_path, w.checksum, w.updated_at_ms,
                    r.checksum, r.transcript_revision_id, r.style_id
             FROM protocol_working w
             JOIN protocol_revisions r ON r.id = w.base_revision_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;
        let mut documents = HashMap::new();
        for row in rows {
            let (
                meeting_id,
                base_revision_id,
                reviewed_revision_id,
                path,
                working_checksum,
                saved_at_ms,
                base_checksum,
                transcript_revision_id,
                style_id,
            ) = row?;
            let bytes = read_verified_artifact(&self.root, &path, &working_checksum)?;
            let markdown = String::from_utf8(bytes)
                .map_err(|_| StorageError::InvalidData("The saved protocol is invalid."))?;
            let is_dirty = working_checksum != base_checksum;
            let review_state = if reviewed_revision_id.as_deref() == Some(&base_revision_id) {
                if is_dirty {
                    "changed_since_review"
                } else {
                    "reviewed"
                }
            } else {
                "draft"
            };
            documents.insert(
                meeting_id.clone(),
                ProtocolDocument {
                    meeting_id: meeting_id.clone(),
                    revision_id: base_revision_id,
                    transcript_revision_id,
                    markdown,
                    style_id,
                    review_state: review_state.to_string(),
                    is_dirty,
                    save_state: "saved".to_string(),
                    saved_at_ms,
                    revisions: self.protocol_revision_summaries(&meeting_id)?,
                },
            );
        }
        Ok(documents)
    }

    fn protocol_revision_summaries(
        &self,
        meeting_id: &str,
    ) -> Result<Vec<ProtocolRevisionSummary>> {
        let mut statement = self.connection.prepare(
            "SELECT id, ordinal, status, created_at_ms FROM protocol_revisions
             WHERE meeting_id = ?1 ORDER BY ordinal DESC",
        )?;
        let rows = statement.query_map([meeting_id], |row| {
            Ok(ProtocolRevisionSummary {
                id: row.get(0)?,
                ordinal: row.get::<_, i64>(1)? as u32,
                status: row.get(2)?,
                created_at_ms: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    #[cfg(test)]
    fn pending_recording_count(&self, meeting_id: &str) -> Result<u32> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM recordings WHERE meeting_id = ?1 AND state = 'pending'",
            [meeting_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }
}

fn schema_version(connection: &Connection) -> Result<i64> {
    Ok(connection.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

fn migrate(connection: &Connection, version: i64) -> Result<()> {
    let mut version = version;
    if version == 0 {
        connection.execute_batch(
            "
            BEGIN IMMEDIATE;

            CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                default_language TEXT NOT NULL,
                default_style_id TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                archived_at_ms INTEGER
            );

            CREATE TABLE meetings (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id),
                title TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                lifecycle TEXT NOT NULL CHECK (
                    lifecycle IN (
                        'draft', 'source_ready', 'transcript_ready',
                        'protocol_draft', 'reviewed', 'archived'
                    )
                ),
                language TEXT NOT NULL,
                style_id TEXT NOT NULL,
                duration_label TEXT,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                archived_at_ms INTEGER
            );

            CREATE INDEX meetings_project_date
                ON meetings(project_id, occurred_at DESC, created_at_ms DESC);

            CREATE TABLE recordings (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id),
                kind TEXT NOT NULL CHECK (kind IN ('imported', 'microphone', 'system_audio')),
                original_name TEXT NOT NULL,
                state TEXT NOT NULL CHECK (state IN ('pending', 'committed', 'failed')),
                managed_path TEXT,
                checksum TEXT,
                byte_count INTEGER CHECK (byte_count IS NULL OR byte_count >= 0),
                created_at_ms INTEGER NOT NULL
            );

            CREATE INDEX recordings_meeting ON recordings(meeting_id);

            PRAGMA user_version = 1;
            COMMIT;
            ",
        )?;
        version = 1;
    }
    if version == 1 {
        connection.execute_batch(
            "
            BEGIN IMMEDIATE;

            ALTER TABLE recordings ADD COLUMN media_type TEXT;

            CREATE TABLE jobs (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id),
                recording_id TEXT NOT NULL REFERENCES recordings(id),
                kind TEXT NOT NULL CHECK (kind IN ('import', 'transcription', 'generation')),
                state TEXT NOT NULL CHECK (
                    state IN (
                        'queued', 'running', 'cancelling', 'failed',
                        'cancelled', 'interrupted', 'completed'
                    )
                ),
                stage TEXT NOT NULL,
                progress_bytes INTEGER NOT NULL DEFAULT 0 CHECK (progress_bytes >= 0),
                total_bytes INTEGER CHECK (total_bytes IS NULL OR total_bytes >= 0),
                attempt INTEGER NOT NULL DEFAULT 1 CHECK (attempt >= 1),
                source_path TEXT,
                error_code TEXT,
                error_message TEXT,
                duplicate_allowed INTEGER NOT NULL DEFAULT 0 CHECK (duplicate_allowed IN (0, 1)),
                result_checksum TEXT,
                result_byte_count INTEGER CHECK (
                    result_byte_count IS NULL OR result_byte_count >= 0
                ),
                result_media_type TEXT,
                final_relative_path TEXT,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                started_at_ms INTEGER,
                finished_at_ms INTEGER
            );

            CREATE INDEX jobs_meeting_updated
                ON jobs(meeting_id, updated_at_ms DESC);
            CREATE INDEX jobs_active_import
                ON jobs(kind, state, updated_at_ms DESC);

            PRAGMA user_version = 2;
            COMMIT;
            ",
        )?;
        version = 2;
    }
    if version == 2 {
        connection.execute_batch(
            "
            BEGIN IMMEDIATE;
            INSERT INTO jobs (
                id, meeting_id, recording_id, kind, state, stage,
                progress_bytes, attempt, source_path, error_code,
                duplicate_allowed, created_at_ms, updated_at_ms, finished_at_ms
            )
            SELECT
                'job-migrated-' || r.id, r.meeting_id, r.id, 'import', 'failed', 'failed',
                0, 1, NULL, 'source_reselection_required', 0,
                r.created_at_ms, r.created_at_ms, r.created_at_ms
            FROM recordings r
            WHERE r.state = 'pending'
              AND NOT EXISTS (
                  SELECT 1 FROM jobs j
                  WHERE j.recording_id = r.id AND j.kind = 'import'
              );
            PRAGMA user_version = 3;
            COMMIT;
            ",
        )?;
        version = 3;
    }
    if version == 3 {
        connection.execute_batch(
            "
            BEGIN IMMEDIATE;

            ALTER TABLE jobs ADD COLUMN input_revision_id TEXT;
            ALTER TABLE jobs ADD COLUMN result_revision_id TEXT;
            ALTER TABLE jobs ADD COLUMN provider TEXT;
            ALTER TABLE jobs ADD COLUMN runtime_version TEXT;
            ALTER TABLE jobs ADD COLUMN model_digest TEXT;
            ALTER TABLE jobs ADD COLUMN settings_json TEXT;
            ALTER TABLE jobs ADD COLUMN style_revision TEXT;
            ALTER TABLE jobs ADD COLUMN vocabulary_revision TEXT;
            ALTER TABLE jobs ADD COLUMN fail_requested INTEGER NOT NULL DEFAULT 0
                CHECK (fail_requested IN (0, 1));

            CREATE TABLE transcript_revisions (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id),
                recording_id TEXT NOT NULL REFERENCES recordings(id),
                ordinal INTEGER NOT NULL CHECK (ordinal >= 1),
                artifact_path TEXT NOT NULL,
                checksum TEXT NOT NULL,
                byte_count INTEGER NOT NULL CHECK (byte_count > 0),
                language TEXT NOT NULL,
                provider TEXT NOT NULL,
                runtime_version TEXT NOT NULL,
                model_digest TEXT NOT NULL,
                settings_json TEXT NOT NULL,
                source_checksum TEXT NOT NULL,
                app_version TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                UNIQUE (meeting_id, ordinal)
            );
            CREATE INDEX transcript_revisions_meeting
                ON transcript_revisions(meeting_id, ordinal DESC);

            CREATE TABLE transcript_working (
                meeting_id TEXT PRIMARY KEY REFERENCES meetings(id),
                base_revision_id TEXT NOT NULL REFERENCES transcript_revisions(id),
                artifact_path TEXT NOT NULL,
                checksum TEXT NOT NULL,
                byte_count INTEGER NOT NULL CHECK (byte_count > 0),
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE protocol_revisions (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id),
                transcript_revision_id TEXT NOT NULL REFERENCES transcript_revisions(id),
                ordinal INTEGER NOT NULL CHECK (ordinal >= 1),
                artifact_path TEXT NOT NULL,
                checksum TEXT NOT NULL,
                byte_count INTEGER NOT NULL CHECK (byte_count > 0),
                status TEXT NOT NULL CHECK (status IN ('draft', 'reviewed')),
                provider TEXT NOT NULL,
                runtime_version TEXT NOT NULL,
                model_digest TEXT NOT NULL,
                settings_json TEXT NOT NULL,
                style_id TEXT NOT NULL,
                style_revision TEXT NOT NULL,
                vocabulary_revision TEXT NOT NULL,
                transcript_checksum TEXT NOT NULL,
                app_version TEXT NOT NULL,
                restored_from_revision_id TEXT REFERENCES protocol_revisions(id),
                created_at_ms INTEGER NOT NULL,
                UNIQUE (meeting_id, ordinal)
            );
            CREATE INDEX protocol_revisions_meeting
                ON protocol_revisions(meeting_id, ordinal DESC);

            CREATE TABLE protocol_working (
                meeting_id TEXT PRIMARY KEY REFERENCES meetings(id),
                base_revision_id TEXT NOT NULL REFERENCES protocol_revisions(id),
                reviewed_revision_id TEXT REFERENCES protocol_revisions(id),
                artifact_path TEXT NOT NULL,
                checksum TEXT NOT NULL,
                byte_count INTEGER NOT NULL CHECK (byte_count > 0),
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE workspace_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            PRAGMA user_version = 4;
            COMMIT;
            ",
        )?;
    }
    Ok(())
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectSummary> {
    let meeting_count: i64 = row.get(5)?;
    Ok(ProjectSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        default_language: row.get(3)?,
        default_style_id: row.get(4)?,
        meeting_count: meeting_count as u32,
    })
}

fn meeting_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MeetingSummary> {
    let lifecycle: String = row.get(5)?;
    let lifecycle = MeetingLifecycle::from_str(&lifecycle).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            format!("unknown meeting lifecycle: {lifecycle}").into(),
        )
    })?;
    let source_byte_count: Option<i64> = row.get(8)?;
    Ok(MeetingSummary {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        occurred_at: row.get(3)?,
        duration_label: row.get(4)?,
        lifecycle,
        language: row.get(6)?,
        source_name: row.get(7)?,
        source_byte_count: source_byte_count.map(|value| value as u64),
        source_media_type: row.get(9)?,
        style_id: row.get(10)?,
    })
}

fn job_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobSummary> {
    let state_value: String = row.get(3)?;
    let state = JobState::from_str(&state_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            format!("unknown job state: {state_value}").into(),
        )
    })?;
    let stage: String = row.get(4)?;
    let progress_bytes: i64 = row.get(5)?;
    let total_bytes: Option<i64> = row.get(6)?;
    let progress = total_bytes
        .filter(|total| *total > 0)
        .map(|total| ((progress_bytes.saturating_mul(100) / total).clamp(0, 100)) as u8)
        .unwrap_or(0);
    let error_code: Option<String> = row.get(8)?;
    let error_message: Option<String> = row.get(9)?;
    let error = error_code.map(|code| job_error_summary(&code, error_message.as_deref()));
    let outcome = match state {
        JobState::Completed => Some("succeeded".to_string()),
        JobState::Cancelled => Some("cancelled".to_string()),
        _ => None,
    };

    Ok(JobSummary {
        id: row.get(0)?,
        meeting_id: row.get(1)?,
        kind: row.get(2)?,
        state,
        outcome,
        progress,
        progress_bytes: progress_bytes as u64,
        total_bytes: total_bytes.map(|value| value as u64),
        requires_duplicate_confirmation: stage == "duplicate_confirmation",
        stage: job_stage_label(&row.get::<_, String>(2)?, &stage, state),
        attempt: row.get::<_, i64>(7)? as u32,
        error,
    })
}

fn import_job_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImportJobRecord> {
    let source_path: Option<String> = row.get(7)?;
    let final_relative_path: Option<String> = row.get(12)?;
    Ok(ImportJobRecord {
        id: row.get(0)?,
        meeting_id: row.get(1)?,
        project_id: row.get(2)?,
        recording_id: row.get(3)?,
        original_name: row.get(4)?,
        state: row.get(5)?,
        stage: row.get(6)?,
        source_path: source_path.map(PathBuf::from).unwrap_or_default(),
        duplicate_allowed: row.get::<_, i64>(8)? != 0,
        checksum: row.get(9)?,
        byte_count: row.get::<_, Option<i64>>(10)?.map(|value| value as u64),
        media_type: row.get(11)?,
        final_relative_path: final_relative_path.map(PathBuf::from),
    })
}

fn job_stage_label(kind: &str, stage: &str, state: JobState) -> String {
    if stage == "completed" {
        return match kind {
            "transcription" => "Transcript revision committed".to_string(),
            "generation" => "Protocol revision committed".to_string(),
            _ => "Import complete — original unchanged".to_string(),
        };
    }
    if stage == "cancelled" && kind != "import" {
        return "Local processing was cancelled — stable work retained".to_string();
    }
    if stage == "interrupted" && kind != "import" {
        return "Local processing was interrupted — stable work retained".to_string();
    }
    if stage == "failed" && kind != "import" {
        return "Local processing could not finish — stable work retained".to_string();
    }
    match (stage, state) {
        ("ready_to_import", _) => "Ready to copy into local storage".to_string(),
        ("copying", JobState::Cancelling) => "Cancelling the local copy safely".to_string(),
        ("copying", _) => "Copying into local managed storage".to_string(),
        ("validating", _) => "Validating the local copy".to_string(),
        ("temporary_complete", _) => "Preparing the committed source".to_string(),
        ("finalizing", _) => "Committing the source safely".to_string(),
        ("duplicate_confirmation", _) => "Possible duplicate found".to_string(),
        ("completed", _) => "Import complete — original unchanged".to_string(),
        ("cancelled", _) => "Import cancelled — original unchanged".to_string(),
        ("interrupted", _) => "Import was interrupted — original unchanged".to_string(),
        ("failed", _) => "Import could not finish — original unchanged".to_string(),
        ("transcription_queued", _) => "Transcription ready to start".to_string(),
        ("checking_source", _) => "Checking the committed source".to_string(),
        ("preparing_fake_transcriber", _) => "Preparing the local fake transcriber".to_string(),
        ("transcribing_synthetic_segments", _) => {
            "Creating transcript segments locally".to_string()
        }
        ("validating_transcript", _) => "Validating the transcript revision".to_string(),
        ("generation_queued", _) => "Protocol generation ready to start".to_string(),
        ("checking_transcript", _) => "Checking the committed transcript".to_string(),
        ("resolving_protocol_inputs", _) => "Resolving style and vocabulary snapshots".to_string(),
        ("generating_protocol", _) => "Creating the protocol draft locally".to_string(),
        ("validating_protocol", _) => "Validating the protocol revision".to_string(),
        ("output_staged", _) => "Committing the new revision safely".to_string(),
        _ => "Preparing local import".to_string(),
    }
}

fn job_error_summary(code: &str, stored_message: Option<&str>) -> JobErrorSummary {
    let (title, default_detail) = match code {
        "interrupted" => (
            "Import was interrupted",
            "LocaLog stopped before the managed copy was committed. The external original remains unchanged and you can retry safely.",
        ),
        "permission_denied" => (
            "LocaLog could not read or store the recording",
            "Check access to the selected file and LocaLog’s local data location, then try again. The external original was not changed.",
        ),
        "insufficient_space" => (
            "There is not enough local storage",
            "Free some space and retry. No partial recording has been presented as complete.",
        ),
        "source_missing" => (
            "The selected recording is no longer available",
            "Restore the file to its original location or create a new meeting import. The meeting remains safely in Draft.",
        ),
        "source_reselection_required" => (
            "Choose the recording again",
            "This meeting was created by an earlier development build that did not retain the source location. Choose the recording again to continue; the meeting has been preserved.",
        ),
        "unsupported_media" => (
            "This media type is not supported yet",
            "Choose a common audio or video file. The external original was not changed.",
        ),
        "empty_source" => (
            "The selected recording is empty",
            "Choose a recording that contains audio or video data. The empty external file was not changed.",
        ),
        "synthetic_failure" => (
            "The development adapter stopped as requested",
            "The injected failure occurred before a revision was committed. Your source and latest stable work remain safe, and you can retry.",
        ),
        "invalid_adapter_output" => (
            "The local output could not be validated",
            "LocaLog did not commit the incomplete result. Your latest stable source and document revisions remain safe.",
        ),
        "processing_failed" => (
            "Local processing could not finish",
            "No incomplete transcript or protocol was presented as ready. Your latest stable work remains available and you can retry.",
        ),
        _ => (
            "Import could not finish",
            "The meeting remains in Draft and the external original was not changed. You can retry safely.",
        ),
    };
    JobErrorSummary {
        code: code.to_string(),
        title: title.to_string(),
        detail: stored_message.unwrap_or(default_detail).to_string(),
    }
}

fn required_text(value: &str, max_chars: usize, message: &'static str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > max_chars || trimmed.contains('\0') {
        return Err(StorageError::InvalidData(message));
    }
    Ok(trimmed.to_string())
}

fn optional_text(value: &str, max_chars: usize, message: &'static str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.chars().count() > max_chars || trimmed.contains('\0') {
        return Err(StorageError::InvalidData(message));
    }
    Ok(trimmed.to_string())
}

fn source_name(value: &str) -> Result<String> {
    let source_name = required_text(value, 512, "Choose a valid source file.")?;
    if source_name.contains(['/', '\\']) || source_name == "." || source_name == ".." {
        return Err(StorageError::InvalidData("Choose a valid source file."));
    }
    Ok(source_name)
}

fn required_source_path(value: Option<&str>) -> Result<String> {
    let value = value.ok_or(StorageError::InvalidData(
        "Choose the source recording again.",
    ))?;
    let path = required_text(value, 32_768, "Choose a valid source recording.")?;
    if !Path::new(&path).is_absolute() {
        return Err(StorageError::InvalidData(
            "Choose a valid source recording.",
        ));
    }
    Ok(path)
}

fn relative_path_text(path: &Path) -> Result<String> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(StorageError::InvalidData(
            "The managed source path is invalid.",
        ));
    }
    path.to_str()
        .map(ToOwned::to_owned)
        .ok_or(StorageError::InvalidData(
            "The managed source path is invalid.",
        ))
}

pub(crate) fn managed_relative_path(path: &Path) -> Result<String> {
    relative_path_text(path)
}

pub(crate) fn checksum_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_verified_artifact(root: &Path, relative_path: &str, checksum: &str) -> Result<Vec<u8>> {
    let relative = Path::new(relative_path);
    relative_path_text(relative)?;
    let bytes = fs::read(root.join(relative))?;
    if checksum_bytes(&bytes) != checksum {
        return Err(StorageError::InvalidData(
            "A saved document did not pass its local integrity check.",
        ));
    }
    Ok(bytes)
}

pub(crate) fn validate_transcript_artifact(
    artifact: &TranscriptArtifact,
    meeting_id: &str,
) -> Result<()> {
    if artifact.schema_version != 1
        || artifact.meeting_id != meeting_id
        || artifact.segments.is_empty()
        || artifact.segments.len() > 100_000
    {
        return Err(StorageError::InvalidData(
            "The transcript output is invalid.",
        ));
    }
    let mut previous_end = 0;
    let mut identifiers = std::collections::HashSet::new();
    for segment in &artifact.segments {
        if segment.id.is_empty()
            || !identifiers.insert(&segment.id)
            || segment.end_ms < segment.start_ms
            || segment.start_ms < previous_end
            || segment.speaker.chars().count() > 200
            || segment.text.trim().is_empty()
            || segment.text.chars().count() > 20_000
        {
            return Err(StorageError::InvalidData(
                "The transcript output is invalid.",
            ));
        }
        previous_end = segment.end_ms;
    }
    Ok(())
}

fn meeting_date(value: &str) -> Result<String> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit());
    if !valid {
        return Err(StorageError::InvalidData("Choose a valid meeting date."));
    }
    Ok(value.to_string())
}

fn title_from_source(source_name: &str) -> String {
    let stem = source_name
        .rsplit_once('.')
        .map_or(source_name, |(stem, _)| stem)
        .replace(['-', '_'], " ");
    let mut characters = stem.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => "Untitled meeting".to_string(),
    }
}

pub(crate) fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::now_v7())
}

pub(crate) fn unix_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn project_input() -> NewProjectInput {
        NewProjectInput {
            name: "Synthetic civic study".to_string(),
            description: "Synthetic repository fixture".to_string(),
            default_language: "English".to_string(),
        }
    }

    fn meeting_input(project_id: &str, source_path: &Path) -> NewMeetingInput {
        NewMeetingInput {
            project_id: project_id.to_string(),
            title: "".to_string(),
            occurred_at: "2026-08-02".to_string(),
            language: "English".to_string(),
            source_name: "synthetic-design-review.wav".to_string(),
            source_path: Some(source_path.to_string_lossy().into_owned()),
            style_id: DEFAULT_STYLE_ID.to_string(),
        }
    }

    fn synthetic_source(root: &Path) -> PathBuf {
        let source = root.join("synthetic-design-review.wav");
        fs::write(&source, b"synthetic audio fixture").unwrap();
        source
    }

    #[test]
    fn project_and_meeting_survive_repository_reopen() {
        let temporary = tempdir().unwrap();
        let source = synthetic_source(temporary.path());
        let (project, meeting) = {
            let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
            let project = repository.create_project(project_input()).unwrap();
            let meeting = repository
                .create_meeting(meeting_input(&project.id, &source))
                .unwrap();
            assert_eq!(repository.pending_recording_count(&meeting.id).unwrap(), 1);
            (project, meeting)
        };

        let repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let snapshot = repository.workspace_snapshot().unwrap();
        assert_eq!(snapshot.projects[0].id, project.id);
        assert_eq!(snapshot.projects[0].meeting_count, 1);
        assert_eq!(snapshot.meetings, vec![meeting]);
        assert_eq!(snapshot.meetings[0].lifecycle, MeetingLifecycle::Draft);
    }

    #[test]
    fn meeting_and_source_assignment_are_one_transaction() {
        let temporary = tempdir().unwrap();
        let source = synthetic_source(temporary.path());
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let project = repository.create_project(project_input()).unwrap();
        let meeting = repository
            .create_meeting(meeting_input(&project.id, &source))
            .unwrap();

        assert_eq!(
            meeting.source_name.as_deref(),
            Some("synthetic-design-review.wav")
        );
        assert_eq!(repository.pending_recording_count(&meeting.id).unwrap(), 1);
    }

    #[test]
    fn meeting_requires_an_existing_project_and_safe_source_name() {
        let temporary = tempdir().unwrap();
        let source = synthetic_source(temporary.path());
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let missing_project = repository.create_meeting(meeting_input("missing-project", &source));
        assert!(matches!(missing_project, Err(StorageError::MissingProject)));

        let project = repository.create_project(project_input()).unwrap();
        let mut hostile = meeting_input(&project.id, &source);
        hostile.source_name = "../outside.wav".to_string();
        assert!(matches!(
            repository.create_meeting(hostile),
            Err(StorageError::InvalidData(_))
        ));
        assert!(repository.workspace_snapshot().unwrap().meetings.is_empty());
    }

    #[test]
    fn title_updates_are_durable() {
        let temporary = tempdir().unwrap();
        let source = synthetic_source(temporary.path());
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let project = repository.create_project(project_input()).unwrap();
        let meeting = repository
            .create_meeting(meeting_input(&project.id, &source))
            .unwrap();
        repository
            .update_meeting_title(&meeting.id, "Revised synthetic title")
            .unwrap();
        drop(repository);

        let repository = WorkspaceRepository::open(temporary.path()).unwrap();
        assert_eq!(
            repository.workspace_snapshot().unwrap().meetings[0].title,
            "Revised synthetic title"
        );
    }

    #[test]
    fn newer_schema_is_rejected_without_mutation() {
        let temporary = tempdir().unwrap();
        fs::create_dir_all(temporary.path()).unwrap();
        let connection = Connection::open(temporary.path().join("localog.sqlite3")).unwrap();
        connection
            .pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION + 1)
            .unwrap();
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        drop(connection);

        assert!(matches!(
            WorkspaceRepository::open(temporary.path()),
            Err(StorageError::UnsupportedSchema(_))
        ));
        let connection = Connection::open(temporary.path().join("localog.sqlite3")).unwrap();
        let journal_mode_after: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode_after, journal_mode);
    }

    #[test]
    fn version_one_workspace_migrates_without_losing_the_hierarchy() {
        let temporary = tempdir().unwrap();
        let database = temporary.path().join("localog.sqlite3");
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;
                CREATE TABLE projects (
                    id TEXT PRIMARY KEY, name TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '', default_language TEXT NOT NULL,
                    default_style_id TEXT NOT NULL, created_at_ms INTEGER NOT NULL,
                    updated_at_ms INTEGER NOT NULL, archived_at_ms INTEGER
                );
                CREATE TABLE meetings (
                    id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
                    title TEXT NOT NULL, occurred_at TEXT NOT NULL, lifecycle TEXT NOT NULL,
                    language TEXT NOT NULL, style_id TEXT NOT NULL, duration_label TEXT,
                    created_at_ms INTEGER NOT NULL, updated_at_ms INTEGER NOT NULL,
                    archived_at_ms INTEGER
                );
                CREATE TABLE recordings (
                    id TEXT PRIMARY KEY, meeting_id TEXT NOT NULL REFERENCES meetings(id),
                    kind TEXT NOT NULL, original_name TEXT NOT NULL, state TEXT NOT NULL,
                    managed_path TEXT, checksum TEXT, byte_count INTEGER,
                    created_at_ms INTEGER NOT NULL
                );
                INSERT INTO projects VALUES (
                    'project-v1', 'Synthetic migrated project', '', 'English',
                    'style-formal', 1, 1, NULL
                );
                INSERT INTO meetings VALUES (
                    'meeting-v1', 'project-v1', 'Synthetic migrated meeting', '2026-08-02',
                    'draft', 'English', 'style-formal', NULL, 1, 1, NULL
                );
                INSERT INTO recordings VALUES (
                    'recording-v1', 'meeting-v1', 'imported', 'synthetic-v1.wav',
                    'pending', NULL, NULL, NULL, 1
                );
                PRAGMA user_version = 1;
                ",
            )
            .unwrap();
        drop(connection);

        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let snapshot = repository.workspace_snapshot().unwrap();
        assert_eq!(snapshot.projects[0].id, "project-v1");
        assert_eq!(snapshot.meetings[0].id, "meeting-v1");
        assert_eq!(snapshot.jobs.len(), 1);
        assert_eq!(snapshot.jobs[0].state, JobState::Failed);
        assert_eq!(
            snapshot.jobs[0]
                .error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("source_reselection_required")
        );
        assert_eq!(
            schema_version(&repository.connection).unwrap(),
            CURRENT_SCHEMA_VERSION
        );

        let replacement = temporary.path().join("synthetic-v1-reselected.wav");
        fs::write(&replacement, b"synthetic replacement source").unwrap();
        repository
            .replace_import_source(
                "meeting-v1",
                "synthetic-v1-reselected.wav",
                &replacement.to_string_lossy(),
            )
            .unwrap();
        let recovered = repository.workspace_snapshot().unwrap();
        assert_eq!(
            recovered.meetings[0].source_name.as_deref(),
            Some("synthetic-v1-reselected.wav")
        );
        assert_eq!(recovered.jobs[0].state, JobState::Queued);
    }
}
