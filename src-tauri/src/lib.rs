#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(test)]
mod e2e_harness;
#[cfg(test)]
mod eval_harness;

mod clustering;
mod corrections;
mod diarisation;
mod domain;
mod edits;
mod facts;
mod imports;
mod media;
mod models;
mod processing;
mod provider;
mod runtime;
mod storage;
/// Dividing a meeting into subjects, kept for evaluation while the decision in
/// docs/PLAN.md milestone 1 is open: whether sectioning is enough, or whether a
/// structured record should become the production intermediate.
///
/// It is not in the generation path. Writing subject by subject was measured and
/// rejected — it produced a document longer than the transcript — and running the
/// pass only to index a protocol would add about seven minutes to a twelve-minute
/// run for a diagnostic. Compiled for evaluation so the evidence stays runnable
/// and the shipped library carries nothing it does not call.
#[cfg(test)]
mod topics;

use domain::{MeetingSummary, NewMeetingInput, NewProjectInput, ProjectSummary, WorkspaceSnapshot};
use rusqlite::OptionalExtension;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use storage::{Result as StorageResult, WorkspaceRepository};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AppIdentity {
    name: &'static str,
    version: &'static str,
    local_only: bool,
}

#[tauri::command]
fn app_identity() -> AppIdentity {
    AppIdentity {
        name: "LocaLog",
        version: env!("CARGO_PKG_VERSION"),
        local_only: true,
    }
}

#[derive(Clone)]
struct StorageState {
    root: PathBuf,
}

/// Admission control for work that is heavy on this machine.
///
/// Transcription, protocol generation and model downloads all compete for memory
/// and disk bandwidth, and running two at once is measurably worse than running
/// them in sequence: a transcription that takes eleven minutes alone took over
/// fifty-six alongside a large download. One heavy task runs at a time.
#[derive(Clone, Default)]
struct JobCoordinatorState {
    cancellations: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    /// What currently occupies the single heavy slot, if anything.
    heavy: Arc<Mutex<Option<HeavyTask>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeavyTask {
    key: String,
    /// Named as the user would recognise it, for the message they are shown.
    description: &'static str,
}

impl JobCoordinatorState {
    /// Take the heavy slot, or explain who holds it. Re-claiming with the same key
    /// succeeds so that a retry of the same work is not treated as a collision.
    fn claim_heavy(&self, key: &str, description: &'static str) -> Result<(), String> {
        let mut held = self
            .heavy
            .lock()
            .map_err(|_| "The local job coordinator is unavailable.".to_string())?;
        match held.as_ref() {
            Some(current) if current.key == key => Ok(()),
            Some(current) => Err(format!(
                "{} is running. Wait for it to finish before starting anything else, so the two do not slow each other down.",
                current.description
            )),
            None => {
                *held = Some(HeavyTask {
                    key: key.to_string(),
                    description,
                });
                Ok(())
            }
        }
    }

