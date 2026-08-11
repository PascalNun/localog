# Architecture

This document explains how LocaLog keeps demanding local work out of the interface, preserves professional data, and remains portable across macOS, Windows, and Linux.

The architecture is intentionally modest. It is a small desktop application with clear boundaries, not a service platform waiting for a business model.

## In plain language

- Svelte and TypeScript present projects, meetings, transcripts, and protocols.
- Tauri provides the desktop shell and a controlled boundary to native capabilities.
- Rust owns product rules, storage, jobs, files, and local runtime integration.
- SQLite remembers relationships, lifecycle, revisions, and job state.
- Immutable versioned files hold the content of committed recordings, transcripts, and protocols.
- Supervised local processes perform media and model work away from the interface thread.
- Platform-specific paths, permissions, process details, acceleration, recording, and packaging stay at the edges.

## The main boundary

The frontend does not construct shell commands, write canonical artifacts, or own long-running processes. It sends typed requests across the Tauri boundary and receives small, bounded progress and state events.

Rust translates those requests into storage operations or runtime-specific commands. The rest of the application sees a stable request and result instead of a whisper.cpp flag, an Ollama response shape, or a platform-specific process trick.

## The repository shape

The current application is one modular Rust crate and one Svelte/TypeScript frontend:

```text
src/                 interface and browser-side workflow boundary
src-tauri/src/       Rust commands, domain rules, storage, jobs, adapters
tests/               cross-boundary tests as they become useful
fixtures/            synthetic, redistributable material only
spikes/              isolated validation studies
docs/                product, decisions, current status, and evidence
```

New crates or packages need evidence. A boundary that has not become real should remain a clear module.

## Storage authority

SQLite is authoritative for:

- identity and relationships;
- stable meeting lifecycle;
- revision metadata;
- active and historical job state;
- artifact paths, checksums, and provenance records.

An immutable versioned file is authoritative for the content of that committed revision. The file is written and made durable before the database transaction makes the revision visible. Working autosaves live separately from committed revisions.

The transcript’s canonical committed content is structured JSON. Database projections may make reading faster, but they are derived and never become a second editable canonical copy.

Imported originals are immutable. Normalised audio is a regenerable cache linked to the source checksum and settings.

## Meeting state and job state

The meeting has a stable lifecycle:

```text
draft → source_ready → transcript_ready → protocol_draft → reviewed → archived
```

A job has a temporary state:

```text
queued → running → cancelling → completed
                         ├── failed
                         └── interrupted
```

The UI may combine these into a sentence such as “Transcript ready — protocol generation failed”, but persistence and domain logic keep the axes separate.

## Background work

Media processing, inference, migrations, large file operations, and database work never run on the interface thread. Workers report bounded progress rather than forwarding raw process output.

The first implementation uses one heavy-work lane. Transcription, generation, diarisation, and model downloads compete for the same memory and disk bandwidth. A general scheduler is not needed until measurements prove that it is.

Cancellation signals the supervised process, waits for a bounded grace period, and escalates when necessary. A cancelled job does not advance the stable meeting lifecycle.

Logs remain bounded and do not contain transcript, protocol, or audio content.

## Runtime boundaries

The application needs only a few narrow capabilities:

- media probing and normalisation;
- transcription with timestamps, uncertainty, provenance, cancellation, and progress;
- optional speaker separation with provisional labels;
- protocol generation with validated output and provenance;
- explicit Markdown and plain-text export.

The first concrete choices are:

| Area               | Current direction                             | Status                                                                  |
| ------------------ | --------------------------------------------- | ----------------------------------------------------------------------- |
| Desktop shell      | Tauri 2                                       | Accepted                                                                |
| Interface          | Svelte with TypeScript                        | Accepted                                                                |
| Core               | Rust                                          | Accepted                                                                |
| Storage            | SQLite plus immutable files                   | Accepted                                                                |
| Media              | FFmpeg/FFprobe through supervised processes   | Accepted for the vertical slice                                         |
| Transcription      | whisper.cpp sidecar boundary                  | Accepted direction; packaging still open                                |
| Speaker separation | sherpa-onnx sidecar plus verified ONNX models | Accepted v0.1 direction; sidecar build path exists, quality provisional |
| Protocol provider  | narrow port, Ollama first                     | Accepted for development and early previews; final public runtime open  |
| Protocol source    | Markdown                                      | Accepted                                                                |

The model-download path is consent-gated and verifies known files by checksum. It is not a model marketplace. The user chooses a quality outcome rather than a runtime path. The runtime-bundling and distribution details remain open.

The protocol model is a global application preference stored in Settings. The UI exposes a small, curated catalogue with hardware and evaluation labels. It may recommend a model, but it does not ask for a model per protocol. A job snapshots the resolved model identifier, digest, runtime and settings so changing the global preference never changes an existing artifact.

## Provenance and repeatability

LocaLog does not promise byte-identical model output. It records the inputs that matter:

- provider and runtime versions;
- model identifier and digest where available;
- resolved generation or transcription settings;
- style and vocabulary revisions;
- normalised input checksums;
- application version.

This makes a result explainable and repeatable in its inputs without pretending that local model inference is mathematically identical across every machine and runtime build.

## Files and app-managed storage

The working directory is app-managed and documented. It contains the database, managed recordings, working files, committed artifacts, downloaded models, and bounded logs. User-selected exports are written only where the user chooses.

The design deliberately leaves room for backup/restore and a portable project bundle. App-managed storage must never become a professional data trap.

## Portability

The domain model and storage rules do not depend on macOS. Platform adapters will handle:

- application-data locations and user file pickers;
- process groups and termination;
- permissions and audio capture;
- hardware acceleration;
- runtime discovery;
- signing, notarisation, installers, and updates.

macOS 13+ on Apple Silicon is the provisional first baseline, with an M1 Mac and 8 GB RAM as the weakest representative test machine. Windows and Linux are intended platforms, but packaged support must be validated rather than inferred from portable Rust code.

## Security and privacy boundaries

- Commands use argument arrays, never shell interpolation.
- Paths are resolved inside the managed root and hostile filenames are rejected.
- Loopback providers have bounded responses, timeouts, and explicit readiness states.
- No cloud service, account, telemetry, analytics, or remote feature flag is required.
- Runtime binaries, fonts, models, and third-party assets require provenance and licence review before distribution.

## What this architecture deliberately does not contain

There is no universal workflow engine, public provider SDK, broad capability negotiation layer, cloud service, dedicated export crate, or multi-crate platform decomposition. Those may become reasonable later, but the current workflow has not earned them.
