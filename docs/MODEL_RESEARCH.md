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

The final public generation runtime remains open. Ollama is useful for development and early previews, but it is not yet the distribution model.

## Next experiments

1. Complete the German quality loop with mechanical checks and human review.
2. Measure the same workflow on M1/8 GB.
3. Test a more efficient Qwen runtime or model variant without changing the application boundary.
4. Compare one or two multilingual candidates on German and English rather than relying on English benchmarks.
5. Record licence, size, memory, speed, and provenance before considering a model for distribution.

No model should become a product requirement merely because it performed well in one isolated run.
