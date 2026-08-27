//! Media facts and the regenerable mono/16 kHz PCM cache.

use crate::runtime::{ProcessLimits, RuntimeConfig, run_process};
use serde::Deserialize;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;

#[derive(Clone, Debug, Deserialize, Default)]
pub(crate) struct Probe {
    pub format: Option<Format>,
    #[serde(default)]
    pub streams: Vec<Stream>,
}
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Format {
    pub duration: Option<String>,
    pub format_name: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Stream {
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
}

pub(crate) fn parse_probe(json: &str) -> Result<Probe, String> {
    serde_json::from_str(json).map_err(|_| "probeInvalid".into())
}

pub(crate) fn probe(
    ffprobe: &Path,
    source: &Path,
    cancellation: &AtomicBool,
) -> Result<Probe, String> {
    let mut command = Command::new(ffprobe);
    command
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(source);
    let output =
        run_process(command, cancellation, media_limits()).map_err(|error| error.to_string())?;
    parse_probe(&output.stdout)
}

/// What a media tool is allowed to say back before it is stopped.
///
/// ffmpeg and ffprobe are terse; this is far more than either uses, and far less
/// than a tool stuck in a loop could fill a disk with.
fn media_limits() -> ProcessLimits {
    ProcessLimits::with_max_output(512 * 1024)
}

/// Run a media tool, taking the half-written file with it if it fails.
///
/// A temporary left behind by a failed encode is a file the next run can mistake
/// for a finished one. All three encoders here want that same clean-up, and each
/// wrote it out again.
fn run_media_tool(
    command: Command,
    cancellation: &AtomicBool,
    temporary: &Path,
) -> Result<(), String> {
    run_process(command, cancellation, media_limits()).map_err(|error| {
        let _ = fs::remove_file(temporary);
        error.to_string()
    })?;
    Ok(())
}

pub(crate) fn normalize(
    ffmpeg: &Path,
    source: &Path,
    destination: &Path,
    cancellation: &AtomicBool,
    mut progress: impl FnMut(u64),
) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or("cachePathInvalid")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = destination.with_extension("wav.part");
    let _ = fs::remove_file(&temporary);
    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-nostdin", "-y", "-i"])
        .arg(source)
        .args([
            "-vn",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ])
        .arg(&temporary);
    progress(10);
    run_media_tool(command, cancellation, &temporary)?;
    progress(90);
    if !temporary.is_file() {
        return Err("normalizerNoOutput".into());
    }
    // Make the derived cache durable before exposing it at its final path.
    if let Err(error) = fs::File::open(&temporary).and_then(|file| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    if destination.exists()
        && let Err(error) = fs::remove_file(destination)
    {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    if let Err(error) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    progress(100);
    Ok(())
}

pub(crate) fn whisper_command(
    config: &RuntimeConfig,
    normalized: &Path,
    output_base: &Path,
    language: &str,
    vocabulary_prompt: Option<&str>,
) -> Command {
    let mut command = Command::new(&config.executable);
    command
        .args(["-m"])
        .arg(&config.model)
        .args(["-f"])
        .arg(normalized)
        // The full form carries per-token probabilities, which is how a passage the
        // model was unsure of can be shown to the reader instead of read as fact.
        .args(["--output-json-full", "--output-file"])
        .arg(output_base)
        .args(["--language", language, "--print-progress"]);
    if let Some(prompt) = vocabulary_prompt.filter(|value| !value.trim().is_empty()) {
        // Without --carry-initial-prompt the terms only bias the first window,
        // which is 30 seconds of a meeting that may run for hours.
        command
            .args(["--prompt", prompt])
            .arg("--carry-initial-prompt");
    }
    command
}

/// Characters of vocabulary the transcription runtime will accept. whisper caps
/// the initial prompt at half its text context, about 224 tokens, so the list has
/// to be prioritised rather than accumulated.
const VOCABULARY_PROMPT_LIMIT: usize = 620;

/// Build the initial prompt from a project's terms, most specific first, stopping
/// before the runtime's limit.
///
/// Ordering matters more than volume. Measured against a real meeting, standard
/// professional terminology was already transcribed correctly with no help, while
/// every term the vocabulary actually corrected was a proper noun. Spending this
/// budget on words the model already knows wastes it.
pub(crate) fn vocabulary_prompt(terms: &[String]) -> Option<String> {
    let mut chosen: Vec<&str> = Vec::new();
    let mut length = 0;
    for term in terms {
        let term = term.trim();
        if term.is_empty() || chosen.contains(&term) {
            continue;
        }
        let addition = term.len() + 2;
        if length + addition > VOCABULARY_PROMPT_LIMIT {
            continue;
        }
        length += addition;
        chosen.push(term);
    }
    if chosen.is_empty() {
        return None;
    }
    Some(chosen.join(", "))
}

/// Build the short working file the diariser listens to.
///
/// Rather than replaying the whole recording, this writes a couple of seconds of
/// each transcript segment end to end, separated by silence. The diariser then
/// embeds a fraction of the audio for the same clustering job, and separation
/// stops costing longer than transcription and generation together. The silence
/// is there so the diariser's own segmentation breaks where we joined rather than
/// running two speakers together.
///
/// The working audio is already 16 kHz mono 16-bit PCM that this module wrote, so
/// the samples are copied out of it by arithmetic rather than by asking a tool to
/// cut them. That is the whole reason the mapping back can be trusted.
///
/// Both tools were tried first and neither is exact. A filter graph of 753 `atrim`
/// filters splits the decoded stream once per sample and reads it through for
/// each; it had not finished after ten minutes, which is worse than the pass it
/// exists to shorten. The concat demuxer builds the file in under four seconds but
/// rounds each out point up to a packet boundary, which measured 1767.7 seconds
/// where 1731.6 was planned - about 48 ms of drift per sample, accumulating, so
/// the last of an eighty-minute meeting's turns would be read back against audio
/// thirty-six seconds away from where they actually are. A speaker pass that is
/// confidently wrong about the end of a meeting is worse than one that is slow.
///
/// Copying bytes has neither problem: every sample lands exactly where the plan
/// says, and a sample running past the end of the recording is padded rather than
/// shortened, so one short read cannot shift everything after it.
pub(crate) fn condense_for_diarisation(
    normalized: &Path,
    samples: &[crate::diarisation::Sample],
    destination: &Path,
) -> Result<(), String> {
    let Some(last) = samples.last() else {
        return Err("speakerPassNoAudio".into());
    };
    let (data_offset, data_length) = pcm16_mono_16k_data(normalized)?;

    let mut source = fs::File::open(normalized)
        .map_err(|error| format!("The speaker pass could not read the working audio: {error}"))?;
    let planned_bytes = usize::try_from(last.condensed_end_ms * BYTES_PER_MS)
        .map_err(|_| "speakerPassTooMuchAudio".to_string())?;
    // Everything not written to is silence, which is what the gaps between the
    // samples are made of.
    let mut condensed = vec![0u8; planned_bytes];

    for sample in samples {
        let at = usize::try_from(sample.condensed_start_ms * BYTES_PER_MS)
            .map_err(|_| "speakerPassTooMuchAudio".to_string())?;
        let wanted = ((sample.source_end_ms - sample.source_start_ms) * BYTES_PER_MS) as usize;
        let from = sample.source_start_ms * BYTES_PER_MS;
        // A sample reaching past the end of the recording is read as far as it
        // goes and the rest of its room stays silent. Shortening it instead would
        // move every later sample and undo the exactness this exists for.
        let available = data_length.saturating_sub(from).min(wanted as u64) as usize;
        if available == 0 {
            continue;
        }
        source
            .seek(SeekFrom::Start(data_offset + from))
            .map_err(|error| {
                format!("The speaker pass could not read the working audio: {error}")
            })?;
        source
            .read_exact(&mut condensed[at..at + available])
            .map_err(|error| {
                format!("The speaker pass could not read the working audio: {error}")
            })?;
    }

    write_pcm16_mono_16k(destination, &condensed)
}

/// Write the working audio a person's edits leave behind.
///
/// The same arithmetic as the speaker pass: the working file is 16 kHz mono 16-bit
/// PCM that this module wrote, so keeping part of it is a byte range and not a
/// question for a tool. Exact by construction, and quick enough that trimming a
/// long meeting is not something to wait for.
///
/// The recording this reads is never modified. That is the whole point of holding
/// the edits as a description: somebody who trims two minutes and then finds the
/// decision was inside them has lost nothing.
/// Store a recording as Opus.
///
/// A recorder writes plain PCM, because a format that survives being killed is
/// worth more during a meeting than a small one. Afterwards the size matters and
/// the risk has passed: 48 kHz mono 16-bit costs 5.76 MB a minute, so two tracks
/// over a ninety-minute meeting is about a gigabyte, and the same recording at 32
/// kbps Opus is about twenty-nine megabytes.
///
/// Opus rather than anything else because it is built for speech, transparent at
/// this rate, and unencumbered — which matters to a GPL project in the way MP3's
/// history should remind everybody.
///
/// This lives here rather than in the recorder so that every platform's recorder
/// can stay as small as possible. Writing a WAV is something a few dozen lines of
/// any language can do; encoding is not, and doing it once here means the Linux
/// and Windows recorders inherit it.
// Reachable once a recorder writes into the application. The encoding lands with
// the rest of the recording work rather than after it, because it is the one part
// that needs no permission from anybody's operating system to be proven.
#[allow(dead_code)]
pub(crate) fn encode_to_opus(
    ffmpeg: &Path,
    source: &Path,
    destination: &Path,
    kilobits: u32,
    cancellation: &AtomicBool,
) -> Result<(), String> {
    let temporary = destination.with_extension("opus.part");
    let _ = fs::remove_file(&temporary);
    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-nostdin", "-y", "-i"])
        .arg(source)
        .args(["-c:a", "libopus", "-b:a"])
        .arg(format!("{kilobits}k"))
        // Mono, because a meeting is speech and the second channel would double
        // the file to say the same thing twice.
        .args(["-ac", "1", "-f", "opus"])
        .arg(&temporary);
    run_media_tool(command, cancellation, &temporary)?;

    // Check what was written against what was read. An encode that stops early
    // loses the end of a meeting and reports success, which is the failure nobody
    // notices until they look for something that was said at the end.
    let written = fs::metadata(&temporary)
        .map_err(|error| {
            let _ = fs::remove_file(&temporary);
            format!("The recording could not be stored: {error}")
        })?
        .len();
    if written == 0 {
        let _ = fs::remove_file(&temporary);
        return Err("recordingEmpty".into());
    }
    let (_, data_length) = pcm16_mono_16k_data(source).unwrap_or((0, 0));
    if data_length > 0 {
        // Roughly what this many seconds should cost at this rate, allowing that
        // a variable bitrate spends less on quiet passages than on speech.
        let seconds = data_length as f64 / (BYTES_PER_MS * 1000) as f64;
        let least = (seconds * kilobits as f64 * 1000.0 / 8.0 * 0.25) as u64;
        if written < least {
            let _ = fs::remove_file(&temporary);
            return Err(format!(
                "recordingTooSmall:{written} bytes for {seconds:.0} seconds"
            ));
        }
    }
    fs::rename(&temporary, destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("The recording could not be stored: {error}")
    })
}

/// The shape of a recording, as peaks, for drawing.
///
/// One value per bucket, from zero to one, taken as the loudest sample in that
/// stretch rather than the average. A meeting is mostly quiet with speech on top of
/// it, and averaging turns that into a flat band that shows a person nothing. The
/// peak is what makes silence look like silence, which is the thing somebody is
/// looking for when they trim the start of a recording.
///
/// Read straight from the working file, in one pass, without holding it all in
/// memory: an eighty-minute meeting is 150 MB and a person waiting to trim it
/// should not wait for that.
pub(crate) fn waveform(source: &Path, buckets: usize) -> Result<Vec<f32>, String> {
    let buckets = buckets.clamp(1, 100_000);
    let (data_offset, data_length) = pcm16_mono_16k_data(source)?;
    let frames = data_length / 2;
    if frames == 0 {
        return Ok(vec![0.0; buckets]);
    }
    let mut file = fs::File::open(source)
        .map_err(|error| format!("The recording could not be read: {error}"))?;
    file.seek(SeekFrom::Start(data_offset))
        .map_err(|error| format!("The recording could not be read: {error}"))?;

    let mut peaks = vec![0.0f32; buckets];
    let mut buffer = vec![0u8; 1 << 16];
    let mut frame = 0u64;
    let mut left = data_length;
    while left > 0 {
        let wanted = buffer.len().min(left as usize) & !1;
        if wanted == 0 {
            break;
        }
        let read = match file.read(&mut buffer[..wanted]) {
            Ok(0) => break,
            Ok(read) => read & !1,
            Err(error) => return Err(format!("The recording could not be read: {error}")),
        };
        for pair in buffer[..read].chunks_exact(2) {
            // Which bucket this frame falls in, by position rather than by
            // counting, so rounding cannot drift over a long recording.
            let bucket = ((frame * buckets as u64) / frames) as usize;
            let sample = i16::from_le_bytes([pair[0], pair[1]]);
            let level = (sample as f32 / 32768.0).abs();
            if let Some(peak) = peaks.get_mut(bucket)
                && level > *peak
            {
                *peak = level;
            }
            frame += 1;
        }
        left -= read as u64;
    }
    Ok(peaks)
}

// Reachable once the review screen exists; the cutting is proven first because a
// screen built on unchecked arithmetic loses minutes of a meeting quietly.
#[allow(dead_code)]
pub(crate) fn apply_edits(
    source: &Path,
    duration_ms: u64,
    edits: &crate::edits::Edits,
    destination: &Path,
) -> Result<(), String> {
    let spans = crate::edits::kept(duration_ms, edits);
    if spans.is_empty() {
        return Err("editsLeaveNothing".into());
    }
    let (data_offset, data_length) = pcm16_mono_16k_data(source)?;
    let mut file = fs::File::open(source)
        .map_err(|error| format!("The recording could not be read: {error}"))?;

    let mut kept = Vec::new();
    for span in &spans {
        let from = span.from_ms * BYTES_PER_MS;
        let wanted = span.length_ms() * BYTES_PER_MS;
        // A span reaching past the end of the audio keeps what is there. The
        // duration comes from a probe and the file is what it is; trusting the
        // probe over the file would read whatever follows it.
        let available = data_length.saturating_sub(from).min(wanted);
        if available == 0 {
            continue;
        }
        let at = kept.len();
        kept.resize(at + available as usize, 0);
        file.seek(SeekFrom::Start(data_offset + from))
            .map_err(|error| format!("The recording could not be read: {error}"))?;
        file.read_exact(&mut kept[at..])
            .map_err(|error| format!("The recording could not be read: {error}"))?;
    }
    if kept.is_empty() {
        return Err("editsLeaveNothing".into());
    }
    write_pcm16_mono_16k(destination, &kept)
}

/// Bytes of working audio per millisecond: 16 kHz, one channel, two bytes a frame.
const BYTES_PER_MS: u64 = 32;

/// Locate the audio inside a WAV file, and refuse anything that is not the
/// working format.
///
/// The format is checked rather than assumed because the arithmetic above is only
/// correct for it, and reading a stereo or 44.1 kHz file as though it were this
/// one would produce noise that the diariser would dutifully cluster.
fn pcm16_mono_16k_data(path: &Path) -> Result<(u64, u64), String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("The speaker pass could not read the working audio: {error}"))?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)
        .map_err(|_| "workingAudioUnreadable".to_string())?;
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err("workingAudioNotWav".into());
    }
    let mut position = 12u64;
    let mut format_seen = false;
    loop {
        let mut chunk = [0u8; 8];
        if file.read_exact(&mut chunk).is_err() {
            return Err("workingAudioSilent".into());
        }
        let id = [chunk[0], chunk[1], chunk[2], chunk[3]];
        let length = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) as u64;
        position += 8;
        if &id == b"fmt " {
            let mut format = [0u8; 16];
            file.read_exact(&mut format)
                .map_err(|_| "workingAudioFormatUnreadable".to_string())?;
            let encoding = u16::from_le_bytes([format[0], format[1]]);
            let channels = u16::from_le_bytes([format[2], format[3]]);
            let rate = u32::from_le_bytes([format[4], format[5], format[6], format[7]]);
            let bits = u16::from_le_bytes([format[14], format[15]]);
            if encoding != 1 || channels != 1 || rate != 16_000 || bits != 16 {
                return Err(format!(
                    "workingAudioFormatWrong:{rate} Hz, {channels} channel(s), {bits}-bit"
                ));
            }
            format_seen = true;
        } else if &id == b"data" {
            if !format_seen {
                return Err("workingAudioNoFormat".into());
            }
            return Ok((position, length));
        }
        // Chunks are padded to an even length.
        position += length + (length % 2);
        file.seek(SeekFrom::Start(position))
            .map_err(|error| error.to_string())?;
    }
}

