//! Copying the workspace somewhere safe, and putting it back.
//!
//! The application tells everybody their meeting work stays on this device. The
//! other half of that sentence is that when the device dies the work dies with
//! it, and for somebody keeping a year of minutes that is not a missing feature
//! but a liability. This is the answer to it.
//!
//! ## What a backup is
//!
//! A folder, not an archive. Nothing here compresses, because what fills a
//! workspace is recorded audio and audio is already compressed — a zip would
//! spend a dependency and a lot of CPU to save nothing. A folder is also
//! inspectable: somebody worried about their backup can open it and see their
//! meetings, which is worth more than a single file that has to be trusted.
//!
//! ```text
//! LocaLog backup 2026-08-26/
//!   manifest.json      what this is, and the checksum of every file in it
//!   localog.sqlite3    projects, meetings, transcripts, protocols, revisions
//!   projects/…         the managed audio
//! ```
//!
//! ## Three things this gets right on purpose
//!
//! **The database is copied by SQLite, not by the filesystem.** There is a
//! write-ahead log beside it, and at any moment the most recent writes live in
//! the `-wal` file rather than in `localog.sqlite3`. Copying the one file gives
//! a backup that is silently missing the newest work — the worst kind of wrong,
//! because it looks complete. `VACUUM INTO` asks SQLite for a transactionally
//! consistent copy with the log folded in.
//!
//! **Models are left out.** They are re-downloadable, checksummed on the way in,
//! and between 77 MB and 1.5 GB each. Including them would turn a twenty-megabyte
//! backup of somebody's actual work into a gigabyte of files they can get again
//! for nothing. The manifest says they were left out rather than leaving somebody
//! to notice.
//!
//! **Nothing is deleted, ever.** Restoring verifies the whole backup before it
//! touches the workspace, and then moves what is there aside instead of removing
//! it. A restore that turns out to be the wrong backup is recoverable; one that
//! deleted first would not be.

use crate::storage::WorkspaceRepository;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The format this module writes. A restore refuses anything it does not know,
/// rather than guessing at a layout written by a later version.
const FORMAT: u32 = 1;

const MANIFEST: &str = "manifest.json";
const DATABASE: &str = "localog.sqlite3";

/// One file in a backup, with what it should be when it comes back.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileRecord {
    /// Relative to the backup folder, always with forward slashes.
    pub path: String,
    pub byte_count: u64,
    pub sha256: String,
}

/// What a backup says about itself.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    pub format: u32,
    pub created_at_ms: u64,
    pub application_version: String,
    pub database: FileRecord,
    pub files: Vec<FileRecord>,
    pub project_count: u32,
    pub meeting_count: u32,
    /// Said out loud so nobody discovers it at the worst moment.
    pub excludes_models: bool,
    /// Where this came from, purely so a person can tell two backups apart.
    pub folder_name: String,
}

#[derive(Debug)]
pub(crate) enum BackupError {
    /// The name the interface supplied was not a single, safe folder name.
    UnsafeName,
    /// A path inside a manifest tried to leave the backup folder.
    UnsafePath(String),
    NotABackup,
    UnknownFormat(u32),
    /// A file is missing, the wrong size, or does not match its checksum.
    Damaged(String),
    AlreadyThere(String),
    Io(String),
    Database(String),
}

impl BackupError {
    pub(crate) fn user_message(&self) -> String {
        match self {
            Self::UnsafeName => "That backup name cannot be used as a folder name.".into(),
            Self::UnsafePath(path) => format!(
                "This backup lists a file outside its own folder ({path}), so it was not restored."
            ),
            Self::NotABackup => {
                "That folder is not a LocaLog backup: it has no manifest.json.".into()
            }
            Self::UnknownFormat(format) => format!(
                "This backup was written in format {format}, which this version of LocaLog does not \
                 know how to read. A newer LocaLog will."
            ),
            Self::Damaged(what) => format!(
                "This backup is incomplete or damaged ({what}), so nothing was changed. Your \
                 current work is untouched."
            ),
            Self::AlreadyThere(name) => {
                format!("There is already something called \"{name}\" in that folder.")
            }
            Self::Io(what) => format!("The backup could not be written or read: {what}"),
            Self::Database(what) => format!("The database could not be copied: {what}"),
        }
    }
}

impl From<std::io::Error> for BackupError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<rusqlite::Error> for BackupError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error.to_string())
    }
}

type Result<T> = std::result::Result<T, BackupError>;

