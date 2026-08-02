#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod imports;
mod processing;
mod storage;

use domain::{MeetingSummary, NewMeetingInput, NewProjectInput, ProjectSummary, WorkspaceSnapshot};
use std::collections::HashMap;
use std::path::PathBuf;
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

#[derive(Clone, Default)]
struct JobCoordinatorState {
    cancellations: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

#[tauri::command]
async fn load_workspace(state: State<'_, StorageState>) -> Result<WorkspaceSnapshot, String> {
    let root = state.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
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
) -> Result<(), String> {
    let root = storage.root.clone();
    let (job, snapshot) = with_repository_root(root.clone(), {
        let meeting_id = meeting_id.clone();
        move |root| processing::queue_transcription(root, &meeting_id, fail_requested)
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
            load_workspace,
            create_project,
            create_meeting,
            start_import,
            cancel_import,
            retry_import,
            replace_import_source,
            update_meeting_title,
            start_transcription,
            start_generation,
            cancel_processing,
            retry_processing,
            update_transcript_segment,
            rename_transcript_speaker,
            autosave_protocol,
            create_protocol_revision,
            mark_protocol_reviewed,
            restore_protocol_revision,
            save_workspace_location
        ])
        .run(tauri::generate_context!())
        .expect("failed to run LocaLog");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_keeps_the_local_first_contract_visible() {
        let identity = app_identity();
        assert_eq!(identity.name, "LocaLog");
        assert!(identity.local_only);
    }
}