/// Write 16 kHz mono 16-bit PCM as a plain WAV, which is what the diariser reads.
fn write_pcm16_mono_16k(destination: &Path, audio: &[u8]) -> Result<(), String> {
    let length = u32::try_from(audio.len())
        .map_err(|_| "condensedAudioTooLarge".to_string())?;
    let mut file = fs::File::create(destination)
        .map_err(|error| format!("The speaker pass could not write its audio: {error}"))?;
    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(36 + length).to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&16u32.to_le_bytes()); // chunk length
    header.extend_from_slice(&1u16.to_le_bytes()); // uncompressed PCM
    header.extend_from_slice(&1u16.to_le_bytes()); // one channel
    header.extend_from_slice(&16_000u32.to_le_bytes());
    header.extend_from_slice(&32_000u32.to_le_bytes()); // bytes a second
    header.extend_from_slice(&2u16.to_le_bytes()); // bytes a frame
    header.extend_from_slice(&16u16.to_le_bytes()); // bits a sample
    header.extend_from_slice(b"data");
    header.extend_from_slice(&length.to_le_bytes());
    file.write_all(&header)
        .and_then(|_| file.write_all(audio))
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("The speaker pass could not write its audio: {error}"))
}

/// How the diariser should be run for one recording.
pub(crate) struct DiarisationRequest<'a> {
    pub executable: &'a Path,
    pub segmentation_model: &'a Path,
    pub embedding_model: &'a Path,
    pub normalized: &'a Path,
    /// Supplied when the number of people in the meeting is known. Clustering a long
    /// recording by similarity alone splits one voice into many as the recording goes
    /// on: an 81-minute meeting of about eleven people produced 86 speakers, while the
    /// same audio with the count supplied produced a sensible number.
    pub expected_speakers: Option<u32>,
}