/// Write the whole workspace into a new folder inside `parent`.
///
/// `folder_name` comes from the interface because that is where a readable date
/// already exists, and is checked here because a name that arrived as a path
/// would write outside the folder somebody chose.
pub(crate) fn create(root: &Path, parent: &Path, folder_name: &str) -> Result<Manifest> {
    let name = folder_name.trim();
    if name.is_empty() || name.starts_with('.') || Path::new(name).components().count() != 1 {
        return Err(BackupError::UnsafeName);
    }
    let folder = parent.join(name);
    if folder.exists() {
        return Err(BackupError::AlreadyThere(name.to_string()));
    }
    fs::create_dir_all(&folder)?;

    // SQLite writes the copy, for the write-ahead-log reason in the module note.
    // Opening the workspace normally first means any pending migration has run,
    // so a backup is never of a half-upgraded database.
    let _ = WorkspaceRepository::open(root).map_err(|error| {
        BackupError::Database(format!("the workspace could not be opened: {error:?}"))
    })?;
    let connection = Connection::open(root.join(DATABASE))?;
    let destination = folder.join(DATABASE);
    // The path goes in as a bound parameter: a workspace under a folder with a
    // quote in its name would otherwise end the string and change the statement.
    connection.execute("VACUUM INTO ?1", [destination.to_string_lossy().as_ref()])?;

    let (project_count, meeting_count) = counts(&destination)?;

    // The managed audio, in the same shape it sits in the workspace.
    let mut files = Vec::new();
    let projects = root.join("projects");
    if projects.is_dir() {
        let mut found = Vec::new();
        collect(&projects, &projects, &mut found)?;
        // Sorted so two backups of the same workspace list their files in the
        // same order, which makes them comparable by eye and by diff.
        found.sort();
        for relative in found {
            let from = projects.join(&relative);
            let to = folder.join("projects").join(&relative);
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to)?;
            let (sha256, byte_count) = checksum(&to)?;
            files.push(FileRecord {
                path: format!("projects/{}", slashed(&relative)),
                byte_count,
                sha256,
            });
        }
    }

    let (database_sha, database_bytes) = checksum(&destination)?;
    let manifest = Manifest {
        format: FORMAT,
        created_at_ms: now_ms(),
        application_version: env!("CARGO_PKG_VERSION").to_string(),
        database: FileRecord {
            path: DATABASE.to_string(),
            byte_count: database_bytes,
            sha256: database_sha,
        },
        files,
        project_count,
        meeting_count,
        excludes_models: true,
        folder_name: name.to_string(),
    };
    let written = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| BackupError::Io(error.to_string()))?;
    fs::write(folder.join(MANIFEST), written)?;
    Ok(manifest)
}

/// Read what a folder claims to be, without checking any of the files.
///
/// Separate from verifying because the interface wants to say "this is a backup
/// of 4 projects from 26 August, restore it?" before spending minutes hashing
/// gigabytes of audio.
pub(crate) fn inspect(folder: &Path) -> Result<Manifest> {
    let path = folder.join(MANIFEST);
    if !path.is_file() {
        return Err(BackupError::NotABackup);
    }
    let text = fs::read_to_string(&path)?;
    let manifest: Manifest =
        serde_json::from_str(&text).map_err(|_| BackupError::NotABackup)?;
    if manifest.format != FORMAT {
        return Err(BackupError::UnknownFormat(manifest.format));
    }
    Ok(manifest)
}

/// Check every file against the manifest before anything is moved.
fn verify(folder: &Path, manifest: &Manifest) -> Result<()> {
    for record in std::iter::once(&manifest.database).chain(manifest.files.iter()) {
        let relative = safe_relative(&record.path)?;
        let path = folder.join(&relative);
        if !path.is_file() {
            return Err(BackupError::Damaged(format!("{} is missing", record.path)));
        }
        let (sha256, byte_count) = checksum(&path)?;
        if byte_count != record.byte_count {
            return Err(BackupError::Damaged(format!(
                "{} is {byte_count} bytes and should be {}",
                record.path, record.byte_count
            )));
        }
        if sha256 != record.sha256 {
            return Err(BackupError::Damaged(format!(
                "{} does not match its checksum",
                record.path
            )));
        }
    }
    Ok(())
}

/// What a restore did, so the interface can say it rather than imply it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RestoreOutcome {
    pub project_count: u32,
    pub meeting_count: u32,
    /// Where the workspace that was replaced now lives. Kept, never deleted.
    pub previous_workspace: String,
}

