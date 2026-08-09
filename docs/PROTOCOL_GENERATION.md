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

## Topics first, then write each one from the source

The design above compresses the transcript into a record and then writes the protocol from that
record. There is a better shape, and it differs in one decisive respect: **the writing goes back to
the transcript rather than to a summary of it.**

1. **Find the topics.** Read the transcript in windows and produce only a list: what was discussed,
   and which segments discussed it. The output is small even for a long meeting — a few dozen lines
   for eighty-one minutes — so the windows can be modest and the list merged in one further pass.
2. **Write each topic from its own segments.** For each topic, gather the segments it cites and write
   that section from them, with the style and its density. Nothing else is in context.
3. **Assemble in plain Rust.** Order the sections, add the participants, and leave the prose alone.

### Why this is better than a record

**It never compresses before writing.** An extraction pass decides what matters before anything is
written, and whatever it dropped cannot be recovered downstream. Here the writing sees what was
actually said.

**It inverts the context problem instead of moving it.** Today one prompt carries the whole
transcript — about 24,600 tokens — which is why the window is set to 40,960 and why generation costs
4.70 GB resident. In this shape no call needs more than a few thousand tokens. On the eight-gigabyte
baseline that is the difference between the machine being the constraint and not being one, and the
memory it frees can be spent on a better model rather than on key-value cache.

**It changes what a model must be good at.** A long context stops being a requirement. That reopens
small models, and moves the question from "what fits an entire meeting" to "what writes German
well" — which is the axis that actually decides quality and the one still unresearched.

**Traceability and absence come out of it rather than being added.** Each section already knows its
segments, so a line can be traced to what was said. And segments belonging to no topic at all are
precisely the answer to "what did not make it in", which a reader needs before deciding a draft is
finished.

**Failure stops being total.** One topic that fails is one topic to write again.

### Iterating, without putting the model in charge

The three steps above read as a fixed pipeline, and that is not quite the intention. The work should
be split the way a person splits work: look at the material, break off a piece, do it, see where that
leaves things, and break off the next piece. The shape of the job comes from the job.

There is a line to hold, though, and it is worth stating because it is easy to cross by analogy. A
coding agent that decomposes its own work is a large model reasoning about its own task with room to
think. The model here has four billion parameters and must also fit an eight-gigabyte machine
alongside everything else. Deciding how to divide a problem is harder than writing one section of it,
so handing the division to the model asks the weakest thing of the weakest component. It also makes
two runs over the same meeting produce differently-shaped documents, which is poor behaviour for
something a person will compare against last month's.

So the division adapts to the meeting while the procedure stays fixed:

| Decision                   | Made by                                     |
| -------------------------- | ------------------------------------------- |
| What the units are         | the meeting, through the topics pass        |
| How large a unit may be    | plain code — split further past a threshold |
| Whether a unit is finished | a check against that unit's own segments    |
| When to stop trying        | plain code — a fixed number of rounds       |

The iteration lives inside a unit rather than above it. Write the section, check whether the figures
its segments state appear in it, and if they do not, write it again with the omission named. Two or
three rounds, then record what is still missing instead of continuing. A person reviewing the draft
is better placed to judge a stubborn omission than another attempt by the same model.

This is what separates it from an agent loop: nothing decides what to do next. The units come from
the material, the bounds come from the code, and the model is only ever asked to write one thing at a
time and told plainly when it left something out.

### The last step before a person

Something has to happen once the sections exist, and it is tempting to make it a review: ask the
model whether the protocol is right, and hand it over when it says yes. That is the one shape to
avoid.

A four-billion-parameter model asked to judge its own output approves it. It can only compare the
document against itself, since checking it against the transcript means re-reading the twenty-four
thousand tokens this design exists to avoid. And the damage is not that the verdict is worthless.
A tool that says it has checked the work and found it sound is asking the reader to look less
carefully, and the reader looking carefully is the mechanism by which this product is any good at
all. A confident machine opinion placed immediately before a person suppresses the only reliable
check in the system.

What a final pass can usefully do is work, not judgement:

- **Write the opening.** It is the one part that needs the finished document in view, and it is a
  real task rather than a verdict. It also answers the coherence risk, since sections written
  independently read as a list until something ties them together.
- **Look for contradictions between sections.** A concrete question with a checkable answer, unlike
  whether a document is good.
- **Say what it could not place.** Which segments belong to no topic, which figures it could not
  situate, which speakers it never identified.

The last of those is the honest form of a final review: the step before a person reports what it is
unsure of, never that it is satisfied. The mechanical checks stay where they are, because a count of
figures is worth more than an opinion about them, and the verdict stays with the reader.

### What has to be true, and what could go wrong

- **The topic pass decides everything downstream.** A topic missed there is a topic missing from the
  protocol, and no later stage can notice. This is the pass to measure first and hardest.
- **Coherence is the real risk.** Sections written independently can read as a list of sections. The
  human reference is continuous prose that refers backwards and forwards. Assembly may need an
  opening written last, once the sections exist.
- **A segment may belong to several topics, or to none.** Both need a decision rather than a default:
  overlap is normal in a meeting that returns to a subject, and orphans are the absence signal.
- **More calls, each smaller.** Total prompt tokens rise while each call falls. Whether wall clock
  improves is genuinely unknown and must be measured, not assumed.

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
