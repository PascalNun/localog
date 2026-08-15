//! One speaker embedding for each segment of a transcript.
//!
//! Speaker separation runs after transcription, so where the speech is is already
//! known. This reads a couple of seconds from the middle of each known segment and
//! turns it into a vector. Deciding which of those vectors are the same person is
//! then arithmetic inside the application, not another pass over the audio, so
//! trying a different number of speakers costs nothing.
//!
//! It replaces a pass that ran a segmentation model to rediscover boundaries
//! transcription had already established. The study behind that decision is in
//! `spikes/speaker-embedding/`.
//!
//! ```text
//! localog-speaker-embedding --model <embedding.onnx> --audio <working.wav>
//!                           --segments <transcript.json> --out <vectors.bin>
//! ```
//!
//! The audio must be 16 kHz mono 16-bit PCM, which the application's working audio
//! already is. The vectors are written as a small binary file rather than to
//! standard output, because a meeting's worth of them is megabytes and a pipe is
//! the wrong place for that.

use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[repr(C)]
struct ExtractorConfig {
    model: *const c_char,
    num_threads: c_int,
    debug: c_int,
    provider: *const c_char,
}

// The four calls this needs from the sherpa-onnx C API, declared rather than
// generated: a binding generator would be a dependency and a build step to save
// writing nine lines.
unsafe extern "C" {
    fn SherpaOnnxCreateSpeakerEmbeddingExtractor(config: *const ExtractorConfig) -> *const c_void;
    fn SherpaOnnxDestroySpeakerEmbeddingExtractor(extractor: *const c_void);
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
}

/// How much of a segment to listen to. Two seconds is about what a speaker
/// embedding needs to place a voice; less starts losing quiet speakers, and more
/// spends time to learn nothing new. Measured in `spikes/speaker-embedding/`.
const SAMPLE_MS: u64 = 2_000;

/// Below this a segment is not embedded at all. Half a second of speech is as
/// likely to add a spurious voice as to identify anybody.
const SHORTEST_MS: u64 = 700;

/// 16 kHz, one channel, two bytes a frame.
const BYTES_PER_MS: u64 = 32;

/// What the application reads back. Versioned, because a file written by one
/// build and read by another must fail loudly rather than be misinterpreted.
const MAGIC: &[u8; 4] = b"LLEM";
const FORMAT_VERSION: u32 = 1;

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let arguments: Vec<String> = std::env::args().collect();
    let option = |name: &str| -> Option<String> {
        arguments
            .iter()
            .position(|argument| argument == name)
            .and_then(|at| arguments.get(at + 1))
            .cloned()
    };
    let (Some(model), Some(audio), Some(segments), Some(out)) = (
        option("--model"),
        option("--audio"),
        option("--segments"),
        option("--out"),
    ) else {
        return Err(
            "usage: localog-speaker-embedding --model <onnx> --audio <wav> \
                    --segments <json> --out <bin>"
                .into(),
        );
    };
    let threads: usize = option("--threads")
        .and_then(|value| value.parse().ok())
        .unwrap_or(4);

    let timings = read_timings(Path::new(&segments))?;
    let audio = read_working_audio(Path::new(&audio))?;

    let model = CString::new(model).map_err(|_| "The model path is not usable.".to_string())?;
    // Core ML on Apple hardware, plain CPU elsewhere, because the product also
    // targets Windows and Linux.
    let provider = CString::new(if cfg!(target_os = "macos") {
        "coreml"
    } else {
        "cpu"
    })
    .expect("a provider name");
    let config = ExtractorConfig {
        model: model.as_ptr(),
        num_threads: threads as c_int,
        debug: 0,
        provider: provider.as_ptr(),
    };
    let extractor = unsafe { SherpaOnnxCreateSpeakerEmbeddingExtractor(&config) };
    if extractor.is_null() {
        return Err("The speaker embedding model could not be loaded.".into());
    }
    let dimensions = unsafe { SherpaOnnxSpeakerEmbeddingExtractorDim(extractor) };
    if dimensions <= 0 {
        unsafe { SherpaOnnxDestroySpeakerEmbeddingExtractor(extractor) };
        return Err("The speaker embedding model reports no dimensions.".into());
    }

    let mut rows: Vec<(u32, Vec<f32>)> = Vec::new();
    let mut skipped = 0usize;
    for (index, (start_ms, end_ms)) in timings.iter().enumerate() {
        let Some(samples) = sample_of(&audio, *start_ms, *end_ms) else {
            skipped += 1;
            continue;
        };
        match embed(extractor, samples, dimensions) {
            Some(vector) => rows.push((index as u32, vector)),
            None => skipped += 1,
        }
    }
    unsafe { SherpaOnnxDestroySpeakerEmbeddingExtractor(extractor) };

    if rows.is_empty() {
        return Err("No segment was long enough to identify a voice from.".into());
    }
    write_vectors(Path::new(&out), dimensions as u32, &rows)?;
    Ok(format!(
        "{} embedded, {skipped} skipped, {dimensions} dimensions",
        rows.len()
    ))
}

