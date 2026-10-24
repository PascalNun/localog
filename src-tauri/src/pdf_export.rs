//! Linux PDF saving through the same WebKit print sheet as the editor.
use crate::pdf_print::print_to_file;
use crate::processing;
use gtk::gio::prelude::FileExt;
use std::sync::mpsc;
use std::time::Duration;

pub(crate) async fn save(window: tauri::WebviewWindow, destination: String) -> Result<(), String> {
    processing::export_destination(&destination).map_err(|error| error.code())?;
    // Keep incomplete renderings private, and remove them on success, failure or timeout.
    let temporary = tempfile::tempdir().map_err(|_| "pdfExportFailed".to_string())?;
    let output = temporary.path().join("protocol.pdf");
    let uri = gtk::gio::File::for_path(&output).uri().to_string();
    let (sender, receiver) = mpsc::sync_channel(1);
    window
        .with_webview(move |webview| print_to_file(&webview.inner(), &uri, sender))
        .map_err(|_| "pdfExportFailed".to_string())?;
    crate::off_thread(move || {
        let _temporary = temporary;
        receiver
            .recv_timeout(Duration::from_secs(120))
            .map_err(|_| "pdfExportFailed".to_string())??;
        let contents = std::fs::read(&output).map_err(|_| "pdfExportFailed".to_string())?;
        if !contents.starts_with(b"%PDF-") {
            return Err("pdfExportFailed".into());
        }
        processing::write_export(&destination, &contents).map_err(|error| error.code())
    })
    .await?
}
