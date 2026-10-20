//! Desktop capture through PulseAudio, including PipeWire's compatible server.
//!
//! ALSA lists hardware and compatibility PCMs, not the server's monitor sources.
//! Resolve the default sink's actual monitor rather than the first device with
//! "monitor" in its name: a desktop may have several unrelated outputs.

use super::{Track, install_stop, report_silence, silence_note, wav};
use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::{Context, FlagSet as ContextFlags, State as ContextState};
use pulse::def::BufferAttr;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::sample::{Format, Spec};
use pulse::stream::{FlagSet, PeekResult, State, Stream};
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const SPEC: Spec = Spec {
    format: Format::F32le,
    channels: 2,
    rate: 48_000,
};

struct Server {
    // Streams are dropped before the context, and the context before its loop.
    context: Context,
    mainloop: Mainloop,
}

impl Server {
    fn connect() -> Result<Self, String> {
        let mainloop = Mainloop::new().ok_or("Could not create the audio main loop")?;
        let mut context =
            Context::new(&mainloop, "LocaLog").ok_or("Could not create the audio connection")?;
        context
            .connect(None, ContextFlags::NOAUTOSPAWN, None)
            .map_err(|error| format!("Could not connect to desktop audio: {error}"))?;
        let mut server = Self { context, mainloop };
        server.wait(|server| match server.context.get_state() {
            ContextState::Ready => Ok(true),
            ContextState::Failed | ContextState::Terminated => {
                Err("Desktop audio is unavailable".into())
            }
            _ => Ok(false),
        })?;
        Ok(server)
    }

    fn poll(&mut self) -> Result<(), String> {
        // One iteration may only dispatch a protocol event, not an audio packet.
        // Drain pending events before sleeping or two streams fall behind the
        // server even though both devices are delivering audio on time.
        for _ in 0..64 {
            match self.mainloop.iterate(false) {
                IterateResult::Success(0) => break,
                IterateResult::Success(_) => {}
                _ => return Err("Desktop audio stopped responding".into()),
            }
        }
        Ok(())
    }