/// Both networks default to a single thread and to plain CPU. Using the machine's
/// cores and its neural accelerator measured 1.64 times faster with no other change.
pub(crate) fn diarisation_command(request: &DiarisationRequest<'_>) -> Command {
    let threads = worker_threads();
    // Core ML is useful on macOS, but the product also targets Windows and Linux.
    // Keep the command portable by selecting the accelerator only where it exists.
    let provider = if cfg!(target_os = "macos") {
        "coreml"
    } else {
        "cpu"
    };
    let mut command = Command::new(request.executable);
    command
        .arg(format!(
            "--segmentation.pyannote-model={}",
            request.segmentation_model.display()
        ))
        .arg(format!("--segmentation.num-threads={threads}"))
        .arg(format!("--segmentation.provider={provider}"))
        .arg(format!(
            "--embedding.model={}",
            request.embedding_model.display()
        ))
        .arg(format!("--embedding.num-threads={threads}"))
        .arg(format!("--embedding.provider={provider}"));
    // Always a count, never a threshold. Clustering by similarity alone was
    // measured on the reference meeting at eighty-six speakers where eleven spoke,
    // because one voice drifts over eighty minutes of videoconference. The
    // pipeline now declines to run without a count rather than spending half an
    // hour reaching that answer.
    if let Some(count) = request.expected_speakers.filter(|count| *count >= 2) {
        command.arg(format!("--clustering.num-clusters={count}"));
    }
    command.arg(request.normalized);
    command
}

