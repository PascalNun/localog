//! Does one embedding per known transcript segment separate speakers?
//!
//! The shipped pass runs a segmentation model to find where speakers change, over
//! audio whose boundaries transcription already established. This study skips it:
//! for each segment it takes a couple of seconds from the middle, computes a single
//! speaker embedding, and prints the vectors. Clustering them is then arithmetic
//! over a few hundred short vectors rather than another pass over the audio, so any
//! number of speakers can be tried at once instead of costing eight minutes each.
//!
//! Nothing here is production code. It exists to find out whether the idea holds
//! before the application is rebuilt around it.
//!
//! Usage:
//!   embed-segments <embedding-model.onnx> <audio.wav> <segments.json> > vectors.json
//!
//! `segments.json` is either a LocaLog transcript document or a bare array of
//! `{"startMs": …, "endMs": …}`. The audio must be 16 kHz mono 16-bit PCM.

use std::env;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::path::Path;
use std::process::ExitCode;

// The sherpa-onnx C API, declared rather than bound by a build script: this is a
// study, and four functions do not warrant a code generator.
#[repr(C)]
struct SpeakerEmbeddingExtractorConfig {
    model: *const c_char,
    num_threads: c_int,
    debug: c_int,
    provider: *const c_char,
}

unsafe extern "C" {
    fn SherpaOnnxCreateSpeakerEmbeddingExtractor(
        config: *const SpeakerEmbeddingExtractorConfig,
    ) -> *const c_void;
    fn SherpaOnnxSpeakerEmbeddingExtractorDim(extractor: *const c_void) -> c_int;
    fn SherpaOnnxSpeakerEmbeddingExtractorCreateStream(extractor: *const c_void) -> *const c_void;
    fn SherpaOnnxOnlineStreamAcceptWaveform(
        stream: *const c_void,
        sample_rate: c_int,
        samples: *const c_float,
        count: c_int,
    );
    fn SherpaOnnxOnlineStreamInputFinished(stream: *const c_void);
    fn SherpaOnnxSpeakerEmbeddingExtractorIsReady(
        extractor: *const c_void,
        stream: *const c_void,
    ) -> c_int;
    fn SherpaOnnxSpeakerEmbeddingExtractorComputeEmbedding(
        extractor: *const c_void,
        stream: *const c_void,
    ) -> *const c_float;
    fn SherpaOnnxSpeakerEmbeddingExtractorDestroyEmbedding(embedding: *const c_float);
    fn SherpaOnnxDestroyOnlineStream(stream: *const c_void);
    fn SherpaOnnxDestroySpeakerEmbeddingExtractor(extractor: *const c_void);
}