    fn wait(&mut self, ready: impl Fn(&Self) -> Result<bool, String>) -> Result<(), String> {
        let deadline = Instant::now() + CONNECT_TIMEOUT;
        loop {
            self.poll()?;
            if ready(self)? {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err("Desktop audio connection timed out".into());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn devices(&mut self) -> Result<(Option<String>, Option<String>), String> {
        let defaults = Rc::new(RefCell::new(None));
        let result = defaults.clone();
        let operation = self.context.introspect().get_server_info(move |info| {
            *result.borrow_mut() = Some((
                info.default_sink_name.as_ref().map(|name| name.to_string()),
                info.default_source_name
                    .as_ref()
                    .map(|name| name.to_string()),
            ));
        });
        self.wait(|_| Ok(defaults.borrow().is_some()))?;
        drop(operation);
        let (sink, microphone) = defaults.borrow_mut().take().unwrap();
        // Honour the same per-process routing overrides as libpulse's streams.
        // This also permits isolated tests without changing desktop defaults.
        let sink = std::env::var("PULSE_SINK")
            .ok()
            .filter(|s| !s.is_empty())
            .or(sink);
        let microphone = std::env::var("PULSE_SOURCE")
            .ok()
            .filter(|s| !s.is_empty())
            .or(microphone);
        let Some(sink) = sink else {
            return Ok((None, microphone));
        };
        let monitor = Rc::new(RefCell::new(None));
        let result = monitor.clone();
        let operation = self
            .context
            .introspect()
            .get_sink_info_by_name(&sink, move |info| match info {
                ListResult::Item(sink) => {
                    *result.borrow_mut() = Some(
                        sink.monitor_source_name
                            .as_ref()
                            .map(|name| name.to_string()),
                    )
                }
                ListResult::End | ListResult::Error => {
                    if result.borrow().is_none() {
                        *result.borrow_mut() = Some(None);
                    }
                }
            });
        self.wait(|_| Ok(monitor.borrow().is_some()))?;
        drop(operation);
        let monitor = monitor.borrow_mut().take().flatten();
        // If the default input is itself a monitor, recording it twice would
        // label the call as a microphone and double it in the eventual mix.
        let microphone = microphone.filter(|name| Some(name) != monitor.as_ref());
        Ok((monitor, microphone))
    }

    fn stream(&mut self, source: &str, label: &str) -> Result<Stream, String> {
        let mut stream = Stream::new(&mut self.context, label, &SPEC, None)
            .ok_or("Could not create the audio stream")?;
        // A short fragment keeps live levels and polite shutdown current; the
        // server default can buffer several seconds before delivering anything.
        let buffering = BufferAttr {
            maxlength: 8 * 48_000 * 8,
            tlength: u32::MAX,
            prebuf: u32::MAX,
            minreq: u32::MAX,
            fragsize: 960 * 8,
        };
        stream
            .connect_record(
                Some(source),
                Some(&buffering),
                FlagSet::ADJUST_LATENCY | FlagSet::START_CORKED,
            )
            .map_err(|error| format!("Could not connect the {label} stream: {error}"))?;
        self.wait(|_| match stream.get_state() {
            State::Ready => Ok(true),
            State::Failed | State::Terminated => {
                Err(format!("The {label} source could not be opened"))
            }
            _ => Ok(false),
        })?;
        Ok(stream)
    }
}

pub(super) fn check() {
    match Server::connect().and_then(|mut server| server.devices()) {
        Ok((system, microphone)) => println!(
            "{{\"systemAudio\":\"{}\",\"microphone\":\"{}\"{}}}",
            if system.is_some() {
                "granted"
            } else {
                "not-granted"
            },
            if microphone.is_some() {
                "granted"
            } else {
                "denied"
            },
            if system.is_none() && microphone.is_none() {
                ",\"unavailable\":\"recorderFoundNoDevices\""
            } else {
                ""
            },
        ),
        Err(message) => {
            eprintln!("{message}");
            println!(
                "{{\"unavailable\":\"recorderFoundNoDevices\",\"systemAudio\":\"not-granted\",\"microphone\":\"denied\"}}"
            );
        }
    }
}

struct Capture {
    stream: Stream,
    track: Arc<Track>,
}

impl Capture {
    fn open(
        server: &mut Server,
        source: Option<&str>,
        label: &str,
        path: &Path,
    ) -> Result<Option<Self>, String> {
        let Some(source) = source else {
            return Ok(None);
        };
        // Failure of one source must not throw away the other available track.
        let stream = match server.stream(source, label) {
            Ok(stream) => stream,
            Err(message) => {
                eprintln!("{message}");
                return Ok(None);
            }
        };
        let file = wav::Wav::create(path, SPEC.rate, u16::from(SPEC.channels))
            .map_err(|error| format!("Could not create the {label} recording: {error}"))?;
        Ok(Some(Self {
            stream,
            track: Track::new(file),
        }))
    }

    fn read(&mut self) -> Result<(), String> {
        if self.stream.get_state() != State::Ready {
            return Err("An audio source disconnected".into());
        }
        // Bound each poll so stopping cannot be starved by incoming audio.
        for _ in 0..32 {
            match self
                .stream
                .peek()
                .map_err(|error| format!("Could not read audio: {error}"))?
            {
                PeekResult::Empty => break,
                PeekResult::Hole(bytes) => {
                    self.track.accept(&vec![0.0; bytes / 4]);
                }
                PeekResult::Data(bytes) => {
                    let samples: Vec<f32> = bytes
                        .chunks_exact(4)
                        .map(|sample| f32::from_le_bytes(sample.try_into().unwrap()))
                        .collect();
                    self.track.accept(&samples);
                }
            }
            self.stream
                .discard()
                .map_err(|error| format!("Could not advance audio: {error}"))?;
            if self.track.write_failed.load(Ordering::Relaxed) {
                return Err("Could not save the recording".into());
            }
        }
        Ok(())
    }
}

pub(super) fn record(system_path: PathBuf, microphone_path: PathBuf) -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    install_stop(running.clone());
    let mut server = Server::connect()?;
    let (system_source, microphone_source) = server.devices()?;
    let mut system = Capture::open(
        &mut server,
        system_source.as_deref(),
        "system",
        &system_path,
    )?;
    let mut microphone = Capture::open(
        &mut server,
        microphone_source.as_deref(),
        "microphone",
        &microphone_path,
    )?;
    if system.is_none() && microphone.is_none() {
        return Err("No audio source could be opened".into());
    }

    let started = Instant::now();
    // Connect both sources first, then start them together. Otherwise the first
    // source runs while the second is still negotiating its format and buffers.
    let _starts: Vec<_> = [&mut system, &mut microphone]
        .into_iter()
        .flatten()
        .map(|capture| capture.stream.uncork(None))
        .collect();
    let mut seconds = 0;
    let result = (|| {
        while running.load(Ordering::Relaxed) {
            server.poll()?;
            for capture in [&mut system, &mut microphone].into_iter().flatten() {
                capture.read()?;
            }
            let elapsed = started.elapsed().as_secs();
            if elapsed > seconds {
                seconds = elapsed;
                for capture in [&system, &microphone].into_iter().flatten() {
                    capture
                        .track
                        .file
                        .lock()
                        .map_err(|_| "Recording lock failed")?
                        .flush()
                        .map_err(|error| format!("Could not save audio: {error}"))?;
                }
                let system = system.as_ref().map(|capture| &capture.track);
                let microphone = microphone.as_ref().map(|capture| &capture.track);
                println!(
                    "{{\"seconds\":{seconds},\"systemPeak\":{:.4},\"microphonePeak\":{:.4}{}}}",
                    system.map(|track| track.take_peak()).unwrap_or(0.0),
                    microphone.map(|track| track.take_peak()).unwrap_or(0.0),
                    silence_note(system, microphone)
                );
                let _ = std::io::stdout().flush();
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    })();
    for capture in [&system, &microphone].into_iter().flatten() {
        capture
            .track
            .file
            .lock()
            .map_err(|_| "Recording lock failed")?
            .flush()
            .map_err(|error| format!("Could not finalise audio: {error}"))?;
    }
    report_silence(
        system.as_ref().map(|c| &c.track),
        microphone.as_ref().map(|c| &c.track),
    );
    result
}