    /// Release the slot, but only if it is still ours.
    fn release_heavy(&self, key: &str) {
        if let Ok(mut held) = self.heavy.lock()
            && held.as_ref().is_some_and(|current| current.key == key)
        {
            *held = None;
        }
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptionRuntimeStatus {
    executable_path: Option<String>,
    model_path: Option<String>,
    executable_found: bool,
    model_found: bool,
    runtime_version: Option<String>,
    model_digest: Option<String>,
    model_byte_count: Option<u64>,
}

fn runtime_status(repository: &WorkspaceRepository) -> TranscriptionRuntimeStatus {
    let executable_path = repository
        .read_setting("transcription.whisperExecutable")
        .ok()
        .flatten()
        .filter(|path| Path::new(path).is_file())
        .or_else(|| {
            runtime::discover_executable(runtime::WHISPER_NAMES)
                .map(|path| path.to_string_lossy().into_owned())
        });
    // The model follows the selected quality preset; it is never a user-entered path.
    let model = models::model_path_for_preset(&repository.root, &selected_preset(repository));
    let model_path = model
        .as_deref()
        .map(|path| path.to_string_lossy().into_owned());
    let executable = executable_path.as_deref().map(PathBuf::from);
    let executable_found = executable.as_deref().is_some_and(|path| path.is_file());
    let model_found = model.is_some();
    let runtime_version = executable.as_deref().and_then(runtime::executable_version);
    let provenance = model
        .as_deref()
        .and_then(|path| processing::cached_model_provenance(repository, path).ok());
    TranscriptionRuntimeStatus {
        executable_path,
        model_path,
        executable_found,
        model_found,
        runtime_version,
        model_digest: provenance.as_ref().map(|value| value.digest.clone()),
        model_byte_count: provenance.map(|value| value.byte_count),
    }
}

#[tauri::command]
async fn transcription_runtime_status(
    storage: State<'_, StorageState>,
) -> Result<TranscriptionRuntimeStatus, String> {
    with_repository(storage.root.clone(), move |repository| {
        Ok(runtime_status(repository))
    })
    .await
}

/// Transitional: the whisper.cpp binary will be bundled as a sidecar, after which
/// this disappears entirely. The model is never configured here — it follows the preset.
#[tauri::command]
async fn configure_transcription_runtime(
    storage: State<'_, StorageState>,
    executable_path: String,
) -> Result<TranscriptionRuntimeStatus, String> {
    with_repository(storage.root.clone(), move |repository| {
        let executable = PathBuf::from(&executable_path);
        if !executable.is_absolute() || !executable.is_file() {
            return Err(storage::StorageError::InvalidData(
                "Choose an existing whisper.cpp executable.",
            ));
        }
        repository.write_setting("transcription.whisperExecutable", &executable_path)?;
        Ok(runtime_status(repository))
    })
    .await
}

fn selected_preset(repository: &WorkspaceRepository) -> String {
    repository
        .read_setting("transcription.preset")
        .ok()
        .flatten()
        .filter(|value| models::is_known_preset(value))
        .unwrap_or_else(|| models::DEFAULT_PRESET.to_string())
}

#[tauri::command]
async fn transcription_capability(
    storage: State<'_, StorageState>,
) -> Result<models::TranscriptionCapability, String> {
    let root = storage.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = WorkspaceRepository::open(&root).map_err(|error| error.user_message())?;
        Ok(models::capability(&root, &selected_preset(&repository)))
    })
    .await
    .map_err(|_| "The local storage task stopped unexpectedly.".to_string())?
}

#[tauri::command]
async fn set_transcription_preset(
    storage: State<'_, StorageState>,
    preset: String,
) -> Result<models::TranscriptionCapability, String> {
    let root = storage.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if !models::is_known_preset(&preset) {
            return Err("Choose a known transcription quality.".to_string());
        }
        let repository = WorkspaceRepository::open(&root).map_err(|error| error.user_message())?;
        repository
            .write_setting("transcription.preset", &preset)
            .map_err(|error| error.user_message())?;
        Ok(models::capability(&root, &preset))
    })
    .await
    .map_err(|_| "The local storage task stopped unexpectedly.".to_string())?
}

#[tauri::command]
async fn remove_transcription_model(
    storage: State<'_, StorageState>,
    model_id: String,
) -> Result<models::TranscriptionCapability, String> {
    let root = storage.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        models::remove_model(&root, &model_id).map_err(|error| error.to_string())?;
        let repository = WorkspaceRepository::open(&root).map_err(|error| error.user_message())?;
        Ok(models::capability(&root, &selected_preset(&repository)))
    })
    .await
    .map_err(|_| "The local storage task stopped unexpectedly.".to_string())?
}

/// Download a model in the background: progress and completion arrive as events so
/// the UI never blocks. The download is cancellable and never installs unverified bytes.
#[tauri::command]
async fn download_transcription_model(
    app: AppHandle,
    storage: State<'_, StorageState>,
    coordinator: State<'_, JobCoordinatorState>,
    model_id: String,
) -> Result<(), String> {
    let root = storage.root.clone();
    let key = format!("model:{model_id}");
    // Downloading competes with transcription and generation for the same machine.
    coordinator.claim_heavy(&key, "A meeting is being processed")?;
    let cancellation = Arc::new(AtomicBool::new(false));
    {
        let mut active = coordinator
            .cancellations
            .lock()
            .map_err(|_| "The local processing coordinator is unavailable.".to_string())?;
        if active.contains_key(&key) {
            return Ok(());
        }
        active.insert(key.clone(), cancellation.clone());
    }
    let state = coordinator.inner().clone();
    tauri::async_runtime::spawn(async move {
        let progress_app = app.clone();
        let progress_model = model_id.clone();
        let run_root = root.clone();
        let run_model = model_id.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            models::download_model(&run_root, &run_model, &cancellation, |percent| {
                let _ = progress_app.emit(
                    "model://progress",
                    serde_json::json!({ "modelId": progress_model, "percent": percent }),
                );
            })
        })
        .await;
        if let Ok(mut active) = state.cancellations.lock() {
            active.remove(&key);
        }
        state.release_heavy(&key);
        match result {
            Ok(Ok(())) => {
                if let Ok(repository) = WorkspaceRepository::open(&root) {
                    let _ = app.emit(
                        "model://changed",
                        models::capability(&root, &selected_preset(&repository)),
                    );
                }
            }
            Ok(Err(models::ModelError::Cancelled)) => {
                // Cancelling is deliberate; report the resulting state, not a failure.
                if let Ok(repository) = WorkspaceRepository::open(&root) {
                    let _ = app.emit(
                        "model://changed",
                        models::capability(&root, &selected_preset(&repository)),
                    );
                }
            }
            Ok(Err(error)) => {
                let _ = app.emit(
                    "model://error",
                    serde_json::json!({ "modelId": model_id, "message": error.to_string() }),
                );
            }
            Err(_) => {
                let _ = app.emit(
                    "model://error",
                    serde_json::json!({ "modelId": model_id, "message": "The download stopped unexpectedly." }),
                );
            }
        }
    });
    Ok(())
}

