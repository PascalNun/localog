# Current plan

This is the short answer to “where is LocaLog now, and what should happen next?” It is intentionally a current-status document, not a diary of every experiment.

The product and architecture documents describe the destination. The decision log records choices. This document describes what the code can honestly claim today.

Last reviewed: 15 August 2026.

## The direction

The first meaningful product path remains:

```text
Project → Meeting → Imported recording → Local transcription
→ Transcript review → Local protocol generation → Markdown editing
→ Markdown/plain-text export
```

The next milestone is not another general framework. It is a protocol that a professional can accept after light editing, produced locally, with enough evidence to understand what the system did and where it may be uncertain.

## What is in place

### Product shell — Done

The Tauri shell and browser preview have the real navigation structure, warm light and dark themes, locally bundled Barlow typography, a resizable sidebar, contextual inspectors, keyboard-visible focus, reduced-motion handling, and the main empty, project, meeting, transcript, protocol, library, and settings screens.

The visual shell is still evolving, but it is no longer a disposable mockup.

### Storage and recovery — Done for the current vertical slice

SQLite stores identity, relationships, lifecycle state, revision metadata, jobs, and artifact paths/checksums. Committed transcript and protocol content lives in immutable versioned files. Working autosaves are separate. Imported originals are never silently changed.

Migrations, staged writes, checksums, interrupted jobs, cancellation, retry, and restart reconciliation are covered by the Rust tests.

### Import and media preparation — Done for the current vertical slice

Supported audio/video files can be copied into managed storage, probed, normalised to working audio, checksummed, cancelled, and recovered after interruption. The original file remains untouched.

### Transcription — Partial

The application has a supervised whisper.cpp boundary, structured JSON parsing, timestamps, uncertainty markers, vocabulary prompts, provenance, model presets, and consent-gated verified model downloads.

Real local runs and a long German evaluation exist. A release-only Tauri configuration and reproducible
sidecar builds now define the distribution path for both whisper.cpp and sherpa-onnx, each pinned to
the revision its behaviour was validated against, and the resolver prefers the shipped runtime over
anything on the machine. The sidecars have not yet been built and run on a clean machine, and signed
artifacts and the M1/8 GB baseline still need validation.

### Speaker separation — Partial and provisional

The application contains diariser output parsing, time-overlap alignment, editable speaker labels, managed
diarisation models, a bundled-runtime discovery boundary, and a first-use preparation action in the
meeting flow. The quality evidence is limited to a short synthetic study and one development-machine
evaluation.

Separation runs only when somebody offers a number of people. Clustering by similarity alone was
measured at eighty-six speakers on a meeting of about eleven, because a voice drifts across eighty
minutes of videoconference, and the models stay installed after the first use — so the pass would
otherwise keep running unasked and keep producing that.

Speaker labels must remain provisional. They are not confirmed identities.

#### The pass now listens to samples rather than the whole recording — measured, kept

It ran for about half an hour on an eighty-one minute meeting, which is longer than transcription
and generation together.

Skipping silence does not fix that: only ten per cent of the reference recording is silent, so
working on speech alone saves about a ninth.

Sampling does. Separation runs after transcription, so the segments are already known, and placing a
voice needs a couple of seconds of it rather than a whole utterance. Two seconds from the middle of
each of the reference meeting's 675 segments, joined by silence, is 25.6 minutes of audio in place of
81.8.

Measured on the reference meeting, both runs asking for eleven speakers:

|                              | Whole recording |      Sampled |
| ---------------------------- | --------------: | -----------: |
| Time                         |          1810 s |    **498 s** |
| Turns                        |             291 |          128 |
| Speakers landing on segments |               8 |           10 |
| Largest speaker's share      |            58 % |         56 % |
| Longest unbroken run         |    126 segments | 126 segments |

The two agree on 88.6 % of segments. Neither is ground truth — see below — so agreement is the
question, and the shapes match: the same dominant speaker, the same 126-segment run. Sampling
resolves a slightly longer tail rather than a shorter one. It is three and a half times faster and
is now the path, with the whole recording kept as the fallback when the condensation cannot be built.

The condensation is byte arithmetic over the working audio, not an ffmpeg call. Both ffmpeg routes
were tried and rejected: a filter graph of one `atrim` per sample had not finished after ten
minutes, and the concat demuxer rounds each out point up to a packet boundary, which drifted 36
seconds across the meeting and would have read the last turns back against the wrong audio.

#### What the number of speakers means — open

The count is treated as a fact and is structurally a guess. Thirty people are invited and fifteen
speak; somebody unexpected joins; two people share one microphone. The owner of this project
attended the reference meeting and cannot say whether ten spoke or eleven.

Asking for a count is also the wrong shape of question for the case that needs it. Four people around
a table and the user knows; twenty on a site call with subcontractors dialling in and they do not,
and that is where separation would earn its place.

What the count is worth was measured by sweeping it on the sampled audio, comparing each run against
the eleven-speaker one:

