use crate::domain::{
    DocumentAppearance, FurnitureField, FurnitureRow, JobErrorSummary, PageFurniture,
    PageWidth, Scale, Spacing, JobState, JobSummary, MeetingLifecycle, MeetingSummary, NewMeetingInput,
    NewProjectInput, ProjectSummary, ProtocolDensity, ProtocolDocument, ProtocolEvidence,
    ProtocolRevisionSummary, ProtocolStyle, SpeakerResolution, TranscriptDocument,
    TranscriptSegment, VocabularyDraft, VocabularyEntry, WorkspaceSnapshot,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const CURRENT_SCHEMA_VERSION: i64 = 19;
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
    pub provider: Option<String>,
    pub runtime_version: Option<String>,
    pub model_digest: Option<String>,
    pub settings_json: Option<String>,
    pub runtime_config_json: Option<String>,
    pub style_revision: Option<String>,
    pub vocabulary_revision: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranscriptArtifact {
    pub schema_version: u8,
    pub meeting_id: String,
    pub revision_id: String,
    pub language: String,
    #[serde(default)]
    pub speaker_resolution: SpeakerResolution,
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

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct ResolvedProtocolStyle {
    pub id: String,
    pub revision: String,
    pub instructions: Vec<String>,
    pub required_sections: Vec<String>,
    pub density: ProtocolDensity,
}

/// A saved way of presenting a protocol.
///
/// The typography and the running header and footer, named. Not the protocol style:
/// that decides what the document says, this decides how it is set, and the two are
/// kept apart on purpose.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub appearance: DocumentAppearance,
    pub furniture: PageFurniture,
    /// One that shipped, which can be used and copied but not overwritten.
    pub built_in: bool,
}

/// A section taken out of a protocol and kept in case it is wanted back.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAsideSection {
    pub title: String,
    /// The whole block, heading and all, exactly as it was.
    pub markdown: String,
}

