//! What the workspace is using, and the part of it that can safely go.
//!
//! A meeting's source is the one file that cannot be rebuilt. Everything else in
//! a meeting's folder either belongs to the database or is derived from that
//! source: the mono 16 kHz audio prepared for transcription is about 115 MB an
//! hour, and it is rebuilt from the source whenever a transcript or a protocol
//! needs it. Two hours of prepared audio therefore cost as much as the whole
//! compact recording, and thirty of them quietly fill a disk.
//!
//! Removing it is safe only while the source it was made from is still there.
//! That is the rule this module enforces: a cache whose recording row is missing,
//! whose source is missing, or whose source was never committed is left alone,
//! because it may be the only copy of the meeting that exists.

use crate::storage::{Result, StorageError, WorkspaceRepository, managed_relative_path};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceUsage {
    pub total_bytes: u64,
    pub recordings_bytes: u64,
    pub prepared_audio_bytes: u64,
    pub models_bytes: u64,
    pub other_bytes: u64,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreparedAudioCleanup {
    pub freed_bytes: u64,
    pub removed_files: u64,
    pub usage: WorkspaceUsage,
}

/// Measure the workspace without following symlinks out of it.
pub(crate) fn workspace_usage(root: &Path) -> Result<WorkspaceUsage> {
    let mut usage = WorkspaceUsage::default();
    measure(root, root, &mut usage);
    Ok(usage)
}

fn measure(root: &Path, directory: &Path, usage: &mut WorkspaceUsage) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // `symlink_metadata` rather than `metadata`: a workspace that contains a
        // link to a large folder outside it must not report that folder's size,
        // and must never be walked into.
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            measure(root, &path, usage);
        } else if metadata.is_file() {
            let bytes = metadata.len();
            usage.total_bytes = usage.total_bytes.saturating_add(bytes);
            match classify(root, &path) {
                Category::Recordings => {
                    usage.recordings_bytes = usage.recordings_bytes.saturating_add(bytes)
                }
                Category::PreparedAudio => {
                    usage.prepared_audio_bytes = usage.prepared_audio_bytes.saturating_add(bytes)
                }
                Category::Models => usage.models_bytes = usage.models_bytes.saturating_add(bytes),
                Category::Other => usage.other_bytes = usage.other_bytes.saturating_add(bytes),
            }
        }
    }
}

enum Category {
    Recordings,
    PreparedAudio,
    Models,
    Other,
}

/// Which part of the workspace a file belongs to, from its position rather than
/// from its name. The managed layout is the one the application itself writes.
fn classify(root: &Path, path: &Path) -> Category {
    let Ok(relative) = path.strip_prefix(root) else {
        return Category::Other;
    };
    let parts: Vec<&str> = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect();
    if parts.first() == Some(&"models") {
        return Category::Models;
    }
    if parts.first() != Some(&"projects") {
        return Category::Other;
    }
    // projects/<project>/meetings/<meeting>/recordings/<file>
    if parts.len() >= 5 && parts[2] == "meetings" && parts[4] == "recordings" {
        return Category::Recordings;
    }
    // projects/<project>/meetings/<meeting>/working/normalized/<file>
    if parts.len() >= 6
        && parts[2] == "meetings"
        && parts[4] == "working"
        && parts[5] == "normalized"
    {
        return Category::PreparedAudio;
    }
    Category::Other
}

