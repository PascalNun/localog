#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod storage;

use domain::{MeetingSummary, NewMeetingInput, NewProjectInput, ProjectSummary, WorkspaceSnapshot};
use std::path::PathBuf;
use storage::{Result as StorageResult, WorkspaceRepository};
use tauri::{Manager, State};

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

#[tauri::command]
async fn load_workspace(state: State<'_, StorageState>) -> Result<WorkspaceSnapshot, String> {
    with_repository(state.root.clone(), |repository| {
        repository.workspace_snapshot()
    })
    .await
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
        .setup(|app| {
            // Resolving the OS-owned location is cheap; all filesystem and SQLite work stays off-thread.
            let root = app.path().app_data_dir()?;
            app.manage(StorageState { root });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_identity,
            load_workspace,
            create_project,
            create_meeting,
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
