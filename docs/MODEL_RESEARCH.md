# Model research

This document records model questions that affect LocaLog’s quality, memory use, and distribution. It is research, not a commitment to ship a particular model.

## The constraint

The weakest representative machine is an Apple Silicon Mac with 8 GB RAM. A model must leave room for the application, transcript, working audio, and the operating system. A model that produces better text but swaps for hours is not a useful product choice.

The relevant question is therefore not “which model is largest?” It is “which local model produces a trustworthy protocol in a reasonable time on the machine people actually have?”

## Current generation evidence

The first long German evaluation used an 81-minute meeting on an M1 Pro with 16 GB RAM. The most useful run so far used `qwen3.5:4b` with project vocabulary and a long context, producing a full-length draft in about six minutes. Vocabulary improved proper names and reduced a recurring transcription error.

The run is valuable evidence, but it is not a release acceptance test. It was one meeting, one field, one development machine, and one language.

## What was ruled out or left open

- Larger models can exceed the memory available on the M1/8 GB baseline.
- Mixture-of-experts models save compute but can still consume too much memory for the baseline.
- Very small or heavily quantised models may lose exactly the multilingual and factual precision a protocol needs.
- Bonsai-8B was not a good candidate: the tested runtime support and available quality evidence were weaker than the current model.
- Mistral and Gemma remain research candidates for machines with more memory, not baseline assumptions.
- The initial catalogue now treats Ministral 3B as the first European baseline candidate, Ministral 8B as a 16 GB comparison, and Granite/Llama 8B models as additional comparison slots. Catalogue entries are not product recommendations until the same German and English harness has measured them.

The final public generation runtime remains open. Ollama is useful for development and early previews, but it is not yet the distribution model.

## Next experiments

1. Complete the German quality loop with mechanical checks and human review.
2. Measure the same workflow on M1/8 GB.
3. Test a more efficient Qwen runtime or model variant without changing the application boundary.
4. Compare one or two multilingual candidates on German and English rather than relying on English benchmarks.
5. Record licence, size, memory, speed, and provenance before considering a model for distribution.

The first user-facing model choice is a global Settings preference. It is intentionally separate from the per-job provenance record and from any future language-specific model profiles.

No model should become a product requirement merely because it performed well in one isolated run.

## The families available on 2 November 2026

Checked against the publishers' Hugging Face listings and Ollama tags. Sizes are
the Q4 files, not parameter counts, because the file is what has to fit.

| Family          | Sizes published                                      | Offered in the ladder    | Left out, and why                                                                                                                                                        |
| --------------- | ---------------------------------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Qwen 3.5        | 0.8B, 2B, 4B, 9B, 27B, 35B A3B, 122B A10B, 397B A17B | 2B, 4B, 9B, 27B, 35B A3B | 122B and 397B need 81 GB and more; 0.8B stays only as the harness model, because it cannot write a protocol                                                              |
| Qwen 3.6        | 27B, 35B A3B                                         | none yet                 | Only the two large sizes exist. Mixing 3.6 into a ladder whose lower rungs are 3.5 would make the family look incoherent; evaluate it as a replacement, not an extra row |
| Gemma 4         | E2B, E4B, 12B, 26B A4B, 31B                          | 12B, 26B A4B, 31B        | E2B is 3.3 GB but sits below the measured 4B without evidence it is better; E4B is dominated by the 12B in the same 16 GB class                                          |
| Ministral 3     | 3B, 8B, 14B                                          | 3B, 14B                  | 8B was dominated by the 14B, which needs the same class, and wrote a usable protocol in only one of three measured runs                                                  |
| Granite / Llama | Granite 4.1 8B, Llama 8B                             | none                     | Granite lost five sixths of a meeting’s figures in one run; Llama has no download and was only a comparison slot                                                         |
| Mistral Small 4 | 119B                                                 | none                     | A Q4 file is roughly 65–70 GB; a future 64 GB-plus class, not a 32 GB one                                                                                                |

Two consequences for the catalogue:

- The 24 GB class is thin. Gemma 4 26B A4B is the only entry that belongs there;
  Qwen3.5 35B A3B is a 22 GB file and was classed at 32 GB rather than pretending
  the context and the operating system are free.
- The European option now reaches 16 GB (Ministral 3 14B) and no further. There is
  no European file in the 22–32 GB range that is both current and Apache 2.0, so
  the largest classes are international-only. That is a fact about the market, not
  a preference.

A model enters the catalogue only with a pinned revision, a published SHA-256 and
an exact size. It becomes the default only when it is marked `baseline`, which is a
product judgement made after measurement; being measured is not enough. The two
rules are separate on purpose: the first lets a larger machine see what exists, the
second stops the picker claiming quality nobody measured, or reusing a model that
measured badly.
