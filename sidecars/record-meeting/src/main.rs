//! Record a meeting on Windows and Linux.
//!
//! ```text
//! localog-record-meeting --check
//! localog-record-meeting --system <path.wav> --microphone <path.wav>
//! ```
//!
//! The same contract the macOS recorder already meets: two paths in, two files
//! out, one line of JSON a second on standard output, and nothing above it knows
//! how the bytes were captured. macOS keeps its own recorder — Core Audio process
//! taps do something neither of these platforms offers, and it is written and
//! working.
//!
//! ## The failure this is shaped around
//!
//! Audio capture fails by producing silence. This project has already paid for
//! that once: a build of the macOS recorder without its usage description was
//! handed silence rather than an error, and a study spent 235 seconds recording
//! nothing before anybody noticed.
//!
//! Neither of these platforms can be tested by the machine this was written on,
//! which makes that failure the likely one. So this does not report success for a
//! track it never heard: a stream that delivers no samples, or nothing but
//! zeroes, is said out loud in the level line and again when the recording ends.
//! A silent file is a possible outcome — somebody may genuinely have muted their
//! microphone — but a silent file nobody was told about is not.

mod wav;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// What one captured track knows about itself.
struct Track {
    file: Mutex<wav::Wav>,
    /// Loudest sample since the last level line, as 0..=10_000 so it can live in
    /// an atomic. Floats do not, and a lock per callback would be a lock inside
    /// the audio thread.
    peak: AtomicU64,
    /// Whether anything at all has arrived, which is not the same as whether it
    /// was loud.
    heard_anything: AtomicBool,
    /// Whether any sample has been non-zero.
    heard_sound: AtomicBool,
}

impl Track {
    fn new(file: wav::Wav) -> Arc<Self> {
        Arc::new(Self {
            file: Mutex::new(file),
            peak: AtomicU64::new(0),
            heard_anything: AtomicBool::new(false),
            heard_sound: AtomicBool::new(false),
        })
    }

    fn take_peak(&self) -> f32 {
        self.peak.swap(0, Ordering::Relaxed) as f32 / 10_000.0
    }

    /// Called from the audio thread. Does as little as possible.
    fn accept(&self, samples: &[f32]) {
        self.heard_anything.store(true, Ordering::Relaxed);
        let mut loudest = 0.0_f32;
        let mut pcm = Vec::with_capacity(samples.len());
        for sample in samples {
            let value = sample.clamp(-1.0, 1.0);
            if value != 0.0 {
                self.heard_sound.store(true, Ordering::Relaxed);
            }
            let magnitude = value.abs();
            if magnitude > loudest {
                loudest = magnitude;
            }
            pcm.push((value * i16::MAX as f32) as i16);
        }
        let scaled = (loudest * 10_000.0) as u64;
        self.peak.fetch_max(scaled, Ordering::Relaxed);
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write(&pcm);
        }
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.iter().any(|value| value == "--check") {
        check();
        return;
    }
    let system = value_after(&arguments, "--system");
    let microphone = value_after(&arguments, "--microphone");
    let (Some(system), Some(microphone)) = (system, microphone) else {
        eprintln!("usage: localog-record-meeting --system <path.wav> --microphone <path.wav>");
        eprintln!("       localog-record-meeting --check");
        std::process::exit(2);
    };
    if let Err(message) = record(system, microphone) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn value_after(arguments: &[String], flag: &str) -> Option<PathBuf> {
    let at = arguments.iter().position(|value| value == flag)?;
    arguments.get(at + 1).map(PathBuf::from)
}

/// What this machine will allow, asked before a meeting rather than during one.
///
/// Neither platform has macOS's per-application audio permission, so there is
/// nothing to be refused by: the question is whether the devices exist. Answered
/// in the words the application already reads, because inventing a third
/// vocabulary for the same three states would mean teaching the interface a
/// dialect per platform.
fn check() {
    let host = cpal::default_host();
    let microphone = host.default_input_device().is_some();
    let system = system_device(&host).is_some();
    println!(
        "{{\"systemAudio\":\"{}\",\"microphone\":\"{}\"{}}}",
        if system { "granted" } else { "not-granted" },
        if microphone { "granted" } else { "denied" },
        if system || microphone {
            String::new()
        } else {
            // Not a refusal. A machine with no audio devices at all is a machine
            // that cannot be asked, and saying "denied" would blame somebody for
            // a choice they were never offered.
            ",\"unavailable\":\"recorderFoundNoDevices\"".to_string()
        }
    );
}

/// The device that hears what the speakers are playing.
///
/// Two platforms, two arrangements, and the difference is real rather than
/// cosmetic. WASAPI captures a rendering device in loopback mode, so the device
/// to open is an *output* one. PipeWire and PulseAudio publish a separate monitor
/// source per sink, which arrives as an ordinary input device with "monitor" in
/// its name.
fn system_device(host: &cpal::Host) -> Option<cpal::Device> {
    if cfg!(target_os = "windows") {
        return host.default_output_device();
    }
    let mut monitors = host.input_devices().ok()?;
    monitors.find(|device| {
        device
            .name()
            .map(|name| name.to_lowercase().contains("monitor"))
            .unwrap_or(false)
    })
}

fn record(system_path: PathBuf, microphone_path: PathBuf) -> Result<(), String> {
    let host = cpal::default_host();
    let running = Arc::new(AtomicBool::new(true));
    install_stop(running.clone());

    let mut streams = Vec::new();
    let system = open(&host, system_device(&host), &system_path, &mut streams)?;
    let microphone = open(
        &host,
        host.default_input_device(),
        &microphone_path,
        &mut streams,
    )?;

    let mut seconds = 0_u64;
    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        seconds += 1;
        // Every second, so a file killed rather than stopped is still a complete
        // recording of everything before the kill.
        if let Some(track) = &system {
            let _ = track.file.lock().map(|mut file| file.flush());
        }
        if let Some(track) = &microphone {
            let _ = track.file.lock().map(|mut file| file.flush());
        }
        println!(
            "{{\"seconds\":{seconds},\"systemPeak\":{:.4},\"microphonePeak\":{:.4}{}}}",
            system.as_ref().map(|t| t.take_peak()).unwrap_or(0.0),
            microphone.as_ref().map(|t| t.take_peak()).unwrap_or(0.0),
            silence_note(system.as_ref(), microphone.as_ref()),
        );
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }

    drop(streams);
    for track in [system.as_ref(), microphone.as_ref()].into_iter().flatten() {
        let _ = track.file.lock().map(|mut file| file.flush());
    }
    report_silence(system.as_ref(), microphone.as_ref());
    Ok(())
}

/// Named in the level line, so somebody watching a meeting being recorded finds
/// out during it rather than afterwards.
fn silence_note(system: Option<&Arc<Track>>, microphone: Option<&Arc<Track>>) -> String {
    let mut quiet = Vec::new();
    if system.is_none_or(|track| !track.heard_sound.load(Ordering::Relaxed)) {
        quiet.push("\"system\"");
    }
    if microphone.is_none_or(|track| !track.heard_sound.load(Ordering::Relaxed)) {
        quiet.push("\"microphone\"");
    }
    if quiet.is_empty() {
        String::new()
    } else {
        format!(",\"silent\":[{}]", quiet.join(","))
    }
}

/// Said again at the end, on standard error, where a log will keep it.
fn report_silence(system: Option<&Arc<Track>>, microphone: Option<&Arc<Track>>) {
    for (name, track) in [("system", system), ("microphone", microphone)] {
        match track {
            None => eprintln!("No {name} device was available; that track is empty."),
            Some(track) if !track.heard_anything.load(Ordering::Relaxed) => {
                eprintln!("The {name} stream delivered no audio at all.");
            }
            Some(track) if !track.heard_sound.load(Ordering::Relaxed) => {
                eprintln!(
                    "The {name} stream delivered {} bytes and every sample was silence.",
                    track.file.lock().map(|f| f.bytes_written()).unwrap_or(0)
                );
            }
            Some(_) => {}
        }
    }
}

/// Open one device and start it writing.
///
/// A device that cannot be opened is not fatal: a machine with no microphone
/// should still be able to record what is being said in a call, and one with no
/// loopback should still record the room. The track is empty and says so.
fn open(
    _host: &cpal::Host,
    device: Option<cpal::Device>,
    path: &PathBuf,
    streams: &mut Vec<cpal::Stream>,
) -> Result<Option<Arc<Track>>, String> {
    let Some(device) = device else {
        return Ok(None);
    };
    // The device's own rate and channel count, not a rate asked of it. Resampling
    // belongs to the media stage, which already normalises everything to 16 kHz
    // mono, and asking a device for a format it does not have is a way to be
    // handed silence.
    let config = device
        .default_input_config()
        .or_else(|_| device.default_output_config())
        .map_err(|error| format!("No usable format for {path:?}: {error}"))?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let file = wav::Wav::create(path, sample_rate, channels)
        .map_err(|error| format!("Could not create {path:?}: {error}"))?;
    let track = Track::new(file);

    let for_stream = track.clone();
    let stream = device
        .build_input_stream(
            &config.config(),
            move |samples: &[f32], _| for_stream.accept(samples),
            |error| eprintln!("The audio stream reported: {error}"),
            None,
        )
        .map_err(|error| format!("Could not open the stream for {path:?}: {error}"))?;
    stream
        .play()
        .map_err(|error| format!("Could not start the stream for {path:?}: {error}"))?;
    streams.push(stream);
    Ok(Some(track))
}

/// Stop on the signal the application sends, where the platform has one.
#[cfg(unix)]
fn install_stop(running: Arc<AtomicBool>) {
    unsafe {
        STOPPING = Some(running);
        libc_signal(15, on_signal); // SIGTERM
        libc_signal(2, on_signal); // SIGINT
    }
}

#[cfg(unix)]
static mut STOPPING: Option<Arc<AtomicBool>> = None;

#[cfg(unix)]
extern "C" fn on_signal(_: i32) {
    // Only an atomic store, which is one of the few things a signal handler may
    // do. The loop in `record` notices and finalises both files.
    unsafe {
        if let Some(running) = (&raw const STOPPING).as_ref().and_then(|it| it.as_ref()) {
            running.store(false, Ordering::Relaxed);
        }
    }
}

#[cfg(unix)]
unsafe fn libc_signal(number: i32, handler: extern "C" fn(i32)) {
    // Declared rather than depended on, the way the embedding sidecar declares
    // the nine C functions it calls: `signal` is POSIX, its shape has not moved
    // in decades, and a crate to reach it would be larger than what it reaches.
    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }
    unsafe {
        signal(number, handler as usize);
    }
}

/// Windows has no polite stop: the application calls TerminateProcess and nothing
/// runs afterwards. Nothing to install, and nothing that could be — which is why
/// the header is rewritten every second rather than at the end.
#[cfg(not(unix))]
fn install_stop(_running: Arc<AtomicBool>) {}