| Asked for | Labels used | Segments per label, largest first       | Agreement |
| --------: | ----------: | --------------------------------------- | --------: |
|         6 |           6 | `385 121 100 43 19 7`                   |      96 % |
|        10 |           9 | `385 121 100 25 16 11 8 7 2`            |      99 % |
|        11 |          10 | `381 121 100 25 16 11 8 7 4 2`          |         — |
|        14 |          10 | `381 121 100 25 16 11 8 7 4 2`          |     100 % |
|        20 |          14 | `378 108 69 31 25 16 13 11 7 4 4 4 3 2` |      92 % |

**The answer barely moves between 6 and 14.** The count a user agonises over mostly does not matter,
and where it does — asking twenty when about ten spoke — the damage is invisible: the top speaker
holds, while the second and third are carved up, 121 to 108 and 100 to 69.

Three ways to avoid asking were measured and none works:

- **The diariser's own automatic mode.** Clustering by distance threshold gives 67 labels on the
  condensed audio and 86 on the whole recording.
- **A quick precheck.** A sparse condensation of every fourth segment — 6.4 minutes, about two
  minutes a run — was swept from 4 to 18. The number of labels simply tracks what is asked for, with
  no plateau anywhere. It measures the sample, not the meeting.
- **Plateau detection on the full condensation.** Eleven and fourteen return identical output, which
  looked like a signal, but twenty returns fifteen. A real estimator would keep answering ten for
  anything above ten. Two agreeing points inside a narrow window are a coincidence, not a method.

#### The pipeline was the wrong shape — replaced, and the count is now an answer

Every problem above traces to one thing: the pass runs pyannote **segmentation** to find where
speakers change, over audio whose boundaries transcription already established. The condensation, the
silence between samples, the 300 ms that turned out to be shorter than the diariser's own
`min_duration_off`, the merged runs of 126 segments, and the eight minutes each count costs are all
consequences of rediscovering what was already known.

sherpa-onnx's C API exposes a speaker embedding extractor that takes the **embedding model alone**:
give it audio, it returns a vector. So the pass should be — two seconds of each transcript segment,
one embedding per segment, cluster the vectors. That removes the segmentation model, removes the
condensation and everything built around it, and makes clustering free, because grouping a few
hundred vectors takes milliseconds rather than minutes.

Free clustering dissolves the count question rather than answering it. Every k can be tried at once
and the affinity matrix's eigengap read directly, which is the actual method for estimating how many
speakers there are; counting non-empty clusters, which is all the CLI permits, is a crude proxy for
it. The user can be shown what was found instead of asked for what they do not know.

What survives from the sampling work is its central finding — two seconds of a segment is enough to
place a voice — which is exactly what makes per-segment embedding cheap. What becomes unnecessary is
the plumbing around it.

That is now built. `localog-speaker-embedding` is a supervised sidecar like the others, built from
the same pinned sherpa-onnx revision as the diariser and linked statically so it carries no
dependency on the machine that built it. It writes one vector per segment as a small versioned
binary file rather than through a pipe, because a meeting's worth is megabytes. The grouping happens
in the application and is checked against the study: on the reference meeting it reproduces it
exactly, `388 120 102 17 17 11 10 7 1 1 1` at eleven voices.

**The interface now offers three answers rather than two.** Leave the speakers together, separate
them into a stated number, or separate them and let LocaLog work out how many. The third was not
offerable before: the diariser answered by re-reading the audio, so a count had to be settled in
advance by somebody who often does not know it. Leaving them together remains a choice somebody
made rather than an absence of one, and the pass does not run because the models happen to be
installed.

The estimate reads the count off where the merging stops joining a person to themselves. The floor
is fitted to one fixture with ground truth and one meeting without, so it is offered as an estimate
that can be replaced with a number, never asserted.

The diariser remains as the fallback where the embedding sidecar is not installed. There is one
genuine loss against it — pyannote can in principle catch a speaker change inside a single transcript
segment, and one embedding per segment cannot. Segments average 7.3 seconds here and the sample is
taken from the middle, and the 126-segment merged runs suggest the older pipeline was not catching
them in practice either.

Not yet built and run on a clean machine, which is the same gap the other two sidecars have.

#### Still unanswered

The embedding model is trained on Chinese, which is a poor match for the first audience. Whether a
different one does better is unresearched.

Underneath all of it: speaker separation exists to improve attribution in the protocol, and that has
never been measured. A protocol generated with speakers and one without, from the same meeting, would
settle whether the pass earns its place at all.

Neither run above can be scored for accuracy, because the reference meeting has no known speaker
count. If a wrong count degrades the result badly and nobody can supply a right one, the honest
conclusion is that automatic separation is not ready to be trusted for attribution, and the useful
feature is one that helps a person label speakers rather than one that claims to know.

### Protocol generation — Partial; the main quality work

The Ollama provider is narrow, loopback-only, cancellable, bounded, provenance-aware, and restricted to already available models. Generation is sectioned for long transcripts and has style and vocabulary inputs.

