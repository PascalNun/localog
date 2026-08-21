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

/// Does the model follow the style it was given?
///
/// Asked because three runs of `qwen3.5:4b` produced no table of next steps, which
/// the formal style demands explicitly and twice, and one of them produced no
/// headings at all. That is a worse failure than any misattributed sentence, and it
/// has one cheap question underneath it: is the model too small, or is the prompt
/// wrong? A larger model producing the table answers "use a better model"; every
/// model failing the same way answers "the instructions do not work".
///
/// Counts structure rather than reading the prose, because the instructions being
/// tested are structural: numbered sections with headings, and a closing table with
/// two columns.
///
/// ```text
/// LOCALOG_ADHERENCE_TRANSCRIPT=<transcript json> \
/// LOCALOG_ADHERENCE_OUT=<a directory outside the repository> \
/// LOCALOG_ADHERENCE_MODELS=qwen3.5:4b,granite4.1:8b,gemma4:12b \
///   cargo test --lib -- --ignored --nocapture does_the_model_follow_the_style
/// ```
#[test]
#[ignore = "requires a real transcript and a running Ollama"]
fn does_the_model_follow_the_style() {
    let transcript_path = std::env::var("LOCALOG_ADHERENCE_TRANSCRIPT").expect("a transcript");
    let out = std::path::PathBuf::from(std::env::var("LOCALOG_ADHERENCE_OUT").expect("a folder"));
    let language = std::env::var("LOCALOG_EVAL_LANGUAGE").unwrap_or("German".into());
    let wanted: Vec<String> = std::env::var("LOCALOG_ADHERENCE_MODELS")
        .unwrap_or("qwen3.5:4b".into())
        .split(',')
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();
    std::fs::create_dir_all(&out).expect("an output folder");

    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&transcript_path).expect("readable"))
            .expect("json");
    let segments: Vec<crate::domain::TranscriptSegment> =
        serde_json::from_value(value["segments"].clone()).expect("segments");
    let transcript: Vec<GenerationSegment> = segments
        .iter()
        .map(|segment| GenerationSegment {
            start_ms: segment.start_ms,
            speaker: segment.speaker.clone(),
            text: segment.text.clone(),
        })
        .collect();
    let stated = crate::facts::quantities(&segments);

    let provider = OllamaProvider::loopback();
    let runtime_version = provider.version().expect("ollama must be running");
    let installed = provider.installed_models().unwrap();

    println!(
        "{:>16} {:>8} {:>9} {:>7} {:>11} {:>8} {:>13}",
        "model", "seconds", "headings", "tables", "table rows", "bullets", "figures kept"
    );
    let mut summary = String::from(
        "| model | seconds | characters | headings | tables | figures kept |\\n\
         | --- | ---: | ---: | ---: | ---: | ---: |\\n",
    );
    for name in &wanted {
        let Some(model) = installed.iter().find(|candidate| candidate.name == *name) else {
            println!("{name:>16}  not installed, skipped");
            continue;
        };
        let request = GenerationRequest {
            model: model.name.clone(),
            model_digest: model.digest.clone(),
            runtime_version: runtime_version.clone(),
            meeting_language: language.clone(),
            style: formal_minutes_style(),
            vocabulary_revision: "adherence".into(),
            vocabulary: Vec::new(),
            transcript: transcript.clone(),
            // Variable, because a fixed seed makes a repeat reproduce the same run
            // rather than test it. A comparison between models is only worth acting
            // on if it survives the sampling.
            seed: std::env::var("LOCALOG_ADHERENCE_SEED")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(7),
            temperature_milli: 200,
            context_tokens: std::env::var("LOCALOG_EVAL_CONTEXT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(40_960),
            maximum_output_tokens: std::env::var("LOCALOG_EVAL_OUTPUT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(8_192),
        };
        let started = Instant::now();
        let markdown =
            match provider.generate(&request, &AtomicBool::new(false), &mut |_, _| Ok(())) {
                Ok(markdown) => markdown,
                Err(error) => {
                    println!("{name:>16}  failed: {error:?}");
                    continue;
                }
            };
        let seconds = started.elapsed().as_secs();

        let lines: Vec<&str> = markdown.lines().collect();
        let headings = lines
            .iter()
            .filter(|line| line.trim_start().starts_with('#'))
            .count();
        let rows: Vec<&&str> = lines
            .iter()
            .filter(|line| line.trim_start().starts_with('|'))
            .collect();
        let dividers = rows
            .iter()
            .filter(|line| {
                line.trim()
                    .chars()
                    .all(|c| matches!(c, '|' | '-' | ':' | ' '))
            })
            .count();
        let bullets = lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("- ") || trimmed.starts_with("* ")
            })
            .count();
        let kept = stated
            .iter()
            .filter(|fact| crate::facts::is_accounted_for(fact, &markdown))
            .count();
        let safe = name.replace([':', '/', '.'], "-");
        let seed = request.seed;
        std::fs::write(
            out.join(format!("protocol-{safe}-seed{seed}.md")),
            &markdown,
        )
        .expect("written");
        println!(
            "{name:>16} {seconds:>8} {headings:>9} {dividers:>7} {:>11} {bullets:>8} {:>13}",
            rows.len().saturating_sub(dividers * 2),
            format!("{kept}/{}", stated.len()),
        );
        summary.push_str(&format!(
            "| {name} | {seconds} | {} | {headings} | {dividers} | {kept}/{} |\\n",
            markdown.len(),
            stated.len()
        ));
    }
    std::fs::write(out.join("adherence.md"), &summary).expect("written");
    println!("\\ndrafts written to {}", out.display());
}

