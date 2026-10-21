# Managed generation models

Settings → Models lists every checksum-verified built-in download, rather than
hard-coding a single large recommendation. No model is downloaded automatically.

| Model          | File                          | RAM class | Intended use                                                              |
| -------------- | ----------------------------- | --------- | ------------------------------------------------------------------------- |
| Gemma 4 12B    | QAT Q4_0, 6,975,879,296 bytes | 16 GB     | Strongest previously measured protocol candidate                          |
| Qwen3.5 4B     | Q4_K_M, 2,740,937,888 bytes   | 8 GB      | Smaller, previously measured candidate; factual/format limitations remain |
| Ministral 3 3B | Q4_K_M, 2,146,497,824 bytes   | 8 GB      | Smaller European candidate; protocol evaluation pending                   |
| Qwen3.5 0.8B   | Q8_0, 833,592,096 bytes       | 4 GB      | Development/smoke tests; professional protocol quality unverified         |

These are conservative installed-memory classes, not free-memory measurements,
GPU capability checks or speed promises. Linux firmware reservations must not turn
an 8 GB Surface into a falsely reported 7 GB model tier. An 8 GB CPU-only machine
still has little headroom, especially for the protocol pipeline’s minimum 16,384
context. Download size does not include runtime, context cache, OS or other apps.

Oversized managed downloads and selections are disabled with an explanation; the
native backend independently enforces the same requirement before network access,
selection and generation. Unknown memory is shown as unknown. The small test model
is never labelled the recommended professional option. Installed Ollama models
remain available through the existing disclosure; known curated model names use
the same memory guard. Arbitrary Ollama models are not assigned invented RAM limits.

The selected runtime and model are persisted. Choosing Ollama takes precedence
over installed managed files. Explicit missing bundled models or runtimes report a
failure instead of substituting another file. Without an explicit choice, the
first installed managed model that fits is used. Generation provenance uses the
catalogued SHA-256, rather than the licence identifier formerly stored as a digest.
Model download completion and cancellation refresh the model list and progress.

## Provenance checked on 21 October 2026

URLs are pinned to the following revisions; exact filenames, sizes and SHA-256
values are in `src-tauri/src/models.rs`. Only metadata was fetched during this
work; no weights were downloaded and no inference-quality claim follows.

- [Google Gemma 4 12B QAT](https://huggingface.co/google/gemma-4-12B-it-qat-q4_0-gguf), revision `29d097773436b69ff9feafd636ab4cf873786537`.
- [Unsloth Qwen3.5 4B](https://huggingface.co/unsloth/Qwen3.5-4B-GGUF), revision `e87f176479d0855a907a41277aca2f8ee7a09523`.
- [Unsloth Ministral 3 3B](https://huggingface.co/unsloth/Ministral-3-3B-Instruct-2512-GGUF), revision `7564922f37fa5bbb62b87f09a55c12f1f91d7a6a`.
- [ggml-org Qwen3.5 0.8B](https://huggingface.co/ggml-org/Qwen3.5-0.8B-GGUF), revision `807dfd9a6fadbf703207aac3bfb0b2b3040d077c`.

These four publisher model cards identify Apache 2.0. In particular, Gemma 4’s
current Google model card supersedes this application’s older Gemma-terms label.
The generic acceptance gate remains available for future agreement-based licences.
The bundled runtime contains Qwen3.5 and Ministral architecture identifiers, but
an actual downloaded-file inference smoke test remains outstanding.

## Validation

354 Rust tests and 212 frontend tests pass, as do frontend type checks, lint,
strict Clippy and native/frontend builds. All four pinned download URLs returned
HTTP 200 with the catalogued content length using HEAD requests. Light/dark
layouts at 1280px and 800px showed every managed option and the disabled oversized
model. Component action spies checked download and installed-model selection
without fetching weights. Full inference and GNOME/KDE behavior remain untested.
