//! Media facts and the regenerable mono/16 kHz PCM cache.

use crate::runtime::{ProcessLimits, RuntimeConfig, run_process};
use serde::Deserialize;
use std::fs;
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
    serde_json::from_str(json).map_err(|_| "The media probe returned invalid metadata.".into())
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
    let output = run_process(
        command,
        cancellation,
        ProcessLimits::with_max_output(512 * 1024),
    )
    .map_err(|error| error.to_string())?;
    parse_probe(&output.stdout)
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
        .ok_or("The normalized cache path is invalid.")?;
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
    if let Err(error) = run_process(
        command,
        cancellation,
        ProcessLimits::with_max_output(512 * 1024),
    ) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    progress(90);
    if !temporary.is_file() {
        return Err("The media normalizer did not produce an audio file.".into());
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
/// Rather than replaying the whole recording, this writes a few seconds of each
/// transcript segment end to end, separated by silence. The diariser then embeds
/// a fraction of the audio for the same clustering job, and separation stops
/// costing longer than transcription and generation together.
///
/// This uses the concat demuxer, which seeks to each sample, and not a filter
/// graph, which does not survive the scale. Trimming with `atrim` splits the
/// decoded stream once per sample and reads it through for each: at the reference
/// meeting's 753 segments that had not finished after ten minutes, which is worse
/// than the pass it exists to shorten. The demuxer builds the same file in under
/// four seconds.
///
/// Silence between samples comes from a small generated file listed between them,
/// so that the diariser's own segmentation breaks where we joined rather than
/// running two speakers together.
pub(crate) fn condense_for_diarisation(
    ffmpeg: &Path,
    normalized: &Path,
    samples: &[crate::diarisation::Sample],
    gap_ms: u64,
    working_directory: &Path,
    destination: &Path,
    cancellation: &AtomicBool,
) -> Result<(), String> {
    if samples.is_empty() {
        return Err("There is nothing for the speaker pass to listen to.".into());
    }
    let gap = working_directory.join("diarisation-gap.wav");
    write_silence(ffmpeg, gap_ms, &gap, cancellation)?;

    // Single quotes are the demuxer's escape, and a path containing one would
    // otherwise end the filename early and read some other file.
    let quoted = |path: &Path| {
        format!(
            "file '{}'\n",
            path.display().to_string().replace('\'', "'\\''")
        )
    };
    let source = quoted(normalized);
    let silence = quoted(&gap);
    let mut list = String::from("ffconcat version 1.0\n");
    for (index, sample) in samples.iter().enumerate() {
        if index > 0 {
            list.push_str(&silence);
        }
        list.push_str(&source);
        list.push_str(&format!(
            "inpoint {:.3}\noutpoint {:.3}\n",
            sample.source_start_ms as f64 / 1000.0,
            sample.source_end_ms as f64 / 1000.0,
        ));
    }
    let list_path = working_directory.join("diarisation-samples.txt");
    std::fs::write(&list_path, list)
        .map_err(|error| format!("The speaker pass could not prepare its audio: {error}"))?;

    let mut command = Command::new(ffmpeg);
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
        ])
        .arg(&list_path)
        .args(["-ac", "1", "-ar", "16000", "-c:a", "pcm_s16le", "-f", "wav"])
        .arg(destination);
    let result = run_process(
        command,
        cancellation,
        ProcessLimits::with_max_output(512 * 1024),
    )
    .map(|_| ())
    .map_err(|error| error.to_string());
    let _ = std::fs::remove_file(&list_path);
    let _ = std::fs::remove_file(&gap);
    result?;

    // Check the file against the plan rather than assuming it was built.
    //
    // The concat demuxer wants its entries to share stream parameters, and where
    // they do not it drops the odd one out and still exits successfully: handed a
    // recording that was not the 16 kHz mono working audio, every gap disappears
    // and the samples run together, which is exactly the thing the gaps exist to
    // prevent. That failure is silent, and it would show up as speakers merged
    // into one for reasons nobody could see. A length that does not match the plan
    // catches it, and any other cause of a short file with it.
    let planned_ms = samples
        .last()
        .map(|sample| sample.condensed_end_ms)
        .unwrap_or_default();
    let written = std::fs::metadata(destination)
        .map_err(|error| format!("The speaker pass could not read its own audio: {error}"))?
        .len();
    // 16 kHz mono at two bytes a frame, less a WAV header of well under a kilobyte.
    let actual_ms = written.saturating_sub(1_024) / 32;
    let shortfall = planned_ms.saturating_sub(actual_ms);
    if shortfall > (planned_ms / 20).max(1_000) {
        return Err(format!(
            "The condensed audio is {actual_ms} ms where {planned_ms} ms was planned."
        ));
    }
    Ok(())
}

