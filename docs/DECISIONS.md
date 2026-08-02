# Decisions and open questions

This is the project’s memory. It records not only what was chosen, but why, what evidence supports it, and which risks remain. Readers looking for the product story should begin with [PRODUCT.md](PRODUCT.md); readers looking for definitions of technical terms can use the [documentation guide](README.md).

Status values: **Accepted**, **Proposed**, **Approval required**, **Deferred**.

## Decision summary

| ID    | Decision                                                                                                             | Status   | Rationale                                                                                                                                      |
| ----- | -------------------------------------------------------------------------------------------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| D-001 | LocaLog is desktop-first and local-first; no cloud/account/telemetry in the core workflow                            | Accepted | Product trust and sensitive professional data                                                                                                  |
| D-002 | Hierarchy is Project → Meeting → Recording(s)/Transcript/Protocol/Exports; no Inbox                                  | Accepted | Context and professional organisation are the differentiator                                                                                   |
| D-003 | Imported audio is the v0.1 input; integrated recording is Phase 2 and omitted from the first functional shell        | Accepted | System audio and permissions are cross-platform risks; inactive controls must not compete with the working path                                |
| D-004 | Tauri desktop shell + Rust application core + Svelte/TypeScript UI                                                   | Accepted | Responsive desktop shell and explicit native/process boundary                                                                                  |
| D-005 | Modular monolith, typed UI boundary, supervised external engines                                                     | Accepted | Shell, storage, and process spikes validated the boundaries without services, plugins, or workflow infrastructure                              |
| D-006 | App-managed working storage for v0.1, with explicit exports and documented data location                             | Accepted | Protects storage invariants without making professional data opaque                                                                            |
| D-007 | FFmpeg normalization plus a narrow validated local transcription boundary                                            | Accepted | Installed-runtime spike proved probing, normalization, cancellation, provenance, timestamps, and validation                                    |
| D-008 | Validate Ollama for spikes and early technical previews behind a narrow local provider boundary                      | Accepted | Ollama accelerates validation but is not accepted as the final public distribution model                                                       |
| D-009 | Markdown is canonical protocol source; Markdown/TXT are first exports                                                | Accepted | Inspectable, portable, and simple to transform                                                                                                 |
| D-010 | Global defaults < project defaults < meeting overrides; snapshot resolved inputs and provenance at job start         | Accepted | Simplicity and repeatability of inputs without promising byte-identical inference output                                                       |
| D-011 | Generated protocols are drafts and revisions are retained                                                            | Accepted | Professional review and recovery                                                                                                               |
| D-012 | Svelte with TypeScript is the frontend                                                                               | Accepted | Compact fit for the bounded desktop workflow                                                                                                   |
| D-013 | Stable meeting lifecycle and transient job state are separate persisted axes                                         | Accepted | Domain state must not be distorted by temporary processing activity                                                                            |
| D-014 | Phase 0 discovers installed or user-provided models only; no model-download manager                                  | Accepted | Validate the workflow before taking on acquisition, licence, storage, and consent UX                                                           |
| D-015 | Provisional v0.1 baseline is macOS 13+, Apple Silicon; weakest test machine is M1/8 GB                               | Accepted | Provides a concrete performance target subject to spike and packaging evidence                                                                 |
| D-016 | Publish the repository as `PascalNun/localog` on GitHub                                                              | Accepted | Public development makes the product reasoning and progress visible; private directional studies remain outside the repository                 |
| D-017 | Protocol styles are controlled professional presets, not arbitrary per-meeting prompt boxes                          | Accepted | Keeps the primary workflow understandable while permitting structured internal LLM instructions                                                |
| D-018 | Interface language and meeting/content language are separate settings                                                | Accepted | Transcription and protocol language must not determine application localisation                                                                |
| D-019 | SQLite metadata/jobs plus immutable versioned artifact files; structured JSON is canonical for committed transcripts | Accepted | The storage spike validated file-before-database visibility, reconciliation, checksums, working-state separation, and long-transcript handling |
| D-020 | Interface quality is a core requirement equal to local-first behaviour and data reliability                          | Accepted | The product differentiator is a calm, immediate, trustworthy professional workflow, not inference engines alone                                |
| D-021 | Barlow is the primary application typeface                                                                           | Accepted | Its clear, contemporary character supports the calm professional interface; font assets must be bundled locally rather than fetched at runtime |
| D-022 | `whisper.cpp` is the first candidate for the distributable transcription runtime                                     | Proposed | Installed Python Whisper validated the contract but was too slow/heavy; no compatible installed `whisper.cpp` model was available              |
| D-023 | License LocaLog under `GPL-3.0-or-later`                                                                             | Accepted | Strong copyleft preserves the freedom to use, study, modify, and redistribute the application while allowing commercial use                    |
| D-024 | Use UUIDv7 opaque identifiers and UTC Unix-millisecond storage timestamps for the first vertical slice               | Accepted | Provides collision-resistant sortable identities and ordinary time storage without a custom ID system or a general time abstraction            |
| D-025 | Protocol review belongs to an exact revision; later edits become `changed since review`                              | Accepted | Keeps normal autosave simple while preserving the reviewed document and making later changes traceable                                         |
| D-026 | Starting protocol generation commits dirty transcript working state as its exact input revision                      | Accepted | Keeps the user action clear while ensuring generation never consumes mutable autosave state or an ambiguous transcript                         |
| D-027 | Normalized audio is a regenerable derived cache, not a user-visible immutable revision                               | Accepted | It protects the imported original, avoids duplicate professional artifacts, and permits safe regeneration when settings or runtimes change     |

