use crate::domain::{
    MeetingLifecycle, MeetingSummary, NewMeetingInput, NewProjectInput, ProjectSummary,
    WorkspaceSnapshot,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const CURRENT_SCHEMA_VERSION: i64 = 1;
const DEFAULT_STYLE_ID: &str = "style-formal";

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    InvalidData(&'static str),
    MissingProject,
    MissingMeeting,
    UnsupportedSchema(i64),
}

impl StorageError {
    /// Commands return bounded, content-free messages rather than database paths or SQL details.
    pub fn user_message(&self) -> String {
        match self {
            Self::InvalidData(message) => (*message).to_string(),
            Self::MissingProject => "The selected project no longer exists.".to_string(),
            Self::MissingMeeting => "The selected meeting no longer exists.".to_string(),
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
    connection: Connection,
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
        Ok(Self { connection })
    }

    pub fn workspace_snapshot(&self) -> Result<WorkspaceSnapshot> {
        Ok(WorkspaceSnapshot {
            projects: self.list_projects()?,
            meetings: self.list_meetings()?,
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

        if !self.project_exists(&project_id)? {
            return Err(StorageError::MissingProject);
        }

        let meeting_id = new_id("meeting");
        let recording_id = new_id("recording");
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
    Ok(MeetingSummary {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        occurred_at: row.get(3)?,
        duration_label: row.get(4)?,
        lifecycle,
        language: row.get(6)?,
        source_name: row.get(7)?,
        style_id: row.get(8)?,
    })
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

fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::now_v7())
}

fn unix_time_millis() -> i64 {
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

    fn meeting_input(project_id: &str) -> NewMeetingInput {
        NewMeetingInput {
            project_id: project_id.to_string(),
            title: "".to_string(),
            occurred_at: "2026-08-02".to_string(),
            language: "English".to_string(),
            source_name: "synthetic-design-review.wav".to_string(),
            style_id: DEFAULT_STYLE_ID.to_string(),
        }
    }

    #[test]
    fn project_and_meeting_survive_repository_reopen() {
        let temporary = tempdir().unwrap();
        let (project, meeting) = {
            let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
            let project = repository.create_project(project_input()).unwrap();
            let meeting = repository
                .create_meeting(meeting_input(&project.id))
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
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let project = repository.create_project(project_input()).unwrap();
        let meeting = repository
            .create_meeting(meeting_input(&project.id))
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
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let missing_project = repository.create_meeting(meeting_input("missing-project"));
        assert!(matches!(missing_project, Err(StorageError::MissingProject)));

        let project = repository.create_project(project_input()).unwrap();
        let mut hostile = meeting_input(&project.id);
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
        let mut repository = WorkspaceRepository::open(temporary.path()).unwrap();
        let project = repository.create_project(project_input()).unwrap();
        let meeting = repository
            .create_meeting(meeting_input(&project.id))
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
}
