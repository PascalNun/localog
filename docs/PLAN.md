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

- Recording from the microphone or system audio is not implemented. The study in
  `spikes/meeting-recording/` defines the platform-neutral recorder contract and has the microphone
  and the survive-a-kill mechanism working. System audio is written but **has never been observed to
  capture anything**: macOS gates it behind Screen & System Audio Recording, hands an unauthorised
  tap silence rather than an error, and will not attribute a permission request from a process
  running below a terminal. Not permitted and not working are indistinguishable from there, so that
  has to be answered from inside the packaged application, which has its own signed identity.
  Storing a recording as Opus is built and measured; nothing writes into it yet.
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

4. Retry one failed section or pass rather than discarding the complete run.
5. Compare the result with the existing human reference on completeness, correctness, attribution, length, and editing effort—not length alone.
6. Repeat the same workflow with an English meeting or synthetic equivalent.

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

1. **Package the application** — `npm run tauri:build`. Nothing has ever been bundled. This is the
   highest-value step because it settles several questions at once: whether the release config
   places all three sidecars, whether the resolver finds them inside the bundle, and whether the
   whole path runs against shipped runtimes rather than a developer's machine.

2. **Answer the system-audio question inside that package.** macOS gates capture behind Screen &
   System Audio Recording and hands an unauthorised tap silence rather than an error, and it will
   not attribute a permission request from a process below a terminal. So the capture code has never
   been observed to work, which is a different claim from it being broken. A packaged application
   has its own signed identity, shows its own dialog naming itself, and is where a user meets this
   anyway. Until it is answered, nothing more should be built on the assumption that the tap works.

3. **FFmpeg.** The only runtime with no sidecar at all: the application still requires it on the
   machine. It needs the same treatment as the others _and_ a licensing review before anything ships,
   which is a decision rather than a build — the obvious builds are GPL, and what that means for
   distributing them alongside this application has not been worked out.

4. **The M1 / 8 GB baseline**, which needs that hardware and has never been measured.

Then, for speaker separation: a multilingual or German-suited embedding model, long recordings,
overlapping speech. Decide whether the optional setting becomes the default once a verified runtime
exists, and what the review interface needs for renaming, reassignment and merging labels.

Underneath all of it, still unmeasured: **whether speaker labels improve the protocol at all.** A
protocol generated with speakers and one without, from the same meeting, would settle whether any of
this earns its place. It is the cheapest experiment on this list and the one that could remove the
most work.

### 4. Harden the product

Add archive and basic backup/restore, finish the language settings, confirm audio playback, perform the accessibility and keyboard pass, audit ordinary logs and privacy boundaries, and remove or isolate unused experimental generation code.

### 5. Package and broaden platform validation

Only after the workflow and runtime choices are credible should the project enable bundling, signing, notarisation, and packaged Windows/Linux validation.

## Definition of done for a milestone

Every milestone should end with:

- what was tested;
- measurements where they matter;
- risks discovered;
- a keep/change decision;
- the documentation update made in the same change;
- a clear statement of whether code is production, provisional, experimental, or discarded.

Green unit tests alone are not enough for a milestone involving real models, long files, user experience, or distribution.
