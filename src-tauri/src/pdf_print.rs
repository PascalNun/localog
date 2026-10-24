//! GTK operations must be created and released on the application main thread.
use std::sync::mpsc;
use webkit2gtk::PrintOperationExt;

pub(crate) fn print_to_file(
    view: &webkit2gtk::WebView,
    uri: &str,
    sender: mpsc::SyncSender<Result<(), String>>,
) {
    let settings = gtk::PrintSettings::new();
    settings.set_printer("Print to File");
    settings.set("output-file-format", Some("pdf"));
    settings.set("output-uri", Some(uri));
    let operation = webkit2gtk::PrintOperation::new(view);
    operation.set_print_settings(&settings);
    let failed = sender.clone();
    operation.connect_failed(move |_, _| {
        let _ = failed.try_send(Err("pdfExportFailed".into()));
    });
    operation.connect_finished(move |_| {
        let _ = sender.try_send(Ok(()));
    });
    operation.print();
}
