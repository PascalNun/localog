//! Durable fake transcription and protocol generation through production-shaped boundaries.

use crate::diarisation;
use crate::domain::{SpeakerResolution, TranscriptSegment, WorkspaceSnapshot};
use crate::media;
use crate::models;
use crate::provider;
use crate::runtime;
use crate::storage;
use crate::storage::{
    ProcessingJobRecord, Result as StorageResult, StorageError, TranscriptArtifact,
    WorkspaceRepository, checksum_bytes, managed_relative_path, new_id, unix_time_millis,
    validate_transcript_artifact,
};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// How much room to allow the model for the protocol itself.
///
/// This is a separate limit from the context window and truncates independently
/// of it: the provider reports a run as incomplete when generation stops because
/// the answer hit its ceiling rather than because the model finished.
///
/// The reference protocol written by a person for the evaluation meeting is about
/// 18,000 characters. German runs near four characters to the token here, so a
/// full-length protocol needs somewhere over 4,500 — and the earlier value of
/// 2,048 could not have produced one at any quality, on any model. The value is
/// generous rather than tight because it is a ceiling, not an allocation: what is
/// actually requested is this or whatever the window still has room for,
/// whichever is smaller, so a wide answer budget never costs a narrow one.
const PROTOCOL_OUTPUT_TOKENS: u32 = 8_192;

/// Room for the protocol, sized to what the style asked for.
///
/// A ceiling is not an allocation — what is requested is this or whatever the
/// window has left, whichever is smaller — but it is still a signal. A style that
/// wants only decisions and next steps, handed room for eight thousand tokens,
/// has been invited to fill them.
fn output_tokens_for(density: crate::domain::ProtocolDensity) -> u32 {
    use crate::domain::ProtocolDensity::*;
    match density {
        Comprehensive => PROTOCOL_OUTPUT_TOKENS,
        Concise => PROTOCOL_OUTPUT_TOKENS * 3 / 4,
        Terse => PROTOCOL_OUTPUT_TOKENS / 4,
    }
}

/// The context window to ask a model for.
///
/// Two failure modes sit either side of this number. Too small truncates: the
/// answer budget is what remains of the window after the prompt, so an
/// eighty-one minute meeting against an 8,192-token window left no room to reply
/// and generation failed with "the local model stopped before returning a
/// complete protocol".
///
/// Too large is not free either. Measured on the development machine, the same
/// model spans 3.6 GB resident at 4,096 tokens and 7.3 GB at 131,072 — roughly
/// 30 KB of key-value cache per token — so a model advertising 262,144 would cost
/// gigabytes of context before any weight is loaded, and the baseline machine has
/// eight gigabytes in total.
///
/// So: ask the model what it supports, and cap it at a width that has actually
/// been run. 40,960 is that width — it holds a whole meeting in one pass and
/// measured 4.70 GB resident. Sizing the cap to the machine's own memory rather
/// than to this one is the next step, and is tracked in the plan.
/// How much context this machine can actually give this model.
///
/// A context length is a memory decision before it is a quality one, and this used
/// to ignore that: it asked for 40,960 tokens of everything, a figure measured
/// against one model. Qwen needs about 4.7 GB there. `mistral-nemo` needs 14 GB at
/// the same setting — three times as much for the same number — so on a 16 GB
/// machine Ollama put 18 % of it on the CPU and the request had not returned after
/// thirty minutes, at which point the person waiting was told `timeout: receive
/// response`.
///
/// That is the failure this exists to make impossible. A model that will not fit
/// should be given a context that does, before anybody waits for it.
///
/// The arithmetic is deliberately pessimistic, because being wrong downwards costs
/// a shorter context and being wrong upwards costs half an hour and a meaningless
/// error. Weights are taken as their size on disk, a fixed amount is left for the
/// operating system and the application, and what remains is divided by a per-token
/// cost taken from the most expensive model measured rather than the cheapest.
fn affordable_context(
    provider: &provider::OllamaProvider,
    model: &str,
    model_bytes: u64,
    machine_bytes: u64,
) -> u32 {
    /// Measured on the reference meeting and not exceeded since.
    const MEASURED_AFFORDABLE: u32 = 40_960;
    /// When the provider will not say how wide its context is. Was 8,192, which was
    /// measured on 16 August 2026 to fail three times out of three: at that width the
    /// window cannot hold the folded notes and a whole protocol at once, so a run
    /// ends with the answer cut off. A guess has to be a width that works.
    const WHEN_UNREPORTED: u32 = 16_384;
    /// `mistral-nemo` spends about 170 KB of key-value cache per token where Qwen
    /// spends about 32. The larger is used for every model, because a context that
    /// turns out to fit easily is a smaller disappointment than one that does not
    /// fit at all.
    const BYTES_PER_TOKEN: u64 = 176 * 1024;
    /// For everything that is not this model: the application, the interface, the
    /// transcription runtime that may still be resident, and the operating system.
    const RESERVED: u64 = 3 * 1024 * 1024 * 1024;
    /// The narrowest width measured to produce a protocol at all. 8,192 fails at
    /// every seed and 4,096 — what this used to be — is narrower still, so a machine
    /// too small for this is better served by being slow than by being handed a
    /// window that cannot finish. Quality does not vary with the window; only time
    /// does, which makes slow the right way to be wrong.
    const LEAST_USEFUL: u32 = 16_384;

    let supported = provider
        .model_context_length(model)
        .unwrap_or(WHEN_UNREPORTED)
        .min(MEASURED_AFFORDABLE);
    if machine_bytes == 0 || model_bytes == 0 {
        return supported;
    }
    let spare = machine_bytes
        .saturating_sub(RESERVED)
        .saturating_sub(model_bytes);
    let affordable = u32::try_from(spare / BYTES_PER_TOKEN).unwrap_or(u32::MAX);
    supported.min(affordable).max(LEAST_USEFUL)
}

/// How much memory this machine has, or zero where that cannot be established —
/// in which case nothing is inferred from it.
fn machine_memory_bytes() -> u64 {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|text| text.trim().parse().ok())
            .unwrap_or(0)
    }
    // Windows and Linux report this differently; until each is read properly the
    // context falls back to the conservative constant rather than to a guess.
    #[cfg(not(target_os = "macos"))]
    {
        0
    }
}

/// Whether this build should use the deterministic adapters instead of the real
/// runtimes.
///
/// The test suite has to pass on a machine with no whisper.cpp, no diariser and no
/// Ollama, and must not depend on what a model happens to say, so a test build
/// substitutes deterministic adapters everywhere.
///
/// That has one consequence worth stating: no test in this crate can exercise the
/// real pipeline, which is why the whole-pipeline harness exists and why it has to
/// ask for the real runtimes by name. Outside a test build this is always false, so
/// the variable cannot change anything for someone running the application.
fn use_synthetic_adapters() -> bool {
    cfg!(test) && std::env::var_os("LOCALOG_REAL_RUNTIMES").is_none()
}

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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct QueuedGenerationConfig {
    model: String,
    model_digest: String,
    runtime_version: String,
    meeting_language: String,
    style: provider::GenerationStyle,
    vocabulary_revision: String,
    vocabulary: Vec<String>,
    seed: u64,
    temperature_milli: u16,
    context_tokens: u32,
    maximum_output_tokens: u32,
}

struct NormalizedCacheRecord {
    source_checksum: String,
    normalized_path: String,
    normalized_checksum: String,
    byte_count: i64,
    runtime_version: String,
    settings_json: String,
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

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(formatter, "processing was cancelled"),
            Self::InjectedFailure => write!(formatter, "the synthetic adapter failed"),
            Self::Storage(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::InvalidOutput => write!(formatter, "the adapter output was invalid"),
            Self::Runtime { message, .. } => write!(formatter, "{message}"),
        }
    }
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

fn provider_processing_error(error: provider::ProviderError) -> ProcessingError {
    match error {
        provider::ProviderError::Cancelled => ProcessingError::Cancelled,
        provider::ProviderError::ModelMissing(_) => ProcessingError::Runtime {
            code: "provider_model_missing",
            message: "The selected Ollama model is no longer installed. Choose another model and retry.".into(),
        },
        provider::ProviderError::ModelChanged => ProcessingError::Runtime {
            code: "provider_model_changed",
            message: "The selected Ollama model changed after this job was queued. Retry to resolve it again.".into(),
        },
        provider::ProviderError::RuntimeChanged => ProcessingError::Runtime {
            code: "provider_runtime_changed",
            message: "The Ollama runtime changed after this job was queued. Retry to resolve it again.".into(),
        },
        provider::ProviderError::Unavailable(message) => ProcessingError::Runtime {
            code: "provider_unavailable",
            message: format!("Ollama could not complete the local request: {message}"),
        },
        provider::ProviderError::InvalidResponse(message) => ProcessingError::Runtime {
            code: "provider_invalid_output",
            message,
        },
        provider::ProviderError::ResponseTooLarge => ProcessingError::Runtime {
            code: "provider_response_too_large",
            message: "The local model response exceeded the safe limit and was not committed.".into(),
        },
        provider::ProviderError::IncompleteResponse => ProcessingError::Runtime {
            code: "provider_incomplete_output",
            message: "The local model stopped before returning a complete protocol.".into(),
        },
        // Said in terms of what happened and what to do, because the alternative is
        // a person watching a progress bar that will never move again.
        provider::ProviderError::Stalled => ProcessingError::Runtime {
            code: "provider_stalled",
            message: "The local model stopped responding partway through and the draft was not \
                      committed. This usually means it needs more memory than is free — a smaller \
                      model, or closing other applications, should let it finish."
                .into(),
        },
    }
}

/// What somebody asked for about speakers when they started a transcription.
///
/// Three answers rather than a number that might be missing. Leaving them together
/// is a choice somebody made, not an absence of one, and the pass must not run
/// because the models happen to be installed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Speakers {
    /// Leave the transcript with one label.
    Together,
    /// Separate them, and work out how many there were.
    Separate,
    /// Separate them; this many people spoke.
    SeparateInto(u32),
}

impl Speakers {
    fn count(self) -> Option<u32> {
        match self {
            Self::SeparateInto(count) => Some(count),
            _ => None,
        }
    }

    fn wanted(self) -> bool {
        !matches!(self, Self::Together)
    }
}

/// Queue transcription without naming a speaker count.
///
/// Used by the tests, which are about the pipeline rather than about speakers.
/// Marked for test builds because the application always has a count to pass,
/// even when that count is nothing.
#[cfg(test)]
pub(crate) fn queue_transcription(
    root: &Path,
    meeting_id: &str,
    fail_requested: bool,
) -> StorageResult<(ProcessingJobRecord, WorkspaceSnapshot)> {
    queue_transcription_with_expected(root, meeting_id, fail_requested, Speakers::Together)
}

