#![cfg(unix)]

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, MediaError>;

#[derive(Debug)]
pub enum MediaError {
    Io(std::io::Error),
    Json(serde_json::Error),
    RuntimeMissing(String),
    RuntimeFailed {
        program: PathBuf,
        status: ExitStatus,
        diagnostic: String,
    },
    NoAudioStream,
    InvalidTranscript(String),
    TimedOut,
}

impl Display for MediaError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "media I/O error: {error}"),
            Self::Json(error) => write!(formatter, "invalid structured runtime output: {error}"),
            Self::RuntimeMissing(name) => {
                write!(formatter, "required local runtime is missing: {name}")
            }
            Self::RuntimeFailed {
                program,
                status,
                diagnostic,
            } => write!(
                formatter,
                "{} failed with {status}: {diagnostic}",
                program.display()
            ),
            Self::NoAudioStream => write!(formatter, "the imported source has no audio stream"),
            Self::InvalidTranscript(reason) => write!(formatter, "invalid transcript: {reason}"),
            Self::TimedOut => write!(formatter, "runtime did not stop within the timeout"),
        }
    }
}

impl std::error::Error for MediaError {}

impl From<std::io::Error> for MediaError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for MediaError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeInfo {
    pub executable: PathBuf,
    pub version_line: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInfo {
    pub path: PathBuf,
    pub name: String,
    pub byte_count: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaProbe {
    pub duration_seconds: f64,
    pub audio_codec: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub has_video: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizationReport {
    pub source_sha256: String,
    pub normalized_sha256: String,
    pub progress_events: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TranscriptSummary {
    pub language: String,
    pub segment_count: usize,
    pub final_timestamp_seconds: f64,
    pub artifact_sha256: String,
}

#[derive(Debug, Deserialize)]
struct ProbeOutput {
    streams: Vec<ProbeStream>,
    format: ProbeFormat,
}

#[derive(Debug, Deserialize)]
struct ProbeStream {
    codec_type: String,
    codec_name: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WhisperOutput {
    language: String,
    segments: Vec<WhisperSegment>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    start: f64,
    end: f64,
    text: String,
}

pub fn discover_executable(name: &str, explicit_candidates: &[PathBuf]) -> Result<PathBuf> {
    for candidate in explicit_candidates {
        if is_executable(candidate) {
            return Ok(candidate.clone());
        }
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&paths) {
            let candidate = directory.join(name);
            if is_executable(&candidate) {
                return Ok(candidate);
            }
        }
    }
    Err(MediaError::RuntimeMissing(name.to_string()))
}

pub fn runtime_version(executable: &Path, arguments: &[&str]) -> Result<RuntimeInfo> {
    let output = command(executable)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        return Err(runtime_failure(executable, output.status, &output.stderr));
    }
    let version_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    Ok(RuntimeInfo {
        executable: executable.to_path_buf(),
        version_line,
    })
}

pub fn inspect_model(path: &Path) -> Result<ModelInfo> {
    let metadata = fs::metadata(path)?;
    let name = path
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| MediaError::InvalidTranscript("model name is not valid UTF-8".into()))?
        .to_string();
    Ok(ModelInfo {
        path: path.to_path_buf(),
        name,
        byte_count: metadata.len(),
        sha256: sha256_file(path)?,
    })
}

pub fn probe(ffprobe: &Path, source: &Path) -> Result<MediaProbe> {
    let output = command(ffprobe)
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(source)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        return Err(runtime_failure(ffprobe, output.status, &output.stderr));
    }
    parse_probe(&output.stdout)
}

pub fn normalize(
    ffmpeg: &Path,
    source: &Path,
    destination: &Path,
    duration_seconds: f64,
) -> Result<NormalizationReport> {
    let source_checksum_before = sha256_file(source)?;
    let temporary = destination.with_extension("wav.part");
    let output = command(ffmpeg)
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
            "-progress",
            "pipe:1",
            "-nostats",
        ])
        .arg(&temporary)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let _ = fs::remove_file(&temporary);
        return Err(runtime_failure(ffmpeg, output.status, &output.stderr));
    }
    fs::rename(&temporary, destination)?;
    let source_checksum_after = sha256_file(source)?;
    if source_checksum_before != source_checksum_after {
        return Err(MediaError::InvalidTranscript(
            "normalization modified the imported source".into(),
        ));
    }
    Ok(NormalizationReport {
        source_sha256: source_checksum_before,
        normalized_sha256: sha256_file(destination)?,
        progress_events: parse_ffmpeg_progress(&output.stdout, duration_seconds),
    })
}

