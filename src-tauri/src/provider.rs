//! The first production-shaped local protocol provider.
//!
//! This module talks to an already running, user-managed Ollama instance. It
//! deliberately does not start the server, pull models, or expose a provider
//! plugin surface. The rest of the application depends on the small typed
//! boundary below, which keeps a later provider decision local to this file.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const DEFAULT_PORT: u16 = 11_434;
const MAX_RESPONSE_BYTES: usize = 128 * 1024;
const MAX_PROMPT_BYTES: usize = 4 * 1024 * 1024;

pub type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Debug)]
pub enum ProviderError {
    Unavailable(String),
    InvalidResponse(String),
    ModelMissing(String),
    ModelChanged,
    RuntimeChanged,
    Cancelled,
    ResponseTooLarge,
    IncompleteResponse,
}

impl Display for ProviderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(message) => write!(formatter, "{message}"),
            Self::InvalidResponse(message) => write!(formatter, "{message}"),
            Self::ModelMissing(model) => write!(formatter, "model is not installed: {model}"),
            Self::ModelChanged => write!(formatter, "the selected Ollama model changed"),
            Self::RuntimeChanged => write!(formatter, "the Ollama runtime changed"),
            Self::Cancelled => write!(formatter, "local generation was cancelled"),
            Self::ResponseTooLarge => write!(formatter, "the local model response was too large"),
            Self::IncompleteResponse => {
                write!(formatter, "the local model response was incomplete")
            }
        }
    }
}

impl std::error::Error for ProviderError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDescriptor {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub digest: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatus {
    pub endpoint: String,
    pub server_reachable: bool,
    pub runtime_version: Option<String>,
    pub models: Vec<ModelDescriptor>,
    pub selected_model: Option<String>,
    pub selected_model_digest: Option<String>,
    pub selected_model_ready: bool,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GenerationStyle {
    pub id: String,
    pub revision: String,
    pub instructions: Vec<String>,
    pub required_sections: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GenerationSegment {
    pub start_ms: u64,
    pub speaker: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GenerationRequest {
    pub model: String,
    pub model_digest: String,
    pub runtime_version: String,
    pub meeting_language: String,
    pub style: GenerationStyle,
    pub vocabulary_revision: String,
    pub vocabulary: Vec<String>,
    pub transcript: Vec<GenerationSegment>,
    pub seed: u64,
    pub temperature_milli: u16,
    pub context_tokens: u32,
    pub maximum_output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<ModelDescriptor>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
    /// Ollama reports "length" when the answer was cut off at the token cap, which
    /// leaves schema-constrained JSON unparseable.
    #[serde(default)]
    done_reason: String,
}

#[derive(Debug, Deserialize)]
struct StructuredProtocol {
    protocol_markdown: String,
}

#[derive(Serialize)]
struct PromptPayload<'a> {
    meeting_language: &'a str,
    style_id: &'a str,
    style_revision: &'a str,
    instructions: &'a [String],
    required_sections: &'a [String],
    vocabulary_revision: &'a str,
    vocabulary: &'a [String],
    transcript: &'a [GenerationSegment],
}

/// A meeting is longer than any context window, so long transcripts are condensed
/// section by section before the protocol is written from those notes.
#[derive(Serialize)]
struct SectionPayload<'a> {
    meeting_language: &'a str,
    section_index: usize,
    section_count: usize,
    transcript: &'a [GenerationSegment],
}

#[derive(Serialize)]
struct MergePayload<'a> {
    meeting_language: &'a str,
    notes: &'a [String],
}

#[derive(Serialize)]
struct SynthesisPayload<'a> {
    meeting_language: &'a str,
    style_id: &'a str,
    style_revision: &'a str,
    instructions: &'a [String],
    required_sections: &'a [String],
    vocabulary_revision: &'a str,
    vocabulary: &'a [String],
    section_notes: &'a [String],
}

#[derive(Debug, Deserialize)]
struct StructuredNotes {
    notes_markdown: String,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    system: &'static str,
    prompt: &'a str,
    stream: bool,
    format: serde_json::Value,
    options: OllamaOptions,
    keep_alive: &'static str,
}

