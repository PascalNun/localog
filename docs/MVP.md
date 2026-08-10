# The first useful version

The v0.1 goal is deliberately simple: prove that LocaLog can take an imported recording and help someone produce a trustworthy, editable protocol entirely through a local desktop workflow.

```text
Project and meeting → imported recording → local transcript
→ transcript review → local protocol draft → Markdown editing/export
```

This is not a disposable demo. The interface, privacy behaviour, storage, and recovery need to be credible enough that a person can evaluate the product honestly.

## Included in v0.1

### A complete meeting path

- create, rename, list, and eventually archive projects and meetings;
- import one audio or video source into a meeting;
- probe the source and create a safe, regenerable working-audio file;
- transcribe locally through the supervised whisper.cpp boundary;
- review timestamped transcript segments, correct text, and rename provisional speaker labels;
- resolve project and global vocabulary into the job that produced a transcript or protocol;
- use a named professional protocol style rather than an arbitrary prompt box;
- generate a local protocol draft through a validated provider boundary;
- edit Markdown with autosave, revision history, review state, and restoration;
- export Markdown or deterministic plain text to a location chosen by the user;
- recover queued, running, cancelled, failed, and interrupted work without presenting partial files as finished.

### The surrounding experience

- a persistent, quiet project sidebar;
- a start state that makes importing a recording the obvious first action;
- project, meeting, transcript-review, protocol-editor, library, and settings views;
- warm light and dark themes using locally bundled Barlow;
- keyboard access, visible focus, text scaling, reduced-motion behaviour, and purposeful empty/error/recovery states;
- progressive disclosure for model and runtime details.

## Explicitly outside v0.1

- microphone or system-audio recording;
- collaboration, sharing, accounts, cloud sync, calendars, live bots, or mobile applications;
- DOCX/PDF export or a template designer;
- semantic search across projects;
- automatic finalisation of a protocol;
- a public provider or plugin SDK;
- arbitrary model marketplaces or model training;
- a permanent AI chat interface;
- avatars, account controls, generic dashboards, or decorative controls taken from reference images without a product reason.

Known transcription and diarisation models may be offered for explicit download. The user chooses a quality outcome, and the application verifies the files before using them. It must not silently fetch a runtime or model.

## What “done” means

The workflow is ready for v0.1 evaluation when:

1. a user can complete the path without a terminal on a supported development installation;
2. imported originals remain unchanged and the managed data location is documented;
3. a restart preserves projects, revisions, edits, settings, and job history;
4. transcript and protocol drafts remain editable and reviewable;
5. generation records the provider, model, settings, style, vocabulary, input checksums, and application version available at the time;
6. cancellation and failure leave the last stable work intact and explain how to continue;
7. the interface remains usable while heavy work runs and feels immediate for ordinary interactions;
8. representative German and English checks are recorded, with quality limits stated plainly;
9. the M1/8 GB baseline has been measured before performance is called acceptable;
10. the remaining runtime, packaging, accessibility, privacy, and backup risks are named rather than hidden.

## Current status

The shell, storage, import path, revision model, editing surface, and most job boundaries are present. Real local runtime work and long-meeting protocol quality are still being validated. See [PLAN.md](PLAN.md) for the current status rather than treating this document as a claim that every item already exists.