/// A protocol style as somebody reading it needs to see it.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolStyleDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub density: ProtocolDensity,
    /// What the style asks the model for, in the order it asks.
    pub instructions: Vec<String>,
    pub required_sections: Vec<String>,
    pub as_shipped: bool,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct ResolvedVocabularyEntry {
    pub term: String,
    pub preferred_spelling: String,
    pub category: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedProtocolInputs {
    pub meeting_language: String,
    pub style: ResolvedProtocolStyle,
    pub vocabulary: Vec<ResolvedVocabularyEntry>,
    pub vocabulary_revision: String,
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
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = FULL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            "#,
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
            styles: self.list_protocol_styles()?,
            vocabulary: self.list_vocabulary()?,
            active_meeting_id: self.workspace_state("active_meeting_id")?,
            active_route: self.workspace_state("active_route")?,
        })
    }

    /// Resolve the current professional preset and vocabulary for a meeting.
    /// The processing layer snapshots the returned values before it starts work.
    pub(crate) fn protocol_inputs(&self, meeting_id: &str) -> Result<ResolvedProtocolInputs> {
        let (project_id, style_id, language): (String, String, String) = self
            .connection
            .query_row(
                "SELECT project_id, style_id, language FROM meetings WHERE id = ?1",
                [meeting_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?
            .ok_or(StorageError::MissingMeeting)?;
        let style = self
            .connection
            .query_row(
                "SELECT id, name, description, language_scope, instructions_json,
                        required_sections_json, revision, density
                 FROM protocol_styles WHERE id = ?1 AND enabled = 1",
                [&style_id],
                protocol_style_from_row,
            )
            .optional()?
            .ok_or(StorageError::InvalidData(
                "The selected protocol style is unavailable.",
            ))?;
        let vocabulary = self.list_vocabulary_for_project(&project_id)?;
        let resolved_vocabulary: Vec<ResolvedVocabularyEntry> = vocabulary
            .iter()
            .map(|entry| ResolvedVocabularyEntry {
                term: entry.term.clone(),
                preferred_spelling: entry.term.clone(),
                category: entry.category.clone(),
            })
            .collect();
        let vocabulary_json = serde_json::to_vec(&resolved_vocabulary)
            .map_err(|_| StorageError::InvalidData("The vocabulary could not be resolved."))?;
        Ok(ResolvedProtocolInputs {
            meeting_language: language,
            style,
            vocabulary: resolved_vocabulary,
            vocabulary_revision: format!("sha256:{}", checksum_bytes(&vocabulary_json)),
        })
    }

    /// Every saved way of presenting a protocol, shipped ones first.
    pub fn list_export_templates(&self) -> Result<Vec<ExportTemplate>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, description, appearance_json, furniture_json, built_in
             FROM export_templates ORDER BY built_in DESC, name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            let appearance: String = row.get(3)?;
            let furniture: String = row.get(4)?;
            let built_in: i64 = row.get(5)?;
            Ok(ExportTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                // A template nobody can read falls back to the defaults rather than
                // taking the whole library down with it.
                appearance: serde_json::from_str(&appearance).unwrap_or_default(),
                furniture: serde_json::from_str(&furniture).unwrap_or_default(),
                built_in: built_in == 1,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// Save how a project sets its protocols, under a name.
    pub fn save_export_template(
        &self,
        name: &str,
        description: &str,
        appearance: &DocumentAppearance,
        furniture: &PageFurniture,
    ) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(StorageError::InvalidData("Give the template a name."));
        }
        let now = unix_time_millis();
        self.connection.execute(
            "INSERT INTO export_templates
                (id, name, description, appearance_json, furniture_json, built_in,
                 created_at_ms, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
            params![
                new_id("template"),
                name,
                description.trim(),
                serde_json::to_string(appearance)
                    .map_err(|_| StorageError::InvalidData("The template could not be saved."))?,
                serde_json::to_string(furniture)
                    .map_err(|_| StorageError::InvalidData("The template could not be saved."))?,
                now
            ],
        )?;
        Ok(())
    }

    /// Remove one that was made here. The shipped ones stay.
    pub fn delete_export_template(&self, template_id: &str) -> Result<()> {
        let changed = self.connection.execute(
            "DELETE FROM export_templates WHERE id = ?1 AND built_in = 0",
            [template_id],
        )?;
        if changed == 0 {
            return Err(StorageError::InvalidData(
                "A template that shipped with LocaLog cannot be deleted.",
            ));
        }
        Ok(())
    }

    /// Sections taken out of a protocol without being thrown away.
    pub fn set_aside_sections(&self, meeting_id: &str) -> Result<Vec<SetAsideSection>> {
        let json: Option<String> = self
            .connection
            .query_row(
                "SELECT set_aside_json FROM protocol_working WHERE meeting_id = ?1",
                [meeting_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        // Unreadable is treated as none: a stash nobody can parse is not worth
        // refusing to open the protocol over.
        Ok(json
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default())
    }

    pub fn write_set_aside_sections(
        &self,
        meeting_id: &str,
        sections: &[SetAsideSection],
    ) -> Result<()> {
        let json = serde_json::to_string(sections)
            .map_err(|_| StorageError::InvalidData("The section could not be set aside."))?;
        let changed = self.connection.execute(
            "UPDATE protocol_working SET set_aside_json = ?1, updated_at_ms = ?2
             WHERE meeting_id = ?3",
            params![json, unix_time_millis(), meeting_id],
        )?;
        if changed == 0 {
            return Err(StorageError::InvalidData(
                "Generate a protocol before setting a section aside.",
            ));
        }
        Ok(())
    }

    pub(crate) fn protocol_working_markdown(&self, meeting_id: &str) -> Result<Vec<u8>> {
        let (path, checksum): (String, String) = self
            .connection
            .query_row(
                "SELECT artifact_path, checksum FROM protocol_working WHERE meeting_id = ?1",
                [meeting_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or(StorageError::InvalidData(
                "Generate a protocol before exporting it.",
            ))?;
        read_verified_artifact(&self.root, &path, &checksum)
    }

    #[cfg(test)]
    fn protocol_inputs_style(&self, style_id: &str) -> ResolvedProtocolStyle {
        self.connection
            .query_row(
                "SELECT id, name, description, language_scope, instructions_json,
                        required_sections_json, revision, density
                 FROM protocol_styles WHERE id = ?1",
                [style_id],
                protocol_style_from_row,
            )
            .expect("seeded style must exist")
    }

    pub(crate) fn read_setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(crate) fn write_setting(&self, key: &str, value: &str) -> Result<()> {
        if key.len() > 128 || value.len() > 32_768 || key.contains('\0') || value.contains('\0') {
            return Err(StorageError::InvalidData(
                "The local runtime setting is invalid.",
            ));
        }
        self.connection.execute(
            "INSERT INTO app_settings (key, value, updated_at_ms) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at_ms = excluded.updated_at_ms",
            params![key, value, unix_time_millis()],
        )?;
        Ok(())
    }

    pub(crate) fn read_model_provenance_cache(
        &self,
        model_path: &str,
        byte_count: u64,
        modified_at_ns: &str,
    ) -> Result<Option<(String, u64)>> {
        let cached: Option<(String, i64)> = self
            .connection
            .query_row(
                "SELECT digest, byte_count FROM model_provenance_cache
                 WHERE model_path = ?1 AND byte_count = ?2 AND modified_at_ns = ?3",
                params![model_path, i64::try_from(byte_count).ok(), modified_at_ns],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(cached.and_then(|(digest, byte_count)| {
            u64::try_from(byte_count)
                .ok()
                .map(|byte_count| (digest, byte_count))
        }))
    }

    pub(crate) fn write_model_provenance_cache(
        &self,
        model_path: &str,
        byte_count: u64,
        modified_at_ns: &str,
        digest: &str,
    ) -> Result<()> {
        let Some(byte_count) = i64::try_from(byte_count).ok() else {
            return Ok(());
        };
        self.connection.execute(
            "INSERT INTO model_provenance_cache
                (model_path, byte_count, modified_at_ns, digest, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(model_path) DO UPDATE SET
                byte_count = excluded.byte_count,
                modified_at_ns = excluded.modified_at_ns,
                digest = excluded.digest,
                updated_at_ms = excluded.updated_at_ms",
            params![
                model_path,
                byte_count,
                modified_at_ns,
                digest,
                unix_time_millis()
            ],
        )?;
        Ok(())
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
        // A meeting about to be recorded has no source yet. Everything else about it
        // is the same, so the absence of a file is the only difference: no recording
        // row, no import job, and a title somebody has to give because there is no
        // filename to take one from.
        let to_record = input.source_name.trim().is_empty() && input.source_path.is_none();
        let source_name = if to_record {
            String::new()
        } else {
            source_name(&input.source_name)?
        };
        let title = if !input.title.trim().is_empty() {
            required_text(&input.title, 240, "The meeting title is too long.")?
        } else if to_record {
            return Err(StorageError::InvalidData(
                "Give the meeting a title. There is no file to take one from.",
            ));
        } else {
            title_from_source(&source_name)
        };
        let occurred_at = meeting_date(&input.occurred_at)?;
        let language = required_text(&input.language, 64, "Choose a valid meeting language.")?;
        let style_id = required_text(&input.style_id, 128, "Choose a valid protocol style.")?;
        let source_path = if to_record {
            String::new()
        } else {
            required_source_path(input.source_path.as_deref())?
        };

        if !self.project_exists(&project_id)? {
            return Err(StorageError::MissingProject);
        }
        if !to_record && self.import_is_active()? {
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
        // A meeting about to be recorded has neither: the recording row is written when
        // the recorder starts, and there is nothing to import.
        if !to_record {
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
        }
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

    pub fn update_meeting_language(&self, meeting_id: &str, language: &str) -> Result<()> {
        let meeting_id = required_text(meeting_id, 128, "Choose a valid meeting.")?;
        let language = required_text(language, 64, "Choose a meeting language.")?;
        let updated = self.connection.execute(
            "UPDATE meetings SET language = ?1, updated_at_ms = ?2 WHERE id = ?3",
            params![language, unix_time_millis(), meeting_id],
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
                COUNT(m.id), p.appearance_json, p.furniture_json
             FROM projects p
             LEFT JOIN meetings m ON m.project_id = p.id AND m.archived_at_ms IS NULL
             WHERE p.archived_at_ms IS NULL
             GROUP BY p.id
             ORDER BY p.created_at_ms, p.id",
        )?;
        let rows = statement.query_map([], project_from_row)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// What the generation run recorded about the draft it produced.
    ///
    /// Read from the job that committed this revision, so a draft always carries
    /// the evidence of its own run rather than of the most recent one. A row that
    /// cannot be parsed is treated as absent: this informs a reader and must never
    /// stop them opening their protocol.
    fn protocol_evidence(&self, revision_id: &str) -> Result<Option<ProtocolEvidence>> {
        let recorded: Option<Option<String>> = self
            .connection
            .query_row(
                "SELECT outcome_json FROM jobs
                 WHERE result_revision_id = ?1 AND kind = 'generation'
                 ORDER BY created_at_ms DESC LIMIT 1",
                [revision_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(recorded
            .flatten()
            .and_then(|value| serde_json::from_str(&value).ok()))
    }

    /// What somebody trimmed from this meeting's recording.
    ///
    /// An unreadable or absent record is an untouched recording. Losing the edits
    /// costs somebody their trims; refusing to open the meeting because of them
    /// would cost them the meeting.
    pub(crate) fn recording_edits(&self, meeting_id: &str) -> Result<crate::edits::Edits> {
        let stored: Option<Option<String>> = self
            .connection
            .query_row(
                "SELECT recording_edits_json FROM meetings WHERE id = ?1",
                [meeting_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(stored
            .flatten()
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or_default())
    }

    /// Record what to leave out. The recording itself is never touched, here or
    /// anywhere: these are read when the working audio is built and at no other
    /// time, so changing them is always reversible.
    pub(crate) fn set_recording_edits(
        &self,
        meeting_id: &str,
        edits: &crate::edits::Edits,
    ) -> Result<()> {
        // An untouched recording stores nothing rather than an empty document, so
        // a meeting nobody edited is indistinguishable from one made before this
        // existed.
        let value = if edits.is_untouched() {
            None
        } else {
            Some(
                serde_json::to_string(edits)
                    .map_err(|_| StorageError::InvalidData("Those edits cannot be recorded."))?,
            )
        };
        let changed = self.connection.execute(
            "UPDATE meetings SET recording_edits_json = ?2, updated_at_ms = ?3 WHERE id = ?1",
            params![meeting_id, value, unix_time_millis()],
        )?;
        if changed == 0 {
            return Err(StorageError::MissingMeeting);
        }
        Ok(())
    }

    /// One style, read in full.
    ///
    /// The list a person browses carries a name, a description and a density, and
    /// none of what the style actually asks the model for. Choosing between three
    /// styles was therefore choosing between three sentences. This is what is really
    /// being chosen.
    pub fn protocol_style_detail(&self, style_id: &str) -> Result<ProtocolStyleDetail> {
        let style = self
            .connection
            .query_row(
                "SELECT id, name, description, language_scope, instructions_json,
                        required_sections_json, revision, density
                 FROM protocol_styles WHERE id = ?1 AND enabled = 1",
                [style_id],
                protocol_style_from_row,
            )
            .optional()?
            .ok_or(StorageError::InvalidData(
                "The selected protocol style is unavailable.",
            ))?;
        let (name, description, edited): (String, String, i64) = self.connection.query_row(
            "SELECT name, description, revision FROM protocol_styles WHERE id = ?1",
            [style_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        Ok(ProtocolStyleDetail {
            id: style.id,
            name,
            description,
            density: style.density,
            instructions: style.instructions,
            required_sections: style.required_sections,
            // A style that has never been edited is the one that shipped. The
            // distinction matters to somebody deciding whether they may change it.
            as_shipped: edited == 1,
        })
    }

    fn list_protocol_styles(&self) -> Result<Vec<ProtocolStyle>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, description, language_scope, density
             FROM protocol_styles WHERE enabled = 1 ORDER BY id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(ProtocolStyle {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                language: "Meeting language".to_string(),
                density: row
                    .get::<_, String>(4)
                    .ok()
                    .and_then(|value| ProtocolDensity::from_str(&value))
                    .unwrap_or_default(),
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// Every entry, including the switched-off ones.
    ///
    /// The places that *use* vocabulary filter to the enabled entries. The library
    /// deliberately does not: a term that has been switched off still has to be
    /// visible, or there is no way to switch it back on.
    fn list_vocabulary(&self) -> Result<Vec<VocabularyEntry>> {
        let mut statement = self.connection.prepare(
            "SELECT id, term, category, scope, project_id, enabled
             FROM vocabulary_entries
             ORDER BY scope, term COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(VocabularyEntry {
                id: row.get(0)?,
                term: row.get(1)?,
                category: row.get(2)?,
                scope: row.get(3)?,
                project_id: row.get(4)?,
                enabled: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// Record which vocabulary actually shaped a transcript.
    ///
    /// The terms are resolved when the job runs rather than when it is queued, so
    /// the library may have changed in between. Storing what was really sent is
    /// what lets a transcript be explained later — including the case where nothing
    /// was sent at all, which is recorded as such rather than left blank.
    pub(crate) fn record_transcription_vocabulary(
        &self,
        job_id: &str,
        prompt: Option<&str>,
    ) -> Result<()> {
        let revision = match prompt {
            Some(prompt) => format!("sha256:{}", checksum_bytes(prompt.as_bytes())),
            None => "none".to_string(),
        };
        self.connection.execute(
            "UPDATE jobs SET vocabulary_revision = ?2 WHERE id = ?1",
            params![job_id, revision],
        )?;
        Ok(())
    }

    /// Add a term, or change one that already exists.
    ///
    /// A project-scoped term needs a project; a global one must not carry one. The
    /// same term is not stored twice in the same scope, since a duplicate would
    /// only spend part of the runtime's short prompt saying the same thing twice.
    pub fn save_vocabulary_entry(&mut self, input: VocabularyDraft) -> Result<()> {
        let term = required_text(&input.term, 200, "Enter a term.")?;
        let category = required_text(&input.category, 64, "Choose a category.")?;
        let project_id = match input.scope.as_str() {
            "Global" => None,
            "Project" => Some(input.project_id.clone().ok_or(StorageError::InvalidData(
                "Choose the project this term belongs to.",
            ))?),
            _ => return Err(StorageError::InvalidData("Choose a valid scope.")),
        };
        let clash: Option<String> = self
            .connection
            .query_row(
                "SELECT id FROM vocabulary_entries
                 WHERE term = ?1 COLLATE NOCASE
                   AND scope = ?2
                   AND project_id IS ?3
                   AND id IS NOT ?4",
                params![term, input.scope, project_id, input.id],
                |row| row.get(0),
            )
            .optional()?;
        if clash.is_some() {
            return Err(StorageError::InvalidData(
                "That term is already in this vocabulary.",
            ));
        }
        let now = unix_time_millis();
        match &input.id {
            Some(id) => {
                let changed = self.connection.execute(
                    "UPDATE vocabulary_entries
                     SET term = ?2, preferred_spelling = ?2, category = ?3, scope = ?4,
                         project_id = ?5, enabled = ?6, revision = revision + 1,
                         updated_at_ms = ?7
                     WHERE id = ?1",
                    params![
                        id,
                        term,
                        category,
                        input.scope,
                        project_id,
                        input.enabled,
                        now
                    ],
                )?;
                if changed == 0 {
                    return Err(StorageError::InvalidData("That term no longer exists."));
                }
            }
            None => {
                self.connection.execute(
                    "INSERT INTO vocabulary_entries
                        (id, term, preferred_spelling, category, scope, project_id,
                         enabled, revision, updated_at_ms)
                     VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
                    params![
                        new_id("vocabulary"),
                        term,
                        category,
                        input.scope,
                        project_id,
                        input.enabled,
                        now
                    ],
                )?;
            }
        }
        Ok(())
    }

    /// Remove a term outright. Switching it off is the reversible alternative and
    /// is what the library offers first.
    pub fn delete_vocabulary_entry(&mut self, id: &str) -> Result<()> {
        let removed = self
            .connection
            .execute("DELETE FROM vocabulary_entries WHERE id = ?1", [id])?;
        if removed == 0 {
            return Err(StorageError::InvalidData("That term no longer exists."));
        }
        Ok(())
    }

    /// A meeting's vocabulary for transcription, most specific first.
    ///
    /// The runtime accepts roughly 224 tokens, so this order decides what actually
    /// reaches it. Two rules, in this priority:
    ///
    /// 1. A project's own entries before shared ones, because a project's names are
    ///    what a transcriber cannot guess.
    /// 2. Within that, proper nouns before terminology. Measured against a real
    ///    German meeting, every term the vocabulary corrected was a company name or
    ///    a surname, while ordinary professional vocabulary was already transcribed
    ///    correctly with no help at all.
    ///
    /// Categories this build does not know about sort between the two groups: they
    /// may well be names, and demoting them below general terminology would be a
    /// guess in the more damaging direction.
    pub(crate) fn transcription_vocabulary(&self, meeting_id: &str) -> Result<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT v.term
             FROM vocabulary_entries v
             JOIN meetings m ON m.id = ?1
             WHERE v.enabled = 1
               AND (v.project_id = m.project_id OR v.scope = 'Global')
             ORDER BY
               CASE WHEN v.project_id = m.project_id THEN 0 ELSE 1 END,
               CASE v.category
                 WHEN 'Person' THEN 0
                 WHEN 'Organisation' THEN 1
                 WHEN 'Project' THEN 2
                 WHEN 'Abbreviation' THEN 3
                 WHEN 'Technical term' THEN 5
                 WHEN 'Other' THEN 6
                 ELSE 4
               END,
               v.term COLLATE NOCASE",
        )?;
        let rows = statement.query_map([meeting_id], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    fn list_vocabulary_for_project(&self, project_id: &str) -> Result<Vec<VocabularyEntry>> {
        let mut statement = self.connection.prepare(
            "SELECT id, term, category, scope, project_id
             FROM vocabulary_entries
             WHERE enabled = 1 AND (scope = 'Global' OR project_id = ?1)
             ORDER BY scope, term COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([project_id], |row| {
            Ok(VocabularyEntry {
                id: row.get(0)?,
                term: row.get(1)?,
                category: row.get(2)?,
                scope: row.get(3)?,
                project_id: row.get(4)?,
                enabled: true,
            })
        })?;
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
                m.style_id,
                (
                    SELECT nm.duration_ms FROM normalized_media nm
                    JOIN recordings r ON r.id = nm.recording_id
                    WHERE r.meeting_id = m.id
                    ORDER BY r.created_at_ms, r.id LIMIT 1
                )
             FROM meetings m
             WHERE m.archived_at_ms IS NULL
             ORDER BY m.occurred_at DESC, m.created_at_ms DESC, m.id",
        )?;
        let rows = statement.query_map([], meeting_from_row)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// Set what repeats at the top and bottom of this project's printed pages.
    pub fn set_project_furniture(
        &self,
        project_id: &str,
        furniture: &PageFurniture,
    ) -> Result<()> {
        let json = serde_json::to_string(furniture)
            .map_err(|_| StorageError::InvalidData("The header and footer could not be stored."))?;
        let changed = self.connection.execute(
            "UPDATE projects SET furniture_json = ?1, updated_at_ms = ?2
             WHERE id = ?3 AND archived_at_ms IS NULL",
            params![json, unix_time_millis(), project_id],
        )?;
        if changed == 0 {
            return Err(StorageError::MissingProject);
        }
        Ok(())
    }

    /// Set how this project's protocols are set.
    pub fn set_project_appearance(
        &self,
        project_id: &str,
        appearance: &DocumentAppearance,
    ) -> Result<()> {
        let json = serde_json::to_string(appearance)
            .map_err(|_| StorageError::InvalidData("The appearance could not be stored."))?;
        let changed = self.connection.execute(
            "UPDATE projects SET appearance_json = ?1, updated_at_ms = ?2
             WHERE id = ?3 AND archived_at_ms IS NULL",
            params![json, unix_time_millis(), project_id],
        )?;
        if changed == 0 {
            return Err(StorageError::MissingProject);
        }
        Ok(())
    }

    fn project_by_id(&self, id: &str) -> Result<Option<ProjectSummary>> {
        Ok(self
            .connection
            .query_row(
                "SELECT
                    p.id, p.name, p.description, p.default_language, p.default_style_id,
                    COUNT(m.id), p.appearance_json, p.furniture_json
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
                    m.style_id,
                    (
                        SELECT nm.duration_ms FROM normalized_media nm
                        JOIN recordings r ON r.id = nm.recording_id
                        WHERE r.meeting_id = m.id
                        ORDER BY r.created_at_ms, r.id LIMIT 1
                    )
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
                    speaker_resolution: artifact.speaker_resolution,
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
            let revision_for_evidence = base_revision_id.clone();
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
                    evidence: self.protocol_evidence(&revision_for_evidence)?,
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

/// Whether a column is already present.
///
/// A migration that adds one must be able to run twice. The version is recorded
/// separately from the change it describes, so a process that stops between the
/// two leaves a database whose schema has moved and whose version has not — and
/// the next start then fails on `duplicate column name` and cannot open the
/// workspace at all. Asking first costs nothing and makes the step repeatable.
fn has_column(connection: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        if row.get::<_, String>(1)? == column {
            return Ok(true);
        }
    }
    Ok(false)
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
        version = 4;
    }
    if version == 4 {
        connection.execute_batch(
            "
            BEGIN IMMEDIATE;
            CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );
            CREATE TABLE normalized_media (
                recording_id TEXT PRIMARY KEY REFERENCES recordings(id),
                source_checksum TEXT NOT NULL,
                normalized_path TEXT NOT NULL,
                normalized_checksum TEXT NOT NULL,
                byte_count INTEGER NOT NULL CHECK (byte_count > 0),
                duration_ms INTEGER,
                audio_codec TEXT,
                sample_rate INTEGER,
                channels INTEGER,
                runtime_version TEXT NOT NULL,
                settings_json TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL
            );
            PRAGMA user_version = 5;
            COMMIT;
            ",
        )?;
        version = 5;
    }
    if version == 5 {
        connection.execute_batch(
            "
            BEGIN IMMEDIATE;
            ALTER TABLE jobs ADD COLUMN runtime_config_json TEXT;
            PRAGMA user_version = 6;
            COMMIT;
            ",
        )?;
        version = 6;
    }
    if version == 6 {
        connection.execute_batch(
            "
            BEGIN IMMEDIATE;
            CREATE TABLE model_provenance_cache (
                model_path TEXT PRIMARY KEY,
                byte_count INTEGER NOT NULL CHECK (byte_count >= 0),
                modified_at_ns TEXT NOT NULL,
                digest TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );
            PRAGMA user_version = 7;
            COMMIT;
            ",
        )?;
        version = 7;
    }
    if version == 7 {
        connection.execute_batch(
            r#"
            BEGIN IMMEDIATE;
            CREATE TABLE protocol_styles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                language_scope TEXT NOT NULL DEFAULT 'meeting',
                instructions_json TEXT NOT NULL,
                required_sections_json TEXT NOT NULL,
                revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
                enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
                updated_at_ms INTEGER NOT NULL
            );
            CREATE TABLE vocabulary_entries (
                id TEXT PRIMARY KEY,
                term TEXT NOT NULL,
                preferred_spelling TEXT NOT NULL,
                category TEXT NOT NULL,
                aliases_json TEXT NOT NULL DEFAULT '[]',
                note TEXT NOT NULL DEFAULT '',
                scope TEXT NOT NULL CHECK (scope IN ('Global', 'Project')),
                project_id TEXT REFERENCES projects(id),
                enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
                revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
                updated_at_ms INTEGER NOT NULL,
                CHECK ((scope = 'Global' AND project_id IS NULL) OR
                       (scope = 'Project' AND project_id IS NOT NULL))
            );
            CREATE INDEX vocabulary_entries_scope
                ON vocabulary_entries(scope, project_id, enabled, term);
            INSERT INTO protocol_styles
                (id, name, description, language_scope, instructions_json,
                 required_sections_json, revision, updated_at_ms)
            VALUES
                ('style-formal', 'Formal minutes',
                 'Structured record of discussion, decisions, and actions.', 'meeting',
                 '["Write the entire protocol in the meeting''s language.", "Organise the protocol by topic, not in the order things were discussed. Gather everything said about one subject into a single numbered section, even if it came up several times.", "Begin with the participants, grouped by the organisation they belong to, and give a role only where it was stated.", "Use numbered sections with descriptive headings, and sub-numbered subsections where a topic has distinct parts.", "Write discussion as calm, factual prose. Use lists only for options, criteria, and open questions.", "Reproduce every number, measurement, area, date, and proper name exactly as stated. Never round or approximate them.", "Separate what was decided from what remains open. Where no decision was reached, say so plainly rather than implying one.", "Mark uncertainty in the words the meeting used, such as an intention, an estimate, or a matter still to be confirmed.", "End with a table of agreed next steps with two columns, the task and the responsible party, followed by a short section for dates and appointments.", "Never invent a decision, an action, an owner, or a date. If the source does not say who is responsible, leave it unattributed.", "Cover every topic that was discussed. A protocol that silently omits a topic is incomplete, even if what remains reads well.", "The table of next steps must list every action that was agreed, not a selection of the clearest ones.", "Write at whatever length the material requires. Do not compress the meeting into a summary: this is a record, and a reader who was absent must be able to follow what was discussed and what follows from it.", "Never leave a placeholder such as [Datum] or [Details]. If something is not in the source, omit the line instead."]',
                 '["Summary","Decisions","Actions","Open questions"]', 1, 0),
                ('style-working-note', 'Internal working note',
                 'Concise working record for an internal project team.', 'meeting',
                 '["Write a concise internal working note.","Preserve useful context and mark uncertainty explicitly.","Do not invent facts that are not in the transcript."]',
                 '["Summary","Discussion","Next steps"]', 1, 0),
                ('style-decision-log', 'Technical decision log',
                 'Emphasises alternatives, constraints, and explicit decisions.', 'meeting',
                 '["Write a precise technical decision record.","Make alternatives, constraints, and consequences visible.","Record only decisions supported by the transcript."]',
                 '["Context","Options considered","Decision","Consequences","Open questions"]', 1, 0);
            PRAGMA user_version = 8;
            COMMIT;
            "#,
        )?;
        version = 8;
    }
    if version == 8 {
        // The formal-minutes style began as three sentences, which produced a
        // protocol a quarter the length of a human one. These instructions were
        // derived from a real professional protocol and measurably changed the
        // result. A style the user has edited is left alone.
        connection.execute_batch(
            r#"
            BEGIN IMMEDIATE;
            UPDATE protocol_styles
               SET instructions_json = '["Write the entire protocol in the meeting''s language.", "Organise the protocol by topic, not in the order things were discussed. Gather everything said about one subject into a single numbered section, even if it came up several times.", "Begin with the participants, grouped by the organisation they belong to, and give a role only where it was stated.", "Use numbered sections with descriptive headings, and sub-numbered subsections where a topic has distinct parts.", "Write discussion as calm, factual prose. Use lists only for options, criteria, and open questions.", "Reproduce every number, measurement, area, date, and proper name exactly as stated. Never round or approximate them.", "Separate what was decided from what remains open. Where no decision was reached, say so plainly rather than implying one.", "Mark uncertainty in the words the meeting used, such as an intention, an estimate, or a matter still to be confirmed.", "End with a table of agreed next steps with two columns, the task and the responsible party, followed by a short section for dates and appointments.", "Never invent a decision, an action, an owner, or a date. If the source does not say who is responsible, leave it unattributed.", "Cover every topic that was discussed. A protocol that silently omits a topic is incomplete, even if what remains reads well.", "The table of next steps must list every action that was agreed, not a selection of the clearest ones.", "Write at whatever length the material requires. Do not compress the meeting into a summary: this is a record, and a reader who was absent must be able to follow what was discussed and what follows from it.", "Never leave a placeholder such as [Datum] or [Details]. If something is not in the source, omit the line instead."]',
                   revision = revision + 1
             WHERE id = 'style-formal' AND revision = 1;
            PRAGMA user_version = 9;
            COMMIT;
            "#,
        )?;
        version = 9;
    }
    if version == 9 {
        // What a completed job found out about its own result. The first use is
        // how many of the quantities the meeting stated survived into the
        // protocol, which is a measure of quality that needs no reader.
        if !has_column(connection, "jobs", "outcome_json")? {
            connection.execute("ALTER TABLE jobs ADD COLUMN outcome_json TEXT", [])?;
        }
        connection.pragma_update(None, "user_version", 10)?;
        version = 10;
    }
    if version == 10 {
        // How much room a style spends saying a thing, never which things it says.
        // Measured on one meeting for which a person wrote three documents: the
        // shortest is half the length of the longest, keeps 25 of its 30 topics,
        // and carries more bullets than it -- so what compression removed was
        // prose, not content.
        if !has_column(connection, "protocol_styles", "density")? {
            connection.execute_batch(
                r#"
                ALTER TABLE protocol_styles ADD COLUMN density TEXT NOT NULL DEFAULT 'concise'
                    CHECK (density IN ('comprehensive', 'concise', 'terse'));
                UPDATE protocol_styles SET density = 'comprehensive' WHERE id = 'style-formal';
                UPDATE protocol_styles SET density = 'terse' WHERE id = 'style-decision-log';
                "#,
            )?;
        }
        connection.pragma_update(None, "user_version", 11)?;
        version = 11;
    }
    if version == 11 {
        // What somebody trimmed from a recording, kept beside the meeting rather
        // than applied to the file. One small document per meeting, because the
        // trims and the removals are read and written together and never queried
        // apart, and because the recording they describe must stay whole.
        if !has_column(connection, "meetings", "recording_edits_json")? {
            connection.execute(
                "ALTER TABLE meetings ADD COLUMN recording_edits_json TEXT",
                [],
            )?;
        }
        connection.pragma_update(None, "user_version", 12)?;
        version = 12;
    }
    if version == 12 {
        // A recording made inside LocaLog is neither an import nor a single track,
        // and while it is being made it is in neither of the states an import can be
        // in. The table said otherwise, so starting a recording failed against its
        // own CHECK constraints and the person was told the workspace was
        // unreachable. SQLite cannot alter a constraint, so the table is rebuilt.
        //
        // Foreign keys are turned off around the rebuild because `jobs` points at
        // this table, and they are turned off outside the transaction because SQLite
        // ignores the pragma inside one.
        let widened = recordings_allows(connection, "recorded")?;
        if !widened {
            connection.pragma_update(None, "foreign_keys", "OFF")?;
            let rebuild = connection.execute_batch(
                r#"
                BEGIN IMMEDIATE;
                CREATE TABLE recordings_widened (
                    id TEXT PRIMARY KEY,
                    meeting_id TEXT NOT NULL REFERENCES meetings(id),
                    kind TEXT NOT NULL
                        CHECK (kind IN ('imported', 'microphone', 'system_audio', 'recorded')),
                    original_name TEXT NOT NULL,
                    state TEXT NOT NULL
                        CHECK (state IN ('pending', 'recording', 'committed', 'failed')),
                    managed_path TEXT,
                    checksum TEXT,
                    byte_count INTEGER CHECK (byte_count IS NULL OR byte_count >= 0),
                    created_at_ms INTEGER NOT NULL,
                    media_type TEXT
                );
                INSERT INTO recordings_widened
                    SELECT id, meeting_id, kind, original_name, state, managed_path,
                           checksum, byte_count, created_at_ms, media_type
                      FROM recordings;
                DROP TABLE recordings;
                ALTER TABLE recordings_widened RENAME TO recordings;
                CREATE INDEX recordings_meeting ON recordings(meeting_id);
                COMMIT;
                "#,
            );
            connection.pragma_update(None, "foreign_keys", "ON")?;
            rebuild?;
        }
        connection.pragma_update(None, "user_version", 13)?;
        version = 13;
    }
    if version == 13 {
        // Repairing the migration above. Its first version listed the columns from
        // the table's original definition and so rebuilt it without `media_type`,
        // which a later migration had added -- and every query that reads a meeting
        // asks for that column, so the workspace stopped opening entirely. The value
        // is written beside the job that produced the recording, so what was dropped
        // can be put back rather than merely re-declared.
        if !has_column(connection, "recordings", "media_type")? {
            connection.execute_batch(
                r#"
                BEGIN IMMEDIATE;
                ALTER TABLE recordings ADD COLUMN media_type TEXT;
                UPDATE recordings
                   SET media_type = (
                        SELECT j.result_media_type FROM jobs j
                         WHERE j.recording_id = recordings.id
                           AND j.kind = 'import'
                           AND j.result_media_type IS NOT NULL
                         ORDER BY j.created_at_ms DESC LIMIT 1
                   )
                 WHERE media_type IS NULL;
                COMMIT;
                "#,
            )?;
        }
        connection.pragma_update(None, "user_version", 14)?;
        version = 14;
    }
    if version == 14 {
        // Repairing the repair. The backfill above first took the newest job of any
        // kind, so a recording that had since been transcribed and written up was
        // restored as `text/markdown` -- the media type of the protocol, not of the
        // audio. Only the import job carries the media type of the file that arrived.
        // Written as an overwrite rather than a fill so that a database which already
        // took the wrong value is corrected rather than left holding it.
        connection.execute(
            "UPDATE recordings
                SET media_type = (
                     SELECT j.result_media_type FROM jobs j
                      WHERE j.recording_id = recordings.id
                        AND j.kind = 'import'
                        AND j.result_media_type IS NOT NULL
                      ORDER BY j.created_at_ms DESC LIMIT 1
                )
              WHERE EXISTS (
                     SELECT 1 FROM jobs j
                      WHERE j.recording_id = recordings.id
                        AND j.kind = 'import'
                        AND j.result_media_type IS NOT NULL
              )",
            [],
        )?;
        connection.pragma_update(None, "user_version", 15)?;
        version = 15;
    }
    if version == 15 {
        // How a project's protocols are set, as opposed to what they say. Held by
        // the project because the reason anybody sets it is that a firm's documents
        // should look alike; absent means the defaults, so nothing needs writing
        // for projects that never touch it.
        if !has_column(connection, "projects", "appearance_json")? {
            connection.execute("ALTER TABLE projects ADD COLUMN appearance_json TEXT", [])?;
        }
        connection.pragma_update(None, "user_version", 16)?;
        version = 16;
    }
    if version == 16 {
        // What repeats at the top and bottom of every printed page. Beside the
        // appearance and for the same reason: it is the firm's, not the meeting's.
        if !has_column(connection, "projects", "furniture_json")? {
            connection.execute("ALTER TABLE projects ADD COLUMN furniture_json TEXT", [])?;
        }
        connection.pragma_update(None, "user_version", 17)?;
        version = 17;
    }
    if version == 17 {
        // Sections somebody took out of a protocol without wanting them gone. Kept
        // beside the working draft rather than inside it, because the document has
        // to remain exactly what every export produces — a section that is in the
        // file but hidden from the page would make the screen and the PDF differ,
        // which is the one thing this editor is built not to do.
        if !has_column(connection, "protocol_working", "set_aside_json")? {
            connection.execute(
                "ALTER TABLE protocol_working ADD COLUMN set_aside_json TEXT",
                [],
            )?;
        }
        connection.pragma_update(None, "user_version", 18)?;
        version = 18;
    }
    if version == 18 {
        // A saved way of presenting a protocol: the typography and the running
        // header and footer together, named and reusable.
        //
        // Deliberately not the protocol style, which decides what the document says.
        // A template decides how it is set, which is why it holds exactly the two
        // things a project already holds and nothing about content.
        connection.execute_batch(
            r#"
            BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS export_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                appearance_json TEXT NOT NULL,
                furniture_json TEXT NOT NULL,
                built_in INTEGER NOT NULL DEFAULT 0 CHECK (built_in IN (0, 1)),
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );
            COMMIT;
            "#,
        )?;
        seed_export_templates(connection)?;
        connection.pragma_update(None, "user_version", 19)?;
    }
    Ok(())
}

/// The two a firm actually has: what goes to a client, and what stays inside.
///
/// Written only when the table is empty, so a workspace that has had them deleted
/// does not grow them back every time it opens.
fn seed_export_templates(connection: &Connection) -> Result<()> {
    let existing: i64 =
        connection.query_row("SELECT COUNT(*) FROM export_templates", [], |row| row.get(0))?;
    if existing > 0 {
        return Ok(());
    }

    let client = DocumentAppearance::default();
    let client_furniture = PageFurniture {
        header: FurnitureRow {
            left: vec![FurnitureField::ProjectName],
            centre: Vec::new(),
            right: vec![FurnitureField::MeetingDate],
        },
        footer: FurnitureRow {
            left: vec![FurnitureField::DocumentType],
            centre: Vec::new(),
            right: vec![FurnitureField::PageOfCount],
        },
        skip_first_page: false,
    };
    let internal = DocumentAppearance {
        body_size: 10,
        heading_scale: Scale::Compact,
        line_spacing: Spacing::Compact,
        page_width: PageWidth::Standard,
        ..DocumentAppearance::default()
    };

    let now = unix_time_millis();
    for (id, name, description, appearance, furniture) in [
        (
            "template-client",
            "Client protocol",
            "A4 with the project and the date at the top and a page count at the foot.",
            client,
            client_furniture,
        ),
        (
            "template-internal",
            "Internal note",
            "Smaller and tighter, with nothing repeated on the page.",
            internal,
            PageFurniture::default(),
        ),
    ] {
        connection.execute(
            "INSERT INTO export_templates
                (id, name, description, appearance_json, furniture_json, built_in,
                 created_at_ms, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
            params![
                id,
                name,
                description,
                serde_json::to_string(&appearance).unwrap_or_default(),
                serde_json::to_string(&furniture).unwrap_or_default(),
                now
            ],
        )?;
    }
    Ok(())
}

/// Whether the recordings table already accepts a value for `kind`.
///
/// Read from the stored DDL rather than attempted with a write, so that a database
/// already carrying the wider constraint is left untouched.
fn recordings_allows(connection: &Connection, kind: &str) -> Result<bool> {
    let ddl: Option<String> = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'recordings'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(ddl.map(|sql| sql.contains(kind)).unwrap_or(false))
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
        // Unset, or unreadable because it was written by a newer version, both mean
        // the defaults: an appearance nobody can read is not worth failing a whole
        // workspace over.
        appearance: row
            .get::<_, Option<String>>(6)
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default(),
        furniture: row
            .get::<_, Option<String>>(7)
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default(),
    })
}

fn protocol_style_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResolvedProtocolStyle> {
    let instructions_json: String = row.get(4)?;
    let required_sections_json: String = row.get(5)?;
    let instructions = serde_json::from_str(&instructions_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let required_sections = serde_json::from_str(&required_sections_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let id: String = row.get(0)?;
    let revision: i64 = row.get(6)?;
    Ok(ResolvedProtocolStyle {
        id,
        revision: format!("{}@{}", row.get::<_, String>(0)?, revision),
        instructions,
        required_sections,
        density: row
            .get::<_, String>(7)
            .ok()
            .and_then(|value| ProtocolDensity::from_str(&value))
            .unwrap_or_default(),
    })
}

fn duration_label_from_ms(duration_ms: i64) -> String {
    let total_seconds = (duration_ms.max(0) / 1000) as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours} h {minutes:02} min")
    } else if minutes > 0 {
        format!("{minutes} min")
    } else {
        format!("{total_seconds} s")
    }
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
    // Probed duration becomes visible once working audio exists; a stored label wins.
    let stored_label: Option<String> = row.get(4)?;
    let duration_ms: Option<i64> = row.get(11)?;
    Ok(MeetingSummary {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        occurred_at: row.get(3)?,
        duration_label: stored_label.or_else(|| duration_ms.map(duration_label_from_ms)),
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

/// What a person is told the application is doing.
///
/// These are read by someone waiting, not by someone reading the code, so they
/// are written in the words that person would use. A revision, a snapshot and a
/// committed source are real things in here and mean nothing out there; a line
/// that says "validating the transcript revision" has described the machine to
/// somebody who wanted to know about their meeting.
///
/// Reassurance is not repeated, and not only because it crowds out the one thing
/// the reader did not already know. Saying a thing that is always true invites the
/// reader to wonder when it might not be: a line that says work is happening
/// locally implies that somewhere there is a run that would not, and the promise
/// starts manufacturing the doubt it was meant to answer. That the work is local
/// and that an imported file leaves the original alone belong in the interface
/// once, stated plainly, where they can be trusted rather than repeated.
///
/// Failure is the exception. When something has gone wrong, that the original is
/// untouched stops being a boast and becomes the answer to the question being
/// asked.
fn job_stage_label(kind: &str, stage: &str, state: JobState) -> String {
    if stage == "completed" {
        return match kind {
            "transcription" => "Transcript saved".to_string(),
            "generation" => "Protocol saved".to_string(),
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
    // A stage may carry a live detail after a colon, so that a step lasting minutes
    // can say where it has got to instead of showing the same words throughout.
    let (stage, detail) = match stage.split_once(':') {
        Some((code, detail)) => (code, Some(detail)),
        None => (stage, None),
    };
    match (stage, state) {
        ("finding_subjects", _) => match detail {
            Some(detail) => format!("Finding what was discussed — passage {detail}"),
            None => "Finding what was discussed".to_string(),
        },
        ("writing_subject", _) => match detail {
            Some(detail) => format!("Writing {detail}"),
            None => "Writing the protocol subject by subject".to_string(),
        },
        ("assembling_protocol", _) => "Putting the sections together".to_string(),
        ("joining_failed", _) => match detail {
            Some(detail) => format!("Subjects could not be joined — {detail}"),
            None => "Subjects could not be joined".to_string(),
        },
        ("joined_subjects", _) => match detail {
            Some(detail) => format!("Joined subjects — {detail}"),
            None => "Joined subjects".to_string(),
        },
        ("joining_subjects", _) => match detail {
            Some(detail) => format!("Joining subjects that belong together — {detail} found"),
            None => "Joining subjects that belong together".to_string(),
        },
        ("ready_to_import", _) => "Ready to bring the recording in".to_string(),
        ("copying", JobState::Cancelling) => "Stopping safely".to_string(),
        ("copying", _) => "Bringing the recording in".to_string(),
        ("validating", _) => "Checking the copy is complete".to_string(),
        ("temporary_complete", _) => "Nearly there".to_string(),
        ("finalizing", _) => "Putting the recording away safely".to_string(),
        ("duplicate_confirmation", _) => "This recording may already be here".to_string(),
        ("completed", _) => "Recording is in".to_string(),
        ("cancelled", _) => "Import cancelled — original unchanged".to_string(),
        ("interrupted", _) => "Import was interrupted — original unchanged".to_string(),
        ("failed", _) => "Import could not finish — original unchanged".to_string(),
        ("transcription_queued", _) => "Ready to transcribe".to_string(),
        ("checking_source", _) => "Checking the recording".to_string(),
        ("preparing_fake_transcriber", _) => "Getting ready".to_string(),
        ("transcribing_synthetic_segments", _) => "Creating transcript segments".to_string(),
        ("validating_transcript", _) => "Saving the transcript".to_string(),
        ("generation_queued", _) => "Ready to write the protocol".to_string(),
        ("checking_transcript", _) => "Checking the transcript".to_string(),
        ("resolving_protocol_inputs", _) => "Gathering the style and the vocabulary".to_string(),
        ("generating_protocol", _) => "Writing the protocol draft".to_string(),
        ("validating_protocol", _) => "Saving the protocol".to_string(),
        ("output_staged", _) => "Saving safely".to_string(),
        // Real transcription stages; without these every step read "Preparing local import".
        ("probing_media", _) => "Looking at the recording".to_string(),
        ("normalizing_audio", _) => "Preparing the audio".to_string(),
        ("loading_transcription_model", _) => "Loading the model".to_string(),
        ("transcribing_audio", _) => "Transcribing".to_string(),
        // A meeting longer than the model's window is condensed section by section first.
        ("condensing_transcript", _) => "Reading the meeting through".to_string(),
        ("separating_speakers", _) => "Telling the speakers apart".to_string(),
        _ => "Working".to_string(),
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
        "runtime_missing" => (
            "Choose a local transcription runtime",
            "Select an installed whisper.cpp executable in Settings → Transcription. LocaLog does not download runtimes.",
        ),
        "model_missing" => (
            "Choose a local transcription model",
            "Select an already available whisper.cpp model in Settings → Transcription. No model was downloaded or changed.",
        ),
        "runtime_changed" => (
            "The transcription runtime changed",
            "The queued job was not run because its whisper.cpp executable no longer matches the recorded runtime. Retry to resolve the current runtime.",
        ),
        "model_changed" => (
            "The transcription model changed",
            "The queued job was not run because its model no longer matches the recorded checksum. Retry to resolve the current model.",
        ),
        "media_probe_failed" => (
            "The recording could not be inspected",
            "Check that FFprobe is installed and that the imported source is still readable. The original remains unchanged.",
        ),
        "normalization_failed" => (
            "The recording could not be prepared",
            "Check that FFmpeg is installed and retry. The normalized cache can be regenerated and the original remains unchanged.",
        ),
        "transcription_failed" => (
            "Local transcription could not finish",
            "The whisper.cpp runtime stopped before a transcript revision was committed. Check its model and retry.",
        ),
        "transcription_timeout" => (
            "Local transcription took too long",
            "The supervised transcription process was stopped before a transcript revision was committed. Check the recording and runtime, then retry.",
        ),
        "provider_model_missing" => (
            "The selected local model is unavailable",
            "The selected Ollama model is no longer installed. Choose an installed model in Settings → Protocol generation, then retry.",
        ),
        "provider_model_changed" => (
            "The local model changed",
            "The model digest changed after this job was queued. Retry to capture the current installed model.",
        ),
        "provider_runtime_changed" => (
            "The local provider changed",
            "The Ollama runtime version changed after this job was queued. Retry to capture the current runtime.",
        ),
        "provider_unavailable" => (
            "Local protocol generation could not connect",
            "Start your existing Ollama installation and retry. LocaLog does not start or download runtimes.",
        ),
        "provider_invalid_output" | "provider_incomplete_output" => (
            "The local model output could not be validated",
            "LocaLog did not commit the incomplete or malformed protocol. Your transcript remains safe and you can retry.",
        ),
        "provider_response_too_large" => (
            "The local model response was too large",
            "The response exceeded LocaLog’s safe limit and was not committed. Try again with a shorter transcript or a different local model.",
        ),
        "invalid_transcript_output" => (
            "The transcription output could not be validated",
            "LocaLog did not commit the runtime output because it was incomplete or malformed. Your source remains safe.",
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
    fn the_formal_style_carries_real_instructions() {
        let temporary = tempdir().unwrap();
        let repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let style = repository.protocol_inputs_style("style-formal");
        assert!(
            style.instructions.len() >= 10,
            "the shipped style should be a real specification, not a sentence: {} instructions",
            style.instructions.len()
        );
        let joined = style.instructions.join(" ");
        // The properties a protocol of this kind depends on.
        assert!(joined.contains("by topic"), "must organise by topic");
        assert!(joined.contains("next steps"), "must end in an action table");
        assert!(joined.contains("Never invent"), "must forbid invention");
        assert!(joined.contains("placeholder"), "must forbid placeholders");
    }

    /// Starting a recording, against the real table rather than against a reading of
    /// it.
    ///
    /// The statement and the constraint were written weeks apart and disagreed: a
    /// recording made inside LocaLog is a fourth kind and passes through a fifth
    /// state, and the table admitted neither. Every attempt to record failed, and
    /// because a constraint failure is an SQLite error like any other, the person was
    /// told LocaLog could not reach its own workspace. A test that reads the source
    /// cannot see this; only the database can.
    #[test]
    fn a_recording_can_be_started_in_a_real_workspace() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let mut repository = WorkspaceRepository::open(root).unwrap();
        let (_, meeting_id) = project_with_meeting(&mut repository, root);

        repository
            .connection
            .execute(
                crate::processing::START_RECORDING_ROW,
                rusqlite::params!["recording-live", meeting_id, "recording-live-system.wav", 1],
            )
            .expect("the table must accept a recording that is being made");

        // And it must still be reachable by the update that finishes it.
        let finished = repository
            .connection
            .execute(
                "UPDATE recordings SET state = 'committed' WHERE id = ?1",
                ["recording-live"],
            )
            .unwrap();
        assert_eq!(finished, 1, "a started recording must be finishable");
    }

    /// A project and a meeting, since vocabulary is always resolved through one.
    fn project_with_meeting(repository: &mut WorkspaceRepository, root: &Path) -> (String, String) {
        let project = repository
            .create_project(NewProjectInput {
                name: "Beispielquartier".to_string(),
                description: String::new(),
                default_language: "German".to_string(),
            })
            .unwrap();
        let source = root.join("vocabulary-fixture.wav");
        fs::write(&source, b"synthetic").unwrap();
        let meeting = repository
            .create_meeting(NewMeetingInput {
                project_id: project.id.clone(),
                title: "Jour fixe".to_string(),
                occurred_at: "2026-08-06".to_string(),
                language: "German".to_string(),
                source_name: "vocabulary-fixture.wav".to_string(),
                source_path: Some(source.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();
        (project.id, meeting.id)
    }

    /// A completed generation job with an outcome recorded against it, using the
    /// meeting's own recording so the foreign keys hold.
    fn record_generation_outcome(
        repository: &WorkspaceRepository,
        meeting_id: &str,
        job_id: &str,
        revision_id: &str,
        outcome: &str,
    ) {
        let recording_id: String = repository
            .connection
            .query_row(
                "SELECT id FROM recordings WHERE meeting_id = ?1 LIMIT 1",
                [meeting_id],
                |row| row.get(0),
            )
            .unwrap();
        repository
            .connection
            .execute(
                "INSERT INTO jobs
                    (id, meeting_id, recording_id, kind, state, stage, progress_bytes, attempt,
                     duplicate_allowed, result_revision_id, outcome_json, created_at_ms,
                     updated_at_ms)
                 VALUES (?1, ?2, ?3, 'generation', 'completed', 'completed', 0, 1, 0, ?4, ?5, 1, 1)",
                params![job_id, meeting_id, recording_id, revision_id, outcome],
            )
            .unwrap();
    }

    fn draft(term: &str, category: &str, project_id: Option<&str>) -> VocabularyDraft {
        VocabularyDraft {
            id: None,
            term: term.to_string(),
            category: category.to_string(),
            scope: if project_id.is_some() {
                "Project"
            } else {
                "Global"
            }
            .to_string(),
            project_id: project_id.map(str::to_string),
            enabled: true,
        }
    }

    #[test]
    fn a_transcript_records_the_vocabulary_that_shaped_it() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let (_, meeting_id) = project_with_meeting(&mut repository, temporary.path());
        let job_id: String = repository
            .connection
            .query_row(
                "SELECT id FROM jobs WHERE meeting_id = ?1 LIMIT 1",
                [&meeting_id],
                |row| row.get(0),
            )
            .unwrap();

        let recorded = |repository: &WorkspaceRepository| -> Option<String> {
            repository
                .connection
                .query_row(
                    "SELECT vocabulary_revision FROM jobs WHERE id = ?1",
                    [&job_id],
                    |row| row.get(0),
                )
                .unwrap()
        };

        repository
            .record_transcription_vocabulary(&job_id, Some("NORVEK, Mustermann"))
            .unwrap();
        let first = recorded(&repository).expect("a revision is recorded");
        assert!(first.starts_with("sha256:"));

        // A different vocabulary must be distinguishable from the first.
        repository
            .record_transcription_vocabulary(&job_id, Some("NORVEK"))
            .unwrap();
        assert_ne!(recorded(&repository).unwrap(), first);

        // Having sent nothing is itself worth knowing, and is not the same as
        // never having asked.
        repository
            .record_transcription_vocabulary(&job_id, None)
            .unwrap();
        assert_eq!(recorded(&repository).as_deref(), Some("none"));
    }

    /// Density is what a style asks for, so the styles that ship must actually
    /// carry different ones — otherwise the field is decoration.
    #[test]
    fn the_shipped_styles_ask_for_different_amounts_of_prose() {
        let temporary = tempdir().unwrap();
        let repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let styles = repository.workspace_snapshot().unwrap().styles;
        let density = |id: &str| {
            styles
                .iter()
                .find(|style| style.id == id)
                .map(|style| style.density)
        };
        assert_eq!(
            density("style-formal"),
            Some(ProtocolDensity::Comprehensive)
        );
        assert_eq!(
            density("style-working-note"),
            Some(ProtocolDensity::Concise)
        );
        assert_eq!(density("style-decision-log"), Some(ProtocolDensity::Terse));
    }

    /// A step that runs for minutes has to say where it has got to. The status
    /// line is the only place the reader is told anything at all while work is
    /// happening, so a stage may carry a live detail and the label must use it.
    #[test]
    fn a_stage_can_report_where_it_has_got_to() {
        let running = JobState::Running;
        assert_eq!(
            job_stage_label("generation", "finding_subjects:3 of 13", running),
            "Finding what was discussed — passage 3 of 13"
        );
        assert_eq!(
            job_stage_label("generation", "joining_subjects:41", running),
            "Joining subjects that belong together — 41 found"
        );
        // Without a detail it still reads as a sentence rather than a code.
        assert_eq!(
            job_stage_label("generation", "finding_subjects", running),
            "Finding what was discussed"
        );
        // Stages that carry no detail are untouched.
        assert_eq!(
            job_stage_label("transcription", "transcribing_audio", running),
            "Transcribing"
        );
    }

    /// A migration must survive being interrupted between changing the schema and
    /// recording that it did. Stopping in that gap left a real workspace with the
    /// columns of two migrations and the version of neither, and every start after
    /// that failed on "duplicate column name" without opening the workspace at all.
    #[test]
    fn a_migration_interrupted_before_it_was_recorded_can_be_run_again() {
        let temporary = tempdir().unwrap();
        WorkspaceRepository::open(temporary.path()).unwrap();
        let path = temporary.path().join("localog.sqlite3");

        // Exactly the state observed: the schema has moved, the version has not.
        let connection = Connection::open(&path).unwrap();
        connection.pragma_update(None, "user_version", 9).unwrap();
        drop(connection);

        // Opening again must migrate rather than fail on what is already there.
        let repository = WorkspaceRepository::open(temporary.path())
            .expect("a half-applied migration must not lock the user out of their work");
        let version: i64 = repository
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
        assert!(
            has_column(&repository.connection, "jobs", "outcome_json").unwrap(),
            "the column must survive, not be added twice or lost"
        );
        assert_eq!(repository.workspace_snapshot().unwrap().styles.len(), 3);
    }

    /// A draft carries the evidence of the run that produced it, not of the most
    /// recent run, so that reading an older revision does not show numbers taken
    /// from a newer one.
    #[test]
    fn a_protocol_carries_what_its_own_run_found() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let (_, meeting_id) = project_with_meeting(&mut repository, temporary.path());
        record_generation_outcome(
            &repository,
            &meeting_id,
            "job-1",
            "rev-1",
            r#"{"quantitiesStated":24,"quantitiesAccounted":15,
                "quantitiesInvented":["90 cm"],"charactersSpoken":73159,
                "charactersWritten":23193}"#,
        );

        let evidence = repository.protocol_evidence("rev-1").unwrap().unwrap();
        assert_eq!(evidence.quantities_stated, 24);
        assert_eq!(evidence.quantities_accounted, 15);
        assert_eq!(evidence.quantities_invented, vec!["90 cm".to_string()]);
        assert_eq!(evidence.characters_written, 23_193);

        // A revision nothing recorded against is simply without evidence.
        assert!(repository.protocol_evidence("rev-2").unwrap().is_none());
    }

    /// Evidence that cannot be read must never stop a person opening their work.
    #[test]
    fn unreadable_evidence_is_absent_rather_than_fatal() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let (_, meeting_id) = project_with_meeting(&mut repository, temporary.path());
        record_generation_outcome(
            &repository,
            &meeting_id,
            "job-2",
            "rev-3",
            "not json at all",
        );
        assert!(repository.protocol_evidence("rev-3").unwrap().is_none());
    }

    #[test]
    fn a_term_can_be_added_edited_switched_off_and_removed() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let (project_id, meeting_id) = project_with_meeting(&mut repository, temporary.path());

        repository
            .save_vocabulary_entry(draft("Norvek", "Organisation", Some(&project_id)))
            .unwrap();
        let entry = repository.workspace_snapshot().unwrap().vocabulary[0].clone();
        assert_eq!(entry.term, "Norvek");

        // Correcting the spelling is the point of the library, so it must survive.
        repository
            .save_vocabulary_entry(VocabularyDraft {
                id: Some(entry.id.clone()),
                term: "NORVEK".to_string(),
                ..draft("NORVEK", "Organisation", Some(&project_id))
            })
            .unwrap();
        assert_eq!(
            repository.transcription_vocabulary(&meeting_id).unwrap(),
            vec!["NORVEK".to_string()]
        );

        // Switched off, it stays in the library but reaches no runtime.
        repository
            .save_vocabulary_entry(VocabularyDraft {
                id: Some(entry.id.clone()),
                enabled: false,
                ..draft("NORVEK", "Organisation", Some(&project_id))
            })
            .unwrap();
        assert!(
            repository
                .transcription_vocabulary(&meeting_id)
                .unwrap()
                .is_empty()
        );
        let listed = repository.workspace_snapshot().unwrap().vocabulary;
        assert_eq!(listed.len(), 1, "a switched-off term stays visible");
        assert!(!listed[0].enabled);

        repository.delete_vocabulary_entry(&entry.id).unwrap();
        assert!(
            repository
                .workspace_snapshot()
                .unwrap()
                .vocabulary
                .is_empty()
        );
    }

    #[test]
    fn the_same_term_is_not_stored_twice_in_one_scope() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let (project_id, _) = project_with_meeting(&mut repository, temporary.path());

        repository
            .save_vocabulary_entry(draft("NORVEK", "Organisation", Some(&project_id)))
            .unwrap();
        assert!(
            repository
                .save_vocabulary_entry(draft("norvek", "Organisation", Some(&project_id)))
                .is_err(),
            "a repeat spends part of a short prompt saying the same thing twice"
        );
        // The same word may still be held globally as well as by one project.
        repository
            .save_vocabulary_entry(draft("NORVEK", "Organisation", None))
            .unwrap();
    }

    #[test]
    fn a_project_term_must_name_its_project() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let orphan = VocabularyDraft {
            project_id: None,
            ..draft("NORVEK", "Organisation", Some("ignored"))
        };
        assert!(repository.save_vocabulary_entry(orphan).is_err());
        assert!(
            repository
                .save_vocabulary_entry(draft("", "Organisation", None))
                .is_err()
        );
    }

    /// The runtime takes about 224 tokens, so what comes first is what survives.
    #[test]
    fn proper_nouns_reach_the_transcriber_before_field_terminology() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let (project_id, meeting_id) = project_with_meeting(&mut repository, temporary.path());
        for (term, category) in [
            ("Bauteil", "Technical term"),
            ("Sonstiges", "Other"),
            ("Mustermann", "Person"),
            ("Musterteil", "Building part"),
            ("GU", "Abbreviation"),
            ("NORVEK", "Organisation"),
        ] {
            repository
                .save_vocabulary_entry(draft(term, category, Some(&project_id)))
                .unwrap();
        }
        repository
            .save_vocabulary_entry(draft("Aaa Global", "Person", None))
            .unwrap();

        assert_eq!(
            repository.transcription_vocabulary(&meeting_id).unwrap(),
            vec![
                // The project's own entries first, names before terminology.
                "Mustermann".to_string(),
                "NORVEK".to_string(),
                "GU".to_string(),
                // A category this build does not know sits above general terms.
                "Musterteil".to_string(),
                "Bauteil".to_string(),
                "Sonstiges".to_string(),
                // Shared entries last, however specific they are.
                "Aaa Global".to_string(),
            ]
        );
    }

    #[test]
    fn transcription_vocabulary_puts_project_terms_before_global_ones() {
        let temporary = tempdir().unwrap();
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let project = repository
            .create_project(NewProjectInput {
                name: "Beispielquartier".to_string(),
                description: String::new(),
                default_language: "German".to_string(),
            })
            .unwrap();
        let source = temporary.path().join("synthetic.wav");
        fs::write(&source, b"synthetic").unwrap();
        let meeting = repository
            .create_meeting(NewMeetingInput {
                project_id: project.id.clone(),
                title: "Jour fixe".to_string(),
                occurred_at: "2026-08-06".to_string(),
                language: "German".to_string(),
                source_name: "synthetic.wav".to_string(),
                source_path: Some(source.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();
        repository
            .connection
            .execute(
                "INSERT INTO vocabulary_entries
                    (id, term, preferred_spelling, category, scope, project_id, enabled,
                     updated_at_ms)
                 VALUES ('v1','Zzz Global','Zzz Global','Term','Global',NULL,1,0),
                        ('v2','NORVEK','NORVEK','Organisation','Project',?1,1,0),
                        ('v3','Disabled','Disabled','Term','Project',?1,0,0)",
                [&project.id],
            )
            .unwrap();

        let terms = repository.transcription_vocabulary(&meeting.id).unwrap();
        // The project's own name comes first despite sorting later alphabetically,
        // and a disabled entry never reaches the runtime.
        assert_eq!(terms, vec!["NORVEK".to_string(), "Zzz Global".to_string()]);
    }

    #[test]
    fn duration_labels_are_derived_from_probed_milliseconds() {
        assert_eq!(duration_label_from_ms(45_000), "45 s");
        assert_eq!(duration_label_from_ms(2_760_000), "46 min");
        assert_eq!(duration_label_from_ms(3_960_000), "1 h 06 min");
        assert_eq!(duration_label_from_ms(-5), "0 s");
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
    fn language_updates_are_durable_and_validated() {
        let temporary = tempdir().unwrap();
        let source = synthetic_source(temporary.path());
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let project = repository.create_project(project_input()).unwrap();
        let meeting = repository
            .create_meeting(meeting_input(&project.id, &source))
            .unwrap();

        repository
            .update_meeting_language(&meeting.id, "German")
            .unwrap();
        assert!(matches!(
            repository.update_meeting_language(&meeting.id, " "),
            Err(StorageError::InvalidData(_))
        ));
        drop(repository);

        let repository = WorkspaceRepository::open(temporary.path()).unwrap();
        assert_eq!(
            repository.workspace_snapshot().unwrap().meetings[0].language,
            "German"
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
        let runtime_config_columns: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('jobs') WHERE name = 'runtime_config_json'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(runtime_config_columns, 1);
        let model_cache_tables: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'model_provenance_cache'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(model_cache_tables, 1);

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

