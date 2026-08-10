# Local protocol-provider study

This isolated crate tested the protocol-provider boundary against an Ollama server that a user has already started on the local machine. It is not production architecture and never pulls a model.

The study covered runtime and installed-model discovery, exact model and digest checks, professional styles, vocabulary, structured output, streaming, bounded responses, cancellation, validation, and provenance.

The installed `qwen2.5-coder:7b` model was convenient test material, not a product-model decision. Ollama remains a development and early technical-preview option; the public runtime is still open.

Run the portable checks with:

```sh
cargo test
```

With a loopback Ollama server already running:

```sh
cargo test --test installed_runtime -- --ignored --nocapture --test-threads=1
```