pub(crate) fn queue_transcription_with_expected(
    root: &Path,
    meeting_id: &str,
    fail_requested: bool,
    speakers: Speakers,
) -> StorageResult<(ProcessingJobRecord, WorkspaceSnapshot)> {
    if let Some(count) = speakers.count()
        && !(2..=64).contains(&count)
    {
        return Err(StorageError::InvalidData(
            "Expected speakers must be between 2 and 64.",
        ));
    }
    let mut repository = WorkspaceRepository::open(root)?;
    ensure_no_active_processing(&repository.connection)?;
    let (project_id, recording_id, lifecycle, language): (String, String, String, String) =
        repository
            .connection
            .query_row(
                "SELECT m.project_id, r.id, m.lifecycle, m.language
             FROM meetings m JOIN recordings r ON r.meeting_id = m.id
             WHERE m.id = ?1 AND r.state = 'committed'
             ORDER BY r.created_at_ms LIMIT 1",
                [meeting_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
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
    let (provider, runtime_version, model_digest, settings_json, runtime_config_json) =
        transcription_metadata(&repository, &language, use_synthetic_adapters())?;
    let settings_json = transcription_settings_with_speakers(&settings_json, speakers);
    let now = unix_time_millis();
    let transaction = repository
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_no_active_processing(&transaction)?;
    transaction.execute(
        "INSERT INTO jobs (
            id, meeting_id, recording_id, kind, state, stage, progress_bytes,
            total_bytes, attempt, duplicate_allowed, result_revision_id,
            provider, runtime_version, model_digest, settings_json,
            runtime_config_json, style_revision, vocabulary_revision, final_relative_path,
            fail_requested, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, 'transcription', 'queued', 'transcription_queued',
                   0, 100, 1, 0, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
        params![
            job_id,
            meeting_id,
            recording_id,
            revision_id,
            provider,
            runtime_version,
            model_digest,
            settings_json,
            runtime_config_json,
            STYLE_REVISION,
            VOCABULARY_REVISION,
            managed_relative_path(&final_relative_path)?,
            i64::from(fail_requested),
            now,
        ],
    )?;
    transaction.commit()?;
    let job = processing_job(&repository, &job_id)?;
    Ok((job, repository.workspace_snapshot()?))
}

fn transcription_settings_with_speakers(settings_json: &str, speakers: Speakers) -> String {
    let mut settings = serde_json::from_str::<serde_json::Value>(settings_json)
        .unwrap_or_else(|_| serde_json::json!({}));
    if let Some(object) = settings.as_object_mut() {
        // Both are written. Whether separation was wanted and how many people
        // there were are different questions, and a job recorded before the
        // second could be answered without the first is a job nobody can read
        // back correctly.
        object.insert(
            "separateSpeakers".to_string(),
            serde_json::json!(speakers.wanted()),
        );
        object.insert(
            "expectedSpeakers".to_string(),
            match speakers.count() {
                Some(count) => serde_json::json!(count),
                None => serde_json::Value::Null,
            },
        );
    }
    serde_json::to_string(&settings).unwrap_or_else(|_| settings_json.to_string())
}

/// Name only what is actually missing, so the message points at one next action.
fn missing_runtime_message(repository: &WorkspaceRepository) -> String {
    let preset = repository
        .read_setting("transcription.preset")
        .ok()
        .flatten()
        .filter(|value| models::is_known_preset(value))
        .unwrap_or_else(|| models::DEFAULT_PRESET.to_string());
    let model_ready = models::model_path_for_preset(&repository.root, &preset).is_some();
    let executable_ready = repository
        .read_setting("transcription.whisperExecutable")
        .ok()
        .flatten()
        .map(PathBuf::from)
        .is_some_and(|path| path.is_file())
        || runtime::discover_executable(runtime::WHISPER_NAMES).is_some();
    match (model_ready, executable_ready) {
        (false, _) => format!(
            "Download the {preset} transcription quality in Settings → Transcription, then try again."
        ),
        (true, false) => {
            "Choose a whisper.cpp executable in Settings → Transcription → advanced details.".into()
        }
        (true, true) => {
            "The transcription runtime could not be prepared. Try again from Settings → Transcription.".into()
        }
    }
}

fn transcription_metadata(
    repository: &WorkspaceRepository,
    language: &str,
    use_fake: bool,
) -> StorageResult<(&'static str, String, String, String, Option<String>)> {
    // Queueing captures the exact runtime inputs that execution must validate later.
    let language_code = transcription_language_code(language);
    if use_fake {
        return Ok((
            FAKE_PROVIDER,
            FAKE_RUNTIME_VERSION.to_string(),
            FAKE_MODEL_DIGEST.to_string(),
            r#"{"language":"meeting","timestamps":"segments"}"#.to_string(),
            None,
        ));
    }

    let executable = repository
        .read_setting("transcription.whisperExecutable")?
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| runtime::discover_executable(runtime::WHISPER_NAMES));
    // The user chooses a quality preset; the model is resolved from what is
    // installed for it, never from a user-entered path.
    let preset = repository
        .read_setting("transcription.preset")?
        .filter(|value| models::is_known_preset(value))
        .unwrap_or_else(|| models::DEFAULT_PRESET.to_string());
    let model = models::model_path_for_preset(&repository.root, &preset);
    let resolved = executable.zip(model).and_then(|(executable, model)| {
        let runtime_config = runtime::validate_config(&executable, &model).ok()?;
        let runtime_version = runtime::executable_version(&runtime_config.executable)?;
        let provenance = cached_model_provenance(repository, &runtime_config.model).ok()?;
        Some(runtime::ResolvedTranscriptionConfig {
            executable_path: runtime_config.executable,
            model_path: runtime_config.model,
            runtime_version,
            model_digest: provenance.digest,
            model_byte_count: provenance.byte_count,
            language_code: language_code.clone(),
            sample_rate: 16_000,
            channels: 1,
            codec: "pcm_s16le".to_string(),
            container: "wav".to_string(),
        })
    });
    let settings_json = serde_json::json!({
        "language": language,
        "languageCode": language_code,
        "timestamps": "segments",
        "normalization": { "sampleRate": 16_000, "channels": 1, "codec": "pcm_s16le", "container": "wav" }
    })
    .to_string();
    let runtime_version = resolved
        .as_ref()
        .map(|value| value.runtime_version.clone())
        .unwrap_or_else(|| "unconfigured".to_string());
    let model_digest = resolved
        .as_ref()
        .map(|value| value.model_digest.clone())
        .unwrap_or_else(|| "unconfigured".to_string());
    let runtime_config_json = resolved
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(|_| {
            StorageError::InvalidData("The transcription runtime configuration could not be saved.")
        })?;
    Ok((
        "whisper.cpp",
        runtime_version,
        model_digest,
        settings_json,
        runtime_config_json,
    ))
}

pub(crate) fn cached_model_provenance(
    repository: &WorkspaceRepository,
    path: &Path,
) -> StorageResult<runtime::ModelProvenance> {
    // Provenance is authoritative; the SQLite entry is only an acceleration cache.
    let identity_before = runtime::model_file_identity(path).ok();
    let path_key = path.to_string_lossy().into_owned();
    if let Some(identity) = identity_before.as_ref()
        && let Ok(Some((digest, byte_count))) = repository.read_model_provenance_cache(
            &path_key,
            identity.byte_count,
            &identity.modified_at_ns,
        )
    {
        return Ok(runtime::ModelProvenance { digest, byte_count });
    }

    let provenance = runtime::model_provenance(path)?;
    if let Some(identity_before) = identity_before
        && let Ok(identity_after) = runtime::model_file_identity(path)
        && identity_before == identity_after
        && provenance.byte_count == identity_after.byte_count
    {
        let _ = repository.write_model_provenance_cache(
            &path_key,
            provenance.byte_count,
            &identity_after.modified_at_ns,
            &provenance.digest,
        );
    }
    Ok(provenance)
}

/// Resolve a meeting language to a code the transcription runtime understands.
///
/// No language is special-cased. A two-letter ISO 639-1 code passes straight
/// through, common names are recognised as a convenience, and anything else
/// falls back to automatic detection rather than being rejected or assumed.
fn transcription_language_code(language: &str) -> String {
    let value = language.trim().to_ascii_lowercase();
    if value.is_empty() || value == "auto" {
        return "auto".to_string();
    }
    // Any ISO 639-1 code is already valid for the runtime.
    if value.len() == 2 && value.chars().all(|c| c.is_ascii_alphabetic()) {
        return value;
    }
    let named = [
        ("english", "en"),
        ("german", "de"),
        ("deutsch", "de"),
        ("french", "fr"),
        ("français", "fr"),
        ("spanish", "es"),
        ("español", "es"),
        ("italian", "it"),
        ("italiano", "it"),
        ("dutch", "nl"),
        ("nederlands", "nl"),
        ("portuguese", "pt"),
        ("português", "pt"),
        ("polish", "pl"),
        ("polski", "pl"),
        ("czech", "cs"),
        ("danish", "da"),
        ("swedish", "sv"),
        ("norwegian", "no"),
        ("finnish", "fi"),
        ("turkish", "tr"),
        ("japanese", "ja"),
        ("chinese", "zh"),
    ];
    named
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, code)| (*code).to_string())
        .unwrap_or_else(|| "auto".to_string())
}

