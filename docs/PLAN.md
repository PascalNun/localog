# Current plan

This is the short answer to “where is LocaLog now, and what should happen next?” It is intentionally a current-status document, not a diary of every experiment.

The product and architecture documents describe the destination. The decision log records choices. This document describes what the code can honestly claim today.

Last reviewed: 14 August 2026.

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

There is evidence the setting is softer than it looks: asked for eleven, the sampled run produced ten
distinct labels and the whole-recording run landed only eight on segments. Whether asking too high
fragments people proportionally or is absorbed is being measured across counts of 6, 10, 14 and 20.
If it is absorbed, the number is a ceiling rather than a target and the interface should ask people
to guess generously, which is a question they can answer.

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

- Recording from the microphone or system audio is not implemented.
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
3. Quantities kept, figures invented, and length against the recording are measured and shown.
   Unsupported statements, actions without owners, and missing sections still need checks that can
   be relied on.
4. Retry one failed section or pass rather than discarding the complete run.
5. Compare the result with the existing human reference on completeness, correctness, attribution, length, and editing effort—not length alone.
6. Repeat the same workflow with an English meeting or synthetic equivalent.

### 2. Measure the approved baseline

Run the complete path on an M1 Mac with 8 GB RAM. Record elapsed time, peak memory, swap behaviour, disk use, cancellation time, and whether the interface remains usable while work runs.

The current M1 Pro/16 GB measurements are valuable development evidence, not the release baseline.

### 3. Validate runtime and speaker distribution

Build and validate the target-specific whisper.cpp, FFmpeg, and sherpa-onnx sidecars without asking a normal user to browse for executables. Before distribution, review licensing, checksums, signing, notarisation, updates, offline behaviour, and model storage.

For speaker separation, test a multilingual or German-suited embedding model, long recordings, overlapping speech, and the M1/8 GB machine. Then decide whether the optional setting should become the default when a verified runtime is available, and what the review interface needs for renaming, reassignment, and merging labels.

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