/// Does attributing speech to speakers make the protocol better?
///
/// The question underneath every hour spent on speaker separation, and it has never
/// been asked. If the answer is no, the whole capability is optional rather than
/// core and a great deal of work is not worth doing.
///
/// Three protocols from one meeting, one model, one style, differing only in the
/// speaker labels the generator is given:
///
/// - **none** — every segment is `Speaker 1`, which is what the product does today
///   when nobody asks for separation;
/// - **grouped** — the labels the embedding pass produces, read from the vectors
///   the sidecar wrote;
/// - **scattered** — the labels a diarisation run without a speaker count produced
///   on this same meeting: fifty-four of them. Included deliberately, because
///   "bad labels are worse than none" is a different and more useful finding than
///   "good labels help".
///
/// ```text
/// LOCALOG_ATTRIBUTION_TRANSCRIPT=<committed transcript json> \
/// LOCALOG_ATTRIBUTION_VECTORS=<vectors.bin from the embedding sidecar> \
/// LOCALOG_ATTRIBUTION_OUT=<a directory outside the repository> \
/// LOCALOG_EVAL_MODEL=qwen3.5:4b \
///   cargo test --lib -- --ignored --nocapture does_attribution_improve_the_protocol
/// ```
///
/// The drafts are written where they are asked for and never into the repository:
/// they are the contents of somebody's meeting.
#[test]
#[ignore = "requires a real transcript, its embeddings and a running Ollama"]
fn does_attribution_improve_the_protocol() {
    let transcript_path = std::env::var("LOCALOG_ATTRIBUTION_TRANSCRIPT").expect("a transcript");
    let vectors_path = std::env::var("LOCALOG_ATTRIBUTION_VECTORS").expect("embeddings");
    let out = std::path::PathBuf::from(std::env::var("LOCALOG_ATTRIBUTION_OUT").expect("a folder"));
    let model_name = std::env::var("LOCALOG_EVAL_MODEL").unwrap_or("qwen3.5:4b".into());
    let language = std::env::var("LOCALOG_EVAL_LANGUAGE").unwrap_or("German".into());
    std::fs::create_dir_all(&out).expect("an output folder");

    let document: crate::domain::TranscriptDocument =
        serde_json::from_str(&std::fs::read_to_string(&transcript_path).expect("readable"))
            .or_else(|_| {
                // A committed artifact carries the durable fields only.
                let value: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&transcript_path).unwrap())
                        .unwrap();
                serde_json::from_value::<Vec<crate::domain::TranscriptSegment>>(
                    value["segments"].clone(),
                )
                .map(|segments| crate::domain::TranscriptDocument {
                    schema_version: 1,
                    meeting_id: String::new(),
                    revision_id: String::new(),
                    language: language.clone(),
                    speaker_resolution: crate::domain::SpeakerResolution::Unknown,
                    segments,
                    base_revision_id: String::new(),
                    is_dirty: false,
                    save_state: String::new(),
                    saved_at_ms: 0,
                })
            })
            .expect("a transcript document");
    let segments = document.segments;

    // The grouping the embedding pass gives, at whatever count it estimates.
    let (owners, vectors) =
        crate::clustering::read_vectors(std::path::Path::new(&vectors_path)).expect("embeddings");
    let merged = crate::clustering::merge(&vectors);
    let estimated = merged.voices_above(crate::clustering::SAME_VOICE_FLOOR);
    let voices = merged.voices(estimated);
    let mut grouped: Vec<String> = vec!["Speaker 1".into(); segments.len()];
    for (segment, voice) in owners.iter().zip(voices) {
        if let Some(slot) = grouped.get_mut(*segment as usize) {
            *slot = format!("Speaker {}", voice + 1);
        }
    }
    println!("the embedding pass estimates {estimated} voices");

    let build = |speakers: &dyn Fn(usize) -> String| -> Vec<GenerationSegment> {
        segments
            .iter()
            .enumerate()
            .map(|(index, segment)| GenerationSegment {
                start_ms: segment.start_ms,
                speaker: speakers(index),
                text: segment.text.clone(),
            })
            .collect()
    };
    let runs: Vec<(&str, Vec<GenerationSegment>)> = vec![
        ("none", build(&|_| "Speaker 1".to_string())),
        ("grouped", build(&|index| grouped[index].clone())),
        ("scattered", build(&|index| segments[index].speaker.clone())),
    ];

    let provider = OllamaProvider::loopback();
    let runtime_version = provider.version().expect("ollama must be running");
    let model = provider
        .installed_models()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.name == model_name)
        .expect("the requested model is not installed");

    let stated = crate::facts::quantities(&segments);
    let spoken: usize = segments.iter().map(|segment| segment.text.len()).sum();
    println!(
        "{} segments, {} quantities stated\n",
        segments.len(),
        stated.len()
    );

    let mut summary = String::from(
        "| labels | distinct | seconds | characters | figures kept | invented | unowned tasks |\\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: |\\n",
    );
    for (name, transcript) in runs {
        let distinct = {
            let mut seen: Vec<&str> = Vec::new();
            for segment in &transcript {
                if !seen.contains(&segment.speaker.as_str()) {
                    seen.push(&segment.speaker);
                }
            }
            seen.len()
        };
        let request = GenerationRequest {
            model: model.name.clone(),
            model_digest: model.digest.clone(),
            runtime_version: runtime_version.clone(),
            meeting_language: language.clone(),
            style: formal_minutes_style(),
            vocabulary_revision: "attribution".into(),
            vocabulary: Vec::new(),
            transcript,
            // Fixed, so the only thing differing between runs is the labels.
            seed: 7,
            temperature_milli: 200,
            context_tokens: std::env::var("LOCALOG_EVAL_CONTEXT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(40_960),
            maximum_output_tokens: std::env::var("LOCALOG_EVAL_OUTPUT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(8_192),
        };
        let started = Instant::now();
        let markdown = provider
            .generate(&request, &AtomicBool::new(false), &mut |_, _| Ok(()))
            .unwrap_or_else(|error| panic!("generation with {name} labels failed: {error:?}"));
        let seconds = started.elapsed().as_secs();

        let kept = stated
            .iter()
            .filter(|fact| crate::facts::is_accounted_for(fact, &markdown))
            .count();
        let invented = crate::facts::invented(&segments, &markdown);
        let unowned = crate::facts::unowned_tasks(&markdown);
        std::fs::write(out.join(format!("protocol-{name}.md")), &markdown).expect("written");
        println!(
            "{name:>10}: {distinct:>3} labels, {seconds:>4}s, {:>6} chars, {kept}/{} figures, {} invented, {} unowned",
            markdown.len(),
            stated.len(),
            invented.len(),
            unowned.len(),
        );
        summary.push_str(&format!(
            "| {name} | {distinct} | {seconds} | {} | {kept}/{} | {} | {} |\\n",
            markdown.len(),
            stated.len(),
            invented.len(),
            unowned.len()
        ));
    }
    println!("\\n{spoken} characters spoken\\n\\n{summary}");
    std::fs::write(out.join("summary.md"), &summary).expect("written");
    println!("drafts written to {}", out.display());
}

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
/// The shipped formal-minutes instructions, so a sizing probe measures the real
/// overhead rather than a guess at it.
pub(crate) fn formal_minutes_instructions() -> Vec<String> {
    formal_minutes_style().instructions
}

fn formal_minutes_style() -> GenerationStyle {
    let mut style = GenerationStyle {
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
            // Density already says how long to write, and at two of its three
            // settings this contradicts it. Removable under an environment variable
            // so the question can be measured before prose is cut from the only
            // thing that produces protocols.
            "Write at whatever length the material requires. Do not compress the meeting into a summary: this is a record, and a reader who was absent must be able to follow what was discussed and what follows from it.".into(),
            "Never leave a placeholder such as [Datum] or [Details]. If something is not in the source, omit the line instead.".into(),
        ],
        expectations: vec![crate::domain::StructuralExpectation::ActionTable],
    };
    if std::env::var("LOCALOG_EVAL_DROP_LENGTH_INSTRUCTION").is_ok() {
        style
            .instructions
            .retain(|line| !line.starts_with("Write at whatever length"));
    }
    style
}

