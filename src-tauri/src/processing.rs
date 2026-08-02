//! Durable fake transcription and protocol generation through production-shaped boundaries.

use crate::domain::{TranscriptSegment, WorkspaceSnapshot};
use crate::media;
use crate::runtime;
use crate::storage::{
    ProcessingJobRecord, Result as StorageResult, StorageError, TranscriptArtifact,
    WorkspaceRepository, checksum_bytes, managed_relative_path, new_id, unix_time_millis,
    validate_transcript_artifact,
};
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const FAKE_PROVIDER: &str = "localog-deterministic-fake";
const FAKE_RUNTIME_VERSION: &str = "1";
const FAKE_MODEL_DIGEST: &str = "sha256:localog-synthetic-model-v1";
const STYLE_REVISION: &str = "style-formal@1";
const VOCABULARY_REVISION: &str = "synthetic-empty@1";

struct TranscriptionRequest<'a> {
    job: &'a ProcessingJobRecord,
    language: &'a str,
    verified_source_checksum: &'a str,
}

struct GenerationRequest<'a> {
    meeting_title: &'a str,
    transcript: &'a TranscriptArtifact,
}

/// A real runtime will implement this same narrow, stage-specific port.
trait TranscriptionAdapter {
    fn transcribe(
        &self,
        request: TranscriptionRequest<'_>,
        cancellation: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<(), ProcessingError>,
    ) -> Result<TranscriptArtifact, ProcessingError>;
}

/// Protocol generation remains a separate capability instead of a generic model plugin.
trait ProtocolGenerationAdapter {
    fn generate(
        &self,
        request: GenerationRequest<'_>,
        cancellation: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<(), ProcessingError>,
    ) -> Result<String, ProcessingError>;
}

struct DeterministicFakeAdapter {
    fail_requested: bool,
}

impl TranscriptionAdapter for DeterministicFakeAdapter {
    fn transcribe(
        &self,
        request: TranscriptionRequest<'_>,
        cancellation: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<(), ProcessingError>,
    ) -> Result<TranscriptArtifact, ProcessingError> {
        progress(20, "preparing_fake_transcriber")?;
        fake_step(request.job, cancellation)?;
        progress(48, "transcribing_synthetic_segments")?;
        fake_step(request.job, cancellation)?;
        if self.fail_requested {
            return Err(ProcessingError::InjectedFailure);
        }
        Ok(deterministic_transcript(
            request.job,
            request.language,
            request.verified_source_checksum,
        ))
    }
}

