//! Whole-pipeline harness.
//!
//! Runs a real recording through the application's own job pipeline — import,
//! transcription with the project's vocabulary, speaker separation, and protocol
//! generation — using the same functions the Tauri commands call. Nothing here is
//! a reimplementation: if this passes, the engine behind the interface works.
//!
//! It is ignored by default because it needs a real recording, both runtimes, and
//! a running Ollama, and takes tens of minutes. The recording it reads never lives
//! in this repository.
//!
//! ```text
//! LOCALOG_E2E_AUDIO=/path/to/meeting.mp3 \
//! LOCALOG_E2E_WHISPER=/path/to/whisper-cli \
//! LOCALOG_E2E_DIARISER=/path/to/sherpa-onnx-offline-speaker-diarization \
//! LOCALOG_E2E_WHISPER_MODEL=/path/to/ggml-medium.bin \
//! LOCALOG_E2E_SEG_MODEL=/path/to/segmentation.onnx \
//! LOCALOG_E2E_EMB_MODEL=/path/to/embedding.onnx \
//! LOCALOG_E2E_SPEAKERS=11 LOCALOG_E2E_LANGUAGE=German \
//!   cargo test --lib -- --ignored --nocapture runs_the_whole_pipeline
//! ```

use crate::domain::{NewMeetingInput, NewProjectInput, VocabularyDraft};
use crate::imports;
use crate::processing::{self, ProcessingOutcome};
use crate::storage::WorkspaceRepository;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

fn required(key: &str) -> PathBuf {
    PathBuf::from(std::env::var(key).unwrap_or_else(|_| panic!("set {key}")))
}

/// Managed models are verified by exact size, so a local copy of the same file is
/// indistinguishable from a downloaded one. This keeps the harness from spending
/// ten minutes re-downloading what the machine already has.
fn stage_model(root: &Path, source: &Path, file_name: &str) {
    let directory = root.join("models");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::copy(source, directory.join(file_name))
        .unwrap_or_else(|error| panic!("staging {file_name}: {error}"));
}

fn drive(root: &Path, job_id: &str, label: &str) {
    let started = Instant::now();
    let outcome = processing::run_job(root, job_id, Arc::new(AtomicBool::new(false)), |_| {})
        .unwrap_or_else(|error| panic!("{label} could not run: {error:?}"));
    println!(
        "  {label}: {outcome:?} in {:.1}s",
        started.elapsed().as_secs_f64()
    );
    assert!(
        matches!(outcome, ProcessingOutcome::Completed),
        "{label} did not complete"
    );
}

