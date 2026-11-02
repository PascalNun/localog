# Managed generation models

Settings → Models offers one ladder of checksum-verified downloads, rather than
hard-coding a single large recommendation. No model is downloaded automatically.

## The ladder

The picker is grouped by installed-RAM class and lists each family at the sizes
it publishes, so a 16 GB or 32 GB machine sees real alternatives instead of the
one model a smaller machine can run.

| Class          | Qwen3.5           | Gemma 4             | Ministral 3 (European) |
| -------------- | ----------------- | ------------------- | ---------------------- |
| 8 GB           | 4B (measured), 2B | —                   | 3B                     |
| 16 GB          | 9B                | 12B (measured), E4B | 8B (measured), 14B     |
| 24 GB          | —                 | 26B A4B             | —                      |
| 32 GB          | 27B, 35B A3B      | 31B                 | —                      |
| 4 GB (testing) | 0.8B              | —                   | —                      |

Only a model measured on the reference meeting may carry “Recommended for this
machine”. The sizes added on 2 November 2026 have been verified as downloads but
not evaluated, so they show “Evaluation pending” and are never the default. The
measured 12B therefore stays the recommendation on a 32 GB machine as well.

Granite 4.1 8B and a future Llama 8B remain as comparison slots: the first is
measured and unstable, the second is planned and not downloadable. They are listed
for honesty about what was considered, not as a family.

| Model           | File                           | RAM class | Intended use                                                              |
| --------------- | ------------------------------ | --------- | ------------------------------------------------------------------------- |
| Gemma 4 12B     | QAT Q4_0, 6,975,879,296 bytes  | 16 GB     | Strongest previously measured protocol candidate                          |
| Gemma 4 E4B     | QAT Q4_0, 5,154,941,280 bytes  | 16 GB     | Smallest Gemma 4; protocol evaluation pending                             |
| Gemma 4 26B A4B | QAT Q4_0, 14,439,363,584 bytes | 24 GB     | Mixture of experts; 4B speed, 26B memory; evaluation pending              |
| Gemma 4 31B     | QAT Q4_0, 17,651,001,568 bytes | 32 GB     | Largest dense Gemma 4; evaluation pending                                 |
| Qwen3.5 0.8B    | Q8_0, 833,592,096 bytes        | 4 GB      | Development/smoke tests; professional protocol quality unverified         |
| Qwen3.5 2B      | Q4_K_M, 1,280,835,840 bytes    | 8 GB      | Cheapest useful Qwen; evaluation pending                                  |
| Qwen3.5 4B      | Q4_K_M, 2,740,937,888 bytes    | 8 GB      | Smaller, previously measured candidate; factual/format limitations remain |
| Qwen3.5 9B      | Q4_K_M, 5,680,522,464 bytes    | 16 GB     | Step above the measured 4B; evaluation pending                            |
| Qwen3.5 27B     | Q4_K_M, 16,740,812,704 bytes   | 32 GB     | Largest dense Qwen 3.5; evaluation pending                                |
| Qwen3.5 35B A3B | Q4_K_M, 22,016,023,168 bytes   | 32 GB     | Mixture of experts; 3B speed, 35B memory; evaluation pending              |
| Ministral 3 3B  | Q4_K_M, 2,146,497,824 bytes    | 8 GB      | Smaller European candidate; protocol evaluation pending                   |
| Ministral 3 14B | Q4_K_M, 8,239,067,840 bytes    | 16 GB     | Largest European candidate; evaluation pending                            |
| Ministral 3 8B  | Q4_K_M, about 6.0 GB (Ollama)  | 16 GB     | European comparison; one usable protocol in three measured runs           |

These are conservative installed-memory classes, not free-memory measurements,
GPU capability checks or speed promises. Linux firmware reservations must not turn
an 8 GB Surface into a falsely reported 7 GB model tier. An 8 GB CPU-only machine
still has little headroom, especially for the protocol pipeline’s minimum 16,384
context. Download size does not include runtime, context cache, OS or other apps.

Oversized managed downloads and selections are disabled with an explanation; the
native backend independently enforces the same requirement before network access,
selection and generation. Unknown memory is shown as unknown. The small test model
is marked as testing only and is never labelled the recommended professional
option. Installed Ollama models remain available through the existing disclosure;
known curated model names use the same memory guard. Arbitrary Ollama models are
not assigned invented RAM limits.

The selected runtime and model are persisted. Choosing Ollama takes precedence
over installed managed files. Explicit missing bundled models or runtimes report a
failure instead of substituting another file. Without an explicit choice, the
first installed managed model that fits is used. Generation provenance uses the
catalogued SHA-256, rather than the licence identifier formerly stored as a digest.
Model download completion and cancellation refresh the model list and progress.

