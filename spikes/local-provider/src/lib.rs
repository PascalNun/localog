use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const MAX_RESPONSE_BYTES: usize = 128 * 1024;

pub type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Debug)]
pub enum ProviderError {
    Http(String),
    Json(serde_json::Error),
    ModelUnavailable(String),
    Cancelled,
    ResponseTooLarge,
    InvalidDocument(String),
    IncompleteResponse,
}

impl Display for ProviderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(message) => write!(formatter, "local provider unavailable: {message}"),
            Self::Json(error) => write!(formatter, "invalid provider JSON: {error}"),
            Self::ModelUnavailable(model) => {
                write!(formatter, "model is not installed locally: {model}")
            }
            Self::Cancelled => write!(formatter, "local generation was cancelled"),
            Self::ResponseTooLarge => {
                write!(formatter, "provider response exceeded the safe limit")
            }
            Self::InvalidDocument(reason) => write!(formatter, "invalid protocol draft: {reason}"),
            Self::IncompleteResponse => {
                write!(formatter, "provider stream ended before completion")
            }
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<serde_json::Error> for ProviderError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ModelDescriptor {
    pub name: String,
    pub size: u64,
    pub digest: String,
    pub details: ModelDetails,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ModelDetails {
    pub family: String,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct TranscriptSegment {
    pub start_ms: u64,
    pub speaker: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ProtocolStyle {
    pub id: String,
    pub revision: String,
    pub instructions: Vec<String>,
    pub required_sections: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct GenerationSettings {
    pub seed: u64,
    pub temperature_milli: u16,
    pub context_tokens: u32,
    pub maximum_output_tokens: u32,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct GenerationRequest {
    pub model: String,
    pub meeting_language: String,
    pub style: ProtocolStyle,
    pub vocabulary_revision: String,
    pub vocabulary: Vec<String>,
    pub transcript: Vec<TranscriptSegment>,
    pub settings: GenerationSettings,
    pub application_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationProvenance {
    pub provider: String,
    pub runtime_version: String,
    pub model: String,
    pub model_digest: String,
    pub style_revision: String,
    pub vocabulary_revision: String,
    pub normalized_input_sha256: String,
    pub settings: GenerationSettings,
    pub application_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationMetrics {
    pub prompt_tokens: u64,
    pub output_tokens: u64,
    pub total_duration_ns: u64,
    pub load_duration_ns: u64,
    pub progress_events: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationResult {
    pub markdown: String,
    pub provenance: GenerationProvenance,
    pub metrics: GenerationMetrics,
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ModelDescriptor>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    prompt_eval_count: u64,
    #[serde(default)]
    eval_count: u64,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    load_duration: u64,
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
    transcript: &'a [TranscriptSegment],
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
    agent: ureq::Agent,
}

impl OllamaProvider {
    pub fn loopback(port: u16, timeout: Duration) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .max_redirects(0)
            .build();
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            agent: config.into(),
        }
    }

    pub fn version(&self) -> Result<String> {
        let mut response = self
            .agent
            .get(format!("{}/api/version", self.base_url))
            .call()
            .map_err(http_error)?;
        let value: VersionResponse = response.body_mut().read_json().map_err(http_error)?;
        Ok(value.version)
    }

    pub fn installed_models(&self) -> Result<Vec<ModelDescriptor>> {
        let mut response = self
            .agent
            .get(format!("{}/api/tags", self.base_url))
            .call()
            .map_err(http_error)?;
        let value: TagsResponse = response.body_mut().read_json().map_err(http_error)?;
        Ok(value.models)
    }

    pub fn generate(
        &self,
        request: &GenerationRequest,
        cancelled: &AtomicBool,
    ) -> Result<GenerationResult> {
        let runtime_version = self.version()?;
        let model = self
            .installed_models()?
            .into_iter()
            .find(|model| model.name == request.model)
            .ok_or_else(|| ProviderError::ModelUnavailable(request.model.clone()))?;
        let prompt_payload = PromptPayload {
            meeting_language: &request.meeting_language,
            style_id: &request.style.id,
            style_revision: &request.style.revision,
            instructions: &request.style.instructions,
            required_sections: &request.style.required_sections,
            vocabulary_revision: &request.vocabulary_revision,
            vocabulary: &request.vocabulary,
            transcript: &request.transcript,
        };
        let prompt_bytes = serde_json::to_vec(&prompt_payload)?;
        let prompt = String::from_utf8(prompt_bytes.clone())
            .expect("serialized prompt payload is valid UTF-8");
        let body = OllamaRequest {
            model: &request.model,
            system: "Create a reviewable professional meeting-protocol draft using only the supplied transcript. Never invent decisions, actions, owners, or dates. State uncertainty explicitly. Follow the controlled style and return only schema-valid JSON.",
            prompt: &prompt,
            stream: true,
            format: protocol_schema(),
            options: OllamaOptions {
                seed: request.settings.seed,
                temperature: f64::from(request.settings.temperature_milli) / 1000.0,
                num_ctx: request.settings.context_tokens,
                num_predict: request.settings.maximum_output_tokens,
            },
            keep_alive: "2m",
        };
        let response = self
            .agent
            .post(format!("{}/api/generate", self.base_url))
            .send_json(&body)
            .map_err(http_error)?;
        let reader = response.into_parts().1.into_reader();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let mut generated = String::new();
        let mut done_chunk = None;
        let mut progress_events = 0;
        let mut last_progress = Instant::now() - Duration::from_millis(100);

        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Err(ProviderError::Cancelled);
            }
            line.clear();
            let read = reader
                .read_line(&mut line)
                .map_err(|error| ProviderError::Http(truncate(&error.to_string(), 400)))?;
            if read == 0 {
                break;
            }
            if line.len() > MAX_RESPONSE_BYTES {
                return Err(ProviderError::ResponseTooLarge);
            }
            let chunk: StreamChunk = serde_json::from_str(line.trim())?;
            if !chunk.response.is_empty() {
                generated.push_str(&chunk.response);
                if last_progress.elapsed() >= Duration::from_millis(100) {
                    progress_events += 1;
                    last_progress = Instant::now();
                }
                if generated.len() > MAX_RESPONSE_BYTES {
                    return Err(ProviderError::ResponseTooLarge);
                }
            }
            if chunk.done {
                done_chunk = Some(chunk);
                break;
            }
        }

        let done = done_chunk.ok_or(ProviderError::IncompleteResponse)?;
        let structured: StructuredProtocol = serde_json::from_str(&generated)?;
        validate_markdown(
            &structured.protocol_markdown,
            &request.style.required_sections,
        )?;

        Ok(GenerationResult {
            markdown: structured.protocol_markdown,
            provenance: GenerationProvenance {
                provider: "ollama".into(),
                runtime_version,
                model: model.name,
                model_digest: model.digest,
                style_revision: request.style.revision.clone(),
                vocabulary_revision: request.vocabulary_revision.clone(),
                normalized_input_sha256: sha256(&prompt_bytes),
                settings: request.settings.clone(),
                application_version: request.application_version.clone(),
            },
            metrics: GenerationMetrics {
                prompt_tokens: done.prompt_eval_count,
                output_tokens: done.eval_count,
                total_duration_ns: done.total_duration,
                load_duration_ns: done.load_duration,
                progress_events,
            },
        })
    }
}

fn protocol_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "protocol_markdown": { "type": "string" }
        },
        "required": ["protocol_markdown"],
        "additionalProperties": false
    })
}