#[derive(Serialize)]
struct OllamaOptions {
    seed: u64,
    temperature: f64,
    num_ctx: u32,
    num_predict: u32,
}

pub struct OllamaProvider {
    base_url: String,
    /// Discovery must answer quickly or be treated as unavailable.
    agent: ureq::Agent,
    /// Generation legitimately takes minutes: a cold model load alone costs seconds,
    /// and a long meeting is written in several passes. Bound the parts that indicate
    /// a dead server, and let cancellation between streamed chunks provide the rest.
    generation_agent: ureq::Agent,
}

impl OllamaProvider {
    pub fn loopback() -> Self {
        Self::at_port(DEFAULT_PORT)
    }

    #[cfg(test)]
    pub(crate) fn at_port(port: u16) -> Self {
        Self::with_url(format!("http://127.0.0.1:{port}"))
    }

    #[cfg(not(test))]
    fn at_port(port: u16) -> Self {
        Self::with_url(format!("http://127.0.0.1:{port}"))
    }

    fn with_url(base_url: String) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .max_redirects(0)
            .build();
        let generation_config = ureq::Agent::config_builder()
            .timeout_global(None)
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_recv_response(Some(Duration::from_secs(120)))
            .max_redirects(0)
            .build();
        Self {
            base_url,
            generation_agent: generation_config.into(),
            agent: config.into(),
        }
    }

    pub fn status(&self, selected_model: Option<String>) -> OllamaStatus {
        let mut status = OllamaStatus {
            endpoint: self.base_url.clone(),
            selected_model,
            ..OllamaStatus::default()
        };
        let version = match self.version() {
            Ok(version) => version,
            Err(error) => {
                status.message = "Start your existing Ollama installation, then refresh.".into();
                status.message.push(' ');
                status.message.push_str(&truncate(&error.to_string(), 220));
                return status;
            }
        };
        let models = match self.installed_models() {
            Ok(models) => models,
            Err(error) => {
                status.message = truncate(&error.to_string(), 280);
                status.runtime_version = Some(version);
                return status;
            }
        };
        status.server_reachable = true;
        status.runtime_version = Some(version);
        status.models = models;
        status.selected_model_digest = status.selected_model.as_deref().and_then(|name| {
            status
                .models
                .iter()
                .find(|model| model.name == name)
                .map(|model| model.digest.clone())
        });
        status.selected_model_ready = status.selected_model_digest.is_some();
        status.message = if status.selected_model.is_none() {
            "Ollama is ready. Select an installed model to generate protocols.".into()
        } else if status.selected_model_ready {
            "The selected local model is ready.".into()
        } else {
            "The selected model is not installed. Choose another already installed model.".into()
        };
        status
    }

    pub fn version(&self) -> Result<String> {
        let mut response = self
            .agent
            .get(format!("{}/api/version", self.base_url))
            .call()
            .map_err(http_error)?;
        let value: VersionResponse = response
            .body_mut()
            .read_json()
            .map_err(|error| ProviderError::InvalidResponse(truncate(&error.to_string(), 280)))?;
        Ok(value.version)
    }

    pub fn installed_models(&self) -> Result<Vec<ModelDescriptor>> {
        let mut response = self
            .agent
            .get(format!("{}/api/tags", self.base_url))
            .call()
            .map_err(http_error)?;
        let value: TagsResponse = response
            .body_mut()
            .read_json()
            .map_err(|error| ProviderError::InvalidResponse(truncate(&error.to_string(), 280)))?;
        Ok(value.models)
    }

    pub fn generate(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<()>,
    ) -> Result<String> {
        let runtime_version = self.version()?;
        if runtime_version != request.runtime_version {
            return Err(ProviderError::RuntimeChanged);
        }
        let model = self
            .installed_models()?
            .into_iter()
            .find(|model| model.name == request.model)
            .ok_or_else(|| ProviderError::ModelMissing(request.model.clone()))?;
        if model.digest != request.model_digest {
            return Err(ProviderError::ModelChanged);
        }
        progress(18, "resolving_protocol_inputs")?;

        let sections = plan_sections(request);
        if sections.len() <= 1 {
            return self.generate_in_one_pass(request, cancelled, progress);
        }
        self.generate_from_sections(request, &sections, cancelled, progress)
    }

    /// Short meetings fit the window, so the protocol is written directly.
    fn generate_in_one_pass(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<()>,
    ) -> Result<String> {
        let payload = PromptPayload {
            meeting_language: &request.meeting_language,
            style_id: &request.style.id,
            style_revision: &request.style.revision,
            instructions: &request.style.instructions,
            required_sections: &request.style.required_sections,
            vocabulary_revision: &request.vocabulary_revision,
            vocabulary: &request.vocabulary,
            transcript: &request.transcript,
        };
        let generated = self.complete(
            request,
            Completion {
                system: PROTOCOL_SYSTEM,
                prompt: &encode_prompt(&payload)?,
                format: protocol_schema(),
                num_predict: request.maximum_output_tokens,
            },
            cancelled,
            &mut |_| progress(60, "generating_protocol"),
        )?;
        let structured: StructuredProtocol = serde_json::from_str(&generated)
            .map_err(|error| ProviderError::InvalidResponse(truncate(&error.to_string(), 280)))?;
        validate_markdown(
            &structured.protocol_markdown,
            &request.style.required_sections,
        )?;
        progress(78, "validating_protocol")?;
        Ok(structured.protocol_markdown)
    }

    /// A real meeting exceeds the window. Each section is condensed first, then the
    /// protocol is written from the collected notes. Nothing is silently dropped:
    /// every segment belongs to exactly one section.
    fn generate_from_sections(
        &self,
        request: &GenerationRequest,
        sections: &[std::ops::Range<usize>],
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<()>,
    ) -> Result<String> {
        let count = sections.len();
        let mut notes = Vec::with_capacity(count);
        for (index, range) in sections.iter().enumerate() {
            if cancelled.load(Ordering::Acquire) {
                return Err(ProviderError::Cancelled);
            }
            // Condensing runs from 20% to 60%; synthesis owns the rest.
            let start = 20 + (index as u64 * 40) / count as u64;
            progress(start, "condensing_transcript")?;
            let payload = SectionPayload {
                meeting_language: &request.meeting_language,
                section_index: index + 1,
                section_count: count,
                transcript: &request.transcript[range.clone()],
            };
            let prompt = encode_prompt(&payload)?;
            let num_predict = answer_budget(
                request.context_tokens,
                prompt.len(),
                request.maximum_output_tokens,
            );
            let generated = self.complete(
                request,
                Completion {
                    system: SECTION_SYSTEM,
                    prompt: &prompt,
                    format: notes_schema(),
                    num_predict,
                },
                cancelled,
                &mut |_| progress(start, "condensing_transcript"),
            )?;
            let structured: StructuredNotes =
                serde_json::from_str(&generated).map_err(|error| {
                    ProviderError::InvalidResponse(truncate(&error.to_string(), 280))
                })?;
            if structured.notes_markdown.trim().is_empty() {
                return Err(ProviderError::InvalidResponse(
                    "The model returned empty notes for a section of the meeting.".into(),
                ));
            }
            notes.push(structured.notes_markdown);
        }

        // The notes must fit the window too, or synthesis silently loses the meeting's
        // start. Fold them until they do; each round halves the count, so this ends.
        let budget = synthesis_budget(request);
        let mut rounds = 0;
        while notes.iter().map(String::len).sum::<usize>() > budget && notes.len() > 1 {
            if cancelled.load(Ordering::Acquire) {
                return Err(ProviderError::Cancelled);
            }
            rounds += 1;
            progress(60, "condensing_transcript")?;
            // Group by size, not in fixed pairs: two large notes plus room for the
            // answer can exceed the window, which is the failure this fold exists to
            // prevent. A note that is already too large on its own passes through.
            let room = budget.saturating_sub(budget / 4).max(1);
            let mut merged: Vec<String> = Vec::new();
            let mut group: Vec<String> = Vec::new();
            let mut group_len = 0;
            for note in notes.iter() {
                if !group.is_empty() && group_len + note.len() > room {
                    merged.push(self.merge_notes(request, &group, cancelled, progress)?);
                    group = Vec::new();
                    group_len = 0;
                }
                group_len += note.len();
                group.push(note.clone());
            }
            if !group.is_empty() {
                merged.push(self.merge_notes(request, &group, cancelled, progress)?);
            }
            if merged.len() == notes.len() {
                // Nothing could be combined; folding further would not help.
                break;
            }
            notes = merged;
            if rounds > 8 {
                break;
            }
        }

        progress(62, "generating_protocol")?;
        let payload = SynthesisPayload {
            meeting_language: &request.meeting_language,
            style_id: &request.style.id,
            style_revision: &request.style.revision,
            instructions: &request.style.instructions,
            required_sections: &request.style.required_sections,
            vocabulary_revision: &request.vocabulary_revision,
            vocabulary: &request.vocabulary,
            section_notes: &notes,
        };
        let prompt = encode_prompt(&payload)?;
        let num_predict = answer_budget(
            request.context_tokens,
            prompt.len(),
            request.maximum_output_tokens,
        );
        let generated = self.complete(
            request,
            Completion {
                system: SYNTHESIS_SYSTEM,
                prompt: &prompt,
                format: protocol_schema(),
                num_predict,
            },
            cancelled,
            &mut |_| progress(70, "generating_protocol"),
        )?;
        let structured: StructuredProtocol = serde_json::from_str(&generated)
            .map_err(|error| ProviderError::InvalidResponse(truncate(&error.to_string(), 280)))?;
        validate_markdown(
            &structured.protocol_markdown,
            &request.style.required_sections,
        )?;
        progress(78, "validating_protocol")?;
        Ok(structured.protocol_markdown)
    }

    /// Combine a group of consecutive notes into one, preserving their content.
    /// A single note is returned unchanged rather than sent through the model again.
    fn merge_notes(
        &self,
        request: &GenerationRequest,
        group: &[String],
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &'static str) -> Result<()>,
    ) -> Result<String> {
        if group.len() == 1 {
            return Ok(group[0].clone());
        }
        let payload = MergePayload {
            meeting_language: &request.meeting_language,
            notes: group,
        };
        let prompt = encode_prompt(&payload)?;
        let num_predict = answer_budget(
            request.context_tokens,
            prompt.len(),
            request.maximum_output_tokens,
        );
        let generated = self.complete(
            request,
            Completion {
                system: MERGE_SYSTEM,
                prompt: &prompt,
                format: notes_schema(),
                num_predict,
            },
            cancelled,
            &mut |_| progress(60, "condensing_transcript"),
        )?;
        let structured: StructuredNotes = serde_json::from_str(&generated)
            .map_err(|error| ProviderError::InvalidResponse(truncate(&error.to_string(), 280)))?;
        Ok(structured.notes_markdown)
    }

    /// One bounded, cancellable, streamed completion.
    fn complete(
        &self,
        request: &GenerationRequest,
        call: Completion<'_>,
        cancelled: &AtomicBool,
        tick: &mut dyn FnMut(()) -> Result<()>,
    ) -> Result<String> {
        let body = OllamaRequest {
            model: &request.model,
            system: call.system,
            prompt: call.prompt,
            stream: true,
            format: call.format,
            options: OllamaOptions {
                seed: request.seed,
                temperature: f64::from(request.temperature_milli) / 1000.0,
                num_ctx: request.context_tokens,
                num_predict: call.num_predict,
            },
            keep_alive: "2m",
        };
        let response = self
            .generation_agent
            .post(format!("{}/api/generate", self.base_url))
            .send_json(&body)
            .map_err(http_error)?;
        let reader = response.into_parts().1.into_reader();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let mut generated = String::new();
        let mut done = false;
        let mut last_progress = Instant::now() - Duration::from_millis(100);
        loop {
            if cancelled.load(Ordering::Acquire) {
                return Err(ProviderError::Cancelled);
            }
            line.clear();
            let read = reader
                .read_line(&mut line)
                .map_err(|error| ProviderError::Unavailable(truncate(&error.to_string(), 280)))?;
            if read == 0 {
                break;
            }
            if line.len() > MAX_RESPONSE_BYTES {
                return Err(ProviderError::ResponseTooLarge);
            }
            let chunk: StreamChunk = serde_json::from_str(line.trim()).map_err(|error| {
                ProviderError::InvalidResponse(truncate(&error.to_string(), 280))
            })?;
            generated.push_str(&chunk.response);
            if generated.len() > MAX_RESPONSE_BYTES {
                return Err(ProviderError::ResponseTooLarge);
            }
            if last_progress.elapsed() >= Duration::from_millis(100) {
                tick(())?;
                last_progress = Instant::now();
            }
            if chunk.done {
                if chunk.done_reason == "length" {
                    return Err(ProviderError::IncompleteResponse);
                }
                done = true;
                break;
            }
        }
        if !done {
            return Err(ProviderError::IncompleteResponse);
        }
        Ok(generated)
    }
}