Generation records what it found about its own result: how many stated quantities the protocol keeps,
which figures it states that the meeting did not, and its length against the transcript. Those numbers
now reach the reader, beside the draft they describe.

They are presented as evidence to look at, never as a verdict. Only one of them is wrong under every
style — a figure the draft states that the meeting did not — and that is the only one shown as a
warning. How much a draft keeps is what its style asked for, and a machine judgement placed in front
of a person asks them to read less carefully, which is the one check in this product that reliably
works.

Dividing a meeting into subjects is compiled for evaluation only, which is its honest status while the
first question below is open. Writing subject by subject was measured and rejected — it produced a
document longer than the transcript — and running the pass merely to index a finished protocol would
add about seven minutes to a twelve-minute run for a diagnostic. The evidence stays runnable through
the evaluation harness; the shipped library carries nothing it does not call.

The generated protocol is not yet proven complete or reliable enough for professional use.

### Editing and export — Done for the current vertical slice

The protocol editor supports Markdown editing, autosave, undo/redo, find, text scaling, review state, immutable revisions, restoration, and explicit Markdown/plain-text export.

A transcript line can now be removed as well as rewritten — for the throat clearing, the crosstalk
and the thirty seconds of somebody's dog. Without a confirmation, deliberately: rewriting a line is
exactly as permanent as removing one and asks nobody's permission, and the committed revision is the
way back from both. The last line cannot be removed, in the application and in the interface.

A recording can be trimmed before it is transcribed. Drag across the timeline to select a stretch,
then start there, end there, or remove it; every cut is listed and undoable one at a time or all at
once. The cuts are held as a description of what to keep and applied when the working audio is
built, so the recording is never modified and none of it is final until then. What is removed is
veiled on the timeline rather than taken off it, because seeing what you cut is what makes putting
it back feel possible.

Editing before transcription is what keeps that simple: no transcript exists yet whose timestamps
would have to be reconciled with a timeline that just got shorter.

The recording timeline is usable without a pointer, which it was not when first built: it takes
focus, the arrow keys move a visible caret along it, holding shift takes the selection with them the
way selection works in a text field, and `Home`, `End` and `Escape` do what they say. Alt gives a
finer step and shift a coarser one, so crossing an eighty-minute meeting is a few keys rather than
a hundred. It announces itself as a slider whose value is the selection, so what a screen reader
says and what the screen shows are the same thing.

The editor still needs long-document, accessibility, and real-background-load validation.

### Libraries and settings — Partial

Vocabulary is editable and resolved into job provenance. The shipped professional styles are structured and versioned, but the style library is not yet fully editable. Language concepts remain separate by design.

The meeting-language flow is now wired through project defaults, per-meeting overrides, transcription
runtime language codes, and protocol-generation inputs. The normal UI offers common languages while
still allowing a language name outside the convenience list. Interface-language selection remains a
separate future setting.

Meeting and transcript review now expose the language as a correction point. A user can change it and
explicitly rerun transcription; the current result remains in place until the new job commits, while
the new run receives its own immutable revision and provenance. Automatic language detection is not
used as a silent replacement for the user's choice. It should first be tested as an advisory preflight,
because a guess is not a safe reason to change a professional record.

Speaker differentiation is now visible as an optional, progressive-disclosure setting. The existing
diariser boundary reports model/runtime readiness, accepts an expected speaker count for each
transcription run, and keeps labels
editable in transcript review. It remains provisional until a distributable runtime and broader
multilingual quality evidence exist.

## What is not yet true

- Recording from the microphone or system audio is not wired into the application, but every part of
  it now works in the study in `spikes/meeting-recording/`: the microphone, system audio, surviving
  a kill without losing audio, and storing the result as Opus. System audio took five attempts and
  failed all five for one reason — macOS gates it behind Screen & System Audio Recording and hands
  an unauthorised tap silence rather than an error. Once granted, it captures. **A tap that is not
  permitted is indistinguishable from one that is broken**, which is why a recorder must show that
  sound is arriving before a meeting starts rather than after it ends.

  What remains is the interface — the record screen — and answering whether the two tracks drift
  apart over a long meeting, which a short run cannot separate from a fixed start offset.

- Project and meeting archive actions are not exposed in the interface.
- Basic backup and restore are not implemented.
- The release config bundles the whisper.cpp and diariser sidecars once the target-specific artifacts have been built. Neither has been produced by the release command and run end to end yet, and FFmpeg still needs the same treatment.
- The final public protocol-generation runtime is undecided; Ollama is for development and early technical previews.
- Windows and Linux have architectural support, but no packaged release or complete runtime validation.
- Performance and accessibility have not been accepted on the M1/8 GB target.
- Real English end-to-end quality evidence is still missing.

## Next milestones

### 1. Make protocol generation trustworthy

This ends when a representative German meeting produces a protocol that a professional accepts after light editing.

Work in this order:

