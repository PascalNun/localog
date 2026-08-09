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

| Area                                | Status      | Note                                                                 |
| ----------------------------------- | ----------- | -------------------------------------------------------------------- |
| Storage, jobs, crash recovery       | **Done**    | 49 Rust tests; staged writes, reconciliation, immutable revisions    |
| Import → probe → normalise          | **Done**    | Checksummed, cancellable, restart-safe                               |
| Local transcription (whisper.cpp)   | **Done**    | Contract validated against real v1.9.2 + Metal; live progress        |
| Model management (download presets) | **Done**    | Verified HTTPS download, checksum, atomic install                    |
| Transcript review + audio player    | **Partial** | Audio, seek, follow-along, unclear-word review; playback unconfirmed |
| Protocol generation (Ollama)        | **Partial** | Produces a full-length German draft; names and attribution wrong     |
| Protocol editing, revisions, export | **Done**    | Autosave, review state, Markdown/TXT export                          |
| Styles + vocabulary library         | **Partial** | Vocabulary is fully editable; styles are still read-only             |
| Speaker diarisation                 | **Planned** | Runtime validated by spike; **no product code**; still `Speaker 1`   |
| Runtime bundling (whisper, FFmpeg)  | **Planned** | User must still supply a whisper.cpp executable                      |
| Packaging / distribution            | **Planned** | `bundle.active: false`; no build exists                              |
| Accessibility + performance audit   | **Planned** | Designed for, never measured on the M1/8 GB baseline                 |

## Known gaps between the documents and the code

Tracked openly so they get fixed rather than forgotten.

- **Placeholder values presented as real.** Largely resolved on 2026-08-05 (see Phase B). Still open:
  the New Meeting language select is still unbound. Speaker labels are no longer a constant when the
  diariser is available, but that path has not yet run inside the application.
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
- `Done` Speaker labelling runs inside the application, with the machine's cores, its neural
  accelerator and a known speaker count. On the real 81-minute meeting the transcription stage took
  32 minutes and produced 753 segments across **8 speakers** — against 86 from the same recording
  when clustering had no expected count to work from.
- `Done` One heavy task at a time. Transcription, generation and model downloads share a single
  admission slot, so a download can no longer quietly halve the speed of a transcription.
- `Planned` Judge the current protocol against the human reference on prose, not only on size.
- `Done` Decided: a missing section no longer discards the draft. The check could never have passed
  anyway — it matched English section names against a protocol written in the meeting's language, so
  every German meeting was rejected for correctly writing "Zusammenfassung". Reporting which sections
  a draft covers belongs beside the text in review, not in a gate, and is part of Block 1e.
  [PROTOCOL_GENERATION.md](PROTOCOL_GENERATION.md) proposes answering this properly rather than
  patching it: separate what was _found_ from what was _written_, so a missing section is either an
  empty finding worth printing or a retry of one pass.
- `Planned` Generate from an English meeting. No English reference pair exists yet.

**Ends when:** a German meeting produces a protocol a professional would accept after light editing.

### Block 1a — Finish speaker differentiation _(next)_

The alignment is built and tested; what remains is everything around it.

1. `Done` The diariser's two models go through the managed download system with verified checksums,
   like the transcription models. 45 MB together, downloaded and verified in twelve seconds. Both are
   required, so a partial set reports as unavailable rather than failing at the point of use.
2. `Planned` Resolve the diariser runtime the same way as whisper.cpp, so no path is typed by hand.
3. `Done` Run it against the real 81-minute meeting inside the application. Eight speakers appear
   where a threshold alone had produced eighty-six. Whether those eight are the people who spoke,
   rather than merely a sensible number, still needs a person to listen.
4. `Planned` Regenerate the protocol with speakers present, and check whether attribution to the
   correct organisation improves. This is the failure that motivated the work.
5. `Planned` Speaker tools in review: rename everywhere, reassign a segment, merge two labels.

**Ends when:** the protocol attributes actions to the right people without hand-editing.

### Block 1b — Put vocabulary in the application

1. `Done` Resolve a meeting's vocabulary from the entries already stored for its project.
2. `Done` Order it by specificity and cut it to fit whisper's ~224-token prompt: the project's own
   entries before shared ones, and within each, people and organisations before abbreviations, with
   general field terminology last because the model already knows it. A category this build does not
   recognise sorts above general terminology rather than below, since it may well be a name.
3. `Done` Pass it as the initial prompt with `--carry-initial-prompt`, and record it in the job's
   provenance so a transcript can be traced to the vocabulary that shaped it.
4. `Done` The vocabulary library is editable: add, edit, switch off without losing, and delete, with
   duplicates within one scope refused.
5. `Done` Generation receives only the terms the meeting used. A list that fits the prompt is sent
   whole; a larger one is narrowed to the terms the transcript actually contains, matched inside
   compounds so a listed term still applies within a longer German word.
6. `Planned` A meeting's own participants, once participants are a real field, ahead of everything
   else. This is the one part of the specificity rule the data model cannot yet express.
7. `Done` Proven end to end on the real meeting through the application's own functions. The client
   firm appears seven times correctly spelled and the name it used to be misheard as appears none,
   so vocabulary works in the product and not only in a harness.

**Ends when:** importing a meeting into a project with vocabulary produces correct names unaided.

### Block 1d — Show the reader where the transcript is weak

1. `Done` Transcription records per-token probabilities and names the words the model was unsure of,
   using a rule measured against real German audio rather than a chosen constant. See
   [MODEL_EVALUATION.md](MODEL_EVALUATION.md).
2. `Done` Review names those words rather than only marking the segment, and can narrow the
   transcript to the unclear passages. Editing a segment settles the doubt.
3. `Planned` Offer the project's vocabulary as suggestions when correcting an unclear word, since a
   misheard name is usually a name the project already knows.
4. `Planned` Confirm the rule holds on English audio. It has only been measured on German.

### Block 1e — Generate by parts rather than in one move _(proposed)_

Design in [PROTOCOL_GENERATION.md](PROTOCOL_GENERATION.md). The intermediate between reading and
writing is currently a blob of prose, which means nothing downstream can be checked, one missing
heading discards a whole run, and a 4B model is asked to judge, attribute and compose at once — the
hardest possible shape for the only class of model the 8 GB baseline can run.

1. `Planned` Define the meeting record and produce it from the extract pass.
2. `Planned` Compose from the record, and compare quality against today's output before keeping it.
3. `Planned` Mechanical checks, driven from a scan of the transcript rather than from the model's
   own account of itself: quantities found by pattern must each be accounted for, statements must
   cite a segment, actions must name an owner or be marked unassigned. Failures are re-asked, not
   re-run whole.
4. `Planned` Persist the record; link protocol lines to transcript segments.
5. `Planned` Retry one failed pass instead of failing the run.

**Ends when:** every quantity the transcript contains is accounted for in the protocol or explicitly
dismissed — ten of ten on the reference meeting, against one of ten today — with nothing stated that
no segment supports, in under ten minutes. Length is deliberately not the measure: it has been the
wrong one twice.

### Block 1c — Configure itself instead of asking

1. `Done` Read the model's real context limit from the provider rather than assuming 8192. It was
   assuming 8,192 against a model reporting 262,144, which left the answer no room; the value is now
   read and capped at 40,960, a width that has been measured at 4.70 GB resident.
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

1. `Partial` Vocabulary is editable; the protocol-style library is still read-only.
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
