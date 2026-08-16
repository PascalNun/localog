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
/// Bounded so a runaway model cannot exhaust memory, but generous enough for a
/// full protocol plus any reasoning a thinking model emits along the way.
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
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
    /// The model accepted the work and then stopped sending anything.
    Stalled,
}

/// How long a model may send nothing before it is treated as stopped.
///
/// Generous, because a large model on a slow machine pauses: reading an
/// eighty-minute transcript is minutes of work before the first token, and
/// `gemma4:12b` was measured at fourteen minutes for a whole protocol. What this
/// catches is the case with no pauses at all — a model that has stopped answering
/// and will never resume, which had no limit before this and would have been waited
/// on until somebody gave up.
const SILENCE_BEFORE_GIVING_UP: Duration = Duration::from_secs(300);

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
            Self::Stalled => write!(
                formatter,
                "the local model stopped responding partway through"
            ),
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
    /// How much prose the style wants. Set as a directive alongside the style's
    /// own instructions, and used to size the answer budget: a style that asks
    /// for only decisions and next steps should not be handed room for eight
    /// thousand tokens, because that much room is an invitation to fill it.
    pub density: crate::domain::ProtocolDensity,
    pub id: String,
    pub revision: String,
    pub instructions: Vec<String>,
    /// What a style intends a protocol to contain. Kept as a description of the
    /// style, and deliberately **not** sent to the model.
    ///
    /// These are stored as literal English strings — "Summary", "Decisions" —
    /// while the protocol is written in the language of the meeting. Putting them
    /// in the prompt told a German-language model to produce four English
    /// headings while the style instructions told it to organise by topic into
    /// numbered sections, and the two instructions fought. Measured on the real
    /// meeting, removing them took the protocol from 2,747 characters and two
    /// headings to 17,393 and forty-one, and the quantities it recorded from one
    /// of nineteen to fourteen.
    ///
    /// The style instructions already prescribe the structure in the meeting's own
    /// language, which is where that belongs. Do not add this back to a payload.
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
    /// Some builds answer here even when reasoning is disabled.
    #[serde(default)]
    thinking: String,
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
    /// What was wrong with the previous answer, when there was one. Present only on
    /// a retry, so a first request is exactly what it always was.
    #[serde(skip_serializing_if = "Option::is_none")]
    correction: Option<&'a str>,
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
    vocabulary_revision: &'a str,
    vocabulary: &'a [String],
    section_notes: &'a [String],
    /// What was wrong with the previous answer, on a retry only.
    #[serde(skip_serializing_if = "Option::is_none")]
    correction: Option<&'a str>,
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
    /// Reasoning models otherwise return their answer in a separate `thinking`
    /// field and leave `response` empty. Protocol writing is an extraction task
    /// against a supplied transcript, so the reasoning channel is not wanted.
    think: bool,
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
            // Processing a large prompt can take minutes before the first byte arrives.
            // A dead server is caught by the connect deadline, not by this one.
            .timeout_recv_response(Some(Duration::from_secs(1800)))
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

    /// The context window the model was actually built with.
    ///
    /// Assuming a number here is not safe in either direction: too small truncates
    /// the answer, and too large costs memory the machine may not have — measured
    /// at roughly 30 KB of key-value cache per token, so a 128K window is gigabytes
    /// before any weights are loaded.
    ///
    /// Ollama reports it under `model_info`, keyed by the model's own architecture
    /// (`qwen3.context_length`, `gemma3.context_length`, and so on), so the key is
    /// found by suffix rather than guessed. Returns `None` when the server does not
    /// report one, leaving the caller to fall back rather than fail.
    pub fn model_context_length(&self, model: &str) -> Option<u32> {
        let mut response = self
            .agent
            .post(format!("{}/api/show", self.base_url))
            .send_json(serde_json::json!({ "model": model }))
            .ok()?;
        let value: serde_json::Value = response.body_mut().read_json().ok()?;
        value
            .get("model_info")?
            .as_object()?
            .iter()
            .find(|(key, _)| key.ends_with(".context_length"))
            .and_then(|(_, value)| value.as_u64())
            .and_then(|value| u32::try_from(value).ok())
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
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
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

        // A library can hold everything a firm has ever written down, but a prompt
        // cannot. Narrowing it here — before sections are planned — keeps the space
        // reserved for vocabulary equal to what is actually sent.
        let focused = focus_vocabulary(request);
        let request = &focused;

        let sections = plan_sections(request);
        if sections.len() <= 1 {
            return self.generate_in_one_pass(request, cancelled, progress);
        }
        self.generate_from_sections(request, &sections, cancelled, progress)
    }

    /// Divide the meeting into the subjects it discussed.
    ///
    /// Reads the transcript in overlapping windows and asks only which subjects each
    /// passage covers, never for prose. The answer is small — a few dozen lines for
    /// an eighty-minute meeting — so this stays cheap however long the recording is,
    /// and everything written afterwards can be written from a handful of segments
    /// rather than from the whole meeting.
    ///
    /// Segments no subject claimed are returned alongside, because a subject this
    /// pass fails to name would otherwise disappear from the protocol with nothing
    /// to show a reader that it ever existed.
    #[cfg(test)]
    pub fn find_topics(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<(Vec<crate::topics::Topic>, Vec<usize>)> {
        let mut found = Vec::new();
        let mut pending: Vec<usize> = (0..request.transcript.len()).collect();
        // Read what no subject claimed again, because on a real meeting the first
        // reading left a quarter of the segments behind and they were not
        // pleasantries: whole consecutive discussions of areas and funding limits,
        // exactly the material a protocol exists to record. Bounded, and stopped as
        // soon as a round claims nothing, so it cannot circle.
        for round in 0..3 {
            if pending.len() < request.transcript.len() / 20 {
                break;
            }
            let claimed = self.scan(request, &pending, round, cancelled, progress, &mut found)?;
            if claimed == 0 {
                break;
            }
            let taken: std::collections::HashSet<usize> = found
                .iter()
                .flat_map(|topic| topic.segments.iter().copied())
                .collect();
            pending.retain(|index| !taken.contains(index));
        }
        let grouped =
            self.group_topics(request, crate::topics::merge(found), cancelled, progress)?;
        let topics = crate::topics::absorb_small(grouped, TOPIC_MINIMUM_SEGMENTS);
        let unclaimed = crate::topics::unclaimed(request.transcript.len(), &topics);
        Ok((topics, unclaimed))
    }

    /// Showing a later reading the subjects already named was tried, on the
    /// reasoning that it sees a fragmented passage and can only name things afresh.
    /// It did not work: on the real meeting the subjects went from 48 to 47 while
    /// the segments belonging to none went from 33 to 62, for eighty seconds more.
    /// Telling the model what to reuse appears to make it claim less rather than
    /// name more consistently. Not repeated.
    ///
    /// One reading of a selection of segments, appending whatever subjects it named.
    /// Returns how many segments were claimed, so a round that achieves nothing can
    /// stop the loop rather than repeat itself.
    #[cfg(test)]
    fn scan(
        &self,
        request: &GenerationRequest,
        selection: &[usize],
        round: usize,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
        found: &mut Vec<crate::topics::Topic>,
    ) -> Result<usize> {
        let sizes: Vec<usize> = selection
            .iter()
            .map(|index| request.transcript[*index].text.len() + 8)
            .collect();
        let windows = crate::topics::plan_windows(&sizes, TOPIC_WINDOW_CHARS, TOPIC_WINDOW_OVERLAP);
        let mut claimed = 0;
        for (index, window) in windows.iter().enumerate() {
            if cancelled.load(Ordering::Acquire) {
                return Err(ProviderError::Cancelled);
            }
            let share = 8 + (index as u64 * 20) / windows.len().max(1) as u64;
            progress(
                share,
                &format!(
                    "finding_subjects:{} of {}{}",
                    index + 1,
                    windows.len(),
                    if round == 0 { "" } else { ", second reading" }
                ),
            )?;
            let prompt = selection[window.clone()]
                .iter()
                .enumerate()
                .map(|(offset, index)| {
                    format!("{}. {}", offset + 1, request.transcript[*index].text.trim())
                })
                .collect::<Vec<_>>()
                .join("\n");
            let generated = self.complete(
                request,
                Completion {
                    system: TOPIC_SYSTEM,
                    prompt: &prompt,
                    format: topics_schema(),
                    // Titles and numbers, never prose — but a window of eighty
                    // segments spread over four subjects is three hundred numbers,
                    // and a ceiling sized for the titles alone truncates the JSON.
                    num_predict: 3_072,
                },
                cancelled,
                &mut |_| Ok(()),
            )?;
            let structured: StructuredTopics = parse_structured(&generated)?;
            for topic in structured.topics {
                let segments = crate::topics::resolve_within(selection, window, &topic.segments);
                if segments.is_empty() || topic.title.trim().is_empty() {
                    continue;
                }
                claimed += segments.len();
                found.push(crate::topics::Topic {
                    title: topic.title.trim().to_string(),
                    segments,
                });
            }
        }
        Ok(claimed)
    }

    /// Join the subjects that name the same thing.
    ///
    /// Only the titles are sent, so this stays one small call however long the
    /// meeting was. If it fails the topics are returned as they were: a protocol
    /// with the facade discussed under six headings is worse than one with a single
    /// heading, and far better than no protocol at all.
    #[cfg(test)]
    fn group_topics(
        &self,
        request: &GenerationRequest,
        topics: Vec<crate::topics::Topic>,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<Vec<crate::topics::Topic>> {
        if topics.len() < 3 {
            return Ok(topics);
        }
        progress(32, &format!("joining_subjects:{}", topics.len()))?;
        // Plain code proposes, the model judges. Sorting by title put subjects
        // together only when they happened to begin with the same word, which is
        // why five subjects naming accessibility were never weighed against each
        // other: two of them begin "Rechtliche" and "Wohnungsgrundriss". Matching
        // on any telling word costs nothing and asks a far smaller question — of
        // these five, which belong together — rather than how to organise a
        // hundred and fifty.
        //
        // Sharing a word is a reason to ask, never an answer. Five subjects naming
        // the facade may be one subject, or two, or five, and the reply may hold
        // as many groups as there really are. Subjects sharing no telling word are
        // not sent anywhere: there is nothing to ask about them.
        let before = topics.len();
        let candidates = crate::topics::candidate_groups(&topics);
        let mut joined: Vec<crate::topics::Topic> = Vec::new();
        let mut asked = vec![false; topics.len()];
        for (index, candidate) in candidates.iter().enumerate() {
            if cancelled.load(Ordering::Acquire) {
                return Err(ProviderError::Cancelled);
            }
            progress(
                32,
                &format!(
                    "joining_subjects:{} of {} possible groups",
                    index + 1,
                    candidates.len()
                ),
            )?;
            let batch: Vec<crate::topics::Topic> = candidate
                .iter()
                .map(|position| {
                    asked[*position] = true;
                    topics[*position].clone()
                })
                .collect();
            joined.extend(self.group_batch(request, batch, cancelled, progress)?);
        }
        for (position, topic) in topics.into_iter().enumerate() {
            if !asked[position] {
                joined.push(topic);
            }
        }
        // Grouping invents headings, and two groups may invent the same one. Merging
        // again costs nothing and is the difference between a protocol with one
        // "Grundrissplanung" and a protocol with two.
        let joined = crate::topics::merge(joined);
        progress(
            34,
            &format!("joined_subjects:{before} subjects to {}", joined.len()),
        )?;
        Ok(joined)
    }

    /// Group one batch of subjects, judging by what was said rather than by how the
    /// titles happen to be worded. If it fails the batch is returned untouched: a
    /// protocol with the facade under six headings is worse than one heading and
    /// far better than no protocol.
    #[cfg(test)]
    fn group_batch(
        &self,
        request: &GenerationRequest,
        topics: Vec<crate::topics::Topic>,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<Vec<crate::topics::Topic>> {
        if topics.len() < 2 {
            return Ok(topics);
        }
        let listing = topics
            .iter()
            .enumerate()
            .map(|(index, topic)| {
                let excerpt = topic
                    .segments
                    .iter()
                    .filter_map(|position| request.transcript.get(*position))
                    .map(|segment| segment.text.trim())
                    .max_by_key(|text| text.len())
                    .unwrap_or_default();
                let excerpt: String = excerpt.chars().take(140).collect();
                format!("{}. {}\n   \"{excerpt}\"", index + 1, topic.title)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let generated = self.complete(
            request,
            Completion {
                system: GROUP_SYSTEM,
                prompt: &listing,
                format: groups_schema(),
                num_predict: 2_048,
            },
            cancelled,
            &mut |_| Ok(()),
        );
        let generated = match generated {
            Ok(generated) => generated,
            Err(error) => {
                progress(
                    33,
                    &format!("joining_failed:{}", truncate(&error.to_string(), 80)),
                )?;
                return Ok(topics);
            }
        };
        let Ok(structured) = parse_structured::<StructuredGroups>(&generated) else {
            progress(33, "joining_failed:the reply was not valid")?;
            return Ok(topics);
        };
        let groups: Vec<(String, Vec<i64>)> = structured
            .groups
            .into_iter()
            .filter(|group| !group.title.trim().is_empty())
            .map(|group| (group.title, group.topics))
            .collect();
        Ok(crate::topics::group(topics, &groups))
    }

    /// Short meetings fit the window, so the protocol is written directly.
    fn generate_in_one_pass(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<String> {
        let instructions = with_density(request);
        let payload = PromptPayload {
            meeting_language: &request.meeting_language,
            style_id: &request.style.id,
            style_revision: &request.style.revision,
            instructions: &instructions,
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
                num_predict: output_allowance(request),
            },
            cancelled,
            &mut |written| {
                progress(
                    arrived(60, 76, written, output_allowance(request)),
                    "generating_protocol",
                )
            },
        )?;
        let structured: StructuredProtocol = parse_structured(&generated)?;
        let markdown = strip_code_fence(&structured.protocol_markdown).to_string();
        validate_markdown(&markdown, spoken_characters(request))?;
        progress(78, "validating_protocol")?;
        Ok(markdown)
    }

    /// A real meeting exceeds the window. Each section is condensed first, then the
    /// protocol is written from the collected notes. Nothing is silently dropped:
    /// every segment belongs to exactly one section.
    fn generate_from_sections(
        &self,
        request: &GenerationRequest,
        sections: &[std::ops::Range<usize>],
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
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
            // Retried on its own when it comes back unusable, rather than failing
            // the meeting. The sections either side of a bad draw were fine, and
            // discarding them costs a quarter of an hour for one request's fault.
            let section_notes = with_correction(
                |correction| {
                    let payload = SectionPayload {
                        meeting_language: &request.meeting_language,
                        section_index: index + 1,
                        section_count: count,
                        transcript: &request.transcript[range.clone()],
                        correction,
                    };
                    let prompt = encode_prompt(&payload)?;
                    // Notes condense a section, so they should not exceed it by much.
                    // An unbounded budget invites a model to write until the cap
                    // instead of until the material runs out; the protocol's own
                    // preference is the wrong bound because it describes the finished
                    // document, not the notes.
                    let notes_ceiling = ((prompt.len() / 2) as u32).max(1024);
                    let num_predict =
                        answer_budget(request.context_tokens, prompt.len(), notes_ceiling);
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
                    let structured: StructuredNotes = parse_structured(&generated)?;
                    Ok(strip_code_fence(&structured.notes_markdown).to_string())
                },
                |notes: &String| {
                    let spoken: usize = request.transcript[range.clone()]
                        .iter()
                        .map(|segment| segment.text.chars().count())
                        .sum();
                    validate_markdown(notes, spoken)
                },
            )?;
            notes.push(section_notes);
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
        let instructions = with_density(request);
        // The last step, and the most expensive to lose: every section has already
        // been condensed by the time it runs.
        let markdown = with_correction(
            |correction| {
                let payload = SynthesisPayload {
                    meeting_language: &request.meeting_language,
                    style_id: &request.style.id,
                    style_revision: &request.style.revision,
                    instructions: &instructions,
                    vocabulary_revision: &request.vocabulary_revision,
                    vocabulary: &request.vocabulary,
                    section_notes: &notes,
                    correction,
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
                    &mut |written| {
                        progress(arrived(70, 76, written, num_predict), "generating_protocol")
                    },
                )?;
                let structured: StructuredProtocol = parse_structured(&generated)?;
                Ok(strip_code_fence(&structured.protocol_markdown).to_string())
            },
            |markdown: &String| validate_markdown(markdown, spoken_characters(request)),
        )?;
        progress(78, "validating_protocol")?;
        Ok(markdown)
    }

    /// Combine a group of consecutive notes into one, preserving their content.
    /// A single note is returned unchanged rather than sent through the model again.
    fn merge_notes(
        &self,
        request: &GenerationRequest,
        group: &[String],
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<String> {
        if group.len() == 1 {
            return Ok(group[0].clone());
        }
        let payload = MergePayload {
            meeting_language: &request.meeting_language,
            notes: group,
        };
        let prompt = encode_prompt(&payload)?;
        let notes_ceiling = ((prompt.len() / 2) as u32).max(1024);
        let num_predict = answer_budget(request.context_tokens, prompt.len(), notes_ceiling);
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
        let structured: StructuredNotes = parse_structured(&generated)?;
        Ok(structured.notes_markdown)
    }

    /// One bounded, cancellable, streamed completion.
    fn complete(
        &self,
        request: &GenerationRequest,
        call: Completion<'_>,
        cancelled: &AtomicBool,
        tick: &mut dyn FnMut(usize) -> Result<()>,
    ) -> Result<String> {
        let body = OllamaRequest {
            model: &request.model,
            system: call.system,
            prompt: call.prompt,
            stream: true,
            think: false,
            format: call.format,
            options: OllamaOptions {
                seed: request.seed,
                temperature: f64::from(request.temperature_milli) / 1000.0,
                num_ctx: request.context_tokens,
                num_predict: call.num_predict,
            },
            // Released the moment the request finishes, rather than held for
            // minutes afterwards.
            //
            // One heavy task runs at a time in this application, but the lock is
            // let go when generation returns while Ollama would still be holding
            // the weights — so a transcription starting straight afterwards would
            // meet a machine with several gigabytes of language model still on it.
            // On the eight-gigabyte target that is the difference between working
            // and swapping.
            //
            // The cost is reloading the model if somebody generates twice in a row:
            // seconds against the fourteen minutes a generation takes.
            keep_alive: "0",
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
        // When the last token arrived, so a model that has stopped answering can be
        // told from one that is still working.
        //
        // There was no deadline here at all: the global timeout is off and the
        // thirty-minute one covers receiving the response headers, not the stream
        // that follows. A model that stalled after its first token would have been
        // waited on indefinitely while the interface showed a fixed seventy per cent
        // — which is exactly the state somebody cannot distinguish from progress.
        let mut last_chunk = Instant::now();
        loop {
            if cancelled.load(Ordering::Acquire) {
                return Err(ProviderError::Cancelled);
            }
            if last_chunk.elapsed() >= SILENCE_BEFORE_GIVING_UP {
                return Err(ProviderError::Stalled);
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
            // One JSON object per line, and the provider escapes what it sends. The
            // model's own answer is the thing that needs repairing, and that is done
            // where it is parsed rather than here.
            let chunk: StreamChunk = serde_json::from_str(line.trim()).map_err(|error| {
                ProviderError::InvalidResponse(truncate(&error.to_string(), 280))
            })?;
            last_chunk = Instant::now();
            generated.push_str(if chunk.response.is_empty() {
                &chunk.thinking
            } else {
                &chunk.response
            });
            if generated.len() > MAX_RESPONSE_BYTES {
                return Err(ProviderError::ResponseTooLarge);
            }
            if last_progress.elapsed() >= Duration::from_millis(100) {
                tick(generated.len())?;
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
        Ok(repair_escaped_newlines(&generated))
    }
}

/// Parse what the model returned as structured output, repairing it first if it is
/// not quite JSON.
///
/// Generation asks the model for a JSON object whose fields hold the protocol, and
/// a protocol is multi-line markdown. A model that writes a raw newline inside that
/// string rather than escaping it has produced invalid JSON, and the whole
/// generation is lost — measured: `ministral-3:8b` failed the reference meeting this
/// way at two seeds out of three, each after minutes of work.
///
/// The repair runs only after a straight parse fails, so a well-formed answer costs
/// nothing, and it only escapes control characters found inside strings.
fn parse_structured<T: serde::de::DeserializeOwned>(generated: &str) -> Result<T> {
    match serde_json::from_str(generated) {
        Ok(value) => Ok(value),
        Err(first) => serde_json::from_str(&escape_raw_controls(generated)).map_err(|second| {
            // Keep what could not be read, when somebody asks for it. Off unless the
            // variable is set, because this is the contents of a meeting and it must
            // not be written anywhere by default.
            if let Some(path) = std::env::var_os("LOCALOG_KEEP_UNREADABLE") {
                let _ = std::fs::write(path, generated);
            }
            // Both, because reporting only the first hid a bug for three attempts:
            // a parse that never reached the repair and a repair that failed produce
            // the same message, and the difference is the whole diagnosis.
            ProviderError::InvalidResponse(truncate(
                &format!("{first}; after repair: {second}"),
                280,
            ))
        }),
    }
}

/// Move a progress figure as an answer arrives, so a person can tell a model that
/// is working from one that has stopped.
///
/// It was a constant. A generation of the reference meeting shows seventy per cent
/// for fourteen minutes and then either finishes or does not, which is
/// indistinguishable from a stall and is the state this project spent nine hours in
/// while watching its own harness.
///
/// The estimate is rough on purpose: what fraction of the tokens the model was
/// allowed has arrived, counted in characters at roughly four to a token. A figure
/// that moves is worth more here than a figure that is exact, and it never reaches
/// the end of its band, because arriving there while still writing would be its own
/// kind of lie.
fn arrived(from: u64, to: u64, written: usize, allowed_tokens: u32) -> u64 {
    let expected = (allowed_tokens as usize).saturating_mul(4).max(1);
    let through = (written as f64 / expected as f64).clamp(0.0, 0.95);
    from + ((to - from) as f64 * through) as u64
}

/// Make a nearly-JSON answer readable, without changing what it says.
///
/// A model asked for JSON whose field holds markdown writes markdown, and markdown
/// and JSON disagree about two characters. Both faults were measured in one answer
/// from `ministral-3:8b`:
///
/// - **Raw newlines inside the string.** A protocol is multi-line, and the model
///   pressed return rather than writing `\n`. Invalid JSON from the second line on.
/// - **A backslash that is not a JSON escape.** Markdown ends a line with a
///   backslash to force a break, so the answer held a backslash followed by a real
///   newline — which is not one of JSON's escapes, and which a first attempt at this
///   made worse by treating the backslash as though it began one.
///
/// Both are repaired by reading the answer as JSON's own grammar does: only inside a
/// string, tracking whether a quote is escaped, and checking that a backslash really
/// begins an escape before trusting it. Nothing outside a string is touched, because
/// a newline between tokens is legal there and escaping it would break an answer that
/// was never broken.
fn escape_raw_controls(text: &str) -> String {
    /// The only characters JSON allows after a backslash.
    const VALID_ESCAPE: &[char] = &['"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u'];
    let mut out = String::with_capacity(text.len() + 64);
    let mut characters = text.chars().peekable();
    let mut in_string = false;
    while let Some(character) = characters.next() {
        match character {
            '\\' if in_string => match characters.peek() {
                // A real escape: keep both halves as they are.
                Some(next) if VALID_ESCAPE.contains(next) => {
                    out.push('\\');
                    out.push(characters.next().expect("peeked"));
                }
                // A backslash the model meant literally. Escaping it here rather
                // than consuming what follows is the whole difference: the newline
                // after it is then seen as a control character and repaired too.
                _ => out.push_str("\\\\"),
            },
            '"' => {
                in_string = !in_string;
                out.push('"');
            }
            control if in_string && (control as u32) < 0x20 => match control {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                other => out.push_str(&format!("\\u{:04x}", other as u32)),
            },
            other => out.push(other),
        }
    }
    out
}

/// Turn a literal backslash-n the model wrote as text into an actual line break.
///
/// The JSON is already unescaped by the time it reaches here, so a `\n` surviving
/// in the text is two characters the model typed rather than an encoding fault.
/// Measured on the reference meeting: gemma4:12b leaves three to nine of them in a
/// protocol, each one appearing mid-sentence in a document somebody is meant to
/// hand to a client.
///
/// Safe to do unconditionally for this product. A meeting protocol is prose,
/// headings and tables, and has no legitimate use for the sequence — where a
/// technical discussion really does mention one, losing it costs less than
/// printing an escape code in the middle of a paragraph.
fn repair_escaped_newlines(markdown: &str) -> String {
    if !markdown.contains("\\n") {
        return markdown.to_string();
    }
    markdown.replace("\\n", "\n")
}

/// One model call: what to ask, how to constrain it, and how much answer to allow.
struct Completion<'a> {
    system: &'static str,
    prompt: &'a str,
    format: serde_json::Value,
    num_predict: u32,
}

const PROTOCOL_SYSTEM: &str = "Create a reviewable professional meeting protocol using only the supplied transcript. Never invent decisions, actions, owners, or dates. State uncertainty explicitly. Follow the controlled style and return only schema-valid JSON. If the payload carries a correction, your previous answer was rejected for the reason it gives; fix exactly that and return the whole answer again.";

const SECTION_SYSTEM: &str = "Record one section of a meeting transcript as detailed factual notes in the meeting's language. Completeness matters more than brevity: keep every topic discussed, every decision, every agreed action and its owner, every open question, and every number, measurement, area, date and proper name exactly as stated. Write one bullet per distinct point rather than merging several into a summary sentence. Do not add anything that was not said, and do not write a protocol yet. Return only schema-valid JSON. If the payload carries a correction, your previous answer was rejected for the reason it gives; fix exactly that and return the whole answer again.";

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

/// Characters of vocabulary a prompt will carry.
///
/// Vocabulary competes with the transcript for the same window, so it cannot grow
/// with the library. Roughly 650 tokens of German: enough for the names, firms and
/// recurring subjects of one meeting, and far short of what a firm accumulates.
const VOCABULARY_BUDGET: usize = 2_000;

/// Narrow a project's vocabulary to the terms this meeting actually needs.
///
/// A vocabulary that fits is sent whole: there is nothing to gain by withholding
/// part of a short list, and a term that was misheard would otherwise lose the very
/// correction it exists to provide.
///
/// A vocabulary that does not fit is reduced to the terms the transcript uses. This
/// is decided on the transcript rather than guessed, so a term drops out only when
/// the meeting did not mention it — in which case it could not have helped write
/// this protocol, and naming it to the model risks it being written in regardless.
fn focus_vocabulary(request: &GenerationRequest) -> GenerationRequest {
    let mut focused = request.clone();
    let total: usize = request.vocabulary.iter().map(|term| term.len() + 2).sum();
    if total <= VOCABULARY_BUDGET {
        return focused;
    }
    let spoken = request
        .transcript
        .iter()
        .map(|segment| segment.text.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let mut used = 0;
    focused.vocabulary = request
        .vocabulary
        .iter()
        .filter(|term| mentions(&spoken, term))
        .take_while(|term| {
            used += term.len() + 2;
            used <= VOCABULARY_BUDGET
        })
        .cloned()
        .collect();
    focused
}

/// Whether a transcript used a term.
///
/// Matching is done on the lowered forms without word boundaries, because German
/// builds compounds out of its terms: a project that lists "Bauteil" needs its
/// spelling respected inside a longer compound built from it, and a listed
/// surname needs it inside the genitive. A term of one or two characters is too short to match
/// anything meaningfully and is treated as always relevant.
fn mentions(spoken: &str, term: &str) -> bool {
    let needle = term.trim().to_lowercase();
    needle.chars().count() <= 2 || spoken.contains(&needle)
}

/// Divide the transcript into contiguous section ranges that each fit the model's
/// window alongside the style, vocabulary and room for the answer. Returns a single
/// range when the whole transcript already fits.
/// How much of the context the answer is allowed to claim.
///
/// The answer and the material it is written from share one window. A configuration
/// that promises the answer the whole context leaves nothing to read from, and the
/// planner below then falls to its last resort of one section per segment: hundreds
/// of sections, none of them long enough to carry a subject. Measured, that produced
/// ninety-four characters for an eighty-minute meeting.
///
/// Half is the most the answer can take while leaving a window worth filling.
fn output_allowance(request: &GenerationRequest) -> u32 {
    request.maximum_output_tokens.min(request.context_tokens / 2)
}

/// The tokens left over for everything the model has to read.
fn reading_window(request: &GenerationRequest) -> usize {
    request.context_tokens.saturating_sub(output_allowance(request)) as usize
}

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
    let window = reading_window(request);
    let budget_chars = window * CHARS_PER_TOKEN * SAFETY_NUMERATOR / SAFETY_DENOMINATOR;
    // Everything in the prompt that is not transcript still has to fit.
    let overhead: usize = request
        .style
        .instructions
        .iter()
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
    let window = reading_window(request);
    let budget_chars = window * CHARS_PER_TOKEN * 7 / 10;
    let overhead: usize = request
        .style
        .instructions
        .iter()
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

/// Reject a protocol that is not a protocol at all.
///
/// This used to also require each of the style's section names to appear
/// literally in the output, and that check could never have passed for the
/// audience this product is built for. A style names its sections in English —
/// "Summary", "Decisions" — while the protocol is written in the language of the
/// meeting, so a German meeting correctly produced "Zusammenfassung" and was
/// rejected for it. Every German protocol failed, and German is the first
/// language LocaLog is meant to serve.
///
/// Matching translated headings by string is not a fixable version of that idea,
/// and discarding a finished draft over a heading was the wrong response in any
/// case: these are drafts for a person to review, and a draft missing a section
/// is more useful to that person than no draft at all. The sections are still
/// given to the model as part of the style, which is where they belong.
///
/// Reporting which sections a draft appears to cover is worth doing, and belongs
/// in the review workspace next to the text rather than in a gate that throws the
/// work away. It is recorded in the plan.
/// The style's instructions with its density directive appended.
///
/// Density is kept structured rather than written into a style's instruction list
/// so that it can also size the answer budget and be shown in the library, but the
/// model still needs telling, and this is where it is told.
fn with_density(request: &GenerationRequest) -> Vec<String> {
    let mut instructions = request.style.instructions.clone();
    instructions.push(request.style.density.directive().to_string());
    instructions
}

/// Characters of transcript to show the topic pass at once.
///
/// Small on purpose. The point of finding subjects first is that nothing after it
/// needs a large context, and the pass itself should not reintroduce the problem
/// it exists to remove — this fits inside the window an eight-gigabyte machine can
/// afford without argument.
#[cfg(test)]
const TOPIC_WINDOW_CHARS: usize = 6_000;

/// Segments each window re-reads from the one before, so a subject that straddles
/// a boundary is seen whole by at least one of them.
#[cfg(test)]
const TOPIC_WINDOW_OVERLAP: usize = 6;

/// Segments a subject needs before it earns a section of its own. Below this it is
/// a remark inside another discussion, and is folded into the nearest one.
#[cfg(test)]
const TOPIC_MINIMUM_SEGMENTS: usize = 4;

/// Asking only for "the subjects" produced one per exchange: fifty titles of two
/// segments each over an eighty-minute meeting, where the protocol a person wrote
/// has thirty sections in total. A small model reads "subject" as "thing just
/// said" unless told the size wanted, so the size is stated, in the terms the
/// answer is for — a section of a document, not a turn in a conversation.
#[cfg(test)]
const TOPIC_SYSTEM: &str = "Divide this passage of a meeting transcript into the few substantial subjects it covers. Each segment is numbered.\n\nGive between one and four subjects for the whole passage. A subject is something a written protocol would give its own section to, gathering many minutes of talk under one heading — not every question, remark or exchange. If the passage is one long discussion of a single thing, return one subject covering it.\n\nFor each subject give a short title in the meeting's language and the numbers of every segment belonging to it. Most segments belong to a subject; a segment of pleasantries or crosstalk may belong to none. Do not summarise, do not write a protocol, and do not name a subject the passage does not discuss. Return only schema-valid JSON.";

/// A meeting that returns to a subject is described afresh by each window that
/// sees it, so the same thing arrives under several names. The list of names is
/// short even for a long meeting, which is why this can be settled in one pass
/// over titles alone rather than by reading the transcript again.
#[cfg(test)]
const GROUP_SYSTEM: &str = "Below is a numbered list of subjects taken from one meeting, each with a sentence spoken about it. Several may name the same subject in different words, because the meeting returned to it more than once.\n\nJudge by what was said, not only by the wording of the titles. Group the subjects that a written protocol would gather under a single heading. For each group give one heading of your own in the meeting's language, naming what the group is about -- never two of the old titles joined together -- and the numbers of the titles it covers. Leave a subject out of every group if it stands on its own. Do not group two subjects merely because they were discussed near each other, and do not invent a subject that is not in the list. Return only schema-valid JSON.";

#[derive(Debug, Deserialize)]
#[cfg(test)]
struct StructuredGroups {
    groups: Vec<StructuredGroup>,
}

#[derive(Debug, Deserialize)]
#[cfg(test)]
struct StructuredGroup {
    title: String,
    topics: Vec<i64>,
}

#[cfg(test)]
fn groups_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "groups": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "topics": { "type": "array", "items": { "type": "integer" } }
                    },
                    "required": ["title", "topics"]
                }
            }
        },
        "required": ["groups"]
    })
}

#[derive(Debug, Deserialize)]
#[cfg(test)]
struct StructuredTopics {
    topics: Vec<StructuredTopic>,
}

#[derive(Debug, Deserialize)]
#[cfg(test)]
struct StructuredTopic {
    title: String,
    segments: Vec<i64>,
}

#[cfg(test)]
fn topics_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "topics": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "segments": { "type": "array", "items": { "type": "integer" } }
                    },
                    "required": ["title", "segments"]
                }
            }
        },
        "required": ["topics"]
    })
}