/// The quiet laid between samples, so the joins read as pauses rather than as one
/// person interrupting another.
fn write_silence(
    ffmpeg: &Path,
    duration_ms: u64,
    destination: &Path,
    cancellation: &AtomicBool,
) -> Result<(), String> {
    let mut command = Command::new(ffmpeg);
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=16000:cl=mono",
            "-t",
        ])
        .arg(format!("{:.3}", duration_ms as f64 / 1000.0))
        .args(["-c:a", "pcm_s16le", "-f", "wav"])
        .arg(destination);
    run_process(
        command,
        cancellation,
        ProcessLimits::with_max_output(512 * 1024),
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
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
fn worker_threads() -> usize {
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

#[cfg(test)]
mod tests {
    /// Runs the real ffmpeg, because the parts that break here are the ones a
    /// hermetic test cannot reach: the concat list's escaping, the header, and
    /// whether the silence lands between the samples rather than around them.
    ///
    /// Set LOCALOG_CONDENSE_SOURCE to any audio file of at least a minute.
    #[test]
    #[ignore = "requires ffmpeg and a real recording"]
    fn condensation_produces_the_planned_length() {
        let Some(source) = std::env::var_os("LOCALOG_CONDENSE_SOURCE").map(PathBuf::from) else {
            panic!("Set LOCALOG_CONDENSE_SOURCE to an audio file.");
        };
        let ffmpeg =
            PathBuf::from(std::env::var("LOCALOG_FFMPEG").unwrap_or_else(|_| "ffmpeg".to_string()));
        let directory = std::env::temp_dir().join("localog-condense-test");
        std::fs::create_dir_all(&directory).expect("working directory");
        let destination = directory.join("condensed.wav");

        // Ten seconds spread across the first minute.
        let timings: Vec<(u64, u64)> = (0..5).map(|i| (i * 10_000, i * 10_000 + 4_000)).collect();
        let samples = crate::diarisation::plan_samples(
            &timings,
            crate::diarisation::SAMPLE_MS,
            crate::diarisation::GAP_MS,
            crate::diarisation::SHORTEST_MS,
        );
        assert_eq!(samples.len(), 5);

        let cancellation = AtomicBool::new(false);
        condense_for_diarisation(
            &ffmpeg,
            &source,
            &samples,
            crate::diarisation::GAP_MS,
            &directory,
            &destination,
            &cancellation,
        )
        .expect("condensation");

        // Five two-second samples with four gaps between them.
        let expected = samples.last().expect("a sample").condensed_end_ms;
        assert_eq!(expected, 5 * 2_000 + 4 * 300);
        let written = std::fs::metadata(&destination).expect("output").len();
        // 16 kHz mono 16-bit, so two bytes a frame, and a header under a kilobyte.
        // A little over the plan is the silence generator rounding up to whole
        // frames; short is the failure that matters, and the function rejects it.
        let seconds = (written.saturating_sub(1_024)) as f64 / 32_000.0;
        assert!(
            seconds > expected as f64 / 1000.0 - 0.3 && seconds < expected as f64 / 1000.0 + 0.5,
            "condensed to {seconds:.2}s, planned {:.2}s",
            expected as f64 / 1000.0
        );
        // The working files are the pass's own business and must not be left behind.
        assert!(!directory.join("diarisation-samples.txt").exists());
        assert!(!directory.join("diarisation-gap.wav").exists());
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