#[tauri::command]
async fn cancel_transcription_download(
    coordinator: State<'_, JobCoordinatorState>,
    model_id: String,
) -> Result<(), String> {
    let key = format!("model:{model_id}");
    if let Ok(active) = coordinator.cancellations.lock()
        && let Some(token) = active.get(&key)
    {
        token.store(true, Ordering::SeqCst);
    }
    Ok(())
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MeetingAudio {
    path: String,
    duration_ms: Option<i64>,
}

/// Locate playable working audio for review. The normalized mono 16 kHz WAV is used
/// because every webview can decode it and its timing matches the transcript exactly.
/// Returns null until working audio exists.
#[tauri::command]
async fn meeting_audio(
    storage: State<'_, StorageState>,
    meeting_id: String,
) -> Result<Option<MeetingAudio>, String> {
    with_repository(storage.root.clone(), move |repository| {
        let found: Option<(String, Option<i64>)> = repository
            .connection
            .query_row(
                "SELECT nm.normalized_path, nm.duration_ms
                 FROM normalized_media nm
                 JOIN recordings r ON r.id = nm.recording_id
                 WHERE r.meeting_id = ?1
                 ORDER BY r.created_at_ms, r.id LIMIT 1",
                [&meeting_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((relative, duration_ms)) = found else {
            return Ok(None);
        };
        let absolute = repository.root.join(&relative);
        if !absolute.is_file() {
            return Ok(None);
        }
        Ok(Some(MeetingAudio {
            path: absolute.to_string_lossy().into_owned(),
            duration_ms,
        }))
    })
    .await
}

/// Everything the review screen needs to show a recording before it is transcribed.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingReview {
    duration_ms: u64,
    /// Peaks from zero to one, for drawing. Coarse on purpose: this crosses to the
    /// interface on every open, and a person cannot see more than a few thousand
    /// bars on a screen.
    waveform: Vec<f32>,
    edits: edits::Edits,
    /// How long what survives the edits runs, so the screen can say what the
    /// meeting will actually be without recomputing it.
    kept_duration_ms: u64,
}

/// The recording, its shape, and what somebody has decided to leave out of it.
#[tauri::command]
async fn recording_review(
    storage: State<'_, StorageState>,
    meeting_id: String,
    buckets: Option<u32>,
) -> Result<Option<RecordingReview>, String> {
    with_repository(storage.root.clone(), move |repository| {
        let found: Option<(String, Option<i64>)> = repository
            .connection
            .query_row(
                "SELECT nm.normalized_path, nm.duration_ms
                 FROM normalized_media nm
                 JOIN recordings r ON r.id = nm.recording_id
                 WHERE r.meeting_id = ?1
                 ORDER BY r.created_at_ms, r.id LIMIT 1",
                [&meeting_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((relative, duration_ms)) = found else {
            return Ok(None);
        };
        let absolute = repository.root.join(&relative);
        if !absolute.is_file() {
            return Ok(None);
        }
        let stored = repository.recording_edits(&meeting_id)?;
        // The probe's duration is the meeting's; the waveform's length is what is
        // really in the file. Where they disagree the file wins, because the edits
        // are applied to the file.
        let duration_ms = duration_ms.unwrap_or_default().max(0) as u64;
        let waveform = media::waveform(&absolute, buckets.unwrap_or(2_000) as usize)
            .map_err(|_| storage::StorageError::InvalidData("That recording could not be read."))?;
        Ok(Some(RecordingReview {
            kept_duration_ms: edits::kept_duration_ms(duration_ms, &stored),
            duration_ms,
            waveform,
            edits: stored,
        }))
    })
    .await
}

/// Record what to leave out of a recording. Nothing is cut here: the edits are a
/// description, applied when the working audio for transcription is built, so any
/// of this is undoable until then and the recording is never modified at all.
#[tauri::command]
async fn set_recording_edits(
    storage: State<'_, StorageState>,
    meeting_id: String,
    edits: edits::Edits,
) -> Result<(), String> {
    with_repository(storage.root.clone(), move |repository| {
        repository.set_recording_edits(&meeting_id, &edits)
    })
    .await
}

/// Whether speaker separation can run, and what it would cost to make it possible.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SpeakerSeparationStatus {
    models_installed: bool,
    runtime_configured: bool,
    runtime_healthy: bool,
    runtime_version: Option<String>,
    runtime_path: Option<String>,
    /// Bytes still to download; zero once the models are present.
    download_bytes: u64,
}

#[tauri::command]
async fn speaker_separation_status(
    storage: State<'_, StorageState>,
) -> Result<SpeakerSeparationStatus, String> {
    with_repository(storage.root.clone(), move |repository| {
        Ok(speaker_status(repository))
    })
    .await
}

fn speaker_status(repository: &WorkspaceRepository) -> SpeakerSeparationStatus {
    let runtime_path = repository
        .read_setting("diarisation.executable")
        .ok()
        .flatten()
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| runtime::discover_executable(runtime::DIARISER_NAMES));
    let runtime_healthy = runtime_path
        .as_deref()
        .is_some_and(runtime::executable_health);
    let runtime_version = runtime_path
        .as_deref()
        .and_then(runtime::executable_version);
    SpeakerSeparationStatus {
        models_installed: models::diarisation_model_paths(&repository.root).is_some(),
        runtime_configured: runtime_path.is_some(),
        runtime_healthy,
        runtime_version,
        runtime_path: runtime_path.map(|path| path.to_string_lossy().into_owned()),
        download_bytes: models::diarisation_download_bytes(&repository.root),
    }
}

/// Transitional, like the transcription runtime: this disappears once the diariser
/// ships as a bundled sidecar.
#[tauri::command]
async fn configure_speaker_runtime(
    storage: State<'_, StorageState>,
    executable_path: String,
) -> Result<SpeakerSeparationStatus, String> {
    with_repository(storage.root.clone(), move |repository| {
        let executable = PathBuf::from(&executable_path);
        if !executable.is_absolute() || !executable.is_file() {
            return Err(storage::StorageError::InvalidData(
                "Choose an existing speaker separation program.",
            ));
        }
        repository.write_setting("diarisation.executable", &executable_path)?;
        Ok(speaker_status(repository))
    })
    .await
}

/// Fetch whichever diariser models are missing, one after another, through the
/// same verified download path the transcription models use.
#[tauri::command]
async fn download_speaker_models(
    app: AppHandle,
    storage: State<'_, StorageState>,
    coordinator: State<'_, JobCoordinatorState>,
) -> Result<(), String> {
    let root = storage.root.clone();
    let key = "model:speaker-separation".to_string();
    coordinator.claim_heavy(&key, "A meeting is being processed")?;
    let cancellation = Arc::new(AtomicBool::new(false));
    {
        let mut active = coordinator
            .cancellations
            .lock()
            .map_err(|_| "The local processing coordinator is unavailable.".to_string())?;
        if active.contains_key(&key) {
            return Ok(());
        }
        active.insert(key.clone(), cancellation.clone());
    }
    let state = coordinator.inner().clone();
    tauri::async_runtime::spawn(async move {
        let progress_app = app.clone();
        let run_root = root.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            for model_id in models::DIARISATION_MODELS {
                let emit_app = progress_app.clone();
                models::download_model(&run_root, model_id, &cancellation, |percent| {
                    let _ = emit_app.emit(
                        "model://progress",
                        serde_json::json!({ "modelId": "speaker-separation", "percent": percent }),
                    );
                })?;
            }
            Ok::<(), models::ModelError>(())
        })
        .await;
        if let Ok(mut active) = state.cancellations.lock() {
            active.remove(&key);
        }
        state.release_heavy(&key);
        match result {
            Ok(Ok(())) | Ok(Err(models::ModelError::Cancelled)) => {
                if let Ok(repository) = WorkspaceRepository::open(&root) {
                    let _ = app.emit("speakers://changed", speaker_status(&repository));
                }
            }
            Ok(Err(error)) => {
                let _ = app.emit(
                    "model://error",
                    serde_json::json!({
                        "modelId": "speaker-separation",
                        "message": error.to_string()
                    }),
                );
            }
            Err(_) => {
                let _ = app.emit(
                    "model://error",
                    serde_json::json!({
                        "modelId": "speaker-separation",
                        "message": "The download stopped unexpectedly."
                    }),
                );
            }
        }
    });
    Ok(())
}