/// One model call: what to ask, how to constrain it, and how much answer to allow.
struct Completion<'a> {
    system: &'static str,
    prompt: &'a str,
    format: serde_json::Value,
    num_predict: u32,
}

const PROTOCOL_SYSTEM: &str = "Create a reviewable professional meeting protocol using only the supplied transcript. Never invent decisions, actions, owners, or dates. State uncertainty explicitly. Follow the controlled style and return only schema-valid JSON.";

const SECTION_SYSTEM: &str = "Record one section of a meeting transcript as detailed factual notes in the meeting's language. Completeness matters more than brevity: keep every topic discussed, every decision, every agreed action and its owner, every open question, and every number, measurement, area, date and proper name exactly as stated. Write one bullet per distinct point rather than merging several into a summary sentence. Do not add anything that was not said, and do not write a protocol yet. Return only schema-valid JSON.";

const SYNTHESIS_SYSTEM: &str = "Write a reviewable professional meeting protocol from ordered notes taken across the whole meeting. The notes are the only source. Never invent decisions, actions, owners, or dates, and state uncertainty explicitly. Group related material by topic rather than by the order it was discussed. Follow the controlled style and return only schema-valid JSON.";

const MERGE_SYSTEM: &str = "Combine consecutive sets of meeting notes into one set, in the meeting's language. Keep every decision, action, owner, open question, number, measurement, date and proper name. Remove only exact repetition between the sets. Do not shorten anything that is stated once. Return only schema-valid JSON.";