## Architecture risks and tensions

1. **Local-first versus external runtimes.** Ollama is local but separately installed, versioned, and configured. A first adapter can accelerate validation, but a self-contained product may eventually need a supervised `llama.cpp` runtime.
2. **Portable artifacts versus relational authority.** SQLite is authoritative for identity, relationships, lifecycle, revision metadata, jobs, and artifact path/checksum records. The immutable file is authoritative for the content of its committed revision. The storage spike validated the basic dual-write/reconciliation sequence; low-disk, migration, real-process crash, and repair UX remain production hardening risks.
3. **Vocabulary expectations.** Prompt/context injection can improve recognition but does not guarantee transcription spelling. The UI must not promise training or perfect terminology.
4. **Cancellation versus resumability.** Sidecars can usually be terminated, but arbitrary inference checkpoints are runtime-specific. v0.1 resumes from durable stages, not from an exact token/audio frame.
5. **macOS-first versus portability.** Metal and macOS permissions should be adapters, not assumptions in domain/application layers.
6. **Bundling versus download size/licensing.** `ffmpeg`, model files, and inference binaries have provenance, architecture, signing, size, and licence implications. Resolve before distribution.
7. **Markdown editor versus rich formatting.** A calm writing experience is required, but premature rich-text adoption can make Markdown round-tripping unreliable.
8. **Responsiveness versus process visibility.** High-frequency runtime output must be bounded and converted into throttled or batched progress events; the frontend must not mirror raw process streams.
9. **Professional portability versus managed storage.** App-managed storage protects invariants but cannot become opaque. User-selected exports, documented data locations, backup/restore, and a later portable project bundle remain required architectural paths.

## Approved Phase 0 constraints

- Ordinary navigation, selection, typing, and editing should generally respond within approximately 100 ms even during background processing.
- Media work, inference, migrations, and large file operations never run on the UI thread.
- Background work is supervised; progress traffic and logs are bounded, and content is excluded from ordinary logs.
- Do not retain large media or transcript bodies in duplicate memory buffers when streaming or incremental processing is practical.
- Start with a small number of modules. Do not begin with five Rust crates, a universal workflow engine, an export crate, a provider SDK, or broad capability negotiation.
- Profile before low-level optimisation; preserve architectural responsiveness first.
- Early visual studies were directional only. Avatars, account controls, sharing, rich formatting tools, recording controls, speaker automation, and generic dashboard elements are not inferred MVP requirements.
- The Phase 0 shell establishes the real navigation, light/dark tokens, typography, spacing, hierarchy, interaction states, accessibility foundation, and responsive behaviour; it is not disposable backend chrome.
- Before detailed UI work, document the design tokens, typography hierarchy, spacing/layout grid, sidebar/workspace behaviour, common control states, contextual-inspector rules, and visual acceptance criteria for key screens.
- Use locally bundled Barlow for the application interface with an appropriate system sans-serif fallback. Add only required weights/styles, and record font source, licence, version, and checksums before committing the assets.
- Raise technical trade-offs for review when they materially weaken the documented calmness, clarity, accessibility, or professional quality of the workflow.

## Accepted storage invariant after the spike

- SQLite is authoritative for identity, relationships, lifecycle state, revision metadata, job state, and artifact path/checksum records.
- An immutable versioned artifact file is authoritative for the content of that specific committed revision.
- A revision becomes visible only after its artifact file has been durably written and its database transaction has completed.
- Autosave working state is separate from immutable committed revisions.
- Imported original media is immutable.
- A versioned structured JSON artifact is canonical for each committed transcript revision. Any SQLite projection is derived and rebuildable, never a separately editable canonical copy.
- Transcript and protocol typing updates one replaceable working artifact. It never creates formal revisions continuously.
- Starting generation is an explicit downstream boundary: if transcript working state differs from its base revision, LocaLog commits it first and records that exact transcript revision on the generation job.
- Marking a protocol reviewed applies to one immutable revision. Autosaved edits based on it preserve the reviewed revision and present the working document as `changed since review`.
- Normalized transcription audio is stored only as a checksummed derived cache linked to its source checksum and normalization settings. It may be regenerated or removed without changing the imported original or transcript revision history.

## Phase 0B/0C implementation record

Implemented and checked on 2026-08-02:

