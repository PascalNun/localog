use localog_local_provider_spike::{
    GenerationRequest, GenerationSettings, OllamaProvider, ProtocolStyle, ProviderError,
    TranscriptSegment,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn request(model: String, maximum_output_tokens: u32) -> GenerationRequest {
    GenerationRequest {
        model,
        meeting_language: "en".into(),
        style: ProtocolStyle {
            id: "formal-minutes".into(),
            revision: "style-synthetic-r1".into(),
            instructions: vec![
                "Use restrained professional language.".into(),
                "Distinguish recorded discussion from explicit decisions and actions.".into(),
                "If no decision was made, state that directly.".into(),
            ],
            required_sections: vec!["Discussion".into(), "Decisions".into(), "Actions".into()],
        },
        vocabulary_revision: "vocabulary-synthetic-r1".into(),
        vocabulary: vec!["LocaLog".into(), "acoustic note".into()],
        transcript: vec![
            TranscriptSegment {
                start_ms: 12_000,
                speaker: "Speaker 1".into(),
                text: "We should carry both envelope options into the next review.".into(),
            },
            TranscriptSegment {
                start_ms: 31_000,
                speaker: "Speaker 2".into(),
                text: "The acoustic note and updated cost range are still missing.".into(),
            },
            TranscriptSegment {
                start_ms: 57_000,
                speaker: "Speaker 1".into(),
                text: "No final assembly decision is being made today.".into(),
            },
            TranscriptSegment {
                start_ms: 81_000,
                speaker: "Speaker 3".into(),
                text: "I will circulate both missing items before Thursday afternoon.".into(),
            },
        ],
        settings: GenerationSettings {
            seed: 42,
            temperature_milli: 100,
            context_tokens: 4096,
            maximum_output_tokens,
        },
        application_version: "0.0.1-spike".into(),
    }
}

#[test]
#[ignore = "uses an explicitly started loopback Ollama runtime and installed model"]
fn installed_ollama_generates_validated_protocol_with_provenance() {
    let provider = OllamaProvider::loopback(11434, Duration::from_secs(120));
    let models = provider.installed_models().unwrap();
    let model = models
        .into_iter()
        .find(|model| model.name == "qwen2.5-coder:7b")
        .expect("installed synthetic-spike model");
    let started = Instant::now();
    let result = provider
        .generate(&request(model.name.clone(), 700), &AtomicBool::new(false))
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(result.provenance.provider, "ollama");
    assert_eq!(result.provenance.model_digest, model.digest);
    assert!(!result.markdown.is_empty());
    assert!(result.metrics.output_tokens > 0);
    assert_eq!(result.provenance.style_revision, "style-synthetic-r1");

    eprintln!("runtime_version={}", result.provenance.runtime_version);
    eprintln!("model={}", result.provenance.model);
    eprintln!("model_digest={}", result.provenance.model_digest);
    eprintln!("generate_ms={:.3}", elapsed.as_secs_f64() * 1000.0);
    eprintln!("prompt_tokens={}", result.metrics.prompt_tokens);
    eprintln!("output_tokens={}", result.metrics.output_tokens);
    eprintln!(
        "load_ms={:.3}",
        result.metrics.load_duration_ns as f64 / 1_000_000.0
    );
    eprintln!("progress_events={}", result.metrics.progress_events);
    eprintln!("input_sha256={}", result.provenance.normalized_input_sha256);
}

#[test]
#[ignore = "uses an explicitly started loopback Ollama runtime and installed model"]
fn installed_ollama_stream_can_be_cancelled_without_stopping_server() {
    let provider = OllamaProvider::loopback(11434, Duration::from_secs(120));
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = cancel.clone();
    let worker = thread::spawn(move || {
        provider.generate(
            &request("qwen2.5-coder:7b".into(), 4096),
            worker_cancel.as_ref(),
        )
    });
    thread::sleep(Duration::from_millis(250));
    let cancellation_started = Instant::now();
    cancel.store(true, Ordering::Relaxed);
    let result = worker.join().unwrap();
    let elapsed = cancellation_started.elapsed();
    assert!(matches!(result, Err(ProviderError::Cancelled)));
    assert!(elapsed < Duration::from_secs(2));

    let health = OllamaProvider::loopback(11434, Duration::from_secs(5));
    assert!(!health.installed_models().unwrap().is_empty());
    eprintln!("cancel_ms={:.3}", elapsed.as_secs_f64() * 1000.0);
}

#[test]
#[ignore = "uses an explicitly started loopback Ollama runtime"]
fn uninstalled_model_is_rejected_without_pull() {
    let provider = OllamaProvider::loopback(11434, Duration::from_secs(10));
    let result = provider.generate(
        &request("localog-model-that-is-not-installed".into(), 100),
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(ProviderError::ModelUnavailable(_))));
}
