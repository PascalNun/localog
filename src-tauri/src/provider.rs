//! The first production-shaped local protocol provider.
//!
//! This module talks to an already running, user-managed Ollama instance. It
//! deliberately does not start the server, pull models, or expose a provider
//! plugin surface. The rest of the application depends on the small typed
//! boundary below, which keeps a later provider decision local to this file.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
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
    /// A protocol of this meeting cannot fit in one answer.
    ///
    /// Known before the first model call, so nothing has failed and nothing was
    /// asked: this is a refusal, not a fault. It was an `InvalidResponse` until
    /// 29 August 2026, which was a category error with a consequence — the two
    /// numbers a person needs in order to act travelled as free prose in the one
    /// variant whose prose is otherwise correction text written for a model.
    BeyondOneAnswer { expected: usize, ceiling: usize },
}

impl ProviderError {
    /// Whether this is one request going wrong rather than the machine being wrong.
    ///
    /// A bad answer, a truncated one, or a model that went quiet says nothing about
    /// the next request, so the meeting can continue without this stretch of it. A
    /// missing model or a changed runtime will fail every remaining section the same
    /// way, and continuing would produce a document that is all holes.
    pub(crate) fn is_a_bad_draw(&self) -> bool {
        matches!(
            self,
            Self::InvalidResponse(_)
                | Self::ResponseTooLarge
                | Self::IncompleteResponse
                | Self::Stalled
        )
    }
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
            Self::BeyondOneAnswer { expected, ceiling } => write!(
                formatter,
                "a protocol of this meeting is about {expected} characters and one \
                 answer holds about {ceiling}"
            ),
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
    #[serde(default)]
    pub details: ModelDetails,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDetails {
    /// Ollama's own reading, such as "8.9B". Absent on older builds.
    ///
    /// The alias is not decoration. This shape is serialised to the interface in
    /// camelCase and read from Ollama in snake_case, and without it every model
    /// arrived with an empty size and was quietly judged too small to ask.
    #[serde(default, alias = "parameter_size")]
    pub parameter_size: String,
}

/// How many billions of parameters a model has, as far as the runtime will say.
pub fn billions_of_parameters(descriptor: &ModelDescriptor) -> Option<f32> {
    let text = descriptor.details.parameter_size.trim();
    let number = text.strip_suffix(['B', 'b'])?;
    number.parse::<f32>().ok()
}

/// Whether a model is worth asking to check a rewrite.
///
/// Measured rather than guessed, on eight pairs of passages where four had a fact
/// genuinely altered: a 4.7B model found three of them and objected to every clean
/// rewrite as well, which is the same as finding nothing. An 8.9B and an 11.9B both
/// found three and objected once, to a commitment becoming an act — which is a fair
/// objection. The line therefore sits between them and not at a round number
/// somebody liked the look of.
pub const LEAST_PARAMETERS_TO_CHECK: f32 = 7.0;

pub fn can_check_a_rewrite(descriptor: &ModelDescriptor) -> bool {
    billions_of_parameters(descriptor).is_some_and(|size| size >= LEAST_PARAMETERS_TO_CHECK)
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
    /// What this machine actually has, in gigabytes, or none where it cannot be
    /// established.
    ///
    /// Read here rather than in the interface because the interface cannot. It asked
    /// `navigator.deviceMemory`, which WebKit does not implement and this shell is
    /// WebKit, so every macOS machine reported nothing, nothing was treated as the
    /// weakest supported machine, and the model picker recommended accordingly — a
    /// 16 GB laptop was offered a model measured at 20 figures against the
    /// baseline's 31.
    pub machine_memory_gb: Option<u32>,
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
    /// What a protocol of this style must contain, checkable in any language.
    pub expectations: Vec<crate::domain::StructuralExpectation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GenerationSegment {
    pub start_ms: u64,
    /// Empty where separation did not run, and then not sent at all. A transcript
    /// that labels every segment the same way says nothing about who spoke, and a
    /// model given it writes that label into the participants list as a person —
    /// twice, under two disciplines, on a real meeting.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub speaker: String,
    pub text: String,
}

/// What the model is told about one segment, and what it is not.
///
/// Narrower than the transcript's own segment by intention: the id, the end, the
/// review flag and the words the transcriber was unsure of are the editor's
/// business, and sending them would only be text the model has to ignore.
///
/// Six places copied these three fields across by hand. A seventh field added to
/// either type would have had to be remembered in all six.
impl From<&crate::domain::TranscriptSegment> for GenerationSegment {
    fn from(segment: &crate::domain::TranscriptSegment) -> Self {
        Self {
            start_ms: segment.start_ms,
            speaker: segment.speaker.clone(),
            text: segment.text.clone(),
        }
    }
}

/// The words LocaLog adds to a protocol when it has to say something about the
/// document itself.
///
/// Supplied by the interface rather than written here, and that is the whole point.
/// These are the only strings this module produces that a person reads in a finished
/// document, and the backend does not know which language the application is in.
/// Rust decides *that* a note is needed — three attempts produced no table, this
/// stretch of the meeting could not be read — and the interface supplies the words,
/// exactly as it does for every failure code.
///
/// Whole sentences rather than fragments, for the reason the dictionary gives: a
/// language that puts the parts in a different order cannot be served by assembling
/// them here. That is also why one gap and several gaps are two sentences instead of
/// a noun and a pronoun chosen separately.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentNotes {
    /// Heading for the note saying the protocol has no table of next steps.
    pub missing_table_heading: String,
    /// The paragraph under it.
    pub missing_table_body: String,
    /// Heading for the list of stretches the protocol does not cover.
    pub gaps_heading: String,
    /// The paragraph above that list, when exactly one stretch is missing.
    pub one_gap: String,
    /// The same, when more than one is.
    pub several_gaps: String,
}

#[cfg(test)]
impl DocumentNotes {
    /// English notes, for the harnesses.
    ///
    /// Here and not in the product because the product's words come from the
    /// dictionary. These exist so an evaluation run produces a protocol that reads
    /// like a real one — empty notes would print a bare `##` and nothing under it,
    /// which is worse output to score than either the real thing or no note at all.
    pub(crate) fn english_for_harnesses() -> Self {
        Self {
            missing_table_heading: "No table of next steps".into(),
            missing_table_body: "This protocol was written three times and none of them ended \
                 with a table of agreed tasks and their owners. Any actions the meeting agreed \
                 are described in the sections above but are not collected here."
                .into(),
            gaps_heading: "Not covered by this protocol".into(),
            one_gap: "One stretch of the recording could not be read, and nothing above \
                 describes it. The recording itself is complete and it can still be listened to."
                .into(),
            several_gaps: "Several stretches of the recording could not be read, and nothing \
                 above describes them. The recording itself is complete and these stretches can \
                 still be listened to."
                .into(),
        }
    }
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
    /// What to say if the document has to admit something about itself.
    pub document_notes: DocumentNotes,
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
/// What the model is given. Section names are deliberately not among them.
///
/// A `required_sections` field once held literal English strings — "Summary",
/// "Decisions" — while the protocol is written in the language of the meeting.
/// Putting them in this payload told a German-language model to produce four English
/// headings while the style instructions told it to organise by topic into numbered
/// sections, and the two fought. Measured on the real meeting, removing them took the
/// protocol from 2,747 characters and two headings to 17,393 and forty-one, and the
/// quantities it recorded from one of nineteen to fourteen.
///
/// The field itself is gone now, replaced by structural expectations that are checked
/// on the finished document rather than asked for in a prompt. Do not add section
/// names back here.
struct PromptPayload<'a> {
    meeting_language: &'a str,
    style_id: &'a str,
    style_revision: &'a str,
    instructions: &'a [String],
    vocabulary_revision: &'a str,
    vocabulary: &'a [String],
    transcript: &'a [GenerationSegment],
    /// What was wrong with the previous answer, on a retry only. Skipped when there
    /// is none, so a first request is byte-for-byte what it has always been.
    #[serde(skip_serializing_if = "Option::is_none")]
    correction: Option<&'a str>,
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
    /// What was wrong with the previous attempt, when there was one.
    #[serde(skip_serializing_if = "Option::is_none")]
    correction: Option<&'a str>,
}

/// One section of a protocol, written from the passages that discussed it.
#[cfg(test)]
#[derive(Serialize)]
struct TopicSectionPayload<'a> {
    meeting_language: &'a str,
    /// What this section is about, as the topic pass named it.
    topic: &'a str,
    /// Every topic in the meeting, in order, so this section knows what the others
    /// cover and does not re-explain them. Titles only: the point is to write around
    /// the neighbours, not to reproduce them.
    outline: &'a [String],
    /// Where this one sits in that outline.
    position: usize,
    /// About how many characters this section is worth, from its share of what was
    /// said. The failure this exists to prevent produced a document four times the
    /// length of a human protocol, with the coverage entirely correct.
    character_budget: usize,
    /// The passages that discussed it, at full resolution rather than as notes.
    transcript: &'a [GenerationSegment],
    instructions: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    correction: Option<&'a str>,
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

/// Somebody who said their own name near the start of a meeting.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Introduction {
    /// The name exactly as the transcript spells it, however wrongly. Wrong is the
    /// point: a person recognises "Person A" as themselves at once, and the
    /// spelling has to match the transcript for a correction to find it.
    pub heard: String,
    /// What they said they do, where they said it.
    #[serde(default)]
    pub role: String,
    /// The sentence they said it in, so a reader sees what was actually heard.
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Deserialize)]
struct StructuredIntroductions {
    introductions: Vec<Introduction>,
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

/// What a run cost, counted while it happened.
///
/// A protocol is written in several passes and each pass may be tried up to
/// ATTEMPTS_PER_STEP times, so the time a generation takes says little on its own:
/// eighteen minutes across four calls and eighteen minutes across forty are
/// different problems with the same clock reading, and nothing distinguished them.
///
/// Counted here rather than estimated afterwards, because the retries are the part
/// that does not show up anywhere else.
/// Measured, not shipped: the reader is the evaluation harness. Give it a reader
/// in the application — a line in the job's own record, say — and this comes out
/// of the test gate.
#[cfg(test)]
#[derive(Default)]
pub struct Spend {
    /// Round trips to the model, retries included.
    pub calls: usize,
    /// Characters sent, which is what the context window is spent on.
    pub prompt_chars: usize,
    /// Characters received.
    pub answer_chars: usize,
    /// Time inside the model, as distinct from time in this crate.
    pub model_millis: u64,
}

pub struct OllamaProvider {
    base_url: String,
    calls: AtomicUsize,
    prompt_chars: AtomicUsize,
    answer_chars: AtomicUsize,
    model_millis: AtomicU64,
    /// Discovery must answer quickly or be treated as unavailable.
    agent: ureq::Agent,
    /// Generation legitimately takes minutes: a cold model load alone costs seconds,
    /// and a long meeting is written in several passes. Bound the parts that indicate
    /// a dead server, and let cancellation between streamed chunks provide the rest.
    generation_agent: ureq::Agent,
}

/// How much memory this machine has, in gigabytes, or none where it cannot be read.
///
/// The interface used to ask the browser and was told nothing on every macOS machine,
/// because `navigator.deviceMemory` is not implemented in WebKit. Nothing was then
/// read as the weakest supported machine and the model picker recommended for one.
fn machine_memory_gb() -> Option<u32> {
    crate::machine::memory_gb()
}

impl OllamaProvider {
    pub fn loopback() -> Self {
        Self::at_port(DEFAULT_PORT)
    }

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
            calls: AtomicUsize::new(0),
            prompt_chars: AtomicUsize::new(0),
            answer_chars: AtomicUsize::new(0),
            model_millis: AtomicU64::new(0),
        }
    }

    /// What this provider has spent so far. One instance serves one generation, so
    /// this is the whole cost of writing a protocol, retries and all.
    #[cfg(test)]
    pub fn spend(&self) -> Spend {
        Spend {
            calls: self.calls.load(Ordering::Relaxed),
            prompt_chars: self.prompt_chars.load(Ordering::Relaxed),
            answer_chars: self.answer_chars.load(Ordering::Relaxed),
            model_millis: self.model_millis.load(Ordering::Relaxed),
        }
    }

    pub fn status(&self, selected_model: Option<String>) -> OllamaStatus {
        let mut status = OllamaStatus {
            endpoint: self.base_url.clone(),
            selected_model,
            machine_memory_gb: machine_memory_gb(),
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
                status.message = truncate(&error.to_string(), MESSAGE_LIMIT);
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
        let value: VersionResponse = response.body_mut().read_json().map_err(|error| {
            ProviderError::InvalidResponse(truncate(&error.to_string(), MESSAGE_LIMIT))
        })?;
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
        let value: TagsResponse = response.body_mut().read_json().map_err(|error| {
            ProviderError::InvalidResponse(truncate(&error.to_string(), MESSAGE_LIMIT))
        })?;
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
        let focused = drop_uninformative_speakers(focus_vocabulary(request));
        let request = &focused;

        // Said before the work starts rather than after three identical failures.
        // The person can shorten the recording, choose a terser style, or accept a
        // protocol that stops early — but only if they are told, and told now.
        if let Some(refusal) = beyond_one_answer(request) {
            progress(19, "protocol_would_not_fit")?;
            return Err(refusal);
        }

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
                    attempt: 0,
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
                attempt: 0,
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
    ///
    /// It now gets the same three attempts, the same fed-back correction and the same
    /// keep-what-arrived as the sectioned path. It had none of them until 29 August
    /// 2026, which put the only unprotected path under the shortest meeting — the ten
    /// minutes somebody records to try the application. The measured one draw in five
    /// that omits the actions table was corrected or kept on a real meeting and failed
    /// the whole run here, and it failed by showing the reader the correction text,
    /// which is an instruction written for a model.
    fn generate_in_one_pass(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<String> {
        let instructions = with_density(request);
        let (markdown, complete) = with_correction_or_keep(
            |correction, attempt| {
                let payload = PromptPayload {
                    meeting_language: &request.meeting_language,
                    style_id: &request.style.id,
                    style_revision: &request.style.revision,
                    instructions: &instructions,
                    vocabulary_revision: &request.vocabulary_revision,
                    vocabulary: &request.vocabulary,
                    transcript: &request.transcript,
                    correction,
                };
                let generated = self.complete(
                    request,
                    Completion {
                        system: PROTOCOL_SYSTEM,
                        prompt: &encode_prompt(&payload)?,
                        format: protocol_schema(),
                        num_predict: output_allowance(request),
                        attempt,
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
                Ok(tidy_protocol(&structured.protocol_markdown))
            },
            |markdown: &String| {
                validate_protocol(markdown, spoken_characters(request), &request.style)
            },
        )?;
        progress(78, "validating_protocol")?;

        // An answer kept after every retry failed must still be a protocol, exactly as
        // on the sectioned path: three tries without a table is a document with
        // something missing, and three JSON dumps is not a document at all.
        validate_markdown(&markdown, spoken_characters(request))?;
        Ok(if complete {
            markdown
        } else {
            note_missing_table(markdown, &request.document_notes)
        })
    }

    /// A real meeting exceeds the window. Each section is condensed first, then the
    /// protocol is written from the collected notes. Nothing is silently dropped:
    /// every segment belongs to exactly one section.
    /// Write the protocol one topic at a time, never holding the whole of it.
    ///
    /// The path this is measured against reads the meeting in sections, condenses
    /// each into notes, folds the notes until they fit, and writes the protocol in a
    /// single answer. That last step bounds the length of meeting the product can
    /// handle: the answer ceiling is a fixed number of tokens, so a meeting about
    /// twice the reference one cannot be written at all, on any machine.
    ///
    /// Here nothing holds the whole document. Each topic is written from the passages
    /// that discussed it, at full resolution rather than from notes — one lossy step
    /// instead of two — and the sections are joined without a model.
    ///
    /// The one previous attempt covered 23 of 24 figures and ran to four times the
    /// length of a human protocol. What it lacked was proportion, which is arithmetic
    /// rather than judgement: a topic holding a third of the segments is worth about a
    /// third of the document.
    #[cfg(test)]
    pub fn write_by_topic(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<String> {
        let (topics, unclaimed) = self.find_topics(request, cancelled, progress)?;
        if topics.is_empty() {
            return Err(ProviderError::InvalidResponse(
                "No subjects were found in this meeting.".into(),
            ));
        }
        let outline: Vec<String> = topics.iter().map(|topic| topic.title.clone()).collect();

        // What the whole protocol should come to, from the ratio a person actually
        // wrote: about 73,000 characters spoken became about 18,000 written.
        let spoken = spoken_characters(request);
        let target = spoken / PROTOCOL_SHARE_OF_SPEECH;
        let claimed: usize = topics.iter().map(|topic| topic.segments.len()).sum();
        if !unclaimed.is_empty() {
            progress(29, "segments_no_subject_claimed")?;
        }

        let instructions = with_density(request);
        let mut written = Vec::with_capacity(topics.len());
        let mut overran = 0usize;
        for (index, topic) in topics.iter().enumerate() {
            if cancelled.load(Ordering::Acquire) {
                return Err(ProviderError::Cancelled);
            }
            let start = 30 + (index as u64 * 45) / topics.len() as u64;
            progress(start, "writing_section")?;

            let passages: Vec<GenerationSegment> = topic
                .segments
                .iter()
                .filter_map(|&at| request.transcript.get(at).cloned())
                .collect();
            if passages.is_empty() {
                continue;
            }
            // This topic's share of what was said, and so of the document.
            let budget = (target * topic.segments.len() / claimed.max(1)).max(400);

            let (section, within) = with_correction_or_keep(
                |correction, attempt| {
                    let payload = TopicSectionPayload {
                        meeting_language: &request.meeting_language,
                        topic: &topic.title,
                        outline: &outline,
                        position: index + 1,
                        character_budget: budget,
                        transcript: &passages,
                        instructions: &instructions,
                        correction,
                    };
                    let prompt = encode_prompt(&payload)?;
                    // Deliberately far above what `within_budget` will accept. A
                    // hard cap set at the same threshold as a soft check truncates
                    // the answer instead of letting the check reject it, and a
                    // truncated answer cannot be corrected — the first version of
                    // this set both to twice the budget and so guaranteed that the
                    // correction it defines could never fire.
                    let num_predict = answer_budget(
                        request.context_tokens,
                        prompt.len(),
                        ((budget * ROOM_TO_OVERRUN / CHARS_PER_OUTPUT_TOKEN) as u32).max(1_024),
                    );
                    let generated = self.complete(
                        request,
                        Completion {
                            system: TOPIC_SECTION_SYSTEM,
                            prompt: &prompt,
                            format: protocol_schema(),
                            num_predict,
                            attempt,
                        },
                        cancelled,
                        &mut |_| progress(start, "writing_section"),
                    )?;
                    let structured: StructuredProtocol = parse_structured(&generated)?;
                    Ok(tidy_protocol(&structured.protocol_markdown))
                },
                |section: &String| within_budget(section, budget),
            )?;
            if !within {
                // Kept rather than lost. A section over its budget is a document
                // slightly longer than intended; a section discarded is a subject
                // missing from the meeting.
                overran += 1;
            }
            written.push(section);
        }

        progress(78, "validating_protocol")?;
        let protocol = written.join("\n\n");
        // The failure this whole scheme exists to prevent was the *total* — 74,000
        // characters against a human protocol of 18,000. That is the number worth
        // judging, and it is judged here rather than section by section.
        if protocol.len() > target * TOTAL_OVERRUN {
            return Err(ProviderError::InvalidResponse(format!(
                "The protocol came to {} characters against a target of about {target}, \
                 which is the failure writing by subject is prone to.",
                protocol.len()
            )));
        }
        if overran > 0 {
            progress(79, "sections_over_their_length")?;
        }
        Ok(protocol)
    }

    /// Who introduced themselves at the start of the meeting, as the transcript
    /// spells them.
    ///
    /// A first meeting has no names list, and asking somebody to write one from
    /// memory before they have heard the recording is asking for the wrong thing at
    /// the wrong time. Most meetings open with people saying who they are: on the
    /// reference meeting, ten of the twelve people its written protocol names
    /// introduce themselves inside eight minutes.
    ///
    /// Every name comes back wrong on a first meeting — "Person A",
    /// "Person B", "Johannes Halle von Kau, drei" — and that is what makes this
    /// work. Somebody who was there recognises each one instantly and is correcting
    /// a list rather than composing one, and the wrong spelling is what a correction
    /// has to match to find it in the transcript.
    ///
    /// It also reaches errors the candidate extractor cannot: that offers words the
    /// transcriber was unsure of, and it was perfectly confident about "Person C".
    pub(crate) fn find_introductions(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<Vec<Introduction>> {
        let opening: Vec<GenerationSegment> = request
            .transcript
            .iter()
            .filter(|segment| segment.start_ms < INTRODUCTIONS_WINDOW_MS)
            .cloned()
            .collect();
        if opening.is_empty() {
            return Ok(Vec::new());
        }
        progress(10, "reading_introductions")?;

        let payload = IntroductionsPayload {
            meeting_language: &request.meeting_language,
            transcript: &opening,
        };
        let prompt = encode_prompt(&payload)?;
        let generated = self.complete(
            request,
            Completion {
                system: INTRODUCTIONS_SYSTEM,
                prompt: &prompt,
                format: introductions_schema(),
                num_predict: answer_budget(request.context_tokens, prompt.len(), 2_048),
                attempt: 0,
            },
            cancelled,
            &mut |_| progress(10, "reading_introductions"),
        )?;
        let structured: StructuredIntroductions = parse_structured(&generated)?;

        // A name the transcript does not contain is one the model tidied on its way
        // out, and correcting it would find nothing to change.
        let spoken: String = opening
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(structured
            .introductions
            .into_iter()
            .filter(|found| !found.heard.trim().is_empty() && spoken.contains(found.heard.trim()))
            .collect())
    }

    fn generate_from_sections(
        &self,
        request: &GenerationRequest,
        sections: &[std::ops::Range<usize>],
        cancelled: &AtomicBool,
        progress: &mut dyn FnMut(u64, &str) -> Result<()>,
    ) -> Result<String> {
        let count = sections.len();
        let mut notes = Vec::with_capacity(count);
        // Stretches no amount of retrying could condense. Kept so the finished
        // protocol can say where they were, rather than closing over the hole.
        let mut gaps: Vec<Gap> = Vec::new();
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
                |correction, attempt| {
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
                            attempt,
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
            );
            // A section that never came back is a quarter of an hour of the meeting,
            // not the meeting. Losing the fifteen minutes either side of it as well
            // is the wrong trade, and a reader who is told where the hole is can go
            // and listen to that stretch. A reader given a protocol that quietly
            // skips it cannot, because nothing tells them to.
            let section_notes = match section_notes {
                Ok(notes) => notes,
                Err(error) if error.is_a_bad_draw() => {
                    let gap = Gap::across(&request.transcript[range.clone()], &error);
                    let placeholder = gap.as_note(&request.meeting_language);
                    gaps.push(gap);
                    placeholder
                }
                Err(error) => return Err(error),
            };
            notes.push(section_notes);
        }
        // Every section failing is a broken run, not a protocol full of holes.
        if gaps.len() == count {
            return Err(ProviderError::InvalidResponse(format!(
                "None of the {count} sections of this meeting could be condensed."
            )));
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
        let (markdown, complete) = with_correction_or_keep(
            |correction, attempt| {
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
                        attempt,
                    },
                    cancelled,
                    &mut |written| {
                        progress(arrived(70, 76, written, num_predict), "generating_protocol")
                    },
                )?;
                let structured: StructuredProtocol = parse_structured(&generated)?;
                Ok(tidy_protocol(&structured.protocol_markdown))
            },
            |markdown: &String| {
                validate_protocol(markdown, spoken_characters(request), &request.style)
            },
        )?;
        progress(78, "validating_protocol")?;

        // An answer kept after every retry failed must still be a protocol. Asking
        // three times for a table and not getting one is a document with something
        // missing; returning a JSON dump three times is not a document.
        validate_markdown(&markdown, spoken_characters(request))?;

        // A protocol without its table of tasks and owners is worth less than one
        // with it, and worth far more than the meeting it would otherwise take with
        // it. Measured, about one draw in five arrives without one, so discarding
        // those runs would lose whole meetings to a fault a reader can see and act
        // on. It is kept, and the document says what it is missing.
        let markdown = if complete {
            markdown
        } else {
            note_missing_table(markdown, &request.document_notes)
        };
        // Said again at the foot of the document, because the note asking for the
        // hole to be described is an instruction to a model and instructions to
        // models are sometimes not followed. This part does not depend on one.
        Ok(append_gap_notice(markdown, &gaps, &request.document_notes))
    }

    /// Combine a group of consecutive notes into one, preserving their content.
    /// A single note is returned unchanged rather than sent through the model again.
    ///
    /// Retried on its own like the passes either side of it. This was the one step
    /// without that protection, so a single bad draw here lost a run that had already
    /// condensed the whole meeting — and this step runs most often at the smallest
    /// contexts, where the notes need folding several times over. An eight-thousand
    /// token run failed exactly this way.
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
        let held: usize = group.iter().map(String::len).sum();
        with_correction(
            |correction, attempt| {
                let payload = MergePayload {
                    meeting_language: &request.meeting_language,
                    notes: group,
                    correction,
                };
                let prompt = encode_prompt(&payload)?;
                let notes_ceiling = ((prompt.len() / 2) as u32).max(1024);
                let num_predict =
                    answer_budget(request.context_tokens, prompt.len(), notes_ceiling);
                let generated = self.complete(
                    request,
                    Completion {
                        system: MERGE_SYSTEM,
                        prompt: &prompt,
                        format: notes_schema(),
                        num_predict,
                        attempt,
                    },
                    cancelled,
                    &mut |_| progress(60, "condensing_transcript"),
                )?;
                let structured: StructuredNotes = parse_structured(&generated)?;
                Ok(strip_code_fence(&structured.notes_markdown).to_string())
            },
            |merged: &String| validate_markdown(merged, held),
        )
    }

    /// One bounded, cancellable, streamed completion.
    /// Rewrite one passage of a protocol, as asked.
    ///
    /// The narrowest use of a model in this application: it is given the passage and
    /// nothing else — not the transcript, not the meeting — because the job is to
    /// say the same thing differently, and everything else it could see is something
    /// it could add.
    ///
    /// Fidelity is not negotiable here any more than it is in a style. A rewrite
    /// that rounds a figure or drops a name has changed the record rather than the
    /// prose, so the instruction says so plainly, and the caller keeps the original
    /// to put back.
    pub fn refine_passage(
        &self,
        request: &GenerationRequest,
        passage: &str,
        instruction: &str,
        cancelled: &AtomicBool,
    ) -> Result<String> {
        let payload = RefinePayload {
            meeting_language: &request.meeting_language,
            instruction,
            passage,
        };
        let prompt = encode_prompt(&payload)?;
        // Room for the passage again, half as much over for a rewrite that runs
        // long, and a floor so that a one-line selection still has somewhere to go.
        let budget = ((passage.len() / 2) as u32 + 256).min(request.maximum_output_tokens);
        let generated = self.complete(
            request,
            Completion {
                system: REFINE_SYSTEM,
                prompt: &prompt,
                // Prose, not a schema: this returns the passage and nothing round it.
                format: serde_json::Value::Null,
                num_predict: budget,
                attempt: 0,
            },
            cancelled,
            &mut |_| Ok(()),
        )?;

        let revised = tidy_refinement(&generated, passage);
        if revised.trim().is_empty() {
            return Err(ProviderError::InvalidResponse(
                "The model returned nothing for that passage.".into(),
            ));
        }
        Ok(revised)
    }

    /// Ask the model what the rewrite changed about the facts.
    ///
    /// A list of differences rather than a verdict, because a verdict is what this
    /// gets wrong: asked whether a fact changed, an 11.9B model wrote "The date was
    /// changed from September to October" and answered no in the same breath. Asked
    /// what differs, it says so.
    ///
    /// Never a gate. What comes back is shown beside a difference somebody is
    /// already reading, because it misses things too.
    pub fn check_rewrite(
        &self,
        request: &GenerationRequest,
        passage: &str,
        revised: &str,
        cancelled: &AtomicBool,
    ) -> Result<Vec<String>> {
        let payload = CheckPayload {
            original: passage,
            rewrite: revised,
        };
        let prompt = encode_prompt(&payload)?;
        let generated = self.complete(
            request,
            Completion {
                system: CHECK_SYSTEM,
                prompt: &prompt,
                format: serde_json::Value::Null,
                num_predict: 320,
                attempt: 0,
            },
            cancelled,
            &mut |_| Ok(()),
        )?;

        let answer = generated.trim();
        if answer.is_empty() || answer.eq_ignore_ascii_case("NOTHING") {
            return Ok(Vec::new());
        }
        Ok(answer
            .lines()
            .map(|line| line.trim().trim_start_matches(['-', '*', '•']).trim())
            .map(|line| line.replace("**", ""))
            .filter(|line| !line.is_empty())
            // The template repeated as a bare header, and the sign-off after the
            // real answers. Both are the model narrating its own form rather than
            // naming a difference, and both were in the first real run.
            .filter(|line| {
                let upper = line.to_uppercase();
                !upper.starts_with("NOTHING")
                    && upper.trim_end_matches([':', '.']) != "ORIGINAL -> REWRITE"
            })
            .take(6)
            .collect())
    }

    fn complete(
        &self,
        request: &GenerationRequest,
        call: Completion<'_>,
        cancelled: &AtomicBool,
        tick: &mut dyn FnMut(usize) -> Result<()>,
    ) -> Result<String> {
        // Whether the answer is prose or a JSON document, asked before the format
        // is handed to the request body.
        let prose = call.format.is_null();
        let call_prompt_chars = call.prompt.len() + call.system.len();
        let started = Instant::now();
        let body = OllamaRequest {
            model: &request.model,
            system: call.system,
            prompt: call.prompt,
            stream: true,
            think: false,
            format: call.format,
            options: OllamaOptions {
                seed: request.seed.wrapping_add(u64::from(call.attempt)),
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
            let read = reader.read_line(&mut line).map_err(|error| {
                ProviderError::Unavailable(truncate(&error.to_string(), MESSAGE_LIMIT))
            })?;
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
                ProviderError::InvalidResponse(truncate(&error.to_string(), MESSAGE_LIMIT))
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
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.prompt_chars
            .fetch_add(call_prompt_chars, Ordering::Relaxed);
        self.answer_chars
            .fetch_add(generated.len(), Ordering::Relaxed);
        self.model_millis
            .fetch_add(started.elapsed().as_millis() as u64, Ordering::Relaxed);

        // Repair prose only. A schema-shaped answer is a JSON document, and turning
        // its \n escapes into real newlines makes it invalid — measured: serde
        // answers "control character (\u0000-\u001F) found while parsing a string".
        // parse_structured then put them back through escape_raw_controls, which
        // exists for a different fault entirely, so every protocol with more than one
        // line took the repair path and the straight parse never once succeeded.
        //
        // Nothing is lost. serde decodes the escapes that were written correctly, and
        // tidy_protocol handles the ones a model wrote into the body by mistake, on
        // the markdown rather than on the envelope around it.
        if prose {
            Ok(repair_escaped_newlines(&generated))
        } else {
            Ok(generated)
        }
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
    /// Which try this is, counted from nought.
    ///
    /// It moves the seed, because otherwise a retry is not a second draw. A check
    /// that fails reports the same words every time — the three validate_protocol
    /// messages are fixed literals — so the correction folded into the next prompt
    /// is the same correction, and with the seed pinned at 42 the third attempt
    /// asks byte-for-byte what the second asked and can only differ by whatever the
    /// backend does not hold steady. Three attempts, one of them free at best.
    ///
    /// Nought for the first try, so an ordinary run is unchanged.
    attempt: u32,
}

const PROTOCOL_SYSTEM: &str = "Create a reviewable professional meeting protocol using only the supplied transcript. Never invent decisions, actions, owners, or dates. State uncertainty explicitly. Follow the controlled style and return only schema-valid JSON. If the payload carries a correction, your previous answer was rejected for the reason it gives; fix exactly that and return the whole answer again.";

const SECTION_SYSTEM: &str = "Record one section of a meeting transcript as detailed factual notes in the meeting's language. Completeness matters more than brevity: keep every topic discussed, every decision, every agreed action and its owner, every open question, and every number, measurement, area, date and proper name exactly as stated. Write one bullet per distinct point rather than merging several into a summary sentence. Do not add anything that was not said, and do not write a protocol yet. Return only schema-valid JSON. If the payload carries a correction, your previous answer was rejected for the reason it gives; fix exactly that and return the whole answer again.";

const SYNTHESIS_SYSTEM: &str = "Write a reviewable professional meeting protocol from ordered notes taken across the whole meeting. The notes are the only source. Never invent decisions, actions, owners, or dates, and state uncertainty explicitly. Group related material by topic rather than by the order it was discussed. Follow the controlled style and return only schema-valid JSON.";

#[cfg(test)]
const TOPIC_SECTION_SYSTEM: &str = "Write one section of a professional meeting protocol, in the meeting's language, from the passages supplied. They are the only source. Never invent decisions, actions, owners or dates, and reproduce every number, measurement, area, date and proper name exactly as stated. Write about this section's own subject only: the outline names what the other sections cover, and repeating them is a fault. Keep close to the character budget given — it is this subject's share of the whole protocol, and overrunning it is the failure this format exists to prevent. Begin with a heading for the subject. Do not write an introduction, a participants list or a table of actions; those are written separately. Return only schema-valid JSON. If the payload carries a correction, your previous answer was rejected for the reason it gives; fix exactly that and return the whole answer again.";

/// What a checking pass is told. It lists differences; it never judges.
const CHECK_SYSTEM: &str = "You are given the original of a passage from a meeting \
protocol and a rewrite of it. Name every fact the rewrite states differently from \
the original, or states that the original did not state, or leaves out. \
A fact is a number, measurement, date, deadline, name, organisation, room or floor, \
an abbreviation, or a stated commitment. \
Writing a fact differently counts: an abbreviation spelled out, a designation \
replaced, a number expressed another way. \
Rewording, reordering and different grammar are not facts — ignore them completely. \
List each one on its own line as: ORIGINAL -> REWRITE. \
If every fact is identical, reply with exactly: NOTHING";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckPayload<'a> {
    original: &'a str,
    rewrite: &'a str,
}

/// What a rewrite is told, which is mostly what it may not do.
const REFINE_SYSTEM: &str = "You revise one passage of a meeting protocol. \
Return only the revised passage: no preamble, no explanation, no quotation marks \
around it, and no note about what you changed. \
Write in the language of the passage. \
Reproduce every number, measurement, date, name and proper noun exactly as they \
appear. Never round them, never rephrase them, never leave one out. \
Never add information that is not already in the passage, and never remove a fact \
in order to make it read better. \
Keep the passage's own form: if it is a list item, return a list item; if it is a \
heading, return a heading. Do not add Markdown formatting that was not there.";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RefinePayload<'a> {
    meeting_language: &'a str,
    instruction: &'a str,
    passage: &'a str,
}

/// What a model wraps an answer in when it has been asked for only the answer.
///
/// Every one of these has been seen from a local model: a fenced block, a leading
/// "Here is the revised passage:", and the whole thing in quotation marks. Stripped
/// rather than retried, because the text inside is usually right.
fn tidy_refinement(generated: &str, passage: &str) -> String {
    let mut text = generated.trim();

    if let Some(rest) = text.strip_prefix("```") {
        let rest = rest.split_once('\n').map(|(_, body)| body).unwrap_or(rest);
        text = rest.trim_end().strip_suffix("```").unwrap_or(rest).trim();
    }

    // A preamble ends at the first line break; a passage that is genuinely one line
    // is left alone rather than beheaded.
    if let Some((first, rest)) = text.split_once('\n') {
        let opener = first.trim_end();
        if opener.ends_with(':') && opener.len() < 80 && !rest.trim().is_empty() {
            text = rest.trim();
        }
    }

    if text.len() > 1 && text.starts_with('"') && text.ends_with('"') {
        text = text[1..text.len() - 1].trim();
    }

    // Emphasis the passage did not have and did not ask for. The instruction says
    // not to add formatting; the first real run came back with the floor and the
    // date in bold anyway, which would have gone into the document as bold.
    if !passage.contains("**") && text.contains("**") {
        return text.replace("**", "");
    }

    text.to_string()
}

const INTRODUCTIONS_SYSTEM: &str = "List the people who introduce themselves in this opening of a meeting: somebody saying their own name, usually alongside what they do or who they work for. Give each one their role or organisation as they stated it, and the sentence they said it in. Copy each name exactly as it appears in the text, character for character, even where it is plainly misspelt — the text is a transcript, that spelling is what has to be corrected afterwards, and correcting it here destroys the only thing this list is for. Skip anybody who is merely mentioned rather than introducing themselves. Return only schema-valid JSON.";

/// The opening of a meeting, for reading who is in it.
#[derive(Serialize)]
struct IntroductionsPayload<'a> {
    meeting_language: &'a str,
    transcript: &'a [GenerationSegment],
}

/// How much of a meeting counts as its opening.
///
/// Eight minutes on the reference meeting holds ten of the twelve introductions and
/// about 6,500 characters, which is a small enough request to be free beside the work
/// around it. A meeting that introduces somebody at minute forty is not one this can
/// help, and the candidate extractor covers what it misses.
const INTRODUCTIONS_WINDOW_MS: u64 = 8 * 60 * 1000;

const MERGE_SYSTEM: &str = "Combine consecutive sets of meeting notes into one set, in the meeting's language. Keep every decision, action, owner, open question, number, measurement, date and proper name. Remove only exact repetition between the sets. Do not shorten anything that is stated once. Return only schema-valid JSON. If a correction field is present, the previous attempt was rejected for the reason it gives; fix that and return the notes again.";

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
/// A stretch of the meeting that could not be condensed, and where it was.
#[derive(Debug, Clone)]
pub(crate) struct Gap {
    from_ms: u64,
    to_ms: u64,
    reason: String,
}

impl Gap {
    fn across(segments: &[GenerationSegment], error: &ProviderError) -> Self {
        Self {
            from_ms: segments
                .first()
                .map(|segment| segment.start_ms)
                .unwrap_or(0),
            to_ms: segments.last().map(|segment| segment.start_ms).unwrap_or(0),
            reason: error.to_string(),
        }
    }

    /// Written into the notes so the model composing the protocol can see the hole
    /// and write around it honestly instead of inventing a bridge across it.
    fn as_note(&self, language: &str) -> String {
        format!(
            "> UNREADABLE SECTION — {} to {}. This stretch of the meeting could not be \
             condensed and its content is unknown. Say so plainly at this point in the \
             protocol, in {language}. Do not guess what was discussed here.",
            clock(self.from_ms),
            clock(self.to_ms)
        )
    }

    fn as_notice(&self) -> String {
        format!(
            "- {} – {} ({})",
            clock(self.from_ms),
            clock(self.to_ms),
            self.reason
        )
    }
}

/// Say that the protocol has no table of tasks and owners, when three attempts have
/// failed to produce one.
///
/// The reader is the one who can do something about it — they were at the meeting and
/// the model was not — but only if they are told. A protocol that quietly omits the
/// part somebody acts on reads exactly like one that had no actions to record.
fn note_missing_table(markdown: String, notes: &DocumentNotes) -> String {
    format!(
        "{}\n\n---\n\n## {}\n\n{}\n",
        markdown.trim_end(),
        notes.missing_table_heading,
        notes.missing_table_body
    )
}

/// State plainly, at the end of the protocol, which stretches of the meeting are
/// missing from it.
///
/// A protocol that silently omits a quarter of an hour reads exactly like one that
/// covers everything, and the reader has no way to tell them apart. This is the
/// difference between the two.
fn append_gap_notice(markdown: String, gaps: &[Gap], notes: &DocumentNotes) -> String {
    if gaps.is_empty() {
        return markdown;
    }
    let listed: Vec<String> = gaps.iter().map(Gap::as_notice).collect();
    format!(
        "{}\n\n---\n\n## {}\n\n{}\n\n{}\n",
        markdown.trim_end(),
        notes.gaps_heading,
        if gaps.len() == 1 {
            &notes.one_gap
        } else {
            &notes.several_gaps
        },
        listed.join("\n")
    )
}

/// Minutes and seconds, which is how somebody scrubs to a place in a recording.
fn clock(ms: u64) -> String {
    let total = ms / 1000;
    let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

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
    request
        .maximum_output_tokens
        .min(request.context_tokens / 2)
}

/// Characters of transcript a token of context carries.
///
/// Three functions declared this for themselves, and one of them wrote the safety
/// ratio out as a bare 7 / 10 where the other named it — so moving the constant
/// would have moved only one of the two budgets it governs.
const CHARS_PER_TOKEN: usize = 3;

/// How much of the window to actually fill. The rest is the margin between a
/// tokeniser's estimate and its behaviour.
const SAFETY_NUMERATOR: usize = 7;
const SAFETY_DENOMINATOR: usize = 10;

/// The tokens left over for everything the model has to read.
fn reading_window(request: &GenerationRequest) -> usize {
    request
        .context_tokens
        .saturating_sub(output_allowance(request)) as usize
}

fn plan_sections(request: &GenerationRequest) -> Vec<std::ops::Range<usize>> {
    // German tokenises to roughly three characters per token; the margin keeps a
    // long word or an unusual name from pushing a section over the edge.

    let total = request.transcript.len();
    if total == 0 {
        return Vec::new();
    }
    // The same sum synthesis_budget makes: the window's usable share, less
    // everything in the prompt that is not transcript.
    let available = synthesis_budget(request);
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
    let window = reading_window(request);
    let budget_chars = window * CHARS_PER_TOKEN * SAFETY_NUMERATOR / SAFETY_DENOMINATOR;
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
    const RESERVED_FOR_SYSTEM: u32 = 256;
    let prompt_tokens = (prompt_chars / CHARS_PER_TOKEN) as u32;
    let room = context_tokens
        .saturating_sub(prompt_tokens)
        .saturating_sub(RESERVED_FOR_SYSTEM);
    requested.min(room).max(256)
}

/// What a context leaves for each stage, without running anything.
///
/// Reports the numbers the planner and the budgets actually compute, so a question
/// about whether a window can hold a protocol is answered by arithmetic rather than
/// by inference from a failure. Used by the evaluation harness.
#[cfg(test)]
mod what_reaches_the_model {
    use super::{FIDELITY_RULES, GenerationRequest, GenerationStyle, with_density};
    use crate::domain::ProtocolDensity;

    fn request(density: ProtocolDensity, instructions: Vec<String>) -> GenerationRequest {
        GenerationRequest {
            document_notes: Default::default(),
            model: "test".into(),
            model_digest: String::new(),
            runtime_version: String::new(),
            meeting_language: "German".into(),
            style: GenerationStyle {
                id: "any".into(),
                revision: "1".into(),
                density,
                instructions,
                expectations: Vec::new(),
            },
            vocabulary_revision: String::new(),
            vocabulary: Vec::new(),
            transcript: Vec::new(),
            seed: 1,
            temperature_milli: 200,
            context_tokens: 8_192,
            maximum_output_tokens: 2_048,
        }
    }

    /// Every protocol carries the fidelity rules, whatever style wrote it — including
    /// one somebody authored here, which is the case the whole arrangement exists for.
    #[test]
    fn a_style_that_carries_nothing_still_gets_the_fidelity_rules() {
        let bare = with_density(&request(ProtocolDensity::Concise, Vec::new()));
        for rule in FIDELITY_RULES {
            assert!(
                bare.iter().any(|line| line == rule),
                "a protocol was written without: {rule}"
            );
        }
    }

    /// The order is deliberate: what to write, then what may never be done to it,
    /// then how long. Fidelity last of the three would read as an afterthought.
    #[test]
    fn the_style_speaks_first_and_the_length_last() {
        let full = with_density(&request(
            ProtocolDensity::Terse,
            vec!["Organise the protocol by topic.".into()],
        ));
        assert_eq!(
            full.first().map(String::as_str),
            Some("Organise the protocol by topic.")
        );
        assert_eq!(
            full.last().map(String::as_str),
            Some(ProtocolDensity::Terse.directive())
        );
        let fidelity_at = full
            .iter()
            .position(|line| line.starts_with("Never invent"))
            .expect("the fidelity rules are in there");
        assert!(fidelity_at > 0 && fidelity_at < full.len() - 1);
    }

    /// The instruction that moved: a terse protocol must not be told to write at
    /// whatever length it likes, which is what carrying it in the style did.
    #[test]
    fn a_terse_protocol_is_never_told_to_take_its_time() {
        let terse = with_density(&request(ProtocolDensity::Terse, Vec::new())).join(" ");
        assert!(!terse.contains("whatever length the material requires"));
        let full = with_density(&request(ProtocolDensity::Comprehensive, Vec::new())).join(" ");
        assert!(full.contains("whatever length the material requires"));
    }
}

#[cfg(test)]
mod the_table_a_style_asks_for {
    use super::{GenerationStyle, validate_protocol};
    use crate::domain::{ProtocolDensity, StructuralExpectation};

    fn with_table() -> String {
        format!("{BODY}{TABLE}")
    }

    fn style(expectations: Vec<StructuralExpectation>) -> GenerationStyle {
        GenerationStyle {
            id: "whatever-it-is-called".into(),
            revision: "1".into(),
            density: ProtocolDensity::Comprehensive,
            instructions: Vec::new(),
            expectations,
        }
    }

    /// Long enough to clear the floor a protocol must reach: `validate_markdown`
    /// refuses anything under two hundred characters as a non-answer.
    const BODY: &str = "# Protokoll der Sitzung\n\n## 1. Fassade\n\nDie Ausführung wurde \
        besprochen und die betroffene Fläche von 148,5 m² im zweiten Obergeschoss \
        bestätigt. Die Anpassungen im Bereich der Lüftung wurden aufgrund von \
        Änderungen seitens der Architekten vorgenommen, und Herr Planung hat zugesagt, \
        die Kostenspanne bis zum 12. September 2026 zu nennen.\n";

    const TABLE: &str = "\n## 2. Nächste Schritte\n\n| Aufgabe | Verantwortlich |\n\
        | --- | --- |\n| Angebot einholen | Frau Bauleitung |\n";

    /// The check now reads what the style requires rather than what it is called,
    /// which is what makes a duplicated style behave like the one it was copied from.
    #[test]
    fn a_style_that_asks_for_a_table_is_refused_a_protocol_without_one() {
        let asking = style(vec![StructuralExpectation::ActionTable]);
        // The transcript length only sets a floor on how short a protocol may be;
        // these fixtures are short, so the transcript they claim to come from is too.
        assert!(validate_protocol(BODY, 900, &asking).is_err());
        assert!(validate_protocol(&with_table(), 900, &asking).is_ok());
    }

    #[test]
    fn a_style_that_does_not_ask_is_content_either_way() {
        let quiet = style(Vec::new());
        assert!(validate_protocol(BODY, 900, &quiet).is_ok());
        assert!(validate_protocol(&with_table(), 900, &quiet).is_ok());
    }

    /// The name is no longer consulted, which is the whole point of the change.
    #[test]
    fn the_style_being_called_something_else_changes_nothing() {
        let mut named = style(vec![StructuralExpectation::ActionTable]);
        named.id = "style-formal".into();
        let mut copied = style(vec![StructuralExpectation::ActionTable]);
        copied.id = "style-01ABCDEF-a-copy".into();
        assert!(validate_protocol(BODY, 900, &named).is_err());
        assert!(
            validate_protocol(BODY, 900, &copied).is_err(),
            "a copy must be held to the same requirement as its original"
        );
    }
}

#[cfg(test)]
mod tidying_a_rewrite {
    use super::tidy_refinement;

    #[test]
    fn strips_the_wrapping_a_model_adds_when_asked_for_only_the_answer() {
        assert_eq!(
            tidy_refinement("```\nDie Fassade.\n```", "x"),
            "Die Fassade."
        );
        assert_eq!(
            tidy_refinement("Here is the revised passage:\nDie Fassade.", "x"),
            "Die Fassade."
        );
        assert_eq!(tidy_refinement("\"Die Fassade.\"", "x"), "Die Fassade.");
    }

    /// From the first run against a real model: it returned the floor and the date
    /// in bold, which the passage did not have and the instruction forbade.
    #[test]
    fn removes_emphasis_the_passage_never_had() {
        assert_eq!(
            tidy_refinement(
                "Im **2. OG** bis **12.09.2026**.",
                "Im 2. OG bis 12.09.2026."
            ),
            "Im 2. OG bis 12.09.2026."
        );
    }

    #[test]
    fn keeps_emphasis_the_passage_already_had() {
        assert_eq!(
            tidy_refinement("Frau **Bauleitung** nennt es.", "Frau **Bauleitung** sagt es."),
            "Frau **Bauleitung** nennt es."
        );
    }
}

#[cfg(test)]
mod checking_capability {
    use super::{ModelDescriptor, ModelDetails, billions_of_parameters, can_check_a_rewrite};

    fn model(parameter_size: &str) -> ModelDescriptor {
        ModelDescriptor {
            name: "test".into(),
            size: 0,
            digest: String::new(),
            details: ModelDetails {
                parameter_size: parameter_size.into(),
            },
        }
    }

    /// The line, and the measurements it sits on.
    ///
    /// Eight pairs of passages, four with a fact genuinely altered. `qwen3.5:4b`
    /// (4.7B) found three and objected to all four clean rewrites as well.
    /// `ministral-3:8b` (8.9B) and `gemma4:12b` (11.9B) each found three and
    /// objected once, to a commitment becoming an act — a fair objection. So the
    /// line goes between 4.7 and 8.9, and the numbers below are the models that
    /// were actually measured rather than a round figure.
    #[test]
    fn only_a_model_measured_to_be_useful_is_asked() {
        assert!(
            !can_check_a_rewrite(&model("4.7B")),
            "4.7B objects to everything"
        );
        assert!(
            can_check_a_rewrite(&model("8.9B")),
            "8.9B was measured useful"
        );
        assert!(
            can_check_a_rewrite(&model("11.9B")),
            "11.9B was measured useful"
        );
    }

    /// An older runtime says nothing about size, and a hint that might be noise is
    /// worse than no hint.
    #[test]
    fn a_model_that_will_not_say_its_size_is_not_asked() {
        assert_eq!(billions_of_parameters(&model("")), None);
        assert!(!can_check_a_rewrite(&model("")));
        assert!(!can_check_a_rewrite(&model("large")));
    }

    /// The fault this alias exists for.
    ///
    /// Ollama says `parameter_size`; this shape is sent to the interface in
    /// camelCase. Without the alias every model arrived sizeless and was judged too
    /// small, which looks exactly like "no capable model installed".
    #[test]
    fn reads_the_size_in_the_words_ollama_uses() {
        let descriptor: ModelDescriptor = serde_json::from_str(
            r#"{"name":"ministral-3:8b","size":6000000000,"digest":"abc",
                "details":{"family":"mistral3","parameter_size":"8.9B"}}"#,
        )
        .expect("Ollama's own shape parses");
        assert_eq!(billions_of_parameters(&descriptor), Some(8.9));
        assert!(can_check_a_rewrite(&descriptor));
    }

    #[test]
    fn reads_the_runtime_own_wording() {
        assert_eq!(billions_of_parameters(&model("8.9B")), Some(8.9));
        assert_eq!(billions_of_parameters(&model("70b")), Some(70.0));
    }
}

/// Only ever built for tests: it reads instructions that live in the eval harness.
#[cfg(test)]
pub(crate) fn sizing_probe(context_tokens: u32, maximum_output_tokens: u32) -> (usize, usize, u32) {
    let style = GenerationStyle {
        id: "style-formal".into(),
        revision: "probe".into(),
        density: crate::domain::ProtocolDensity::Comprehensive,
        instructions: crate::eval_harness::formal_minutes_instructions(),
        expectations: vec![crate::domain::StructuralExpectation::ActionTable],
    };
    // A transcript the size of the reference meeting, in segments its size.
    let transcript: Vec<GenerationSegment> = (0..675)
        .map(|index| GenerationSegment {
            start_ms: index as u64 * 7000,
            speaker: "Speaker 1".into(),
            text: "x".repeat(108),
        })
        .collect();
    let request = GenerationRequest {
        document_notes: Default::default(),
        model: "probe".into(),
        model_digest: "probe".into(),
        runtime_version: "probe".into(),
        meeting_language: "German".into(),
        style,
        vocabulary_revision: "probe".into(),
        vocabulary: Vec::new(),
        transcript,
        seed: 7,
        temperature_milli: 200,
        context_tokens,
        maximum_output_tokens,
    };
    let sections = plan_sections(&request).len();
    let notes_chars = synthesis_budget(&request);
    // The synthesis prompt is the folded notes plus the style and the scaffolding.
    let overhead: usize = request
        .style
        .instructions
        .iter()
        .map(|value| value.len() + 8)
        .sum::<usize>()
        + 512;
    let answer_tokens = answer_budget(
        context_tokens,
        notes_chars + overhead,
        maximum_output_tokens,
    );
    (sections, notes_chars, answer_tokens)
}

fn segment_chars(segment: &GenerationSegment) -> usize {
    // Speaker and timestamp are serialised alongside the text.
    segment.text.len() + segment.speaker.len() + 40
}

/// The people who said their own name, and what they said they do.
fn introductions_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["introductions"],
        "properties": {
            "introductions": {
                "type": "array",
                "items": {
                    "type": "object",
                    // All three required, because a name alone is not always enough
                    // to recognise: "Person C" could be anybody, and "Person C, die
                    // Planung Haus B und Fassadenplanung macht" is one person. Left
                    // optional first, and the model then returned none of them.
                    "required": ["heard", "role", "context"],
                    "properties": {
                        "heard": { "type": "string" },
                        "role": { "type": "string" },
                        "context": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
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
/// The rules no style may set, because they are not matters of style.
///
/// A style decides how a protocol is written. These decide whether it is true, and
/// a document that reports a decision nobody made is not a differently-styled
/// protocol — it is a wrong one. They are held here rather than in each style's own
/// list so that they cannot be edited away: authoring a style cannot reach them,
/// not because the interface forbids it but because they are not in the thing being
/// edited.
///
/// Every one of these was earned. The placeholder rule exists because drafts came
/// back with `[Datum]` in them; the coverage rule because a protocol that reads well
/// and silently drops a subject is the failure a reader cannot see.
pub const FIDELITY_RULES: &[&str] = &[
    "Write the entire protocol in the meeting's language.",
    "Reproduce every number, measurement, area, date, and proper name exactly as stated. Never round or approximate them.",
    "Never invent a decision, an action, an owner, or a date. If the source does not say who is responsible, leave it unattributed.",
    "Separate what was decided from what remains open. Where no decision was reached, say so plainly rather than implying one.",
    "Mark uncertainty in the words the meeting used, such as an intention, an estimate, or a matter still to be confirmed.",
    "Cover every topic that was discussed. A protocol that silently omits a topic is incomplete, even if what remains reads well.",
    "Never leave a placeholder such as [Datum] or [Details]. If something is not in the source, omit the line instead.",
];

/// A style's own instructions, the fidelity rules, and how much room to take.
///
/// The order is deliberate: what to write, then what may never be done to it, then
/// how long. Fidelity last of the three would be read as an afterthought.
fn with_density(request: &GenerationRequest) -> Vec<String> {
    let mut instructions = request.style.instructions.clone();
    instructions.extend(FIDELITY_RULES.iter().map(|rule| (*rule).to_string()));
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
/// Try, correct, and keep the last answer rather than losing it.
///
/// For a check whose failure is a matter of degree rather than of kind. A section
/// that will not come down to its budget is still the content of that section, and
/// discarding it takes the meeting with it: a per-section budget enforced strictly
/// aborted a twenty-three-minute run over 198 characters on a document of 26,000.
///
/// Returns whether the answer ended up acceptable, so a caller can say what happened
/// rather than pretending it did not.
fn with_correction_or_keep<T>(
    mut attempt: impl FnMut(Option<&str>, u32) -> Result<T>,
    mut check: impl FnMut(&T) -> Result<()>,
) -> Result<(T, bool)> {
    let mut correction: Option<String> = None;
    let mut last: Option<T> = None;
    for attempt_number in 0..ATTEMPTS_PER_STEP {
        let answer = match attempt(correction.as_deref(), attempt_number as u32) {
            Ok(answer) => answer,
            // A complete answer arrived on an earlier attempt and this retry went
            // wrong. Keeping it is the whole purpose of this function, and `?` here
            // discarded it — at the step whose own comment calls it the most expensive
            // to lose, because the entire meeting has already been condensed by the
            // time it runs. The trade is the same one the retry itself makes: an
            // imperfect protocol beats no protocol.
            //
            // Only a bad draw is survivable this way. A cancellation is an instruction
            // and must be obeyed; a missing model, a changed runtime or an unreachable
            // server fails the next attempt identically, so there is nothing to keep
            // trying for and the reason should reach the person unchanged.
            Err(error) if last.is_some() && error.is_a_bad_draw() => break,
            Err(error) => return Err(error),
        };
        match check(&answer) {
            Ok(()) => return Ok((answer, true)),
            Err(problem) => {
                correction = Some(problem.to_string());
                last = Some(answer);
            }
        }
    }
    match last {
        Some(answer) => Ok((answer, false)),
        None => Err(ProviderError::InvalidResponse(
            "The model could not produce a usable answer.".into(),
        )),
    }
}

fn with_correction<T>(
    mut attempt: impl FnMut(Option<&str>, u32) -> Result<T>,
    mut check: impl FnMut(&T) -> Result<()>,
) -> Result<T> {
    let mut correction: Option<String> = None;
    let mut last: Option<ProviderError> = None;
    for attempt_number in 0..ATTEMPTS_PER_STEP {
        let answer = attempt(correction.as_deref(), attempt_number as u32)?;
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

/// Characters of German a token carries, measured on this project's own drafts.
const CHARS_PER_OUTPUT_TOKEN: usize = 4;

/// Roughly what fraction of what was said ends up in a protocol of it.
///
/// The reference meeting spoke about 73,000 characters and its written protocol is
/// about 18,000, so a quarter. Deliberately an underestimate of the danger rather
/// than an overestimate: a style asking for full prose lands nearer this than a terse
/// one does, and warning somebody unnecessarily is worse than not warning them.
const PROTOCOL_SHARE_OF_SPEECH: usize = 4;

/// Whether a meeting is too long for a protocol of it to be written in one answer.
///
/// Not a property of the machine, which is what makes it worth saying out loud. The
/// answer ceiling is a fixed number of tokens, so it binds identically on a laptop
/// and on a workstation, and a meeting large enough to exceed it fails the same way
/// every time — three identical `IncompleteResponse` failures and a message about the
/// model stopping early, which sounds like a fault and is arithmetic.
///
/// Measured at 8,192 tokens of context, where the window binds before this does: all
/// three seeds failed identically. This is the same limit seen from the other end.
fn beyond_one_answer(request: &GenerationRequest) -> Option<ProviderError> {
    let expected = spoken_characters(request) / PROTOCOL_SHARE_OF_SPEECH;
    let ceiling = output_allowance(request) as usize * CHARS_PER_OUTPUT_TOKEN;
    if expected <= ceiling {
        return None;
    }
    Some(ProviderError::BeyondOneAnswer { expected, ceiling })
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

/// Tidy the things a model gets nearly right, rather than making it try again.
///
/// Measured across twenty-eight drafts from this project's evaluation runs, thirteen
/// carried at least one of these — more than carried any other defect, including the
/// missing action table. Every one of them reaches the reader as visible rubbish in
/// their document, and every one is the model's own scaffolding rather than a mistake
/// about the meeting.
///
/// Repairing beats rejecting here. The answer is right and its packaging is not, so a
/// retry would spend fifteen minutes reproducing the same content and might arrive
/// with the same wrapping.
///
/// A trailing backslash is deliberately left alone. It looks like leakage and is a
/// legitimate CommonMark hard line break, and this project has done enough damage
/// today by acting on things that merely looked wrong.
fn tidy_protocol(markdown: &str) -> String {
    let mut text = strip_code_fence(markdown).to_string();

    // An escaped newline written into the body rather than used as one. It reaches
    // the reader as the two characters, usually welding a bullet onto the line above.
    if text.contains("\\n") {
        text = text.replace("\\\\n", "\n").replace("\\n", "\n");
    }

    // Scaffolding after the protocol has ended: the closing braces of the JSON the
    // model was answering into, and any fence that came with them.
    if let Some(at) = text.rfind("```") {
        let tail = &text[at..];
        if tail.len() < 40 && !tail.contains('\n') {
            text.truncate(at);
        }
    }
    // The braces usually sit on the end of the last sentence rather than on a line of
    // their own — the draft that prompted this ended `...laufenden Planung."} }`. So
    // the suffix is what gets measured, not the line.
    //
    // A brace or a bracket has to be present. A protocol may legitimately end on a
    // closing quotation mark, and removing that would be a repair worse than the
    // fault.
    let scaffold = |glyph: char| matches!(glyph, '}' | ']' | '"' | ',' | ' ' | '`' | '\n');
    let kept = text.trim_end_matches(scaffold).len();
    let suffix = &text[kept..];
    if suffix.contains('}') || suffix.contains(']') {
        text.truncate(kept);
    }

    // A backslash ending a block, which is a hard line break with nothing to break
    // before it. Kept where it separates two lines of text, which is what it is for;
    // removed at the end of a paragraph or list item, where it does nothing in a
    // strict renderer and shows as a stray mark in a lenient one. Eight of them
    // appeared in one measured draft, all at the ends of bullets.
    let lines: Vec<&str> = text.lines().collect();
    let trimmed: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(at, line)| {
            let ends_a_block = lines.get(at + 1).is_none_or(|next| next.trim().is_empty());
            match line.strip_suffix('\\') {
                Some(without) if ends_a_block => without.trim_end().to_string(),
                _ => (*line).to_string(),
            }
        })
        .collect();
    text = trimmed.join("\n");

    // A heading marked twice, which happens when the model writes its own hashes into
    // a field that already carries them. Renders as literal hashes in the heading.
    let doubled = |line: &str| -> Option<String> {
        let rest = line.trim_start_matches('#');
        let hashes = line.len() - rest.len();
        let inner = rest.trim_start();
        if hashes == 0 || !inner.starts_with('#') {
            return None;
        }
        let after = inner.trim_start_matches('#').trim_start();
        Some(format!("{} {}", "#".repeat(hashes), after))
    };
    let repaired: Vec<String> = text
        .lines()
        .map(|line| doubled(line).unwrap_or_else(|| line.to_string()))
        .collect();
    repaired.join("\n").trim().to_string()
}

/// How far past its budget a section may run before it is rejected and asked again.
#[cfg(test)]
const ALLOWED_OVERRUN: usize = 2;

/// How far past its budget a section is *allowed to finish writing*, so that an
/// overrun arrives whole and can be corrected rather than arriving cut off.
///
/// Must be comfortably larger than `ALLOWED_OVERRUN`. They were equal once and the
/// result was that every overrunning section was truncated by the token cap before
/// the check could reject it, which made the correction unreachable.
#[cfg(test)]
const ROOM_TO_OVERRUN: usize = 5;

/// How far past the target the whole protocol may run before the attempt is called a
/// failure. This is the level the original fault lived at: 74,000 characters against
/// a human protocol of 18,000, which is four times over. A section being half again
/// its own budget is not that, and treating it as though it were cost a run.
#[cfg(test)]
const TOTAL_OVERRUN: usize = 3;

/// Sized against both measurements on the reference meeting, whose written protocol
/// is about 18,000 characters: the 26,100 that per-topic writing with budgets
/// produced must pass, and the 74,000 it produced without them must not.
#[cfg(test)]
const _: () = {
    assert!(26_100 < 18_000 * TOTAL_OVERRUN);
    assert!(74_000 > 18_000 * TOTAL_OVERRUN);
};

/// The cap must sit above the check, or an overrunning section arrives truncated
/// rather than correctable. Checked when this compiles rather than when it runs,
/// because the two constants read as interchangeable and are not: they were equal
/// once, and it cost a 797-second run and made the correction beside them
/// unreachable.
#[cfg(test)]
const _: () = assert!(ROOM_TO_OVERRUN > ALLOWED_OVERRUN);

/// Remove speaker labels that distinguish nobody.
///
/// Separation may not have run, or may have failed, and then every segment carries
/// the same label. That is not evidence of a single speaker — it is an absence of
/// evidence — but a model reading it cannot tell the difference, and on a real
/// meeting wrote "Speaker 1" into the participants list as a person, once under
/// electrical planning and once under fire safety.
///
/// Telling the model in the style that a label is not a name was tried and did not
/// work: it does not supply the information the model is missing, so the model went
/// on naming the label in a different format. Not sending it does.
fn drop_uninformative_speakers(mut request: GenerationRequest) -> GenerationRequest {
    let distinct = request
        .transcript
        .iter()
        .map(|segment| segment.speaker.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    if distinct <= 1 {
        for segment in &mut request.transcript {
            segment.speaker.clear();
        }
    }
    request
}

/// Reject a section that grossly overruns the length it was given.
///
/// Generous, because a model asked for 900 characters will not land on 900 and should
/// not be made to try. What this catches is the failure that produced a document four
/// times the length of a human protocol: a section that ignored its budget entirely
/// rather than one that missed it.
#[cfg(test)]
fn within_budget(section: &str, budget: usize) -> Result<()> {
    if section.len() > budget * ALLOWED_OVERRUN {
        return Err(ProviderError::InvalidResponse(format!(
            "This section is {} characters and its budget is about {budget}. It is one \
             of several sections of one protocol, not a document of its own. Write the \
             same content in about {budget} characters.",
            section.len()
        )));
    }
    Ok(())
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
/// Whether a finished protocol must carry a table of tasks and owners.
///
/// **This is a placeholder for something a style should declare.** Styles are meant to
/// be authored by the people using them — decision 7 — and an authored style would
/// silently escape this check, which is the opposite of what a firm authoring its own
/// protocol shape would want. Only the shipped style exists today and nothing can
/// create another, so keying on it is honest about what is true now and wrong about
/// where this is going. It must be replaced when authoring arrives.
///
/// Now read from the style rather than from its name.
///
/// It used to be `style.id == "style-formal"`, which was honest about what was true
/// then and became wrong the moment a style could be copied: a duplicate has a new
/// id, so the copy of the formal style would have quietly stopped requiring the
/// table its original demands.
///
/// The field this replaced — `required_sections` — held English section names while
/// the protocol is written in the meeting's language, so matching "Actions" against
/// "Aufgaben" needed something this application does not have. It was never checked
/// anywhere and has been removed rather than left to look load-bearing. A table needs
/// no translating.
fn wants_an_action_table(style: &GenerationStyle) -> bool {
    style
        .expectations
        .contains(&crate::domain::StructuralExpectation::ActionTable)
}

/// Whether the document contains a markdown table.
///
/// Judged by the delimiter row, which is the one line a table cannot do without and
/// which no prose produces by accident. Counting pipe characters would match a
/// sentence that happened to use one.
fn has_a_table(markdown: &str) -> bool {
    markdown.lines().any(|line| {
        let line = line.trim();
        line.starts_with('|')
            && line.contains("--")
            && line
                .chars()
                .all(|glyph| matches!(glyph, '|' | '-' | ':' | ' '))
    })
}

/// Everything `validate_markdown` checks, plus what this particular style promised.
///
/// Measured on the reference meeting, roughly one draw in five returns a protocol with
/// no table of tasks and owners, having been told twice to end with one. That is the
/// omission a reader notices first, because it is the part of a protocol they act on,
/// and nothing caught it. The message is written as an instruction because it is fed
/// back to the model as the correction.
fn validate_protocol(
    markdown: &str,
    transcript_chars: usize,
    style: &GenerationStyle,
) -> Result<()> {
    validate_markdown(markdown, transcript_chars)?;
    if wants_an_action_table(style) && !has_a_table(markdown) {
        return Err(ProviderError::InvalidResponse(
            "The protocol has no table of agreed next steps. End it with a markdown \
             table of two columns, the task and the responsible party, listing every \
             action that was agreed."
                .into(),
        ));
    }
    Ok(())
}

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
    ProviderError::Unavailable(truncate(&error.to_string(), MESSAGE_LIMIT))
}

/// How much of a failure's own words to keep.
///
/// Long enough for a sentence from ureq or serde to say what went wrong, short
/// enough that it cannot carry a paragraph of a meeting into a status line. It was
/// written out as a bare 280 at six places.
const MESSAGE_LIMIT: usize = 280;

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {

    /// Why the escaped-newline repair is prose-only.
    ///
    /// It used to run on everything `complete()` returned, and most of what it
    /// returns is a JSON document. Turning a `\n` escape inside a JSON string into
    /// a real newline makes the document invalid, so `parse_structured` fell through
    /// to `escape_raw_controls` — which exists for models that emit raw control
    /// characters, not for damage done on the way in. Every protocol with more than
    /// one line went that way, and the straight parse never once succeeded.
    ///
    /// If this assertion ever fails, the repair has become harmless and `complete()`
    /// may stop asking whether the answer is prose.
    #[test]
    fn the_newline_repair_would_break_a_structured_answer() {
        let envelope = r#"{"protocol_markdown":"First line\nSecond line"}"#;
        serde_json::from_str::<serde_json::Value>(envelope)
            .expect("what a model actually returns is valid JSON");

        let repaired = repair_escaped_newlines(envelope);
        assert!(
            serde_json::from_str::<serde_json::Value>(&repaired).is_err(),
            "repairing the envelope no longer breaks it"
        );
    }
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
            |correction, attempt| {
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
    /// A retry has to be a second draw, not the same one again.
    ///
    /// The three checks report fixed sentences, so the correction folded into the
    /// next prompt is the same correction; with the seed pinned, attempt three asked
    /// byte-for-byte what attempt two asked and could differ only by whatever the
    /// backend does not hold steady. The attempt number moves the seed, so each try
    /// is a genuinely different draw — and the first is nought, so an ordinary run
    /// that passes its check the first time is unchanged.
    #[test]
    fn each_try_is_a_different_draw_and_the_first_is_unchanged() {
        let seen = std::cell::RefCell::new(Vec::new());
        let outcome = with_correction(
            |_, attempt| {
                seen.borrow_mut().push(attempt);
                Ok(String::new())
            },
            |markdown: &String| validate_markdown(markdown, 0),
        );
        assert!(outcome.is_err());
        assert_eq!(*seen.borrow(), vec![0, 1, 2]);
    }

    #[test]
    fn asking_forever_is_not_the_alternative() {
        let attempts = std::cell::RefCell::new(0);
        let outcome = with_correction(
            |_, _| {
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

    /// Taken verbatim from a draft: the escape reaches the reader as two characters
    /// and welds the next bullet onto the line above it.
    #[test]
    fn an_escaped_newline_written_into_the_body_becomes_one() {
        let leaked = "teilweise zu schließen.\\n*   **Erdgeschoss:** Die Erschließung";
        let tidied = tidy_protocol(leaked);
        assert!(!tidied.contains("\\n"), "{tidied}");
        assert!(
            tidied.contains("zu schließen.\n*   **Erdgeschoss:**"),
            "{tidied}"
        );
    }

    #[test]
    fn a_doubly_escaped_newline_is_also_a_newline() {
        let tidied = tidy_protocol("Förderfähigkeit.\\\\n*   Für die Wohnung");
        assert!(!tidied.contains('\\'), "{tidied}");
    }

    /// Also verbatim: a draft that ended with the closing braces of the JSON the
    /// model was answering into, and the fence that came with them.
    #[test]
    fn scaffolding_after_the_end_of_the_protocol_is_removed() {
        let with_tail = "## 9. Nächste Schritte\n\n*   Die Abstimmung erfolgt.\"} }```json";
        let tidied = tidy_protocol(with_tail);
        assert!(tidied.ends_with("Die Abstimmung erfolgt."), "{tidied}");
        assert!(!tidied.contains("```"), "{tidied}");
    }

    #[test]
    fn a_heading_marked_twice_is_marked_once() {
        let doubled = "## ### 1. Grundrissgestaltung\n\nInhalt.\n\n### ## 2. Fassade";
        let tidied = tidy_protocol(doubled);
        assert!(tidied.contains("## 1. Grundrissgestaltung"), "{tidied}");
        assert!(!tidied.contains("## ###"), "{tidied}");
        assert!(!tidied.contains("### ##"), "{tidied}");
    }

    /// A trailing backslash between two lines of text is a hard line break and means
    /// something. This is why they are not all stripped.
    #[test]
    fn a_hard_line_break_is_left_where_the_model_put_it() {
        let hard_break = "Erste Zeile.\\\nZweite Zeile.";
        assert_eq!(tidy_protocol(hard_break), hard_break);
    }

    /// At the end of a block it breaks nothing. Eight appeared in one measured draft,
    /// every one at the end of a bullet, where a strict renderer shows nothing and a
    /// lenient one shows a stray mark.
    #[test]
    fn a_backslash_ending_a_block_is_removed() {
        let ending = "* Ein Punkt.\\\n\n## Nächster Abschnitt";
        let tidied = tidy_protocol(ending);
        assert!(tidied.starts_with("* Ein Punkt."), "{tidied}");
        assert!(!tidied.contains("Punkt.\\"), "{tidied}");
        assert!(tidied.contains("## Nächster Abschnitt"));
    }

    #[test]
    fn a_backslash_ending_the_document_is_removed() {
        assert_eq!(tidy_protocol("Der letzte Satz.\\"), "Der letzte Satz.");
    }

    #[test]
    fn a_clean_protocol_survives_tidying_unchanged() {
        let clean =
            "# Protokoll\n\n## 1. Thema\n\nInhalt.\n\n| Aufgabe | Wer |\n|---|---|\n| Tun | KSP |";
        assert_eq!(tidy_protocol(clean), clean);
    }

    /// A protocol may legitimately contain a fenced block; only a stray fence at the
    /// very end, with nothing after it, is scaffolding.
    #[test]
    fn a_real_fenced_block_inside_the_protocol_is_kept() {
        let with_block = "# P\n\n```text\nAblauf\n```\n\n## 2. Weiter\n\nInhalt.";
        let tidied = tidy_protocol(with_block);
        assert!(tidied.contains("```text"), "{tidied}");
        assert!(tidied.contains("Ablauf"), "{tidied}");
    }

    /// The failure this exists to catch: measured, about one draw in five returned a
    /// protocol with no table of tasks and owners, having been told twice to end with
    /// one. It is the part of the document a reader acts on.
    #[test]
    fn a_formal_protocol_without_its_action_table_is_refused() {
        let style = formal_style();
        let without = "# Protokoll\n\n## 1. Fassade\n\nEs wurde besprochen.\n";
        let error = validate_protocol(without, 0, &style).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("table"), "{message}");
        // The message is fed back to the model, so it has to say what to do.
        assert!(message.contains("End it with"), "{message}");
    }

    #[test]
    fn a_formal_protocol_with_its_action_table_passes() {
        let style = formal_style();
        let with = "# Protokoll\n\n## 9. Nächste Schritte\n\n\
                    | Aufgabe | Zuständigkeit |\n|---|---|\n| Grundrisse fixieren | KSP |\n";
        assert!(validate_protocol(with, 0, &style).is_ok());
    }

    /// A table is a table in any language, which is the whole reason the check is
    /// structural rather than a list of section names.
    #[test]
    fn the_table_check_does_not_depend_on_language() {
        assert!(has_a_table(
            "| Task | Owner |\n| --- | --- |\n| Do it | Me |"
        ));
        assert!(has_a_table(
            "| Aufgabe | Zuständigkeit |\n|:---|---:|\n| Tun | Ich |"
        ));
    }

    /// Prose that happens to contain a pipe is not a table, and a delimiter row is
    /// the one line a table cannot do without.
    #[test]
    fn prose_containing_a_pipe_is_not_mistaken_for_a_table() {
        assert!(!has_a_table("Die Wand | die Stütze war das Thema."));
        assert!(!has_a_table("| Aufgabe | Zuständigkeit |"));
        assert!(!has_a_table("Ein Strich --- steht hier allein."));
    }

    fn formal_style() -> GenerationStyle {
        GenerationStyle {
            id: "style-formal".into(),
            revision: "formal-minutes@2".into(),
            density: crate::domain::ProtocolDensity::Comprehensive,
            instructions: Vec::new(),
            expectations: vec![crate::domain::StructuralExpectation::ActionTable],
        }
    }

    /// The reference meeting, which fits: about 73,000 characters spoken, a protocol
    /// of about 18,000, and an answer that holds about 32,000.
    #[test]
    fn a_meeting_that_fits_in_one_answer_is_not_warned_about() {
        // 675 segments of about 105 characters is the reference meeting's shape:
        // roughly 71,000 characters spoken, a protocol of about 18,000.
        let mut request = synthetic_request(675, 21);
        request.context_tokens = 40_960;
        request.maximum_output_tokens = 8_192;
        assert!(beyond_one_answer(&request).is_none());
    }

    /// The case the owner raised: somebody records a meeting several times longer.
    /// It fails identically on every machine, because the ceiling is tokens.
    #[test]
    fn a_meeting_too_long_for_one_answer_is_named_before_the_work_starts() {
        // Twice the reference meeting, which is where the ceiling starts to bite.
        let mut request = synthetic_request(675, 45);
        request.context_tokens = 40_960;
        request.maximum_output_tokens = 8_192;

        let Some(ProviderError::BeyondOneAnswer { expected, ceiling }) =
            beyond_one_answer(&request)
        else {
            panic!("a meeting this long cannot fit in one answer");
        };
        // The numbers rather than the sentence. The sentence is the interface's to
        // write now, and in whichever language it is in; what has to be right here is
        // the arithmetic, and it is also the only part a person can act on — being
        // told "too long" without them cannot say whether to trim a minute or half
        // the meeting.
        assert!(
            expected > ceiling,
            "{expected} characters wanted against a ceiling of {ceiling}"
        );
        // 8,192 tokens requested, and the window is wide enough not to bind first.
        assert_eq!(ceiling, 8_192 * CHARS_PER_OUTPUT_TOKEN);
    }

    /// A bigger machine does not help, which is the point worth making to somebody
    /// who would otherwise go looking for more memory.
    #[test]
    fn a_wider_context_does_not_raise_the_answer_ceiling() {
        let mut narrow = synthetic_request(675, 45);
        narrow.context_tokens = 16_384;
        narrow.maximum_output_tokens = 8_192;
        let mut wide = narrow.clone();
        wide.context_tokens = 131_072;

        assert!(beyond_one_answer(&narrow).is_some());
        assert!(
            beyond_one_answer(&wide).is_some(),
            "the ceiling is the answer, not the window"
        );
    }

    /// A section that will not shorten is still that section's content. Losing it
    /// takes a subject out of the meeting; keeping it makes the document slightly
    /// longer than intended. The second is obviously the better trade, and the first
    /// is what the code did.
    #[test]
    fn a_section_that_will_not_shorten_is_kept_rather_than_lost() {
        let mut tries = 0;
        let (answer, within) = with_correction_or_keep(
            |_correction, attempt| {
                tries += 1;
                Ok("x".repeat(1_922))
            },
            |section: &String| within_budget(section, 862),
        )
        .expect("an answer that overruns is still an answer");

        assert_eq!(
            tries, ATTEMPTS_PER_STEP,
            "it should have tried to correct it"
        );
        assert!(!within, "and should report that it did not succeed");
        assert_eq!(answer.len(), 1_922, "while keeping what it got");
    }

    #[test]
    fn a_section_that_shortens_when_asked_reports_success() {
        let mut tries = 0;
        let (answer, within) = with_correction_or_keep(
            |_correction, attempt| {
                tries += 1;
                Ok("x".repeat(if tries == 1 { 9_000 } else { 900 }))
            },
            |section: &String| within_budget(section, 1_000),
        )
        .unwrap();

        assert_eq!(tries, 2);
        assert!(within);
        assert_eq!(answer.len(), 900);
    }

    #[test]
    fn a_section_within_its_budget_is_accepted() {
        assert!(within_budget(&"x".repeat(900), 1_000).is_ok());
        assert!(
            within_budget(&"x".repeat(1_900), 1_000).is_ok(),
            "some slack is fine"
        );
    }

    #[test]
    fn a_section_that_ignored_its_budget_is_told_what_to_do() {
        let error = within_budget(&"x".repeat(9_000), 1_000).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("9000 characters"), "{message}");
        assert!(message.contains("about 1000"), "{message}");
        assert!(message.contains("not a document of its own"), "{message}");
    }

    /// The consistency this restores: gaps are marked and kept, over-long sections
    /// are kept and counted, and a protocol that never grew its table was thrown
    /// away along with the meeting it recorded.
    #[test]
    fn a_protocol_that_never_grew_its_table_is_kept_and_says_so() {
        let without = "# Protokoll\n\n## 1. Fassade\n\nEs wurde besprochen.";
        let noted = note_missing_table(without.into(), &DocumentNotes::english_for_harnesses());

        assert!(
            noted.starts_with("# Protokoll"),
            "the protocol itself is kept"
        );
        assert!(noted.contains("No table of next steps"));
        assert!(
            noted.contains("written three times"),
            "and says why: {noted}"
        );
    }

    /// The notes are the only words this module writes that a person reads in a
    /// finished document, and they come from the interface for the same reason every
    /// failure sentence does: this side does not know which language the application
    /// is in. A German set has to produce a German note with nothing English left in
    /// it — the test that would have caught these strings being hard-coded.
    #[test]
    fn the_notes_a_document_carries_are_written_in_the_words_it_was_given() {
        let german = DocumentNotes {
            missing_table_heading: "Keine Tabelle der nächsten Schritte".into(),
            missing_table_body: "Dieses Protokoll wurde dreimal geschrieben.".into(),
            gaps_heading: "Nicht in diesem Protokoll enthalten".into(),
            one_gap: "Ein Abschnitt der Aufnahme konnte nicht ausgewertet werden.".into(),
            several_gaps: "Mehrere Abschnitte konnten nicht ausgewertet werden.".into(),
        };

        let noted = note_missing_table("# Protokoll\n\nInhalt.".into(), &german);
        assert!(noted.contains("## Keine Tabelle der nächsten Schritte"));
        assert!(noted.contains("Dieses Protokoll wurde dreimal geschrieben."));
        assert!(
            !noted.contains("next steps") && !noted.contains("three times"),
            "nothing of the English is left: {noted}"
        );

        // Singular and plural are two whole sentences rather than a noun and a
        // pronoun picked apart, because the parts do not line up between languages.
        let one = Gap {
            from_ms: 60_000,
            to_ms: 90_000,
            reason: "unreadable".into(),
        };
        let two = Gap {
            from_ms: 120_000,
            to_ms: 150_000,
            reason: "unreadable".into(),
        };
        let single = append_gap_notice("# Protokoll".into(), std::slice::from_ref(&one), &german);
        assert!(single.contains("Ein Abschnitt der Aufnahme"));
        assert!(single.contains("1:00 – 1:30"), "and where it was: {single}");

        let several = append_gap_notice("# Protokoll".into(), &[one, two], &german);
        assert!(several.contains("Mehrere Abschnitte"));
        assert!(!several.contains("Ein Abschnitt der Aufnahme"));
    }

    /// A kept answer still has to be a protocol. Three JSON dumps are not a document
    /// with something missing; they are not a document.
    #[test]
    fn keeping_an_answer_does_not_mean_keeping_anything() {
        let spoken = 73_000;
        let dump = r#"{"metadata": {"title": "x"}, "organisations": []}"#;
        assert!(validate_markdown(dump, spoken).is_err());
        assert!(validate_markdown("", spoken).is_err());
    }

    /// The fault: a transcript labelling every segment "Speaker 1" is an absence of
    /// evidence about who spoke, and a model reading it wrote that label into the
    /// participants list as a person, under two different disciplines.
    #[test]
    fn a_label_that_distinguishes_nobody_is_not_sent() {
        let mut request = synthetic_request(4, 5);
        for segment in &mut request.transcript {
            segment.speaker = "Speaker 1".into();
        }
        let cleaned = drop_uninformative_speakers(request);

        assert!(cleaned.transcript.iter().all(|s| s.speaker.is_empty()));
        let sent = serde_json::to_string(&cleaned.transcript).unwrap();
        assert!(!sent.contains("Speaker 1"), "{sent}");
        assert!(
            !sent.contains("speaker"),
            "the field itself goes too: {sent}"
        );
    }

    /// Where separation did run, the labels are evidence and are kept.
    #[test]
    fn labels_that_tell_speakers_apart_are_kept() {
        let mut request = synthetic_request(4, 5);
        for (index, segment) in request.transcript.iter_mut().enumerate() {
            segment.speaker = format!("Speaker {}", index % 3 + 1);
        }
        let cleaned = drop_uninformative_speakers(request);

        assert!(cleaned.transcript.iter().all(|s| !s.speaker.is_empty()));
        assert!(
            serde_json::to_string(&cleaned.transcript)
                .unwrap()
                .contains("Speaker 2")
        );
    }

    #[test]
    fn a_transcript_with_no_segments_is_left_alone() {
        let request = synthetic_request(0, 0);
        assert!(drop_uninformative_speakers(request).transcript.is_empty());
    }

    #[test]
    fn a_protocol_with_no_gaps_is_left_exactly_as_written() {
        let markdown = "# Protokoll\n\n## 1. Thema\n\nInhalt.\n".to_string();
        assert_eq!(
            append_gap_notice(markdown.clone(), &[], &DocumentNotes::english_for_harnesses()),
            markdown
        );
    }

    /// The failure this exists to prevent: fifteen minutes missing from a document
    /// that reads as though it covers everything.
    #[test]
    fn a_missing_stretch_is_named_at_the_foot_of_the_protocol() {
        let gaps = vec![Gap {
            from_ms: 1_500_000,
            to_ms: 2_400_000,
            reason: "the model stopped answering".into(),
        }];
        let out = append_gap_notice(
            "# Protokoll\n\nInhalt.".into(),
            &gaps,
            &DocumentNotes::english_for_harnesses(),
        );

        assert!(
            out.starts_with("# Protokoll"),
            "the protocol itself is kept"
        );
        assert!(out.contains("Not covered by this protocol"));
        assert!(out.contains("25:00"), "the start is scrubbable: {out}");
        assert!(out.contains("40:00"), "the end is scrubbable: {out}");
        assert!(out.contains("the model stopped answering"));
    }

    #[test]
    fn several_missing_stretches_all_appear() {
        let gaps = vec![
            Gap {
                from_ms: 0,
                to_ms: 60_000,
                reason: "a".into(),
            },
            Gap {
                from_ms: 3_600_000,
                to_ms: 3_720_000,
                reason: "b".into(),
            },
        ];
        let out = append_gap_notice(
            "# P".into(),
            &gaps,
            &DocumentNotes::english_for_harnesses(),
        );
        assert!(out.contains("0:00 – 1:00"));
        assert!(out.contains("1:00:00 – 1:02:00"), "past an hour: {out}");
        assert!(out.contains("Several stretches"));
    }

    /// A bad answer is one request going wrong; a missing model will go wrong the
    /// same way every time and must not be papered over as a gap.
    #[test]
    fn only_a_bad_draw_is_survivable() {
        assert!(ProviderError::InvalidResponse("x".into()).is_a_bad_draw());
        assert!(ProviderError::Stalled.is_a_bad_draw());
        assert!(ProviderError::IncompleteResponse.is_a_bad_draw());
        assert!(ProviderError::ResponseTooLarge.is_a_bad_draw());

        assert!(!ProviderError::Cancelled.is_a_bad_draw());
        assert!(!ProviderError::ModelChanged.is_a_bad_draw());
        assert!(!ProviderError::RuntimeChanged.is_a_bad_draw());
        assert!(!ProviderError::ModelMissing("m".into()).is_a_bad_draw());
        assert!(!ProviderError::Unavailable("m".into()).is_a_bad_draw());
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
            document_notes: DocumentNotes::english_for_harnesses(),
            model: "test".into(),
            model_digest: "digest".into(),
            runtime_version: "0".into(),
            meeting_language: "German".into(),
            style: GenerationStyle {
                id: "style-formal".into(),
                revision: "1".into(),
                density: crate::domain::ProtocolDensity::Comprehensive,
                instructions: vec!["Write a calm, factual professional protocol.".into()],
                expectations: vec![crate::domain::StructuralExpectation::ActionTable],
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

    /// The step `with_correction_or_keep` protects is the last one, after the whole
    /// meeting has already been condensed. Losing a complete protocol there because a
    /// *retry* went wrong is the one outcome it exists to prevent, and `?` on the
    /// attempt did exactly that: the answer already in hand was dropped on the floor.
    #[test]
    fn an_answer_already_in_hand_survives_a_retry_that_fails() {
        let mut tries = 0;
        let (answer, within) = with_correction_or_keep(
            |_correction, _attempt| {
                tries += 1;
                if tries == 1 {
                    Ok("a whole protocol, lacking only its table".to_string())
                } else {
                    Err(ProviderError::Stalled)
                }
            },
            |_: &String| Err(ProviderError::InvalidResponse("no table".into())),
        )
        .expect("the protocol from the first attempt is still a protocol");

        assert_eq!(answer, "a whole protocol, lacking only its table");
        assert!(!within, "and it is still reported as not what was asked for");
        assert_eq!(tries, 2, "it stopped once the retry failed");
    }

    /// The exception, and it is not negotiable. Somebody pressed cancel, and handing
    /// them a protocol anyway would be the application deciding it knew better.
    #[test]
    fn a_cancellation_is_obeyed_even_with_an_answer_in_hand() {
        let mut tries = 0;
        let outcome = with_correction_or_keep(
            |_correction, _attempt| {
                tries += 1;
                if tries == 1 {
                    Ok("a draft".to_string())
                } else {
                    Err(ProviderError::Cancelled)
                }
            },
            |_: &String| Err(ProviderError::InvalidResponse("not yet".into())),
        );
        assert!(matches!(outcome, Err(ProviderError::Cancelled)));
    }

    /// Nothing in hand and the machine is wrong rather than the draw: the reason has
    /// to reach the person unchanged. A missing model fails the next attempt
    /// identically, so there is nothing to keep trying for.
    #[test]
    fn a_failure_with_nothing_to_keep_reports_why_it_failed() {
        let outcome = with_correction_or_keep(
            |_correction, _attempt| {
                Err::<String, _>(ProviderError::ModelMissing("gemma4:12b".into()))
            },
            |_: &String| Ok(()),
        );
        assert!(matches!(outcome, Err(ProviderError::ModelMissing(_))));
    }

    /// One NDJSON line holding a whole answer, which is what Ollama streams.
    fn one_streamed_answer(protocol: &str) -> String {
        let payload = serde_json::json!({ "protocol_markdown": protocol }).to_string();
        format!(
            "{}\n",
            serde_json::json!({ "response": payload, "done": true })
        )
    }

    /// Serve at most `ATTEMPTS_PER_STEP` answers and report how many were actually
    /// asked for. A connection that sends nothing ends the loop, so a regression that
    /// asks fewer times fails the assertion instead of hanging the suite.
    fn serve(listener: TcpListener, body: String) -> thread::JoinHandle<usize> {
        thread::spawn(move || {
            let mut served = 0;
            for _ in 0..ATTEMPTS_PER_STEP {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let mut buffer = [0_u8; 8192];
                match stream.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
                served += 1;
                write_http_response(&mut stream, body.as_bytes());
            }
            served
        })
    }

    /// A meeting long enough to be worth 200 characters of protocol, which is the
    /// floor `validate_markdown` puts under a plausible one.
    const A_PROTOCOL: &str = "## Besprechung\n\nDie Fassade wurde besprochen und der         Tragwerksplaner hat den Stand der Bauteilplanung dargestellt. Offen bleibt die         Frage der Anschlusspunkte, die in der kommenden Woche geklärt werden soll. Der         Bauherr hat den Terminplan zur Kenntnis genommen und keine Einwände erhoben.";

    /// The shortest meeting was the only path with no second chance: one call, one
    /// parse, one validation, every one of them `?`. Measured, about one draw in five
    /// omits the actions table — so a ten-minute recording, which is exactly what
    /// somebody makes to try the application, failed outright one time in five while a
    /// real meeting was corrected or kept.
    #[test]
    fn a_short_meeting_retries_a_draw_without_its_table_and_keeps_what_arrived() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = serve(listener, one_streamed_answer(A_PROTOCOL));

        let request = synthetic_request(4, 5);
        let protocol = OllamaProvider::at_port(port)
            .generate_in_one_pass(&request, &AtomicBool::new(false), &mut |_, _| Ok(()))
            .expect("a protocol without its table is still a protocol");
        let _ = std::net::TcpStream::connect(("127.0.0.1", port));

        assert_eq!(
            handle.join().unwrap(),
            ATTEMPTS_PER_STEP,
            "it should have asked again with the correction"
        );
        assert!(
            protocol.contains("Anschlusspunkte"),
            "and kept the meeting it was given: {protocol}"
        );
        assert!(
            protocol.contains("No table of next steps"),
            "while saying what is missing from it: {protocol}"
        );
    }

    /// The other half, and the one that would notice an over-eager retry: a good draw
    /// costs exactly one request and is returned untouched.
    #[test]
    fn a_short_meeting_whose_draw_is_good_is_asked_for_once() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let complete = format!(
            "{A_PROTOCOL}\n\n| Aufgabe | Verantwortlich |\n| --- | --- |\n|              Anschlusspunkte klären | Prüfstelle |\n"
        );
        let handle = serve(listener, one_streamed_answer(&complete));

        let request = synthetic_request(4, 5);
        let protocol = OllamaProvider::at_port(port)
            .generate_in_one_pass(&request, &AtomicBool::new(false), &mut |_, _| Ok(()))
            .expect("a protocol with its table is accepted");
        let _ = std::net::TcpStream::connect(("127.0.0.1", port));

        assert_eq!(handle.join().unwrap(), 1, "one good answer is one request");
        assert!(
            !protocol.contains("No table of next steps"),
            "and nothing is added to it: {protocol}"
        );
        assert!(protocol.contains("| Aufgabe | Verantwortlich |"));
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
