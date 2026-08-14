# Current plan

This is the short answer to “where is LocaLog now, and what should happen next?” It is intentionally a current-status document, not a diary of every experiment.

The product and architecture documents describe the destination. The decision log records choices. This document describes what the code can honestly claim today.

Last reviewed: 13 August 2026.

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

Speaker labels must remain provisional. They are not confirmed identities.

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