/// The middle of a segment, where a voice is steadiest — the edges hold the breath
/// before a sentence and the fade after it.
fn sample_of(audio: &[i16], start_ms: u64, end_ms: u64) -> Option<&[i16]> {
    let length = end_ms.checked_sub(start_ms)?;
    if length < SHORTEST_MS {
        return None;
    }
    let taken = length.min(SAMPLE_MS);
    let from = ((start_ms + (length - taken) / 2) * BYTES_PER_MS / 2) as usize;
    let count = (taken * BYTES_PER_MS / 2) as usize;
    if from >= audio.len() {
        return None;
    }
    let until = (from + count).min(audio.len());
    // A sample cut short by the end of the recording is still worth embedding, but
    // a sliver of one is not.
    if (until - from) as u64 * 2 / BYTES_PER_MS < SHORTEST_MS {
        return None;
    }
    Some(&audio[from..until])
}

fn embed(extractor: *const c_void, samples: &[i16], dimensions: c_int) -> Option<Vec<f32>> {
    let floats: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / 32768.0)
        .collect();
    unsafe {
        let stream = SherpaOnnxSpeakerEmbeddingExtractorCreateStream(extractor);
        if stream.is_null() {
            return None;
        }
        SherpaOnnxOnlineStreamAcceptWaveform(
            stream,
            16_000,
            floats.as_ptr(),
            floats.len() as c_int,
        );
        SherpaOnnxOnlineStreamInputFinished(stream);
        let vector = if SherpaOnnxSpeakerEmbeddingExtractorIsReady(extractor, stream) != 0 {
            let raw = SherpaOnnxSpeakerEmbeddingExtractorComputeEmbedding(extractor, stream);
            if raw.is_null() {
                None
            } else {
                let copied = std::slice::from_raw_parts(raw, dimensions as usize).to_vec();
                SherpaOnnxSpeakerEmbeddingExtractorDestroyEmbedding(raw);
                Some(copied)
            }
        } else {
            None
        };
        SherpaOnnxDestroyOnlineStream(stream);
        vector
    }
}

/// Segment timings from a committed transcript, or a bare array of the same two
/// fields. Read by hand because a sidecar should not carry a JSON parser to find
/// two numbers, and because what it accepts is then exactly what is documented.
fn read_timings(path: &Path) -> Result<Vec<(u64, u64)>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut timings = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("\"startMs\"") {
        rest = &rest[at + "\"startMs\"".len()..];
        let start = number_after(rest).ok_or("A startMs has no number after it.")?;
        let at = rest
            .find("\"endMs\"")
            .ok_or("A startMs has no endMs after it.")?;
        rest = &rest[at + "\"endMs\"".len()..];
        let end = number_after(rest).ok_or("An endMs has no number after it.")?;
        timings.push((start, end));
    }
    if timings.is_empty() {
        return Err(format!("{} holds no segments.", path.display()));
    }
    Ok(timings)
}

