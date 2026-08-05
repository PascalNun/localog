# Working plan

This is the project's **living execution board**: what is actually true today, and what happens next. It is the one document expected to change constantly.

It does not replace the others, and it must not duplicate them:

- [PRODUCT.md](PRODUCT.md), [UX.md](UX.md), [MVP.md](MVP.md) — the **goals** being built toward. Stable.
- [DECISIONS.md](DECISIONS.md) — **choices** and dated implementation records. Append-only history.
- **This file** — **execution state**. Rewritten as work lands.

## How to keep this honest

Every goal in this project is achievable; none of it is speculative. Not-yet-built is an ordinary position on the path, not a shortfall. The failure mode this file exists to prevent is only this: documentation drifting ahead of the code, so that a goal reads as though it has already been reached.

Rules:

1. **Update this file in the same change that alters reality.** A step becomes `Done` only when it is verified, not when it is written.
2. **Status words mean exactly this:**
   - `Done` — built, tested, and verified working.
   - `Partial` — usable but with a named gap, and the gap is written down.
   - `Planned` — decided and specified, **no code**.
   - `Unverified` — code exists but has never been proven to work end to end.
3. **Never write a capability in the present tense until it is `Done`.** Describing a goal is fine anywhere; describing it as achieved is not. If a goal document reads as though something exists, correct it there too.
4. When something turns out to be a placeholder or is described as further along than it is, record it under **Known gaps** rather than quietly fixing the wording.

## Where the project stands (2026-08-04)

The durable vertical slice is real: a meeting can go from imported audio to an exported Markdown protocol, surviving restarts and crashes. Local transcription with whisper.cpp is proven on real audio.

| Area                                | Status         | Note                                                               |
| ----------------------------------- | -------------- | ------------------------------------------------------------------ |
| Storage, jobs, crash recovery       | **Done**       | 49 Rust tests; staged writes, reconciliation, immutable revisions  |
| Import → probe → normalise          | **Done**       | Checksummed, cancellable, restart-safe                             |
| Local transcription (whisper.cpp)   | **Done**       | Contract validated against real v1.9.2 + Metal; live progress      |
| Model management (download presets) | **Done**       | Verified HTTPS download, checksum, atomic install                  |
| Transcript review + audio player    | **Partial**    | Real audio/duration, seek, follow-along; playback unconfirmed      |
| Protocol generation (Ollama)        | **Unverified** | Adapter built and unit-tested; **never run for real**              |
| Protocol editing, revisions, export | **Done**       | Autosave, review state, Markdown/TXT export                        |
| Styles + vocabulary library         | **Partial**    | Read-only view only; not editable                                  |
| Speaker diarisation                 | **Planned**    | Runtime validated by spike; **no product code**; still `Speaker 1` |
| Runtime bundling (whisper, FFmpeg)  | **Planned**    | User must still supply a whisper.cpp executable                    |
| Packaging / distribution            | **Planned**    | `bundle.active: false`; no build exists                            |
| Accessibility + performance audit   | **Planned**    | Designed for, never measured on the M1/8 GB baseline               |

## Known gaps between the documents and the code

Tracked openly so they get fixed rather than forgotten.

- **Placeholder values presented as real.** Largely resolved on 2026-08-05 (see Phase B). Still open:
  every transcript segment shows the constant speaker `Speaker 1` in a per-segment column, and the
  New Meeting language select is still unbound.
- **Archive is unreachable.** The schema supports `archived_at_ms`, but no command exposes it, so `MVP.md`'s "archive" capability does not exist in the UI.
- **Network acceptance criterion is stale.** `MVP.md` says tests must fail if the core workflow makes non-loopback network access. Consented model downloads now do. Meeting content still never leaves the device — the criterion needs rewording to say exactly that.
- **German is unvalidated.** The first proving audience is German-speaking project teams, but no German audio has been transcribed or turned into a protocol.

## Plan

Ordered by what each block teaches or unlocks, not by what is easiest. Sizes are relative effort, not
calendar promises. Each block states the condition that ends it, so "done" is not a judgement call.

### Block 1 — Make the protocol good _(next, and the largest single unknown)_

The product exists to produce a protocol, and no real one has ever been generated. Everything after
this is built on an assumption until it is done. It is also cheap to start: Ollama is installed and a
model is already present.

Three things are entangled here and must be worked together rather than in sequence. The style is the
instruction the model receives, the vocabulary is the terminology it needs, and the model is only the
third variable. Judging a model against a one-sentence style would measure the wrong thing.