1. Decide whether the current sectioned approach is enough or whether the structured evidence record should become the production intermediate.
2. Keep the useful transcript-to-segment links and unclaimed-segment reporting.
3. Quantities kept, figures invented, length against the recording, and tasks recorded with nobody
   against them are measured and shown. Unsupported statements still need a check that can be
   relied on, and the obvious ones need a model to judge its own work, which this project has
   already found unreliable.

   Missing sections cannot be checked as things stand. A style's `required_sections` are stored as
   English literals while the protocol is written in the meeting's language, and they are
   deliberately kept out of the prompt — sending them once took the reference protocol from 17,393
   characters to 2,747. Matching "Actions" against "Aufgaben" needs something the application does
   not have.

   The action table is now checked structurally instead, which sidesteps the language entirely: a
   markdown delimiter row looks the same in German. Measured across nine runs, about one draw in
   five omitted the table having been told twice to produce one, and the rejection is fed back
   through the existing correcting retry.

### 1b. What a style is, before anybody can author one

Decision 7 says a person authors their own styles. Before that is built, the object needs taking
apart, because what it currently holds is four different things under one name.

Sorting the shipped style's fourteen instructions:

| Kind          | Count | Example                                                   |
| ------------- | ----: | --------------------------------------------------------- |
| **Structure** |     6 | "End with a table of agreed next steps"                   |
| **Fidelity**  |     5 | "Never invent a decision, an action, an owner, or a date" |
| **Voice**     |     1 | "Write discussion as calm, factual prose"                 |
| Density       |     1 | "Write at whatever length the material requires"          |
| Language      |     1 | "Write the entire protocol in the meeting's language"     |

**One instruction of fourteen concerns how language is used**, which is the thing the owner means
by style — direct or lofty, plain or formal. The rest is document shape and rules about telling the
truth, both wearing the name "style".

That suggests four axes rather than one, and they do not all belong to the author:

- **Fidelity is invariant and not authorable.** "Never invent a decision", "reproduce every figure
  exactly", "never leave a placeholder", "cover every topic". A person authoring a house style is
  choosing how their firm writes, not whether the record may be wrong. These belong to the
  application and should be applied to every style, including authored ones.
- **Structure is authorable and checkable.** Numbered sections, participants first, ends with a
  table of tasks and owners. Declared as structure rather than as prose, each one becomes a check
  that plugs into the correcting retry — which is exactly how the action table now works, and it
  needed no translation to do it.
- **Density is already a separate setting** and should stay one. `ProtocolDensity` exists, is
  applied by `with_density`, and is not stylistic: it is how much of the meeting a reader wants.
- **Voice is the remaining thing, and is the smallest part of the object today.** It is also the
  part that most needs prose rather than structure, because "calm and factual" cannot be expressed
  as a field.

The immediate consequence is that `required_sections` should be replaced rather than repaired.
Structural expectations in a checkable form are what it was reaching for; English section names
were the wrong shape for a document written in German.

4. Retry one failed section or pass rather than discarding the complete run.
5. Compare the result with the existing human reference on completeness, correctness, attribution, length, and editing effort—not length alone.
6. Repeat the same workflow with an English meeting or synthetic equivalent.

### 1z. Ask for the vocabulary, because it is the largest measured win in the project

Do this before any further work on protocol models. It is the cheapest improvement
found so far and it improves every model at once.

The reference meeting was transcribed with an empty vocabulary — the workspace holds
zero entries — and every model comparison in this project rests on that transcript.
Giving whisper thirty proper nouns as its initial prompt, on the _same_ model, with
nothing else changed:

| Term                                  | Without vocabulary      | With vocabulary   |
| ------------------------------------- | ----------------------- | ----------------- |
| The housing form the meeting is about | 0 right, 40 wrong       | 32 right, 4 wrong |
| The building system supplier          | 0 right, 13 wrong       | 6 right, 2 wrong  |
| The client's name                     | 0 right, 1 wrong        | 4 right, 0 wrong  |
| The word for the building envelope    | 19 right, 11 wrong      | 35 right, 6 wrong |
| The word for structural engineering   | **never occurs at all** | occurs            |

The last row is the one to act on. Without the vocabulary that word does not exist
anywhere in seventy-two thousand characters, and the generated protocol therefore
files a named structural engineer under a discipline he does not practise. No
protocol model can recover from that, and every hour spent comparing protocol models
on that transcript was measuring the transcriber.

The mechanism is built and correct: `vocabulary_prompt` prioritises proper nouns,
respects whisper's limit, and passes `--carry-initial-prompt` so the terms bias the
whole meeting rather than its first thirty seconds. **What is missing is the asking.**
`NewProjectView` mentions in one line that vocabulary "can be configured in the
project"; `NewMeetingView` says per-meeting vocabulary "is not available yet". A
person who creates a project, records a meeting and presses go gets the bad
transcript, because nothing ever asked them for the twelve words that would fix it.