pub fn transcribe_openai_whisper(
    whisper: &Path,
    model: &ModelInfo,
    normalized_audio: &Path,
    output_directory: &Path,
    language: &str,
) -> Result<TranscriptSummary> {
    fs::create_dir_all(output_directory)?;
    let model_directory = model.path.parent().expect("model file has a parent");
    let output = command(whisper)
        .arg(normalized_audio)
        .args(["--model", &model.name, "--model_dir"])
        .arg(model_directory)
        .args(["--device", "cpu", "--output_dir"])
        .arg(output_directory)
        .args([
            "--output_format",
            "json",
            "--verbose",
            "False",
            "--task",
            "transcribe",
            "--language",
            language,
            "--fp16",
            "False",
            "--word_timestamps",
            "False",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        return Err(runtime_failure(whisper, output.status, &output.stderr));
    }
    let stem = normalized_audio
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| MediaError::InvalidTranscript("audio name is not valid UTF-8".into()))?;
    let artifact = output_directory.join(format!("{stem}.json"));
    let bytes = fs::read(&artifact)?;
    let parsed: WhisperOutput = serde_json::from_slice(&bytes)?;
    validate_transcript(&parsed)?;
    Ok(TranscriptSummary {
        language: parsed.language,
        segment_count: parsed.segments.len(),
        final_timestamp_seconds: parsed
            .segments
            .last()
            .map(|segment| segment.end)
            .unwrap_or(0.0),
        artifact_sha256: sha256_bytes(&bytes),
    })
}