/// Ask again when an answer is unusable, telling the model what was wrong with the
/// last one.
///
/// A section is one request among many, and the sections around a bad one were fine.
/// Losing the whole run to a single bad draw costs a quarter of an hour of somebody's
/// machine; asking that one section again costs a fraction of it.
///
/// The correction is the point rather than the repetition. "You returned JSON,
/// return markdown" is a strong instruction, and a model that has just done so is
/// far likelier to comply than one asked the same question twice. Measured need:
/// `ministral-3:8b` returned a JSON document at one seed and a two-line stub at
/// another, from the same prompt that worked at a third.
///
/// Bounded and small. A model that fails three times is not having bad luck, and
/// spending a person's afternoon proving it is worse than telling them.
const ATTEMPTS_PER_STEP: usize = 3;

/// Run one generation step, correcting and retrying while the answer is unusable.
///
/// `check` is what decides usable, and its message becomes the correction — so the
/// same rule states the requirement and explains the failure, and the two cannot
/// drift apart.
fn with_correction<T>(
    mut attempt: impl FnMut(Option<&str>) -> Result<T>,
    mut check: impl FnMut(&T) -> Result<()>,
) -> Result<T> {
    let mut correction: Option<String> = None;
    let mut last: Option<ProviderError> = None;
    for _ in 0..ATTEMPTS_PER_STEP {
        let answer = attempt(correction.as_deref())?;
        match check(&answer) {
            Ok(()) => return Ok(answer),
            Err(problem) => {
                correction = Some(problem.to_string());
                last = Some(problem);
            }
        }
    }
    Err(last.unwrap_or_else(|| {
        ProviderError::InvalidResponse("The model could not produce a usable answer.".into())
    }))
}

