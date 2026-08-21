//! What the machine offers somebody who is editing.
//!
//! A rewritten passage, the names it heard people introduce themselves by, and
//! every other place a word it got wrong appears. None of it is applied on its
//! own: each of these reads the working transcript and hands back a proposal,
//! and it is the interface that decides whether to keep it.
//!
//! That is the whole reason this is a module of its own rather than part of the
//! editing it sits on top of. A local model alters a fact in roughly one rewrite
//! in eight, so nothing here may write without being asked twice — once to
//! produce the change and once to accept it.

use super::editing::{persist_transcript_working, working_transcript};
use crate::domain::WorkspaceSnapshot;
use crate::provider;
use crate::storage::{Result as StorageResult, StorageError, WorkspaceRepository};
use std::path::Path;

/// Who introduced themselves in a meeting's opening, as the transcript spells them.
///
/// A light path: this needs a model, a language and the first minutes of the
/// transcript, not the style, the vocabulary or any of the rest a protocol run
/// resolves. It is one small request, and it runs when somebody asks for it rather
/// than automatically, because it is model work and this application runs one heavy
/// task at a time.
/// Rewrite one passage of a protocol, as asked.
///
/// The passage travels alone: no transcript, no meeting, no vocabulary. The job is
/// to say the same thing differently, and everything else the model could see is
/// something it could add.
/// A rewritten passage, and what checking it found.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RefinedPassage {
    pub text: String,
    /// Figures that were in the passage and are not in what came back.
    ///
    /// Measured rather than trusted. On a real German passage the shipped small
    /// model altered a fact in three of twenty-four rewrites — "2. Obergeschoss"
    /// became "Obergeschoss (Etage II)" — and a protocol whose figures drift is
    /// worse than one nobody rewrote.
    pub missing_figures: Vec<String>,
    /// What a second pass thought the rewrite changed about the facts.
    ///
    /// Empty when it found nothing, and also empty when the installed model is too
    /// small to be worth asking — `checked` says which.
    pub noticed_changes: Vec<String>,
    /// Whether a model was asked at all.
    pub checked: bool,
}
pub(crate) fn refine_passage(
    root: &Path,
    meeting_id: &str,
    passage: &str,
    instruction: &str,
) -> StorageResult<RefinedPassage> {
    if passage.trim().is_empty() {
        return Err(StorageError::InvalidData("Select some text to change."));
    }
    // A whole protocol is not a passage. The limit is generous — several paragraphs
    // — and exists so that "rewrite" cannot quietly become "regenerate", which is a
    // different operation with a different cost and its own button.
    const LONGEST_PASSAGE: usize = 6_000;
    if passage.len() > LONGEST_PASSAGE {
        return Err(StorageError::InvalidData(
            "That is too much text to change at once. Select a section rather than the document.",
        ));
    }

    let repository = WorkspaceRepository::open(root)?;
    let language: String = repository
        .connection
        .query_row(
            "SELECT language FROM meetings WHERE id = ?1",
            [meeting_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "German".to_string());

    let selected = repository
        .read_setting("generation.ollamaModel")?
        .filter(|value| !value.is_empty());
    let status = provider::OllamaProvider::loopback().status(selected);
    if !status.server_reachable {
        return Err(StorageError::InvalidData(
            "Start your existing Ollama installation before changing a passage.",
        ));
    }
    let model = status.selected_model.ok_or(StorageError::InvalidData(
        "Choose an installed Ollama model in Settings → Protocol generation.",
    ))?;

    let request = provider::GenerationRequest {
        model,
        model_digest: status.selected_model_digest.unwrap_or_default(),
        runtime_version: status.runtime_version.unwrap_or_else(|| "unknown".into()),
        meeting_language: language,
        style: provider::GenerationStyle {
            id: "refine".into(),
            revision: "1".into(),
            density: crate::domain::ProtocolDensity::Concise,
            instructions: Vec::new(),
            expectations: Vec::new(),
        },
        vocabulary_revision: "refine".into(),
        vocabulary: Vec::new(),
        transcript: Vec::new(),
        seed: 7,
        // Low, but not zero: a rewrite asked for twice should be able to differ.
        temperature_milli: 300,
        // The passage is all that is sent, so the window can be small — which on an
        // eight-gigabyte machine is the difference between an answer and a wait.
        context_tokens: 8_192,
        maximum_output_tokens: 2_048,
    };

    let text = provider::OllamaProvider::loopback()
        .refine_passage(
            &request,
            passage,
            instruction,
            &std::sync::atomic::AtomicBool::new(false),
        )
        // The provider's own message is not passed on: a model's complaint can quote
        // the meeting back at whoever reads the error.
        .map_err(|_| StorageError::InvalidData("That passage could not be rewritten."))?;

    // The instruction tells the model to keep every figure. Whether it did is a
    // separate question, and one that can be answered rather than assumed.
    let mut remaining = crate::facts::numbers_in(&text);
    let mut missing_figures = Vec::new();
    for figure in crate::facts::numbers_in(passage) {
        match remaining.iter().position(|candidate| *candidate == figure) {
            Some(at) => {
                remaining.remove(at);
            }
            None => missing_figures.push(figure),
        }
    }

    // A second opinion, where the installed model is big enough for one to be worth
    // having. Measured: a 4.7B model objects to every rewrite, clean or not, which
    // is the same as objecting to none.
    let capable = provider::OllamaProvider::loopback()
        .installed_models()
        .ok()
        .and_then(|models| {
            models
                .into_iter()
                .find(|model| model.name == request.model)
                .map(|model| provider::can_check_a_rewrite(&model))
        })
        .unwrap_or(false);

    let noticed_changes = if capable {
        provider::OllamaProvider::loopback()
            .check_rewrite(
                &request,
                passage,
                &text,
                &std::sync::atomic::AtomicBool::new(false),
            )
            // A checking pass that fails is a hint that did not arrive, not a
            // rewrite that failed. The rewrite is still returned.
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(RefinedPassage {
        text,
        missing_figures,
        noticed_changes,
        checked: capable,
    })
}
pub(crate) fn find_introductions(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<Vec<provider::Introduction>> {
    let repository = WorkspaceRepository::open(root)?;
    let artifact = working_transcript(root, &repository, meeting_id)?.1;
    let language: String = repository
        .connection
        .query_row(
            "SELECT language FROM meetings WHERE id = ?1",
            [meeting_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "German".to_string());

    let selected = repository
        .read_setting("generation.ollamaModel")?
        .filter(|value| !value.is_empty());
    let status = provider::OllamaProvider::loopback().status(selected);
    if !status.server_reachable {
        return Err(StorageError::InvalidData(
            "Start your existing Ollama installation before reading the introductions.",
        ));
    }
    let model = status.selected_model.ok_or(StorageError::InvalidData(
        "Choose an installed Ollama model in Settings → Protocol generation.",
    ))?;
    let model_digest = status.selected_model_digest.unwrap_or_default();

    let request = provider::GenerationRequest {
        model,
        model_digest,
        runtime_version: status.runtime_version.unwrap_or_else(|| "unknown".into()),
        meeting_language: language,
        style: provider::GenerationStyle {
            id: "introductions".into(),
            revision: "1".into(),
            density: crate::domain::ProtocolDensity::Concise,
            instructions: Vec::new(),
            expectations: Vec::new(),
        },
        vocabulary_revision: "introductions".into(),
        vocabulary: Vec::new(),
        transcript: artifact
            .segments
            .iter()
            .map(provider::GenerationSegment::from)
            .collect(),
        seed: 7,
        temperature_milli: 200,
        context_tokens: 16_384,
        maximum_output_tokens: 2_048,
    };

    provider::OllamaProvider::loopback()
        .find_introductions(
            &request,
            &std::sync::atomic::AtomicBool::new(false),
            &mut |_, _| Ok(()),
        )
        // The provider's own message is not passed on: these are bounded,
        // content-free strings by convention, and a model's complaint can quote the
        // meeting back at whoever reads the error.
        .map_err(|_| StorageError::InvalidData("The meeting's opening could not be read."))
}
/// The words the transcriber was never sure of in a meeting's working transcript.
pub(crate) fn name_candidates(
    root: &Path,
    meeting_id: &str,
) -> StorageResult<Vec<crate::corrections::Candidate>> {
    let repository = WorkspaceRepository::open(root)?;
    let artifact = working_transcript(root, &repository, meeting_id)?.1;
    Ok(crate::corrections::name_candidates(&artifact.segments))
}
/// Every place a correction would apply. Nothing is changed.
pub(crate) fn preview_correction(
    root: &Path,
    meeting_id: &str,
    wrong: &str,
    right: &str,
) -> StorageResult<Vec<crate::corrections::Match>> {
    let repository = WorkspaceRepository::open(root)?;
    let artifact = working_transcript(root, &repository, meeting_id)?.1;
    let correction = crate::corrections::Correction {
        wrong: wrong.to_string(),
        right: right.to_string(),
    };
    Ok(crate::corrections::preview(
        &artifact.segments,
        &[correction],
    ))
}
/// Correct a spelling in the working transcript, and optionally remember it.
///
/// Two outcomes from one action: this transcript is repaired, and the project keeps
/// the spelling so the next meeting is transcribed correctly. The committed revision
/// this was edited from is untouched, which is where somebody goes if the correction
/// was wrong.
pub(crate) fn apply_correction(
    root: &Path,
    meeting_id: &str,
    wrong: &str,
    right: &str,
    kept_segment_ids: &[String],
    remember: bool,
) -> StorageResult<AppliedCorrectionResult> {
    let right = right.trim();
    if wrong.trim().is_empty() || right.is_empty() || right.chars().count() > 200 {
        return Err(StorageError::InvalidData("Enter a valid spelling."));
    }
    let mut repository = WorkspaceRepository::open(root)?;
    let (path, mut artifact) = working_transcript(root, &repository, meeting_id)?;

    let correction = crate::corrections::Correction {
        wrong: wrong.to_string(),
        right: right.to_string(),
    };
    let changed = if kept_segment_ids.is_empty() {
        crate::corrections::apply(&mut artifact.segments, &[correction])
            .into_iter()
            .sum::<usize>()
    } else {
        let kept: Vec<crate::corrections::Match> =
            crate::corrections::preview(&artifact.segments, &[correction])
                .into_iter()
                .filter(|found| kept_segment_ids.contains(&found.segment_id))
                .collect();
        crate::corrections::apply_kept(&mut artifact.segments, &kept)
    };
    persist_transcript_working(&repository, meeting_id, &path, &artifact)?;

    if remember {
        let project_id: String = repository.connection.query_row(
            "SELECT project_id FROM meetings WHERE id = ?1",
            [meeting_id],
            |row| row.get(0),
        )?;
        // A term that is already known is left as it is rather than duplicated.
        let known: bool = repository.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM vocabulary_entries WHERE term = ?1)",
            [right],
            |row| row.get(0),
        )?;
        if !known {
            repository.save_vocabulary_entry(crate::domain::VocabularyDraft {
                id: None,
                term: right.to_string(),
                category: "Person".into(),
                scope: "Project".into(),
                project_id: Some(project_id),
                enabled: true,
            })?;
        }
    }
    Ok(AppliedCorrectionResult {
        workspace: repository.workspace_snapshot()?,
        changed,
    })
}
/// What a correction actually did, as against what was asked of it.
///
/// The count matters because it can differ from the number of places somebody
/// approved: a match whose text has moved since the review is skipped rather than
/// applied to whatever now sits at that position. Telling them "corrected in five
/// places" when it changed none would be a lie the interface had no way to notice.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppliedCorrectionResult {
    pub workspace: WorkspaceSnapshot,
    pub changed: usize,
}

#[cfg(test)]
mod refine_against_the_real_model {
    /// The whole path, on this machine's own Ollama.
    ///
    /// Ignored by default because it needs a running runtime and a model, and takes
    /// tens of seconds. Run with the model to use:
    ///     LOCALOG_REFINE_MODEL=ministral-3:8b cargo test --lib refine_against -- --ignored --nocapture
    #[test]
    #[ignore = "requires a running Ollama and an installed model"]
    fn a_passage_is_rewritten_and_checked() {
        let model = std::env::var("LOCALOG_REFINE_MODEL")
            .expect("set LOCALOG_REFINE_MODEL to an installed model");
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();

        let mut repository = crate::storage::WorkspaceRepository::open(root).unwrap();
        repository
            .write_setting("generation.ollamaModel", &model)
            .unwrap();
        let project = repository
            .create_project(crate::domain::NewProjectInput {
                name: "Nordenstadt".to_string(),
                description: String::new(),
                default_language: "German".to_string(),
            })
            .unwrap();
        let placeholder = root.join("placeholder.wav");
        std::fs::write(&placeholder, b"placeholder").unwrap();
        let meeting = repository
            .create_meeting(crate::domain::NewMeetingInput {
                project_id: project.id,
                title: "Jour fixe".to_string(),
                occurred_at: "2026-08-20".to_string(),
                language: "German".to_string(),
                source_name: "placeholder.wav".to_string(),
                source_path: Some(placeholder.to_string_lossy().into_owned()),
                style_id: "style-formal".to_string(),
            })
            .unwrap();
        drop(repository);

        let passage = "Die Anpassungen im 2. Obergeschoss im Bereich der Lüftung wurden \
                       aufgrund von Änderungen seitens der Architekten vorgenommen. Die \
                       betroffene Fläche beträgt 148,5 m². Herr Planung hat zugesagt, die \
                       Kostenspanne bis zum 12. September 2026 zu nennen.";
        let refined = super::refine_passage(root, &meeting.id, passage, "Say this in fewer words.")
            .expect("the passage is rewritten");

        println!("--- rewrite ---\n{}", refined.text);
        println!("checked: {}", refined.checked);
        println!("missing figures: {:?}", refined.missing_figures);
        println!("noticed changes: {:?}", refined.noticed_changes);

        assert!(!refined.text.trim().is_empty(), "a rewrite comes back");
        assert!(
            refined.text != passage || refined.missing_figures.is_empty(),
            "an unchanged passage cannot have lost a figure"
        );
    }
}