pub fn cancel_realtime_ffmpeg(ffmpeg: &Path) -> Result<Duration> {
    use std::os::unix::process::CommandExt;

    let mut child = command(ffmpeg)
        .args([
            "-hide_banner",
            "-nostdin",
            "-re",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=30",
            "-f",
            "null",
            "-",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    thread::sleep(Duration::from_millis(150));
    let started = Instant::now();
    // SAFETY: the child was placed in a dedicated process group identified by its PID.
    unsafe { libc::kill(-(child.id() as i32), libc::SIGTERM) };
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait()?.is_none() {
        if Instant::now() >= deadline {
            // SAFETY: escalation targets only the dedicated FFmpeg process group.
            unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
            child.wait()?;
            return Err(MediaError::TimedOut);
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(started.elapsed())
}

fn parse_probe(bytes: &[u8]) -> Result<MediaProbe> {
    let parsed: ProbeOutput = serde_json::from_slice(bytes)?;
    let audio = parsed
        .streams
        .iter()
        .find(|stream| stream.codec_type == "audio")
        .ok_or(MediaError::NoAudioStream)?;
    let duration_seconds = parsed
        .format
        .duration
        .as_deref()
        .and_then(|duration| duration.parse().ok())
        .unwrap_or(0.0);
    Ok(MediaProbe {
        duration_seconds,
        audio_codec: audio.codec_name.clone().unwrap_or_else(|| "unknown".into()),
        sample_rate: audio
            .sample_rate
            .as_deref()
            .and_then(|rate| rate.parse().ok())
            .unwrap_or(0),
        channels: audio.channels.unwrap_or(0),
        has_video: parsed
            .streams
            .iter()
            .any(|stream| stream.codec_type == "video"),
    })
}

fn parse_ffmpeg_progress(bytes: &[u8], duration_seconds: f64) -> Vec<u8> {
    let mut events = Vec::new();
    for line in String::from_utf8_lossy(bytes).lines() {
        if let Some(value) = line.strip_prefix("out_time_us=")
            && let Ok(microseconds) = value.parse::<f64>()
            && duration_seconds > 0.0
        {
            let percent =
                ((microseconds / 1_000_000.0 / duration_seconds) * 100.0).clamp(0.0, 99.0) as u8;
            if events.last() != Some(&percent) {
                events.push(percent);
            }
        }
        if line == "progress=end" && events.last() != Some(&100) {
            events.push(100);
        }
    }
    events
}

fn validate_transcript(transcript: &WhisperOutput) -> Result<()> {
    if transcript.language.trim().is_empty() || transcript.segments.is_empty() {
        return Err(MediaError::InvalidTranscript(
            "language and timestamped segments are required".into(),
        ));
    }
    let mut previous_end = 0.0;
    for segment in &transcript.segments {
        if segment.text.trim().is_empty()
            || segment.start < 0.0
            || segment.end < segment.start
            || segment.start + 0.01 < previous_end
        {
            return Err(MediaError::InvalidTranscript(
                "segments must be non-empty, ordered, and time-bounded".into(),
            ));
        }
        previous_end = segment.end;
    }
    Ok(())
}

fn command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .env_clear()
        .env("LANG", "C")
        .env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin")
        .env("PYTHONUTF8", "1")
        .env("TOKENIZERS_PARALLELISM", "false");
    command
}

fn runtime_failure(program: &Path, status: ExitStatus, stderr: &[u8]) -> MediaError {
    MediaError::RuntimeFailed {
        program: program.to_path_buf(),
        status,
        diagnostic: truncate(&String::from_utf8_lossy(stderr), 800),
    }
}

fn truncate(value: &str, maximum_chars: usize) -> String {
    value.chars().take(maximum_chars).collect()
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    Ok(hex_bytes(&digest))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    hex_bytes(&digest)
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

pub fn os_arguments(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_probe_and_progress_without_runtime_text_assumptions() {
        let probe = parse_probe(
            br#"{"streams":[{"codec_type":"video","codec_name":"h264"},{"codec_type":"audio","codec_name":"aac","sample_rate":"48000","channels":2}],"format":{"duration":"4.25"}}"#,
        )
        .unwrap();
        assert_eq!(probe.audio_codec, "aac");
        assert_eq!(probe.sample_rate, 48_000);
        assert_eq!(probe.channels, 2);
        assert!(probe.has_video);
        assert_eq!(
            parse_ffmpeg_progress(
                b"out_time_us=1000000\nprogress=continue\nout_time_us=4000000\nprogress=end\n",
                4.0,
            ),
            vec![25, 99, 100]
        );
    }

    #[test]
    fn transcript_validation_rejects_empty_or_reversed_segments() {
        let empty = WhisperOutput {
            language: "en".into(),
            segments: Vec::new(),
        };
        assert!(validate_transcript(&empty).is_err());
        let reversed = WhisperOutput {
            language: "en".into(),
            segments: vec![WhisperSegment {
                start: 2.0,
                end: 1.0,
                text: "Synthetic".into(),
            }],
        };
        assert!(validate_transcript(&reversed).is_err());
    }

    #[test]
    fn missing_runtime_model_and_audio_are_actionable() {
        assert!(matches!(
            discover_executable("localog-runtime-that-does-not-exist", &[]),
            Err(MediaError::RuntimeMissing(_))
        ));
        assert!(matches!(
            inspect_model(Path::new("/definitely/missing/localog-model.pt")),
            Err(MediaError::Io(_))
        ));
        assert!(matches!(
            parse_probe(br#"{"streams":[{"codec_type":"video"}],"format":{"duration":"1"}}"#),
            Err(MediaError::NoAudioStream)
        ));
    }
}