1. `Planned` Run generation end to end with the installed model and commit a real protocol revision.
   This proves the pipeline, not the quality. **Ends when:** a protocol revision exists on disk.
2. `Planned` Write the "Formal minutes" style properly — sections, tone, rules about not inventing
   decisions, German and English variants. The current styles are single sentences.
3. `Planned` Pull a general instruct model suited to German (Gemma and Qwen are both candidates; the
   8 GB baseline caps this at roughly 5 GB) and compare output on the same transcript and style.
4. `Planned` Generate from real German meeting audio, not only English. This is the first proving
   audience and has never been tested.
5. `Planned` Test whether project vocabulary measurably improves the result. If it does, the library
   editor in Block 3 is justified by evidence rather than by intent.
6. `Planned` Record the finding in `DECISIONS.md`, including whichever model is chosen and why.

**Ends when:** a German meeting produces a protocol a professional would accept after light editing,
or it is established that it does not and the plan changes accordingly.

**Named risk:** good German professional prose may need a model larger than the M1/8 GB baseline
comfortably runs. Transcription fits there; generation may not. If that turns out to be true it puts
pressure on D-015 rather than on the design, and it should be discovered here rather than during
packaging.

### Block 2 — Speaker attribution end to end

Now de-risked by the spike, and it feeds Block 1's output directly: a decision or action without a
name is close to useless in a protocol.

1. `Planned` Compare a German-suited speaker-embedding model against the Chinese-trained one used in
   the spike; test overlapping speech and a long recording on the M1/8 GB baseline.
2. `Planned` Solve alignment between diariser turns and whisper segments. This is the substantive
   design problem and is self-contained enough to test on its own.
3. `Planned` Build the adapter behind the accepted supervised-process boundary, with cancellation,
   bounded output and provenance, as the whisper.cpp adapter does. Degrade honestly to a single
   speaker when the diariser is unavailable.
4. `Planned` Participants on a meeting, inherited from the project. This is the cheapest naming win
   and needs no new inference: naming becomes choosing from a short list.
5. `Planned` Real speaker tools in review: reassign a segment, merge two labels, split one.

**Ends when:** a three-person German recording produces named speakers in the protocol, with the
naming done in a few clicks.

### Block 3 — Reach the remaining capability goals

1. `Planned` Editable vocabulary and protocol-style library. Both are named differentiators and are
   currently a read-only shell. Block 1 decides how much this matters.
2. `Planned` Expose archive, or remove the claim from `MVP.md`.
3. `Planned` Backup and restore.

### Block 4 — Make it runnable by someone else

Deliberately after the product is worth running. Its value is other people's feedback, and feedback
on an unvalidated protocol teaches less.

1. `Planned` Build whisper.cpp statically and ship it as a signed sidecar; remove the executable
   setting entirely. The diariser ships the same way, and the spike already showed that unsigned
   binaries are killed silently on Apple Silicon.
2. `Planned` Select a redistributable, licensed, checksummed FFmpeg build.
3. `Planned` Enable bundling, signing and notarisation; produce the first installable build.

### Block 5 — Harden for use

1. `Planned` Accessibility and keyboard pass with visible focus and text scaling.
2. `Planned` Measure responsiveness and long-recording behaviour on the M1/8 GB baseline.
3. `Planned` Privacy and log audit, including the derived-data rules in `PRODUCT.md`.
4. `Planned` Release checklist.

### Carried alongside

Small items that do not deserve a block and should be picked up when adjacent work touches them:
confirm audio playback; rework the playback **Follow** control; replace the per-segment `Speaker 1`
column (resolved by Block 2); bind or remove the New Meeting language select; narrow the asset
protocol scope from `$APPDATA/**` to the working-audio directory.

## Sequencing rationale

Block 1 is first because it is the only block that can change the shape of the product. Every other
block improves something whose value depends on the protocol being good. If local generation cannot
produce a usable German protocol, that is worth discovering before building a library editor around
it or packaging it for other people.

Block 2 follows because attribution is part of a good protocol rather than a separate feature, and
because the spike removed most of its risk. Its cheapest step — participants — needs no inference at
all.

Block 4 is deliberately late. Packaging is what lets other people use LocaLog, and their feedback is
valuable, but feedback on an unvalidated protocol teaches less than feedback on a good one. The order
should be revisited if the goal changes from "make it good" to "get it in front of people", which is
a legitimate choice rather than a mistake.

Blocks 3 and 5 are sized by what Block 1 finds. If vocabulary turns out to matter a great deal to
protocol quality, the library editor moves up.