/// Put a verified backup in place of the current workspace.
///
/// The whole backup is checked first: a restore that failed halfway would leave
/// somebody with neither their old work nor their new. By the time anything
/// moves, every file has been read and matched.
pub(crate) fn restore(root: &Path, folder: &Path) -> Result<RestoreOutcome> {
    let manifest = inspect(folder)?;
    verify(folder, &manifest)?;

    // What is here now goes sideways, not away.
    let aside = root.join(format!("replaced-{}", now_ms()));
    fs::create_dir_all(&aside)?;
    for name in [DATABASE, "localog.sqlite3-wal", "localog.sqlite3-shm"] {
        let from = root.join(name);
        if from.exists() {
            fs::rename(&from, aside.join(name))?;
        }
    }
    let projects = root.join("projects");
    if projects.exists() {
        fs::rename(&projects, aside.join("projects"))?;
    }

    fs::copy(folder.join(DATABASE), root.join(DATABASE))?;
    for record in &manifest.files {
        let relative = safe_relative(&record.path)?;
        let to = root.join(&relative);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(folder.join(&relative), to)?;
    }

    Ok(RestoreOutcome {
        project_count: manifest.project_count,
        meeting_count: manifest.meeting_count,
        previous_workspace: aside.to_string_lossy().to_string(),
    })
}

/// A manifest path that cannot leave the folder it belongs to.
///
/// A backup is a file somebody may have been sent, so its manifest is input
/// rather than something this wrote. `../../.ssh/authorized_keys` is a path a
/// naive restore would happily write to.
fn safe_relative(path: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(path);
    let ordinary = candidate
        .components()
        .all(|part| matches!(part, Component::Normal(_)));
    if !ordinary || candidate.as_os_str().is_empty() {
        return Err(BackupError::UnsafePath(path.to_string()));
    }
    Ok(candidate)
}

/// Every file under `directory`, as paths relative to `base`.
fn collect(base: &Path, directory: &Path, found: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(base, &path, found)?;
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(base) {
                found.push(relative.to_path_buf());
            }
        }
    }
    Ok(())
}

/// Written with forward slashes whatever wrote them, so a backup made on Windows
/// restores on macOS and the other way about.
fn slashed(path: &Path) -> String {
    path.components()
        .filter_map(|part| match part {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// SHA-256 and length of one file.
///
/// `processing::durability` has a streaming checksum already, but it is
/// `pub(super)`, speaks `ProcessingError`, and validates paths against the
/// managed-artifact rules that do not apply to a backup folder. Widening it to
/// reach here would drag the pipeline's vocabulary into a module that has
/// nothing to do with jobs.
fn checksum(path: &Path) -> Result<(String, u64)> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 256 * 1024];
    let mut byte_count = 0_u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        byte_count += read as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), byte_count))
}

