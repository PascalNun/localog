# Polishing plan

What stands between a working pipeline and something a professional would happily use every week.
The machinery now runs; this is about how it is met, named, and configured.

Ordered by how much friction each item removes, not by how hard it is. Items are not started unless
they appear in [PLAN.md](PLAN.md) as well — this document says _what good looks like_, the plan says
_when_.

## 1. The user should never configure a runtime

Today someone must find and select a whisper.cpp executable before anything works, and the diariser
needs three more paths. Nothing about that belongs in a professional tool.

- Runtimes are bundled and resolved internally; no executable setting exists anywhere.
- Models are chosen as a **quality**, never as a file. This already works for transcription and must
  work the same way for the diariser and for protocol generation.
- The application reads what the machine and the model can actually do — installed memory, the
  model's real context limit — and configures itself. It recommends; the user may override.
- Nothing in the normal path asks a question the application could answer for itself.

**Test of success:** a new user reaches a finished protocol without opening Advanced once.

## 1b. First run: decide for the user, then ask them to confirm

The first time LocaLog opens it should already know what this machine can do, and should lead the
user to the models that suit it. Nobody should face a list of options they have no way to choose
between, and nobody should be able to pick one that cannot work.

What that means concretely:

- **Measure first, then offer.** Read installed memory before showing anything. The transcription and
  generation qualities that fit are offered normally; those that do not are shown as unavailable on
  this machine, with the reason, rather than hidden or silently offered.
- **Recommend one, clearly.** There is a single suggested quality for this machine, marked as such.
  The alternatives remain visible so the choice is real, but the default is the one that works.
- **Downloading is the obvious next step, not a discovered setting.** The recommended models are
  presented as the thing to do now, with their size, so the wait is expected rather than a surprise
  later when a meeting is already waiting.
- **Say what each choice costs.** Size on disk, roughly how long a meeting takes, and what improves.
  A user choosing "Accurate" should know it is slower before choosing it, not after.
- **Nothing is downloaded without the user agreeing to it**, and the application works as soon as the
  recommended set is present — no further configuration.
- **Re-check when the machine changes.** A recommendation made on one machine should not silently
  persist onto another after a restore or a hardware upgrade.

**Test of success:** someone opening LocaLog for the first time on an unfamiliar machine ends up with
a working setup by accepting what is offered, and never wonders whether they picked the right thing.

## 1a. Guard rails: the application must not let a user start something that cannot finish

Everything below was learned the hard way during evaluation. A user should never discover these by
watching a progress bar for forty minutes.

- **Do not offer a model the machine cannot run.** Installed memory is readable on every target
  platform. A model whose weights plus working memory exceed what is available should be marked as
  such, not silently offered. Measured: a 12 B model at long context drove a 16 GB machine into
  swap, and an 18 GB mixture-of-experts model produced 0.6 tokens per second — a protocol would have
  taken over two hours.
- **Never request a context larger than the model supports.** The model reports its own limit;
  assuming a fixed one both wastes a capable model and silently truncates a smaller one.
- **Estimate before starting, honestly.** Transcription and generation each take minutes on a real
  meeting. A rough expectation set beforehand is worth more than a precise number invented afterwards.
- **Detect pathological slowness and say so.** A job producing tokens far below the expected rate is
  not progressing normally, and the user should be told that rather than left watching.
- **Check disk before downloading**, which the model manager already does, and check memory before
  loading.
- **Never run two heavy local jobs at once.** Measured: a transcription that took eleven minutes on a
  quiet machine took over fifty-six while a large download competed for bandwidth and memory. This is
  currently documented but **not enforced**: the job manager admits one model-heavy job, and model
  downloads bypass that admission entirely. Transcription, diarisation, generation and downloads all
  belong under one admission rule, because two of them at once on a machine near its memory limit is
  how a user loses an afternoon.
- **Prefer a smaller model that finishes to a larger one that might not.** The default should be what
  works on the machine in front of the user, with larger options available and clearly labelled.

## 1c. One place always says what the machine is doing

The sidebar already carries a persistent status line, and it is the right idea in the right place:
always visible, never in the way, and not tied to whichever screen happens to be open. It should be
the single answer to "what is happening right now", and today it is not, because it only knows about
meeting jobs.