/// Threads to give a model runtime: enough to use the machine, while leaving room
/// for the interface to stay responsive.
pub(crate) fn worker_threads() -> usize {
    std::thread::available_parallelism()
        .map(|value| (value.get().saturating_sub(2)).clamp(1, 8))
        .unwrap_or(1)
}

pub(crate) fn expected_json_path(base: &Path) -> PathBuf {
    base.with_extension("json")
}

/// whisper.cpp documents `--output-file` as a path without an extension. A few
/// packaged builds have nevertheless written the JSON directly to that path;
/// accept that harmless variation while keeping the normal `.json` contract.
pub(crate) fn json_output_path(base: &Path) -> Option<PathBuf> {
    let expected = expected_json_path(base);
    if expected.is_file() {
        return Some(expected);
    }
    if base.is_file() {
        return Some(base.to_path_buf());
    }
    None
}

/// Parse a whisper.cpp `--print-progress` stderr line into a 0..=100 percentage.
/// The real format is `whisper_print_progress_callback: progress =  43%` with
/// variable leading whitespace before the number.
pub(crate) fn parse_whisper_progress(line: &str) -> Option<u8> {
    let marker = "progress =";
    let start = line.find(marker)? + marker.len();
    let number = line[start..].trim().strip_suffix('%')?.trim();
    number.parse::<u8>().ok().map(|value| value.min(100))
}

/// Combine a recording's two tracks into the one file everything downstream expects.
///
/// A meeting recorded here arrives as two: the room and the call. Everything after
/// this point — normalising, transcribing, separating speakers — takes one source,
/// and a person wants one transcript of one meeting rather than two of halves.
///
/// Mixed rather than concatenated, obviously, since the tracks are simultaneous. The
/// inputs are the same length by construction, but `amix` is told the longest anyway:
/// a recorder killed mid-second finalises its two files independently and they can
/// differ by a checkpoint, and truncating to the shorter would silently drop the end
/// of a meeting.
///
/// Both originals are kept. Mixing is the only step in this pipeline that cannot be
/// undone from what it produces, and the tracks are worth keeping for their own sake:
/// knowing which words came from the room and which from the call is speaker
/// separation of the coarsest and most reliable kind, and nothing uses it yet.
pub(crate) fn combine_tracks(
    ffmpeg: &Path,
    system: &Path,
    microphone: &Path,
    destination: &Path,
    cancellation: &AtomicBool,
) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or("combinedPathInvalid")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = destination.with_extension("wav.part");
    let _ = fs::remove_file(&temporary);

    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-nostdin", "-y", "-i"])
        .arg(microphone)
        .arg("-i")
        .arg(system)
        .args([
            "-filter_complex",
            // normalize=0 keeps each track at its own level. Averaging them, which is
            // the default, halves a room that is already the quieter of the two.
            "[0:a][1:a]amix=inputs=2:duration=longest:normalize=0[out]",
            "-map",
            "[out]",
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ])
        .arg(&temporary);

    run_media_tool(command, cancellation, &temporary)?;
    fs::rename(&temporary, destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        error.to_string()
    })
}

