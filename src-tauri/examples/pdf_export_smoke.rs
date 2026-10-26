//! Explicit Linux graphical check, on the OS main thread rather than a test worker.
#[cfg(target_os = "linux")]
#[path = "../src/pdf_print.rs"]
mod pdf_print;

#[cfg(target_os = "linux")]
fn main() {
    use gtk::gio::prelude::FileExt;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    use webkit2gtk::WebViewExt;

    gtk::init().unwrap();
    let output = tempfile::tempdir().unwrap();
    let path = output.path().join("synthetic.pdf");
    let uri = gtk::gio::File::for_path(&path).uri().to_string();
    let view = webkit2gtk::WebView::new();
    let loop_ = gtk::glib::MainLoop::new(None, false);
    let (sender, receiver) = mpsc::sync_channel(1);
    view.connect_load_changed(move |view, event| {
        if event == webkit2gtk::LoadEvent::Finished {
            pdf_print::print_to_file(view, &uri, 20.0, sender.clone());
        }
    });
    let outcome = std::rc::Rc::new(std::cell::RefCell::new(None));
    let result = outcome.clone();
    let waiting = loop_.clone();
    let started = Instant::now();
    gtk::glib::timeout_add_local(Duration::from_millis(50), move || {
        let finished = match receiver.try_recv() {
            Ok(result) => Some(result),
            _ if started.elapsed() > Duration::from_secs(30) => Some(Err("timed out".into())),
            _ => None,
        };
        if let Some(finished) = finished {
            *result.borrow_mut() = Some(finished);
            waiting.quit();
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });
    view.load_html("<html><head><style>@page{size:A4}.next{break-before:page}</style></head><body><h1>Synthetic meeting</h1><p>Ben will send a quote by Friday.</p><p class='next'>Flooring remains undecided.</p></body></html>", None);
    loop_.run();
    outcome.borrow_mut().take().unwrap().unwrap();
    let contents = std::fs::read(&path).unwrap();
    assert!(contents.starts_with(b"%PDF-"));
    assert!(contents.len() > 500);
    println!("Saved and verified a synthetic PDF without a print dialog.");
    drop(view);
    while gtk::glib::MainContext::default().pending() {
        gtk::glib::MainContext::default().iteration(false);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("This check requires Linux and a graphical GTK session.");
    std::process::exit(1);
}
