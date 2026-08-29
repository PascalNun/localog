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
//! LOCALOG_E2E_SPEAKERS=auto LOCALOG_E2E_LANGUAGE=German \
//!   cargo test --lib -- --ignored --nocapture runs_the_whole_pipeline
//! ```

use crate::domain::{NewMeetingInput, NewProjectInput, VocabularyDraft};
use crate::imports;
use crate::processing::{self, ProcessingOutcome};
use crate::storage::WorkspaceRepository;
use std::cell::RefCell;
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

/// Run a job and say what each of its stages cost.
///
/// Timed from out here rather than from inside the pipeline: the job reports its
/// stage on every progress tick and the snapshot carries it, so the breakdown
/// needs nothing added to the code being measured — and it measures exactly what
/// the interface is told, rather than a second account of it.
///
/// A total on its own does not say where the minutes went. Normalising audio,
/// loading a model and transcribing are three very different costs to reduce, and
/// three that respond to entirely different changes.
fn drive(root: &Path, job_id: &str, label: &str) -> Vec<(String, f64)> {
    let started = Instant::now();
    // Each stage with the moment it began. A stage may carry a live detail after a
    // colon; the code before it is the stage.
    let marks: RefCell<Vec<(String, f64)>> = RefCell::new(Vec::new());
    let outcome = processing::run_job(root, job_id, Arc::new(AtomicBool::new(false)), |snapshot| {
        let Some(job) = snapshot.jobs.iter().find(|job| job.id == job_id) else {
            return;
        };
        let stage = job.stage.split(':').next().unwrap_or_default().to_string();
        let mut marks = marks.borrow_mut();
        if marks.last().map(|(seen, _)| seen != &stage).unwrap_or(true) {
            marks.push((stage, started.elapsed().as_secs_f64()));
        }
    })
    .unwrap_or_else(|error| panic!("{label} could not run: {error:?}"));

    let total = started.elapsed().as_secs_f64();
    println!("  {label}: {outcome:?} in {total:.1}s");

    let marks = marks.into_inner();
    let mut spent: Vec<(String, f64)> = Vec::new();
    for (index, (stage, at)) in marks.iter().enumerate() {
        let until = marks.get(index + 1).map(|(_, at)| *at).unwrap_or(total);
        spent.push((stage.clone(), until - at));
    }
    for (stage, seconds) in &spent {
        // Below a tenth of a second a stage is a transition, not a cost.
        if *seconds >= 0.1 {
            println!(
                "      {stage:<28} {seconds:>7.1}s  {:>4.0}%",
                seconds / total * 100.0
            );
        }
    }

    assert!(
        matches!(outcome, ProcessingOutcome::Completed),
        "{label} did not complete"
    );
    spent
}