/// How much is in the copy, read from the copy rather than from the original,
/// so the number describes what was actually written.
fn counts(database: &Path) -> Result<(u32, u32)> {
    let connection = Connection::open(database)?;
    let projects: u32 = connection.query_row(
        "SELECT COUNT(*) FROM projects WHERE archived_at_ms IS NULL",
        [],
        |row| row.get(0),
    )?;
    let meetings: u32 = connection.query_row(
        "SELECT COUNT(*) FROM meetings WHERE archived_at_ms IS NULL",
        [],
        |row| row.get(0),
    )?;
    Ok((projects, meetings))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{NewMeetingInput, NewProjectInput};
    use tempfile::tempdir;

    /// A workspace with one project, one meeting and one managed file in it.
    fn workspace(root: &Path) {
        let mut repository = WorkspaceRepository::open(root).unwrap();
        let project = repository
            .create_project(NewProjectInput {
                name: "Harbour".to_string(),
                description: String::new(),
                default_language: "German".to_string(),
            })
            .unwrap();
        let source = root.join("fixture.wav");
        fs::write(&source, b"not really audio").unwrap();
        repository
            .create_meeting(NewMeetingInput {
                project_id: project.id.clone(),
                title: "Kick-off".to_string(),
                occurred_at: "2026-08-26".to_string(),
                language: "German".to_string(),
                source_name: "fixture.wav".to_string(),
                source_path: Some(source.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();
        // One managed file, standing in for the audio a real meeting carries.
        let managed = root.join("projects").join(&project.id).join("meetings");
        fs::create_dir_all(&managed).unwrap();
        fs::write(managed.join("source.wav"), b"not really audio").unwrap();
    }

    #[test]
    fn a_backup_carries_the_database_and_the_audio() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());

        let manifest = create(home.path(), away.path(), "LocaLog backup").unwrap();
        assert_eq!(manifest.format, FORMAT);
        assert_eq!(manifest.project_count, 1);
        assert_eq!(manifest.meeting_count, 1);
        assert!(manifest.excludes_models);
        assert_eq!(manifest.files.len(), 1);
        assert!(manifest.files[0].path.starts_with("projects/"));
        // Forward slashes whatever the platform writes.
        assert!(!manifest.files[0].path.contains('\\'));
        assert!(away.path().join("LocaLog backup").join(DATABASE).is_file());
    }

    #[test]
    fn the_copied_database_is_readable_and_holds_the_work() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());
        create(home.path(), away.path(), "b").unwrap();

        // The point of VACUUM INTO: the copy is a whole database, not a file
        // missing whatever was still in the write-ahead log.
        let copied = Connection::open(away.path().join("b").join(DATABASE)).unwrap();
        let projects: u32 = copied
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(projects, 1);
    }

    #[test]
    fn a_name_that_is_a_path_is_refused() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());
        for bad in ["../escape", "a/b", "", "   ", ".hidden"] {
            assert!(matches!(
                create(home.path(), away.path(), bad),
                Err(BackupError::UnsafeName)
            ));
        }
    }

    #[test]
    fn backing_up_twice_into_the_same_name_refuses_rather_than_merging() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());
        create(home.path(), away.path(), "same").unwrap();
        assert!(matches!(
            create(home.path(), away.path(), "same"),
            Err(BackupError::AlreadyThere(_))
        ));
    }

    #[test]
    fn a_damaged_backup_is_refused_before_anything_is_moved() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());
        create(home.path(), away.path(), "b").unwrap();

        // Somebody's audio file was truncated in transit.
        let folder = away.path().join("b");
        let audio = folder.join("projects");
        let file = collect_one(&audio);
        fs::write(&file, b"short").unwrap();

        let before = fs::read(home.path().join(DATABASE)).unwrap();
        assert!(matches!(
            restore(home.path(), &folder),
            Err(BackupError::Damaged(_))
        ));
        // The workspace is exactly as it was.
        assert_eq!(fs::read(home.path().join(DATABASE)).unwrap(), before);
        assert!(home.path().join("projects").is_dir());
    }

    #[test]
    fn restoring_puts_the_work_back_and_keeps_what_it_replaced() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());
        create(home.path(), away.path(), "b").unwrap();

        // The workspace is then lost: a different project, none of the old work.
        fs::remove_file(home.path().join(DATABASE)).unwrap();
        fs::remove_dir_all(home.path().join("projects")).unwrap();
        {
            let mut repository = WorkspaceRepository::open(home.path()).unwrap();
            repository
                .create_project(NewProjectInput {
                    name: "Something else".to_string(),
                    description: String::new(),
                    default_language: "German".to_string(),
                })
                .unwrap();
        }

        let outcome = restore(home.path(), &away.path().join("b")).unwrap();
        assert_eq!(outcome.project_count, 1);
        assert!(!outcome.previous_workspace.is_empty());
        // What was there is kept rather than removed.
        assert!(Path::new(&outcome.previous_workspace).is_dir());

        let repository = WorkspaceRepository::open(home.path()).unwrap();
        let snapshot = repository.workspace_snapshot().unwrap();
        assert_eq!(snapshot.projects.len(), 1);
        assert_eq!(snapshot.projects[0].name, "Harbour");
    }

    #[test]
    fn a_manifest_pointing_outside_the_folder_is_refused() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());
        let manifest = create(home.path(), away.path(), "b").unwrap();

        let folder = away.path().join("b");
        let mut tampered = manifest;
        tampered.files[0].path = "../../somewhere/else.wav".into();
        fs::write(
            folder.join(MANIFEST),
            serde_json::to_vec_pretty(&tampered).unwrap(),
        )
        .unwrap();

        assert!(matches!(
            restore(home.path(), &folder),
            Err(BackupError::UnsafePath(_))
        ));
    }

    #[test]
    fn a_folder_without_a_manifest_is_not_a_backup() {
        let empty = tempdir().unwrap();
        assert!(matches!(inspect(empty.path()), Err(BackupError::NotABackup)));
    }

    #[test]
    fn a_backup_from_a_later_format_is_refused_rather_than_guessed_at() {
        let home = tempdir().unwrap();
        let away = tempdir().unwrap();
        workspace(home.path());
        let manifest = create(home.path(), away.path(), "b").unwrap();
        let folder = away.path().join("b");
        let mut later = manifest;
        later.format = FORMAT + 1;
        fs::write(
            folder.join(MANIFEST),
            serde_json::to_vec_pretty(&later).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            inspect(&folder),
            Err(BackupError::UnknownFormat(_))
        ));
    }

    /// The one file under a directory, for tests that put exactly one there.
    fn collect_one(directory: &Path) -> PathBuf {
        let mut found = Vec::new();
        collect(directory, directory, &mut found).unwrap();
        assert_eq!(found.len(), 1);
        directory.join(&found[0])
    }
}