#[tauri::command]
async fn protocol_provider_status(
    storage: State<'_, StorageState>,
) -> Result<provider::OllamaStatus, String> {
    with_repository(storage.root.clone(), move |repository| {
        let selected_model = repository.read_setting("generation.ollamaModel")?;
        Ok(provider::OllamaProvider::loopback().status(selected_model))
    })
    .await
}

#[tauri::command]
async fn configure_protocol_provider(
    storage: State<'_, StorageState>,
    model: Option<String>,
) -> Result<provider::OllamaStatus, String> {
    with_repository(storage.root.clone(), move |repository| {
        let model = model
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if let Some(model_name) = model.as_deref() {
            let status = provider::OllamaProvider::loopback().status(None);
            if !status.server_reachable {
                return Err(storage::StorageError::InvalidData(
                    "Start your existing Ollama installation before selecting a model.",
                ));
            }
            if !status
                .models
                .iter()
                .any(|candidate| candidate.name == model_name)
            {
                return Err(storage::StorageError::InvalidData(
                    "Choose a model that is already installed in Ollama.",
                ));
            }
            repository.write_setting("generation.ollamaModel", model_name)?;
        } else {
            repository.write_setting("generation.ollamaModel", "")?;
        }
        let selected_model = repository
            .read_setting("generation.ollamaModel")?
            .filter(|value| !value.is_empty());
        Ok(provider::OllamaProvider::loopback().status(selected_model))
    })
    .await
}

