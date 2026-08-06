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
        .args(["--output-json", "--output-file"])
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

/// Build the diarisation invocation. The runtime is separate from transcription
/// and its models are separate too, so the caller supplies all three paths.
pub(crate) fn diarisation_command(
    executable: &Path,
    segmentation_model: &Path,
    embedding_model: &Path,
    normalized: &Path,
) -> Command {
    let mut command = Command::new(executable);
    command
        .arg("--clustering.cluster-threshold=0.6")
        .arg(format!(
            "--segmentation.pyannote-model={}",
            segmentation_model.display()
        ))
        .arg(format!("--embedding.model={}", embedding_model.display()))
        .arg(normalized);
    command
}

pub(crate) fn expected_json_path(base: &Path) -> PathBuf {
    base.with_extension("json")
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
    fn ignores_non_progress_lines() {
        assert_eq!(parse_whisper_progress("main: processing 'audio.wav'"), None);
        assert_eq!(parse_whisper_progress("progress = notanumber%"), None);
        assert_eq!(parse_whisper_progress("progress = 50"), None);
    }
}
