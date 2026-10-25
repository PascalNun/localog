//! GTK operations must be created and released on the application main thread.
use std::sync::mpsc;
use webkit2gtk::PrintOperationExt;

pub(crate) fn print_to_file(
    view: &webkit2gtk::WebView,
    uri: &str,
    side_margin_mm: f64,
    sender: mpsc::SyncSender<Result<(), String>>,
) {
    let settings = print_settings_for_pdf(uri);
    let operation = webkit2gtk::PrintOperation::new(view);
    operation.set_print_settings(&settings);
    // WebKitGTK takes neither the paper size nor the margins from the sheet's
    // CSS `@page` rule when printing to file — measured: a `@page` asking for
    // `25mm 10mm 22mm` came out at 18pt on every side. The sheet is laid out for
    // A4 (the readers are German) with the top and foot margins below and the
    // side margin the project's measure asks for, so the breaks the editor
    // measured are the breaks the printer makes.
    let page_setup = gtk::PageSetup::new();
    page_setup.set_paper_size(&gtk::PaperSize::new(Some("iso_a4")));
    page_setup.set_top_margin(25.0, gtk::Unit::Mm);
    page_setup.set_bottom_margin(22.0, gtk::Unit::Mm);
    page_setup.set_left_margin(side_margin_mm, gtk::Unit::Mm);
    page_setup.set_right_margin(side_margin_mm, gtk::Unit::Mm);
    operation.set_page_setup(&page_setup);
    let failed = sender.clone();
    operation.connect_failed(move |_, _| {
        let _ = failed.try_send(Err("pdfExportFailed".into()));
    });
    operation.connect_finished(move |_| {
        let _ = sender.try_send(Ok(()));
    });
    operation.print();
}

/// The print settings for a Linux PDF export.
///
/// The sheet is laid out for A4 (the readers are German). Without naming the
/// paper explicitly, the machine's own default wins — letter on many systems —
/// and the running header and page count land on paper narrower than the layout.
fn print_settings_for_pdf(uri: &str) -> gtk::PrintSettings {
    let settings = gtk::PrintSettings::new();
    settings.set_printer("Print to File");
    settings.set("output-file-format", Some("pdf"));
    settings.set("output-uri", Some(uri));
    settings.set_paper_size(&gtk::PaperSize::new(Some("iso_a4")));
    settings
}