## Provenance checked on 21 October and 2 November 2026

URLs are pinned to the following revisions; exact filenames, sizes and SHA-256
values are in `src-tauri/src/models.rs`. Only metadata was fetched during this
work; no weights were downloaded and no inference-quality claim follows.

- [Google Gemma 4 12B QAT](https://huggingface.co/google/gemma-4-12B-it-qat-q4_0-gguf), revision `29d097773436b69ff9feafd636ab4cf873786537`.
- [Google Gemma 4 E4B QAT](https://huggingface.co/google/gemma-4-E4B-it-qat-q4_0-gguf), revision `4b4a2c1d584be7264f87aac328a1bc739ce81b6c`.
- [Google Gemma 4 26B A4B QAT](https://huggingface.co/google/gemma-4-26B-A4B-it-qat-q4_0-gguf), revision `d1c082be9cf3c8a514acf63b8761f4b41935842e`.
- [Google Gemma 4 31B QAT](https://huggingface.co/google/gemma-4-31B-it-qat-q4_0-gguf), revision `59dde24573e7e61570dba08b18a2e1fe246955ed`.
- [Unsloth Qwen3.5 0.8B](https://huggingface.co/ggml-org/Qwen3.5-0.8B-GGUF), revision `807dfd9a6fadbf703207aac3bfb0b2b3040d077c`.
- [Unsloth Qwen3.5 2B](https://huggingface.co/unsloth/Qwen3.5-2B-GGUF), revision `f6d5376be1edb4d416d56da11e5397a961aca8ae`.
- [Unsloth Qwen3.5 4B](https://huggingface.co/unsloth/Qwen3.5-4B-GGUF), revision `e87f176479d0855a907a41277aca2f8ee7a09523`.
- [Unsloth Qwen3.5 9B](https://huggingface.co/unsloth/Qwen3.5-9B-GGUF), revision `3885219b6810b007914f3a7950a8d1b469d598a5`.
- [Unsloth Qwen3.5 27B](https://huggingface.co/unsloth/Qwen3.5-27B-GGUF), revision `3221f178a6b842d04f1fb42f1c413534adcc0a6a`.
- [Unsloth Qwen3.5 35B A3B](https://huggingface.co/unsloth/Qwen3.5-35B-A3B-GGUF), revision `bc014a17be43adabd7066b7a86075ff935c6a4e2`.
- [Unsloth Ministral 3 3B](https://huggingface.co/unsloth/Ministral-3-3B-Instruct-2512-GGUF), revision `7564922f37fa5bbb62b87f09a55c12f1f91d7a6a`.
- [Unsloth Ministral 3 14B](https://huggingface.co/unsloth/Ministral-3-14B-Instruct-2512-GGUF), revision `9b948841b705cc11285ff274b84ba6c885908b19`.

All of these publisher model cards identify Apache 2.0. In particular, Gemma 4’s
current Google model card supersedes this application’s older Gemma-terms label.
The generic acceptance gate remains available for future agreement-based licences.
The bundled runtime contains Qwen3.5 and Ministral architecture identifiers, but
an actual downloaded-file inference smoke test remains outstanding for every new
file.

## Validation

383 Rust tests and 228 frontend tests pass, as do frontend type checks, lint,
strict Clippy and native/frontend builds. The three files added on 2 November 2026
returned HTTP 200 with the catalogued content length using HEAD requests. The real
Settings → Models screen was checked at 672x872 in both themes: the 8 GB class, the
larger-models disclosure holding the 16, 24 and 32 GB classes, and the disabled
downloads below the machine's class. Full inference and GNOME/KDE behavior remain
untested.

## First Surface inference check — 22 October 2026

Ministral 3B Q4_K_M was installed into app-managed storage and verified against its
catalogued size and SHA-256. The imported development engine raised SIGILL in its
CPU backend; rebuilding the same pinned revision with `GGML_NATIVE=OFF` fixed it.
See [Linux development](LINUX_DEVELOPMENT.md#cpu-compatible-generation-engine).

With a 16,384-token context the rebuilt server became ready in 17.7 seconds. A
synthetic German instruction returned `BEREIT.` after 43.2 seconds including prompt
processing (546 input tokens from the chat template, 5 output tokens). Peak RSS
was about 4.3 GiB. This confirms one local model load/completion, not practical
long-meeting speed or professional protocol quality. The server was stopped after
the check; the application starts it only when generating.