- One Tauri/Rust crate and one Svelte/TypeScript frontend; no workflow framework, provider SDK, storage crate, or export crate was introduced.
- Locally bundled Barlow 400/500/600, semantic light/dark tokens, a persistent/overlay sidebar, focused workspaces, contextual inspectors, keyboard-visible focus, reduced-motion handling, and compact-window rules.
- Start, project overview, new project, new meeting/import, meeting stage, transcript review, protocol editor, library, and settings surfaces using synthetic content only.
- A typed in-memory `WorkflowBridge` fake covering import, transcription, cancellation, failure, retry, transcript edits, generation, protocol edits, review state, and Markdown/plain-text browser export.
- Meeting lifecycle and active-job state remain distinct in the fake model and tests. A cancelled or failed job does not advance stable lifecycle.

Measurements and checks:

- Production web assets: approximately 106 kB JavaScript, 25 kB CSS, and three Barlow WOFF2 files totalling approximately 67 kB before gzip.
- The production frontend build completed in under one second on the development machine; this is a scaffold observation, not the M1/8 GB acceptance measurement.
- The workflow was visually exercised at 1280 × 720 and at a 900 × 700 compact breakpoint in both themes. No browser console errors or warnings were observed.
- Type checking, linting/formatting, four fake-boundary tests, the production frontend build, Rust formatting, one Rust unit test, and warning-denied Clippy all passed.

Keep/change decision:

- **Keep** the visual tokens, navigation/workspace composition, contextual-inspector pattern, typed UI boundary, and fake adapter as the default automated/demo runtime.
- **Rewrite behind the boundary** when spikes add persistence, supervised processes, and real providers. The in-memory fake is not persistence or recovery architecture.
- **Keep provisional** the waveform app icon and browser-only export implementation; revisit identity separately and replace export with the Rust file boundary in the vertical slice.

Known risks:

- Refresh or restart loses all fake workflow changes; no storage or recovery claim is made.
- Timing and responsiveness under real CPU/GPU load remain unmeasured.
- Transcript editing and protocol autosave need long-document, failure, revision, and accessibility evidence in their focused spike.
- Native distribution signing, notarisation, sandbox behaviour, and the final runtime/provider model remain unresolved.

## Phase 1A durable hierarchy implementation record

Implemented and checked on 2026-08-02:

- Added the first production schema inside the existing Rust/Tauri crate: projects, meetings, and one-to-many recording metadata. No storage crate, ORM, service process, or workflow framework was introduced.
- Creating a meeting and its pending imported-source record uses one SQLite transaction, preserving the no-orphan-source invariant before the real copy job exists.
- SQLite uses WAL mode, `synchronous = FULL`, foreign keys, a bounded busy timeout, and an explicit schema version. A database created by a newer schema is refused without mutation.
- Project/meeting loading, creation, and title changes cross four narrow Tauri commands. Opening, migration, and every query run on the blocking executor rather than the interface thread.
- The native shell loads the durable hierarchy without flashing or seeding synthetic projects. The ordinary browser preview continues using the deterministic fake workspace.
- Storage startup and form submission failures have visible, bounded messages that do not expose SQL, content, or managed paths.
- UUIDv7 was selected for opaque sortable record identity. Timestamps are stored as UTC Unix milliseconds; meeting dates remain explicit `YYYY-MM-DD` professional values.

Verification:

- Six Rust tests cover reopen/restart persistence, meeting/source transactionality, foreign-key placement, hostile source names, durable title updates, and refusing a newer schema.
- Two additional TypeScript tests cover native-workspace hydration/writes and a bounded startup failure, while the existing fake lifecycle/job tests remain unchanged.
- The native application created an empty schema-v1 workspace and reopened it cleanly. No synthetic project or meeting was written to app-managed storage.

Keep/change decision:

- **Keep** the small `domain` and `storage` modules, direct `rusqlite` repository, blocking-worker command helper, and narrow TypeScript `WorkspaceStore` port as the Phase 1A starting shape.
- **Keep** browser-only synthetic fixtures for design and automated workflow development; native project/meeting state comes from SQLite.
- **Do not persist fake lifecycle advancement.** A native meeting remains durably `draft` until the later import job has actually committed an original source; transcript/protocol lifecycle will advance only with immutable artifact revisions.
- **Next at that milestone:** add the minimum durable job envelope and original-source staged copy/recovery path. This work is now recorded in the durable-import implementation section below.

Known risks and deliberate limits:

- At this milestone the pending recording stored only the selected display name; the later durable-import milestone supersedes that limitation.
- Committed transcript/protocol revisions, autosave, archive/restore, and exports were not part of this hierarchy slice.
- The first migration creates a new database. Backup/recovery-point behaviour must exist before a later migration transforms professional data.
- Database-busy, permission, low/disk-full, real-process crash, Windows filesystem, and repair UX remain Phase 1A/1C boundaries rather than claims made by this implementation.

## Phase 1A durable source-import implementation record

Implemented and checked on 2026-08-02:

- Schema v2 adds a minimal persistent job envelope for import, transcription, and generation kinds while implementing import stages only; schema v3 adds a recovery backfill for legacy pending meetings that need source reselection. Persisted states are `queued`, `running`, `cancelling`, `failed`, `cancelled`, `interrupted`, and `completed`.
- The user chooses a native file, then confirms its project and meeting before the source path enters the managed import command. Meeting, pending source artifact, and queued job are created in one SQLite transaction.
- The Rust worker streams the external file read-only through a 256 KiB buffer into a managed temporary file, calculates SHA-256 in the same pass, and persists truthful byte progress at a roughly 10 Hz ceiling.
- The temporary copy is flushed, synced, checked for a probable checksum duplicate, renamed to an opaque final managed path, and made visible through one SQLite artifact/lifecycle/job transaction. The meeting remains `draft` until that transaction completes.
- Duplicate content pauses for an explicit **Import another copy** or cancellation choice. LocaLog does not merge, discard, or reuse it silently.
- Missing or legacy source paths can be reselected for the same preserved meeting instead of forcing the user to recreate its context.
- Native progress, cancellation, interruption, retry, duplicate choice, source size/type, and external-original safety are presented through the existing typed UI boundary. The browser preview retains its deterministic fake and explicitly does not claim to store media.
- The official Tauri dialog plugin is the only new frontend/native capability. `sha2` provides incremental SHA-256; no scheduler, ORM, workflow framework, file-service process, or extra Rust crate was introduced.

Authority and recovery decision:

- SQLite is authoritative for import intent, job state, source metadata, and meeting lifecycle. The final managed file is authoritative for committed source bytes only after its database record exists.
- Pending or failed imports retain the external source path locally for restart-safe retry; a successful commit clears it. Ordinary logs, events, errors, and UI copy exclude the path.
- Startup turns abandoned running/cancelling jobs into `interrupted`. Partial temporary files are removed, while a validated duplicate-confirmation copy is retained for the user’s decision.
- If termination happens after final rename but before the metadata transaction, startup verifies the persisted size/checksum and completes the transaction. Unverifiable final copies are quarantined under `working/recovery/` and are never shown as complete.
- Media type is provisional extension classification until the accepted FFprobe adapter supplies content-based probing.

Verification:

- Rust tests cover the five explicit termination boundaries: before copy, during copy, after temporary-file durability, after final rename, and after database commit.
- Additional Rust tests cover original preservation, checksum/size/type metadata, byte-copy cancellation, injected permission and no-space failures, explicit duplicate acceptance, schema-v1 migration, hierarchy restart, hostile source names, and newer-schema refusal.
- TypeScript tests cover stable lifecycle versus jobs, cancellation, retry, native hydration/failure, and restart-visible interrupted imports routed to the correct meeting.
- The native Tauri app launched with the dialog capability and schema migration. Browser visual checks covered project-first import, source-ready truthfulness, accessible labels, and the compact 900 × 700 layout.

Keep/change decision:

- **Keep** the job envelope, staged source layout, incremental hash/copy, explicit duplicate pause, startup reconciliation rules, throttled full-snapshot event, and narrow `WorkspaceStore` integration.
- **Keep** the import stages specific. Do not add a scheduler or general workflow-definition layer when persistent fake transcription/generation begins.
- **Change later** from extension classification to FFprobe metadata.

Known risks and deliberate limits:

- Real low-disk and permission failures are handled from OS errors but automated deterministically through injected boundaries; destructive device-level tests were not run.
- Directory syncing is implemented on Unix/macOS. Windows durability, rename, and quarantine behaviour still require its platform test lane.
- A quarantined orphan is retained safely but does not yet have repair/cleanup UI. Backup/recovery points must precede migrations that transform professional content.
- Import does not yet probe duration, streams, codecs, corrupt content, or normalize media. Those belong to the next real media adapter, after the persistent fake workflow is approved.

## Phase 1A durable fake workflow implementation record

Implemented and checked on 2026-08-02:

- Schema v4 adds immutable transcript/protocol revision metadata, separate working-state pointers and checksums, exact generation-to-transcript links, protocol review identity, provenance snapshots, deterministic failure intent, and the last active meeting/workspace route.
- Committed transcript content is one structured JSON artifact containing schema version, meeting/revision identity, language, and ordered stable segments with millisecond bounds, editable speaker labels, text, and an optional review flag. SQLite does not contain a second editable segment copy.
- Committed protocol content is immutable Markdown. One separate transcript working JSON file and protocol working Markdown file use recoverable replacement; the previous complete working file remains until SQLite acknowledges the new checksum.
- Two narrow Rust traits describe transcription and protocol generation. The deterministic fake adapters report stage progress, observe cancellation, inject one-shot failures, and produce repeatable synthetic professional content without bypassing job, validation, staging, or commit logic.
- Starting generation commits dirty transcript working state first and records that exact revision as job input. Normal typing only updates autosave. Explicit revision, review, regeneration, and restoration boundaries retain immutable history.
- Marking reviewed creates and records one exact reviewed revision. Later autosaved editing keeps meeting lifecycle stable, preserves that revision, and derives the visible state `changed since review`.
- Startup changes abandoned processing work to `interrupted`, removes staged files, quarantines renamed-but-uncommitted artifacts, restores the database-acknowledged autosave backup when needed, and reopens the last durable meeting stage.
- The transcript workspace now provides timestamped editable segments, source context, search, generic speaker renaming, keyboard movement between segments, visible save states, and truthful job controls. The protocol workspace provides Markdown editing, native undo/redo, contextual find, text scaling, save/review status, explicit revisions, restoration, and transcript navigation.