#[test]
#[ignore = "requires a real recording, both runtimes and a running Ollama"]
fn runs_the_whole_pipeline_on_a_real_meeting() {
    let audio = required("LOCALOG_E2E_AUDIO");
    let language = std::env::var("LOCALOG_E2E_LANGUAGE").unwrap_or_else(|_| "German".to_string());
    // A number separates into that many; `auto` separates and works it out;
    // unset leaves the speakers alone, which is what the application does when
    // nobody asks.
    let speakers = match std::env::var("LOCALOG_E2E_SPEAKERS").ok().as_deref() {
        None | Some("") => processing::Speakers::Together,
        Some("auto") => processing::Speakers::Separate,
        Some(value) => match value.parse::<u32>() {
            Ok(count) => processing::Speakers::SeparateInto(count),
            Err(_) => panic!("LOCALOG_E2E_SPEAKERS must be a number or `auto`."),
        },
    };
    // A run costs half an hour, so the workspace can be kept for inspection
    // afterwards rather than vanishing with the temporary directory.
    let kept = std::env::var("LOCALOG_E2E_ROOT").ok().map(PathBuf::from);
    let temporary = tempfile::tempdir().unwrap();
    let root: &Path = match kept.as_deref() {
        Some(path) => {
            std::fs::create_dir_all(path).unwrap();
            path
        }
        None => temporary.path(),
    };
    println!("workspace: {}", root.display());

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
    // Generation needs a model that Ollama actually has installed; the provider
    // rejects an unknown one before any work starts.
    repository
        .write_setting(
            "generation.ollamaModel",
            &std::env::var("LOCALOG_E2E_MODEL").unwrap_or_else(|_| "qwen3.5:4b".to_string()),
        )
        .unwrap();
    let project = repository
        .create_project(NewProjectInput {
            names: Vec::new(),
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
    let (transcription, _) =
        processing::queue_transcription_with_expected(root, &meeting.id, false, speakers).unwrap();
    let transcription_spent = drive(root, &transcription.id, "transcription");

    let repository = WorkspaceRepository::open(root).unwrap();
    // What transcription costs is not seconds but seconds per second of meeting:
    // a number that can be compared between machines, models and recordings, which
    // a wall-clock reading of one run cannot.
    let audio_ms: Option<i64> = repository
        .connection
        .query_row(
            "SELECT nm.duration_ms FROM normalized_media nm
             JOIN recordings r ON r.id = nm.recording_id
             WHERE r.meeting_id = ?1",
            [&meeting.id],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    if let Some(audio_ms) = audio_ms {
        let audio_seconds = audio_ms as f64 / 1000.0;
        let spent: f64 = transcription_spent.iter().map(|(_, seconds)| seconds).sum();
        println!(
            "  {audio_seconds:.0}s of audio in {spent:.0}s — {:.2}x real time",
            spent / audio_seconds.max(1.0)
        );
        // The two stages worth reducing, named separately: preparing the audio is
        // ffmpeg's cost and transcribing is the model's, and they move for entirely
        // different reasons.
        for stage in [
            "normalizing_audio",
            "transcribing_audio",
            "separating_speakers",
        ] {
            if let Some((_, seconds)) = transcription_spent.iter().find(|(name, _)| name == stage) {
                println!(
                    "      {stage:<28} {:.2}x real time",
                    seconds / audio_seconds.max(1.0)
                );
            }
        }
    }
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
    let (generation, _) = processing::queue_generation(
        root,
        &meeting.id,
        false,
        &crate::provider::DocumentNotes::english_for_harnesses(),
    )
    .unwrap();
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

/// Generate a protocol from a workspace a previous run left behind.
///
/// The full pipeline costs half an hour, almost all of it transcription and
/// speaker separation. When only generation is in question, repeating those is
/// waste, so this picks up the finished workspace instead.
///
/// ```text
/// LOCALOG_REAL_RUNTIMES=1 LOCALOG_E2E_ROOT=/path/to/workspace \
/// LOCALOG_E2E_MODEL=qwen3.5:4b LOCALOG_E2E_OUT=/path/to/protocol.md \
///   cargo test --lib -- --ignored --nocapture generates_from_an_existing_workspace
/// ```
#[test]
#[ignore = "requires a workspace left by the whole-pipeline harness"]
fn generates_from_an_existing_workspace() {
    let root = required("LOCALOG_E2E_ROOT");
    let repository = WorkspaceRepository::open(&root).unwrap();
    let meeting_id: String = repository
        .connection
        .query_row(
            "SELECT id FROM meetings WHERE lifecycle IN ('transcript_ready', 'protocol_draft')
             ORDER BY created_at_ms DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("the workspace holds no transcribed meeting");
    if let Ok(model) = std::env::var("LOCALOG_E2E_MODEL") {
        repository
            .write_setting("generation.ollamaModel", &model)
            .unwrap();
    }
    drop(repository);

    let (generation, _) = processing::queue_generation(
        &root,
        &meeting_id,
        false,
        &crate::provider::DocumentNotes::english_for_harnesses(),
    )
    .unwrap();
    drive(&root, &generation.id, "generation");

    let repository = WorkspaceRepository::open(&root).unwrap();
    let snapshot = repository.workspace_snapshot().unwrap();
    let protocol = snapshot
        .protocols
        .get(&meeting_id)
        .expect("a protocol was produced");
    println!("  protocol: {} characters", protocol.markdown.len());
    if let Ok(out) = std::env::var("LOCALOG_E2E_OUT") {
        std::fs::write(&out, &protocol.markdown).unwrap();
        println!("  written to {out}");
    }
    assert!(protocol.markdown.len() > 2_000, "the protocol is too short");
}

/// Run only the topic pass over a real transcript and print what it found.
///
/// The topic pass decides everything written after it, so it is inspected on its
/// own before anything is built on top of it. Nothing else runs: no protocol is
/// written and no workspace is touched.
///
/// ```text
/// LOCALOG_REAL_RUNTIMES=1 LOCALOG_E2E_ROOT=/path/to/workspace \
/// LOCALOG_E2E_MODEL=qwen3.5:4b \
///   cargo test --lib -- --ignored --nocapture finds_the_topics_of_a_real_meeting
/// ```
#[test]
#[ignore = "requires a transcribed workspace and a running Ollama"]
fn finds_the_topics_of_a_real_meeting() {
    use crate::provider::{GenerationRequest, GenerationSegment, GenerationStyle, OllamaProvider};

    let root = required("LOCALOG_E2E_ROOT");
    let repository = WorkspaceRepository::open(&root).unwrap();
    let snapshot = repository.workspace_snapshot().unwrap();
    let transcript = snapshot
        .transcripts
        .values()
        .max_by_key(|document| document.segments.len())
        .expect("the workspace holds no transcript");
    let segments: Vec<GenerationSegment> = transcript
        .segments
        .iter()
        .map(GenerationSegment::from)
        .collect();

    let provider = OllamaProvider::loopback();
    let model_name =
        std::env::var("LOCALOG_E2E_MODEL").unwrap_or_else(|_| "qwen3.5:4b".to_string());
    let model = provider
        .installed_models()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.name == model_name)
        .expect("the requested model is not installed");
    let request = GenerationRequest {
        document_notes: crate::provider::DocumentNotes::english_for_harnesses(),
        model: model.name.clone(),
        model_digest: model.digest.clone(),
        runtime_version: provider.version().expect("ollama must be running"),
        meeting_language: "German".to_string(),
        style: GenerationStyle {
            id: "style-formal".into(),
            revision: "1".into(),
            density: crate::domain::ProtocolDensity::Comprehensive,
            instructions: Vec::new(),
            expectations: Vec::new(),
        },
        vocabulary_revision: "topics".into(),
        vocabulary: Vec::new(),
        transcript: segments,
        seed: 42,
        temperature_milli: 200,
        context_tokens: 8_192,
        maximum_output_tokens: 1_024,
    };

    let started = Instant::now();
    let (topics, unclaimed) = provider
        .find_topics(&request, &AtomicBool::new(false), &mut |_, stage| {
            if stage.starts_with("join") {
                println!("  {stage}");
            }
            Ok(())
        })
        .expect("the topic pass failed");
    let listing = topics
        .iter()
        .map(|topic| format!("{:>3} segments  {}", topic.segments.len(), topic.title))
        .collect::<Vec<_>>()
        .join("\n");
    if let Ok(path) = std::env::var("LOCALOG_E2E_OUT") {
        std::fs::write(&path, &listing).unwrap();
    }
    println!("{listing}");
    // What no subject claimed is the interesting part: crosstalk is fine to lose,
    // but a real discussion sitting here is a subject the pass failed to find.
    let by_length = |indices: &[usize]| -> (usize, usize) {
        let lengths: Vec<usize> = indices
            .iter()
            .map(|index| request.transcript[*index].text.trim().len())
            .collect();
        let short = lengths.iter().filter(|length| **length < 60).count();
        (short, lengths.iter().sum::<usize>() / lengths.len().max(1))
    };
    let (short, mean) = by_length(&unclaimed);
    println!(
        "\nunclaimed: {} segments, {short} of them under 60 characters, {mean} characters on average",
        unclaimed.len()
    );
    println!("longest unclaimed passages:");
    let mut longest = unclaimed.clone();
    longest.sort_by_key(|index| std::cmp::Reverse(request.transcript[*index].text.trim().len()));
    for index in longest.iter().take(6) {
        let text = request.transcript[*index].text.trim();
        println!("  [{index}] {}", &text[..text.len().min(110)]);
    }
    let covered: usize = topics.iter().map(|topic| topic.segments.len()).sum();
    println!(
        "\n{} segments -> {} subjects in {:.1}s\n{} segments claimed by no subject, {covered} placements",
        request.transcript.len(),
        topics.len(),
        started.elapsed().as_secs_f64(),
        unclaimed.len()
    );
    assert!(!topics.is_empty(), "no topics were found at all");
}

/// Exercise the correction commands against a real workspace on disk.
///
/// The panel was built against the fake bridge and the Rust behind it against unit
/// tests, which between them prove that the types line up and the arithmetic is
/// right. Neither proves that the commands read the transcript the application
/// actually stores, write a revision it can load again, or leave the workspace in a
/// state it can still open.
///
/// Non-destructive: it corrects a word to itself, so the transcript is rewritten with
/// identical content and nothing a person typed is changed. What is under test is the
/// path, not the substitution.
///
///   LOCALOG_E2E_ROOT=~/Library/Application\ Support/app.localog.desktop \
///     cargo test --lib -- --ignored --nocapture corrects_a_transcript_through_the_real_commands
#[test]
#[ignore = "requires a real workspace with a transcribed meeting"]
fn corrects_a_transcript_through_the_real_commands() {
    let root = required("LOCALOG_E2E_ROOT");
    let repository = WorkspaceRepository::open(&root).unwrap();
    let meeting_id: String = repository
        .connection
        .query_row(
            "SELECT meeting_id FROM transcript_working ORDER BY rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("the workspace holds no working transcript");
    drop(repository);

    let candidates = processing::name_candidates(&root, &meeting_id).unwrap();
    println!("{} candidates offered:", candidates.len());
    for candidate in &candidates {
        println!(
            "  {:>3}x  {:<24} {}",
            candidate.occurrences, candidate.heard, candidate.context
        );
    }
    let Some(first) = candidates.first() else {
        println!("nothing to correct in this workspace; the path is untested");
        return;
    };

    let matches = processing::preview_correction(&root, &meeting_id, &first.heard, &first.heard)
        .expect("preview must read the stored transcript");
    println!("\n{} places for {}", matches.len(), first.heard);
    assert!(
        !matches.is_empty(),
        "a candidate the extractor offered must be findable in the same transcript"
    );

    // Correcting the word to itself: every code path runs, no content changes.
    let result =
        processing::apply_correction(&root, &meeting_id, &first.heard, &first.heard, &[], false)
            .expect("applying must write a working transcript the workspace can load");

    assert!(
        result.workspace.transcripts.contains_key(&meeting_id),
        "the workspace must still hold this meeting's transcript afterwards"
    );
    assert_eq!(
        result.changed,
        matches.len(),
        "what it reports changing must be what it changed"
    );
    println!("changed {} places, as previewed", result.changed);
    let after = processing::name_candidates(&root, &meeting_id)
        .expect("the rewritten transcript must be readable again");
    assert_eq!(
        after.len(),
        candidates.len(),
        "a correction to itself must change nothing about what is offered"
    );
    println!("\nthe path holds: read, preview, apply, and read again");
}