fn encode_prompt<T: Serialize>(payload: &T) -> Result<String> {
    let bytes = serde_json::to_vec(payload)
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
    if bytes.len() > MAX_PROMPT_BYTES {
        return Err(ProviderError::ResponseTooLarge);
    }
    String::from_utf8(bytes).map_err(|error| ProviderError::InvalidResponse(error.to_string()))
}

/// Divide the transcript into contiguous section ranges that each fit the model's
/// window alongside the style, vocabulary and room for the answer. Returns a single
/// range when the whole transcript already fits.
fn plan_sections(request: &GenerationRequest) -> Vec<std::ops::Range<usize>> {
    // German tokenises to roughly three characters per token; the margin keeps a
    // long word or an unusual name from pushing a section over the edge.
    const CHARS_PER_TOKEN: usize = 3;
    const SAFETY_NUMERATOR: usize = 7;
    const SAFETY_DENOMINATOR: usize = 10;

    let total = request.transcript.len();
    if total == 0 {
        return Vec::new();
    }
    let window = request
        .context_tokens
        .saturating_sub(request.maximum_output_tokens) as usize;
    let budget_chars = window * CHARS_PER_TOKEN * SAFETY_NUMERATOR / SAFETY_DENOMINATOR;
    // Everything in the prompt that is not transcript still has to fit.
    let overhead: usize = request
        .style
        .instructions
        .iter()
        .chain(request.style.required_sections.iter())
        .chain(request.vocabulary.iter())
        .map(|value| value.len() + 8)
        .sum::<usize>()
        + 512;
    let available = budget_chars.saturating_sub(overhead);
    if available == 0 {
        // Degenerate configuration: one segment per section is the safest fallback.
        return (0..total).map(|index| index..index + 1).collect();
    }

    let transcript_chars: usize = request.transcript.iter().map(segment_chars).sum();
    if transcript_chars <= available {
        return std::iter::once(0..total).collect();
    }

    let mut sections = Vec::new();
    let mut start = 0;
    let mut used = 0;
    for (index, segment) in request.transcript.iter().enumerate() {
        let size = segment_chars(segment);
        if used > 0 && used + size > available {
            sections.push(start..index);
            start = index;
            used = 0;
        }
        used += size;
    }
    if start < total {
        sections.push(start..total);
    }
    sections
}