Verification:

- Twenty Rust tests cover the durable import suite plus fake adapter determinism, committed revision persistence, transcript-to-generation input identity, autosave reopen, cancellation, injected failure, retry, interruption, staged and renamed output recovery, autosave rollback, review/change semantics, revision restoration, and meeting-stage reopen.
- Thirteen frontend tests preserve fake/native boundary behaviour, lifecycle/job separation, cancellation/retry, startup failure, and import recovery. Type checking, ESLint, Prettier, production build, Rust formatting, warning-denied Clippy, and the existing editor/spike checks pass.
- Browser interaction at 1440 × 900 and 900 × 700 exercised transcript search/edit structure, protocol editing, review followed by changed-since-review, revision controls, the contextual inspector/drawer, and accessible control names. No browser warnings or errors were observed.

Keep/change decision:

- **Keep** the stage-specific adapter traits, shared durable job envelope, structured transcript JSON, Markdown protocol artifacts, atomic working-file replacement, exact input/review links, full-snapshot event boundary, and purpose-built transcript/protocol workspaces.
- **Keep specific** transcription and generation orchestration. Do not turn the explicit stages into a workflow-definition system or provider ecosystem.
- **Rewrite only the fake adapter implementations** when real runtimes enter. Their persistence, validation, cancellation, progress, and provenance contracts remain.

Known risks and deliberate limits:

- The fake work runs in a supervised blocking task but not an external process group. Real adapter cancellation/termination must use the accepted process-supervision boundary.
- Transcript JSON is currently serialized and loaded as one bounded document. The 7,200-segment spike was responsive, but real long-recording integration should stream/hash where practical and remeasure on the M1/8 GB baseline.
- Windows atomic replacement, directory durability, and quarantine behaviour still require a Windows test lane; Linux packaging and path behaviour also remain unverified.
- Database-busy, real disk-full, application-process termination at every production boundary, migration backup, repair/cleanup UI, and polished backup/restore remain hardening work.
- The source-context transport remains deliberately limited in this fake milestone. Real probing, duration, waveform/playback authority, transcription quality, diarisation, real provider output, and export are not claimed.

## Storage and recovery spike result

The isolated crate under `spikes/storage-recovery/` tested the provisional authority model without wiring it into the application.

What was tested:

- SQLite in WAL mode with `synchronous = FULL` for meeting lifecycle, job state, revision metadata, and original-media path/checksum records.
- Durable file write, file sync, atomic rename, directory-chain sync, then database transaction; an injected failure between file durability and database commit did not expose a revision.
- Startup reconciliation of incomplete/unreferenced/missing files and queued/running/cancelling jobs; terminal jobs were preserved.
- A separate full-integrity mode for checksum scans, so ordinary startup does not read every large recording.
- Immutable committed revisions, replaceable autosave working state, source-preserving original import, verified reads, checksum mismatch detection, and hostile identifier rejection.
- A canonical structured JSON transcript with 7,200 synthetic segments (approximately 1.18 MB). No real meeting data was used.

Measurements on an Apple M1 Pro with 16 GB RAM running macOS 26.3 (not the weakest representative machine):

- Schema/open: approximately 2.4–3.4 ms.
- Durable artifact commit plus SQLite visibility: approximately 13–24 ms.
- Verified artifact read: approximately 0.9–1.2 ms; JSON parse: approximately 3.0–3.7 ms.
- Lightweight startup reconciliation: approximately 0.7 ms; full checksum scan of the 1.18 MB artifact: approximately 1.0 ms.
- SQLite main/WAL/shared-memory set after the synthetic commit: approximately 128 kB.

Keep/change decision:

- **Keep** SQLite as metadata/job authority and immutable versioned files as committed-content authority.
- **Keep** `rusqlite` as the initial Rust binding. Execute it on a bounded blocking worker; synchronous calls never run on the UI thread. The spike used bundled SQLite for deterministic validation, but final bundling remains part of distribution hardening.
- **Accept** a versioned structured JSON artifact as canonical for each committed transcript revision. Future SQLite segment/search rows, if needed, are derived, rebuildable, and not editable authority.
- **Keep** mutable autosave outside committed revisions and split lightweight startup reconciliation from an explicit/on-demand full integrity scan.
- **Retain the spike only as an executable reference and fault-test oracle. Rewrite the production repository/storage module behind the application boundary; do not import the spike crate.**

Risks and required production changes:

- Measurements must be repeated on the M1/8 GB baseline with much larger artifacts and a realistic number of meetings/files.
- The spike injected crash windows in-process; production work still needs subprocess termination, database-busy, migration, permission, low/disk-full, and real restart tests.
- Production transcript serialization/loading should stream and hash in one pass where practical instead of holding raw JSON and a parsed transcript in duplicate buffers.
- Windows-safe atomic autosave replacement and filesystem-specific durability behaviour require verification before cross-platform release.
- Backup/restore, repair UX, and policy for quarantining or deleting unreferenced files remain hardening work; recovery must report before destructive cleanup.

## Process supervision spike result