/// Run the transcript corrections over a real transcript and report what they change.
///
/// The claim this checks was first measured in a throwaway script, which is not the
/// same as measuring the code that would ship. Building it already found one thing the
/// script had hidden: German lower-cases the interior of a compound, so a stem
/// correction is two rules rather than one, and the script had listed both by hand
/// without anybody noticing.
///
///   LOCALOG_CORRECTIONS_TRANSCRIPT=… \
///   LOCALOG_CORRECTIONS="Klaster=Cluster,Hoai=HOAI" \
///     cargo test --lib -- --ignored --nocapture what_the_corrections_change
#[test]
#[ignore = "requires a real transcript"]
fn what_the_corrections_change() {
    let path = std::env::var("LOCALOG_CORRECTIONS_TRANSCRIPT").expect("a transcript");
    let pairs = std::env::var("LOCALOG_CORRECTIONS").expect("wrong=right pairs, comma separated");
    let corrections: Vec<crate::corrections::Correction> = pairs
        .split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(wrong, right)| crate::corrections::Correction {
            wrong: wrong.trim().to_string(),
            right: right.trim().to_string(),
        })
        .collect();

    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
    let mut segments: Vec<crate::domain::TranscriptSegment> =
        serde_json::from_value(value["segments"].clone()).expect("segments");

    let found = crate::corrections::preview(&segments, &corrections);
    println!("{} places would change:", found.len());
    for shown in found.iter().take(8) {
        println!(
            "  {:>7} ms  {}",
            shown.start_ms,
            shown.context.replace('\n', " ")
        );
    }
    if found.len() > 8 {
        println!("  … and {} more", found.len() - 8);
    }

    let counts = crate::corrections::apply(&mut segments, &corrections);
    println!("\n{:<24}occurrences", "correction");
    for (correction, count) in corrections.iter().zip(&counts) {
        println!(
            "{:<24}{count}",
            format!("{} -> {}", correction.wrong, correction.right)
        );
    }
    println!("\ntotal: {}", counts.iter().sum::<usize>());
    assert_eq!(
        found.len(),
        counts.iter().sum::<usize>(),
        "every place offered for review must be a place that changes"
    );
}