So the work is a question of product, not of models: at project creation, ask for
the names — the client, the firms, the people, the project. It is the one input where
a minute of somebody's typing is worth more than any model choice this project has
measured. The exact shape of that asking is a design decision and is the owner's.

The screen is now called **Names & terms**. Vocabulary oversold it and glossary would
be wrong — a glossary carries definitions and this carries none. It is a spelling
list, and measured, almost entirely a list of proper nouns.

#### Where the application asks, and what it does with the answer

Designed 16 August 2026 with the owner. Not built.

**1. Offer candidates instead of a number nobody can act on.** whisper already records
which words it was unsure of, and the transcript view already has a panel for them.
For the reference meeting that panel says **322 to check** out of 675 segments, which
is not a task anybody starts. But the words it flags do contain the mis-heard names —
`Trakwerk`, `Klasterwohnung`, `Nukera`, `Vermessung`.

Keeping only words the transcriber was unsure of _every_ time it heard them, heard at
least twice, gives **six candidates** for an eighty-minute meeting, of which two or
three are the names that matter. That is a thirty-second job with a large payoff, in a
panel that currently asks for an impossible one. Loosening the filter to catch more
names costs precision quickly: a top-fifteen list picks up two more surnames and about
as many ordinary German compounds.

Open question for the owner: whether the flag count stays visible alongside, or the
candidates replace it. The flags are also how somebody finds passages to re-read
before trusting the protocol, which is an argument for keeping them.

**1b. The protocol model already flags names the transcriber was sure about.** Found
by accident, 16 August 2026, reading the foot of a draft:

> [Note: The term "Klinker-Nord" is used in the source text; it is unclear if this
> refers to a specific project name or location.]

That is the client's name, mis-heard. Unprompted, the model noticed the word behaves
like a name it does not recognise and told the reader instead of using it silently.

The important part is which error it caught. whisper flagged the catastrophically
mangled form of that name — `Lärgedorf-Bildes-Fropette-Reit` — and did **not** flag
the plain wrong spelling, because it was confident about it. **A transcriber's
confidence cannot flag an error it is confident about.** That is a structural blind
spot, and it is the dangerous class: a confidently wrong name is the one that reaches
a client's inbox looking correct.

So there are three sources of candidate terms, and they overlap less than expected:

1. Words the transcriber was unsure of every time — catches garbled forms.
2. The same, grouped by stem — catches names whose mis-hearing varied.
3. **The protocol model's own notes — catches names the transcriber was sure about.**

The third is weak on its own: four of thirteen drafts carried such a note, each
flagging one name. It is also free, since the model is already writing them, and it is
the only one of the three that can see this class at all. Harvesting them into the
candidate list is a small piece of work with no new model call.

**2. One correction, two jobs.** Correcting `Klaster → Cluster` should fix the current
transcript _and_ enter Names & terms so the next meeting is transcribed correctly.
Cure and prevention from the same keystroke, which is what makes the thirty seconds
worth spending.

**3. Fix the current transcript deterministically, before the protocol harness sees
it.** Measured on the reference meeting, plain substring replacement of eleven
corrected stems fixed **80 occurrences** in milliseconds — reaching roughly what a
seven-minute re-transcription with the larger model achieved:

| Term     | Before | After replacement | (`medium` + vocabulary) |
| -------- | -----: | ----------------: | ----------------------: |
| Cluster  |      0 |            **40** |                      40 |
| HOAI   |      0 |            **16** |                      35 |
| Tragwerk |      0 |             **5** |                       6 |
| Fassade  |     19 |            **30** |                      50 |

Compounds are the easy case, not the hard one, because German builds them by
concatenation: fixing the stem repaired `Clusterwohnung`, `Raumcluster` and
`Einraumcluster` without anyone listing them. Of 74 occurrences, 59 came out clean and
about **three** were genuinely still wrong — `Clusterwund`, `Clusterwohnenheit`.

No model, instant, and auditable: the change can be shown and undone, which a model
pass over the whole transcript cannot offer.

**4. The replacement must be reviewable, because some wrong spellings are real words.**
`Halle` should be `Halde`, a participant's surname — and _Halle_ is the German word
for cross. In this meeting all three occurrences are the person, so replacing blind
would have been safe; that will not always hold. Show the matches in context and let
them be deselected.

**5. Only then, a small model, on what is left.** Three ragged words per meeting is
where substitution cannot help, because the mis-hearing itself varied and there is no
consistent stem to catch. This is the one stage in the pipeline where a long context
is provably unnecessary: deciding whether `Halle` is a person or a crucifix needs one
sentence, not eighty minutes. So it is a small model, a few hundred characters of
window, and only for passages the deterministic pass could not settle — seconds of
work, not minutes.

Two constraints make it safe:

- **It proposes; it never applies.** The transcript changes only where somebody
  approved the change.
- **It is given Names & terms, and may only propose corrections built from them.** It
  may offer `Clusterwohnenheit → Clusterwohneinheit` because `Cluster` is listed. It
  cannot invent a name nobody entered. That bounds the single risk of letting a model
  near the evidence record.

