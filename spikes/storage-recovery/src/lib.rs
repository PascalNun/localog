use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    InvalidIdentifier(String),
    InjectedFault,
    MissingArtifact(PathBuf),
    ChecksumMismatch(PathBuf),
}

impl Display for StorageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Sql(error) => write!(formatter, "SQLite error: {error}"),
            Self::InvalidIdentifier(value) => write!(formatter, "unsafe identifier: {value}"),
            Self::InjectedFault => write!(formatter, "injected failure before database commit"),
            Self::MissingArtifact(path) => {
                write!(formatter, "missing artifact: {}", path.display())
            }
            Self::ChecksumMismatch(path) => {
                write!(formatter, "artifact checksum mismatch: {}", path.display())
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactKind {
    Transcript,
    Protocol,
}

impl ArtifactKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Transcript => "transcript",
            Self::Protocol => "protocol",
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Transcript => "json",
            Self::Protocol => "md",
        }
    }

    fn lifecycle(self) -> &'static str {
        match self {
            Self::Transcript => "transcript_ready",
            Self::Protocol => "protocol_draft",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultPoint {
    None,
    AfterDurableFileBeforeDatabase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryMode {
    Startup,
    FullIntegrity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionRecord {
    pub id: String,
    pub meeting_id: String,
    pub kind: String,
    pub version: i64,
    pub relative_path: PathBuf,
    pub checksum: String,
    pub byte_count: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecoveryReport {
    pub interrupted_jobs: usize,
    pub incomplete_writes: Vec<PathBuf>,
    pub unreferenced_files: Vec<PathBuf>,
    pub missing_files: Vec<PathBuf>,
    pub checksum_mismatches: Vec<PathBuf>,
}

pub struct SpikeStore {
    root: PathBuf,
    connection: Connection,
}

impl SpikeStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("artifacts"))?;
        fs::create_dir_all(root.join("working"))?;
        fs::create_dir_all(root.join("originals"))?;
        let connection = Connection::open(root.join("localog.sqlite3"))?;
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = FULL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;

            CREATE TABLE IF NOT EXISTS meetings (
                id TEXT PRIMARY KEY,
                lifecycle TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS revisions (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id),
                kind TEXT NOT NULL CHECK (kind IN ('transcript', 'protocol')),
                version INTEGER NOT NULL CHECK (version > 0),
                artifact_path TEXT NOT NULL UNIQUE,
                checksum TEXT NOT NULL,
                byte_count INTEGER NOT NULL CHECK (byte_count >= 0),
                committed_at_ms INTEGER NOT NULL,
                UNIQUE (meeting_id, kind, version)
            );

            CREATE TABLE IF NOT EXISTS original_media (
                meeting_id TEXT PRIMARY KEY REFERENCES meetings(id),
                artifact_path TEXT NOT NULL UNIQUE,
                checksum TEXT NOT NULL,
                byte_count INTEGER NOT NULL CHECK (byte_count >= 0)
            );

            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id),
                kind TEXT NOT NULL,
                state TEXT NOT NULL CHECK (
                    state IN ('queued', 'running', 'cancelling', 'failed', 'interrupted', 'completed')
                )
            );
            ",
        )?;
        Ok(Self { root, connection })
    }

    pub fn create_meeting(&self, meeting_id: &str) -> Result<()> {
        validate_identifier(meeting_id)?;
        self.connection.execute(
            "INSERT INTO meetings (id, lifecycle) VALUES (?1, 'source_ready')",
            [meeting_id],
        )?;
        Ok(())
    }

    pub fn meeting_lifecycle(&self, meeting_id: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT lifecycle FROM meetings WHERE id = ?1",
                [meeting_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn create_job(
        &self,
        job_id: &str,
        meeting_id: &str,
        kind: &str,
        state: &str,
    ) -> Result<()> {
        validate_identifier(job_id)?;
        validate_identifier(meeting_id)?;
        self.connection.execute(
            "INSERT INTO jobs (id, meeting_id, kind, state) VALUES (?1, ?2, ?3, ?4)",
            params![job_id, meeting_id, kind, state],
        )?;
        Ok(())
    }

    pub fn job_state(&self, job_id: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row("SELECT state FROM jobs WHERE id = ?1", [job_id], |row| {
                row.get(0)
            })
            .optional()?)
    }

    pub fn import_original(&mut self, meeting_id: &str, source: &Path) -> Result<RevisionRecord> {
        validate_identifier(meeting_id)?;
        let relative_path = PathBuf::from("originals")
            .join(meeting_id)
            .join("source.bin");
        let destination = self.root.join(&relative_path);
        let parent = destination.parent().expect("managed original has a parent");
        fs::create_dir_all(parent)?;
        let temporary = parent.join("source.bin.part");

        let mut input = File::open(source)?;
        let mut output = create_new_file(&temporary)?;
        let mut hasher = Sha256::new();
        let mut byte_count = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = input.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            output.write_all(&buffer[..read])?;
            hasher.update(&buffer[..read]);
            byte_count += read as u64;
        }
        output.sync_all()?;
        drop(output);
        fs::rename(&temporary, &destination)?;
        sync_directory_chain(parent, &self.root)?;

        let digest = hasher.finalize();
        let checksum = hex_bytes(&digest);
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO original_media (meeting_id, artifact_path, checksum, byte_count)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                meeting_id,
                path_text(&relative_path),
                checksum,
                byte_count as i64
            ],
        )?;
        transaction.commit()?;

        Ok(RevisionRecord {
            id: format!("original-{meeting_id}"),
            meeting_id: meeting_id.to_string(),
            kind: "original".to_string(),
            version: 1,
            relative_path,
            checksum,
            byte_count,
        })
    }

    pub fn commit_revision(
        &mut self,
        revision_id: &str,
        meeting_id: &str,
        kind: ArtifactKind,
        version: i64,
        contents: &[u8],
        fault: FaultPoint,
    ) -> Result<RevisionRecord> {
        validate_identifier(revision_id)?;
        validate_identifier(meeting_id)?;
        let relative_path = PathBuf::from("artifacts")
            .join(meeting_id)
            .join(kind.as_str())
            .join(format!("{revision_id}.{}", kind.extension()));
        let destination = self.root.join(&relative_path);
        let parent = destination.parent().expect("managed revision has a parent");
        fs::create_dir_all(parent)?;
        let temporary = parent.join(format!(".{revision_id}.part"));

        let mut file = create_new_file(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &destination)?;
        sync_directory_chain(parent, &self.root)?;

        if fault == FaultPoint::AfterDurableFileBeforeDatabase {
            return Err(StorageError::InjectedFault);
        }

        let checksum = sha256(contents);
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO revisions (
                id, meeting_id, kind, version, artifact_path, checksum, byte_count, committed_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                revision_id,
                meeting_id,
                kind.as_str(),
                version,
                path_text(&relative_path),
                checksum,
                contents.len() as i64,
                unix_time_millis()
            ],
        )?;
        transaction.execute(
            "UPDATE meetings SET lifecycle = ?1 WHERE id = ?2",
            params![kind.lifecycle(), meeting_id],
        )?;
        transaction.commit()?;

        Ok(RevisionRecord {
            id: revision_id.to_string(),
            meeting_id: meeting_id.to_string(),
            kind: kind.as_str().to_string(),
            version,
            relative_path,
            checksum,
            byte_count: contents.len() as u64,
        })
    }

    pub fn visible_revisions(&self, meeting_id: &str) -> Result<Vec<RevisionRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id, meeting_id, kind, version, artifact_path, checksum, byte_count
             FROM revisions WHERE meeting_id = ?1 ORDER BY kind, version",
        )?;
        let rows = statement.query_map([meeting_id], |row| {
            Ok(RevisionRecord {
                id: row.get(0)?,
                meeting_id: row.get(1)?,
                kind: row.get(2)?,
                version: row.get(3)?,
                relative_path: PathBuf::from(row.get::<_, String>(4)?),
                checksum: row.get(5)?,
                byte_count: row.get::<_, i64>(6)? as u64,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn load_verified(&self, record: &RevisionRecord) -> Result<Vec<u8>> {
        let path = self.root.join(&record.relative_path);
        let contents = match fs::read(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::MissingArtifact(record.relative_path.clone()));
            }
            Err(error) => return Err(error.into()),
        };
        if sha256(&contents) != record.checksum {
            return Err(StorageError::ChecksumMismatch(record.relative_path.clone()));
        }
        Ok(contents)
    }

    pub fn save_working(
        &self,
        meeting_id: &str,
        kind: ArtifactKind,
        contents: &[u8],
    ) -> Result<PathBuf> {
        validate_identifier(meeting_id)?;
        let relative_path = PathBuf::from("working")
            .join(meeting_id)
            .join(format!("{}.autosave", kind.as_str()));
        let destination = self.root.join(&relative_path);
        let parent = destination
            .parent()
            .expect("managed working file has a parent");
        fs::create_dir_all(parent)?;
        let temporary = parent.join(format!(".{}.autosave.part", kind.as_str()));
        if temporary.exists() {
            fs::remove_file(&temporary)?;
        }
        let mut file = create_new_file(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &destination)?;
        sync_directory_chain(parent, &self.root)?;
        Ok(relative_path)
    }

    pub fn recover(&self, mode: RecoveryMode) -> Result<RecoveryReport> {
        let interrupted_jobs = self.connection.execute(
            "UPDATE jobs SET state = 'interrupted'
             WHERE state IN ('queued', 'running', 'cancelling')",
            [],
        )?;

        let mut referenced = HashMap::new();
        {
            let mut statement = self
                .connection
                .prepare("SELECT artifact_path, checksum FROM revisions")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (path, checksum) = row?;
                referenced.insert(PathBuf::from(path), checksum);
            }
        }
        {
            let mut statement = self
                .connection
                .prepare("SELECT artifact_path, checksum FROM original_media")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (path, checksum) = row?;
                referenced.insert(PathBuf::from(path), checksum);
            }
        }

        let mut report = RecoveryReport {
            interrupted_jobs,
            ..RecoveryReport::default()
        };

        for (relative_path, expected_checksum) in &referenced {
            let path = self.root.join(relative_path);
            if !path.exists() {
                report.missing_files.push(relative_path.clone());
                continue;
            }
            if mode == RecoveryMode::FullIntegrity && sha256_file(&path)? != *expected_checksum {
                report.checksum_mismatches.push(relative_path.clone());
            }
        }

        for (managed_root, requires_reference) in
            [("artifacts", true), ("originals", true), ("working", false)]
        {
            let root = self.root.join(managed_root);
            let mut files = Vec::new();
            collect_files(&root, &mut files)?;
            for file in files {
                let relative_path = file
                    .strip_prefix(&self.root)
                    .expect("managed file is below storage root")
                    .to_path_buf();
                if file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".part"))
                {
                    report.incomplete_writes.push(relative_path);
                } else if requires_reference && !referenced.contains_key(&relative_path) {
                    report.unreferenced_files.push(relative_path);
                }
            }
        }

        report.incomplete_writes.sort();
        report.unreferenced_files.sort();
        report.missing_files.sort();
        report.checksum_mismatches.sort();
        Ok(report)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn validate_identifier(value: &str) -> Result<()> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(StorageError::InvalidIdentifier(value.to_string()))
    }
}