/// What the candidate extractor offers on a real transcript.
///
///   LOCALOG_CORRECTIONS_TRANSCRIPT=… \
///     cargo test --lib -- --ignored --nocapture what_the_extractor_offers
#[test]
#[ignore = "requires a real transcript"]
fn what_the_extractor_offers() {
    let path = std::env::var("LOCALOG_CORRECTIONS_TRANSCRIPT").expect("a transcript");
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
    let segments: Vec<crate::domain::TranscriptSegment> =
        serde_json::from_value(value["segments"].clone()).expect("segments");

    let flagged = segments.iter().filter(|s| s.needs_review).count();
    println!(
        "{} segments, {flagged} flagged as containing something uncertain",
        segments.len()
    );

    let candidates = crate::corrections::name_candidates(&segments);
    println!("\n{} offered:\n", candidates.len());
    for candidate in &candidates {
        println!(
            "  {:>3}x  {:<22} {}",
            candidate.occurrences, candidate.heard, candidate.context
        );
    }
}

/// What room a context actually leaves for the protocol itself.
///
/// Answers a question the failing 8,192 runs raised: whether they were unlucky or
/// whether the window cannot hold the notes and a whole protocol at once. Needs no
/// Ollama — it is the harness's own arithmetic, reported rather than estimated.
#[test]
#[ignore = "reports arithmetic rather than asserting behaviour"]
fn what_room_each_context_leaves_for_the_protocol() {
    // A German protocol of the reference meeting runs 11,000 to 18,000 characters,
    // measured across this project's drafts. Three characters to the token.
    const CHARS_PER_TOKEN: usize = 3;
    println!(
        "{:>8} {:>10} {:>12} {:>14} {:>10}",
        "context", "sections", "notes fit", "answer room", "verdict"
    );
    for context in [8_192u32, 16_384, 24_576, 32_768, 40_960] {
        let request = crate::provider::sizing_probe(context, 8_192);
        let (sections, notes_chars, answer_tokens) = request;
        let answer_chars = answer_tokens as usize * CHARS_PER_TOKEN;
        let verdict = if answer_chars < 11_000 {
            "too small"
        } else if answer_chars < 18_000 {
            "marginal"
        } else {
            "fits"
        };
        println!(
            "{context:>8} {sections:>10} {notes_chars:>12} {:>14} {verdict:>10}",
            format!("{answer_tokens} tok / {answer_chars} ch")
        );
    }
}