#[tauri::command]
async fn load_workspace(state: State<'_, StorageState>) -> Result<WorkspaceSnapshot, String> {
    let root = state.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        models::discard_staged_downloads(&root);
        imports::reconcile_imports(&root)?;
        processing::reconcile(&root)
    })
    .await
    .map_err(|_| "The local recovery check stopped unexpectedly.".to_string())?
    .map_err(|error| error.user_message())
}

#[tauri::command]
async fn start_transcription(
    app: AppHandle,
    storage: State<'_, StorageState>,
    coordinator: State<'_, JobCoordinatorState>,
    meeting_id: String,
    fail_requested: bool,
    // Whether separation was asked for at all, kept distinct from the count
    // because asking without knowing how many is now a usable answer.
    separate_speakers: Option<bool>,
    expected_speakers: Option<u32>,
) -> Result<(), String> {
    // An older caller that sends only a count still means "separate into this
    // many", so the flag defaults to whether one was given.
    let speakers = match (
        separate_speakers.unwrap_or(expected_speakers.is_some()),
        expected_speakers,
    ) {
        (false, _) => processing::Speakers::Together,
        (true, Some(count)) => processing::Speakers::SeparateInto(count),
        (true, None) => processing::Speakers::Separate,
    };
    let root = storage.root.clone();
    let (job, snapshot) = with_repository_root(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |root| {
            processing::queue_transcription_with_expected(
                root,
                &meeting_id,
                fail_requested,
                speakers,
            )
        }
    })
    .await?;
    let _ = app.emit("workspace://changed", snapshot);
    launch_processing(app, root, coordinator.inner().clone(), job)
}

#[tauri::command]
async fn start_generation(
    app: AppHandle,
    storage: State<'_, StorageState>,
    coordinator: State<'_, JobCoordinatorState>,
    meeting_id: String,
    fail_requested: bool,
) -> Result<(), String> {
    let root = storage.root.clone();
    let (job, snapshot) = with_repository_root(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |root| processing::queue_generation(root, &meeting_id, fail_requested)
    })
    .await?;
    let _ = app.emit("workspace://changed", snapshot);
    launch_processing(app, root, coordinator.inner().clone(), job)
}

#[tauri::command]
async fn cancel_processing(
    app: AppHandle,
    storage: State<'_, StorageState>,
    coordinator: State<'_, JobCoordinatorState>,
    meeting_id: String,
) -> Result<(), String> {
    let root = storage.root.clone();
    let job_id = with_repository_root(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |root| processing::request_cancellation(root, &meeting_id)
    })
    .await?;
    let cancellation = coordinator
        .cancellations
        .lock()
        .map_err(|_| "The local processing coordinator is unavailable.".to_string())?
        .get(&meeting_id)
        .cloned();
    if let Some(cancellation) = cancellation {
        cancellation.store(true, Ordering::Release);
    } else {
        let snapshot = with_repository_root(root, move |root| {
            processing::cancel_unstarted(root, &job_id)
        })
        .await?;
        let _ = app.emit("workspace://changed", snapshot);
    }
    Ok(())
}