The isolated crate under `spikes/process-supervision/` tested Unix/macOS process mechanics with a purpose-built synthetic worker and descendant process.

What was tested:

- Direct executable launch with argument arrays, a controlled working directory, an allowlisted environment, and no shell.
- A dedicated process group for each job, including a spawned descendant.
- Typed progress parsing with approximately 100 ms throttling and a bounded event channel.
- Concurrent stdout/stderr draining with bounded diagnostic tails under thousands of synthetic lines.
- Cooperative SIGTERM cancellation, forced SIGKILL escalation, process-tree termination, single-heavy-job rejection, malformed progress, missing executables, and hostile-looking literal arguments.

Measurements on the M1 Pro/16 GB development machine:

- Synthetic process launch: approximately 0.7–1.3 ms.
- Six progress events crossed the boundary during a 550 ms high-frequency run.
- Diagnostic storage remained approximately 16 kB after 1,000 long stderr lines.
- Cooperative process-group cancellation completed in approximately 10–15 ms; the forced escalation path was also verified.

Keep/change decision:

- **Keep** one supervised process group per external job, direct argument-array execution, a controlled environment/working directory, bounded concurrent pipe readers, and a roughly 10 Hz typed progress ceiling.
- **Keep** one model-heavy lane initially. Separate lightweight import/export work only after measurements justify it; do not introduce a general scheduler.
- **Keep** cooperative termination followed by a short grace period and forced group termination. The production adapter must persist `cancelling` before signalling and record the terminal outcome after reaping.
- **Retain the spike only as an executable test oracle. Rewrite the supervisor behind a small platform adapter and application job service; do not import the spike crate.**

Risks and required production changes:

- The spike validates Unix process groups on macOS. Windows requires a Job Object adapter and its own descendant/cancellation tests.
- Runtime-specific parsers must never forward raw stdout/stderr or transcript/model content to the UI or ordinary logs.
- Real `ffmpeg`, transcription, and LLM runtimes may buffer output or ignore signals differently; each adapter needs contract tests.
- Application restart classification still relies on the accepted durable-job storage design; this spike did not wire process identity into SQLite.

## Media normalization and transcription spike result

The isolated crate under `spikes/media-transcription/` used only already-installed runtimes and models. It generated synthetic speech and a synthetic MP4 at test time; no media, model, or runtime was downloaded or committed.

What was tested:

- FFprobe structured discovery of audio/video streams and duration from a hostile-looking literal path.
- FFmpeg conversion of an AAC/video container into mono 16 kHz PCM WAV, with bounded progress and source checksum preservation.
- Real FFmpeg process-group cancellation, missing runtime/model/audio diagnostics, and malformed transcript rejection.
- Installed OpenAI Whisper CLI 20250625 with the user-provided `medium.pt` model, producing non-empty ordered timestamped segments and a JSON artifact.
- Provenance capture for runtime version, model name/size/checksum, resolved settings, normalized input checksum, transcript checksum, and timings.

Measurements on the M1 Pro/16 GB development machine:

- FFmpeg 8.1.2 normalized an 8.263-second synthetic MP4 in approximately 37 ms.
- Real FFmpeg process-group cancellation completed in approximately 27 ms.
- The installed `medium.pt` model was 1,528,008,539 bytes with SHA-256 `345ae4da62f9b3d59415adc60127b97c714f32e89e936602e85993674d08dcb1`.
- Hashing that model took approximately 17 seconds, proving model digests must be cached/computed outside startup rather than rescanned synchronously.
- CPU transcription took approximately 21.8 seconds for 8.263 seconds of audio (real-time factor 2.64) and produced two timestamped segments.

Keep/change decision:

- **Keep** FFprobe + FFmpeg behind a narrow media-normalization adapter with explicit arguments, structured probing/progress, temporary working output, source checksum protection, and process supervision.
- **Keep** the normalized contract as mono 16 kHz PCM WAV for the first transcription adapters.
- **Keep** a narrow transcription result containing language, ordered timestamped segments, resolved settings, runtime/model provenance, and checksums. Validate before committing the canonical JSON revision.
- **Reject** Python/PyTorch Whisper as the public distribution baseline: its runtime/model footprint and CPU real-time factor are unsuitable despite validating the contract.
- **Keep `whisper.cpp` proposed**, not accepted. Validate a user-provided compatible model with Metal on the M1/8 GB machine before selecting it for production.
- **Retain the spike as an installed-runtime contract test and reference. Rewrite production adapters behind the application ports; do not import the spike crate.**

Risks and required production changes:

- The installed Homebrew FFmpeg build enables GPL components. Do not redistribute it. Distribution needs a deliberately configured, licensed, checksummed, signed, and size-measured audio-focused build or another approved acquisition strategy.
- Transcription quality was only checked structurally against synthetic speech, not scored on representative consented multilingual/noisy meetings.
- Model hashing, loading, and inference must run off the UI thread; digest results should be persisted against stable file identity and recomputed only when needed.
- Long-file chunking, memory pressure, thermal behavior, vocabulary effectiveness, and Metal cancellation remain for the `whisper.cpp` validation.

## Phase 1B implementation record: media and local transcription boundary