/// Write a protocol topic by topic, with each section given a share of the length.
///
/// The one previous attempt at this covered 23 of 24 figures and produced roughly
/// 74,000 characters — four times a human protocol — so what failed was proportion,
/// not comprehension. This supplies the missing part: each topic is told about how
/// many characters it is worth, from its share of what was said.
///
/// Nothing here ever holds the whole protocol, which is the point. The largest thing
/// in any request is one topic's passages, so the length of the meeting stops being
/// bounded by what fits in a single answer.
///
///   LOCALOG_ADHERENCE_TRANSCRIPT=… LOCALOG_ADHERENCE_OUT=… \
///     cargo test --lib -- --ignored --nocapture does_writing_by_topic_stay_in_proportion
#[test]
#[ignore = "requires a real transcript and a running Ollama"]
fn does_writing_by_topic_stay_in_proportion() {
    let transcript_path = std::env::var("LOCALOG_ADHERENCE_TRANSCRIPT").expect("a transcript");
    let out = std::path::PathBuf::from(std::env::var("LOCALOG_ADHERENCE_OUT").expect("a folder"));
    let name = std::env::var("LOCALOG_ADHERENCE_MODELS").unwrap_or("gemma4:12b".into());
    let seed: u64 = std::env::var("LOCALOG_ADHERENCE_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(7);
    std::fs::create_dir_all(&out).expect("an output folder");

    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&transcript_path).expect("readable"))
            .expect("json");
    let segments: Vec<crate::domain::TranscriptSegment> =
        serde_json::from_value(value["segments"].clone()).expect("segments");
    let stated = crate::facts::quantities(&segments);
    let transcript: Vec<GenerationSegment> = segments
        .iter()
        .map(|segment| GenerationSegment {
            start_ms: segment.start_ms,
            speaker: segment.speaker.clone(),
            text: segment.text.clone(),
        })
        .collect();

    let provider = OllamaProvider::loopback();
    let runtime_version = provider.version().expect("ollama must be running");
    let model = provider
        .installed_models()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.name == name)
        .expect("the model must be installed");

    let request = GenerationRequest {
        model: model.name.clone(),
        model_digest: model.digest.clone(),
        runtime_version,
        meeting_language: "German".into(),
        style: formal_minutes_style(),
        vocabulary_revision: "by-topic".into(),
        vocabulary: Vec::new(),
        transcript,
        seed,
        temperature_milli: 200,
        context_tokens: std::env::var("LOCALOG_EVAL_CONTEXT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(40_960),
        maximum_output_tokens: 8_192,
    };

    let started = Instant::now();
    let markdown =
        match provider.write_by_topic(&request, &AtomicBool::new(false), &mut |_, _| Ok(())) {
            Ok(markdown) => markdown,
            Err(error) => {
                println!("failed: {error:?}");
                return;
            }
        };
    let seconds = started.elapsed().as_secs();

    let path = out.join(format!("by-topic-{}-seed{seed}.md", name.replace(':', "-")));
    std::fs::write(&path, &markdown).expect("writable");

    let kept = stated
        .iter()
        .filter(|fact| crate::facts::is_accounted_for(fact, &markdown))
        .count();
    let headings = markdown
        .lines()
        .filter(|line| line.starts_with('#'))
        .count();
    let table_rows = markdown
        .lines()
        .filter(|line| line.trim_start().starts_with('|'))
        .count();
    println!(
        "\n{seconds} s, {} characters, {headings} headings, {table_rows} table lines, {kept} of {} figures",
        markdown.len(),
        stated.len()
    );
    println!("A human wrote about 18,000 characters for this meeting.");
    println!("The previous attempt at writing by topic produced about 74,000.");
    println!("written to {}", path.display());
}

