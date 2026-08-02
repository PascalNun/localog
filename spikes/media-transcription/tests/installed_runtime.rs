use localog_media_transcription_spike::{
    cancel_realtime_ffmpeg, discover_executable, inspect_model, normalize, probe, runtime_version,
    transcribe_openai_whisper,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

fn discover(name: &str, candidates: &[&str]) -> PathBuf {
    discover_executable(
        name,
        &candidates.iter().map(PathBuf::from).collect::<Vec<_>>(),
    )
    .unwrap()
}

fn model_path() -> PathBuf {
    if let Some(path) = std::env::var_os("LOCALOG_WHISPER_MODEL") {
        return PathBuf::from(path);
    }
    let home = std::env::var_os("HOME").expect("HOME for installed model discovery");
    PathBuf::from(home).join(".cache/whisper/medium.pt")
}

fn whisper_path() -> PathBuf {
    if let Some(path) = std::env::var_os("LOCALOG_WHISPER_BIN") {
        return PathBuf::from(path);
    }
    let candidates = std::env::var_os("HOME")
        .map(|home| vec![PathBuf::from(home).join(".local/bin/whisper")])
        .unwrap_or_default();
    discover_executable("whisper", &candidates).unwrap()
}

#[test]
#[ignore = "uses explicitly installed FFmpeg, Whisper, and model assets"]
fn installed_media_to_timestamped_transcript_contract() {
    let temporary = tempfile::tempdir().unwrap();
    let ffmpeg = discover(
        "ffmpeg",
        &["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"],
    );
    let ffprobe = discover(
        "ffprobe",
        &["/opt/homebrew/bin/ffprobe", "/usr/local/bin/ffprobe"],
    );
    let whisper = whisper_path();
    let ffmpeg_info = runtime_version(&ffmpeg, &["-version"]).unwrap();
    let model_started = Instant::now();
    let model = inspect_model(&model_path()).unwrap();
    let model_hash_elapsed = model_started.elapsed();

    let speech = temporary.path().join("synthetic-speech.aiff");
    let say_status = Command::new("/usr/bin/say")
        .args(["-o", speech.to_str().unwrap()])
        .arg("LocaLog keeps professional meeting records on this Mac. The project team will review the acoustic note before Thursday.")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(say_status.success());

    let source = temporary.path().join("synthetic review ; literal.mp4");
    create_synthetic_video(&ffmpeg, &speech, &source);
    let source_probe = probe(&ffprobe, &source).unwrap();
    assert!(source_probe.has_video);
    assert!(source_probe.duration_seconds > 3.0);

    let normalized = temporary.path().join("normalized.wav");
    let normalize_started = Instant::now();
    let normalization =
        normalize(&ffmpeg, &source, &normalized, source_probe.duration_seconds).unwrap();
    let normalize_elapsed = normalize_started.elapsed();
    assert_eq!(normalization.progress_events.last(), Some(&100));
    let normalized_probe = probe(&ffprobe, &normalized).unwrap();
    assert_eq!(normalized_probe.audio_codec, "pcm_s16le");
    assert_eq!(normalized_probe.sample_rate, 16_000);
    assert_eq!(normalized_probe.channels, 1);

    let transcription_started = Instant::now();
    let transcript = transcribe_openai_whisper(
        &whisper,
        &model,
        &normalized,
        &temporary.path().join("transcript-output"),
        "en",
    )
    .unwrap();
    let transcription_elapsed = transcription_started.elapsed();
    assert_eq!(transcript.language, "en");
    assert!(transcript.segment_count > 0);
    assert!(transcript.final_timestamp_seconds > 0.0);

    eprintln!("ffmpeg_version={}", ffmpeg_info.version_line);
    eprintln!("model_name={}", model.name);
    eprintln!("model_bytes={}", model.byte_count);
    eprintln!("model_sha256={}", model.sha256);
    eprintln!(
        "model_hash_ms={:.3}",
        model_hash_elapsed.as_secs_f64() * 1000.0
    );
    eprintln!("source_duration_s={:.3}", source_probe.duration_seconds);
    eprintln!(
        "normalize_ms={:.3}",
        normalize_elapsed.as_secs_f64() * 1000.0
    );
    eprintln!(
        "normalize_progress_events={}",
        normalization.progress_events.len()
    );
    eprintln!(
        "transcribe_ms={:.3}",
        transcription_elapsed.as_secs_f64() * 1000.0
    );
    eprintln!(
        "realtime_factor={:.3}",
        transcription_elapsed.as_secs_f64() / source_probe.duration_seconds
    );
    eprintln!("segments={}", transcript.segment_count);
    eprintln!("source_sha256={}", normalization.source_sha256);
    eprintln!("normalized_sha256={}", normalization.normalized_sha256);
    eprintln!("transcript_sha256={}", transcript.artifact_sha256);
}

#[test]
#[ignore = "uses explicitly installed FFmpeg"]
fn installed_ffmpeg_process_group_cancels() {
    let ffmpeg = discover(
        "ffmpeg",
        &["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"],
    );
    let elapsed = cancel_realtime_ffmpeg(&ffmpeg).unwrap();
    assert!(elapsed.as_millis() < 1000);
    eprintln!("ffmpeg_cancel_ms={:.3}", elapsed.as_secs_f64() * 1000.0);
}

fn create_synthetic_video(ffmpeg: &Path, speech: &Path, destination: &Path) {
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-nostdin", "-y", "-f", "lavfi", "-i"])
        .arg("color=c=0xf5f2ec:s=640x360:r=1")
        .arg("-i")
        .arg(speech)
        .args([
            "-shortest",
            "-c:v",
            "mpeg4",
            "-q:v",
            "8",
            "-c:a",
            "aac",
            "-b:a",
            "96k",
        ])
        .arg(destination)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
}