#[cfg(test)]
mod tests {
    /// Write a WAV where every frame's value is the millisecond it belongs to, so
    /// that a byte in the condensed file says exactly where it came from.
    fn recognisable_recording(path: &Path, milliseconds: u64) {
        let mut audio = Vec::with_capacity((milliseconds * BYTES_PER_MS) as usize);
        for ms in 0..milliseconds {
            for _ in 0..16 {
                audio.extend_from_slice(&(ms as i16).to_le_bytes());
            }
        }
        write_pcm16_mono_16k(path, &audio).expect("a source recording");
    }

    fn millisecond_at(audio: &[u8], ms: u64) -> i16 {
        let at = (ms * BYTES_PER_MS) as usize;
        i16::from_le_bytes([audio[at], audio[at + 1]])
    }

    /// The claim this whole scheme rests on: a turn found at a given moment in the
    /// condensed file came from exactly the segment the plan says it did. Both
    /// ffmpeg routes tried before this one drifted, so it is checked rather than
    /// asserted in a comment.
    #[test]
    fn every_sample_lands_exactly_where_the_plan_says() {
        let directory = std::env::temp_dir().join("localog-condense-exact");
        std::fs::create_dir_all(&directory).expect("working directory");
        let source = directory.join("normalized.wav");
        let destination = directory.join("condensed.wav");
        recognisable_recording(&source, 60_000);

        // Sixty segments across the minute, so any per-sample drift accumulates
        // into something a test can see.
        let timings: Vec<(u64, u64)> = (0..60).map(|i| (i * 1_000, i * 1_000 + 900)).collect();
        let samples = crate::diarisation::plan_samples(
            &timings,
            crate::diarisation::SAMPLE_MS,
            crate::diarisation::GAP_MS,
            crate::diarisation::SHORTEST_MS,
        );
        assert_eq!(samples.len(), 60);

        condense_for_diarisation(&source, &samples, &destination).expect("condensation");
        let written = std::fs::read(&destination).expect("output");
        let (offset, length) = pcm16_mono_16k_data(&destination).expect("a readable result");
        let audio = &written[offset as usize..(offset + length) as usize];

        let planned = samples.last().expect("a sample").condensed_end_ms;
        assert_eq!(
            length,
            planned * BYTES_PER_MS,
            "the file is not the length that was planned"
        );

        for sample in &samples {
            // The first and last millisecond of each sample, so a sample that is
            // in the right place but the wrong length is caught too.
            assert_eq!(
                millisecond_at(audio, sample.condensed_start_ms),
                sample.source_start_ms as i16,
                "sample from segment {} starts in the wrong place",
                sample.segment
            );
            assert_eq!(
                millisecond_at(audio, sample.condensed_end_ms - 1),
                (sample.source_end_ms - 1) as i16,
                "sample from segment {} ends in the wrong place",
                sample.segment
            );
            // And the gap after it really is silent, or the diariser's
            // segmentation would not break where we joined.
            if sample.condensed_end_ms * BYTES_PER_MS < length {
                assert_eq!(millisecond_at(audio, sample.condensed_end_ms), 0);
            }
        }
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// A sample running past the end of the recording keeps its room, because
    /// shortening it would move every sample after it.
    #[test]
    fn a_sample_past_the_end_is_padded_rather_than_shortened() {
        let directory = std::env::temp_dir().join("localog-condense-short");
        std::fs::create_dir_all(&directory).expect("working directory");
        let source = directory.join("normalized.wav");
        let destination = directory.join("condensed.wav");
        recognisable_recording(&source, 3_000);

        // The second segment is beyond the recording entirely.
        let timings = [(0, 2_000), (10_000, 12_000)];
        let samples = crate::diarisation::plan_samples(
            &timings,
            crate::diarisation::SAMPLE_MS,
            crate::diarisation::GAP_MS,
            crate::diarisation::SHORTEST_MS,
        );
        condense_for_diarisation(&source, &samples, &destination).expect("condensation");

        let (_, length) = pcm16_mono_16k_data(&destination).expect("a readable result");
        let planned = samples.last().expect("a sample").condensed_end_ms;
        assert_eq!(length, planned * BYTES_PER_MS);
    }

    /// Build the condensed file from a real transcript and a real recording, so
    /// the speaker pass can be measured against audio it will actually meet.
    ///
    /// Set LOCALOG_CONDENSE_TRANSCRIPT to a committed transcript JSON,
    /// LOCALOG_CONDENSE_SOURCE to its 16 kHz mono working audio, and
    /// LOCALOG_CONDENSE_OUT to where the result should go. The file is left behind
    /// on purpose: running the diariser over it is the measurement.
    #[test]
    #[ignore = "requires a real transcript and its working audio"]
    fn condense_a_real_meeting() {
        let transcript = std::env::var("LOCALOG_CONDENSE_TRANSCRIPT").expect("a transcript");
        let source = PathBuf::from(std::env::var("LOCALOG_CONDENSE_SOURCE").expect("audio"));
        let destination = PathBuf::from(std::env::var("LOCALOG_CONDENSE_OUT").expect("a result"));

        // Only the timings are wanted, and a committed artifact carries the durable
        // fields rather than the whole in-memory document.
        let document: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&transcript).expect("readable"))
                .expect("a transcript document");
        let timings: Vec<(u64, u64)> = document["segments"]
            .as_array()
            .expect("segments")
            .iter()
            .map(|segment| {
                (
                    segment["startMs"].as_u64().expect("a start"),
                    segment["endMs"].as_u64().expect("an end"),
                )
            })
            .collect();
        let spoken: u64 = timings.iter().map(|(from, to)| to - from).sum();