/// Who the opening of a real meeting says is in it.
///
/// The names come back wrong on a first meeting, which is the point: somebody who
/// was there recognises each one instantly, and the wrong spelling is what a
/// correction has to match to find it in the transcript.
///
///   LOCALOG_ADHERENCE_TRANSCRIPT=… \
///     cargo test --lib -- --ignored --nocapture who_introduced_themselves
#[test]
#[ignore = "requires a real transcript and a running Ollama"]
fn who_introduced_themselves() {
    let path = std::env::var("LOCALOG_ADHERENCE_TRANSCRIPT").expect("a transcript");
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
    let segments: Vec<crate::domain::TranscriptSegment> =
        serde_json::from_value(value["segments"].clone()).expect("segments");
    let transcript: Vec<GenerationSegment> = segments
        .iter()
        .map(|segment| GenerationSegment {
            start_ms: segment.start_ms,
            speaker: segment.speaker.clone(),
            text: segment.text.clone(),
        })
        .collect();

    let provider = OllamaProvider::loopback();
    let runtime_version = provider.version().expect("ollama must be running");
    let name = std::env::var("LOCALOG_ADHERENCE_MODELS").unwrap_or("gemma4:12b".into());
    let model = provider
        .installed_models()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.name == name)
        .expect("the model must be installed");

    let request = GenerationRequest {
        model: model.name.clone(),
        model_digest: model.digest.clone(),
        runtime_version,
        meeting_language: "German".into(),
        style: formal_minutes_style(),
        vocabulary_revision: "introductions".into(),
        vocabulary: Vec::new(),
        transcript,
        seed: 7,
        temperature_milli: 200,
        context_tokens: 40_960,
        maximum_output_tokens: 8_192,
    };

    let started = Instant::now();
    let found = provider
        .find_introductions(&request, &AtomicBool::new(false), &mut |_, _| Ok(()))
        .expect("the opening must be readable");

    println!(
        "\n{} introductions in {} s:\n",
        found.len(),
        started.elapsed().as_secs()
    );
    for person in &found {
        println!("  {:<28} {}", person.heard, person.role);
    }
    println!("\nEvery spelling above is what the transcript says, not what is correct.");
}
