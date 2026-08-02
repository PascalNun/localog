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
    run_process(
        command,
        cancellation,
        ProcessLimits::with_max_output(512 * 1024),
    )
    .map_err(|error| error.to_string())?;
    progress(90);
    if !temporary.is_file() {
        return Err("The media normalizer did not produce an audio file.".into());
    }
    fs::rename(&temporary, destination).map_err(|error| error.to_string())?;
    progress(100);
    Ok(())
}

pub(crate) fn whisper_command(
    config: &RuntimeConfig,
    normalized: &Path,
    output_base: &Path,
    language: &str,
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
    command
}

pub(crate) fn expected_json_path(base: &Path) -> PathBuf {
    base.with_extension("json")
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
}