/// Characters of source material that fit alongside the style, the vocabulary and
/// room for the answer. Used both to divide the transcript and to decide whether the
/// collected notes still need folding.
fn synthesis_budget(request: &GenerationRequest) -> usize {
    const CHARS_PER_TOKEN: usize = 3;
    let window = request
        .context_tokens
        .saturating_sub(request.maximum_output_tokens) as usize;
    let budget_chars = window * CHARS_PER_TOKEN * 7 / 10;
    let overhead: usize = request
        .style
        .instructions
        .iter()
        .chain(request.style.required_sections.iter())
        .chain(request.vocabulary.iter())
        .map(|value| value.len() + 8)
        .sum::<usize>()
        + 512;
    budget_chars.saturating_sub(overhead)
}

/// How many tokens the model may write back, given what the prompt already occupies.
/// Asking for more than the window allows is what cuts an answer off mid-JSON.
fn answer_budget(context_tokens: u32, prompt_chars: usize, requested: u32) -> u32 {
    const CHARS_PER_TOKEN: usize = 3;
    const RESERVED_FOR_SYSTEM: u32 = 256;
    let prompt_tokens = (prompt_chars / CHARS_PER_TOKEN) as u32;
    let room = context_tokens
        .saturating_sub(prompt_tokens)
        .saturating_sub(RESERVED_FOR_SYSTEM);
    requested.min(room).max(256)
}