pub(crate) fn queue_generation(
    root: &Path,
    meeting_id: &str,
    fail_requested: bool,
) -> StorageResult<(ProcessingJobRecord, WorkspaceSnapshot)> {
    let mut repository = WorkspaceRepository::open(root)?;
    ensure_no_active_processing(&repository.connection)?;
    let transcript_revision_id = commit_transcript_working_if_dirty(&mut repository, meeting_id)?;
    let (project_id, recording_id): (String, String) = repository
        .connection
        .query_row(
            "SELECT m.project_id, t.recording_id
             FROM meetings m JOIN transcript_revisions t ON t.id = ?2
             WHERE m.id = ?1",
            params![meeting_id, transcript_revision_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or(StorageError::MissingMeeting)?;
    let protocol_inputs = repository.protocol_inputs(meeting_id)?;
    let (provider_name, runtime_version, model_digest, settings_json, runtime_config_json) =
        generation_metadata(&repository, &protocol_inputs, use_synthetic_adapters())?;
    let job_id = new_id("job");
    let revision_id = new_id("protocol");
    let final_relative_path = meeting_root(&project_id, meeting_id)
        .join("protocols/revisions")
        .join(format!("{revision_id}.md"));
    let now = unix_time_millis();
    let transaction = repository
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_no_active_processing(&transaction)?;
    transaction.execute(
        "INSERT INTO jobs (
            id, meeting_id, recording_id, kind, state, stage, progress_bytes,
            total_bytes, attempt, duplicate_allowed, input_revision_id,
            result_revision_id, provider, runtime_version, model_digest,
            settings_json, runtime_config_json, style_revision, vocabulary_revision,
            final_relative_path, fail_requested, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, 'generation', 'queued', 'generation_queued',
                   0, 100, 1, 0, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
        params![
            job_id,
            meeting_id,
            recording_id,
            transcript_revision_id,
            revision_id,
            provider_name,
            runtime_version,
            model_digest,
            settings_json,
            runtime_config_json,
            protocol_inputs.style.revision,
            protocol_inputs.vocabulary_revision,
            managed_relative_path(&final_relative_path)?,
            i64::from(fail_requested),
            now,
        ],
    )?;
    transaction.commit()?;
    let job = processing_job(&repository, &job_id)?;
    Ok((job, repository.workspace_snapshot()?))
}

fn generation_metadata(
    repository: &WorkspaceRepository,
    inputs: &storage::ResolvedProtocolInputs,
    use_fake: bool,
) -> StorageResult<(&'static str, String, String, String, Option<String>)> {
    let settings_json = serde_json::json!({
        "seed": 42,
        "temperature": 0.2,
        "contextTokens": 8192,
        "maximumOutputTokens": 2048,
        "meetingLanguage": inputs.meeting_language,
    })
    .to_string();
    if use_fake {
        return Ok((
            FAKE_PROVIDER,
            FAKE_RUNTIME_VERSION.to_string(),
            FAKE_MODEL_DIGEST.to_string(),
            settings_json,
            None,
        ));
    }
    let selected_model = repository
        .read_setting("generation.ollamaModel")?
        .filter(|value| !value.is_empty());
    let status = provider::OllamaProvider::loopback().status(selected_model);
    if !status.server_reachable {
        return Err(StorageError::InvalidData(
            "Start your existing Ollama installation before generating a protocol.",
        ));
    }
    let model = status.selected_model.ok_or(StorageError::InvalidData(
        "Choose an installed Ollama model in Settings → Protocol generation.",
    ))?;
    let model_digest = status
        .selected_model_digest
        .ok_or(StorageError::InvalidData(
            "The selected Ollama model is no longer installed. Choose another model.",
        ))?;
    let runtime_version = status
        .runtime_version
        .unwrap_or_else(|| "unknown".to_string());
    // The model's own size, which with the machine's memory decides how much
    // context it can be given without spilling onto the CPU.
    let model_bytes = status
        .models
        .iter()
        .find(|candidate| candidate.name == model)
        .map(|candidate| candidate.size)
        .unwrap_or_default();
    let config = QueuedGenerationConfig {
        model: model.clone(),
        model_digest: model_digest.clone(),
        runtime_version: runtime_version.clone(),
        meeting_language: inputs.meeting_language.clone(),
        style: provider::GenerationStyle {
            id: inputs.style.id.clone(),
            revision: inputs.style.revision.clone(),
            density: inputs.style.density,
            instructions: inputs.style.instructions.clone(),
            required_sections: inputs.style.required_sections.clone(),
        },
        vocabulary_revision: inputs.vocabulary_revision.clone(),
        vocabulary: inputs
            .vocabulary
            .iter()
            .map(|entry| entry.preferred_spelling.clone())
            .collect(),
        seed: 42,
        temperature_milli: 200,
        context_tokens: affordable_context(
            &provider::OllamaProvider::loopback(),
            &model,
            model_bytes,
            machine_memory_bytes(),
        ),
        maximum_output_tokens: output_tokens_for(inputs.style.density),
    };
    let runtime_config_json = serde_json::to_string(&config).map_err(|_| {
        StorageError::InvalidData("The protocol provider configuration could not be saved.")
    })?;
    Ok((
        "ollama",
        runtime_version,
        model_digest,
        settings_json,
        Some(runtime_config_json),
    ))
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
    let mut report = |value: u64, stage: &str| progress(repository, job, value, stage, notify);
    let artifact = if use_synthetic_adapters() {
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
    // Speaker separation is the final expensive phase. Keep validation after it
    // near completion so the progress bar never jumps backwards from 90%.
    progress(repository, job, 96, "validating_transcript", notify)?;
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
    let mut report = |value: u64, stage: &str| progress(repository, job, value, stage, notify);
    let markdown = if use_synthetic_adapters() {
        DeterministicFakeAdapter {
            fail_requested: job.fail_requested,
        }
        .generate(
            GenerationRequest {
                meeting_title: &meeting_title,
                transcript: &transcript,
            },
            cancellation,
            &mut report,
        )?
    } else {
        let config: QueuedGenerationConfig = job
            .runtime_config_json
            .as_deref()
            .ok_or_else(|| ProcessingError::Runtime {
                code: "provider_missing",
                message: "Choose an installed Ollama model in Settings → Protocol generation."
                    .into(),
            })
            .and_then(|value| {
                serde_json::from_str(value).map_err(|_| ProcessingError::Runtime {
                    code: "provider_invalid",
                    message: "The saved protocol provider configuration is invalid.".into(),
                })
            })?;
        let request = provider::GenerationRequest {
            model: config.model,
            model_digest: config.model_digest,
            runtime_version: config.runtime_version,
            meeting_language: config.meeting_language,
            style: config.style,
            vocabulary_revision: config.vocabulary_revision,
            vocabulary: config.vocabulary,
            transcript: transcript
                .segments
                .iter()
                .map(|segment| provider::GenerationSegment {
                    start_ms: segment.start_ms,
                    speaker: segment.speaker.clone(),
                    text: segment.text.clone(),
                })
                .collect(),
            seed: config.seed,
            temperature_milli: config.temperature_milli,
            context_tokens: config.context_tokens,
            maximum_output_tokens: config.maximum_output_tokens,
        };
        // Annotated so the closure is general over the borrow: a stage may now be a
        // string built at the moment it is reported, not only a literal.
        let mut provider_progress = |value: u64, stage: &str| -> provider::Result<()> {
            report(value, stage).map_err(|error| match error {
                ProcessingError::Cancelled => provider::ProviderError::Cancelled,
                other => provider::ProviderError::Unavailable(other.to_string()),
            })
        };
        provider::OllamaProvider::loopback()
            .generate(&request, cancellation, &mut provider_progress)
            .map_err(provider_processing_error)?
    };
    if markdown.trim().is_empty() || markdown.len() > 5_000_000 {
        return Err(ProcessingError::InvalidOutput);
    }
    progress(repository, job, 82, "validating_protocol", notify)?;
    record_quantity_coverage(repository, job, &transcript, &markdown);
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
    let resolved: runtime::ResolvedTranscriptionConfig = job
        .runtime_config_json
        .as_deref()
        .ok_or_else(|| ProcessingError::Runtime {
            code: "runtime_missing",
            message: missing_runtime_message(repository),
        })
        .and_then(|value| {
            serde_json::from_str(value).map_err(|_| ProcessingError::Runtime {
                code: "runtime_missing",
                message: "The saved transcription runtime configuration is invalid.".into(),
            })
        })?;
    let config = runtime::validate_config(&resolved.executable_path, &resolved.model_path)
        .map_err(|message| ProcessingError::Runtime {
            code: "runtime_changed",
            message,
        })?;
    let current_runtime_version =
        runtime::executable_version(&config.executable).ok_or_else(|| {
            ProcessingError::Runtime {
                code: "runtime_changed",
                message:
                    "The configured whisper.cpp executable no longer reports a usable version."
                        .into(),
            }
        })?;
    if current_runtime_version != resolved.runtime_version {
        return Err(ProcessingError::Runtime {
            code: "runtime_changed",
            message: "The configured whisper.cpp executable changed after this job was queued."
                .into(),
        });
    }
    // Rehash at execution so a changed model cannot be hidden by the acceleration cache.
    let current_model =
        runtime::model_provenance(&config.model).map_err(|error| ProcessingError::Runtime {
            code: "model_changed",
            message: error.to_string(),
        })?;
    if current_model.digest != resolved.model_digest
        || current_model.byte_count != resolved.model_byte_count
    {
        return Err(ProcessingError::Runtime {
            code: "model_changed",
            message: "The configured whisper.cpp model changed after this job was queued.".into(),
        });
    }
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
    let settings = serde_json::json!({
        "sampleRate": resolved.sample_rate,
        "channels": resolved.channels,
        "codec": resolved.codec,
        "container": resolved.container,
    })
    .to_string();
    let settings_hash = &checksum_bytes(settings.as_bytes())[..16];
    let normalized_relative = meeting_root(&job.project_id, &job.meeting_id)
        .join("working/normalized")
        .join(format!("{}-{settings_hash}.wav", job.recording_id));
    let normalized = root.join(&normalized_relative);
    let normalized_relative_text = managed_relative_path(&normalized_relative)?;
    let normalizer_version =
        runtime::ffmpeg_version(&ffmpeg).unwrap_or_else(|| "unknown".to_string());
    let audio_stream = probe
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"));
    let source_sample_rate = audio_stream
        .and_then(|stream| stream.sample_rate.as_deref())
        .and_then(|value| value.parse::<u32>().ok());
    let source_channels = audio_stream.and_then(|stream| stream.channels);
    let cached: Option<NormalizedCacheRecord> = repository
        .connection
        .query_row(
            "SELECT source_checksum, normalized_path, normalized_checksum, byte_count,
                    runtime_version, settings_json
             FROM normalized_media WHERE recording_id = ?1",
            [&job.recording_id],
            |row| {
                Ok(NormalizedCacheRecord {
                    source_checksum: row.get(0)?,
                    normalized_path: row.get(1)?,
                    normalized_checksum: row.get(2)?,
                    byte_count: row.get(3)?,
                    runtime_version: row.get(4)?,
                    settings_json: row.get(5)?,
                })
            },
        )
        .optional()?;
    let cache_is_valid = cached.as_ref().map_or(Ok(false), |record| {
        normalized_cache_matches(
            root,
            record,
            source_checksum,
            &normalized_relative_text,
            &normalizer_version,
            &settings,
            cancellation,
        )
    })?;
    if !cache_is_valid {
        report(25, "normalizing_audio")?;
        media::normalize(&ffmpeg, &source, &normalized, cancellation, |value| {
            let _ = report(25 + value / 3, "normalizing_audio");
        })
        .map_err(|message| ProcessingError::Runtime {
            code: "normalization_failed",
            message,
        })?;
        let (normalized_checksum, normalized_byte_count) =
            streamed_checksum(root, &normalized_relative_text, cancellation)?;
        repository.connection.execute(
            "INSERT INTO normalized_media (recording_id, source_checksum, normalized_path, normalized_checksum, byte_count, duration_ms, audio_codec, sample_rate, channels, runtime_version, settings_json, created_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(recording_id) DO UPDATE SET source_checksum=excluded.source_checksum, normalized_path=excluded.normalized_path, normalized_checksum=excluded.normalized_checksum, byte_count=excluded.byte_count, duration_ms=excluded.duration_ms, audio_codec=excluded.audio_codec, sample_rate=excluded.sample_rate, channels=excluded.channels, runtime_version=excluded.runtime_version, settings_json=excluded.settings_json, created_at_ms=excluded.created_at_ms",
            params![job.recording_id, source_checksum, normalized_relative_text, normalized_checksum, normalized_byte_count as i64, probe.format.as_ref().and_then(|format| format.duration.as_deref()).and_then(|value| value.parse::<f64>().ok()).map(|value| (value * 1000.0) as i64), audio_stream.and_then(|stream| stream.codec_name.clone()).or_else(|| probe.format.as_ref().and_then(|format| format.format_name.clone())), source_sample_rate, source_channels, normalizer_version, settings, unix_time_millis()],
        )?;
    }
    report(65, "loading_transcription_model")?;
    let output_base = root
        .join(meeting_root(&job.project_id, &job.meeting_id))
        .join("working/jobs")
        .join(format!("{}-transcript", job.id));
    fs::create_dir_all(output_base.parent().ok_or(ProcessingError::InvalidOutput)?)?;
    // A project's own names are what transcription cannot guess; supplying them
    // measurably corrects company and participant names throughout.
    let vocabulary_terms = repository.transcription_vocabulary(&job.meeting_id)?;
    let vocabulary_prompt = media::vocabulary_prompt(&vocabulary_terms);
    repository.record_transcription_vocabulary(&job.id, vocabulary_prompt.as_deref())?;
    report(70, "transcribing_audio")?;
    let output = runtime::run_process_with_progress(
        media::whisper_command(
            &config,
            &normalized,
            &output_base,
            &resolved.language_code,
            vocabulary_prompt.as_deref(),
        ),
        cancellation,
        runtime::ProcessLimits::with_max_output(2 * 1024 * 1024),
        media::parse_whisper_progress,
        |percent| {
            // Map whisper's 0..=100 onto this stage's 70..=87 band.
            let scaled = 70 + u64::from(percent) * 17 / 100;
            let _ = report(scaled, "transcribing_audio");
        },
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
    let json_path = media::json_output_path(&output_base).ok_or_else(|| {
        let diagnostic = output
            .stderr
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .rev()
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" ");
        let message = if diagnostic.is_empty() {
            "whisper.cpp completed without producing its JSON transcript. Check that the selected executable is whisper-cli and supports JSON output.".to_string()
        } else {
            format!(
                "whisper.cpp completed without producing its JSON transcript. Runtime message: {}",
                diagnostic.chars().take(900).collect::<String>()
            )
        };
        ProcessingError::Runtime {
            code: "invalid_transcript_output",
            message,
        }
    })?;
    let json_bytes = fs::read(&json_path).map_err(|error| ProcessingError::Runtime {
        code: "invalid_transcript_output",
        message: format!(
            "The whisper.cpp JSON transcript could not be read: {}",
            error
        ),
    })?;
    // Some whisper.cpp builds have emitted an invalid byte in an otherwise
    // usable JSON string. Lossy decoding replaces only those bytes and lets the
    // structural JSON parser reject genuinely broken output below.
    let json = String::from_utf8_lossy(&json_bytes);
    let mut artifact = parse_whisper_json(
        &json,
        &job.meeting_id,
        &job.result_revision_id,
        language,
        source_checksum,
    )?;
    let _ = fs::remove_file(json_path);

    // Speakers are a separate capability. When it is unavailable the transcript is
    // still committed with the neutral label rather than the job failing.
    let settings = job
        .settings_json
        .as_deref()
        .and_then(|settings| serde_json::from_str::<serde_json::Value>(settings).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let expected_speakers = settings
        .get("expectedSpeakers")
        .and_then(serde_json::Value::as_u64)
        .and_then(|count| u32::try_from(count).ok());
    // Three answers, not two. A count means separate and this is how many; asking
    // for separation without one means separate and work it out; neither means
    // leave it alone, which is a choice somebody made and not an absence.
    let separate = settings
        .get("separateSpeakers")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(expected_speakers.is_some());
    let timings: Vec<(u64, u64)> = artifact
        .segments
        .iter()
        .map(|segment| (segment.start_ms, segment.end_ms))
        .collect();
    match diarise(
        repository,
        &normalized,
        &timings,
        separate.then_some(expected_speakers),
        cancellation,
        report,
    )? {
        DiarisationOutcome::Resolved(names) => {
            for (segment, name) in artifact.segments.iter_mut().zip(names) {
                segment.speaker = name;
            }
            artifact.speaker_resolution = SpeakerResolution::Resolved;
        }
        DiarisationOutcome::Failed => artifact.speaker_resolution = SpeakerResolution::Failed,
        DiarisationOutcome::Unavailable => {}
    }
    let _ = output.stderr;
    Ok(artifact)
}

/// The optional diariser must never make a usable transcript disappear. Its
/// outcome is retained on the transcript so the review UI can distinguish a
/// real speaker pass from the neutral `Speaker 1` fallback.
enum DiarisationOutcome {
    Unavailable,
    Failed,
    /// One name per transcript segment, in the transcript's own order.
    Resolved(Vec<String>),
}

/// Separate speakers by embedding each segment and grouping the vectors.
///
/// `Ok(None)` means the runtime or its model is not installed and the older
/// diariser should be tried; every other outcome is this pass's answer.
///
/// The count is used when somebody offered one and estimated otherwise. Estimating
/// is possible here and was not before: the diariser answered by re-reading the
/// audio, so a different number cost another eight minutes and had to be settled
/// in advance by somebody who often does not know it. Here the merging happens once
/// and any number is read off it.
fn embed_and_group(
    repository: &WorkspaceRepository,
    normalized: &Path,
    timings: &[(u64, u64)],
    expected_speakers: Option<u32>,
    cancellation: &AtomicBool,
    report: &mut dyn FnMut(u64, &'static str) -> Result<(), ProcessingError>,
) -> Result<Option<DiarisationOutcome>, ProcessingError> {
    let Some(executable) = runtime::discover_executable(runtime::EMBEDDING_NAMES) else {
        return Ok(None);
    };
    // The embedding model is the one the diarisation pass already manages; this
    // pass simply does not need the segmentation model beside it.
    let Some((_, embedding_model)) = models::diarisation_model_paths(&repository.root) else {
        return Ok(None);
    };
    if timings.is_empty() {
        return Ok(Some(DiarisationOutcome::Unavailable));
    }
    report(90, "separating_speakers")?;

    let working = normalized.parent().unwrap_or(&repository.root);
    let segments_path = working.join("speaker-segments.json");
    let vectors_path = working.join("speaker-vectors.bin");
    let segments_json = serde_json::to_string(
        &timings
            .iter()
            .map(|(start, end)| serde_json::json!({"startMs": start, "endMs": end}))
            .collect::<Vec<_>>(),
    )
    .map_err(|error| ProcessingError::Runtime {
        code: "speaker_segments_unwritable",
        message: error.to_string(),
    })?;
    if std::fs::write(&segments_path, segments_json).is_err() {
        return Ok(None);
    }

    let mut command = std::process::Command::new(&executable);
    command
        .arg("--model")
        .arg(&embedding_model)
        .arg("--audio")
        .arg(normalized)
        .arg("--segments")
        .arg(&segments_path)
        .arg("--out")
        .arg(&vectors_path)
        .arg("--threads")
        .arg(media::worker_threads().to_string());
    let outcome = runtime::run_process(
        command,
        cancellation,
        runtime::ProcessLimits::with_max_output(64 * 1024),
    );
    let _ = std::fs::remove_file(&segments_path);

    let result = match outcome {
        Ok(_) => match crate::clustering::read_vectors(&vectors_path) {
            Ok((segments, vectors)) if !vectors.is_empty() => {
                let merged = crate::clustering::merge(&vectors);
                let voices = match expected_speakers.filter(|count| *count >= 2) {
                    Some(count) => merged.voices(count as usize),
                    // Nobody said, so the merging is asked where it stopped
                    // joining a person to themselves. An estimate, offered as one.
                    None => merged.voices(merged.voices_above(crate::clustering::SAME_VOICE_FLOOR)),
                };
                // A segment too short to embed has no vector and keeps the neutral
                // label rather than being given somebody else's.
                let mut named: Vec<Option<usize>> = vec![None; timings.len()];
                for (segment, voice) in segments.iter().zip(voices) {
                    if let Some(slot) = named.get_mut(*segment as usize) {
                        *slot = Some(voice);
                    }
                }
                Some(DiarisationOutcome::Resolved(
                    named
                        .into_iter()
                        .map(|voice| format!("Speaker {}", voice.unwrap_or(0) + 1))
                        .collect(),
                ))
            }
            // The runtime ran and produced nothing usable, which is a failure of
            // this pass rather than a reason to spend half an hour on the other.
            _ => Some(DiarisationOutcome::Failed),
        },
        Err(runtime::ProcessFailure::Cancelled) => return Err(ProcessingError::Cancelled),
        Err(_) => Some(DiarisationOutcome::Failed),
    };
    let _ = std::fs::remove_file(&vectors_path);
    Ok(result)
}

/// `asked` is `None` when nobody wanted speakers separated, and `Some(count)` when
/// they did — where the count inside is `None` if they asked LocaLog to work out
/// how many people spoke.
fn diarise(
    repository: &WorkspaceRepository,
    normalized: &Path,
    timings: &[(u64, u64)],
    asked: Option<Option<u32>>,
    cancellation: &AtomicBool,
    report: &mut dyn FnMut(u64, &'static str) -> Result<(), ProcessingError>,
) -> Result<DiarisationOutcome, ProcessingError> {
    // Nobody asked. The models stay installed after the first use, so without this
    // the pass would run on every later transcription unbidden.
    let Some(expected_speakers) = asked else {
        return Ok(DiarisationOutcome::Unavailable);
    };
    // Speaker separation runs when somebody says how many people were speaking,
    // and not otherwise.
    //
    // Without a count the clustering has only similarity to go on, and a voice
    // drifts across a long recording — different microphones, connection quality,
    // compression — so one person becomes many. Measured on the reference meeting:
    // eight speakers when the count was supplied, eighty-six when it was not, and
    // twelve from a ten-minute excerpt of the same audio.
    //
    // The models stay installed after the first run, so this pass would otherwise
    // keep running on later transcriptions whether or not anybody asked for it,
    // producing a result already known to be unusable. Declining is not a
    // limitation: it is refusing to spend half an hour on an answer that would be
    // wrong.
    // The embedding runtime replaces all of the below where it is available: one
    // vector per segment, grouped by arithmetic, so the number of speakers is an
    // answer rather than a question and costs nothing to change. Measured on the
    // reference meeting at 29 seconds against the diariser's 498.
    if let Some(outcome) = embed_and_group(
        repository,
        normalized,
        timings,
        expected_speakers,
        cancellation,
        report,
    )? {
        return Ok(outcome);
    }

    let Some(expected_speakers) = expected_speakers.filter(|count| *count >= 2) else {
        return Ok(DiarisationOutcome::Unavailable);
    };
    let Some(executable) = repository
        .read_setting("diarisation.executable")?
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| runtime::discover_executable(runtime::DIARISER_NAMES))
    else {
        return Ok(DiarisationOutcome::Unavailable);
    };
    // Both models come from managed storage, like the transcription models. If
    // either is missing the diariser cannot run, and the transcript proceeds
    // without speaker labels rather than failing.
    let Some((segmentation, embedding)) = models::diarisation_model_paths(&repository.root) else {
        return Ok(DiarisationOutcome::Unavailable);
    };
    report(90, "separating_speakers")?;

    // The diariser listens to a couple of seconds of each segment rather than to
    // the whole recording. Transcription has already run, so where the speech is
    // is known, and identifying a voice does not need a whole utterance. On the
    // reference meeting's 675 segments that is 25.6 minutes of audio in place of
    // 81.8.
    //
    // If the condensation cannot be built the pass runs on the full recording
    // instead: slower, and the answer the product had before.
    let samples = diarisation::plan_samples(
        timings,
        diarisation::SAMPLE_MS,
        diarisation::GAP_MS,
        diarisation::SHORTEST_MS,
    );
    let condensed = normalized
        .parent()
        .unwrap_or(&repository.root)
        .join("diarisation-condensed.wav");
    let sampled = !samples.is_empty()
        && media::condense_for_diarisation(normalized, &samples, &condensed).is_ok();

    let listen_to = if sampled { &condensed } else { normalized };
    let output = runtime::run_process(
        media::diarisation_command(&media::DiarisationRequest {
            executable: &executable,
            segmentation_model: &segmentation,
            embedding_model: &embedding,
            normalized: listen_to,
            expected_speakers: Some(expected_speakers),
        }),
        cancellation,
        runtime::ProcessLimits::with_max_output(2 * 1024 * 1024),
    );
    if sampled {
        let _ = fs::remove_file(&condensed);
    }
    match output {
        Ok(output) => {
            let turns = diarisation::parse_turns(&output.stdout);
            if turns.is_empty() {
                return Ok(DiarisationOutcome::Failed);
            }
            let names = if sampled {
                // Each turn is read back through the sample it fell in. The mapping
                // is exact because the condensation is ours.
                let mut found =
                    diarisation::speakers_from_condensed(timings.len(), &samples, &turns);
                diarisation::fill_gaps(&mut found);
                diarisation::name_in_order(found)
            } else {
                diarisation::assign_speakers(timings, &turns)
            };
            Ok(DiarisationOutcome::Resolved(names))
        }
        Err(runtime::ProcessFailure::Cancelled) => Err(ProcessingError::Cancelled),
        // Any other diariser problem leaves the transcript intact and unlabelled.
        Err(_) => Ok(DiarisationOutcome::Failed),
    }
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
        let uncertain = uncertain_words(row);
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
            needs_review: !uncertain.is_empty(),
            uncertain_words: uncertain,
        });
    }
    Ok(TranscriptArtifact {
        schema_version: 1,
        meeting_id: meeting_id.into(),
        revision_id: revision_id.into(),
        language: language.into(),
        speaker_resolution: SpeakerResolution::Unavailable,
        segments,
    })
}

/// How unsure the model must be before the reader is asked about a word.
///
/// Measured against four minutes of the real German meeting: at this value the
/// pass flagged two words in thirty-five segments, one of which was the client's
/// company name misheard. Raising it to 0.5 began flagging ordinary German
/// compounds — the vocabulary of any building text — that were entirely correct.
const UNCERTAIN_BELOW: f64 = 0.40;

/// Word pieces a word must be built from before its doubt is worth reporting.
///
/// A common word is one token, and low confidence there means the model was
/// choosing between two ordinary words: "oder", "hier", "acht" all scored badly
/// and none of them would change a protocol. A rare word — a company, a surname,
/// a technical term — has to be assembled from pieces, and that is where a wrong
/// guess does real damage. Punctuation is not counted, since a doubtful comma is
/// not a question worth putting to anyone.
const UNCERTAIN_MINIMUM_PIECES: usize = 2;

/// One word as whisper spelled it, with the confidence of its least certain piece.
struct TokenizedWord {
    text: String,
    lowest_probability: f64,
    pieces: usize,
}

/// The words in a segment the model was unsure of, in the order they were said.
///
/// whisper reports a probability per token, but a token is a piece of a word: a
/// surname the model guessed at arrives in several fragments. Naming the whole
/// word is what lets the reader be asked a question they can answer, so the
/// fragments are rejoined first — a token that does not begin with a space
/// continues the word before it.
///
/// Special markers such as `[_BEG_]` carry probabilities of their own and are not
/// words, so they are skipped.
fn uncertain_words(row: &serde_json::Value) -> Vec<String> {
    let Some(tokens) = row.get("tokens").and_then(serde_json::Value::as_array) else {
        return Vec::new();
    };
    let mut words: Vec<TokenizedWord> = Vec::new();
    for token in tokens {
        let raw = token
            .get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with("[_") {
            continue;
        }
        let is_punctuation = !trimmed.chars().any(char::is_alphanumeric);
        let probability = token
            .get("p")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0);
        match words.last_mut() {
            Some(word) if !raw.starts_with(char::is_whitespace) => {
                word.text.push_str(raw);
                if !is_punctuation {
                    word.lowest_probability = word.lowest_probability.min(probability);
                    word.pieces += 1;
                }
            }
            _ => words.push(TokenizedWord {
                text: trimmed.to_string(),
                lowest_probability: if is_punctuation { 1.0 } else { probability },
                pieces: usize::from(!is_punctuation),
            }),
        }
    }
    words
        .into_iter()
        .filter(|word| {
            word.pieces >= UNCERTAIN_MINIMUM_PIECES && word.lowest_probability < UNCERTAIN_BELOW
        })
        .map(|word| trim_word_punctuation(&word.text))
        .filter(|word| !word.is_empty())
        .collect()
}

/// Strip the punctuation a word was written with, keeping what belongs to it.
/// Hyphens and apostrophes are part of names and compounds rather than around them.
fn trim_word_punctuation(word: &str) -> String {
    word.trim()
        .trim_matches(|character: char| {
            !character.is_alphanumeric() && character != '-' && character != '\''
        })
        .to_string()
}

/// Locate one of the media tools, preferring what the application ships.
///
/// This used to search PATH alone, which meant a bundled FFmpeg would have been
/// built, signed and never used, while the transcription and speaker runtimes
/// beside it were found correctly. That is the same fault runtime.rs already
/// carries a comment about, made a second time in a different function - which is
/// why both now go through the one resolver.
fn find_tool(name: &str) -> Option<PathBuf> {
    let names = match name {
        "ffmpeg" => runtime::FFMPEG_NAMES,
        "ffprobe" => runtime::FFPROBE_NAMES,
        _ => return None,
    };
    runtime::discover_executable(names)
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
        speaker_resolution: SpeakerResolution::Resolved,
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
                    uncertain_words: Vec::new(),
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

/// Record how much of what the meeting actually stated survived into the protocol.
///
/// A quantity was either said or it was not, so this needs no model and no reader:
/// the transcript is scanned, and each number is either present in the protocol or
/// missing from it. On the reference meeting the first measurement was one of
/// nineteen, which is the kind of thing a length check cannot see at all.
///
/// It is written to the job rather than enforced. A draft that loses a number is
/// still a draft worth having, and the number belongs next to the text where a
/// reader can act on it — which is what the generation redesign is for.
fn record_quantity_coverage(
    repository: &WorkspaceRepository,
    job: &ProcessingJobRecord,
    transcript: &TranscriptArtifact,
    markdown: &str,
) {
    let stated = crate::facts::quantities(&transcript.segments);
    if stated.is_empty() {
        return;
    }
    let accounted = stated
        .iter()
        .filter(|fact| crate::facts::is_accounted_for(fact, markdown))
        .count();
    // How much a protocol keeps depends on the style: a formal record keeps
    // nearly everything and a set of brief notes deliberately keeps little, so
    // coverage is a target a style sets rather than a virtue in itself. Stating a
    // figure the meeting never stated is wrong under every style, which is why
    // both are recorded and only one of them is ever a defect on its own.
    let invented = crate::facts::invented(&transcript.segments, markdown);
    // Length against the transcript, because coverage alone cannot see the failure
    // that matters most. A protocol written subject by subject once scored 23 of 24
    // quantities while being longer than the recording it described: the meeting
    // retyped under headings, which every figure-based measure calls excellent.
    let spoken: usize = transcript
        .segments
        .iter()
        .map(|segment| segment.text.len())
        .sum();
    // Something agreed with nobody against it costs nothing to fix now and an
    // argument at the next meeting otherwise. Read from the shape of the table
    // rather than from its words, so it holds in whatever language the meeting
    // was held in.
    let unowned = crate::facts::unowned_tasks(markdown);
    let coverage = serde_json::json!({
        "quantitiesStated": stated.len(),
        "quantitiesAccounted": accounted,
        "quantitiesInvented": invented,
        "tasksUnowned": unowned,
        "charactersSpoken": spoken,
        "charactersWritten": markdown.len(),
    });
    // Provenance, not a gate: failing to record it must never fail the protocol.
    let _ = repository.connection.execute(
        "UPDATE jobs SET outcome_json = ?2 WHERE id = ?1",
        params![job.id, coverage.to_string()],
    );
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
    let provider_name = job.provider.as_deref().unwrap_or(FAKE_PROVIDER);
    let runtime_version = job
        .runtime_version
        .as_deref()
        .unwrap_or(FAKE_RUNTIME_VERSION);
    let model_digest = job.model_digest.as_deref().unwrap_or(FAKE_MODEL_DIGEST);
    let settings_json = job
        .settings_json
        .as_deref()
        .unwrap_or(r#"{"temperature":0}"#);
    let style_revision = job.style_revision.as_deref().unwrap_or("style@1");
    let vocabulary_revision = job
        .vocabulary_revision
        .as_deref()
        .unwrap_or(VOCABULARY_REVISION);
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
            provider_name,
            runtime_version,
            model_digest,
            settings_json,
            style_id,
            style_revision,
            vocabulary_revision,
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
    let (path, checksum): (String, String) = repository.connection.query_row(
        "SELECT artifact_path, checksum FROM transcript_working WHERE meeting_id = ?1",
        [meeting_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let bytes = read_verified(root, &path, &checksum).map_err(processing_to_storage)?;
    let mut artifact: TranscriptArtifact = serde_json::from_slice(&bytes)
        .map_err(|_| StorageError::InvalidData("The saved transcript is invalid."))?;
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
    let mut repository = WorkspaceRepository::open(root)?;
    let (job_id, kind, prior_settings): (String, String, Option<String>) = repository
        .connection
        .query_row(
            "SELECT id, kind, settings_json FROM jobs WHERE meeting_id = ?1 AND kind IN ('transcription', 'generation')
             AND state IN ('queued', 'failed', 'cancelled', 'interrupted')
             ORDER BY created_at_ms DESC LIMIT 1",
            [meeting_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or(StorageError::MissingJob)?;
    ensure_no_other_active_processing(&repository.connection, &job_id)?;
    let transcription_snapshot = if kind == "transcription" {
        let language: String = repository.connection.query_row(
            "SELECT language FROM meetings WHERE id = ?1",
            [meeting_id],
            |row| row.get(0),
        )?;
        Some({
            let metadata =
                transcription_metadata(&repository, &language, use_synthetic_adapters())?;
            let stored = prior_settings
                .as_deref()
                .and_then(|settings| serde_json::from_str::<serde_json::Value>(settings).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            // A retry repeats what was asked for the first time, including the
            // choice to leave the speakers alone.
            let count = stored
                .get("expectedSpeakers")
                .and_then(serde_json::Value::as_u64)
                .and_then(|count| u32::try_from(count).ok());
            let speakers = match (
                stored
                    .get("separateSpeakers")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(count.is_some()),
                count,
            ) {
                (false, _) => Speakers::Together,
                (true, Some(count)) => Speakers::SeparateInto(count),
                (true, None) => Speakers::Separate,
            };
            (
                metadata.0,
                metadata.1,
                metadata.2,
                transcription_settings_with_speakers(&metadata.3, speakers),
                metadata.4,
            )
        })
    } else {
        None
    };
    let generation_snapshot = if kind == "generation" {
        let inputs = repository.protocol_inputs(meeting_id)?;
        Some((
            generation_metadata(&repository, &inputs, use_synthetic_adapters())?,
            inputs.style.revision,
            inputs.vocabulary_revision,
        ))
    } else {
        None
    };
    let transaction = repository
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_no_other_active_processing(&transaction, &job_id)?;
    transaction.execute(
        "UPDATE jobs SET state = 'queued', stage = CASE kind
                WHEN 'transcription' THEN 'transcription_queued' ELSE 'generation_queued' END,
                progress_bytes = 0,
                attempt = attempt + CASE WHEN state = 'queued' THEN 0 ELSE 1 END,
                error_code = NULL,
                error_message = NULL, fail_requested = 0, updated_at_ms = ?1,
                started_at_ms = NULL, finished_at_ms = NULL WHERE id = ?2",
        params![unix_time_millis(), job_id],
    )?;
    if let Some((provider, runtime_version, model_digest, settings_json, runtime_config_json)) =
        transcription_snapshot
    {
        transaction.execute(
            "UPDATE jobs SET provider = ?1, runtime_version = ?2, model_digest = ?3,
                    settings_json = ?4, runtime_config_json = ?5, updated_at_ms = ?6
             WHERE id = ?7",
            params![
                provider,
                runtime_version,
                model_digest,
                settings_json,
                runtime_config_json,
                unix_time_millis(),
                job_id,
            ],
        )?;
    }
    if let Some((
        (provider, runtime_version, model_digest, settings_json, runtime_config_json),
        style_revision,
        vocabulary_revision,
    )) = generation_snapshot
    {
        transaction.execute(
            "UPDATE jobs SET provider = ?1, runtime_version = ?2, model_digest = ?3,
                    settings_json = ?4, runtime_config_json = ?5,
                    style_revision = ?6, vocabulary_revision = ?7, updated_at_ms = ?8
             WHERE id = ?9",
            params![
                provider,
                runtime_version,
                model_digest,
                settings_json,
                runtime_config_json,
                style_revision,
                vocabulary_revision,
                unix_time_millis(),
                job_id,
            ],
        )?;
    }
    transaction.commit()?;
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

/// Export only the verified working protocol. Exporting never changes the meeting
/// lifecycle or creates an application revision.
pub(crate) fn export_protocol(
    root: &Path,
    meeting_id: &str,
    format: &str,
    destination: &str,
) -> StorageResult<()> {
    if !matches!(format, "markdown" | "text") {
        return Err(StorageError::InvalidData("Choose a valid export format."));
    }
    let repository = WorkspaceRepository::open(root)?;
    let markdown = String::from_utf8(repository.protocol_working_markdown(meeting_id)?)
        .map_err(|_| StorageError::InvalidData("The saved protocol is not valid UTF-8."))?;
    let content = if format == "text" {
        markdown_to_plain_text(&markdown)
    } else {
        markdown
    };
    write_export(destination, content.as_bytes())
}

/// Write an exported file, or refuse for a reason that can be said out loud.
///
/// Shared by the formats built here and the ones built in the interface, so that a
/// Word document is written with the same care as a text file: never over an
/// existing file, and never half-written — the bytes go to a neighbouring temporary
/// and are renamed into place, which is atomic on the same filesystem.
pub(crate) fn write_export(destination: &str, contents: &[u8]) -> StorageResult<()> {
    let destination = PathBuf::from(destination);
    if !destination.is_absolute()
        || destination.to_string_lossy().contains('\0')
        || destination.file_name().is_none()
    {
        return Err(StorageError::InvalidData(
            "Choose a valid export destination.",
        ));
    }
    if destination.exists() {
        return Err(StorageError::InvalidData(
            "Choose a new export filename; existing files are not overwritten automatically.",
        ));
    }
    let parent = destination.parent().ok_or(StorageError::InvalidData(
        "Choose a valid export destination.",
    ))?;
    if !parent.is_dir() {
        return Err(StorageError::InvalidData(
            "The selected export folder is not available.",
        ));
    }
    let temporary = parent.join(format!(
        ".{}.localog-export-{}",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("protocol"),
        new_id("tmp")
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(contents)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, &destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        StorageError::Io(error)
    })?;
    Ok(())
}

pub(crate) fn markdown_to_plain_text(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let without_heading = trimmed.trim_start_matches('#').trim_start();
            let without_bullet = without_heading
                .strip_prefix("- ")
                .or_else(|| without_heading.strip_prefix("* "))
                .or_else(|| without_heading.strip_prefix("+ "))
                .unwrap_or(without_heading);
            without_bullet
                .replace("**", "")
                .replace("__", "")
                .replace('`', "")
        })
        .collect::<Vec<_>>()
        .join("\n")
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
                    j.provider, j.runtime_version, j.model_digest, j.settings_json,
                    j.runtime_config_json, j.style_revision, j.vocabulary_revision
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
                    runtime_config_json: row.get(16)?,
                    style_revision: row.get(17)?,
                    vocabulary_revision: row.get(18)?,
                })
            },
        )
        .optional()?
        .ok_or(StorageError::MissingJob)
}

fn ensure_no_active_processing(connection: &rusqlite::Connection) -> StorageResult<()> {
    let active = connection
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

fn ensure_no_other_active_processing(
    connection: &rusqlite::Connection,
    job_id: &str,
) -> StorageResult<()> {
    let active = connection
        .query_row(
            "SELECT 1 FROM jobs WHERE id != ?1 AND kind IN ('transcription', 'generation')
             AND state IN ('queued', 'running', 'cancelling') LIMIT 1",
            [job_id],
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
    let (checksum, _) = streamed_checksum(root, relative, cancellation)?;
    if checksum != expected {
        return Err(ProcessingError::InvalidOutput);
    }
    Ok(())
}

fn normalized_cache_matches(
    root: &Path,
    record: &NormalizedCacheRecord,
    source_checksum: &str,
    expected_path: &str,
    runtime_version: &str,
    settings_json: &str,
    cancellation: &AtomicBool,
) -> Result<bool, ProcessingError> {
    // A cache row is only a hint; the file itself remains the authority for its bytes.
    if managed_relative_path(Path::new(&record.normalized_path)).is_err() {
        return Ok(false);
    }
    if record.source_checksum != source_checksum
        || record.normalized_path != expected_path
        || record.runtime_version != runtime_version
        || record.settings_json != settings_json
        || record.byte_count <= 0
        || !root.join(&record.normalized_path).is_file()
    {
        return Ok(false);
    }
    let (checksum, byte_count) = streamed_checksum(root, &record.normalized_path, cancellation)?;
    Ok(checksum == record.normalized_checksum && byte_count == record.byte_count as u64)
}

fn streamed_checksum(
    root: &Path,
    relative: &str,
    cancellation: &AtomicBool,
) -> Result<(String, u64), ProcessingError> {
    managed_relative_path(Path::new(relative))?;
    let mut file = File::open(root.join(relative))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 256 * 1024];
    let mut byte_count = 0_u64;
    loop {
        if cancellation.load(Ordering::Acquire) {
            return Err(ProcessingError::Cancelled);
        }
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        byte_count += read as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), byte_count))
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

/// Where a meeting's own recording is written while it happens.
///
/// Beside an imported original rather than somewhere separate: a recording made here
/// and a file somebody sent are the same thing to everything downstream, and the one
/// difference — that this one did not exist five minutes ago — stops mattering the
/// moment it is finished.
pub(crate) fn recording_paths(
    root: &Path,
    project_id: &str,
    meeting_id: &str,
    recording_id: &str,
) -> (PathBuf, PathBuf) {
    let directory = root
        .join(meeting_root(project_id, meeting_id))
        .join("recordings");
    (
        directory.join(format!("{recording_id}-system.wav")),
        directory.join(format!("{recording_id}-microphone.wav")),
    )
}

/// The row a recording in progress is written as.
///
/// Named rather than inline so that a test can put it to a real database. It once
/// used a kind and a state the table forbade, which SQLite reported as a constraint
/// failure, which the person starting a meeting was shown as the workspace being
/// unreachable.
pub(crate) const START_RECORDING_ROW: &str =
    "INSERT INTO recordings (id, meeting_id, kind, original_name, state, created_at_ms)
     VALUES (?1, ?2, 'recorded', ?3, 'recording', ?4)";

/// Start recording a meeting, and record that it is being recorded.
///
/// The row is written first. A recorder running with nothing in the database saying
/// so is the state that leaves a tap open and a machine without sound, and the cost
/// of the reverse — a row with no recorder — is a meeting somebody has to start
/// again.
pub(crate) fn begin_recording(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<(crate::recording::Recording, String)> {
    let repository = WorkspaceRepository::open(root)?;
    let project_id: String = repository.connection.query_row(
        "SELECT project_id FROM meetings WHERE id = ?1",
        [meeting_id],
        |row| row.get(0),
    )?;
    let recording_id = new_id("recording");
    let (system_path, microphone_path) =
        recording_paths(root, &project_id, meeting_id, &recording_id);
    if let Some(directory) = system_path.parent() {
        std::fs::create_dir_all(directory).map_err(StorageError::Io)?;
    }

    repository.connection.execute(
        START_RECORDING_ROW,
        rusqlite::params![
            recording_id,
            meeting_id,
            format!("{recording_id}-system.wav"),
            unix_time_millis()
        ],
    )?;

    let recording = crate::recording::Recording::start(root, system_path, microphone_path)
        .map_err(|_| StorageError::InvalidData("The recorder could not be started."))?;
    Ok((recording, recording_id))
}

/// Finish a recording: combine its two tracks, check the result, and hand the
/// meeting to the rest of the pipeline.
///
/// The state written here is `committed` rather than `ready`, and a checksum is
/// recorded, because that is what transcription looks for — a recording finished any
/// other way is a file nothing will ever read. Getting that wrong is how the first
/// version of this shipped: it recorded perfectly and produced a meeting that could
/// not be transcribed.
pub(crate) fn finish_recording(
    root: &Path,
    recording_id: &str,
    system_path: &Path,
    microphone_path: &Path,
) -> StorageResult<WorkspaceSnapshot> {
    let repository = WorkspaceRepository::open(root)?;
    let ffmpeg = find_tool("ffmpeg").ok_or(StorageError::InvalidData(
        "FFmpeg is needed to finish a recording and could not be found.",
    ))?;

    let combined = system_path.with_file_name(format!("{recording_id}.wav"));
    crate::media::combine_tracks(
        &ffmpeg,
        system_path,
        microphone_path,
        &combined,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .map_err(|_| StorageError::InvalidData("The recording's tracks could not be combined."))?;

    let bytes = std::fs::read(&combined).map_err(StorageError::Io)?;
    let checksum = checksum_bytes(&bytes);
    let relative = combined
        .strip_prefix(root)
        .unwrap_or(&combined)
        .to_string_lossy()
        .to_string();

    repository.connection.execute(
        "UPDATE recordings
            SET state = 'committed', managed_path = ?2, checksum = ?3, byte_count = ?4
          WHERE id = ?1",
        rusqlite::params![recording_id, relative, checksum, bytes.len() as i64],
    )?;
    // The meeting has a source now, which is what every screen keys off.
    repository.connection.execute(
        "UPDATE meetings SET lifecycle = 'source_ready'
          WHERE id = (SELECT meeting_id FROM recordings WHERE id = ?1)",
        [recording_id],
    )?;
    repository.workspace_snapshot()
}

/// Who introduced themselves in a meeting's opening, as the transcript spells them.
///
/// A light path: this needs a model, a language and the first minutes of the
/// transcript, not the style, the vocabulary or any of the rest a protocol run
/// resolves. It is one small request, and it runs when somebody asks for it rather
/// than automatically, because it is model work and this application runs one heavy
/// task at a time.
/// Rewrite one passage of a protocol, as asked.
///
/// The passage travels alone: no transcript, no meeting, no vocabulary. The job is
/// to say the same thing differently, and everything else the model could see is
/// something it could add.
/// A rewritten passage, and what checking it found.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RefinedPassage {
    pub text: String,
    /// Figures that were in the passage and are not in what came back.
    ///
    /// Measured rather than trusted. On a real German passage the shipped small
    /// model altered a fact in three of twenty-four rewrites — "2. Obergeschoss"
    /// became "Obergeschoss (Etage II)" — and a protocol whose figures drift is
    /// worse than one nobody rewrote.
    pub missing_figures: Vec<String>,
    /// What a second pass thought the rewrite changed about the facts.
    ///
    /// Empty when it found nothing, and also empty when the installed model is too
    /// small to be worth asking — `checked` says which.
    pub noticed_changes: Vec<String>,
    /// Whether a model was asked at all.
    pub checked: bool,
}

pub(crate) fn refine_passage(
    root: &Path,
    meeting_id: &str,
    passage: &str,
    instruction: &str,
) -> StorageResult<RefinedPassage> {
    if passage.trim().is_empty() {
        return Err(StorageError::InvalidData("Select some text to change."));
    }
    // A whole protocol is not a passage. The limit is generous — several paragraphs
    // — and exists so that "rewrite" cannot quietly become "regenerate", which is a
    // different operation with a different cost and its own button.
    const LONGEST_PASSAGE: usize = 6_000;
    if passage.len() > LONGEST_PASSAGE {
        return Err(StorageError::InvalidData(
            "That is too much text to change at once. Select a section rather than the document.",
        ));
    }

    let repository = WorkspaceRepository::open(root)?;
    let language: String = repository
        .connection
        .query_row(
            "SELECT language FROM meetings WHERE id = ?1",
            [meeting_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "German".to_string());

    let selected = repository
        .read_setting("generation.ollamaModel")?
        .filter(|value| !value.is_empty());
    let status = provider::OllamaProvider::loopback().status(selected);
    if !status.server_reachable {
        return Err(StorageError::InvalidData(
            "Start your existing Ollama installation before changing a passage.",
        ));
    }
    let model = status.selected_model.ok_or(StorageError::InvalidData(
        "Choose an installed Ollama model in Settings → Protocol generation.",
    ))?;

    let request = provider::GenerationRequest {
        model,
        model_digest: status.selected_model_digest.unwrap_or_default(),
        runtime_version: status.runtime_version.unwrap_or_else(|| "unknown".into()),
        meeting_language: language,
        style: provider::GenerationStyle {
            id: "refine".into(),
            revision: "1".into(),
            density: crate::domain::ProtocolDensity::Concise,
            instructions: Vec::new(),
            required_sections: Vec::new(),
        },
        vocabulary_revision: "refine".into(),
        vocabulary: Vec::new(),
        transcript: Vec::new(),
        seed: 7,
        // Low, but not zero: a rewrite asked for twice should be able to differ.
        temperature_milli: 300,
        // The passage is all that is sent, so the window can be small — which on an
        // eight-gigabyte machine is the difference between an answer and a wait.
        context_tokens: 8_192,
        maximum_output_tokens: 2_048,
    };

    let text = provider::OllamaProvider::loopback()
        .refine_passage(
            &request,
            passage,
            instruction,
            &std::sync::atomic::AtomicBool::new(false),
        )
        // The provider's own message is not passed on: a model's complaint can quote
        // the meeting back at whoever reads the error.
        .map_err(|_| StorageError::InvalidData("That passage could not be rewritten."))?;

    // The instruction tells the model to keep every figure. Whether it did is a
    // separate question, and one that can be answered rather than assumed.
    let mut remaining = crate::facts::numbers_in(&text);
    let mut missing_figures = Vec::new();
    for figure in crate::facts::numbers_in(passage) {
        match remaining.iter().position(|candidate| *candidate == figure) {
            Some(at) => {
                remaining.remove(at);
            }
            None => missing_figures.push(figure),
        }
    }

    // A second opinion, where the installed model is big enough for one to be worth
    // having. Measured: a 4.7B model objects to every rewrite, clean or not, which
    // is the same as objecting to none.
    let capable = provider::OllamaProvider::loopback()
        .installed_models()
        .ok()
        .and_then(|models| {
            models
                .into_iter()
                .find(|model| model.name == request.model)
                .map(|model| provider::can_check_a_rewrite(&model))
        })
        .unwrap_or(false);

    let noticed_changes = if capable {
        provider::OllamaProvider::loopback()
            .check_rewrite(
                &request,
                passage,
                &text,
                &std::sync::atomic::AtomicBool::new(false),
            )
            // A checking pass that fails is a hint that did not arrive, not a
            // rewrite that failed. The rewrite is still returned.
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(RefinedPassage {
        text,
        missing_figures,
        noticed_changes,
        checked: capable,
    })
}

pub(crate) fn find_introductions(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<Vec<provider::Introduction>> {
    let repository = WorkspaceRepository::open(root)?;
    let artifact = working_transcript(root, &repository, meeting_id)?.1;
    let language: String = repository
        .connection
        .query_row(
            "SELECT language FROM meetings WHERE id = ?1",
            [meeting_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "German".to_string());

    let selected = repository
        .read_setting("generation.ollamaModel")?
        .filter(|value| !value.is_empty());
    let status = provider::OllamaProvider::loopback().status(selected);
    if !status.server_reachable {
        return Err(StorageError::InvalidData(
            "Start your existing Ollama installation before reading the introductions.",
        ));
    }
    let model = status.selected_model.ok_or(StorageError::InvalidData(
        "Choose an installed Ollama model in Settings → Protocol generation.",
    ))?;
    let model_digest = status.selected_model_digest.unwrap_or_default();

    let request = provider::GenerationRequest {
        model,
        model_digest,
        runtime_version: status.runtime_version.unwrap_or_else(|| "unknown".into()),
        meeting_language: language,
        style: provider::GenerationStyle {
            id: "introductions".into(),
            revision: "1".into(),
            density: crate::domain::ProtocolDensity::Concise,
            instructions: Vec::new(),
            required_sections: Vec::new(),
        },
        vocabulary_revision: "introductions".into(),
        vocabulary: Vec::new(),
        transcript: artifact
            .segments
            .iter()
            .map(|segment| provider::GenerationSegment {
                start_ms: segment.start_ms,
                speaker: segment.speaker.clone(),
                text: segment.text.clone(),
            })
            .collect(),
        seed: 7,
        temperature_milli: 200,
        context_tokens: 16_384,
        maximum_output_tokens: 2_048,
    };

    provider::OllamaProvider::loopback()
        .find_introductions(
            &request,
            &std::sync::atomic::AtomicBool::new(false),
            &mut |_, _| Ok(()),
        )
        // The provider's own message is not passed on: these are bounded,
        // content-free strings by convention, and a model's complaint can quote the
        // meeting back at whoever reads the error.
        .map_err(|_| StorageError::InvalidData("The meeting's opening could not be read."))
}

/// The words the transcriber was never sure of in a meeting's working transcript.
pub(crate) fn name_candidates(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<Vec<crate::corrections::Candidate>> {
    let repository = WorkspaceRepository::open(root)?;
    let artifact = working_transcript(root, &repository, meeting_id)?.1;
    Ok(crate::corrections::name_candidates(&artifact.segments))
}

/// Every place a correction would apply. Nothing is changed.
pub(crate) fn preview_correction(
    root: &Path,
    meeting_id: &str,
    wrong: &str,
    right: &str,
) -> StorageResult<Vec<crate::corrections::Match>> {
    let repository = WorkspaceRepository::open(root)?;
    let artifact = working_transcript(root, &repository, meeting_id)?.1;
    let correction = crate::corrections::Correction {
        wrong: wrong.to_string(),
        right: right.to_string(),
    };
    Ok(crate::corrections::preview(
        &artifact.segments,
        &[correction],
    ))
}

/// Correct a spelling in the working transcript, and optionally remember it.
///
/// Two outcomes from one action: this transcript is repaired, and the project keeps
/// the spelling so the next meeting is transcribed correctly. The committed revision
/// this was edited from is untouched, which is where somebody goes if the correction
/// was wrong.
pub(crate) fn apply_correction(
    root: &Path,
    meeting_id: &str,
    wrong: &str,
    right: &str,
    kept_segment_ids: &[String],
    remember: bool,
) -> StorageResult<AppliedCorrectionResult> {
    let right = right.trim();
    if wrong.trim().is_empty() || right.is_empty() || right.chars().count() > 200 {
        return Err(StorageError::InvalidData("Enter a valid spelling."));
    }
    let mut repository = WorkspaceRepository::open(root)?;
    let (path, mut artifact) = working_transcript(root, &repository, meeting_id)?;

    let correction = crate::corrections::Correction {
        wrong: wrong.to_string(),
        right: right.to_string(),
    };
    let changed = if kept_segment_ids.is_empty() {
        crate::corrections::apply(&mut artifact.segments, &[correction])
            .into_iter()
            .sum::<usize>()
    } else {
        let kept: Vec<crate::corrections::Match> =
            crate::corrections::preview(&artifact.segments, &[correction])
                .into_iter()
                .filter(|found| kept_segment_ids.contains(&found.segment_id))
                .collect();
        crate::corrections::apply_kept(&mut artifact.segments, &kept)
    };
    persist_transcript_working(&repository, meeting_id, &path, &artifact)?;

    if remember {
        let project_id: String = repository.connection.query_row(
            "SELECT project_id FROM meetings WHERE id = ?1",
            [meeting_id],
            |row| row.get(0),
        )?;
        // A term that is already known is left as it is rather than duplicated.
        let known: bool = repository.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM vocabulary_entries WHERE term = ?1)",
            [right],
            |row| row.get(0),
        )?;
        if !known {
            repository.save_vocabulary_entry(crate::domain::VocabularyDraft {
                id: None,
                term: right.to_string(),
                category: "Person".into(),
                scope: "Project".into(),
                project_id: Some(project_id),
                enabled: true,
            })?;
        }
    }
    Ok(AppliedCorrectionResult {
        workspace: repository.workspace_snapshot()?,
        changed,
    })
}

/// What a correction actually did, as against what was asked of it.
///
/// The count matters because it can differ from the number of places somebody
/// approved: a match whose text has moved since the review is skipped rather than
/// applied to whatever now sits at that position. Telling them "corrected in five
/// places" when it changed none would be a lie the interface had no way to notice.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppliedCorrectionResult {
    pub workspace: WorkspaceSnapshot,
    pub changed: usize,
}

/// The working transcript of a meeting, with the path it is stored at.
fn working_transcript(
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

#[cfg(test)]
mod recording_contract {
    use super::*;

    /// A one-second 16 kHz mono PCM tone, written as a WAV by hand.
    fn write_tone(path: &std::path::Path, hertz: f32, amplitude: f32) {
        const RATE: u32 = 16_000;
        let frames: Vec<i16> = (0..RATE)
            .map(|frame| {
                let phase = frame as f32 / RATE as f32 * hertz * std::f32::consts::TAU;
                (phase.sin() * amplitude * i16::MAX as f32) as i16
            })
            .collect();
        let data: Vec<u8> = frames.iter().flat_map(|s| s.to_le_bytes()).collect();
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&RATE.to_le_bytes());
        wav.extend_from_slice(&(RATE * 2).to_le_bytes()); // byte rate
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav.extend_from_slice(&16u16.to_le_bytes()); // bits
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data.len() as u32).to_le_bytes());
        wav.extend_from_slice(&data);
        std::fs::write(path, wav).unwrap();
    }

    /// A recording carried the whole way: two tracks in, one committed recording out.
    ///
    /// The two pinned strings below are the reason this exists, and they were not
    /// enough. They check that the source says the right words; they cannot check that
    /// the database accepts the row, that FFmpeg accepts the two tracks, or that what
    /// is written can be found again. Every one of those has been broken at least
    /// once. Needs FFmpeg, so it is asked for rather than assumed.
    #[test]
    #[ignore = "requires ffmpeg on this machine"]
    fn two_tracks_become_one_committed_recording() {
        find_tool("ffmpeg").expect("the FFmpeg sidecar must be built to run this");
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();

        let mut repository = crate::storage::WorkspaceRepository::open(root).unwrap();
        let project = repository
            .create_project(crate::domain::NewProjectInput {
                name: "Recorded here".to_string(),
                description: String::new(),
                default_language: "German".to_string(),
            })
            .unwrap();
        let placeholder = root.join("placeholder.wav");
        std::fs::write(&placeholder, b"placeholder").unwrap();
        let meeting = repository
            .create_meeting(crate::domain::NewMeetingInput {
                project_id: project.id.clone(),
                title: "Aufnahme".to_string(),
                occurred_at: "2026-08-18".to_string(),
                language: "German".to_string(),
                source_name: "placeholder.wav".to_string(),
                source_path: Some(placeholder.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();

        // Two real tracks of the kind the recorder writes, written here rather than
        // generated: the shipped FFmpeg has no synthetic sources, on purpose, and a
        // test that reaches for one is testing a build nobody gets.
        let recording_id = "recording-carried";
        let (system_path, microphone_path) =
            recording_paths(root, &project.id, &meeting.id, recording_id);
        std::fs::create_dir_all(system_path.parent().unwrap()).unwrap();
        write_tone(&system_path, 440.0, 0.30);
        write_tone(&microphone_path, 660.0, 0.20);

        repository
            .connection
            .execute(
                START_RECORDING_ROW,
                rusqlite::params![
                    recording_id,
                    meeting.id,
                    format!("{recording_id}-system.wav"),
                    1
                ],
            )
            .expect("the row a recording in progress is written as");
        drop(repository);

        finish_recording(root, recording_id, &system_path, &microphone_path)
            .expect("a recording must finish");

        let repository = crate::storage::WorkspaceRepository::open(root).unwrap();
        let (state, checksum, relative): (String, Option<String>, Option<String>) = repository
            .connection
            .query_row(
                "SELECT state, checksum, managed_path FROM recordings WHERE id = ?1",
                [recording_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(state, "committed", "transcription only reads committed rows");
        let checksum = checksum.expect("transcription verifies a checksum");
        assert_eq!(checksum.len(), 64, "a sha-256 digest, as everywhere else");
        let combined = root.join(relative.expect("a committed recording has a path"));
        assert!(combined.is_file(), "the file the row points at must exist");
        assert!(
            combined.metadata().unwrap().len() > 1024,
            "a two-second mix of two tracks is not a stub"
        );
    }

    /// What transcription reads before it will touch a recording.
    ///
    /// Pinned as text because the two ends are three hundred lines apart and were
    /// written weeks apart: the first version of the recorder wrote `ready` with no
    /// checksum, and produced meetings that recorded perfectly and could never be
    /// transcribed. Nothing in the type system connects a string in one SQL statement
    /// to a string in another.
    #[test]
    fn a_finished_recording_is_in_the_state_transcription_looks_for() {
        let source = include_str!("processing.rs");

        let wanted = source.contains("WHERE r.id = ?1 AND r.state = 'committed'");
        assert!(
            wanted,
            "transcription's own query has changed; update this test"
        );

        let written = source.contains("SET state = 'committed', managed_path = ?2, checksum = ?3");
        assert!(
            written,
            "a finished recording must be left in the state transcription requires, \
             with the checksum it verifies"
        );
    }
}

#[cfg(test)]
mod tests {

    /// The failure this arithmetic exists to prevent, as numbers.
    ///
    /// `mistral-nemo` is 7.1 GB on disk and needs 14 GB at 40,960 tokens. Asked for
    /// that on a 16 GB machine it ran 18 % on the CPU, never returned, and wedged
    /// Ollama in `Stopping...` while still holding the memory. Nothing saw it
    /// coming, because the context was a constant measured against a smaller model.
    #[test]
    fn a_model_is_not_given_more_context_than_the_machine_can_hold() {
        let sixteen_gb = 16 * 1024 * 1024 * 1024u64;
        let eight_gb = 8 * 1024 * 1024 * 1024u64;
        let nemo = 7_100_000_000u64;
        let qwen = 3_400_000_000u64;
        // No Ollama is needed: an unreachable provider reports no context length and
        // the arithmetic still applies to the fallback.
        let provider = provider::OllamaProvider::loopback();

        let for_nemo = affordable_context(&provider, "mistral-nemo:latest", nemo, sixteen_gb);
        let for_qwen = affordable_context(&provider, "qwen3.5:4b", qwen, sixteen_gb);
        assert!(
            for_nemo < for_qwen,
            "the larger model must be given less: {for_nemo} vs {for_qwen}"
        );

        let cramped = affordable_context(&provider, "mistral-nemo:latest", nemo, eight_gb);
        assert!(
            cramped <= for_nemo,
            "less memory must never buy more context"
        );
        // 8,192 was measured to fail at every seed and 4,096 is narrower still, so
        // the floor is the narrowest width that has been seen to produce a protocol.
        assert!(
            cramped >= 16_384,
            "and never fall below a width measured to work: {cramped}"
        );

        // What cannot be established is not guessed at.
        assert_eq!(
            affordable_context(&provider, "qwen3.5:4b", qwen, 0),
            affordable_context(&provider, "qwen3.5:4b", 0, sixteen_gb),
        );
    }

    /// Token shapes taken from real whisper `--output-json-full` output, with the
    /// strings replaced: a rare all-capitals firm name arrives in pieces, and the
    /// model scores the first piece badly. Probabilities are the measured ones.
    /// Runs the real parser over a real whisper `--output-json-full` file and
    /// reports what it would ask the reader about. Ignored by default because it
    /// needs meeting audio, which never lives in this repository.
    ///
    /// `LOCALOG_EVAL_WHISPER_JSON=/path/to/out.json cargo test --lib \
    ///     uncertain_words_against_real_output -- --ignored --nocapture`
    #[test]
    #[ignore = "requires a local whisper JSON transcript"]
    fn uncertain_words_against_real_output() {
        let path = std::env::var("LOCALOG_EVAL_WHISPER_JSON")
            .expect("set LOCALOG_EVAL_WHISPER_JSON to a whisper --output-json-full file");
        let json = std::fs::read_to_string(&path).expect("the transcript could be read");
        let artifact =
            parse_whisper_json(&json, "meeting", "revision", "German", "checksum").unwrap();
        let flagged: Vec<&TranscriptSegment> = artifact
            .segments
            .iter()
            .filter(|segment| segment.needs_review)
            .collect();
        println!(
            "{} of {} segments flagged",
            flagged.len(),
            artifact.segments.len()
        );
        for segment in &flagged {
            println!(
                "  {:>7.1}s  {:?}",
                segment.start_ms as f64 / 1000.0,
                segment.uncertain_words
            );
        }
        assert!(
            !artifact.segments.is_empty(),
            "the transcript should hold segments"
        );
        for segment in &flagged {
            assert!(
                !segment.uncertain_words.is_empty(),
                "a flagged segment must name what to check"
            );
        }
    }

    #[test]
    fn a_misheard_name_is_offered_for_correction() {
        let row = serde_json::json!({
            "text": " Das hat Norwegen bestätigt.",
            "tokens": [
                { "text": "[_BEG_]", "p": 0.31 },
                { "text": " Das", "p": 0.98 },
                { "text": " hat", "p": 0.95 },
                { "text": " Nor", "p": 0.138 },
                { "text": "wegen", "p": 0.91 },
                { "text": " bestätigt", "p": 0.96 },
                { "text": ".", "p": 0.99 }
            ]
        });
        // The whole word, not the "Nor" fragment that scored badly, and the
        // begin marker is ignored despite sitting below the threshold.
        assert_eq!(uncertain_words(&row), vec!["Norwegen".to_string()]);
    }

    /// Every one of these scored below the threshold in the real recording. None
    /// of them is a question worth putting to a reader, and a review pass that
    /// asks about them teaches people to ignore it.
    #[test]
    fn doubt_about_ordinary_words_is_not_reported() {
        let row = serde_json::json!({
            "text": " oder Hier acht wäre. Nee, da.",
            "tokens": [
                { "text": " oder", "p": 0.197 },
                { "text": " Hier", "p": 0.217 },
                { "text": " acht", "p": 0.376 },
                { "text": " wäre", "p": 0.399 },
                { "text": ".", "p": 0.62 },
                { "text": " Nee", "p": 0.224 },
                { "text": ",", "p": 0.55 },
                { "text": " da", "p": 0.357 },
                { "text": ".", "p": 0.30 }
            ]
        });
        // A word plus its punctuation is one piece, so trailing marks cannot
        // promote a common word into a reported one.
        assert!(uncertain_words(&row).is_empty());
    }

    /// German compounds are assembled from many pieces and scored between 0.46
    /// and 0.54 while being transcribed perfectly.
    #[test]
    fn correctly_heard_compounds_are_left_alone() {
        let row = serde_json::json!({
            "text": " Die Zufahrtsstraße liegt im Süden.",
            "tokens": [
                { "text": " Die", "p": 0.99 },
                { "text": " Zufahrts", "p": 0.473 },
                { "text": "stra", "p": 0.88 },
                { "text": "ße", "p": 0.94 },
                { "text": " liegt", "p": 0.97 },
                { "text": " im", "p": 0.98 },
                { "text": " Süden", "p": 0.90 }
            ]
        });
        assert!(uncertain_words(&row).is_empty());
    }

    #[test]
    fn a_transcript_without_tokens_is_not_doubtful() {
        let row = serde_json::json!({ "text": " Something said." });
        assert!(uncertain_words(&row).is_empty());
    }
    use super::*;
    use crate::domain::{JobState, MeetingLifecycle, NewMeetingInput, NewProjectInput};
    use crate::imports;
    use std::sync::{Arc, Barrier};
    use std::thread;
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

    /// Locks the whisper.cpp JSON contract against a real capture from whisper.cpp v1.9.2
    /// (`--output-json` on the public-domain jfk.wav sample). If a future build changes the
    /// transcript shape, this fails instead of silently producing an empty transcript.
    #[test]
    fn parses_real_whisper_cpp_v1_9_2_output_json() {
        // Verbatim `--output-json` output; only the systeminfo string is shortened.
        let json = r#"{
            "systeminfo": "MTL : EMBED_LIBRARY = 1",
            "model": { "type": "base", "multilingual": true },
            "params": { "model": "models/ggml-base.bin", "language": "en", "translate": false },
            "result": { "language": "en" },
            "transcription": [
                {
                    "timestamps": { "from": "00:00:00,000", "to": "00:00:10,500" },
                    "offsets": { "from": 0, "to": 10500 },
                    "text": " And so my fellow Americans ask not what your country can do for you, ask what you can do for your country."
                }
            ]
        }"#;
        let checksum = "abcdef0123456789".repeat(4); // 64-char synthetic checksum
        let artifact = parse_whisper_json(json, "meeting-1", "transcript-1", "English", &checksum)
            .expect("real whisper.cpp v1.9.2 output must parse");
        assert_eq!(artifact.language, "English");
        assert_eq!(artifact.segments.len(), 1);
        let segment = &artifact.segments[0];
        assert_eq!(segment.start_ms, 0);
        assert_eq!(segment.end_ms, 10_500);
        assert_eq!(segment.speaker, "Speaker 1");
        assert!(segment.text.starts_with("And so my fellow Americans"));
        assert!(
            !segment.text.starts_with(' '),
            "leading whitespace must be trimmed"
        );
        assert_eq!(segment.id, "segment-abcdef01-0001");
    }

    /// A queued generation job stores its provenance in named columns, and the
    /// values have to land in the columns they are named after.
    ///
    /// They did not: three parameters were rotated by one, so the provider
    /// configuration was written into `style_revision`'s neighbour and the job
    /// failed at the point of use with "the saved protocol provider configuration
    /// is invalid". Nothing caught it, because a test build substitutes
    /// deterministic adapters and those never read the column. Generation had
    /// therefore never once succeeded through the application's own pipeline.
    #[test]
    fn a_queued_generation_job_stores_each_value_in_its_own_column() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let (generation, _) = queue_generation(&fixture.root, &fixture.meeting_id, false).unwrap();

        let repository = WorkspaceRepository::open(&fixture.root).unwrap();
        let (style_revision, vocabulary_revision): (Option<String>, Option<String>) = repository
            .connection
            .query_row(
                "SELECT style_revision, vocabulary_revision FROM jobs WHERE id = ?1",
                [&generation.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        let inputs = repository.protocol_inputs(&fixture.meeting_id).unwrap();
        assert_eq!(
            style_revision.as_deref(),
            Some(inputs.style.revision.as_str()),
            "style_revision must hold the style's revision"
        );
        assert_eq!(
            vocabulary_revision.as_deref(),
            Some(inputs.vocabulary_revision.as_str()),
            "vocabulary_revision must hold the vocabulary's revision"
        );
    }

    /// Deleting takes the segment out rather than emptying it, so nothing
    /// downstream has to decide what an empty segment means.
    #[test]
    fn a_deleted_segment_leaves_the_transcript() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let before = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        let transcript = &before.transcripts[&fixture.meeting_id];
        let gone = transcript.segments[2].id.clone();
        let count = transcript.segments.len();

        let after = delete_transcript_segment(&fixture.root, &fixture.meeting_id, &gone).unwrap();
        let segments = &after.transcripts[&fixture.meeting_id].segments;
        assert_eq!(segments.len(), count - 1);
        assert!(!segments.iter().any(|segment| segment.id == gone));
        // The rest keep their order and their words.
        assert_eq!(segments[0].id, transcript.segments[0].id);
        assert_eq!(segments[2].id, transcript.segments[3].id);
        // And the working copy is now different from what was committed, which is
        // what puts the transcript back in review.
        assert!(after.transcripts[&fixture.meeting_id].is_dirty);
    }

    /// A transcript of nothing is not a document somebody meant to make, and every
    /// screen after this one would have to learn to read one.
    #[test]
    fn the_last_segment_cannot_be_deleted() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let snapshot = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        let ids: Vec<String> = snapshot.transcripts[&fixture.meeting_id]
            .segments
            .iter()
            .map(|segment| segment.id.clone())
            .collect();
        for id in &ids[..ids.len() - 1] {
            delete_transcript_segment(&fixture.root, &fixture.meeting_id, id).unwrap();
        }
        let refused =
            delete_transcript_segment(&fixture.root, &fixture.meeting_id, &ids[ids.len() - 1]);
        assert!(refused.is_err(), "the last segment must survive");
    }

    /// A segment somebody else already removed, or an id from nowhere.
    #[test]
    fn deleting_a_segment_that_is_not_there_is_refused() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        assert!(
            delete_transcript_segment(&fixture.root, &fixture.meeting_id, "no-such-id").is_err()
        );
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
    fn export_reads_verified_working_protocol_without_mutating_lifecycle() {
        let fixture = Fixture::source_ready();
        fixture.transcribe();
        let (generation, _) = queue_generation(&fixture.root, &fixture.meeting_id, false).unwrap();
        run_job(
            &fixture.root,
            &generation.id,
            Arc::new(AtomicBool::new(false)),
            |_| {},
        )
        .unwrap();
        let before = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        let destination = fixture._temporary.path().join("protocol.md");
        export_protocol(
            &fixture.root,
            &fixture.meeting_id,
            "markdown",
            destination.to_str().unwrap(),
        )
        .unwrap();
        assert!(
            fs::read_to_string(&destination)
                .unwrap()
                .contains("# Synthetic design review")
        );
        let text_destination = fixture._temporary.path().join("protocol.txt");
        export_protocol(
            &fixture.root,
            &fixture.meeting_id,
            "text",
            text_destination.to_str().unwrap(),
        )
        .unwrap();
        assert!(!fs::read_to_string(text_destination).unwrap().contains("# "));
        let after = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(before.meetings[0].lifecycle, after.meetings[0].lifecycle);
        assert_eq!(before.protocols, after.protocols);
    }
    #[test]
    fn concurrent_transcription_starts_admit_only_one_active_job() {
        let fixture = Fixture::source_ready();
        let root = Arc::new(fixture.root.clone());
        let barrier = Arc::new(Barrier::new(2));
        let handles = (0..2)
            .map(|_| {
                let root = Arc::clone(&root);
                let barrier = Arc::clone(&barrier);
                let meeting_id = fixture.meeting_id.clone();
                thread::spawn(move || {
                    barrier.wait();
                    queue_transcription(root.as_path(), &meeting_id, false).map(|_| ())
                })
            })
            .collect::<Vec<_>>();
        let results = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert!(results.iter().any(|result| {
            matches!(result, Err(StorageError::InvalidData(message)) if message.contains("already active"))
        }));
        let snapshot = WorkspaceRepository::open(&fixture.root)
            .unwrap()
            .workspace_snapshot()
            .unwrap();
        assert_eq!(
            snapshot
                .jobs
                .iter()
                .filter(|job| matches!(job.state, JobState::Queued | JobState::Running))
                .count(),
            1
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

    #[test]
    fn lossy_utf8_decoding_keeps_structurally_valid_transcript_json_usable() {
        let bytes = b"{\"transcription\":[{\"offsets\":{\"from\":0,\"to\":1200},\"text\":\"Guten \xFFTag\"}]}";
        let json = String::from_utf8_lossy(bytes);
        let artifact = parse_whisper_json(
            &json,
            "meeting-1",
            "revision-1",
            "German",
            "abcdef0123456789",
        )
        .unwrap();
        assert_eq!(artifact.segments[0].text, "Guten �Tag");
    }

    #[test]
    fn transcription_language_mapping_handles_names_and_codes() {
        assert_eq!(transcription_language_code("English"), "en");
        assert_eq!(transcription_language_code("de"), "de");
        // Previously fell back to detection; named languages are no longer a narrow set.
        assert_eq!(transcription_language_code(" Français "), "fr");
        assert_eq!(transcription_language_code("unknown language"), "auto");
    }

    #[test]
    fn language_resolution_special_cases_nothing() {
        // Named languages are a convenience, not the supported set.
        assert_eq!(transcription_language_code("German"), "de");
        assert_eq!(transcription_language_code("Nederlands"), "nl");
        // Any ISO 639-1 code passes through, including ones with no alias.
        assert_eq!(transcription_language_code("sv"), "sv");
        assert_eq!(transcription_language_code("KO"), "ko");
        // Unknown input detects rather than guessing or failing.
        assert_eq!(transcription_language_code("Schwyzerdütsch"), "auto");
        assert_eq!(transcription_language_code(""), "auto");
    }

    #[test]
    fn real_transcription_metadata_snapshots_runtime_and_model_provenance() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        // The model is resolved from the selected preset, not a user path. Place a
        // size-matching model for "fast" (tiny) in managed storage. set_len makes it
        // sparse, so the registry-sized file exists without writing 77 MB.
        let model_dir = root.join("models");
        fs::create_dir_all(&model_dir).unwrap();
        let model = model_dir.join("ggml-tiny.bin");
        let file = fs::File::create(&model).unwrap();
        file.set_len(77_691_713).unwrap();
        drop(file);
        let repository = WorkspaceRepository::open(root).unwrap();
        repository
            .write_setting("transcription.whisperExecutable", "/bin/echo")
            .unwrap();
        repository
            .write_setting("transcription.preset", "fast")
            .unwrap();

        let (_, runtime_version, model_digest, settings_json, runtime_config_json) =
            transcription_metadata(&repository, "English", false).unwrap();
        let config: runtime::ResolvedTranscriptionConfig =
            serde_json::from_str(&runtime_config_json.unwrap()).unwrap();
        let provenance = runtime::model_provenance(&model).unwrap();

        assert_eq!(config.model_path, model);
        assert_eq!(runtime_version, config.runtime_version);
        assert_eq!(model_digest, provenance.digest);
        assert_eq!(config.model_byte_count, provenance.byte_count);
        assert_eq!(config.language_code, "en");
        assert!(settings_json.contains("\"languageCode\":\"en\""));
    }

    #[test]
    fn model_provenance_cache_reuses_and_invalidates_by_file_identity() {
        let temporary = tempdir().unwrap();
        let model = temporary.path().join("model.ggml");
        fs::write(&model, b"synthetic model bytes").unwrap();
        let repository = WorkspaceRepository::open(temporary.path()).unwrap();

        let first = cached_model_provenance(&repository, &model).unwrap();
        let second = cached_model_provenance(&repository, &model).unwrap();
        assert_eq!(first.digest, second.digest);
        assert_eq!(first.byte_count, second.byte_count);
        let cache_rows: i64 = repository
            .connection
            .query_row("SELECT COUNT(*) FROM model_provenance_cache", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(cache_rows, 1);

        thread::sleep(Duration::from_millis(2));
        fs::write(&model, b"changed model bytes with a new size").unwrap();
        let changed = cached_model_provenance(&repository, &model).unwrap();
        assert_ne!(first.digest, changed.digest);
        assert_ne!(first.byte_count, changed.byte_count);
    }

    #[test]
    fn normalized_cache_requires_matching_bytes_and_metadata() {
        let temporary = tempdir().unwrap();
        let relative = "working/normalized/synthetic.wav";
        let path = temporary.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = b"synthetic normalized audio";
        fs::write(&path, bytes).unwrap();
        let checksum = checksum_bytes(bytes);
        let record = NormalizedCacheRecord {
            source_checksum: "source-sha".to_string(),
            normalized_path: relative.to_string(),
            normalized_checksum: checksum,
            byte_count: bytes.len() as i64,
            runtime_version: "ffmpeg 8".to_string(),
            settings_json: "{\"sampleRate\":16000}".to_string(),
        };
        let cancellation = AtomicBool::new(false);

        assert!(
            normalized_cache_matches(
                temporary.path(),
                &record,
                "source-sha",
                relative,
                "ffmpeg 8",
                "{\"sampleRate\":16000}",
                &cancellation,
            )
            .unwrap()
        );

        fs::write(&path, b"changed normalized audio").unwrap();
        assert!(
            !normalized_cache_matches(
                temporary.path(),
                &record,
                "source-sha",
                relative,
                "ffmpeg 8",
                "{\"sampleRate\":16000}",
                &cancellation,
            )
            .unwrap()
        );
    }
}

#[cfg(test)]
mod refine_against_the_real_model {
    /// The whole path, on this machine's own Ollama.
    ///
    /// Ignored by default because it needs a running runtime and a model, and takes
    /// tens of seconds. Run with the model to use:
    ///     LOCALOG_REFINE_MODEL=ministral-3:8b cargo test --lib refine_against -- --ignored --nocapture
    #[test]
    #[ignore = "requires a running Ollama and an installed model"]
    fn a_passage_is_rewritten_and_checked() {
        let model = std::env::var("LOCALOG_REFINE_MODEL")
            .expect("set LOCALOG_REFINE_MODEL to an installed model");
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();

        let mut repository = crate::storage::WorkspaceRepository::open(root).unwrap();
        repository
            .write_setting("generation.ollamaModel", &model)
            .unwrap();
        let project = repository
            .create_project(crate::domain::NewProjectInput {
                name: "Nordenstadt".to_string(),
                description: String::new(),
                default_language: "German".to_string(),
            })
            .unwrap();
        let placeholder = root.join("placeholder.wav");
        std::fs::write(&placeholder, b"placeholder").unwrap();
        let meeting = repository
            .create_meeting(crate::domain::NewMeetingInput {
                project_id: project.id,
                title: "Jour fixe".to_string(),
                occurred_at: "2026-08-20".to_string(),
                language: "German".to_string(),
                source_name: "placeholder.wav".to_string(),
                source_path: Some(placeholder.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();
        drop(repository);

        let passage = "Die Anpassungen im 2. Obergeschoss im Bereich der Lüftung wurden \
                       aufgrund von Änderungen seitens der Architekten vorgenommen. Die \
                       betroffene Fläche beträgt 148,5 m². Herr Planung hat zugesagt, die \
                       Kostenspanne bis zum 12. September 2026 zu nennen.";
        let refined = super::refine_passage(root, &meeting.id, passage, "Say this in fewer words.")
            .expect("the passage is rewritten");

        println!("--- rewrite ---\n{}", refined.text);
        println!("checked: {}", refined.checked);
        println!("missing figures: {:?}", refined.missing_figures);
        println!("noticed changes: {:?}", refined.noticed_changes);

        assert!(!refined.text.trim().is_empty(), "a rewrite comes back");
        assert!(
            refined.text != passage || refined.missing_figures.is_empty(),
            "an unchanged passage cannot have lost a figure"
        );
    }
}