#[tauri::command]
async fn retry_processing(
    app: AppHandle,
    storage: State<'_, StorageState>,
    coordinator: State<'_, JobCoordinatorState>,
    meeting_id: String,
) -> Result<(), String> {
    let root = storage.root.clone();
    let (job, snapshot) = with_repository_root(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |root| processing::retry_job(root, &meeting_id)
    })
    .await?;
    let _ = app.emit("workspace://changed", snapshot);
    launch_processing(app, root, coordinator.inner().clone(), job)
}

fn launch_processing(
    app: AppHandle,
    root: PathBuf,
    state: JobCoordinatorState,
    job: storage::ProcessingJobRecord,
) -> Result<(), String> {
    let meeting_id = job.meeting_id.clone();
    let heavy_key = format!("job:{meeting_id}");
    state.claim_heavy(&heavy_key, "Another meeting is being processed")?;
    let cancellation = Arc::new(AtomicBool::new(false));
    {
        let mut active = state
            .cancellations
            .lock()
            .map_err(|_| "The local processing coordinator is unavailable.".to_string())?;
        if active.contains_key(&meeting_id) {
            return Ok(());
        }
        active.insert(meeting_id.clone(), cancellation.clone());
    }
    tauri::async_runtime::spawn(async move {
        let event_app = app.clone();
        let run_root = root.clone();
        let job_id = job.id.clone();
        let _ = tauri::async_runtime::spawn_blocking(move || {
            processing::run_job(&run_root, &job_id, cancellation, |snapshot| {
                let _ = event_app.emit("workspace://changed", snapshot);
            })
        })
        .await;
        if let Ok(mut active) = state.cancellations.lock() {
            active.remove(&meeting_id);
        }
        state.release_heavy(&heavy_key);
    });
    Ok(())
}

#[tauri::command]
async fn update_transcript_segment(
    state: State<'_, StorageState>,
    meeting_id: String,
    segment_id: String,
    text: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::autosave_transcript_segment(root, &meeting_id, &segment_id, &text)
    })
    .await
}

/// Remove a segment from the working transcript. The committed revision it was
/// edited from is untouched, which is where somebody goes if they delete the wrong
/// line.
#[tauri::command]
async fn delete_transcript_segment(
    state: State<'_, StorageState>,
    meeting_id: String,
    segment_id: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::delete_transcript_segment(root, &meeting_id, &segment_id)
    })
    .await
}

/// The words the transcriber was never sure of, offered as possible names.
#[tauri::command]
async fn find_name_candidates(
    state: State<'_, StorageState>,
    meeting_id: String,
) -> Result<Vec<corrections::Candidate>, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::name_candidates(root, &meeting_id)
    })
    .await
}

/// Every place a correction would apply, so it can be judged before it is made.
#[tauri::command]
async fn preview_correction(
    state: State<'_, StorageState>,
    meeting_id: String,
    wrong: String,
    right: String,
) -> Result<Vec<corrections::Match>, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::preview_correction(root, &meeting_id, &wrong, &right)
    })
    .await
}

/// Correct a spelling in the working transcript, and remember it for the project.
#[tauri::command]
async fn apply_correction(
    state: State<'_, StorageState>,
    meeting_id: String,
    correction: domain::AppliedCorrection,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::apply_correction(
            root,
            &meeting_id,
            &correction.wrong,
            &correction.right,
            &correction.kept_segment_ids,
            correction.remember,
        )
    })
    .await
}

#[tauri::command]
async fn rename_transcript_speaker(
    state: State<'_, StorageState>,
    meeting_id: String,
    speaker: String,
    replacement: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::rename_speaker(root, &meeting_id, &speaker, &replacement)
    })
    .await
}

/// Add a term or change one. The library is the only place vocabulary is edited,
/// so both cases arrive here and are told apart by whether an id was supplied.
#[tauri::command]
async fn save_vocabulary_entry(
    state: State<'_, StorageState>,
    entry: domain::VocabularyDraft,
) -> Result<WorkspaceSnapshot, String> {
    with_repository(state.root.clone(), move |repository| {
        repository.save_vocabulary_entry(entry)?;
        repository.workspace_snapshot()
    })
    .await
}

#[tauri::command]
async fn delete_vocabulary_entry(
    state: State<'_, StorageState>,
    entry_id: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository(state.root.clone(), move |repository| {
        repository.delete_vocabulary_entry(&entry_id)?;
        repository.workspace_snapshot()
    })
    .await
}

#[tauri::command]
async fn autosave_protocol(
    state: State<'_, StorageState>,
    meeting_id: String,
    markdown: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::autosave_protocol(root, &meeting_id, &markdown)
    })
    .await
}

#[tauri::command]
async fn create_protocol_revision(
    state: State<'_, StorageState>,
    meeting_id: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::create_protocol_revision(root, &meeting_id)
    })
    .await
}