- **Everything heavy appears there**, not only transcription and generation. A model download holds
  the same admission slot and takes minutes, yet is currently invisible in the status line.
- **A refusal explains itself.** Now that one heavy task runs at a time, an action can be declined
  because something else is running. The reason belongs where the user is already looking, not only
  in the dialog they just dismissed.
- **State is described, not implied.** Queued, running, needing a decision, failed and interrupted
  each read differently. Silence should mean nothing is happening, and never mean the interface has
  stopped noticing.
- **It says what, and roughly how far.** For long work a stage and a sense of progress prevents the
  reasonable assumption that something has hung.
- **It survives navigation.** Work continues while the user reads a transcript or edits a protocol,
  and the status line is what makes that visible without pulling them back.

- **It says what is happening, not that it is happening here.** The heading read "Processing
  locally", which is true of everything this application has ever done and therefore tells a reader
  nothing, while pushing the one thing they wanted to know into the small line beneath it. Several
  stage labels said "locally" a second time under it. A promise that holds always is not news, and
  repeating it spends the most-read line in the interface on reassurance instead of information.

  The exception is worth naming, because it is the only thing that would change this: if work can
  ever happen somewhere other than this device — a firm's own server, as
  [ROADMAP.md](ROADMAP.md) contemplates — then where becomes a fact that varies, and a varying fact
  belongs in the status line. Until then it belongs in the trust surface, said once, not in the line
  that reports work.

- **A long step says where it has got to, not what it is called.** A stage name that does not change
  for four minutes reads as a hung program, however accurate it is. Stages may carry a live detail —
  "Finding what was discussed — passage 3 of 13", "Joining subjects that belong together — 41 found"
  — and any step measured in minutes should use one. This was only possible after the provider was
  allowed to report a string built at the moment rather than a fixed name.

The underlying rule: a user should never have to guess whether the application is working, waiting,
or stuck. Anything that takes longer than a moment is announced in one predictable place.

**Applied to work as it is built.** This is not a polish item to be done at the end. Every stage
added since has reported itself: the topic pass counts its passages, the grouping pass says how many
subjects it is weighing. A step written without its status line is a step that will have to be opened
again, and the moment to decide what a step should say is while writing it, when what it is doing is
still obvious.

The next thing this asks for, and which does not exist yet: **showing the work, not only the
counter.** A reader watching subjects appear as the meeting is divided learns what the machine
understood, which a percentage never conveys. That is also the cheapest moment to notice that a
subject is wrong — before anything has been written from it.

## 2. Names and language

The interface should use the words a project team uses, not the words the implementation uses.

- Review every visible string for implementation vocabulary. "Runtime", "adapter", "provider",
  "context window" and "job" are our words, not the user's.
- Stage names during processing should describe what is happening to the meeting, not which
  subprocess is running. "Reading the meeting in sections" is closer than "condensing transcript".
- Errors say what happened, what is still safe, and what to do next — in that order, without codes.
- Keep German and English equal in the interface's own language handling. No content language is
  privileged.

## 3. The shape of the pipeline

The sequence is right; the joins between stages are where it feels unfinished.

- **Import → transcribe** currently requires an explicit action after import completes. Decide
  whether that is deliberate (reviewing resolved settings first) or merely unfinished.
- **Transcribe → review** should land the user at the first thing needing attention, not at the top
  of an 800-segment transcript.
- **Review → generate** should make it obvious what the protocol will be generated _from_, and that
  editing the transcript first is worthwhile.
- **Generate → edit** should show what changed if a protocol is regenerated, rather than silently
  replacing the previous draft.
- Long operations need honest, moving progress at every stage. Transcription and generation both take
  minutes on real meetings; a still bar reads as a hang.

## 4. Waiting well

A real meeting takes about six minutes to transcribe and six more to generate. That is the single
biggest experience problem, and it cannot be engineered away entirely.

- Say what is happening and roughly how much is left, without inventing precision.
- Never block navigation or editing while work runs.
- Make the wait useful: the transcript should be readable and correctable while generation is queued.
- Consider whether a first draft can appear progressively rather than only at the end.

