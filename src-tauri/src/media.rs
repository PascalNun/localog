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