Implementation date: 2026-08-02.

What was tested:

- Added a production-shaped FFprobe JSON parser and FFmpeg normalisation boundary for mono 16 kHz PCM WAV.
- Added a bounded child-process supervisor with cancellation polling, process-group termination on Unix, and capped stdout/stderr capture.
- Added a user-configured whisper.cpp command boundary and JSON transcript parser. The parser accepts the common `transcription` and `segments` arrays, validates non-empty timestamped text, and assigns a conservative `Speaker 1` label because diarisation is not an MVP requirement.
- Added schema v5 settings and `normalized_media` records. A normalised file is reused only when source checksum, settings-derived path, runtime/settings metadata, recorded byte count, and a freshly streamed file checksum all match; otherwise it is regenerated. The imported source is never overwritten.
- Added native Settings controls for existing whisper.cpp executable/model paths. No download manager or automatic runtime acquisition was introduced.

Measurements and environment:

- Rust suite: 23 tests pass, including probe parsing and whisper JSON mapping.
- The current development machine has FFprobe and FFmpeg available, but no whisper.cpp executable or compatible model. Real inference latency, memory use, long-file behavior, and Metal cancellation therefore remain unmeasured.

Keep/change decision:

- **Keep** the media/cache and process-supervision boundaries, with the normalized file treated as a regenerable derived cache rather than a professional artifact revision.
- **Keep** whisper.cpp as an opt-in user-provided adapter for the next technical validation; missing configuration becomes a durable, retryable job failure with an actionable Settings message.
- **Change before public distribution:** validate one compatible whisper.cpp build and model on macOS M1/8 GB, then repeat the same contract on Windows and Linux before selecting packaging or a bundled runtime.
- **Retain** the implementation as a boundary for the vertical slice, but do not generalize it into a provider/plugin SDK or workflow engine.

Known risks:

- whisper.cpp CLI flags and JSON details vary by build; the first real runtime fixture must lock the supported command contract.
- Large model hashing is currently performed when Settings status is requested; cache that provenance by file identity before exposing it in a frequent UI path.
- FFmpeg/FFprobe remain user-installed development dependencies until distribution licensing and packaging are explicitly decided.

Implementation follow-up, 2026-08-02:

- Schema v6 snapshots the resolved transcription runtime inputs on each durable transcription job: executable and model paths, runtime version, model digest/size, language, and normalization settings. Queue and retry capture a fresh snapshot; execution validates that the recorded files and provenance still match before invoking the runtime.
- Normalized-cache reuse now streams and verifies the cached file checksum and byte count, in addition to the source checksum, derived path, runtime version, and settings. A stale or modified cache is regenerated without touching the imported original.
- These safeguards improve provenance and crash recovery without promising byte-identical model output. The implementation remains a narrow vertical-slice boundary, not a generalized workflow or provider framework.

## Local protocol-provider spike result

The isolated crate under `spikes/local-provider/` exercised an already-installed Ollama runtime and model through loopback HTTP only. It did not download a model, call a pull endpoint, or make a non-loopback request. The installed coding model was a convenient contract fixture, not a product-model selection.

What was tested:

- Loopback-only endpoint validation, runtime-version discovery, installed-model discovery, and exact model-name/digest capture before generation.
- A controlled protocol-style request with versioned style and vocabulary inputs, resolved settings, timestamped transcript input, and a JSON-schema-constrained Markdown response.
- Streaming newline-delimited responses with bounded response size and approximately 10 Hz typed progress updates rather than token-by-token UI traffic.
- Required-section validation, malformed/oversized output handling, cancellation by closing the in-flight request, and a post-cancellation health check.
- Rejection of an uninstalled model without attempting an automatic pull, plus provenance capture for provider/runtime/model, settings, style/vocabulary revisions, input checksum, and application version.

Measurements on the M1 Pro/16 GB development machine with Ollama 0.30.10 and the installed `qwen2.5-coder:7b` fixture:

- The installed model was 4,683,087,561 bytes with digest `dae161e27b0e90dd1856c8bb3209201fd6736d8eb66298e75ed87571486f4364`.
- A cold generation loaded the model in approximately 4.5 seconds and completed in approximately 16.7 seconds; a warm run loaded in approximately 0.14 seconds and completed in approximately 10.6 seconds.
- The synthetic request used 257 prompt tokens and produced 120 output tokens. Throttling reduced raw token/chunk traffic to at most roughly 10 progress events per second.
- Closing a generation request cancelled the client path in approximately 71 ms, and the separately owned Ollama server remained healthy.

Keep/change decision:

- **Keep** a narrow `ProtocolGenerator` port with explicit discovery, generation/cancellation, bounded progress, strict input/output validation, and complete provenance.
- **Keep** Ollama as the first development and early technical-preview adapter only. Require an exact installed model and never pull or select a cloud-backed model implicitly.
- **Keep** synchronous loopback HTTP on a bounded blocking worker for this adapter; it does not justify adopting a general async runtime. Restrict the endpoint to loopback and do not add TLS dependencies for this local-only path.
- **Keep** schema-constrained generation followed by application validation before an output can become a protocol draft revision. Schema validity is not factual correctness; human review remains mandatory.
- **Retain the spike as an installed-runtime contract test and reference. Rewrite the production provider adapter behind the application port; do not import the spike crate.**