fn create_new_file(path: &Path) -> Result<File> {
    Ok(OpenOptions::new().write(true).create_new(true).open(path)?)
}

fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

fn sync_directory_chain(start: &Path, storage_root: &Path) -> Result<()> {
    let mut current = Some(start);
    while let Some(directory) = current {
        sync_directory(directory)?;
        if directory == storage_root {
            break;
        }
        current = directory.parent();
    }
    Ok(())
}

fn unix_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_millis() as i64
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sha256(contents: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(contents);
    let digest = hasher.finalize();
    hex_bytes(&digest)
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    Ok(hex_bytes(&digest))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn revision_is_visible_only_after_durable_file_and_database_commit() {
        let temporary = tempdir().unwrap();
        let mut store = SpikeStore::open(temporary.path()).unwrap();
        store.create_meeting("meeting-1").unwrap();

        let failed = store.commit_revision(
            "transcript-v1-faulted",
            "meeting-1",
            ArtifactKind::Transcript,
            1,
            br#"{"segments":[]}"#,
            FaultPoint::AfterDurableFileBeforeDatabase,
        );
        assert!(matches!(failed, Err(StorageError::InjectedFault)));
        assert!(store.visible_revisions("meeting-1").unwrap().is_empty());
        assert_eq!(
            store.meeting_lifecycle("meeting-1").unwrap().unwrap(),
            "source_ready"
        );

        let report = store.recover(RecoveryMode::Startup).unwrap();
        assert_eq!(report.unreferenced_files.len(), 1);

        let committed = store
            .commit_revision(
                "transcript-v1",
                "meeting-1",
                ArtifactKind::Transcript,
                1,
                br#"{"segments":[{"start_ms":0,"text":"Synthetic opening"}]}"#,
                FaultPoint::None,
            )
            .unwrap();
        assert_eq!(
            store.visible_revisions("meeting-1").unwrap(),
            vec![committed]
        );
        assert_eq!(
            store.meeting_lifecycle("meeting-1").unwrap().unwrap(),
            "transcript_ready"
        );
    }

    #[test]
    fn recovery_reports_interrupted_jobs_and_file_damage_without_hiding_it() {
        let temporary = tempdir().unwrap();
        let mut store = SpikeStore::open(temporary.path()).unwrap();
        store.create_meeting("meeting-1").unwrap();
        for (id, state) in [
            ("job-queued", "queued"),
            ("job-running", "running"),
            ("job-cancelling", "cancelling"),
            ("job-completed", "completed"),
            ("job-failed", "failed"),
        ] {
            store
                .create_job(id, "meeting-1", "transcription", state)
                .unwrap();
        }
        let committed = store
            .commit_revision(
                "transcript-v1",
                "meeting-1",
                ArtifactKind::Transcript,
                1,
                b"trusted transcript",
                FaultPoint::None,
            )
            .unwrap();
        let missing = store
            .commit_revision(
                "protocol-v1",
                "meeting-1",
                ArtifactKind::Protocol,
                1,
                b"# Synthetic protocol",
                FaultPoint::None,
            )
            .unwrap();

        fs::write(
            store.root().join(&committed.relative_path),
            b"changed outside the app",
        )
        .unwrap();
        fs::remove_file(store.root().join(&missing.relative_path)).unwrap();
        let incomplete = store
            .root()
            .join("artifacts/meeting-1/transcript/.later.part");
        fs::write(incomplete, b"partial").unwrap();
        let incomplete_autosave = store
            .root()
            .join("working/meeting-1/.transcript.autosave.part");
        fs::create_dir_all(incomplete_autosave.parent().unwrap()).unwrap();
        fs::write(incomplete_autosave, b"partial working state").unwrap();

        let report = store.recover(RecoveryMode::FullIntegrity).unwrap();
        assert_eq!(report.interrupted_jobs, 3);
        assert_eq!(report.incomplete_writes.len(), 2);
        assert_eq!(report.missing_files, vec![missing.relative_path]);
        assert_eq!(report.checksum_mismatches, vec![committed.relative_path]);
        assert_eq!(
            store.job_state("job-running").unwrap().unwrap(),
            "interrupted"
        );
        assert_eq!(
            store.job_state("job-completed").unwrap().unwrap(),
            "completed"
        );
        assert_eq!(store.job_state("job-failed").unwrap().unwrap(), "failed");
    }

    #[test]
    fn autosave_and_original_media_do_not_mutate_committed_or_source_bytes() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("synthetic-source.bin");
        fs::write(&source, b"synthetic original media bytes").unwrap();
        let source_before = fs::read(&source).unwrap();

        let storage_root = temporary.path().join("managed");
        let mut store = SpikeStore::open(&storage_root).unwrap();
        store.create_meeting("meeting-1").unwrap();
        let original = store.import_original("meeting-1", &source).unwrap();
        assert_eq!(fs::read(&source).unwrap(), source_before);
        assert_eq!(store.load_verified(&original).unwrap(), source_before);

        let revision = store
            .commit_revision(
                "transcript-v1",
                "meeting-1",
                ArtifactKind::Transcript,
                1,
                b"committed revision one",
                FaultPoint::None,
            )
            .unwrap();
        store
            .save_working("meeting-1", ArtifactKind::Transcript, b"working edit one")
            .unwrap();
        store
            .save_working("meeting-1", ArtifactKind::Transcript, b"working edit two")
            .unwrap();
        assert_eq!(
            store.load_verified(&revision).unwrap(),
            b"committed revision one"
        );
    }

    #[test]
    fn hostile_identifiers_never_become_paths() {
        let temporary = tempdir().unwrap();
        let store = SpikeStore::open(temporary.path()).unwrap();
        let error = store.create_meeting("../outside").unwrap_err();
        assert!(matches!(error, StorageError::InvalidIdentifier(_)));
        assert!(!temporary.path().join("outside").exists());
    }
}
