//! Manual evaluation harness.
//!
//! Runs a real transcript through the protocol generator against a live provider
//! and writes the result outside the repository. It is ignored by default because
//! it needs a running Ollama, takes minutes, and depends on material that is never
//! committed.
//!
//! ```text
//! LOCALOG_EVAL_TRANSCRIPT=... LOCALOG_EVAL_MODEL=... LOCALOG_EVAL_OUT=... \
//!   cargo test --lib -- --ignored --nocapture generates_a_protocol
//! ```

use crate::provider::*;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

/// Ignored by default: needs a running Ollama and a real transcript, and writes
/// its result outside the repository. Run with
/// `LOCALOG_EVAL_TRANSCRIPT=... LOCALOG_EVAL_MODEL=... LOCALOG_EVAL_OUT=... \
///  cargo test --lib -- --ignored --nocapture generates_a_protocol`
#[test]
#[ignore]
fn generates_a_protocol_from_a_real_transcript() {
    let transcript_path = std::env::var("LOCALOG_EVAL_TRANSCRIPT").unwrap();
    let model_name = std::env::var("LOCALOG_EVAL_MODEL").unwrap();
    let out_path = std::env::var("LOCALOG_EVAL_OUT").unwrap();
    let language = std::env::var("LOCALOG_EVAL_LANGUAGE").unwrap_or_else(|_| "German".to_string());

    let raw = std::fs::read_to_string(&transcript_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let rows = value
        .get("transcription")
        .and_then(serde_json::Value::as_array)
        .expect("whisper transcription array");
    let transcript: Vec<GenerationSegment> = rows
        .iter()
        .filter_map(|row| {
            let text = row.get("text")?.as_str()?.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(GenerationSegment {
                start_ms: row
                    .get("offsets")
                    .and_then(|o| o.get("from"))
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                speaker: "Speaker 1".to_string(),
                text,
            })
        })
        .collect();
    assert!(!transcript.is_empty(), "transcript had no usable segments");

    let provider = OllamaProvider::loopback();
    let runtime_version = provider.version().expect("ollama must be running");
    let model = provider
        .installed_models()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.name == model_name)
        .expect("requested model is not installed");

    let request = GenerationRequest {
        model: model.name.clone(),
        model_digest: model.digest.clone(),
        runtime_version,
        meeting_language: language,
        style: formal_minutes_style(),
        vocabulary_revision: "eval".into(),
        vocabulary: Vec::new(),
        transcript,
        seed: 7,
        temperature_milli: 200,
        context_tokens: std::env::var("LOCALOG_EVAL_CONTEXT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8192),
        maximum_output_tokens: std::env::var("LOCALOG_EVAL_OUTPUT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(4096),
    };
    println!("segments={}", request.transcript.len());

    let started = Instant::now();
    let cancelled = AtomicBool::new(false);
    // A stage may now be built at the moment it is reported, so it is kept by value.
    let mut last_stage = String::new();
    let markdown = provider
        .generate(&request, &cancelled, &mut |percent, stage| {
            if stage != last_stage {
                println!("  {percent}% {stage} ({:?})", started.elapsed());
                last_stage = stage.to_string();
            }
            Ok(())
        })
        .expect("generation must succeed");
    println!(
        "generated {} chars in {:?}",
        markdown.len(),
        started.elapsed()
    );
    std::fs::write(&out_path, &markdown).unwrap();
}

/// Derived from a real professional protocol: topic-structured, explicit about
/// what was not decided, and ending in an owner-attributed action table.
fn formal_minutes_style() -> GenerationStyle {
    GenerationStyle {
        id: "style-formal".into(),
        revision: "formal-minutes@2".into(),
        density: crate::domain::ProtocolDensity::Comprehensive,
        instructions: vec![
            "Write the entire protocol in the meeting's language.".into(),
            "Organise the protocol by topic, not in the order things were discussed. Gather everything said about one subject into a single numbered section, even if it came up several times.".into(),
            "Begin with the participants, grouped by the organisation they belong to, and give a role only where it was stated.".into(),
            "Use numbered sections with descriptive headings, and sub-numbered subsections where a topic has distinct parts.".into(),
            "Write discussion as calm, factual prose. Use lists only for options, criteria, and open questions.".into(),
            "Reproduce every number, measurement, area, date, and proper name exactly as stated. Never round or approximate them.".into(),
            "Separate what was decided from what remains open. Where no decision was reached, say so plainly rather than implying one.".into(),
            "Mark uncertainty in the words the meeting used, such as an intention, an estimate, or a matter still to be confirmed.".into(),
            "End with a table of agreed next steps with two columns, the task and the responsible party, followed by a short section for dates and appointments.".into(),
            "Never invent a decision, an action, an owner, or a date. If the source does not say who is responsible, leave it unattributed.".into(),
            "Cover every topic that was discussed. A protocol that silently omits a topic is incomplete, even if what remains reads well.".into(),
            "The table of next steps must list every action that was agreed, not a selection of the clearest ones.".into(),
            "Write at whatever length the material requires. Do not compress the meeting into a summary: this is a record, and a reader who was absent must be able to follow what was discussed and what follows from it.".into(),
            "Never leave a placeholder such as [Datum] or [Details]. If something is not in the source, omit the line instead.".into(),
        ],
        required_sections: vec!["Teilnehmende".into()],
    }
}