/// The same sampling the application already does: a couple of seconds from the
/// middle of a segment, where a voice is steadiest.
const SAMPLE_MS: u64 = 2_000;
const SHORTEST_MS: u64 = 700;
const BYTES_PER_MS: u64 = 32;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().collect();
    let [_, model, audio, segments] = arguments.as_slice() else {
        eprintln!("usage: embed-segments <embedding-model.onnx> <audio.wav> <segments.json>");
        return ExitCode::FAILURE;
    };

    let timings = match read_timings(Path::new(segments)) {
        Ok(timings) => timings,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let audio = match read_pcm16_mono_16k(Path::new(audio)) {
        Ok(audio) => audio,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    let model = CString::new(model.as_str()).expect("a model path");
    let provider = CString::new(if cfg!(target_os = "macos") {
        "coreml"
    } else {
        "cpu"
    })
    .expect("a provider");
    let config = SpeakerEmbeddingExtractorConfig {
        model: model.as_ptr(),
        num_threads: 8,
        debug: 0,
        provider: provider.as_ptr(),
    };

    let extractor = unsafe { SherpaOnnxCreateSpeakerEmbeddingExtractor(&config) };
    if extractor.is_null() {
        eprintln!("The embedding model could not be loaded.");
        return ExitCode::FAILURE;
    }
    let dimensions = unsafe { SherpaOnnxSpeakerEmbeddingExtractorDim(extractor) };

    let started = std::time::Instant::now();
    let mut rows: Vec<String> = Vec::new();
    let mut skipped = 0usize;
    for (index, (start_ms, end_ms)) in timings.iter().enumerate() {
        let length = end_ms.saturating_sub(*start_ms);
        if length < SHORTEST_MS {
            skipped += 1;
            continue;
        }
        let taken = length.min(SAMPLE_MS);
        let from = ((start_ms + (length - taken) / 2) * BYTES_PER_MS / 2) as usize;
        let count = (taken * BYTES_PER_MS / 2) as usize;
        if from >= audio.len() {
            skipped += 1;
            continue;
        }
        let samples = &audio[from..(from + count).min(audio.len())];

        let Some(embedding) = embed(extractor, samples, dimensions) else {
            skipped += 1;
            continue;
        };
        let numbers: Vec<String> = embedding
            .iter()
            .map(|value| format!("{value:.6}"))
            .collect();
        rows.push(format!(
            "{{\"segment\":{index},\"startMs\":{start_ms},\"endMs\":{end_ms},\"embedding\":[{}]}}",
            numbers.join(",")
        ));
    }
    let elapsed = started.elapsed();

    unsafe { SherpaOnnxDestroySpeakerEmbeddingExtractor(extractor) };

    println!(
        "{{\"dimensions\":{dimensions},\"vectors\":[{}]}}",
        rows.join(",")
    );
    eprintln!(
        "{} segments embedded, {skipped} skipped, {dimensions} dimensions, {:.1} s",
        rows.len(),
        elapsed.as_secs_f64()
    );
    ExitCode::SUCCESS
}

/// One vector for one stretch of audio.
fn embed(extractor: *const c_void, samples: &[i16], dimensions: c_int) -> Option<Vec<f32>> {
    // The extractor wants floats in -1..1, and the working audio is 16-bit.
    let floats: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / 32768.0)
        .collect();
    unsafe {
        let stream = SherpaOnnxSpeakerEmbeddingExtractorCreateStream(extractor);
        SherpaOnnxOnlineStreamAcceptWaveform(
            stream,
            16_000,
            floats.as_ptr(),
            floats.len() as c_int,
        );
        SherpaOnnxOnlineStreamInputFinished(stream);
        let vector = if SherpaOnnxSpeakerEmbeddingExtractorIsReady(extractor, stream) != 0 {
            let raw = SherpaOnnxSpeakerEmbeddingExtractorComputeEmbedding(extractor, stream);
            let copied = std::slice::from_raw_parts(raw, dimensions as usize).to_vec();
            SherpaOnnxSpeakerEmbeddingExtractorDestroyEmbedding(raw);
            Some(copied)
        } else {
            None
        };
        SherpaOnnxDestroyOnlineStream(stream);
        vector
    }
}

/// Segment timings, from a transcript document or a bare array. Written by hand
/// because a study should not drag a JSON parser in for two field names.
fn read_timings(path: &Path) -> Result<Vec<(u64, u64)>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut timings = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("\"startMs\"") {
        rest = &rest[at + "\"startMs\"".len()..];
        let start = read_number(rest).ok_or("a startMs without a number")?;
        let at = rest.find("\"endMs\"").ok_or("a startMs with no endMs")?;
        rest = &rest[at + "\"endMs\"".len()..];
        let end = read_number(rest).ok_or("an endMs without a number")?;
        timings.push((start, end));
    }
    if timings.is_empty() {
        return Err(format!("{} holds no segments.", path.display()));
    }
    Ok(timings)
}

fn read_number(text: &str) -> Option<u64> {
    let digits: String = text
        .trim_start()
        .trim_start_matches(':')
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// The working audio, as samples. The format is checked rather than assumed,
/// because reading anything else as though it were this would produce noise that
/// the model would dutifully turn into a vector.
fn read_pcm16_mono_16k(path: &Path) -> Result<Vec<i16>, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{} is not a WAV file.", path.display()));
    }
    let mut at = 12usize;
    let mut format_seen = false;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let length =
            u32::from_le_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]])
                as usize;
        let body = at + 8;
        if id == b"fmt " && body + 16 <= bytes.len() {
            let channels = u16::from_le_bytes([bytes[body + 2], bytes[body + 3]]);
            let rate = u32::from_le_bytes([
                bytes[body + 4],
                bytes[body + 5],
                bytes[body + 6],
                bytes[body + 7],
            ]);
            let bits = u16::from_le_bytes([bytes[body + 14], bytes[body + 15]]);
            if channels != 1 || rate != 16_000 || bits != 16 {
                return Err(format!(
                    "This study needs 16 kHz mono 16-bit audio; {} is {rate} Hz, {channels} channel(s), {bits}-bit.",
                    path.display()
                ));
            }
            format_seen = true;
        } else if id == b"data" {
            if !format_seen {
                return Err("The audio describes no format.".into());
            }
            let end = (body + length).min(bytes.len());
            return Ok(bytes[body..end]
                .chunks_exact(2)
                .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
                .collect());
        }
        at = body + length + (length % 2);
    }
    Err(format!("{} holds no audio.", path.display()))
}