impl ProtocolGenerationAdapter for DeterministicFakeAdapter {
    fn generate(
        &self,
        request: GenerationRequest<'_>,
        cancellation: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<(), ProcessingError>,
    ) -> Result<String, ProcessingError> {
        progress(24, "resolving_protocol_inputs")?;
        // The fake uses resolved snapshots only for provenance; no arbitrary prompt surface exists.
        fake_step_placeholder(cancellation)?;
        progress(56, "generating_protocol")?;
        fake_step_placeholder(cancellation)?;
        if self.fail_requested {
            return Err(ProcessingError::InjectedFailure);
        }
        Ok(deterministic_protocol(
            request.meeting_title,
            request.transcript,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProcessingOutcome {
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug)]
enum ProcessingError {
    Cancelled,
    InjectedFailure,
    Storage(StorageError),
    Io(std::io::Error),
    InvalidOutput,
    Runtime { code: &'static str, message: String },
}

impl From<StorageError> for ProcessingError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<std::io::Error> for ProcessingError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for ProcessingError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::Sql(error))
    }
}

pub(crate) fn queue_transcription(
    root: &Path,
    meeting_id: &str,
    fail_requested: bool,
) -> StorageResult<(ProcessingJobRecord, WorkspaceSnapshot)> {
    let repository = WorkspaceRepository::open(root)?;
    ensure_no_active_processing(&repository)?;
    let (project_id, recording_id, lifecycle): (String, String, String) = repository
        .connection
        .query_row(
            "SELECT m.project_id, r.id, m.lifecycle
             FROM meetings m JOIN recordings r ON r.meeting_id = m.id
             WHERE m.id = ?1 AND r.state = 'committed'
             ORDER BY r.created_at_ms LIMIT 1",
            [meeting_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or(StorageError::MissingMeeting)?;
    if !matches!(
        lifecycle.as_str(),
        "source_ready" | "transcript_ready" | "protocol_draft" | "reviewed"
    ) {
        return Err(StorageError::InvalidData(
            "Commit the meeting source before transcription.",
        ));
    }

    let job_id = new_id("job");
    let revision_id = new_id("transcript");
    let final_relative_path = meeting_root(&project_id, meeting_id)
        .join("transcripts/revisions")
        .join(format!("{revision_id}.json"));
    let configured_executable = repository.read_setting("transcription.whisperExecutable")?;
    let configured_model = repository.read_setting("transcription.whisperModel")?;
    let provider = if cfg!(test) {
        FAKE_PROVIDER
    } else if configured_executable.is_some() && configured_model.is_some() {
        "whisper.cpp"
    } else {
        FAKE_PROVIDER
    };
    let runtime_version = configured_executable
        .as_deref()
        .map(Path::new)
        .and_then(runtime::executable_version)
        .unwrap_or_else(|| FAKE_RUNTIME_VERSION.to_string());
    let model_digest = configured_model
        .as_deref()
        .map(Path::new)
        .and_then(|path| runtime::model_provenance(path).ok())
        .map(|value| value.digest)
        .unwrap_or_else(|| FAKE_MODEL_DIGEST.to_string());
    let settings_json = if provider == "whisper.cpp" {
        r#"{"language":"meeting","timestamps":"segments","normalization":{"sampleRate":16000,"channels":1}}"#
    } else {
        r#"{"language":"meeting","timestamps":"segments"}"#
    };
    let now = unix_time_millis();
    repository.connection.execute(
        "INSERT INTO jobs (
            id, meeting_id, recording_id, kind, state, stage, progress_bytes,
            total_bytes, attempt, duplicate_allowed, result_revision_id,
            provider, runtime_version, model_digest, settings_json,
            style_revision, vocabulary_revision, final_relative_path,
            fail_requested, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, 'transcription', 'queued', 'transcription_queued',
                   0, 100, 1, 0, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
        params![
            job_id,
            meeting_id,
            recording_id,
            revision_id,
            provider,
            runtime_version,
            model_digest,
            settings_json,
            STYLE_REVISION,
            VOCABULARY_REVISION,
            managed_relative_path(&final_relative_path)?,
            i64::from(fail_requested),
            now,
        ],
    )?;
    let job = processing_job(&repository, &job_id)?;
    Ok((job, repository.workspace_snapshot()?))
}

pub(crate) fn queue_generation(
    root: &Path,
    meeting_id: &str,
    fail_requested: bool,
) -> StorageResult<(ProcessingJobRecord, WorkspaceSnapshot)> {
    let mut repository = WorkspaceRepository::open(root)?;
    ensure_no_active_processing(&repository)?;
    let transcript_revision_id = commit_transcript_working_if_dirty(&mut repository, meeting_id)?;
    let (project_id, recording_id, style_id): (String, String, String) = repository
        .connection
        .query_row(
            "SELECT m.project_id, t.recording_id, m.style_id
             FROM meetings m JOIN transcript_revisions t ON t.id = ?2
             WHERE m.id = ?1",
            params![meeting_id, transcript_revision_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or(StorageError::MissingMeeting)?;
    let job_id = new_id("job");
    let revision_id = new_id("protocol");
    let final_relative_path = meeting_root(&project_id, meeting_id)
        .join("protocols/revisions")
        .join(format!("{revision_id}.md"));
    let now = unix_time_millis();
    repository.connection.execute(
        "INSERT INTO jobs (
            id, meeting_id, recording_id, kind, state, stage, progress_bytes,
            total_bytes, attempt, duplicate_allowed, input_revision_id,
            result_revision_id, provider, runtime_version, model_digest,
            settings_json, style_revision, vocabulary_revision,
            final_relative_path, fail_requested, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, 'generation', 'queued', 'generation_queued',
                   0, 100, 1, 0, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
        params![
            job_id,
            meeting_id,
            recording_id,
            transcript_revision_id,
            revision_id,
            FAKE_PROVIDER,
            FAKE_RUNTIME_VERSION,
            FAKE_MODEL_DIGEST,
            serde_json::json!({ "styleId": style_id, "temperature": 0 }).to_string(),
            format!("{style_id}@1"),
            VOCABULARY_REVISION,
            managed_relative_path(&final_relative_path)?,
            i64::from(fail_requested),
            now,
        ],
    )?;
    let job = processing_job(&repository, &job_id)?;
    Ok((job, repository.workspace_snapshot()?))
}

pub(crate) fn run_job(
    root: &Path,
    job_id: &str,
    cancellation: Arc<AtomicBool>,
    notify: impl Fn(WorkspaceSnapshot),
) -> StorageResult<ProcessingOutcome> {
    let result = execute_job(root, job_id, &cancellation, &notify);
    match result {
        Ok(()) => Ok(ProcessingOutcome::Completed),
        Err(ProcessingError::Cancelled) => {
            finish_non_success(root, job_id, "cancelled", "cancelled", None, None, &notify)?;
            Ok(ProcessingOutcome::Cancelled)
        }
        Err(error) => {
            let code = match &error {
                ProcessingError::InjectedFailure => "synthetic_failure",
                ProcessingError::InvalidOutput => "invalid_adapter_output",
                ProcessingError::Io(io_error) if io_error.raw_os_error() == Some(28) => {
                    "insufficient_space"
                }
                ProcessingError::Storage(storage_error) => {
                    let _ = storage_error;
                    "processing_failed"
                }
                ProcessingError::Io(_) => "processing_failed",
                ProcessingError::Cancelled => "cancelled",
                ProcessingError::Runtime { code, .. } => code,
            };
            let message = match &error {
                ProcessingError::Runtime { message, .. } => Some(message.as_str()),
                _ => None,
            };
            finish_non_success(
                root,
                job_id,
                "failed",
                "failed",
                Some(code),
                message,
                &notify,
            )?;
            Ok(ProcessingOutcome::Failed)
        }
    }
}

fn execute_job(
    root: &Path,
    job_id: &str,
    cancellation: &AtomicBool,
    notify: &impl Fn(WorkspaceSnapshot),
) -> Result<(), ProcessingError> {
    let repository = WorkspaceRepository::open(root)?;
    let job = processing_job(&repository, job_id)?;
    repository.connection.execute(
        "UPDATE jobs SET state = 'running', stage = ?1, progress_bytes = 4,
                started_at_ms = COALESCE(started_at_ms, ?2), updated_at_ms = ?2,
                error_code = NULL, error_message = NULL, finished_at_ms = NULL
         WHERE id = ?3 AND state IN ('queued', 'failed', 'cancelled', 'interrupted')",
        params![
            if job.kind == "transcription" {
                "checking_source"
            } else {
                "checking_transcript"
            },
            unix_time_millis(),
            job.id
        ],
    )?;
    notify(repository.workspace_snapshot()?);

    if job.kind == "transcription" {
        execute_transcription(root, &repository, &job, cancellation, notify)
    } else if job.kind == "generation" {
        execute_generation(root, &repository, &job, cancellation, notify)
    } else {
        Err(ProcessingError::InvalidOutput)
    }
}

fn execute_transcription(
    root: &Path,
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    cancellation: &AtomicBool,
    notify: &impl Fn(WorkspaceSnapshot),
) -> Result<(), ProcessingError> {
    let (source_path, expected_checksum, language): (String, String, String) =
        repository.connection.query_row(
            "SELECT r.managed_path, r.checksum, m.language
             FROM recordings r JOIN meetings m ON m.id = r.meeting_id
             WHERE r.id = ?1 AND r.state = 'committed'",
            [&job.recording_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
    verify_streamed_checksum(root, &source_path, &expected_checksum, cancellation)?;
    let mut report = |value, stage| progress(repository, job, value, stage, notify);
    let artifact = if cfg!(test) {
        DeterministicFakeAdapter {
            fail_requested: job.fail_requested,
        }
        .transcribe(
            TranscriptionRequest {
                job,
                language: &language,
                verified_source_checksum: &expected_checksum,
            },
            cancellation,
            &mut report,
        )?
    } else {
        execute_real_transcription(
            root,
            repository,
            job,
            &source_path,
            &expected_checksum,
            &language,
            cancellation,
            &mut report,
        )?
    };
    validate_transcript_artifact(&artifact, &job.meeting_id)?;
    let bytes = serde_json::to_vec_pretty(&artifact).map_err(|_| ProcessingError::InvalidOutput)?;
    progress(repository, job, 76, "validating_transcript", notify)?;
    commit_transcript_output(root, repository, job, &artifact, &bytes)?;
    notify(WorkspaceRepository::open(root)?.workspace_snapshot()?);
    Ok(())
}

fn execute_generation(
    root: &Path,
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    cancellation: &AtomicBool,
    notify: &impl Fn(WorkspaceSnapshot),
) -> Result<(), ProcessingError> {
    let transcript_id = job
        .input_revision_id
        .as_deref()
        .ok_or(ProcessingError::InvalidOutput)?;
    let (path, checksum, meeting_title, style_id): (String, String, String, String) =
        repository.connection.query_row(
            "SELECT t.artifact_path, t.checksum, m.title, m.style_id
             FROM transcript_revisions t JOIN meetings m ON m.id = t.meeting_id
             WHERE t.id = ?1 AND t.meeting_id = ?2",
            params![transcript_id, job.meeting_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
    let bytes = read_verified(root, &path, &checksum)?;
    let transcript: TranscriptArtifact =
        serde_json::from_slice(&bytes).map_err(|_| ProcessingError::InvalidOutput)?;
    validate_transcript_artifact(&transcript, &job.meeting_id)?;
    let adapter = DeterministicFakeAdapter {
        fail_requested: job.fail_requested,
    };
    let mut report = |value, stage| progress(repository, job, value, stage, notify);
    let markdown = adapter.generate(
        GenerationRequest {
            meeting_title: &meeting_title,
            transcript: &transcript,
        },
        cancellation,
        &mut report,
    )?;
    if markdown.trim().is_empty() || markdown.len() > 5_000_000 {
        return Err(ProcessingError::InvalidOutput);
    }
    progress(repository, job, 82, "validating_protocol", notify)?;
    commit_protocol_output(
        root,
        repository,
        job,
        transcript_id,
        &checksum,
        &style_id,
        markdown.as_bytes(),
    )?;
    notify(WorkspaceRepository::open(root)?.workspace_snapshot()?);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn execute_real_transcription(
    root: &Path,
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    source_relative: &str,
    source_checksum: &str,
    language: &str,
    cancellation: &AtomicBool,
    report: &mut dyn FnMut(u64, &'static str) -> Result<(), ProcessingError>,
) -> Result<TranscriptArtifact, ProcessingError> {
    let executable = repository
        .read_setting("transcription.whisperExecutable")?
        .map(PathBuf::from)
        .ok_or_else(|| ProcessingError::Runtime {
            code: "runtime_missing",
            message: "Choose a whisper.cpp executable in Settings → Transcription.".into(),
        })?;
    let model = repository
        .read_setting("transcription.whisperModel")?
        .map(PathBuf::from)
        .ok_or_else(|| ProcessingError::Runtime {
            code: "model_missing",
            message: "Choose a whisper.cpp model in Settings → Transcription.".into(),
        })?;
    let config = runtime::validate_config(&executable, &model).map_err(|message| {
        ProcessingError::Runtime {
            code: "runtime_missing",
            message,
        }
    })?;
    let ffprobe = find_tool("ffprobe").ok_or_else(|| ProcessingError::Runtime {
        code: "media_probe_failed",
        message: "Install FFprobe to inspect imported media.".into(),
    })?;
    let ffmpeg = find_tool("ffmpeg").ok_or_else(|| ProcessingError::Runtime {
        code: "normalization_failed",
        message: "Install FFmpeg to prepare local transcription audio.".into(),
    })?;
    let source = root.join(source_relative);
    report(10, "probing_media")?;
    let probe = media::probe(&ffprobe, &source, cancellation).map_err(|message| {
        ProcessingError::Runtime {
            code: "media_probe_failed",
            message,
        }
    })?;
    let settings = r#"{"sampleRate":16000,"channels":1,"format":"wav"}"#;
    let settings_hash = &checksum_bytes(settings.as_bytes())[..16];
    let normalized_relative = meeting_root(&job.project_id, &job.meeting_id)
        .join("working/normalized")
        .join(format!("{}-{settings_hash}.wav", job.recording_id));
    let normalized = root.join(&normalized_relative);
    let audio_stream = probe
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"));
    let source_sample_rate = audio_stream
        .and_then(|stream| stream.sample_rate.as_deref())
        .and_then(|value| value.parse::<u32>().ok());
    let source_channels = audio_stream.and_then(|stream| stream.channels);
    let cached: Option<(String, String)> = repository
        .connection
        .query_row(
            "SELECT source_checksum, normalized_path FROM normalized_media WHERE recording_id = ?1",
            [&job.recording_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if cached.as_ref().is_none_or(|(checksum, path)| {
        checksum != source_checksum
            || path != normalized_relative.to_str().unwrap_or_default()
            || !root.join(path).is_file()
    }) {
        report(25, "normalizing_audio")?;
        media::normalize(&ffmpeg, &source, &normalized, cancellation, |value| {
            let _ = report(25 + value / 3, "normalizing_audio");
        })
        .map_err(|message| ProcessingError::Runtime {
            code: "normalization_failed",
            message,
        })?;
        let normalized_bytes = fs::read(&normalized)?;
        repository.connection.execute(
            "INSERT INTO normalized_media (recording_id, source_checksum, normalized_path, normalized_checksum, byte_count, duration_ms, audio_codec, sample_rate, channels, runtime_version, settings_json, created_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(recording_id) DO UPDATE SET source_checksum=excluded.source_checksum, normalized_path=excluded.normalized_path, normalized_checksum=excluded.normalized_checksum, byte_count=excluded.byte_count, duration_ms=excluded.duration_ms, audio_codec=excluded.audio_codec, sample_rate=excluded.sample_rate, channels=excluded.channels, runtime_version=excluded.runtime_version, settings_json=excluded.settings_json, created_at_ms=excluded.created_at_ms",
            params![job.recording_id, source_checksum, managed_relative_path(&normalized_relative)?, checksum_bytes(&normalized_bytes), normalized_bytes.len() as i64, probe.format.as_ref().and_then(|format| format.duration.as_deref()).and_then(|value| value.parse::<f64>().ok()).map(|value| (value * 1000.0) as i64), audio_stream.and_then(|stream| stream.codec_name.clone()).or_else(|| probe.format.as_ref().and_then(|format| format.format_name.clone())), source_sample_rate, source_channels, runtime::executable_version(&ffmpeg).unwrap_or_else(|| "unknown".into()), settings, unix_time_millis()],
        )?;
    }
    report(65, "loading_transcription_model")?;
    let provenance =
        runtime::model_provenance(&config.model).map_err(|error| ProcessingError::Runtime {
            code: "model_missing",
            message: error.to_string(),
        })?;
    let output_base = root
        .join(meeting_root(&job.project_id, &job.meeting_id))
        .join("working/jobs")
        .join(format!("{}-transcript", job.id));
    fs::create_dir_all(output_base.parent().ok_or(ProcessingError::InvalidOutput)?)?;
    report(70, "transcribing_audio")?;
    let output = runtime::run_process(
        media::whisper_command(&config, &normalized, &output_base, language),
        cancellation,
        runtime::ProcessLimits::with_max_output(2 * 1024 * 1024),
    )
    .map_err(|failure| match failure {
        runtime::ProcessFailure::Cancelled => ProcessingError::Cancelled,
        runtime::ProcessFailure::TimedOut => ProcessingError::Runtime {
            code: "transcription_timeout",
            message: failure.to_string(),
        },
        failure => ProcessingError::Runtime {
            code: "transcription_failed",
            message: failure.to_string(),
        },
    })?;
    let json_path = media::expected_json_path(&output_base);
    let json = fs::read_to_string(&json_path).map_err(|_| ProcessingError::Runtime {
        code: "invalid_transcript_output",
        message: "whisper.cpp did not produce its JSON transcript.".into(),
    })?;
    let artifact = parse_whisper_json(
        &json,
        &job.meeting_id,
        &job.result_revision_id,
        language,
        source_checksum,
    )?;
    report(88, "validating_transcript")?;
    let _ = fs::remove_file(json_path);
    let _ = output.stderr;
    let _ = provenance;
    Ok(artifact)
}

fn parse_whisper_json(
    json: &str,
    meeting_id: &str,
    revision_id: &str,
    language: &str,
    source_checksum: &str,
) -> Result<TranscriptArtifact, ProcessingError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|_| ProcessingError::Runtime {
            code: "invalid_transcript_output",
            message: "The whisper.cpp JSON transcript is invalid.".into(),
        })?;
    let rows = value
        .get("transcription")
        .or_else(|| value.get("segments"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ProcessingError::Runtime {
            code: "invalid_transcript_output",
            message: "The whisper.cpp JSON transcript has no segments.".into(),
        })?;
    let mut segments = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let text = row
            .get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }
        let start = row
            .get("offsets")
            .and_then(|v| v.get("from"))
            .and_then(serde_json::Value::as_u64)
            .or_else(|| row.get("start").and_then(serde_json::Value::as_u64))
            .unwrap_or(0);
        let end = row
            .get("offsets")
            .and_then(|v| v.get("to"))
            .and_then(serde_json::Value::as_u64)
            .or_else(|| row.get("end").and_then(serde_json::Value::as_u64))
            .unwrap_or(start + 1);
        segments.push(TranscriptSegment {
            id: format!(
                "segment-{}-{:04}",
                &source_checksum[..source_checksum.len().min(8)],
                index + 1
            ),
            start_ms: start,
            end_ms: end.max(start + 1),
            speaker: "Speaker 1".into(),
            text,
            needs_review: false,
        });
    }
    Ok(TranscriptArtifact {
        schema_version: 1,
        meeting_id: meeting_id.into(),
        revision_id: revision_id.into(),
        language: language.into(),
        segments,
    })
}

fn find_tool(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
}

fn fake_step(job: &ProcessingJobRecord, cancellation: &AtomicBool) -> Result<(), ProcessingError> {
    for _ in 0..4 {
        if cancellation.load(Ordering::Acquire) {
            return Err(ProcessingError::Cancelled);
        }
        std::thread::sleep(Duration::from_millis(if cfg!(test) { 1 } else { 90 }));
    }
    if job.state == "cancelling" || cancellation.load(Ordering::Acquire) {
        return Err(ProcessingError::Cancelled);
    }
    Ok(())
}

fn fake_step_placeholder(cancellation: &AtomicBool) -> Result<(), ProcessingError> {
    for _ in 0..4 {
        if cancellation.load(Ordering::Acquire) {
            return Err(ProcessingError::Cancelled);
        }
        std::thread::sleep(Duration::from_millis(if cfg!(test) { 1 } else { 90 }));
    }
    Ok(())
}

fn progress(
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    value: u64,
    stage: &str,
    notify: &impl Fn(WorkspaceSnapshot),
) -> Result<(), ProcessingError> {
    repository.connection.execute(
        "UPDATE jobs SET progress_bytes = ?1, stage = ?2, updated_at_ms = ?3
         WHERE id = ?4 AND state IN ('running', 'cancelling')",
        params![value as i64, stage, unix_time_millis(), job.id],
    )?;
    notify(repository.workspace_snapshot()?);
    Ok(())
}

fn deterministic_transcript(
    job: &ProcessingJobRecord,
    language: &str,
    source_checksum: &str,
) -> TranscriptArtifact {
    let marker = &source_checksum[..source_checksum.len().min(8)];
    let rows = [
        (
            0,
            11_400,
            "Speaker 1",
            "We will use today’s review to agree the next practical steps.",
            false,
        ),
        (
            12_000,
            25_800,
            "Speaker 2",
            "The current proposal is workable, but the open cost range should be confirmed.",
            false,
        ),
        (
            26_500,
            40_200,
            "Speaker 1",
            "Please record that the technical note is due before the next review.",
            false,
        ),
        (
            41_000,
            54_700,
            "Speaker 3",
            "I will prepare the note and circulate it to the project team.",
            false,
        ),
        (
            55_400,
            69_100,
            "Speaker 2",
            "The final decision remains open until that information is available.",
            true,
        ),
        (
            70_000,
            82_600,
            "Speaker 1",
            "We will reconvene next week and close the remaining decision.",
            false,
        ),
    ];
    TranscriptArtifact {
        schema_version: 1,
        meeting_id: job.meeting_id.clone(),
        revision_id: job.result_revision_id.clone(),
        language: language.to_string(),
        segments: rows
            .into_iter()
            .enumerate()
            .map(
                |(index, (start_ms, end_ms, speaker, text, needs_review))| TranscriptSegment {
                    id: format!("segment-{marker}-{:02}", index + 1),
                    start_ms,
                    end_ms,
                    speaker: speaker.to_string(),
                    text: text.to_string(),
                    needs_review,
                },
            )
            .collect(),
    }
}

fn deterministic_protocol(title: &str, transcript: &TranscriptArtifact) -> String {
    let open_point = transcript
        .segments
        .iter()
        .find(|segment| segment.needs_review)
        .map(|segment| segment.text.as_str())
        .unwrap_or("The remaining technical information must be confirmed.");
    format!(
        "# {title}\n\n## Purpose\n\nAgree the next practical steps and record the information still required for a final decision.\n\n## Discussion\n\n- The current proposal is workable.\n- The open cost range and technical note must be confirmed.\n- The project team will review the outstanding information next week.\n\n## Decisions\n\nNo final decision was made. The decision remains open until the requested information is available.\n\n## Actions\n\n- **Speaker 3** will prepare and circulate the technical note before the next review.\n- **Project team** will confirm the cost range.\n- **Meeting chair** will reconvene the review next week.\n\n## Point to verify\n\n{open_point}\n"
    )
}

fn commit_transcript_output(
    root: &Path,
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    artifact: &TranscriptArtifact,
    bytes: &[u8],
) -> Result<(), ProcessingError> {
    let checksum = checksum_bytes(bytes);
    let staged = staged_path(root, job, "json");
    write_durable_new(&staged, bytes)?;
    record_staged(repository, job, &checksum, bytes.len())?;
    let final_path = root.join(&job.final_relative_path);
    finalize_staged(&staged, &final_path)?;
    let working_relative =
        meeting_root(&job.project_id, &job.meeting_id).join("transcripts/working.json");
    replace_working_file(root, &working_relative, bytes, None)?;
    let source_checksum: String = repository.connection.query_row(
        "SELECT checksum FROM recordings WHERE id = ?1",
        [&job.recording_id],
        |row| row.get(0),
    )?;
    let language = artifact.language.clone();
    let transaction = repository.connection.unchecked_transaction()?;
    let ordinal: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(ordinal), 0) + 1 FROM transcript_revisions WHERE meeting_id = ?1",
        [&job.meeting_id],
        |row| row.get(0),
    )?;
    transaction.execute(
        "INSERT INTO transcript_revisions (
            id, meeting_id, recording_id, ordinal, artifact_path, checksum,
            byte_count, language, provider, runtime_version, model_digest,
            settings_json, source_checksum, app_version, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            job.result_revision_id,
            job.meeting_id,
            job.recording_id,
            ordinal,
            managed_relative_path(&job.final_relative_path)?,
            checksum,
            bytes.len() as i64,
            language,
            job.provider.as_deref().unwrap_or(FAKE_PROVIDER),
            job.runtime_version
                .as_deref()
                .unwrap_or(FAKE_RUNTIME_VERSION),
            job.model_digest.as_deref().unwrap_or(FAKE_MODEL_DIGEST),
            job.settings_json
                .as_deref()
                .unwrap_or(r#"{"language":"meeting","timestamps":"segments"}"#),
            source_checksum,
            APP_VERSION,
            unix_time_millis(),
        ],
    )?;
    transaction.execute(
        "INSERT INTO transcript_working (
            meeting_id, base_revision_id, artifact_path, checksum, byte_count, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(meeting_id) DO UPDATE SET
            base_revision_id = excluded.base_revision_id,
            artifact_path = excluded.artifact_path, checksum = excluded.checksum,
            byte_count = excluded.byte_count, updated_at_ms = excluded.updated_at_ms",
        params![
            job.meeting_id,
            job.result_revision_id,
            managed_relative_path(&working_relative)?,
            checksum,
            bytes.len() as i64,
            unix_time_millis(),
        ],
    )?;
    complete_job_transaction(&transaction, job, "transcript_ready")?;
    transaction.commit()?;
    cleanup_working_backup(root, &working_relative);
    Ok(())
}

fn commit_protocol_output(
    root: &Path,
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    transcript_revision_id: &str,
    transcript_checksum: &str,
    style_id: &str,
    bytes: &[u8],
) -> Result<(), ProcessingError> {
    let checksum = checksum_bytes(bytes);
    let staged = staged_path(root, job, "md");
    write_durable_new(&staged, bytes)?;
    record_staged(repository, job, &checksum, bytes.len())?;
    let final_path = root.join(&job.final_relative_path);
    finalize_staged(&staged, &final_path)?;
    let working_relative =
        meeting_root(&job.project_id, &job.meeting_id).join("protocols/working.md");
    replace_working_file(root, &working_relative, bytes, None)?;
    let transaction = repository.connection.unchecked_transaction()?;
    let ordinal: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(ordinal), 0) + 1 FROM protocol_revisions WHERE meeting_id = ?1",
        [&job.meeting_id],
        |row| row.get(0),
    )?;
    transaction.execute(
        "INSERT INTO protocol_revisions (
            id, meeting_id, transcript_revision_id, ordinal, artifact_path,
            checksum, byte_count, status, provider, runtime_version, model_digest,
            settings_json, style_id, style_revision, vocabulary_revision,
            transcript_checksum, app_version, restored_from_revision_id, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'draft', ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            job.result_revision_id,
            job.meeting_id,
            transcript_revision_id,
            ordinal,
            managed_relative_path(&job.final_relative_path)?,
            checksum,
            bytes.len() as i64,
            FAKE_PROVIDER,
            FAKE_RUNTIME_VERSION,
            FAKE_MODEL_DIGEST,
            r#"{"temperature":0}"#,
            style_id,
            format!("{style_id}@1"),
            VOCABULARY_REVISION,
            transcript_checksum,
            APP_VERSION,
            Option::<String>::None,
            unix_time_millis(),
        ],
    )?;
    transaction.execute(
        "INSERT INTO protocol_working (
            meeting_id, base_revision_id, reviewed_revision_id, artifact_path,
            checksum, byte_count, updated_at_ms
         ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6)
         ON CONFLICT(meeting_id) DO UPDATE SET
            base_revision_id = excluded.base_revision_id,
            artifact_path = excluded.artifact_path, checksum = excluded.checksum,
            byte_count = excluded.byte_count, updated_at_ms = excluded.updated_at_ms",
        params![
            job.meeting_id,
            job.result_revision_id,
            managed_relative_path(&working_relative)?,
            checksum,
            bytes.len() as i64,
            unix_time_millis(),
        ],
    )?;
    complete_job_transaction(&transaction, job, "protocol_draft")?;
    transaction.commit()?;
    cleanup_working_backup(root, &working_relative);
    Ok(())
}

fn complete_job_transaction(
    transaction: &rusqlite::Transaction<'_>,
    job: &ProcessingJobRecord,
    lifecycle: &str,
) -> StorageResult<()> {
    let now = unix_time_millis();
    transaction.execute(
        "UPDATE meetings SET lifecycle = ?1, updated_at_ms = ?2 WHERE id = ?3",
        params![lifecycle, now, job.meeting_id],
    )?;
    transaction.execute(
        "UPDATE jobs SET state = 'completed', stage = 'completed', progress_bytes = 100,
                total_bytes = 100, error_code = NULL, error_message = NULL,
                updated_at_ms = ?1, finished_at_ms = ?1
         WHERE id = ?2",
        params![now, job.id],
    )?;
    Ok(())
}

pub(crate) fn autosave_transcript_segment(
    root: &Path,
    meeting_id: &str,
    segment_id: &str,
    text: &str,
) -> StorageResult<WorkspaceSnapshot> {
    let repository = WorkspaceRepository::open(root)?;
    let (path, checksum): (String, String) = repository.connection.query_row(
        "SELECT artifact_path, checksum FROM transcript_working WHERE meeting_id = ?1",
        [meeting_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let bytes = read_verified(root, &path, &checksum).map_err(processing_to_storage)?;
    let mut artifact: TranscriptArtifact = serde_json::from_slice(&bytes)
        .map_err(|_| StorageError::InvalidData("The saved transcript is invalid."))?;
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
    segment.needs_review = false;
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
    let (path, checksum): (String, String) = repository.connection.query_row(
        "SELECT artifact_path, checksum FROM transcript_working WHERE meeting_id = ?1",
        [meeting_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let bytes = read_verified(root, &path, &checksum).map_err(processing_to_storage)?;
    let mut artifact: TranscriptArtifact = serde_json::from_slice(&bytes)
        .map_err(|_| StorageError::InvalidData("The saved transcript is invalid."))?;
    for segment in &mut artifact.segments {
        if segment.speaker == speaker {
            segment.speaker = replacement.to_string();
        }
    }
    persist_transcript_working(&repository, meeting_id, &path, &artifact)?;
    repository.workspace_snapshot()
}

fn persist_transcript_working(
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

fn force_protocol_revision(
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

fn commit_transcript_working_if_dirty(
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

fn commit_protocol_bytes(
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

pub(crate) fn request_cancellation(root: &Path, meeting_id: &str) -> StorageResult<String> {
    let repository = WorkspaceRepository::open(root)?;
    let job_id: String = repository
        .connection
        .query_row(
            "SELECT id FROM jobs WHERE meeting_id = ?1 AND kind IN ('transcription', 'generation')
             AND state IN ('queued', 'running', 'interrupted')
             ORDER BY created_at_ms DESC LIMIT 1",
            [meeting_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(StorageError::MissingJob)?;
    repository.connection.execute(
        "UPDATE jobs SET state = 'cancelling', updated_at_ms = ?1 WHERE id = ?2",
        params![unix_time_millis(), job_id],
    )?;
    Ok(job_id)
}

pub(crate) fn retry_job(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<(ProcessingJobRecord, WorkspaceSnapshot)> {
    let repository = WorkspaceRepository::open(root)?;
    let job_id: String = repository
        .connection
        .query_row(
            "SELECT id FROM jobs WHERE meeting_id = ?1 AND kind IN ('transcription', 'generation')
             AND state IN ('queued', 'failed', 'cancelled', 'interrupted')
             ORDER BY created_at_ms DESC LIMIT 1",
            [meeting_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(StorageError::MissingJob)?;
    let another_active = repository
        .connection
        .query_row(
            "SELECT 1 FROM jobs WHERE id != ?1 AND kind IN ('transcription', 'generation')
             AND state IN ('queued', 'running', 'cancelling') LIMIT 1",
            [&job_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if another_active {
        return Err(StorageError::InvalidData(
            "Another local processing job is already active.",
        ));
    }
    repository.connection.execute(
        "UPDATE jobs SET state = 'queued', stage = CASE kind
                WHEN 'transcription' THEN 'transcription_queued' ELSE 'generation_queued' END,
                progress_bytes = 0,
                attempt = attempt + CASE WHEN state = 'queued' THEN 0 ELSE 1 END,
                error_code = NULL,
                error_message = NULL, fail_requested = 0, updated_at_ms = ?1,
                started_at_ms = NULL, finished_at_ms = NULL WHERE id = ?2",
        params![unix_time_millis(), job_id],
    )?;
    let job = processing_job(&repository, &job_id)?;
    Ok((job, repository.workspace_snapshot()?))
}

pub(crate) fn cancel_unstarted(root: &Path, job_id: &str) -> StorageResult<WorkspaceSnapshot> {
    let repository = WorkspaceRepository::open(root)?;
    repository.connection.execute(
        "UPDATE jobs SET state = 'cancelled', stage = 'cancelled', progress_bytes = 0,
                updated_at_ms = ?1, finished_at_ms = ?1
         WHERE id = ?2 AND state IN ('queued', 'cancelling', 'interrupted')",
        params![unix_time_millis(), job_id],
    )?;
    remove_staged(root, &processing_job(&repository, job_id)?);
    repository.workspace_snapshot()
}

pub(crate) fn reconcile(root: &Path) -> StorageResult<WorkspaceSnapshot> {
    let repository = WorkspaceRepository::open(root)?;
    repository.connection.execute(
        "UPDATE jobs SET state = 'interrupted', stage = 'interrupted',
                error_code = 'interrupted', updated_at_ms = ?1, finished_at_ms = ?1
         WHERE kind IN ('transcription', 'generation') AND state IN ('running', 'cancelling')",
        [unix_time_millis()],
    )?;
    reconcile_working_files(&repository, "transcript_working")?;
    reconcile_working_files(&repository, "protocol_working")?;
    let mut statement = repository.connection.prepare(
        "SELECT id FROM jobs WHERE kind IN ('transcription', 'generation') AND state != 'completed'",
    )?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(statement);
    for id in ids {
        let job = processing_job(&repository, &id)?;
        remove_staged(root, &job);
        // A final file is never exposed without its revision transaction in this milestone.
        if root.join(&job.final_relative_path).exists() {
            quarantine_final(root, &job);
        }
    }
    repository.workspace_snapshot()
}

fn reconcile_working_files(repository: &WorkspaceRepository, table: &str) -> StorageResult<()> {
    let sql = format!("SELECT artifact_path, checksum FROM {table}");
    let mut statement = repository.connection.prepare(&sql)?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(statement);
    for (path, expected) in rows {
        let relative = Path::new(&path);
        let current = repository.root.join(relative);
        let previous = backup_path(&current);
        let current_matches = fs::read(&current)
            .map(|bytes| checksum_bytes(&bytes) == expected)
            .unwrap_or(false);
        if current_matches {
            let _ = fs::remove_file(previous);
            continue;
        }
        let previous_matches = fs::read(&previous)
            .map(|bytes| checksum_bytes(&bytes) == expected)
            .unwrap_or(false);
        if previous_matches {
            if current.exists() {
                let _ = fs::remove_file(&current);
            }
            fs::rename(previous, current)?;
        }
    }
    Ok(())
}

fn processing_job(
    repository: &WorkspaceRepository,
    job_id: &str,
) -> StorageResult<ProcessingJobRecord> {
    repository
        .connection
        .query_row(
            "SELECT j.id, j.meeting_id, m.project_id, j.recording_id, j.kind,
                    j.state, j.stage, j.attempt, j.input_revision_id,
                    j.result_revision_id, j.final_relative_path, j.fail_requested,
                    j.provider, j.runtime_version, j.model_digest, j.settings_json
             FROM jobs j JOIN meetings m ON m.id = j.meeting_id
             WHERE j.id = ?1 AND j.kind IN ('transcription', 'generation')",
            [job_id],
            |row| {
                Ok(ProcessingJobRecord {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    project_id: row.get(2)?,
                    recording_id: row.get(3)?,
                    kind: row.get(4)?,
                    state: row.get(5)?,
                    input_revision_id: row.get(8)?,
                    result_revision_id: row.get(9)?,
                    final_relative_path: PathBuf::from(row.get::<_, String>(10)?),
                    fail_requested: row.get::<_, i64>(11)? != 0,
                    provider: row.get(12)?,
                    runtime_version: row.get(13)?,
                    model_digest: row.get(14)?,
                    settings_json: row.get(15)?,
                })
            },
        )
        .optional()?
        .ok_or(StorageError::MissingJob)
}

fn ensure_no_active_processing(repository: &WorkspaceRepository) -> StorageResult<()> {
    let active = repository
        .connection
        .query_row(
            "SELECT 1 FROM jobs WHERE kind IN ('transcription', 'generation')
             AND state IN ('queued', 'running', 'cancelling') LIMIT 1",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if active {
        return Err(StorageError::InvalidData(
            "Another local processing job is already active.",
        ));
    }
    Ok(())
}

fn finish_non_success(
    root: &Path,
    job_id: &str,
    state: &str,
    stage: &str,
    error_code: Option<&str>,
    error_message: Option<&str>,
    notify: &impl Fn(WorkspaceSnapshot),
) -> StorageResult<()> {
    let repository = WorkspaceRepository::open(root)?;
    let job = processing_job(&repository, job_id)?;
    remove_staged(root, &job);
    repository.connection.execute(
        "UPDATE jobs SET state = ?1, stage = ?2, error_code = ?3, error_message = ?4,
                updated_at_ms = ?5, finished_at_ms = ?5 WHERE id = ?6 AND state != 'completed'",
        params![
            state,
            stage,
            error_code,
            error_message,
            unix_time_millis(),
            job_id
        ],
    )?;
    notify(repository.workspace_snapshot()?);
    Ok(())
}

fn record_staged(
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    checksum: &str,
    byte_count: usize,
) -> StorageResult<()> {
    repository.connection.execute(
        "UPDATE jobs SET stage = 'output_staged', result_checksum = ?1,
                result_byte_count = ?2, result_media_type = ?3, updated_at_ms = ?4
         WHERE id = ?5 AND state = 'running'",
        params![
            checksum,
            byte_count as i64,
            if job.kind == "transcription" {
                "application/json"
            } else {
                "text/markdown"
            },
            unix_time_millis(),
            job.id,
        ],
    )?;
    Ok(())
}

fn verify_streamed_checksum(
    root: &Path,
    relative: &str,
    expected: &str,
    cancellation: &AtomicBool,
) -> Result<(), ProcessingError> {
    managed_relative_path(Path::new(relative))?;
    let mut file = File::open(root.join(relative))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 256 * 1024];
    loop {
        if cancellation.load(Ordering::Acquire) {
            return Err(ProcessingError::Cancelled);
        }
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if format!("{:x}", hasher.finalize()) != expected {
        return Err(ProcessingError::InvalidOutput);
    }
    Ok(())
}

fn read_verified(root: &Path, relative: &str, expected: &str) -> Result<Vec<u8>, ProcessingError> {
    managed_relative_path(Path::new(relative))?;
    let bytes = fs::read(root.join(relative))?;
    if checksum_bytes(&bytes) != expected {
        return Err(ProcessingError::InvalidOutput);
    }
    Ok(bytes)
}

fn write_durable_new(path: &Path, bytes: &[u8]) -> Result<(), ProcessingError> {
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

fn replace_working_file(
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

fn cleanup_working_backup(root: &Path, relative: &Path) {
    let path = root.join(relative);
    let _ = fs::remove_file(backup_path(&path));
    let _ = fs::remove_file(next_path(&path));
}

fn finalize_staged(staged: &Path, final_path: &Path) -> Result<(), ProcessingError> {
    fs::create_dir_all(
        final_path
            .parent()
            .ok_or_else(|| std::io::Error::other("missing parent"))?,
    )?;
    fs::rename(staged, final_path)?;
    sync_directory(final_path.parent())?;
    Ok(())
}

fn staged_path(root: &Path, job: &ProcessingJobRecord, extension: &str) -> PathBuf {
    root.join(meeting_root(&job.project_id, &job.meeting_id))
        .join("working/jobs")
        .join(format!("{}.{}.part", job.id, extension))
}

fn remove_staged(root: &Path, job: &ProcessingJobRecord) {
    for extension in ["json", "md"] {
        let _ = fs::remove_file(staged_path(root, job, extension));
    }
}

fn quarantine_final(root: &Path, job: &ProcessingJobRecord) {
    let final_path = root.join(&job.final_relative_path);
    let recovery = root
        .join(meeting_root(&job.project_id, &job.meeting_id))
        .join("working/recovery");
    if fs::create_dir_all(&recovery).is_ok() {
        let _ = fs::rename(final_path, recovery.join(format!("{}.orphan", job.id)));
    }
}

fn meeting_root(project_id: &str, meeting_id: &str) -> PathBuf {
    PathBuf::from("projects")
        .join(project_id)
        .join("meetings")
        .join(meeting_id)
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension(format!(
        "{}.previous",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("work")
    ))
}

fn next_path(path: &Path) -> PathBuf {
    path.with_extension(format!(
        "{}.next",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("work")
    ))
}

#[cfg(unix)]
fn sync_directory(directory: Option<&Path>) -> Result<(), ProcessingError> {
    File::open(directory.ok_or_else(|| std::io::Error::other("missing directory"))?)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_directory: Option<&Path>) -> Result<(), ProcessingError> {
    Ok(())
}

fn processing_to_storage(error: ProcessingError) -> StorageError {
    match error {
        ProcessingError::Storage(error) => error,
        ProcessingError::Io(error) => StorageError::Io(error),
        ProcessingError::Cancelled
        | ProcessingError::InjectedFailure
        | ProcessingError::InvalidOutput
        | ProcessingError::Runtime { .. } => {
            StorageError::InvalidData("The local document operation could not finish.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{JobState, MeetingLifecycle, NewMeetingInput, NewProjectInput};
    use crate::imports;
    use tempfile::{TempDir, tempdir};

    struct Fixture {
        _temporary: TempDir,
        root: PathBuf,
        meeting_id: String,
    }

    impl Fixture {
        fn source_ready() -> Self {
            let temporary = tempdir().unwrap();
            let root = temporary.path().join("managed");
            let source = temporary.path().join("synthetic-workflow.wav");
            fs::write(&source, b"synthetic audio boundary".repeat(40_000)).unwrap();
            let mut repository = WorkspaceRepository::open(&root).unwrap();
            let project = repository
                .create_project(NewProjectInput {
                    name: "Synthetic workflow".to_string(),
                    description: "No real meeting data".to_string(),
                    default_language: "English".to_string(),
                })
                .unwrap();
            let meeting = repository
                .create_meeting(NewMeetingInput {
                    project_id: project.id,
                    title: "Synthetic design review".to_string(),
                    occurred_at: "2026-08-02".to_string(),
                    language: "English".to_string(),
                    source_name: "synthetic-workflow.wav".to_string(),
                    source_path: Some(source.to_string_lossy().into_owned()),
                    style_id: "style-formal".to_string(),
                })
                .unwrap();
            drop(repository);
            imports::run_import(&root, &meeting.id, Arc::new(AtomicBool::new(false)), |_| {})
                .unwrap();
            Self {
                _temporary: temporary,
                root,
                meeting_id: meeting.id,
            }
        }

        fn transcribe(&self) {
            let (job, _) = queue_transcription(&self.root, &self.meeting_id, false).unwrap();
            assert_eq!(
                run_job(
                    &self.root,
                    &job.id,
                    Arc::new(AtomicBool::new(false)),
                    |_| {}
                )
                .unwrap(),
                ProcessingOutcome::Completed
            );
        }
    }

    #[test]
    fn complete_fake_workflow_persists_revisions_autosave_and_review_semantics() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let first = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(
            first.meetings[0].lifecycle,
            MeetingLifecycle::TranscriptReady
        );
        let transcript = &first.transcripts[&fixture.meeting_id];
        assert_eq!(transcript.segments.len(), 6);
        let first_segment = transcript.segments[0].id.clone();

        autosave_transcript_segment(
            &fixture.root,
            &fixture.meeting_id,
            &first_segment,
            "A deliberately corrected synthetic opening statement.",
        )
        .unwrap();
        let edited = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert!(edited.transcripts[&fixture.meeting_id].is_dirty);

        let (generation, _) = queue_generation(&fixture.root, &fixture.meeting_id, false).unwrap();
        let committed_input = generation.input_revision_id.clone().unwrap();
        assert_ne!(committed_input, transcript.base_revision_id);
        run_job(
            &fixture.root,
            &generation.id,
            Arc::new(AtomicBool::new(false)),
            |_| {},
        )
        .unwrap();
        let generated = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(
            generated.meetings[0].lifecycle,
            MeetingLifecycle::ProtocolDraft
        );
        assert_eq!(
            generated.protocols[&fixture.meeting_id].transcript_revision_id,
            committed_input
        );

        let revised_markdown = format!(
            "{}\n## Additional note\n\nSynthetic working edit.\n",
            generated.protocols[&fixture.meeting_id].markdown
        );
        autosave_protocol(&fixture.root, &fixture.meeting_id, &revised_markdown).unwrap();
        let reviewed = mark_protocol_reviewed(&fixture.root, &fixture.meeting_id).unwrap();
        assert_eq!(reviewed.meetings[0].lifecycle, MeetingLifecycle::Reviewed);
        assert_eq!(
            reviewed.protocols[&fixture.meeting_id].review_state,
            "reviewed"
        );
        let reviewed_revision = reviewed.protocols[&fixture.meeting_id].revision_id.clone();

        autosave_protocol(
            &fixture.root,
            &fixture.meeting_id,
            &(revised_markdown + "\nChanged after review.\n"),
        )
        .unwrap();
        let reopened = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(reopened.meetings[0].lifecycle, MeetingLifecycle::Reviewed);
        assert_eq!(
            reopened.protocols[&fixture.meeting_id].review_state,
            "changed_since_review"
        );
        assert_eq!(
            reopened.protocols[&fixture.meeting_id].revision_id,
            reviewed_revision
        );
    }

    #[test]
    fn cancellation_failure_retry_and_interruption_never_advance_lifecycle() {
        let fixture = Fixture::source_ready();
        let (cancelled_job, _) =
            queue_transcription(&fixture.root, &fixture.meeting_id, false).unwrap();
        let cancellation = Arc::new(AtomicBool::new(true));
        assert_eq!(
            run_job(&fixture.root, &cancelled_job.id, cancellation, |_| {}).unwrap(),
            ProcessingOutcome::Cancelled
        );
        let cancelled = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(
            cancelled.meetings[0].lifecycle,
            MeetingLifecycle::SourceReady
        );

        let (retry, _) = retry_job(&fixture.root, &fixture.meeting_id).unwrap();
        run_job(
            &fixture.root,
            &retry.id,
            Arc::new(AtomicBool::new(false)),
            |_| {},
        )
        .unwrap();
        assert_eq!(
            WorkspaceRepository::open(&fixture.root)
                .unwrap()
                .workspace_snapshot()
                .unwrap()
                .meetings[0]
                .lifecycle,
            MeetingLifecycle::TranscriptReady
        );

        let (generation, _) = queue_generation(&fixture.root, &fixture.meeting_id, true).unwrap();
        assert_eq!(
            run_job(
                &fixture.root,
                &generation.id,
                Arc::new(AtomicBool::new(false)),
                |_| {}
            )
            .unwrap(),
            ProcessingOutcome::Failed
        );
        let failed = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(
            failed.meetings[0].lifecycle,
            MeetingLifecycle::TranscriptReady
        );
        assert_eq!(failed.jobs[0].state, JobState::Failed);

        let (interrupted, _) = retry_job(&fixture.root, &fixture.meeting_id).unwrap();
        let repository = WorkspaceRepository::open(&fixture.root).unwrap();
        repository
            .connection
            .execute(
                "UPDATE jobs SET state = 'running' WHERE id = ?1",
                [&interrupted.id],
            )
            .unwrap();
        drop(repository);
        let recovered = reconcile(&fixture.root).unwrap();
        assert_eq!(recovered.jobs[0].state, JobState::Interrupted);
        assert_eq!(
            recovered.meetings[0].lifecycle,
            MeetingLifecycle::TranscriptReady
        );
        assert!(recovered.protocols.is_empty());
    }

    #[test]
    fn restoring_an_older_protocol_creates_a_new_draft_revision() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let (job, _) = queue_generation(&fixture.root, &fixture.meeting_id, false).unwrap();
        run_job(
            &fixture.root,
            &job.id,
            Arc::new(AtomicBool::new(false)),
            |_| {},
        )
        .unwrap();
        let original = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap()
            .protocols[&fixture.meeting_id]
            .revision_id
            .clone();
        autosave_protocol(
            &fixture.root,
            &fixture.meeting_id,
            "# Second synthetic version\n",
        )
        .unwrap();
        create_protocol_revision(&fixture.root, &fixture.meeting_id).unwrap();
        let restored =
            restore_protocol_revision(&fixture.root, &fixture.meeting_id, &original).unwrap();
        let protocol = &restored.protocols[&fixture.meeting_id];
        assert_ne!(protocol.revision_id, original);
        assert_eq!(protocol.review_state, "draft");
        assert_eq!(protocol.revisions.len(), 3);
        assert!(protocol.markdown.contains("## Purpose"));
    }

    #[test]
    fn staged_or_renamed_output_is_never_presented_as_a_revision_after_restart() {
        for after_rename in [false, true] {
            let fixture = Fixture::source_ready();
            let (job, _) = queue_transcription(&fixture.root, &fixture.meeting_id, false).unwrap();
            let repository = WorkspaceRepository::open(&fixture.root).unwrap();
            repository
                .connection
                .execute(
                    "UPDATE jobs SET state = 'running', stage = 'output_staged',
                            result_checksum = 'synthetic', result_byte_count = 9 WHERE id = ?1",
                    [&job.id],
                )
                .unwrap();
            drop(repository);
            let staged = staged_path(&fixture.root, &job, "json");
            fs::create_dir_all(staged.parent().unwrap()).unwrap();
            fs::write(&staged, b"unfinished").unwrap();
            if after_rename {
                let final_path = fixture.root.join(&job.final_relative_path);
                fs::create_dir_all(final_path.parent().unwrap()).unwrap();
                fs::rename(&staged, &final_path).unwrap();
            }

            let recovered = reconcile(&fixture.root).unwrap();
            assert_eq!(
                recovered.meetings[0].lifecycle,
                MeetingLifecycle::SourceReady
            );
            assert!(recovered.transcripts.is_empty());
            assert_eq!(recovered.jobs[0].state, JobState::Interrupted);
            assert!(!staged.exists());
            assert!(!fixture.root.join(&job.final_relative_path).exists());
        }
    }

    #[test]
    fn interrupted_autosave_restores_the_database_acknowledged_working_file() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let repository = WorkspaceRepository::open(&fixture.root).unwrap();
        let (path, checksum): (String, String) = repository
            .connection
            .query_row(
                "SELECT artifact_path, checksum FROM transcript_working WHERE meeting_id = ?1",
                [&fixture.meeting_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let working = fixture.root.join(&path);
        let acknowledged = fs::read(&working).unwrap();
        assert_eq!(checksum_bytes(&acknowledged), checksum);
        fs::rename(&working, backup_path(&working)).unwrap();
        fs::write(&working, b"new bytes not acknowledged by SQLite").unwrap();
        drop(repository);

        let recovered = reconcile(&fixture.root).unwrap();
        assert_eq!(recovered.transcripts[&fixture.meeting_id].segments.len(), 6);
        assert_eq!(fs::read(working).unwrap(), acknowledged);
    }

    #[test]
    fn workspace_location_reopens_the_same_meeting_and_stage() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let repository = WorkspaceRepository::open(&fixture.root).unwrap();
        repository
            .save_workspace_location(&fixture.meeting_id, "transcript")
            .unwrap();
        drop(repository);
        let reopened = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(
            reopened.active_meeting_id.as_deref(),
            Some(&*fixture.meeting_id)
        );
        assert_eq!(reopened.active_route.as_deref(), Some("transcript"));
    }

    #[test]
    fn fake_adapters_are_deterministic_and_obey_failure_and_cancellation_contracts() {
        let fixture = Fixture::source_ready();
        let (job, _) = queue_transcription(&fixture.root, &fixture.meeting_id, false).unwrap();
        let adapter = DeterministicFakeAdapter {
            fail_requested: false,
        };
        let request = || TranscriptionRequest {
            job: &job,
            language: "English",
            verified_source_checksum: "12345678abcdef",
        };
        let mut stages = Vec::new();
        let first = adapter
            .transcribe(request(), &AtomicBool::new(false), &mut |value, stage| {
                stages.push((value, stage));
                Ok(())
            })
            .unwrap();
        let second = adapter
            .transcribe(request(), &AtomicBool::new(false), &mut |_, _| Ok(()))
            .unwrap();
        assert_eq!(first.segments, second.segments);
        assert_eq!(stages.len(), 2);

        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            adapter.transcribe(request(), &cancelled, &mut |_, _| Ok(())),
            Err(ProcessingError::Cancelled)
        ));
        let failing = DeterministicFakeAdapter {
            fail_requested: true,
        };
        assert!(matches!(
            failing.transcribe(request(), &AtomicBool::new(false), &mut |_, _| Ok(())),
            Err(ProcessingError::InjectedFailure)
        ));
    }

    #[test]
    fn whisper_json_maps_segments_without_diarisation() {
        let artifact = parse_whisper_json(
            r#"{"transcription":[{"offsets":{"from":0,"to":1200},"text":" First point "},{"offsets":{"from":1300,"to":2200},"text":"Second point"}]}"#,
            "meeting-1", "revision-1", "English", "abcdef0123456789",
        ).unwrap();
        assert_eq!(artifact.segments.len(), 2);
        assert_eq!(artifact.segments[0].start_ms, 0);
        assert_eq!(artifact.segments[0].speaker, "Speaker 1");
        assert_eq!(artifact.segments[1].text, "Second point");
    }
}