#[tauri::command]
async fn mark_protocol_reviewed(
    state: State<'_, StorageState>,
    meeting_id: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::mark_protocol_reviewed(root, &meeting_id)
    })
    .await
}

#[tauri::command]
async fn restore_protocol_revision(
    state: State<'_, StorageState>,
    meeting_id: String,
    revision_id: String,
) -> Result<WorkspaceSnapshot, String> {
    with_repository_root(state.root.clone(), move |root| {
        processing::restore_protocol_revision(root, &meeting_id, &revision_id)
    })
    .await
}

#[tauri::command]
async fn export_protocol(
    storage: State<'_, StorageState>,
    meeting_id: String,
    format: String,
    destination: String,
) -> Result<(), String> {
    with_repository_root(storage.root.clone(), move |root| {
        processing::export_protocol(root, &meeting_id, &format, &destination)
    })
    .await
}

#[tauri::command]
async fn save_workspace_location(
    state: State<'_, StorageState>,
    meeting_id: String,
    route: String,
) -> Result<(), String> {
    with_repository(state.root.clone(), move |repository| {
        repository.save_workspace_location(&meeting_id, &route)
    })
    .await
}

#[tauri::command]
async fn start_import(
    app: AppHandle,
    storage: State<'_, StorageState>,
    imports: State<'_, JobCoordinatorState>,
    meeting_id: String,
) -> Result<(), String> {
    launch_import(
        app,
        storage.root.clone(),
        imports.inner().clone(),
        meeting_id,
    )
}

#[tauri::command]
async fn cancel_import(
    app: AppHandle,
    storage: State<'_, StorageState>,
    imports: State<'_, JobCoordinatorState>,
    meeting_id: String,
) -> Result<(), String> {
    let root = storage.root.clone();
    let cancellation = imports
        .cancellations
        .lock()
        .map_err(|_| "The import coordinator is unavailable.".to_string())?
        .get(&meeting_id)
        .cloned();

    let (job, snapshot) = with_repository(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |repository| {
            let job = repository.request_import_cancellation(&meeting_id)?;
            let snapshot = repository.workspace_snapshot()?;
            Ok((job, snapshot))
        }
    })
    .await?;
    let _ = app.emit("workspace://changed", snapshot);

    if let Some(cancellation) = cancellation {
        cancellation.store(true, Ordering::Release);
    } else {
        let snapshot = tauri::async_runtime::spawn_blocking(move || {
            imports::cancel_unstarted_import(&root, &job.meeting_id)
        })
        .await
        .map_err(|_| "The local cancellation task stopped unexpectedly.".to_string())?
        .map_err(|error| error.user_message())?;
        let _ = app.emit("workspace://changed", snapshot);
    }
    Ok(())
}

#[tauri::command]
async fn retry_import(
    app: AppHandle,
    storage: State<'_, StorageState>,
    imports: State<'_, JobCoordinatorState>,
    meeting_id: String,
    allow_duplicate: bool,
) -> Result<(), String> {
    let root = storage.root.clone();
    let snapshot = with_repository(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |repository| {
            repository.retry_import(&meeting_id, allow_duplicate)?;
            repository.workspace_snapshot()
        }
    })
    .await?;
    let _ = app.emit("workspace://changed", snapshot);
    launch_import(app, root, imports.inner().clone(), meeting_id)
}

#[tauri::command]
async fn replace_import_source(
    app: AppHandle,
    storage: State<'_, StorageState>,
    imports: State<'_, JobCoordinatorState>,
    meeting_id: String,
    source_name: String,
    source_path: String,
) -> Result<(), String> {
    let root = storage.root.clone();
    let snapshot = with_repository(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |repository| {
            repository.replace_import_source(&meeting_id, &source_name, &source_path)?;
            repository.workspace_snapshot()
        }
    })
    .await?;
    let _ = app.emit("workspace://changed", snapshot);
    launch_import(app, root, imports.inner().clone(), meeting_id)
}

fn launch_import(
    app: AppHandle,
    root: PathBuf,
    state: JobCoordinatorState,
    meeting_id: String,
) -> Result<(), String> {
    let cancellation = Arc::new(AtomicBool::new(false));
    {
        let mut active = state
            .cancellations
            .lock()
            .map_err(|_| "The import coordinator is unavailable.".to_string())?;
        if active.contains_key(&meeting_id) {
            return Ok(());
        }
        active.insert(meeting_id.clone(), cancellation.clone());
    }

    tauri::async_runtime::spawn(async move {
        let event_app = app.clone();
        let run_root = root.clone();
        let run_meeting_id = meeting_id.clone();
        let _ = tauri::async_runtime::spawn_blocking(move || {
            imports::run_import(&run_root, &run_meeting_id, cancellation, |snapshot| {
                let _ = event_app.emit("workspace://changed", snapshot);
            })
        })
        .await;
        if let Ok(mut active) = state.cancellations.lock() {
            active.remove(&meeting_id);
        }
    });
    Ok(())
}

