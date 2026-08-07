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

| Area                                | Status      | Note                                                               |
| ----------------------------------- | ----------- | ------------------------------------------------------------------ |
| Storage, jobs, crash recovery       | **Done**    | 49 Rust tests; staged writes, reconciliation, immutable revisions  |
| Import → probe → normalise          | **Done**    | Checksummed, cancellable, restart-safe                             |
| Local transcription (whisper.cpp)   | **Done**    | Contract validated against real v1.9.2 + Metal; live progress      |
| Model management (download presets) | **Done**    | Verified HTTPS download, checksum, atomic install                  |
| Transcript review + audio player    | **Partial** | Real audio/duration, seek, follow-along; playback unconfirmed      |
| Protocol generation (Ollama)        | **Partial** | Produces a full-length German draft; names and attribution wrong   |
| Protocol editing, revisions, export | **Done**    | Autosave, review state, Markdown/TXT export                        |
| Styles + vocabulary library         | **Partial** | Read-only view only; not editable                                  |
| Speaker diarisation                 | **Planned** | Runtime validated by spike; **no product code**; still `Speaker 1` |
| Runtime bundling (whisper, FFmpeg)  | **Planned** | User must still supply a whisper.cpp executable                    |
| Packaging / distribution            | **Planned** | `bundle.active: false`; no build exists                            |
| Accessibility + performance audit   | **Planned** | Designed for, never measured on the M1/8 GB baseline               |

## Known gaps between the documents and the code

Tracked openly so they get fixed rather than forgotten.

- **Placeholder values presented as real.** Largely resolved on 2026-08-05 (see Phase B). Still open:
  every transcript segment shows the constant speaker `Speaker 1` in a per-segment column, and the
  New Meeting language select is still unbound.
- **Archive is unreachable.** The schema supports `archived_at_ms`, but no command exposes it, so `MVP.md`'s "archive" capability does not exist in the UI.
- **Network acceptance criterion is stale.** `MVP.md` says tests must fail if the core workflow makes non-loopback network access. Consented model downloads now do. Meeting content still never leaves the device — the criterion needs rewording to say exactly that.
- **German is unvalidated.** The first proving audience is German-speaking project teams, but no German audio has been transcribed or turned into a protocol.
- **A real meeting does not fit the generation context window.** Addressed on 2026-08-05 by
  sectioned generation, but **not yet proven against a real model** — no generation has run.

## Plan

Ordered by what each block teaches or unlocks. Each block states the condition that ends it.

### Block 1 — Make the protocol good _(in progress)_

The working configuration today is **qwen3.5:4b with project vocabulary**: 15,610 characters in six
minutes against an 18,212-character human reference, with company and participant names correct. See
[MODEL_EVALUATION.md](MODEL_EVALUATION.md) for every run.

- `Done` Generation runs end to end on the real 81-minute German meeting.
- `Done` Sectioned generation, for transcripts larger than a model's window.
- `Done` Vocabulary demonstrably improves transcription and the protocol that follows.
- `Done` The "Formal minutes" style is a real specification in the product, not three sentences. It
  was derived from a real professional protocol, and existing databases receive it through a
  migration that leaves an edited style alone.
- `Done` Speaker alignment: joining diariser turns to transcript segments by overlap.
- `Partial` Speaker labelling is wired into the transcription job, now with the machine's cores, its
  neural accelerator, and a known speaker count when one is set. It has never run against the real
  diariser inside the application.
- `Done` One heavy task at a time. Transcription, generation and model downloads share a single
  admission slot, so a download can no longer quietly halve the speed of a transcription.
- `Planned` Judge the current protocol against the human reference on prose, not only on size.
- `Planned` Decide what a missing required section should do. A completed draft was once discarded
  over one absent heading, which sits badly with a product whose protocols are drafts for review.
- `Planned` Generate from an English meeting. No English reference pair exists yet.

**Ends when:** a German meeting produces a protocol a professional would accept after light editing.

### Block 1a — Finish speaker differentiation _(next)_

The alignment is built and tested; what remains is everything around it.

1. `Done` The diariser's two models go through the managed download system with verified checksums,
   like the transcription models. 45 MB together, downloaded and verified in twelve seconds. Both are
   required, so a partial set reports as unavailable rather than failing at the point of use.
2. `Planned` Resolve the diariser runtime the same way as whisper.cpp, so no path is typed by hand.
3. `Planned` Run it against the real 81-minute meeting inside the application and confirm the
   speakers that appear are the people who spoke.
4. `Planned` Regenerate the protocol with speakers present, and check whether attribution to the
   correct organisation improves. This is the failure that motivated the work.
5. `Planned` Speaker tools in review: rename everywhere, reassign a segment, merge two labels.

**Ends when:** the protocol attributes actions to the right people without hand-editing.

### Block 1b — Put vocabulary in the application _(next)_

Vocabulary is proven and lives only in a test harness. The application does not use it.

1. `Planned` Resolve a meeting's vocabulary from the entries already stored for its project.
2. `Planned` Order it by specificity and cut it to fit whisper's ~224-token prompt: this meeting's
   participants first, then the project's own names, then ambiguous abbreviations. General field
   terminology comes last, because the model already knows it.
3. `Planned` Pass it as the initial prompt with `--carry-initial-prompt`, and record it in the job's
   provenance so a transcript can be traced to the vocabulary that shaped it.
4. `Planned` Make the vocabulary library editable, which Block 3 already required.

**Ends when:** importing a meeting into a project with vocabulary produces correct names unaided.

### Block 1c — Configure itself instead of asking

1. `Planned` Read the model's real context limit from the provider rather than assuming 8192.
2. `Planned` Detect installed memory and recommend a model tier from it.
   2b. `Planned` Turn that into a first-run flow: measure the machine, offer only what can run, mark one
   quality as recommended, and make downloading it the obvious next step rather than a setting to
   discover. See `POLISH.md` section 1b.
3. `Planned` Recommend rather than dictate, as the transcription presets already do.
4. `Planned` Evaluate a mixture-of-experts model. Weights are memory-mapped and only a fraction is
   active per token, so the resident working set may be far smaller than the file.

### Block 2 — Make it runnable by someone else

1. `Planned` Bundle whisper.cpp and the diariser as signed sidecars; remove every executable setting.
2. `Planned` A redistributable, licensed, checksummed FFmpeg build.
3. `Planned` Enable bundling, signing and notarisation; produce the first installable build.

### Block 3 — Reach the remaining capability goals

1. `Planned` Editable vocabulary and protocol-style library.
2. `Planned` Expose archive, or remove the claim from `MVP.md`.
3. `Planned` Backup and restore.

### Block 4 — Harden for use

1. `Planned` Accessibility and keyboard pass.
2. `Planned` Measure on the M1/8 GB baseline. Nothing has been measured there.
3. `Planned` Privacy and log audit, including the derived-data rules in `PRODUCT.md`.

### Carried alongside

Confirm audio playback; rework the playback **Follow** control; narrow the asset protocol scope from
`$APPDATA/**` to the working-audio directory.

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