        let samples = crate::diarisation::plan_samples(
            &timings,
            crate::diarisation::SAMPLE_MS,
            crate::diarisation::GAP_MS,
            crate::diarisation::SHORTEST_MS,
        );
        condense_for_diarisation(&source, &samples, &destination).expect("condensation");
        let condensed = samples.last().expect("a sample").condensed_end_ms;
        println!(
            "{} segments, {} sampled; {:.1} min of speech in a {:.1} min recording condensed to {:.1} min",
            timings.len(),
            samples.len(),
            spoken as f64 / 60_000.0,
            timings.last().expect("a segment").1 as f64 / 60_000.0,
            condensed as f64 / 60_000.0,
        );
    }

    /// Storing a recording must actually store it. Runs the real ffmpeg, because
    /// the failure this guards against — an encode that stops early and reports
    /// success — cannot happen without one.
    #[test]
    #[ignore = "requires ffmpeg"]
    fn a_recording_stored_as_opus_keeps_its_length() {
        let ffmpeg = PathBuf::from(std::env::var("LOCALOG_FFMPEG").unwrap_or("ffmpeg".into()));
        let directory = std::env::temp_dir().join("localog-opus");
        std::fs::create_dir_all(&directory).expect("working directory");
        let source = directory.join("recording.wav");
        let destination = directory.join("recording.opus");

        // Sixty seconds of a tone, so the encoder has something to spend bits on;
        // pure silence compresses to almost nothing and would prove little.
        let mut audio = Vec::new();
        for frame in 0..(16_000 * 60u32) {
            let value = ((frame as f32 * 0.05).sin() * 8_000.0) as i16;
            audio.extend_from_slice(&value.to_le_bytes());
        }
        write_pcm16_mono_16k(&source, &audio).expect("a recording");

        let cancellation = AtomicBool::new(false);
        encode_to_opus(&ffmpeg, &source, &destination, 32, &cancellation).expect("stored");

        let raw = std::fs::metadata(&source).expect("source").len();
        let stored = std::fs::metadata(&destination).expect("stored").len();
        println!("{raw} bytes of PCM stored as {stored} bytes of Opus");
        assert!(stored > 0);
        // A minute at 32 kbps is roughly 240 KB; well under a tenth of the PCM.
        assert!(
            stored < raw / 5,
            "Opus should be far smaller: {stored} vs {raw}"
        );
        // And not so small that the encode clearly stopped early.
        assert!(stored > 60 * 32 * 1000 / 8 / 4, "too small to be a minute");
        // The partial file must never be left behind.
        assert!(!directory.join("recording.opus.part").exists());
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// A waveform has to show where the sound is, or trimming the quiet start of a
    /// recording is guesswork. Built from a fixture that is silent, then loud, then
    /// silent again.
    #[test]
    fn a_waveform_shows_where_the_sound_is() {
        let directory = std::env::temp_dir().join("localog-waveform");
        std::fs::create_dir_all(&directory).expect("working directory");
        let source = directory.join("recording.wav");
        let mut audio = Vec::new();
        for ms in 0..30_000u64 {
            // Loud only in the middle third.
            let value: i16 = if (10_000..20_000).contains(&ms) {
                30_000
            } else {
                0
            };
            for _ in 0..16 {
                audio.extend_from_slice(&value.to_le_bytes());
            }
        }
        write_pcm16_mono_16k(&source, &audio).expect("a recording");

        let peaks = waveform(&source, 30).expect("a waveform");
        assert_eq!(peaks.len(), 30);
        assert!(
            peaks[..10].iter().all(|peak| *peak == 0.0),
            "the quiet start"
        );
        assert!(
            peaks[10..20].iter().all(|peak| *peak > 0.9),
            "the loud middle: {:?}",
            &peaks[10..20]
        );
        assert!(peaks[20..].iter().all(|peak| *peak == 0.0), "the quiet end");
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// The peak and not the average: a meeting is mostly quiet with speech on top,
    /// and averaging flattens exactly the thing somebody is looking at.
    #[test]
    fn a_lone_loud_moment_survives_into_the_waveform() {
        let directory = std::env::temp_dir().join("localog-waveform-peak");
        std::fs::create_dir_all(&directory).expect("working directory");
        let source = directory.join("recording.wav");
        // Ten seconds of quiet with a single loud millisecond in the middle.
        let mut audio = Vec::new();
        for ms in 0..10_000u64 {
            let value: i16 = if ms == 5_000 { 32_000 } else { 0 };
            for _ in 0..16 {
                audio.extend_from_slice(&value.to_le_bytes());
            }
        }
        write_pcm16_mono_16k(&source, &audio).expect("a recording");
        let peaks = waveform(&source, 10).expect("a waveform");
        assert!(peaks[5] > 0.9, "the loud moment should still be visible");
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// Cutting must take exactly the audio the edits describe, and must leave the
    /// recording it read alone. A test that only checks the length would pass on a
    /// file that kept the wrong seconds.
    #[test]
    fn edits_keep_exactly_the_audio_they_describe() {
        let directory = std::env::temp_dir().join("localog-apply-edits");
        std::fs::create_dir_all(&directory).expect("working directory");
        let source = directory.join("recording.wav");
        let destination = directory.join("edited.wav");
        // Sixty seconds where every frame records the millisecond it belongs to.
        let mut audio = Vec::new();
        for ms in 0..60_000u64 {
            for _ in 0..16 {
                audio.extend_from_slice(&(ms as i16).to_le_bytes());
            }
        }
        write_pcm16_mono_16k(&source, &audio).expect("a recording");
        let before = std::fs::read(&source).expect("readable");

        // Start at ten seconds, end at fifty, and drop twenty to thirty.
        let edits = crate::edits::Edits {
            start_ms: 10_000,
            end_ms: Some(50_000),
            removed: vec![crate::edits::Span {
                from_ms: 20_000,
                to_ms: 30_000,
            }],
        };
        apply_edits(&source, 60_000, &edits, &destination).expect("edited");

        let written = std::fs::read(&destination).expect("output");
        let (offset, length) = pcm16_mono_16k_data(&destination).expect("readable");
        let kept = &written[offset as usize..(offset + length) as usize];
        // Ten to twenty and thirty to fifty: thirty seconds.
        assert_eq!(length, 30_000 * BYTES_PER_MS);
        let at = |ms: u64| {
            let byte = (ms * BYTES_PER_MS) as usize;
            i16::from_le_bytes([kept[byte], kept[byte + 1]])
        };
        // Compared through the same conversion the fixture used, because a
        // millisecond past 32767 does not fit in a sample and wraps.
        let millisecond = |ms: u64| ms as i16;
        assert_eq!(at(0), millisecond(10_000), "the edit begins at the trim");
        assert_eq!(
            at(9_999),
            millisecond(19_999),
            "the last moment before the cut"
        );
        assert_eq!(at(10_000), millisecond(30_000), "the first moment after it");
        assert_eq!(at(29_999), millisecond(49_999), "the edit ends at the trim");

        // And the recording it read is untouched, which is the promise.
        assert_eq!(std::fs::read(&source).expect("readable"), before);
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// Edits that keep nothing must refuse rather than write an empty file that
    /// later looks like a recording of silence.
    #[test]
    fn edits_that_keep_nothing_are_refused() {
        let directory = std::env::temp_dir().join("localog-apply-nothing");
        std::fs::create_dir_all(&directory).expect("working directory");
        let source = directory.join("recording.wav");
        write_pcm16_mono_16k(&source, &vec![0u8; 32_000]).expect("a recording");
        let edits = crate::edits::Edits {
            removed: vec![crate::edits::Span {
                from_ms: 0,
                to_ms: 1_000,
            }],
            ..Default::default()
        };
        assert!(apply_edits(&source, 1_000, &edits, &directory.join("out.wav")).is_err());
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// The arithmetic is only right for the working format, so anything else is
    /// refused rather than read as noise the diariser would cluster anyway.
    #[test]
    fn audio_that_is_not_the_working_format_is_refused() {
        let directory = std::env::temp_dir().join("localog-condense-format");
        std::fs::create_dir_all(&directory).expect("working directory");
        let path = directory.join("stereo.wav");
        // The same header, with two channels at 44.1 kHz.
        let mut header: Vec<u8> = Vec::new();
        header.extend_from_slice(b"RIFF");
        header.extend_from_slice(&36u32.to_le_bytes());
        header.extend_from_slice(b"WAVEfmt ");
        header.extend_from_slice(&16u32.to_le_bytes());
        header.extend_from_slice(&1u16.to_le_bytes());
        header.extend_from_slice(&2u16.to_le_bytes());
        header.extend_from_slice(&44_100u32.to_le_bytes());
        header.extend_from_slice(&176_400u32.to_le_bytes());
        header.extend_from_slice(&4u16.to_le_bytes());
        header.extend_from_slice(&16u16.to_le_bytes());
        header.extend_from_slice(b"data");
        header.extend_from_slice(&0u32.to_le_bytes());
        std::fs::write(&path, &header).expect("a stereo file");

        let error = pcm16_mono_16k_data(&path).expect_err("refusal");
        assert!(error.contains("44100"), "{error}");
        let _ = std::fs::remove_dir_all(&directory);
    }

    use super::*;

    #[test]
    fn parses_audio_probe_facts() {
        let probe = parse_probe(r#"{"format":{"duration":"12.5","format_name":"wav"},"streams":[{"codec_type":"audio","codec_name":"pcm_s16le","sample_rate":"16000","channels":1}]}"#).unwrap();
        assert_eq!(probe.streams[0].codec_name.as_deref(), Some("pcm_s16le"));
        assert_eq!(probe.format.unwrap().duration.as_deref(), Some("12.5"));
    }

    #[test]
    fn rejects_invalid_probe_json() {
        assert!(parse_probe("not json").is_err());
    }

    #[test]
    fn accepts_documented_json_path_and_extensionless_runtime_variant() {
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path().join("transcript");
        assert!(json_output_path(&base).is_none());
        std::fs::write(base.with_extension("json"), "{}").unwrap();
        assert_eq!(json_output_path(&base), Some(base.with_extension("json")));
        std::fs::remove_file(base.with_extension("json")).unwrap();
        std::fs::write(&base, "{}").unwrap();
        assert_eq!(json_output_path(&base), Some(base));
    }

    #[test]
    fn parses_real_whisper_progress_lines() {
        // Verbatim whisper.cpp v1.9.2 shapes, including variable leading whitespace.
        assert_eq!(
            parse_whisper_progress("whisper_print_progress_callback: progress =  43%"),
            Some(43)
        );
        assert_eq!(
            parse_whisper_progress("whisper_print_progress_callback: progress = 100%"),
            Some(100)
        );
        assert_eq!(parse_whisper_progress("progress = 5%"), Some(5));
    }

    #[test]
    fn vocabulary_prompt_keeps_the_most_specific_terms_within_the_limit() {
        let terms: Vec<String> = ["NORVEK", "Mustermann", "Beispielhuber"]
            .iter()
            .map(|value| value.to_string())
            .chain((0..200).map(|index| format!("Fuellbegriff{index:03}")))
            .collect();
        let prompt = vocabulary_prompt(&terms).unwrap();
        assert!(prompt.len() <= VOCABULARY_PROMPT_LIMIT);
        // The terms supplied first are the ones that survive.
        assert!(prompt.starts_with("NORVEK, Mustermann, Beispielhuber"));
        assert!(!prompt.contains("Fuellbegriff199"));
    }

    #[test]
    fn vocabulary_prompt_skips_blanks_and_repeats() {
        let terms = ["NORVEK", "  ", "NORVEK", "MUSTER BAU"].map(str::to_string);
        assert_eq!(
            vocabulary_prompt(&terms).as_deref(),
            Some("NORVEK, MUSTER BAU")
        );
        assert_eq!(vocabulary_prompt(&[]), None);
        assert_eq!(vocabulary_prompt(&["".to_string()]), None);
    }

    #[test]
    fn whisper_command_only_carries_a_prompt_when_there_is_one() {
        let config = RuntimeConfig {
            executable: PathBuf::from("/bin/echo"),
            model: PathBuf::from("/tmp/model.bin"),
        };
        let without = whisper_command(
            &config,
            Path::new("/tmp/a.wav"),
            Path::new("/tmp/out"),
            "de",
            None,
        );
        let args: Vec<_> = without
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(!args.iter().any(|a| a == "--prompt"));

        let with = whisper_command(
            &config,
            Path::new("/tmp/a.wav"),
            Path::new("/tmp/out"),
            "de",
            Some("NORVEK, Mustermann"),
        );
        let args: Vec<_> = with
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.iter().any(|a| a == "--prompt"));
        assert!(args.iter().any(|a| a == "--carry-initial-prompt"));
        assert!(args.iter().any(|a| a == "NORVEK, Mustermann"));
    }

    #[test]
    fn diarisation_uses_the_machine_and_a_known_speaker_count() {
        let request = DiarisationRequest {
            executable: Path::new("/bin/echo"),
            segmentation_model: Path::new("/tmp/seg.onnx"),
            embedding_model: Path::new("/tmp/emb.onnx"),
            normalized: Path::new("/tmp/a.wav"),
            expected_speakers: Some(11),
        };
        let args: Vec<String> = diarisation_command(&request)
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.iter().any(|a| a == "--clustering.num-clusters=11"));
        // A supplied count replaces similarity-only clustering rather than joining it.
        assert!(
            !args
                .iter()
                .any(|a| a.starts_with("--clustering.cluster-threshold"))
        );
        assert!(args.iter().any(|a| a == "--segmentation.provider=coreml"));
        assert!(args.iter().any(|a| a == "--embedding.provider=coreml"));
        assert!(
            args.iter()
                .any(|a| a.starts_with("--segmentation.num-threads="))
        );
        assert!(
            args.iter()
                .any(|a| a.starts_with("--embedding.num-threads="))
        );
        // The audio is the final positional argument.
        assert_eq!(args.last().map(String::as_str), Some("/tmp/a.wav"));
    }

    /// Clustering by similarity alone was measured at eighty-six speakers on a
    /// meeting where eleven spoke, because a voice drifts across eighty minutes of
    /// videoconference. There is therefore no threshold fallback: without a count
    /// the pipeline declines to run the pass at all rather than spend half an hour
    /// producing an answer already known to be wrong.
    #[test]
    fn diarisation_never_guesses_how_many_people_spoke() {
        for count in [None, Some(0), Some(1)] {
            let request = DiarisationRequest {
                executable: Path::new("/bin/echo"),
                segmentation_model: Path::new("/tmp/seg.onnx"),
                embedding_model: Path::new("/tmp/emb.onnx"),
                normalized: Path::new("/tmp/a.wav"),
                expected_speakers: count,
            };
            let args: Vec<String> = diarisation_command(&request)
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            assert!(
                !args.iter().any(|a| a.starts_with("--clustering.")),
                "count {count:?} must not produce a clustering instruction"
            );
        }
    }

    #[test]
    fn a_known_speaker_count_is_given_to_the_clustering() {
        let request = DiarisationRequest {
            executable: Path::new("/bin/echo"),
            segmentation_model: Path::new("/tmp/seg.onnx"),
            embedding_model: Path::new("/tmp/emb.onnx"),
            normalized: Path::new("/tmp/a.wav"),
            expected_speakers: Some(11),
        };
        let args: Vec<String> = diarisation_command(&request)
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.iter().any(|a| a == "--clustering.num-clusters=11"));
    }

    #[test]
    fn worker_threads_leaves_headroom_and_is_never_zero() {
        let threads = worker_threads();
        assert!(
            (1..=8).contains(&threads),
            "unreasonable thread count: {threads}"
        );
    }

    #[test]
    fn ignores_non_progress_lines() {
        assert_eq!(parse_whisper_progress("main: processing 'audio.wav'"), None);
        assert_eq!(parse_whisper_progress("progress = notanumber%"), None);
        assert_eq!(parse_whisper_progress("progress = 50"), None);
    }
}

#[cfg(test)]
mod shipped_build {
    /// Every filter the application asks FFmpeg for, in one place.
    ///
    /// The build LocaLog ships is configured down to what is used, which is right and
    /// is also a contract nobody was checking. `amix` was left out of it — the list
    /// predates the recorder — so combining a recording's two tracks failed on any
    /// machine that did not happen to have a full FFmpeg installed. This machine did,
    /// and found it instead, which is why nothing noticed.
    const FILTERS_THE_CODE_USES: &[&str] = &["aresample", "aformat", "atrim", "volume", "amix"];

    #[test]
    fn the_ffmpeg_we_ship_has_every_filter_we_ask_it_for() {
        let Some(ffmpeg) = crate::runtime::discover_executable(crate::runtime::FFMPEG_NAMES) else {
            // Nothing to check against; the sidecar is built by a script, not by cargo.
            return;
        };
        let listed = std::process::Command::new(&ffmpeg)
            .args(["-hide_banner", "-filters"])
            .output()
            .expect("the FFmpeg we ship must run");
        let listed = String::from_utf8_lossy(&listed.stdout);
        let present: Vec<&str> = listed
            .lines()
            .filter_map(|line| line.split_whitespace().nth(1))
            .collect();

        let missing: Vec<&&str> = FILTERS_THE_CODE_USES
            .iter()
            .filter(|filter| !present.contains(*filter))
            .collect();
        assert!(
            missing.is_empty(),
            "{} is missing {missing:?}; add them to scripts/build-ffmpeg-sidecar.sh and rebuild",
            ffmpeg.display()
        );
    }
}