Risks and required production changes:

- Ollama remains a separately installed/configured privacy and lifecycle boundary. LocaLog must not start, stop, reconfigure, download through, or assume ownership of a user-managed server without explicit consent.
- The public distribution runtime/model choice, licensing, acquisition UX, memory use on the M1/8 GB baseline, multilingual protocol quality, and model suitability remain unresolved.
- Cancellation closes LocaLog's request but does not terminate a user-owned Ollama server. A future bundled sidecar would instead use the accepted process-supervision contract.
- Required-section checks only prove structure. Representative, consented evaluation data and a review rubric are needed before accepting a protocol model or preset for professional use.

## Markdown editing and autosave spike result

The isolated TypeScript spike under `spikes/markdown-editor/` validated the writing-state boundary without importing spike code into the Svelte application or selecting a rich-text framework.

What was tested:

- Exact canonical Markdown round-tripping and a deterministic, deliberately conservative plain-text transformation.
- Debounced autosave to a mutable working record identified by document, immutable base revision, and monotonically increasing sequence.
- A single in-flight save with coalescing to the newest full document, so rapid edits do not create unbounded IPC traffic or queued duplicate buffers.
- Dirty/waiting/saving/failed states, explicit retry after disk-style failure, immediate flush, and rejection of stale or malformed acknowledgements.
- A generated Markdown document over 1 MB with no real project data.

Measurements on the development machine:

- Applying an edit to a 1,080,044-byte synthetic document took approximately 0.02 ms in the state model.
- Conservative plain-text export of that document took approximately 6.2 ms.
- Rapid edits produced one debounced save; edits made during a slow write produced one follow-up save containing only the latest complete value.

These are state-model measurements, not browser rendering, IPC, durable-disk, or M1/8 GB acceptance measurements.

Keep/change decision:

- **Keep** canonical Markdown and separate mutable working state. Immutable revisions are created only through the accepted storage commit boundary, never by each keystroke.
- **Keep** a 500 ms starting debounce, one in-flight full-document save, one coalesced latest value, sequence/base-revision guards, explicit failure state, and flush at deliberate navigation/commit boundaries.
- **Keep** exact Markdown export and the initial narrow plain-text transform in the application layer; do not create an export crate or add a Markdown dependency until supported syntax requires one.
- **Do not adopt a rich-text/editor framework from this spike.** Begin the vertical slice with the deliberate Markdown-backed writing workspace and add editing technology only when interaction testing demonstrates a concrete need that native controls cannot satisfy.
- **Retain the spike as a state-machine test oracle. Rewrite the production autosave coordinator against the real typed UI/storage boundary; do not import the spike module.**

Risks and required production changes:

- The browser must still be profiled with large documents while real model work runs; DOM updates, spellcheck, text selection, IPC serialization, and durable file replacement are outside this synthetic measurement.
- Navigating or closing with an in-flight/failed save requires explicit UX. A component destructor cannot silently promise that asynchronous persistence completed.
- The writing workspace needs keyboard, focus, text scaling, screen-reader naming, undo/redo, find, selection, recovery, and long-document visual acceptance checks during the vertical slice.
- Edited-after-review semantics are now accepted in D-025: autosave protects the changed working draft, the exact reviewed revision remains preserved, and the current presentation becomes `changed since review`. Implementation belongs to the later protocol stage.

## Remaining approval questions

1. What runtime/distribution model should the first public build use? Ollama is approved only for spikes and early technical previews.
2. What exact backup/restore scope is required for v0.1 hardening, and when should a portable project bundle become an acceptance criterion?
3. Which interface locales ship initially, and how is the independent first-run meeting/content language selected?
4. Will distribution use a direct notarized build, the Mac App Store, or both? This may affect sandbox and sidecar choices.

## Deferred decisions

- Automatic model acquisition and management.
- Portable project-bundle format beyond the v0.1 backup/restore consideration.
- Integrated microphone/system-audio recording and automatic diarisation.
- Public provider/plugin SDK and broad provider capability negotiation.

## GitHub publication

The public repository is `PascalNun/localog`. Private directional studies are excluded; the written design contract and the implemented application remain public. Representative screenshots may be added when the working interface is ready to stand on its own.

Issues, milestones, and a project board should follow real implementation needs rather than reproduce the speculative roadmap. The intended milestone outline is:

- Phase 0 — Architecture validation and UI shell
- v0.1 — Imported audio vertical slice
- v0.1 Hardening — Packaging, privacy, accessibility, recovery
- Phase 2 — Recording and traceability

Suggested labels: `area:ui`, `area:rust-core`, `area:storage`, `area:jobs`, `area:transcription`, `area:generation`, `area:export`, `platform:macos`, `platform:windows`, `platform:linux`, `type:decision`, `type:spike`, `type:bug`, `privacy`, `accessibility`, `blocked:approval`.

Create issues from the implementation sequence; do not create a large speculative backlog from the later roadmap.