fn number_after(text: &str) -> Option<u64> {
    text.trim_start()
        .strip_prefix(':')?
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

/// The application's working audio, as samples.
///
/// The format is checked rather than assumed, because the arithmetic that finds a
/// segment inside it is only correct for this one, and reading a 44.1 kHz stereo
/// file as though it were this would produce noise the model would dutifully turn
/// into a vector.
fn read_working_audio(path: &Path) -> Result<Vec<i16>, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{} is not a WAV file.", path.display()));
    }
    let mut at = 12usize;
    let mut format_seen = false;
    while at + 8 <= bytes.len() {
        let identifier = &bytes[at..at + 4];
        let length =
            u32::from_le_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]])
                as usize;
        let body = at + 8;
        if identifier == b"fmt " && body + 16 <= bytes.len() {
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
                    "The speaker pass needs 16 kHz mono 16-bit audio; {} is {rate} Hz, \
                     {channels} channel(s), {bits}-bit.",
                    path.display()
                ));
            }
            format_seen = true;
        } else if identifier == b"data" {
            if !format_seen {
                return Err("The working audio describes no format.".into());
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

/// `LLEM`, a version, the dimensions, then each vector preceded by the segment it
/// belongs to. Segments too short to embed are absent rather than zeroed, so the
/// application can tell "no voice found here" from "a voice of nothing".
fn write_vectors(path: &Path, dimensions: u32, rows: &[(u32, Vec<f32>)]) -> Result<(), String> {
    let mut out = Vec::with_capacity(16 + rows.len() * (4 + dimensions as usize * 4));
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&(rows.len() as u32).to_le_bytes());
    out.extend_from_slice(&dimensions.to_le_bytes());
    for (segment, vector) in rows {
        out.extend_from_slice(&segment.to_le_bytes());
        for value in vector {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    let temporary = PathBuf::from(format!("{}.part", path.display()));
    fs::write(&temporary, &out).map_err(|error| format!("{}: {error}", temporary.display()))?;
    // Renamed into place, so a reader never sees a half-written file.
    fs::rename(&temporary, path).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_segment_is_sampled_from_its_middle() {
        // Ten seconds where every sample records its own millisecond.
        let audio: Vec<i16> = (0..10_000)
            .flat_map(|ms| std::iter::repeat_n(ms as i16, 16))
            .collect();
        let sample = sample_of(&audio, 0, 6_000).expect("a sample");
        // Six seconds long, two taken from the middle: two to four.
        assert_eq!(sample[0], 2_000);
        assert_eq!(sample.len(), 32_000);
    }

    #[test]
    fn a_segment_too_short_to_place_a_voice_is_skipped() {
        let audio: Vec<i16> = vec![0; 16_000];
        assert!(sample_of(&audio, 0, 500).is_none());
    }

    #[test]
    fn a_segment_past_the_end_of_the_recording_is_skipped() {
        let audio: Vec<i16> = vec![0; 16_000];
        assert!(sample_of(&audio, 60_000, 63_000).is_none());
    }

    /// A segment the recording only partly covers is embedded from what is there,
    /// unless what is there is too little to identify anybody.
    #[test]
    fn a_segment_the_recording_cuts_short_is_used_if_enough_remains() {
        let audio: Vec<i16> = vec![7; 16_000]; // one second
        assert!(sample_of(&audio, 900, 3_000).is_none());
        let longer: Vec<i16> = vec![7; 32_000]; // two seconds
        assert!(sample_of(&longer, 900, 3_000).is_some());
    }

    #[test]
    fn timings_are_read_from_a_committed_transcript() {
        let file = std::env::temp_dir().join("localog-embed-timings.json");
        fs::write(
            &file,
            r#"{"segments":[{"startMs":0,"endMs":1200,"text":"a"},
                            {"startMs":1500,"endMs":4000,"text":"b"}]}"#,
        )
        .expect("a transcript");
        assert_eq!(
            read_timings(&file).expect("timings"),
            vec![(0, 1200), (1500, 4000)]
        );
        let _ = fs::remove_file(&file);
    }

    #[test]
    fn the_written_file_says_what_it_is() {
        let file = std::env::temp_dir().join("localog-embed-vectors.bin");
        write_vectors(&file, 2, &[(0, vec![1.0, 2.0]), (3, vec![-1.0, 0.5])]).expect("written");
        let bytes = fs::read(&file).expect("readable");
        assert_eq!(&bytes[0..4], MAGIC);
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 0);
        assert_eq!(bytes.len(), 16 + 2 * (4 + 2 * 4));
        let _ = fs::remove_file(&file);
    }
}