## 5. Speakers and vocabulary as ordinary work

Both are now real capabilities and both are currently invisible in the interface.

- Renaming a speaker should be one action that applies everywhere, from the segment being read.
- Reassigning, merging and splitting speakers belong in review, not in a settings screen.
- Vocabulary should be reachable from the meeting where a term was wrong, not only from a library.
- A term corrected in a transcript should be offerable as a vocabulary entry in one step.

## 6. Trust made visible

The product's central claim is that this material stays on the device. That should be evident without
being shouted.

- The protocol is visibly a draft until reviewed, and visibly changed after later editing.
- Where the data lives is discoverable in one step.
- Provenance — which model, which vocabulary, which style — is available without dominating the page.
- No permanent "local mode" badge. Status is for things that can change and need attention.

## 7. Accessibility and physical comfort

Not a late pass. These are a professional tool's table stakes.

- Full keyboard reachability with visible focus, including transcript correction.
- Text scaling to 200% without clipping or overlap.
- Reduced-motion respected.
- Colour never the only signal.
- Long reading sessions: line length, contrast, and the transcript's typography deserve attention
  since people will spend real time in them.

## What appears to be missing

Analysis rather than agreed decisions. The first is closer to a design error than a gap.

### Review is required, but the tool that makes review possible is deferred

The product rests on one rule: generated text stays provisional until a person has reviewed it. Yet
`MVP.md` excludes traceability from protocol statements back to the transcript, and the roadmap places
it in Phase 2.

Consider what review actually involves. The reference meeting produced 788 transcript segments and a
protocol of about thirty sections. Reading a claim in the protocol and wanting to know whether it is
true, the reviewer has no way to reach the passage it came from. They can search text that may have
been paraphrased, or scrub audio, or trust it. In practice they will trust it — which is precisely
the outcome the whole design exists to prevent.

Traceability is not a later refinement of review. It is what makes review cheaper than redoing the
work, and without it the review step is honest in intention and hollow in practice. It deserves
reconsideration for v0.1 even in a reduced form: a protocol section knowing which transcript segments
it was built from is enough to jump to them, and the sectioned generation path already handles
material in identifiable groups.

### The product is designed for the first meeting; its value is in the twelfth

Everything specified so far treats a meeting as standalone. But this kind of meeting recurs — the
reference protocol ends by scheduling the next one, with the same firms, the same people, the same
vocabulary, and the same open questions.

Two consequences are missing entirely:

- **A recurring meeting should start nearly configured.** Participants, vocabulary, style and
  language are all known from the previous one. Re-entering them is work the product could avoid.
- **Open actions should carry forward.** Professional minutes almost always open by reviewing what
  was agreed last time. The reference protocol's action table is exactly that kind of list. Nothing
  in the data model connects one meeting's actions to the next meeting's agenda, so every protocol is
  an island.

The second is arguably the sharpest differentiator against a generic transcription tool, which can
only ever produce isolated documents. It is also the reason a firm would keep using this rather than
try it once.

### Failure of the model is not designed for, only failure of the software

Crashes, disk exhaustion, missing runtimes and cancelled jobs all have designed states. "Section four
is wrong" does not. Today the options are to regenerate everything, which costs several minutes and
discards good sections, or to rewrite by hand.

What a person actually wants is narrower: regenerate this section, keep the rest; or correct the
transcript and update only what depended on it.

### Nothing tells the reviewer where to look

Review takes attention, and attention is finite. The application knows things that could direct it —
which sections rest on few transcript segments, where speaker attribution was uncertain, where the
audio was poor. None of it is surfaced, so every part of the document appears equally trustworthy,
including the parts that are not.

## Known rough edges, specifically

Concrete items already observed, small enough to fix once their area is touched:

- The playback **Follow** control's treatment is not accepted.
- Audio playback has never been confirmed by ear.
- Every transcript segment still shows a per-segment speaker column that was a constant until
  diarisation landed.
- The asset protocol is scoped to the whole application-data directory rather than the working-audio
  folder.
- Archive exists in the schema but is unreachable.
- A finished protocol draft can be discarded for missing one required heading.
