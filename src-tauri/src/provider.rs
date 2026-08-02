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
        Self {
            base_url,
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
        let prompt_bytes = serde_json::to_vec(&prompt_payload)
            .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
        if prompt_bytes.len() > MAX_PROMPT_BYTES {
            return Err(ProviderError::ResponseTooLarge);
        }
        let prompt = String::from_utf8(prompt_bytes)
            .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
        let body = OllamaRequest {
            model: &request.model,
            system: "Create a reviewable professional meeting protocol using only the supplied transcript. Never invent decisions, actions, owners, or dates. State uncertainty explicitly. Follow the controlled style and return only schema-valid JSON.",
            prompt: &prompt,
            stream: true,
            format: protocol_schema(),
            options: OllamaOptions {
                seed: request.seed,
                temperature: f64::from(request.temperature_milli) / 1000.0,
                num_ctx: request.context_tokens,
                num_predict: request.maximum_output_tokens,
            },
            keep_alive: "2m",
        };
        progress(18, "resolving_protocol_inputs")?;
        let response = self
            .agent
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
                progress(60, "generating_protocol")?;
                last_progress = Instant::now();
            }
            if chunk.done {
                done = true;
                break;
            }
        }
        if !done {
            return Err(ProviderError::IncompleteResponse);
        }
        let structured: StructuredProtocol = serde_json::from_str(&generated)
            .map_err(|error| ProviderError::InvalidResponse(truncate(&error.to_string(), 280)))?;
        validate_markdown(
            &structured.protocol_markdown,
            &request.style.required_sections,
        )?;
        progress(78, "validating_protocol")?;
        Ok(structured.protocol_markdown)
    }
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
