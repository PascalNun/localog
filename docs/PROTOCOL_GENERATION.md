# Generating a protocol by parts

A design proposal. Nothing here is built yet; `PLAN.md` carries the execution state.

## The problem with asking once

Generation today asks the model to do everything in one move: read the transcript, judge what mattered,
attribute it to the right people, and write a professional document — all while emitting valid JSON.
For a long meeting the transcript is split into sections first, but that split is by **size**, not by
task. Inside each section the model is still doing every job at once, and the intermediate result is a
blob of prose:

```rust
struct StructuredNotes {
    notes_markdown: String,
}
```

Three consequences follow from that one field.

**Nothing downstream can be checked.** Whether an action item survived from the transcript into the
notes is not a question the code can ask. It can only count characters.

**Failure is total.** A completed draft was once discarded because one required heading was missing.
The work that produced the other headings was thrown away with it, because there was no
representation of "what we found" separate from "what we wrote".

**Small models are asked to do the hardest possible thing.** Judging, attributing and composing at
once is precisely where a 4B model is weakest — and the 8 GB baseline cannot run anything larger.

## The change: separate reading from writing

Keep the same number of model calls. Change what passes between them from prose to a **record**:

```
MeetingRecord
  topics         [ title, segments ]
  decisions      [ statement, segments, certainty ]
  actions        [ what, owner, due, segments ]
  open_questions [ question, segments ]
  figures        [ statement, segments ]   // numbers, areas, dates, quantities
```

Every item names the transcript segments it came from. That single requirement does a surprising
amount of work: it is the traceability the plan already wants, and it is also an invention check,
because an item citing no segment is by definition something the model made up.

### The passes

1. **Extract** — per section, produce a `MeetingRecord` fragment. Narrow output, no prose, no
   judgement about importance. This replaces the current section pass rather than adding to it.
2. **Consolidate** — merge fragments into one record. Exact and near-duplicate actions can be merged
   in plain Rust; only genuine conflicts need the model, which is cheaper and more predictable than
   asking it to fold prose.
3. **Compose** — write the protocol from the consolidated record and the style. This stays a single
   pass, deliberately: prose written in fragments reads like prose written in fragments, and the
   human reference protocol is continuous text, not a list.
4. **Check** — verify mechanically. Required sections present; every action either has an owner or is
   explicitly marked unassigned; every statement cites a segment.

## What this buys

- **Localised failure.** A missing "Decisions" section is now answerable: either the record held no
  decisions, which is a fact worth printing, or composition dropped them, which is a retry of one
  pass rather than the whole meeting.
- **Traceability without extra work.** Clicking a protocol line to hear it said is a UI change once
  the segment references exist.
- **Checks that mean something.** "Seventeen actions found, sixteen have owners" is a real statement
  about quality. Character count is not.
- **Smaller models become viable.** Each pass has one job and a narrow schema. This is the same
  reason coding agents decompose rather than asking for a finished program in one reply.
- **Progress the user can read.** "Reading section 3 of 9 — 12 decisions so far" beats a bar.

## What it costs, honestly

- **Wall clock.** Consolidation is a pass the current design partly avoids. The extract and compose
  passes replace existing ones, so the increase should be one pass, not five — but it is an increase,
  against a run that already takes six minutes.
- **A schema to maintain.** The record becomes a persisted artifact with its own versioning, like the
  transcript.
- **Prose quality is the risk.** A protocol written from a list can read like a list. Composition
  must keep the whole record in view, which puts pressure back on the context window — and
  `MODEL_EVALUATION.md` shows context is the scarcest resource on the baseline machine. If the record
  does not fit, this design has moved the problem rather than solved it, and that must be measured
  before the work is trusted.

## Staging

Each step is useful alone and can be abandoned without stranding the next.

1. Define `MeetingRecord` and have the extract pass produce it, converting to today's prose notes for
   composition. Nothing visible changes; the record can be compared against the current pipeline on
   the same meeting.
2. Compose from the record instead of from prose. Compare protocol quality against the current
   output on the reference meeting before keeping it.
3. Add the mechanical checks and report them in the review workspace.
4. Persist the record and link protocol lines back to transcript segments.
5. Retry a single failed pass instead of failing the run.

## What would show it worked

Against the reference meeting, compared with the current pipeline: **more of the reference protocol's
actions and decisions present, with no increase in invented material**, at a wall-clock cost that is
still under ten minutes. If extraction finds more but composition writes worse prose, that is a
result too — it would mean keeping the record for checks and traceability while leaving composition
as it is.