/// Remove the prepared audio that can be rebuilt, and say what that freed.
///
/// Only rows whose recording still has the source file it was made from are
/// removed. A cache without its source is not a cache; it may be the meeting.
pub(crate) fn remove_prepared_audio(root: &Path) -> Result<PreparedAudioCleanup> {
    let repository = WorkspaceRepository::open(root)?;
    // A running transcription is reading exactly the files this would delete.
    // Refusing is clearer than deleting a file out from under a live process.
    let active: i64 = repository.connection.query_row(
        "SELECT COUNT(*) FROM jobs WHERE state IN ('queued', 'running', 'cancelling')",
        [],
        |row| row.get(0),
    )?;
    if active > 0 {
        return Err(StorageError::InvalidData("workspaceBusy"));
    }

    let mut freed_bytes = 0u64;
    let mut removed_files = 0u64;
    let mut referenced: HashSet<String> = HashSet::new();

    let rows = {
        let mut statement = repository.connection.prepare(
            "SELECT nm.normalized_path, nm.source_checksum, r.checksum, r.managed_path
             FROM normalized_media nm
             LEFT JOIN recordings r ON r.id = nm.recording_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;
        let mut collected = Vec::new();
        for row in rows {
            collected.push(row?);
        }
        collected
    };

    for (normalized_path, source_checksum, recording_checksum, managed_path) in rows {
        referenced.insert(normalized_path.clone());
        if !is_prepared_audio(&normalized_path) {
            continue;
        }
        // The source is what makes this file expendable, and it has to be the
        // same source the cache was made from. A cache from before a recording
        // was edited is the only copy of what was edited away; leave it.
        if recording_checksum.as_deref() != Some(source_checksum.as_str()) {
            continue;
        }
        let Some(managed_path) = managed_path else {
            continue;
        };
        let Ok(managed_path) = managed_relative_path(Path::new(&managed_path)) else {
            continue;
        };
        if !root.join(managed_path).is_file() {
            continue;
        }

        let file = root.join(&normalized_path);
        let bytes = fs::metadata(&file)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        if file.exists() && fs::remove_file(&file).is_err() {
            // Not ours to remove any more; leave the row so a later attempt can try.
            continue;
        }
        repository.connection.execute(
            "DELETE FROM normalized_media WHERE normalized_path = ?1",
            [&normalized_path],
        )?;
        freed_bytes = freed_bytes.saturating_add(bytes);
        removed_files += 1;
    }

    // An older settings hash or a new ffmpeg leaves a complete, unreferenced file
    // behind under a name no row points at. It is only removed when the recording
    // it belongs to still has a source to rebuild it from.
    let sources = recordings_with_sources(root, &repository)?;
    let (orphan_files, orphan_bytes) = remove_orphaned_prepared_audio(root, &referenced, &sources)?;
    removed_files += orphan_files;
    freed_bytes = freed_bytes.saturating_add(orphan_bytes);

    Ok(PreparedAudioCleanup {
        freed_bytes,
        removed_files,
        usage: workspace_usage(root)?,
    })
}

fn remove_orphaned_prepared_audio(
    root: &Path,
    referenced: &HashSet<String>,
    sources: &HashSet<String>,
) -> Result<(u64, u64)> {
    let mut files = 0u64;
    let mut bytes = 0u64;
    for directory in normalized_directories(root) {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let relative = relative.to_string_lossy().to_string();
            if referenced.contains(&relative) {
                continue;
            }
            let Some(recording_id) = prepared_audio_owner(&path) else {
                continue;
            };
            // Only a recording that still has its source can rebuild this file.
            if !sources.contains(&recording_id) {
                continue;
            }
            if fs::remove_file(&path).is_ok() {
                files += 1;
                bytes = bytes.saturating_add(metadata.len());
            }
        }
    }
    Ok((files, bytes))
}

