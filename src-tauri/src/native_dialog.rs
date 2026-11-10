//! Native file dialogs that open as the app's own modal window.
//!
//! `tauri-plugin-dialog` hands Linux dialogs to `rfd`, whose GTK backend builds
//! `gtk_file_chooser_native_new` with no parent window. A tiling compositor then
//! lays the chooser out as an ordinary window beside the app, and when the app is
//! fullscreen the chooser inherits the full screen and its action bar is clipped
//! off the bottom — the reported Linux file-chooser "non-response".
//!
//! These commands build the same GTK native chooser with the real window as its
//! transient parent, so it floats and is modal. Other platforms keep the dialog
//! plugin and never call these.

#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::WebviewWindow;

#[cfg(target_os = "linux")]
use gtk::prelude::*;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
// Non-Linux commands keep the same IPC shape but use the dialog plugin instead.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub struct DialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogPath {
    pub path: String,
}

#[cfg(target_os = "linux")]
fn apply_filters(dialog: &gtk::FileChooserNative, filters: &[DialogFilter]) {
    for entry in filters {
        let filter = gtk::FileFilter::new();
        if !entry.name.is_empty() {
            filter.set_name(Some(&entry.name));
        }
        for extension in &entry.extensions {
            filter.add_pattern(&format!("*.{extension}"));
        }
        dialog.add_filter(filter);
    }
}

#[cfg(target_os = "linux")]
fn choose(
    window: &WebviewWindow,
    action: gtk::FileChooserAction,
    title: &str,
    filters: &[DialogFilter],
    configure: impl FnOnce(&gtk::FileChooserNative) + Send + 'static,
) -> Result<Option<PathBuf>, String> {
    let (sender, receiver) = std::sync::mpsc::channel::<Result<Option<PathBuf>, String>>();
    let window = window.clone();
    let title = title.to_owned();
    let filters = filters.to_vec();
    let handle = window.clone();
    // A modal dialog runs on the GTK main loop. Scheduling it can fail before it
    // answers, and a panic inside the loop would otherwise leave this command
    // waiting forever; both are caught so it returns an error instead of hanging.
    handle
        .run_on_main_thread(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let parent = window.gtk_window().ok();
                let dialog =
                    gtk::FileChooserNative::new(Some(&title), parent.as_ref(), action, None, None);
                apply_filters(&dialog, &filters);
                configure(&dialog);
                if dialog.run() == gtk::ResponseType::Accept {
                    dialog.filename()
                } else {
                    None
                }
            }));
            let _ = sender.send(match outcome {
                Ok(chosen) => Ok(chosen),
                Err(_) => Err("dialogUnavailable".to_string()),
            });
        })
        .map_err(|_| "dialogUnavailable".to_string())?;
    receiver
        .recv()
        .map_err(|_| "dialogUnavailable".to_string())?
}

#[tauri::command]
// `return` keeps the cfg'd Linux body separate from the non-Linux fallback below.
#[allow(clippy::needless_return)]
pub async fn native_open_path(
    window: WebviewWindow,
    title: String,
    directory: bool,
    multiple: bool,
    filters: Vec<DialogFilter>,
) -> Result<Option<DialogPath>, String> {
    #[cfg(target_os = "linux")]
    {
        let _ = multiple;
        let action = if directory {
            gtk::FileChooserAction::SelectFolder
        } else {
            gtk::FileChooserAction::Open
        };
        return choose(&window, action, &title, &filters, |_| {}).map(|found| {
            found.map(|path| DialogPath {
                path: path.to_string_lossy().into_owned(),
            })
        });
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Unreachable: the frontend routes non-Linux through the dialog plugin.
        let _ = (window, title, directory, multiple, filters);
        Err("dialogUnavailable".to_string())
    }
}

#[tauri::command]
// `return` keeps the cfg'd Linux body separate from the non-Linux fallback below.
#[allow(clippy::needless_return)]
pub async fn native_save_path(
    window: WebviewWindow,
    title: String,
    default_path: Option<String>,
    filters: Vec<DialogFilter>,
) -> Result<Option<DialogPath>, String> {
    #[cfg(target_os = "linux")]
    {
        return choose(
            &window,
            gtk::FileChooserAction::Save,
            &title,
            &filters,
            move |dialog| {
                dialog.set_do_overwrite_confirmation(true);
                if let Some(path) = default_path.as_deref() {
                    let path = Path::new(path);
                    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                        dialog.set_current_name(name);
                    }
                    if let Some(parent) = path.parent() {
                        let _ = dialog.set_current_folder(parent);
                    }
                }
            },
        )
        .map(|found| {
            found.map(|path| DialogPath {
                path: path.to_string_lossy().into_owned(),
            })
        });
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Unreachable: the frontend routes non-Linux through the dialog plugin.
        let _ = (window, title, default_path, filters);
        Err("dialogUnavailable".to_string())
    }
}
