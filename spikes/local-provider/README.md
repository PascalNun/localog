# Local protocol-provider spike

This crate is isolated from `src-tauri`. It validates the narrow provider boundary against an explicitly started loopback Ollama runtime and must not be imported as production architecture.

## Scope

- Discover runtime version and already-installed models from loopback only.
- Refuse model names that are not returned by installed-model discovery; never call pull/download APIs.
- Submit a controlled professional style, required sections, vocabulary revision, timestamped synthetic transcript, and resolved generation settings.
- Request schema-constrained JSON containing Markdown, stream token chunks, bound the response, and validate the document before it becomes a revision.
- Record provider/runtime/model/settings/style/vocabulary/input/application provenance.
- Cancel by closing a streaming request rather than terminating the user-owned Ollama server.

The installed `qwen2.5-coder:7b` model is sufficient to test the boundary but is not selected as LocaLog's product model. Ollama remains accepted only for development spikes and early technical previews.

Run portable checks with `cargo test`. With a loopback Ollama server already running, execute:

```sh
cargo test --test installed_runtime -- --ignored --nocapture --test-threads=1
```

The keep/change decision and measurements are recorded in `docs/DECISIONS.md` after validation.
