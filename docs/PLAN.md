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

| Area                                | Status         | Note                                                              |
| ----------------------------------- | -------------- | ----------------------------------------------------------------- |
| Storage, jobs, crash recovery       | **Done**       | 49 Rust tests; staged writes, reconciliation, immutable revisions |
| Import → probe → normalise          | **Done**       | Checksummed, cancellable, restart-safe                            |
| Local transcription (whisper.cpp)   | **Done**       | Contract validated against real v1.9.2 + Metal; live progress     |
| Model management (download presets) | **Done**       | Verified HTTPS download, checksum, atomic install                 |
| Transcript review + audio player    | **Partial**    | Real audio, real duration, click-to-seek, follow-along            |
| Protocol generation (Ollama)        | **Unverified** | Adapter built and unit-tested; **never run for real**             |
| Protocol editing, revisions, export | **Done**       | Autosave, review state, Markdown/TXT export                       |
| Styles + vocabulary library         | **Partial**    | Read-only view only; not editable                                 |
| Speaker diarisation                 | **Planned**    | D-029 accepted; **no code**; every segment is `Speaker 1`         |
| Runtime bundling (whisper, FFmpeg)  | **Planned**    | User must still supply a whisper.cpp executable                   |
| Packaging / distribution            | **Planned**    | `bundle.active: false`; no build exists                           |
| Accessibility + performance audit   | **Planned**    | Designed for, never measured on the M1/8 GB baseline              |

## Known gaps between the documents and the code

Tracked openly so they get fixed rather than forgotten.

- **Placeholder values presented as real.** The meeting screen shows a hardcoded `Balanced / Global default` preset and `Global + project` vocabulary; the New Meeting language, preset, and vocabulary selects and the Settings General selects are bound to nothing. These contradict "never present incomplete work as finished".
- **Archive is unreachable.** The schema supports `archived_at_ms`, but no command exposes it, so `MVP.md`'s "archive" capability does not exist in the UI.
- **Network acceptance criterion is stale.** `MVP.md` says tests must fail if the core workflow makes non-loopback network access. Consented model downloads now do. Meeting content still never leaves the device — the criterion needs rewording to say exactly that.
- **German is unvalidated.** The first proving audience is German-speaking project teams, but no German audio has been transcribed or turned into a protocol.

## Plan

### Phase A — Consolidate _(in progress)_

1. `Planned` Split the current uncommitted work into focused commits (progress streaming, model management, TLS, Settings UI, audio player, documentation).
2. `Unverified` Confirm working audio actually plays through the asset protocol.
3. `Planned` Rework the playback **Follow** control; its current treatment is not accepted.

### Phase B — Stop showing values that are not real

1. `Planned` Resolve the transcription preset, meeting language, and vocabulary from real state, or hide them until they are real.
2. `Planned` Bind or remove the Settings General controls.
3. `Planned` Expose archive, or remove the claim from `MVP.md`.

### Phase C — Validate the product's core _(highest information value)_

The protocol is the product's purpose, and it is the least validated part.

1. `Planned` Run real Ollama generation end to end and commit an actual protocol revision.
2. `Planned` Produce protocols from representative **German and English** synthetic audio.
3. `Planned` Judge professional usefulness against the style presets; record the finding in `DECISIONS.md`.
4. `Planned` Decide from evidence whether the local generation approach and the style/vocabulary design hold.

### Phase D — Make it runnable by someone else

1. `Planned` Build whisper.cpp statically and ship it as a signed sidecar; remove the executable setting entirely.
2. `Planned` Select a redistributable, licensed, checksummed FFmpeg build.
3. `Planned` Enable bundling, signing, and notarisation; produce the first installable build.

### Phase E — Reach the remaining capability goals

1. `Planned` Editable vocabulary and protocol-style library (a core differentiator that is currently a shell).
2. `Planned` Speaker diarisation spike (sherpa-onnx: contract, quality, alignment, M1/8 GB memory), then implementation and real speaker tools.
3. `Planned` Backup and restore.

### Phase F — Harden for use

1. `Planned` Accessibility and keyboard pass with visible focus and text scaling.
2. `Planned` Measure responsiveness and long-recording behaviour on the M1/8 GB baseline.
3. `Planned` Privacy and log audit; release checklist.

## Sequencing rationale

Phases A and B are short and remove active harm: unreviewable work and an interface that states things that are not true.

Phase C comes before packaging deliberately. Polishing installation for a product whose central output has never been produced would be optimising the wrong end. One real German protocol on screen says more about whether LocaLog is worth building than another week of distribution work.

Phases D and E are both large. D is what lets other people use it at all; E is what reaches the remaining stated goals. Their order should be decided after Phase C, because C may change what E needs to contain.
