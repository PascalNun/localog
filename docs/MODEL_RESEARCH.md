# Model research

What is known about candidate models from published sources, as of **August 2026**. This is
deliberately separate from [MODEL_EVALUATION.md](MODEL_EVALUATION.md), which records only what has
actually been run on real material. Nothing here has been measured by this project.

Each claim carries the confidence it was recorded with. `verified` means it was checked against a
primary source such as a model card, official documentation or release notes; `likely` means it is
consistent across secondary sources; `uncertain` means it could not be confirmed.

## The short answer

For the **8 GB baseline**, nothing surveyed beats the incumbent `qwen3.5:4b`. For the **16 GB dev
machine** there are two arguable candidates and one clear experiment worth running first — and it is
not a different model at all, but a different build of the one already in use.

## Apple Silicon and Ollama

The most immediately useful findings, because they affect the model already running.

- `verified` **Ollama's default context on any Mac under 24 GB is 4,096 tokens.** A 24,500-token
  transcript is silently truncated unless `num_ctx` is set explicitly. LocaLog now sets it, which
  turns out to matter more than it looked.
- `verified` **The KV cache formula** is `2 × full_attention_layers × kv_heads × head_dim ×
bytes_per_element`. For Qwen3.5-4B this gives exactly 32 KiB/token at f16 — which matches this
  project's own measured ~30 KB/token, so the two independent numbers agree.
- `verified` **Qwen3.5's hybrid attention makes its KV cache about 4.5× cheaper per token** than an
  equally-sized dense model, because only 8 of its 32 layers use full attention. This is why a
  long-transcript workload fits 8 GB at all.
- `verified` **Qwen3.5-9B has identical attention geometry to the 4B**, so moving up a tier costs
  weights only and leaves the context budget unchanged. That makes it an unusually cheap upgrade to
  try on 16 GB.
- `verified` **Ollama v0.32.6 uses Qwen3.5's MTP head for speculative decoding on Apple GPUs** via
  its MLX engine. This is the most direct available lever on generation time, and it is reached by
  changing a tag rather than changing models.
- `verified` The MLX build is 4.0 GB against 3.4 GB for GGUF — a 0.6 GB penalty that is material on
  8 GB and free on 16 GB.
- `verified` `OLLAMA_KV_CACHE_TYPE=q8_0` halves KV cache memory with little precision loss, worth
  about 0.67 GB at the current 40,960 context. It is a server-wide setting rather than per-request,
  so it is a deployment note, not something the application can choose per meeting.
- `verified` **Ollama keeps a model resident for five minutes by default; LocaLog asks for two.**
  That is shorter than the default and may be forcing avoidable reloads between generation passes.

## Bonsai

Investigated because it was raised directly. The finding is clear and negative.

- `verified` **Bonsai-8B is not a new model.** It is a 1-bit (~1.15 GB) rebuild of Qwen3-8B by
  PrismML, a Caltech-originated **US** lab — so it scores nothing on the European-provenance
  argument, and the base model is Alibaba's.
- `verified` Dense, not MoE. Apache 2.0. 65,536-token context natively.
- `verified` **Stock Ollama cannot load its Q1_0 weights at all** — `invalid ggml type 41`. The one
  community repack that does load drops context to 16K, below what an 81-minute meeting needs.
- `verified` **On the one independent head-to-head it scores below what this project already runs**:
  78.9% against 85.2% for Qwen3.5-4B. The ternary variant ties at 85.0%.
- `verified` The only independent multilingual datapoint shows 1-bit quantisation devastating
  non-English knowledge — 45.2% on Persian against 91.7% for the full model. **No German evaluation
  exists for any Bonsai model, from any party.**
- `likely` Community reports describe it hallucinating names and inventing content — precisely the
  failure mode a meeting protocol cannot tolerate, and the one this project measures quality by.

**Verdict: not a candidate.** It cannot load through the required runtime, it is behind the
incumbent on the only independent comparison, and its known weakness is exactly this product's
requirement.

## Mixture of experts

- `verified` **`gemma4:26b` is real, is MoE, and the remembered figures were exactly right**: 25.2B
  total, 3.8B active, 128 routed experts plus 1 shared, top-8 per token, 30 layers, 256K context.
- `likely` **Gemma 4 is Apache 2.0**, not the restricted Gemma Terms. If that holds it removes the
  licence objection that previously counted against the family, including for later fine-tuning.
- `likely` The `gemma4:12b` failure does not transfer to `26b` — they are different architectures.
- `verified` `gpt-oss:20b`: 21B total, 3.6B active, 32 experts per layer, top-4, 128K context,
  Apache 2.0, 14 GB.
- `verified` `qwen3.5:35b-a3b`: 256 routed experts plus 1 shared, top-8, 262,144-token context.

**But the trade is the wrong way round for this product.** MoE spends memory to save compute, and
memory is the binding constraint on the baseline machine — which this project has since confirmed by
measurement (see [MODEL_EVALUATION.md](MODEL_EVALUATION.md)). Nothing MoE plausibly beats the
incumbent dense model on 8 GB. On 16 GB, `gpt-oss:20b` (14 GB) and `gemma4:26b-a4b-it-qat` (16 GB)
become arguable, and both leave almost nothing for a 24,500-token KV cache.

## Mistral

- `verified` The lineup is barbell-shaped: an Apache 2.0 top end far out of reach (Large 3 at 675B,
  Small 4 at 119B, both MoE), and the **Ministral 3 family (3B/8B/14B dense, 256K context, Apache
  2.0)** which is the only part that fits a Mac.
- `verified` Ollama ships it as `ministral-3`, at 3.0 GB (3B), 6.0 GB (8B), 9.1 GB (14B) at q4_K_M.
- `likely` The realistic candidate is `ministral-3:8b`, and it is a **16 GB experiment, not an 8 GB
  swap**: roughly 6.0 GB of weights plus ~4.6 GB of KV cache at 32K context is about 10.5 GB
  resident, and KV quantisation does not rescue it.
- `verified` **Ministral 3's own paper shows the 3B losing to the previous-generation Qwen 3 4B** on
  multilingual MMLU, which is a warning sign for beating `qwen3.5:4b`.
- `likely` The decisive risk is not visible in benchmarks: Mistral instruct models are tuned for
  terse, high-adherence answers, and **short output is already this project's measured failure
  mode**.
- `verified` On procurement grounds the case is clean — every plausible Mistral candidate is
  Apache 2.0, with no restricted-terms trap.

## What to try, in order

1. **`qwen3.5:4b-mlx`** — same model, MLX engine, automatic speculative decoding. Costs 0.6 GB more
   in weights and may cut generation time materially. Cheapest possible experiment: one tag change.
2. **Raise `keep_alive` above two minutes**, so the model is not evicted between passes of a
   sectioned run.
3. **`qwen3.5:9b`** on the 16 GB machine — identical attention geometry, so the context budget is
   unchanged and only weights grow.
4. **`gemma4:26b-a4b-it-qat`** on 16 GB, if the Apache 2.0 licence is confirmed.
5. **`ministral-3:8b`** on 16 GB, for the European-provenance argument, accepting that its known
   tendency to write tersely is this product's existing problem.

## What is still unresearched

The German-language investigation and the final synthesis did not complete — both runs were cut
short. So there is still **no survey of how these candidates handle German specifically**, which is
the single most important axis for this product and the one where English benchmarks are least
trustworthy. That gap is the reason the ordering above rests on architecture and memory rather than
on measured German quality.
