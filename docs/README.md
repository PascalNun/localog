# Reading the LocaLog documentation

The documentation is organised around a simple question: what does a reader need to understand before the next question becomes useful?

## If you are new to LocaLog

Read these in order:

1. [Product](PRODUCT.md) — the problem, the promise, and the first workflow.
2. [UX](UX.md) — how that promise should feel in use.
3. [Visual direction](VISUAL_DIRECTION.md) — the visual language behind the interface.
4. [MVP](MVP.md) — what the first useful version includes and excludes.
5. [Architecture](ARCHITECTURE.md) — how the product keeps work local, responsive, and recoverable.
6. [Decisions](DECISIONS.md) — choices that are already accepted and questions that still need an answer.
7. [Current plan](PLAN.md) — what is implemented, what is only partly working, and what comes next.

The README is the short public introduction. It should be understandable without a software background.

## If you are working on the project

Use the documents for different kinds of truth:

- `PRODUCT.md`, `UX.md`, `VISUAL_DIRECTION.md`, and `MVP.md` describe the product we are trying to build.
- `DECISIONS.md` records choices that change the direction or constrain future work.
- `PLAN.md` is the current implementation status. It is the document that must be corrected when the code moves ahead or falls behind.
- `ARCHITECTURE.md` explains the technical shape in plain language and marks proposals as proposals.
- `ROADMAP.md` keeps later ideas visible without turning them into current requirements.
- `MODEL_EVALUATION.md`, `MODEL_RESEARCH.md`, `PROTOCOL_GENERATION.md`, and `TRANSCRIPTION_EXPERIENCE.md` contain evidence and investigations. They inform decisions but do not replace them.
- The documents under `spikes/` explain isolated experiments. Spike code is not automatically production code.

## Language and status

The documents use these status words consistently:

- **Accepted** — a product or architecture decision has been made.
- **Proposed** — a direction is being explored but should not be treated as settled.
- **Open** — a decision is still needed.
- **Deferred** — the question is deliberately postponed.
- **Done** — built and verified at the level stated.
- **Partial** — usable or present, but with a named gap.
- **Unverified** — code exists, but the relevant real-world or end-to-end proof is still missing.
- **Planned** — agreed work that has not been implemented.

These words are deliberately different. A working parser is not the same as a validated product feature, and a good idea is not the same as an accepted decision.

## A note about history

Decisions and measurements are useful because they explain why the project took its current shape. History should support the current product, not obscure it. When an old experiment is no longer the current path, say so plainly and keep the detail in the evidence document rather than presenting it as the next task.

## Privacy

Real meeting recordings, transcripts, protocols, names, client information, and local paths do not belong in the public repository. The `eval/` directory is intentionally local-only. Use the synthetic fixtures for tests and examples.