#[tauri::command]
async fn create_project(
    state: State<'_, StorageState>,
    input: NewProjectInput,
) -> Result<ProjectSummary, String> {
    with_repository(state.root.clone(), move |repository| {
        repository.create_project(input)
    })
    .await
}

#[tauri::command]
async fn create_meeting(
    state: State<'_, StorageState>,
    input: NewMeetingInput,
) -> Result<MeetingSummary, String> {
    with_repository(state.root.clone(), move |repository| {
        repository.create_meeting(input)
    })
    .await
}

#[tauri::command]
async fn update_meeting_title(
    state: State<'_, StorageState>,
    meeting_id: String,
    title: String,
) -> Result<(), String> {
    with_repository(state.root.clone(), move |repository| {
        repository.update_meeting_title(&meeting_id, &title)
    })
    .await
}

#[tauri::command]
async fn update_meeting_language(
    state: State<'_, StorageState>,
    meeting_id: String,
    language: String,
) -> Result<(), String> {
    with_repository(state.root.clone(), move |repository| {
        repository.update_meeting_language(&meeting_id, &language)
    })
    .await
}

async fn with_repository<T, F>(root: PathBuf, operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut WorkspaceRepository) -> StorageResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let mut repository = WorkspaceRepository::open(root)?;
        operation(&mut repository)
    })
    .await
    .map_err(|_| "The local storage task stopped unexpectedly.".to_string())?
    .map_err(|error| error.user_message())
}

async fn with_repository_root<T, F>(root: PathBuf, operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&std::path::Path) -> StorageResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || operation(&root))
        .await
        .map_err(|_| "The local storage task stopped unexpectedly.".to_string())?
        .map_err(|error| error.user_message())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Resolving the OS-owned location is cheap; all filesystem and SQLite work stays off-thread.
            let root = app.path().app_data_dir()?;
            app.manage(StorageState { root });
            app.manage(JobCoordinatorState::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_identity,
            transcription_runtime_status,
            configure_transcription_runtime,
            transcription_capability,
            set_transcription_preset,
            download_transcription_model,
            cancel_transcription_download,
            remove_transcription_model,
            speaker_separation_status,
            configure_speaker_runtime,
            download_speaker_models,
            meeting_audio,
            recording_review,
            set_recording_edits,
            protocol_provider_status,
            configure_protocol_provider,
            load_workspace,
            create_project,
            create_meeting,
            start_import,
            cancel_import,
            retry_import,
            replace_import_source,
            update_meeting_title,
            update_meeting_language,
            start_transcription,
            start_generation,
            cancel_processing,
            find_name_candidates,
            preview_correction,
            apply_correction,
            retry_processing,
            update_transcript_segment,
            delete_transcript_segment,
            rename_transcript_speaker,
            save_vocabulary_entry,
            delete_vocabulary_entry,
            autosave_protocol,
            create_protocol_revision,
            mark_protocol_reviewed,
            restore_protocol_revision,
            export_protocol,
            save_workspace_location
        ])
        .run(tauri::generate_context!())
        .expect("failed to run LocaLog");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_heavy_task_runs_at_a_time() {
        let state = JobCoordinatorState::default();
        state
            .claim_heavy("job:meeting-1", "Another meeting is being processed")
            .unwrap();

        // A download cannot start while a meeting is being processed.
        let refused = state.claim_heavy("model:medium", "A meeting is being processed");
        let message = refused.expect_err("a second heavy task must be refused");
        assert!(
            message.contains("is running"),
            "unhelpful message: {message}"
        );

        // Re-claiming the same work is not a collision, so a retry still works.
        state
            .claim_heavy("job:meeting-1", "Another meeting is being processed")
            .unwrap();

        // Releasing something we do not hold must not free the slot.
        state.release_heavy("model:medium");
        assert!(state.claim_heavy("model:medium", "x").is_err());

        state.release_heavy("job:meeting-1");
        state
            .claim_heavy("model:medium", "A model is downloading")
            .unwrap();
    }

    #[test]
    fn identity_keeps_the_local_first_contract_visible() {
        let identity = app_identity();
        assert_eq!(identity.name, "LocaLog");
        assert!(identity.local_only);
    }
}