**6. Say what is being done, at every step.** `docs/UX.md` already requires this —
the status answers "what is happening?" in a reader's words, with a moving detail on
long steps — and the five steps above were written without it, which is how a stage
that quietly rewrites the evidence record gets built.

This stage needs it more than most, because it changes a document the person is
holding. Concretely:

- The substitution is instant, so it needs a **result**, not a progress bar:
  "12 corrections applied in 80 places" with the places listed and undoable. A silent
  transformation of the transcript would contradict the standing rule that imported
  originals and existing exports are never silently changed.
- The model pass gets a stage in a reader's words — "Checking 3 passages that could
  not be settled" — with the count as the moving detail, not a spinner.
- Waiting for approval is already one of the states the sidebar distinguishes, and
  this stage is the clearest case of it in the application.
- Nothing here is a heavy-lane task, so it must not block or be blocked by one.

Build order is deliberate: the deterministic pass first. If a few meetings show the
leftover is consistently three-ish words, a person fixes them faster than the
suggestion could be built.

### 1a. Strengthen the harness so a bad draw is not a lost run

Generation is already sectioned: a long meeting is condensed section by section and
then synthesised. When any one step returns something unusable the whole run is
discarded, which is a quarter of an hour of somebody's machine for a fault that
affected one section.

Measured on the reference meeting, `ministral-3:8b` returned a usable protocol at one
seed of three and `gemma4:12b` at three of three — but Gemma missed the required
table at one of them. So this is not about rescuing weak models: every model has bad
draws, and the harness currently converts each one into a total loss.

Done, and worth having before the rest:

- Code fences stripped deterministically, including ` ```json `.
- An answer refused when it parses as a JSON object, or is shorter than a hundredth
  of what was said. Both of `ministral-3:8b`'s failures are caught by this, and the
  JSON one had scored 28 of 35 figures.
- Every parse of a model's answer repaired when it is nearly-JSON, rather than the
  run being lost to a newline the model wrote inside a string.

To build, in order:

1. **Retry the step that failed, not the run.** A section that comes back empty,
   fenced-as-JSON or implausibly short is one request, and the sections around it
   were fine. Retry it a small fixed number of times before failing the job.
2. **Tell the model what was wrong.** "You returned JSON; return markdown" is a
   strong correction and costs one request. This is the whole of the agentic part —
   it needs no planner and no extra pass, only the rejection fed back.
3. ~~**Keep what survived.**~~ Built. A section that fails every retry becomes a
   marked hole: the notes carry an instruction to say at that point that the content
   is unknown and not to guess at it, and the finished protocol carries a closing
   section naming each missing stretch by its position in the recording, so somebody
   can scrub to it and listen. The second of those does not depend on the model doing
   as it was told. Only a bad answer, a truncated one or a model gone quiet is
   survivable this way; a missing model or a changed runtime fails every remaining
   section identically and still fails the run, as does every section failing.
4. **Ask for less at a time**, which is the same experiment as the context question
   below and worth running once for both. Partly measured — see below.

### The context window is a parameter, not a requirement

Worth recording because it is easy to assume otherwise. `plan_sections` divides a
transcript to fit `context_tokens`, and `synthesis_budget` folds the notes until they
fit the same window. The harness already works the way a person would: read the
meeting in pieces, then bring the pieces together on far less material than the
whole. 40,960 tokens was chosen because it was measured as affordable, not because
the design needs it. Setting it to 8,192 makes more, smaller sections and more
folding, and changes nothing else.

Two reasons to try it:

- **Memory.** At 8,192 tokens `gemma4:12b` needs roughly a fifth of the key-value
  cache — about 1 GB rather than 5, bringing it near 7 GB total instead of 12. The
  best measured model does not currently fit the 8 GB target, and this is the most
  plausible route to making it.
- **Weaker models.** `ministral-3:8b` wrote 12 KB of usable protocol at one seed and
  collapsed at two others from the same prompt. A smaller ask may be a steadier one,
  which would matter far more to it than to Gemma.

The cost is more requests, so more wall-clock, and more joins — a subject discussed
in two places is likelier to be split between sections. That is exactly what figures
kept and a reading of the draft would show, so the experiment is: `gemma4:12b` and
`ministral-3:8b` at 8,192 against 40,960 on the reference meeting. The harness takes
the context as an environment variable, so it needs no code.

#### First measurements, 16 August 2026

`gemma4:12b`, seed 7. One draw at each point, which is why the reading below is corrected further down:

| Context | Sections | Seconds | Headings | Table rows | Figures |
| ------: | -------: | ------: | -------: | ---------: | ------: |
|   8,192 |        — |       — |        — |          — | invalid |
|  16,384 |        5 |       — |        — |          — |  failed |
|  24,576 |        3 |    1345 |       10 |     **12** |   29/35 |
|  32,768 |        2 |     895 |       25 |          0 |   29/35 |
|  40,960 |        2 |    ~840 |        — |          0 |   29/35 |

Three things, in descending order of confidence.

The 8,192 row is not a measurement. The harness passes a maximum output of 8,192
tokens, so at that context the reading window was exactly zero and `plan_sections`
fell to its last resort of one section per segment. That was a defect in the code as
well as in the test — the answer can no longer claim more than half the window — and
the point has to be measured again.

**Neither the actions table nor the figures depend on the context, and both claims
that they did were wrong.** They were each made from one draw at each point. Repeated
across three seeds:

| Seed |       24,576 |       32,768 |     40,960 |
| ---: | -----------: | -----------: | ---------: |
|    7 | table, 29/35 | **—**, 29/35 | **—**, ~29 |
|  101 | table, 25/35 | table, 27/35 |      table |
|  202 | table, 27/35 | table, 23/35 |      table |

Two things follow, and the second matters more.

The context does nothing measurable between 24,576 and 40,960 — not to figures kept,
not to whether the protocol carries its table. So **the window can be chosen for what
the machine can hold rather than for quality**, which is the answer the 8 GB target
needed. Figures range 23 to 29 across seeds at a fixed context, which is wider than
any difference between contexts; "29 of 35 at every context" was three coincidences
read as a constant.

**The variation that matters is between draws, not between settings.** Both runs
missing the table are at seed 7 — the harness default, and therefore the draw behind
every early single-seed comparison in this project and the draft that was read against
the written protocol. "This draft has no actions table" became "this context produces
no actions table" on that basis.

This is the third time the same mistake has been recorded here: `granite4.1:8b`
scoring 22, 19 and 6 on three seeds is the same finding wearing different clothes.
Knowing about the trap is evidently not the same as being out of it, so the rule now
is that no comparison at a single seed is written down as a property of anything.

What remains genuinely open is why one draw in eight omits the table entirely, since
that is the failure a reader would notice first.

One distinction to keep. The first phase currently **condenses** — it writes detailed
notes keeping every point — rather than **indexing**, which is what a person does
when they scan a transcript marking where each subject lives. An index would be
cheaper again. `topics.rs` already does it and is compiled for evaluation only:
what was measured and rejected there was _writing_ subject by subject, which produced
a document longer than the transcript. Finding the subjects was never the failure.

What this cannot do is worth stating plainly. It enforces the shape of an answer and
never its substance. A model that writes 211 bytes about an eighty-minute meeting
will not be made to understand it by being asked again — the floor rises, the ceiling
does not.

### 2. Measure the approved baseline

Run the complete path on an M1 Mac with 8 GB RAM. Record elapsed time, peak memory, swap behaviour, disk use, cancellation time, and whether the interface remains usable while work runs.

The current M1 Pro/16 GB measurements are valuable development evidence, not the release baseline.

### 3. Validate runtime and speaker distribution — the next thing to do

**Three of four sidecars now build and are self-contained.** On an Apple Silicon machine,
`localog-whisper` (3.3 MB), `localog-speaker-diarization` (23.4 MB) and `localog-speaker-embedding`
(14.5 MB) each link nothing outside `/usr/lib` and `/System/Library`, so there is no library to place
beside them. `npm run build:sidecar` builds all three from pinned revisions. Whisper transcribes real
audio with Metal; the embedding sidecar reproduces the reference meeting's grouping exactly.

What to do next, in this order, because each unblocks what follows:

1. ~~Package the application.~~ **Done on 15 August 2026, and it works.** `LocaLog.app` bundles all
   five sidecars into `Contents/MacOS` with the target suffix stripped, which is the shape the
   resolver expects; each runs from inside the bundle, and the FFmpeg licence texts ship in
   `Contents/Resources`. The application is ad-hoc signed, with no team identifier — enough to run
   locally, not enough to distribute.

   The `.dmg` step fails, and not for a reason in this project: `create-dmg` drives Finder over
   AppleScript to arrange the window, and the build machine has not granted Automation permission
   for Finder, so it stops with `-1743`. The application bundle is complete either way. A machine
   with that permission, or a CI runner, produces the disk image.

2. **Answer the system-audio question inside that package.** macOS gates capture behind Screen &
   System Audio Recording and hands an unauthorised tap silence rather than an error, and it will
   not attribute a permission request from a process below a terminal. So the capture code has never
   been observed to work, which is a different claim from it being broken. A packaged application
   has its own signed identity, shows its own dialog naming itself, and is where a user meets this
   anyway. Until it is answered, nothing more should be built on the assumption that the tap works.

3. **FFmpeg**, the only runtime with no sidecar at all: the application still requires it on the
   machine. It is the most predictable item left, because the licensing turns out to be the easy part
   and the build is the work.

   Licensing is straightforward here for one reason: FFmpeg is invoked as a separate executable
   rather than linked, so this is two programs talking and not one derived work. LocaLog is
   GPL-3.0-or-later and FFmpeg's GPL components are GPL-2.0-**or-later**, which is compatible; built
   without the GPL-only components it is LGPL and simpler again. What is still owed is the ordinary
   obligation — ship the licence texts, and be able to supply the source for the exact build.

   The build should be small rather than stock. The application uses FFmpeg to probe a file, turn
   anything into 16 kHz mono PCM, and encode Opus. A stock build is tens of megabytes of encoders,
   filters and network protocols that are never called; configured with `--disable-everything` and
   only the demuxers, decoders and encoders in use it is a few megabytes — less to ship, far less to
   audit, fewer advisories arriving for code that is never reached, and it avoids the GPL-only pieces
   by construction rather than by argument.

   Two alternatives were considered and rejected. Writing the decoding is not sensible: reading MP3,
   AAC and MP4 correctly is decades of accumulated edge cases, and it is the one dependency worth
   having. Using each platform's own decoders — AVFoundation, Media Foundation — removes the binary
   but costs three implementations with different format support, and worse, the same recording would
   produce different working audio and therefore different transcripts depending on the machine. For
   a tool whose output people rely on as a record, one decoder everywhere is worth more than the
   binary it would save.

4. **The M1 / 8 GB baseline**, which needs that hardware and has never been measured.

Then, for speaker separation: **replace the embedding model**, which is trained on Chinese and has
never been revisited. sherpa-onnx publishes several, and swapping one is a file change. This used to
cost eight minutes an attempt and now costs thirty-nine seconds, so trying two or three is an
afternoon — a question that was too expensive to ask casually is now cheap, which is the clearest
dividend of moving to embeddings. Also long recordings and overlapping speech. Decide whether the optional setting becomes the default once a verified runtime
exists, and what the review interface needs for renaming, reassignment and merging labels.

**That question has now been asked, and the answer reorders this list.** Three protocols from the
reference meeting differing only in their speaker labels — none, the embedding pass's twelve, and a
scattered fifty-four — kept 24, 20 and 23 of 35 stated figures. No benefit from the labels is
visible, and across all three drafts the string `Speaker N` appears **once**: the generator is handed
the speakers and attributes almost nothing.

The same runs exposed two failures that matter more than the speaker count. **No draft produced the
table of next steps** the style asks for explicitly and twice — which means the unowned-tasks check
shown beside a draft can never fire on this model, because it reads table rows and there are none.
And the unlabelled run produced **no headings at all**, 98 bullets against a style asking for
numbered sections.

That instruction-adherence question has since been asked too, across the installed models and at
three seeds each. The instructions are followable: both larger models produce the table
`qwen3.5:4b` never produced in five runs. **So the prompt is not the fault**, and the largest lever
available is the model.

What the seeds changed is which model. On identical input `granite4.1:8b` keeps 22, 19 and 6 of 35
stated figures — a run that loses five sixths of a meeting's figures is not a tool for producing a
record, and nothing in the output says which run it was. `gemma4:12b` keeps 27, 29 and 31, better at
its worst than granite at its best, at about 1.5× the time.

**`gemma4:12b` is the candidate for the default**, and the endorsement `qwen3.5:4b` carries in the
evaluation predates any structural measurement. See `docs/MODEL_EVALUATION.md`.

### 4. Harden the product

Add archive and basic backup/restore, finish the language settings, confirm audio playback, perform the accessibility and keyboard pass, audit ordinary logs and privacy boundaries, and remove or isolate unused experimental generation code.

### 5. Package and broaden platform validation

Only after the workflow and runtime choices are credible should the project enable bundling, signing, notarisation, and packaged Windows/Linux validation.

## Running the evaluation harnesses

They take minutes to hours against real models, which makes a few things easy to get
wrong and expensive to get wrong:

- **Check what is already running before starting anything.** Two generations on a
  16 GB machine compete for memory, spill onto the CPU and produce timings that mean
  nothing — and one run was left going for nine hours while a second was started
  beside it. `ps` and `ollama ps` answer this in a second.
- **`ollama ps` is the honest view of memory**, not the model's size on disk. A 7.1 GB
  model at a 40,960-token context occupies 14 GB. Anything reporting a CPU share
  rather than 100 % GPU is swapping and its timings should be discarded.
- **Never wait on a `pgrep` for a string the waiting command itself contains.** It
  matches its own command line and waits forever, which is how the nine hours passed.
- **Vary the seed.** A repeat at a fixed seed reproduces the run rather than testing
  it, and the spread between seeds has been larger than the spread between models.
- **Read the drafts, do not only count them.** Both faults worth fixing in August
  2026 — a literal `\n` printed mid-sentence, and an evidence check defeated by a
  model behaving well — were invisible to every structural measure.

## Definition of done for a milestone

Every milestone should end with:

- what was tested;
- measurements where they matter;
- risks discovered;
- a keep/change decision;
- the documentation update made in the same change;
- a clear statement of whether code is production, provisional, experimental, or discarded.

Green unit tests alone are not enough for a milestone involving real models, long files, user experience, or distribution.