/// How many characters the meeting itself holds, which is what a protocol of it is
/// judged plausible against.
fn spoken_characters(request: &GenerationRequest) -> usize {
    request
        .transcript
        .iter()
        .map(|segment| segment.text.chars().count())
        .sum()
}

/// Take a protocol out of the code fence a model wrapped it in.
///
/// Asked for markdown, models return it fenced — ```` ```markdown ```` and even
/// ```` ```json ````. The fence is not part of the protocol and would be shown to a
/// person as literal backticks, so it is removed. Deterministic, and cheaper than
/// asking the model again.
fn strip_code_fence(markdown: &str) -> &str {
    let trimmed = markdown.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return trimmed;
    };
    // The word after the backticks, if any, names the language and is not content.
    let after_language = match rest.find('\n') {
        Some(at) if rest[..at].chars().all(char::is_alphanumeric) => &rest[at + 1..],
        _ => rest,
    };
    after_language
        .trim_end()
        .strip_suffix("```")
        .unwrap_or(after_language)
        .trim()
}

/// Refuse a protocol that is plainly not one.
///
/// `ministral-3:8b` returned two of these in three runs on the reference meeting: a
/// 211-byte stub carrying the placeholders the style forbids, and a JSON document
/// with `metadata` and `organisations` keys where markdown was asked for. Both were
/// accepted by a check that only looked for emptiness, and the second scored 28 of
/// 35 figures because every number was in it as text.
///
/// The bounds are deliberately loose. A terse style legitimately writes little, so
/// this is not a quality judgement — it catches the answer that is not a protocol at
/// all, and leaves everything else to the person reading it.
fn validate_markdown(markdown: &str, transcript_chars: usize) -> Result<()> {
    let markdown = markdown.trim();
    if markdown.is_empty() {
        return Err(ProviderError::InvalidResponse(
            "The model returned an empty protocol.".into(),
        ));
    }
    // A JSON document where markdown was asked for.
    //
    // Judged by shape rather than by whether it parses. The first version required
    // it to parse, and a model then returned twenty kilobytes of malformed JSON
    // which passed as markdown for exactly that reason: the check was strictest
    // about the answers closest to being right.
    //
    // Opening with a brace or a bracket is not enough on its own, because a protocol
    // may legitimately begin with one. It must also carry a quoted key, which prose
    // does not.
    let opens_as_data = markdown.starts_with('{') || markdown.starts_with('[');
    let carries_keys = markdown
        .chars()
        .take(400)
        .collect::<String>()
        .contains("\":");
    if opens_as_data && carries_keys {
        return Err(ProviderError::InvalidResponse(
            "The model returned a JSON document instead of a protocol.".into(),
        ));
    }
    // A hundredth of what was said, which even the tersest style clears. The stub
    // that prompted this was three thousandths.
    let least = (transcript_chars / 100).clamp(200, 4_000);
    if transcript_chars > 0 && markdown.chars().count() < least {
        return Err(ProviderError::InvalidResponse(format!(
            "The model returned {} characters for a meeting of {transcript_chars}, which is too \
             little to be a protocol of it.",
            markdown.chars().count()
        )));
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

    /// Run the repair against an answer a model really produced.
    ///
    /// `LOCALOG_KEEP_UNREADABLE` writes one out when parsing fails; point this at
    /// it. Guessing at the shape of these is what cost three wrong attempts.
    #[test]
    #[ignore = "requires a captured unreadable answer"]
    fn a_captured_answer_can_be_read() {
        let path = std::env::var("LOCALOG_UNREADABLE").expect("a captured answer");
        let raw = std::fs::read_to_string(&path).expect("readable");
        println!("{} bytes captured", raw.len());
        let straight = serde_json::from_str::<serde_json::Value>(&raw);
        println!(
            "straight parse: {:?}",
            straight.as_ref().err().map(|e| e.to_string())
        );
        let repaired = escape_raw_controls(&raw);
        match serde_json::from_str::<serde_json::Value>(&repaired) {
            Ok(value) => {
                let keys: Vec<&String> = value.as_object().expect("an object").keys().collect();
                println!("REPAIRED — keys {keys:?}");
            }
            Err(error) => panic!("the repair did not fix it: {error}"),
        }
    }

    /// A bad draw costs one request, not the meeting.
    #[test]
    fn an_unusable_answer_is_asked_again_with_the_reason() {
        use std::cell::RefCell;
        let corrections: RefCell<Vec<Option<String>>> = RefCell::new(Vec::new());
        let attempts = RefCell::new(0);
        let answer = with_correction(
            |correction| {
                corrections
                    .borrow_mut()
                    .push(correction.map(str::to_string));
                let mut count = attempts.borrow_mut();
                *count += 1;
                // Unusable twice, then right: the shape ministral-3:8b produced.
                Ok(if *count < 3 {
                    r#"{"a":1}"#.to_string()
                } else {
                    "## Beschluss\n\nDie Fassade bleibt.".to_string()
                })
            },
            |markdown: &String| validate_markdown(markdown, 0),
        )
        .expect("the third answer is usable");
        assert!(answer.starts_with("## Beschluss"));
        assert_eq!(*attempts.borrow(), 3);
        let seen = corrections.borrow();
        assert_eq!(seen[0], None, "the first request is what it always was");
        assert!(
            seen[1].as_deref().unwrap_or_default().contains("JSON"),
            "the retry must say what was wrong: {:?}",
            seen[1]
        );
    }

    /// A model failing three times is not having bad luck, and spending somebody's
    /// afternoon proving it is worse than telling them.
    #[test]
    fn asking_forever_is_not_the_alternative() {
        let attempts = std::cell::RefCell::new(0);
        let outcome = with_correction(
            |_| {
                *attempts.borrow_mut() += 1;
                Ok(String::new())
            },
            |markdown: &String| validate_markdown(markdown, 0),
        );
        assert!(outcome.is_err());
        assert_eq!(*attempts.borrow(), ATTEMPTS_PER_STEP);
    }

    /// The two answers ministral-3:8b really returned, which a check for emptiness
    /// let through. Both are shaped wrongly rather than written badly, which is what
    /// makes them catchable without a model.
    #[test]
    fn an_answer_that_is_not_a_protocol_is_refused() {
        let spoken = 73_000;
        // A JSON document where markdown was asked for. It scored 28 of 35 figures,
        // because every number is in it as text.
        let as_json = r#"{"meeting_protocol":{"metadata":{"language":"de"},
            "participants":{"organisations":[{"name":"Nokia"}]}}}"#;
        assert!(validate_markdown(as_json, spoken).is_err());

        // And the same thing malformed, which a check that required it to parse let
        // through — twenty kilobytes of it, accepted as a protocol.
        let broken_json = r#"{
  "meeting_protocol": {
    "language": "German",
    "uncertainty_notes": ["Keine expliziten V"#;
        assert!(
            validate_markdown(broken_json, spoken).is_err(),
            "malformed JSON is still not a protocol"
        );

        // A stub: a heading and two of the placeholders the style forbids.
        let stub = "# Protokoll der Besprechung\n*Datum: [nicht im Transkript genannt]*";
        assert!(validate_markdown(stub, spoken).is_err());

        // And a real protocol passes, terse though it is.
        let real = "## Beschlüsse\n\n".to_string() + &"Die Fassade bleibt. ".repeat(60);
        assert!(validate_markdown(&real, spoken).is_ok());
    }

    /// A protocol that happens to open with a brace is not a JSON document, and
    /// refusing it would throw away somebody's meeting over a punctuation mark.
    #[test]
    fn a_protocol_beginning_with_a_brace_is_not_mistaken_for_json() {
        let awkward = "{ Anmerkung } Die Fassade bleibt. ".repeat(30);
        assert!(validate_markdown(&awkward, 73_000).is_ok());
    }

    /// Models return markdown fenced, including — measured — fenced as json.
    #[test]
    fn a_fenced_protocol_is_unwrapped() {
        assert_eq!(
            strip_code_fence("```markdown\n## Beschluss\n\nDie Fassade bleibt.\n```"),
            "## Beschluss\n\nDie Fassade bleibt."
        );
        assert_eq!(strip_code_fence("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        // An unfenced protocol is returned as it is, and one whose own content uses
        // a fence for a code sample keeps it.
        assert_eq!(strip_code_fence("## Beschluss"), "## Beschluss");
    }

    /// A progress figure that does not move cannot be told from a stalled one, and
    /// this project spent nine hours in exactly that state watching its own harness.
    #[test]
    fn progress_moves_as_the_answer_arrives() {
        // 8,192 tokens allowed is roughly 32,768 characters expected.
        let allowed = 8_192;
        let start = arrived(60, 76, 0, allowed);
        let quarter = arrived(60, 76, 8_000, allowed);
        let most = arrived(60, 76, 30_000, allowed);
        assert_eq!(start, 60);
        assert!(quarter > start, "it must move: {start} then {quarter}");
        assert!(most > quarter, "and keep moving: {quarter} then {most}");
        // Never the end of its band while still writing, which would be its own lie.
        assert!(most < 76);
        assert!(arrived(60, 76, usize::MAX, allowed) < 76);
    }

    /// A model that writes nothing at all must not make the arithmetic divide by it.
    #[test]
    fn progress_survives_a_model_allowed_no_tokens() {
        assert_eq!(arrived(60, 76, 0, 0), 60);
        assert!(arrived(60, 76, 1_000, 0) <= 76);
    }

    /// Both faults exactly as they arrived in one answer from ministral-3:8b: raw
    /// newlines inside the string because a protocol is multi-line, and a backslash
    /// ending a line because that is how markdown forces a break. The second was
    /// what a first attempt at this repair got wrong, by trusting the backslash to
    /// begin a JSON escape.
    #[test]
    fn a_markdown_answer_that_is_not_quite_json_is_still_read() {
        // Raw newlines, and a backslash forcing a markdown line break before one.
        let broken = "{\"protocol_markdown\":\"## Beschluss\n\nDie Fassade bleibt.\\\nEnde.\"}";
        assert!(
            serde_json::from_str::<serde_json::Value>(broken).is_err(),
            "the fixture must really be invalid JSON"
        );
        let structured: StructuredProtocol = parse_structured(broken).expect("repaired");
        assert!(structured.protocol_markdown.starts_with("## Beschluss"));
        assert!(structured.protocol_markdown.ends_with("Ende."));
        // Four lines: the heading, a blank, the sentence, and what the backslash broke.
        assert_eq!(structured.protocol_markdown.lines().count(), 4);
    }

    /// A backslash that does begin a real escape must be left alone, or every
    /// quotation mark in a German protocol would gain a stray backslash.
    #[test]
    fn a_real_escape_is_not_escaped_twice() {
        let fine = "{\"protocol_markdown\":\"Er sagte \\\"ja\\\" dazu.\"}";
        let structured: StructuredProtocol = parse_structured(fine).expect("parsed");
        assert_eq!(structured.protocol_markdown, "Er sagte \"ja\" dazu.");
    }

    /// A well-formed answer must not be touched by the repair, and must not pay for
    /// it either — the straight parse is tried first.
    #[test]
    fn a_well_formed_structured_answer_is_parsed_unchanged() {
        let fine = "{\"protocol_markdown\":\"## Beschluss\\n\\nDie Fassade bleibt.\"}";
        let structured: StructuredProtocol = parse_structured(fine).expect("parsed");
        assert_eq!(
            structured.protocol_markdown,
            "## Beschluss\n\nDie Fassade bleibt."
        );
    }

    /// Whitespace between tokens is legal JSON and must not be touched, or a
    /// document that was never broken would be broken by the repair.
    #[test]
    fn whitespace_between_tokens_is_left_alone() {
        let fine = "{\"response\": \"ok\",\n \"done\": true}";
        let value: serde_json::Value =
            serde_json::from_str(&escape_raw_controls(fine)).expect("still valid");
        assert_eq!(value["response"], "ok");
        assert_eq!(value["done"], true);
    }

    /// An escaped quote does not end the string it sits in, so what follows is
    /// still inside one and still needs repairing.
    #[test]
    fn an_escaped_quote_does_not_confuse_the_repair() {
        let broken = "{\"response\":\"sagte \\\"ja\\\" und\ndann\",\"done\":false}";
        let value: serde_json::Value =
            serde_json::from_str(&escape_raw_controls(broken)).expect("repaired");
        assert_eq!(value["response"], "sagte \"ja\" und\ndann");
    }

    /// A model that types the two characters instead of a line break leaves an
    /// escape code in the middle of a professional document. Measured: three to
    /// nine per protocol from the model this project is moving to.
    #[test]
    fn an_escape_the_model_typed_becomes_a_line_break() {
        let written = "Der Aufzug wurde eingerückt.\\n*   **Treppenhaus:** Sicherheitstreppenhaus.";
        let repaired = repair_escaped_newlines(written);
        assert!(
            !repaired.contains("\\n"),
            "the escape should be gone: {repaired}"
        );
        assert_eq!(repaired.lines().count(), 2);
        // A protocol without any is returned untouched rather than rebuilt.
        let clean = "## Beschluss\n\nDie Fassade bleibt.";
        assert_eq!(repair_escaped_newlines(clean), clean);
    }
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

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

    /// A context no larger than the promised answer used to leave a zero-width
    /// reading window, and the planner fell to one section per segment. Six hundred
    /// and seventy-five sections is not a plan, and the model returned ninety-four
    /// characters for an eighty-minute meeting.
    #[test]
    fn the_answer_never_claims_the_whole_context() {
        let mut request = synthetic_request(300, 200);
        request.context_tokens = 8_192;
        request.maximum_output_tokens = 8_192;

        assert_eq!(output_allowance(&request), 4_096);
        assert!(reading_window(&request) > 0);
        let sections = plan_sections(&request);
        assert!(
            sections.len() < request.transcript.len(),
            "one section per segment is the last resort, not a plan: {} sections for {} segments",
            sections.len(),
            request.transcript.len()
        );
    }

    #[test]
    fn a_generous_context_leaves_the_requested_answer_alone() {
        let mut request = synthetic_request(300, 200);
        request.context_tokens = 40_960;
        request.maximum_output_tokens = 8_192;
        assert_eq!(output_allowance(&request), 8_192);
    }

    #[test]
    fn each_section_stays_within_the_character_budget() {
        let request = synthetic_request(300, 200);
        let window = reading_window(&request);
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
                density: crate::domain::ProtocolDensity::Comprehensive,
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
    fn a_vocabulary_that_fits_is_sent_whole() {
        let mut request = synthetic_request(4, 5);
        // Including a term the meeting never mentioned: withholding it from a short
        // list saves nothing, and a misheard term needs its correction present.
        request.vocabulary = vec!["NORVEK".into(), "Mustermann".into()];
        assert_eq!(
            focus_vocabulary(&request).vocabulary,
            vec!["NORVEK".to_string(), "Mustermann".to_string()]
        );
    }

    #[test]
    fn an_oversized_vocabulary_keeps_only_what_the_meeting_said() {
        let mut request = synthetic_request(3, 4);
        request.transcript[1].text = "Die Bauteilplanung von NORVEK ist unverändert.".into();
        // A firm's accumulated library, of which this meeting used two entries.
        let mut vocabulary: Vec<String> =
            (0..400).map(|index| format!("Fremdwort{index}")).collect();
        vocabulary.insert(0, "NORVEK".into());
        vocabulary.insert(1, "Bauteil".into());
        request.vocabulary = vocabulary;

        let focused = focus_vocabulary(&request).vocabulary;
        // "Bauteil" survives inside the compound "Bauteilplanung", which is how
        // German uses its own terminology.
        assert_eq!(focused, vec!["NORVEK".to_string(), "Bauteil".to_string()]);
    }

    #[test]
    fn a_narrowed_vocabulary_still_respects_the_budget() {
        let mut request = synthetic_request(2, 3);
        request.transcript[0].text = "wort ".repeat(4_000);
        // Every one of these is genuinely used, so only the budget can stop them.
        request.vocabulary = (0..500).map(|_| "wort".to_string()).collect();
        let focused = focus_vocabulary(&request).vocabulary;
        let length: usize = focused.iter().map(|term| term.len() + 2).sum();
        assert!(length <= VOCABULARY_BUDGET, "sent {length} characters");
        assert!(!focused.is_empty());
    }

    #[test]
    fn rejects_only_a_protocol_that_is_not_one() {
        assert!(validate_markdown("# Summary\n\n# Actions\n", 0).is_ok());
        // A protocol written in the meeting's language names its sections in that
        // language. Rejecting it for that made every German meeting fail.
        assert!(
            validate_markdown("# Zusammenfassung\n\n# Maßnahmen\n", 0).is_ok(),
            "a translated heading is not a defect"
        );
    }

    #[test]
    fn rejects_empty_protocols() {
        assert!(validate_markdown("  ", 0).is_err());
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