#[test]
#[ignore = "requires a real recording, both runtimes and a running Ollama"]
fn runs_the_whole_pipeline_on_a_real_meeting() {
    let audio = required("LOCALOG_E2E_AUDIO");
    let language = std::env::var("LOCALOG_E2E_LANGUAGE").unwrap_or_else(|_| "German".to_string());
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();

    stage_model(
        root,
        &required("LOCALOG_E2E_WHISPER_MODEL"),
        "ggml-medium.bin",
    );
    stage_model(
        root,
        &required("LOCALOG_E2E_SEG_MODEL"),
        "speaker-segmentation.onnx",
    );
    stage_model(
        root,
        &required("LOCALOG_E2E_EMB_MODEL"),
        "speaker-embedding.onnx",
    );

    let mut repository = WorkspaceRepository::open(root).unwrap();
    repository
        .write_setting(
            "transcription.whisperExecutable",
            required("LOCALOG_E2E_WHISPER").to_str().unwrap(),
        )
        .unwrap();
    repository
        .write_setting("transcription.preset", "accurate")
        .unwrap();
    repository
        .write_setting(
            "diarisation.executable",
            required("LOCALOG_E2E_DIARISER").to_str().unwrap(),
        )
        .unwrap();
    if let Ok(count) = std::env::var("LOCALOG_E2E_SPEAKERS") {
        repository
            .write_setting("diarisation.expectedSpeakers", &count)
            .unwrap();
    }

    let project = repository
        .create_project(NewProjectInput {
            name: "Pipeline exercise".to_string(),
            description: String::new(),
            default_language: language.clone(),
        })
        .unwrap();

    // The vocabulary the transcriber cannot guess. Supplied through the same
    // library the interface writes to, so ordering and the prompt cap apply.
    for (term, category) in vocabulary_from_environment() {
        repository
            .save_vocabulary_entry(VocabularyDraft {
                id: None,
                term,
                category,
                scope: "Project".to_string(),
                project_id: Some(project.id.clone()),
                enabled: true,
            })
            .unwrap();
    }

    let meeting = repository
        .create_meeting(NewMeetingInput {
            project_id: project.id.clone(),
            title: "Pipeline exercise".to_string(),
            occurred_at: "2026-08-08".to_string(),
            language: language.clone(),
            source_name: audio
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("meeting.mp3")
                .to_string(),
            source_path: Some(audio.to_string_lossy().into_owned()),
            style_id: "style-formal".to_string(),
        })
        .unwrap();

    println!("stage 1 — import");
    drop(repository);
    // Import has its own runner; `run_job` only drives transcription and generation.
    let started = Instant::now();
    let outcome = imports::run_import(root, &meeting.id, Arc::new(AtomicBool::new(false)), |_| {})
        .expect("import could not run");
    println!(
        "  import: {outcome:?} in {:.1}s",
        started.elapsed().as_secs_f64()
    );

    println!("stage 2 — transcription, vocabulary and speakers");
    let (transcription, _) = processing::queue_transcription(root, &meeting.id, false).unwrap();
    drive(root, &transcription.id, "transcription");

    let repository = WorkspaceRepository::open(root).unwrap();
    let snapshot = repository.workspace_snapshot().unwrap();
    let transcript = snapshot
        .transcripts
        .get(&meeting.id)
        .expect("a transcript was produced");
    let speakers: std::collections::BTreeSet<&str> = transcript
        .segments
        .iter()
        .map(|segment| segment.speaker.as_str())
        .collect();
    let flagged = transcript
        .segments
        .iter()
        .filter(|segment| segment.needs_review)
        .count();
    println!(
        "  {} segments, {} speakers, {} flagged as unclear",
        transcript.segments.len(),
        speakers.len(),
        flagged
    );
    assert!(!transcript.segments.is_empty(), "the transcript is empty");
    // The vocabulary that shaped it is recorded against the job, so a transcript
    // can be explained after the fact.
    let recorded: Option<String> = repository
        .connection
        .query_row(
            "SELECT vocabulary_revision FROM jobs WHERE id = ?1",
            [&transcription.id],
            |row| row.get(0),
        )
        .unwrap();
    println!("  vocabulary recorded against the job: {recorded:?}");
    assert!(recorded.is_some(), "no vocabulary provenance was recorded");
    drop(repository);

    println!("stage 3 — protocol");
    let (generation, _) = processing::queue_generation(root, &meeting.id, false).unwrap();
    drive(root, &generation.id, "generation");

    let repository = WorkspaceRepository::open(root).unwrap();
    let snapshot = repository.workspace_snapshot().unwrap();
    let protocol = snapshot
        .protocols
        .get(&meeting.id)
        .expect("a protocol was produced");
    println!("  protocol: {} characters", protocol.markdown.len());
    assert!(
        protocol.markdown.len() > 2_000,
        "the protocol is implausibly short"
    );

    if let Ok(out) = std::env::var("LOCALOG_E2E_OUT") {
        std::fs::write(&out, &protocol.markdown).unwrap();
        println!("  written to {out}");
    }
    println!(
        "speakers found: {}",
        speakers.into_iter().collect::<Vec<_>>().join(", ")
    );
}

/// Terms are supplied by the operator rather than committed, because a real
/// project's vocabulary is the names of real people and firms.
fn vocabulary_from_environment() -> Vec<(String, String)> {
    let Ok(raw) = std::env::var("LOCALOG_E2E_VOCABULARY") else {
        return Vec::new();
    };
    raw.split(';')
        .filter_map(|entry| {
            let (term, category) = entry.split_once('=')?;
            let term = term.trim();
            (!term.is_empty()).then(|| (term.to_string(), category.trim().to_string()))
        })
        .collect()
}