fn segment_chars(segment: &GenerationSegment) -> usize {
    // Speaker and timestamp are serialised alongside the text.
    segment.text.len() + segment.speaker.len() + 40
}

fn notes_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["notes_markdown"],
        "properties": { "notes_markdown": { "type": "string" } },
        "additionalProperties": false
    })
}

fn protocol_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["protocol_markdown"],
        "properties": { "protocol_markdown": { "type": "string" } },
        "additionalProperties": false
    })
}

fn validate_markdown(markdown: &str, required_sections: &[String]) -> Result<()> {
    if markdown.trim().is_empty() {
        return Err(ProviderError::InvalidResponse(
            "The model returned an empty protocol.".into(),
        ));
    }
    let normalized = markdown.to_ascii_lowercase();
    for section in required_sections {
        if !normalized.contains(&section.to_ascii_lowercase()) {
            return Err(ProviderError::InvalidResponse(format!(
                "The model output is missing the required section: {section}"
            )));
        }
    }
    Ok(())
}

fn http_error(error: ureq::Error) -> ProviderError {
    ProviderError::Unavailable(truncate(&error.to_string(), 280))
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

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
        let language =
            std::env::var("LOCALOG_EVAL_LANGUAGE").unwrap_or_else(|_| "German".to_string());

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
        let sections = plan_sections(&request);
        println!(
            "segments={} sections={}",
            request.transcript.len(),
            sections.len()
        );

        let started = Instant::now();
        let cancelled = AtomicBool::new(false);
        let mut last_stage = "";
        let markdown = provider
            .generate(&request, &cancelled, &mut |percent, stage| {
                if stage != last_stage {
                    println!("  {percent}% {stage} ({:?})", started.elapsed());
                    last_stage = stage;
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

    #[test]
    fn sections_cover_every_segment_exactly_once() {
        let request = synthetic_request(400, 220);
        let sections = plan_sections(&request);
        assert!(
            sections.len() > 1,
            "a long transcript must be divided, got {} section(s)",
            sections.len()
        );
        // The invariant that matters: contiguous, ordered, nothing dropped or repeated.
        let mut expected = 0;
        for range in &sections {
            assert_eq!(range.start, expected, "sections must be contiguous");
            assert!(range.end > range.start, "sections must not be empty");
            expected = range.end;
        }
        assert_eq!(
            expected,
            request.transcript.len(),
            "every segment is covered"
        );
    }

    #[test]
    fn a_short_transcript_stays_in_one_section() {
        let request = synthetic_request(5, 60);
        assert_eq!(plan_sections(&request), vec![0..5]);
    }

    #[test]
    fn an_empty_transcript_plans_no_sections() {
        let request = synthetic_request(0, 0);
        assert!(plan_sections(&request).is_empty());
    }

    #[test]
    fn each_section_stays_within_the_character_budget() {
        let request = synthetic_request(300, 200);
        let window = request
            .context_tokens
            .saturating_sub(request.maximum_output_tokens) as usize;
        let budget = window * 3 * 7 / 10;
        for range in plan_sections(&request) {
            let size: usize = request.transcript[range.clone()]
                .iter()
                .map(segment_chars)
                .sum();
            // A single oversized segment cannot be split, so only multi-segment
            // sections are required to stay inside the budget.
            if range.end - range.start > 1 {
                assert!(
                    size <= budget,
                    "section {range:?} of {size} chars exceeds {budget}"
                );
            }
        }
    }

    fn synthetic_request(segments: usize, words_each: usize) -> GenerationRequest {
        GenerationRequest {
            model: "test".into(),
            model_digest: "digest".into(),
            runtime_version: "0".into(),
            meeting_language: "German".into(),
            style: GenerationStyle {
                id: "style-formal".into(),
                revision: "1".into(),
                instructions: vec!["Write a calm, factual professional protocol.".into()],
                required_sections: vec!["Zusammenfassung".into()],
            },
            vocabulary_revision: "1".into(),
            vocabulary: Vec::new(),
            transcript: (0..segments)
                .map(|index| GenerationSegment {
                    start_ms: index as u64 * 1000,
                    speaker: "Speaker 1".into(),
                    text: "Wort ".repeat(words_each),
                })
                .collect(),
            seed: 1,
            temperature_milli: 100,
            context_tokens: 8192,
            maximum_output_tokens: 2048,
        }
    }

    #[test]
    fn validates_required_sections_without_prompt_leakage() {
        let sections = vec!["Summary".into(), "Actions".into()];
        assert!(validate_markdown("# Summary\n\n# Actions\n", &sections).is_ok());
        assert!(validate_markdown("# Summary\n", &sections).is_err());
    }

    #[test]
    fn rejects_empty_protocols() {
        assert!(validate_markdown("  ", &[]).is_err());
    }

    #[test]
    fn discovers_exact_model_from_a_loopback_provider() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for response in [
                r#"{"version":"0.30.10"}"#,
                r#"{"models":[{"name":"qwen2.5:7b","size":123,"digest":"sha256:test"}]}"#,
            ] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 2048];
                let _ = stream.read(&mut request);
                let body = response.as_bytes();
                write_http_response(&mut stream, body);
            }
        });
        let status = OllamaProvider::at_port(port).status(Some("qwen2.5:7b".into()));
        handle.join().unwrap();
        assert!(status.server_reachable);
        assert!(status.selected_model_ready);
        assert_eq!(status.selected_model_digest.as_deref(), Some("sha256:test"));
    }

    fn write_http_response(stream: &mut std::net::TcpStream, body: &[u8]) {
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(body).unwrap();
    }
}