fn validate_markdown(markdown: &str, required_sections: &[String]) -> Result<()> {
    if markdown.trim().is_empty() || markdown.len() > MAX_RESPONSE_BYTES {
        return Err(ProviderError::InvalidDocument(
            "document is empty or exceeds the size limit".into(),
        ));
    }
    for section in required_sections {
        let heading = format!("## {}", section.trim());
        if !markdown.lines().any(|line| line.trim() == heading) {
            return Err(ProviderError::InvalidDocument(format!(
                "missing required section: {section}"
            )));
        }
    }
    Ok(())
}

fn http_error(error: ureq::Error) -> ProviderError {
    ProviderError::Http(truncate(&error.to_string(), 400))
}

fn truncate(value: &str, maximum_chars: usize) -> String {
    value.chars().take(maximum_chars).collect()
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> ProtocolStyle {
        ProtocolStyle {
            id: "formal-minutes".into(),
            revision: "style-r1".into(),
            instructions: vec!["Separate discussion, decisions, and actions.".into()],
            required_sections: vec!["Discussion".into(), "Decisions".into(), "Actions".into()],
        }
    }

    #[test]
    fn document_validation_enforces_the_style_contract() {
        let valid =
            "# Protocol\n\n## Discussion\nSynthetic.\n\n## Decisions\nNone.\n\n## Actions\nNone.";
        assert!(validate_markdown(valid, &style().required_sections).is_ok());
        assert!(matches!(
            validate_markdown(
                "# Protocol\n\n## Discussion\nSynthetic.",
                &style().required_sections
            ),
            Err(ProviderError::InvalidDocument(_))
        ));
    }

    #[test]
    fn input_checksum_changes_with_style_or_transcript_revision() {
        let first = serde_json::to_vec(&PromptPayload {
            meeting_language: "en",
            style_id: "formal-minutes",
            style_revision: "r1",
            instructions: &["Be concise".into()],
            required_sections: &["Actions".into()],
            vocabulary_revision: "v1",
            vocabulary: &["LocaLog".into()],
            transcript: &[TranscriptSegment {
                start_ms: 0,
                speaker: "Speaker 1".into(),
                text: "Synthetic".into(),
            }],
        })
        .unwrap();
        let mut second = first.clone();
        second.push(b' ');
        assert_ne!(sha256(&first), sha256(&second));
    }
}