fn recordings_with_sources(
    root: &Path,
    repository: &WorkspaceRepository,
) -> Result<HashSet<String>> {
    let mut statement = repository.connection.prepare(
        "SELECT id, managed_path FROM recordings
         WHERE state = 'committed' AND managed_path IS NOT NULL",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut found = HashSet::new();
    for row in rows {
        let (recording_id, managed_path) = row?;
        let Ok(managed_path) = managed_relative_path(Path::new(&managed_path)) else {
            continue;
        };
        if root.join(managed_path).is_file() {
            found.insert(recording_id);
        }
    }
    Ok(found)
}

/// Every `working/normalized` directory in the managed layout.
fn normalized_directories(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let projects = root.join("projects");
    let Ok(project_entries) = fs::read_dir(&projects) else {
        return found;
    };
    for project in project_entries.flatten() {
        let meetings = project.path().join("meetings");
        let Ok(meeting_entries) = fs::read_dir(&meetings) else {
            continue;
        };
        for meeting in meeting_entries.flatten() {
            let directory = meeting.path().join("working/normalized");
            if directory.is_dir() {
                found.push(directory);
            }
        }
    }
    found
}

/// Whether a path is a prepared-audio file this module may remove.
fn is_prepared_audio(relative: &str) -> bool {
    let path = Path::new(relative);
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".wav"))
        && path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            == Some("normalized")
}

