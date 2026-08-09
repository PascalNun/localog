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

## Driving the model instead of asking it

The passes above split the work. They do not yet **check** it, and checking is what makes the
difference between a smaller task and a reliable one. A coding agent does not ask for a finished
program and hope; it works in steps and verifies after each, feeding failures back. The same shape
applies here, and one observation makes it concrete.

**For some facts, ground truth can be computed without a model.** Scanning the reference meeting's
transcript with plain code takes milliseconds and finds **nineteen quantities** — areas,
percentages, measurements, sums. The generated protocol accounts for **one** of them. That gap is
not a matter of judgement: the information was present, findable without a model, and lost.

So the question put to the model can be bounded and answerable rather than open:

1. **Scan first, in code.** Quantities with units, money, percentages, the project's own vocabulary
   terms, speaker turns — anything a regular expression can find. The result is a checklist with a
   known count.
2. **Give the model the checklist, not only the text.** "These ten quantities were said. For each,
   state what it refers to, or mark it as not worth recording." Every answer is checkable against a
   number that was known before the model ran.
3. **Re-ask only about what failed.** If seven of ten are unaccounted for, ask again about those
   seven with the segments around them. Two or three rounds, then stop and record what is still
   missing rather than looping.
4. **Let it look things up instead of holding everything.** A pass that can retrieve the segments
   mentioning a term does not need the whole transcript in context, which is the constraint that
   binds hardest on the baseline machine.

### What cannot be checked this way

Most of a protocol is judgement — which discussion mattered, how a decision should be phrased — and
no regular expression will find that. The scan covers the part that is mechanical, which is
precisely the part currently being lost, and leaves the rest to the model and the reader.

### What the meeting did not say

The same exercise corrected an assumption worth recording. The generated protocols contain no dates,
and that first looked like the same kind of loss. It is not: **the transcript contains no dates
either** — no month names, no weekdays, two mentions of a deadline in eighty-one minutes. People in
meetings say "next week" and "before the review", and the human protocol's dates came from the
author's own knowledge of the schedule rather than from the recording.

Two consequences. **A protocol cannot contain what the meeting did not say**, so measuring against a
reference that includes outside knowledge sets a bar nothing local can reach; the honest bar is
everything the meeting _did_ say that mattered. And where something is expected but absent, the
right output is to say so — "no date was stated" — and let the reader supply it. Inventing a
plausible date would be the worse failure by a wide margin.

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

**Coverage of what was actually said, not length.** Length has now been the wrong measure twice: once
when a 2,048-token ceiling made every protocol short regardless of model, and again when a protocol
that matched the human reference on structure — thirty headings against thirty — still carried one
quantity where the reference carried nine.

The acceptance test is therefore mechanical and known in advance:

- **Every quantity found by the scan is accounted for**, either recorded in the protocol or
  explicitly dismissed. On the reference meeting that is nineteen of nineteen, against one today.
- **Every vocabulary term that occurs in the transcript** appears spelled correctly, which the
  current pipeline already achieves.
- **Nothing is stated that no segment supports**, which the segment references make checkable rather
  than a matter of reading carefully.
- Wall clock under ten minutes, against six today.

Length is not in the list. A shorter protocol that keeps the numbers is worth more than a long one
that loses them, and that is the whole finding.

If extraction covers more but composition writes worse prose, that is a result too: it would mean
keeping the record for its checks and traceability while leaving composition alone.
