#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod imports;
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
struct ImportState {
    cancellations: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

#[tauri::command]
async fn load_workspace(state: State<'_, StorageState>) -> Result<WorkspaceSnapshot, String> {
    let root = state.root.clone();
    tauri::async_runtime::spawn_blocking(move || imports::reconcile_imports(&root))
        .await
        .map_err(|_| "The local recovery check stopped unexpectedly.".to_string())?
        .map_err(|error| error.user_message())
}

#[tauri::command]
async fn start_import(
    app: AppHandle,
    storage: State<'_, StorageState>,
    imports: State<'_, ImportState>,
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
    imports: State<'_, ImportState>,
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
    imports: State<'_, ImportState>,
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

fn launch_import(
    app: AppHandle,
    root: PathBuf,
    state: ImportState,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Resolving the OS-owned location is cheap; all filesystem and SQLite work stays off-thread.
            let root = app.path().app_data_dir()?;
            app.manage(StorageState { root });
            app.manage(ImportState::default());
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
            update_meeting_title
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