/// The recording a prepared-audio file was made for, from `<id>-<hash>.wav`.
fn prepared_audio_owner(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let without_extension = name.strip_suffix(".wav")?;
    let (recording_id, _hash) = without_extension.rsplit_once('-')?;
    recording_id
        .starts_with("recording-")
        .then(|| recording_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{NewMeetingInput, NewProjectInput};
    use tempfile::tempdir;

    fn touch(path: &Path, bytes: usize) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, vec![7u8; bytes]).unwrap();
    }

    #[test]
    fn usage_says_which_part_of_the_workspace_costs_what() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        touch(&root.join("models/ggml.bin"), 1_000);
        touch(
            &root.join("projects/p/meetings/m/recordings/recording-a.opus"),
            2_000,
        );
        touch(
            &root.join("projects/p/meetings/m/working/normalized/recording-a-abc.wav"),
            3_000,
        );
        touch(&root.join("projects/p/meetings/m/transcripts/t.json"), 500);
        touch(&root.join("localog.sqlite3"), 100);

        let usage = workspace_usage(root).unwrap();
        assert_eq!(usage.models_bytes, 1_000);
        assert_eq!(usage.recordings_bytes, 2_000);
        assert_eq!(usage.prepared_audio_bytes, 3_000);
        assert_eq!(usage.other_bytes, 600);
        assert_eq!(usage.total_bytes, 6_600);
    }

    /// A cache whose source is gone may be the only audio there is.
    #[test]
    fn prepared_audio_without_its_source_is_kept() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let mut repository = WorkspaceRepository::open(root).unwrap();
        let project = repository
            .create_project(NewProjectInput {
                names: Vec::new(),
                name: "Synthetic".to_string(),
                description: String::new(),
                default_language: "English".to_string(),
            })
            .unwrap();
        let project_id = project.id.clone();
        let source = root.join("outside.mp3");
        fs::write(&source, b"audio").unwrap();
        let meeting = repository
            .create_meeting(NewMeetingInput {
                project_id: project.id,
                title: "Synthetic".to_string(),
                occurred_at: "2026-10-25".to_string(),
                language: "English".to_string(),
                source_name: "synthetic.mp3".to_string(),
                source_path: Some(source.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();
        let recording: String = repository
            .connection
            .query_row(
                "SELECT id FROM recordings WHERE meeting_id = ?1",
                [&meeting.id],
                |row| row.get(0),
            )
            .unwrap();
        // A committed meeting has no import waiting; the fixture's freshly created
        // job would otherwise (correctly) make maintenance refuse to run.
        repository
            .connection
            .execute("UPDATE jobs SET state = 'completed'", [])
            .unwrap();
        // A committed row, but the managed file it names was lost.
        repository
            .connection
            .execute(
                "UPDATE recordings SET state = 'committed', managed_path = ?1,
                     checksum = 'abc', byte_count = 5, media_type = 'audio/mpeg'
                 WHERE id = ?2",
                rusqlite::params!["missing/source.mp3", recording],
            )
            .unwrap();
        let normalized = format!(
            "projects/{}/meetings/{}/working/normalized/{recording}-hash.wav",
            project_id, meeting.id
        );
        touch(&root.join(&normalized), 1_000);
        repository
            .connection
            .execute(
                "INSERT INTO normalized_media (recording_id, source_checksum, normalized_path,
                     normalized_checksum, byte_count, duration_ms, audio_codec, sample_rate,
                     channels, runtime_version, settings_json, created_at_ms)
                 VALUES (?1, 'abc', ?2, 'def', 1000, 1000, 'pcm_s16le', 16000, 1, 'v', '{}', 1)",
                rusqlite::params![recording, normalized],
            )
            .unwrap();

        let cleanup = remove_prepared_audio(root).unwrap();
        assert_eq!(cleanup.removed_files, 0);
        assert!(root.join(&normalized).is_file());
        assert_eq!(
            repository
                .connection
                .query_row("SELECT COUNT(*) FROM normalized_media", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            1
        );
    }

    /// With the source present the cache is expendable — including an older copy
    /// that no row points at any more.
    #[test]
    fn prepared_audio_is_removed_when_its_source_is_present() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let mut repository = WorkspaceRepository::open(root).unwrap();
        let project = repository
            .create_project(NewProjectInput {
                names: Vec::new(),
                name: "Synthetic".to_string(),
                description: String::new(),
                default_language: "English".to_string(),
            })
            .unwrap();
        let source = root.join("outside.mp3");
        fs::write(&source, b"audio").unwrap();
        let meeting = repository
            .create_meeting(NewMeetingInput {
                project_id: project.id.clone(),
                title: "Synthetic".to_string(),
                occurred_at: "2026-10-25".to_string(),
                language: "English".to_string(),
                source_name: "synthetic.mp3".to_string(),
                source_path: Some(source.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();
        let recording: String = repository
            .connection
            .query_row(
                "SELECT id FROM recordings WHERE meeting_id = ?1",
                [&meeting.id],
                |row| row.get(0),
            )
            .unwrap();
        repository
            .connection
            .execute("UPDATE jobs SET state = 'completed'", [])
            .unwrap();
        let managed = format!(
            "projects/{}/meetings/{}/recordings/{recording}.opus",
            project.id, meeting.id
        );
        touch(&root.join(&managed), 400);
        repository
            .connection
            .execute(
                "UPDATE recordings SET state = 'committed', managed_path = ?1,
                     checksum = 'abc', byte_count = 400, media_type = 'audio/opus'
                 WHERE id = ?2",
                rusqlite::params![managed, recording],
            )
            .unwrap();
        let normalized = format!(
            "projects/{}/meetings/{}/working/normalized/{recording}-hash.wav",
            project.id, meeting.id
        );
        let orphan = format!(
            "projects/{}/meetings/{}/working/normalized/{recording}-older.wav",
            project.id, meeting.id
        );
        touch(&root.join(&normalized), 1_000);
        touch(&root.join(&orphan), 500);
        repository
            .connection
            .execute(
                "INSERT INTO normalized_media (recording_id, source_checksum, normalized_path,
                     normalized_checksum, byte_count, duration_ms, audio_codec, sample_rate,
                     channels, runtime_version, settings_json, created_at_ms)
                 VALUES (?1, 'abc', ?2, 'def', 1000, 1000, 'pcm_s16le', 16000, 1, 'v', '{}', 1)",
                rusqlite::params![recording, normalized],
            )
            .unwrap();

        let cleanup = remove_prepared_audio(root).unwrap();
        assert_eq!(cleanup.removed_files, 2);
        assert_eq!(cleanup.freed_bytes, 1_500);
        assert!(!root.join(&normalized).exists());
        assert!(!root.join(&orphan).exists());
        assert!(root.join(&managed).is_file());
        assert_eq!(cleanup.usage.prepared_audio_bytes, 0);
        assert_eq!(cleanup.usage.recordings_bytes, 400);
    }
}
