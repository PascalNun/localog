# Technical architecture

This document explains how LocaLog can keep professional meeting data safe while local transcription and language models perform demanding work in the background. The purpose of the architecture is not technical novelty. It is to keep the interface responsive, make failures recoverable, and prevent one operating system or model provider from defining the product.

## In plain language

- The Svelte interface presents projects, meetings, transcripts, and protocols.
- A Rust application core applies the product rules and coordinates storage and background work.
- SQLite remembers identities, relationships, stable meeting stages, revision records, and job history.
- Versioned files hold the committed content of recordings, transcripts, and protocols.
- Specialised local tools run outside the interface and report bounded progress through narrow adapters.
- macOS, Windows, and Linux share the same product core; platform-specific process, filesystem, permission, recording, and packaging behaviour stays at the edges.

The [documentation guide](README.md) explains recurring terms such as adapter, artifact, canonical content, provenance, and spike.

## Goals

- Keep the UI responsive while CPU/GPU-heavy local processes run.
- Make jobs cancellable, observable, and recoverable after a restart.
- Preserve user data across model/runtime failures and app upgrades.
- Avoid coupling product concepts to `whisper.cpp`, Ollama, or any one model.
- Validate and package macOS first while preserving Windows and Linux as intended platforms and keeping operating-system-specific code out of the domain core.
- Make local-only behaviour auditable and testable.

## Working shape

Use a small modular monolith inside Tauri. Tauri, Rust, Svelte, TypeScript, app-managed working storage, and the main runtime boundaries are accepted choices. Individual production modules still earn their final shape through the vertical slice; completed spikes are evidence, not code to import unchanged.

```text
TypeScript UI
    │ typed commands + event stream
Tauri boundary
    │
Rust application services ── durable job manager
    │                              │
Domain model                 supervised processes
    │                         ffmpeg / whisper / LLM
Repositories + file store          │
    │                         runtime adapters
SQLite + app-managed files
```

The UI never constructs shell commands, owns runtime processes, or writes canonical artifacts directly. Adapters translate stable application requests into runtime-specific invocations and normalise progress/errors/results.

## Initial technology choices

| Area             | Choice / hypothesis                                               | Status                          | Gate                                                                                                           |
| ---------------- | ----------------------------------------------------------------- | ------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Shell            | Tauri 2                                                           | Accepted                        | Reconsider only if a spike exposes a blocking limitation                                                       |
| UI               | TypeScript + Svelte                                               | Accepted                        | Validate accessibility, responsiveness, and packaging in the shell                                             |
| Core             | Rust application and domain logic                                 | Accepted                        | Keep product logic independent of Tauri and runtime implementations                                            |
| Rust concurrency | Standard process/thread/channel building blocks initially         | Accepted for the vertical slice | Add Tokio only when a real adapter demonstrates an async-I/O need; keep domain logic runtime-agnostic          |
| Persistence      | `rusqlite`, SQLite metadata/jobs, immutable artifact files        | Accepted for the vertical slice | Production module must preserve staged durability, blocking-worker isolation, migrations, and recovery         |
| Media            | Supervised FFprobe/FFmpeg normalization adapter                   | Accepted for the vertical slice | Distribution still requires a minimal licensed, checksummed, signed, size-measured binary strategy             |
| Transcription    | `whisper.cpp` CLI/sidecar adapter                                 | Proposed                        | Measure offline transcription, timestamps, Metal behaviour, cancellation, and baseline-machine performance     |
| LLM              | Narrow provider port; Ollama adapter first                        | Accepted for Phase 0 validation | Ollama is not accepted as the final public distribution model                                                  |
| Protocol editor  | Canonical Markdown plus sequenced single-flight autosave boundary | Accepted for the vertical slice | Preserve the deliberate writing workspace; select richer editing technology only for a proven interaction need |
| IDs/time         | One ordinary opaque sortable ID implementation and UTC timestamps | Proposed implementation detail  | Select during scaffolding; do not spike competing ID libraries                                                 |

Do not pin versions in this specification. Pin exact toolchains and dependencies when the scaffold is generated, and record upgrades deliberately.

## Minimal repository tree

```text
localog/
├── README.md
├── CONTRIBUTING.md
├── docs/
│   └── ...
├── src/                       # Svelte/TypeScript presentation layer
├── src-tauri/                 # Rust commands, application modules, adapters, composition root
│   └── src/                   # split into modules only as validated boundaries emerge
├── tests/                     # boundary and end-to-end tests
├── fixtures/                  # synthetic, redistributable data only
├── spikes/                    # isolated validation crates; never imported as production modules
└── .github/                   # minimal CI-ready structure when useful; no project backlog yet
```

Do not start with five Rust crates or a dedicated export crate. Preserve the logical dependency direction inside the initial crate: presentation and adapters depend inward; domain rules never depend on Tauri, storage, or a model runtime. Split crates only when a tested boundary has become real.

## Data model

The data model is the application’s map of a project. It records what belongs together, which revision is current, and what stage a meeting has reached. Large document and media content remains in files rather than being hidden inside one opaque database.

Key records:

- `Project`: id, name, optional client/number, defaults, timestamps, archived_at
- `Meeting`: id, project_id, title, occurred_at, stable lifecycle state, overrides, review state
- `Recording`: id, meeting_id, kind, original name, managed relative path, media metadata, checksum
- `Transcript`: id, meeting_id, source recording ids, revision, language, model snapshot, status
- `TranscriptSegment`: transcript_id, ordinal, start/end ms, speaker id, text, confidence/flags where available
- `Protocol`: id, meeting_id, revision, Markdown artifact path/checksum, style snapshot, model snapshot, status
- `Export`: id, protocol_id, format, relative path, created_at
- `VocabularySet` and `VocabularyEntry`: scope and mergeable structured terms
- `ProtocolStyle`: named/versioned instructions and output contract
- `Job`: type, entity id, transient job state, stage, progress, request/provenance snapshot, timestamps, error class, attempts

Protocols and transcripts need revisions; overwriting their only copy makes review and regeneration unsafe. v0.1 may expose only the latest revision while retaining prior revisions internally.

The storage spike selected a versioned structured JSON artifact as the canonical content for each committed transcript revision. SQLite stores the revision identity, relationship, lifecycle, path, checksum, byte count, and provenance. Any future segment/search projection in SQLite is derived and rebuildable; it is never a second editable canonical copy. Mutable transcript autosave is a separate working artifact and cannot overwrite an immutable committed revision.

Stable meeting lifecycle and transient work are separate axes. Initial stable lifecycle values are `draft`, `source_ready`, `transcript_ready`, `protocol_draft`, `reviewed`, and `archived`. Job state is stored on `Job`, not encoded into `Meeting.lifecycle_state`. The UI may derive presentation states such as “Transcribing” from both axes.

## Storage layout

LocaLog uses two complementary forms of storage. SQLite holds the map: identities, relationships, stages, jobs, revision records, paths, and checksums. Versioned files hold the committed content itself. This keeps professional documents inspectable while still allowing the application to protect consistency and recover from interruption.

Default to an OS-provided application-data directory, never the current directory or an assumed home path.

```text
<app-data>/
├── localog.sqlite3
├── projects/<project-id>/meetings/<meeting-id>/
│   ├── recordings/<source-artifact-id>.<validated-extension>
│   ├── working/imports/<source-artifact-id>.part
│   ├── working/recovery/<source-artifact-id>.orphan
│   ├── working/jobs/<job-id>.<json|md>.part
│   ├── transcripts/
│   │   ├── revisions/<transcript-revision-id>.json
│   │   └── working.json
│   ├── protocols/
│   │   ├── revisions/<protocol-revision-id>.md
│   │   └── working.md
│   └── exports/
├── models/        # only app-managed/downloaded models, if later supported
├── logs/
└── tmp/
```

Database paths are relative and validated beneath the managed root. Copy imports rather than depending on removable/source paths. Use write-to-temp + flush + atomic rename for canonical text artifacts. Never follow user-controlled symlinks into managed storage. Check disk space before copying/normalising.

The storage spike accepted this invariant for the vertical slice:

- SQLite is authoritative for identity, relationships, stable lifecycle state, revision metadata, job state, and artifact path/checksum records.
- An immutable versioned artifact file is authoritative for the content of that specific committed revision.
- A revision becomes visible only after its file has been durably written and the corresponding database transaction has completed.
- Autosave working state is separate from immutable committed revisions.
- Imported original media is never modified silently.

Updates that touch SQLite and files use an explicit staged protocol and reconciliation scan because the two cannot share one atomic transaction. Committed transcript revisions use versioned structured JSON artifacts; SQLite segment/search projections, if later justified, are derived and rebuildable. Do not maintain a database copy and a file copy as separately editable authorities.

### Transcript and protocol revision protocol

The durable fake workflow establishes the same boundaries later inference adapters must use:

1. A transcription or generation intent is written to SQLite as `queued`, including its immutable input identity and resolved synthetic provenance.
2. The stage-specific adapter reads only a committed source or transcript revision. It writes validated output to `working/jobs/` while reporting bounded progress and observing cancellation.
3. The staged artifact is flushed and synced, hashed, and renamed to its generated immutable revision path. Only then does one SQLite transaction insert the revision metadata, advance stable lifecycle, and complete the job.
4. Transcript JSON is validated for schema version, ordered stable segment identifiers, non-negative start/end timestamps, and bounded text fields before commit. Protocol output must be non-empty UTF-8 Markdown.
5. A committed revision is copied atomically to the stage's replaceable working artifact. Autosave uses a sibling temporary file, flush, sync, and rename so interruption retains the previous complete working state.
6. Starting protocol generation commits dirty transcript working state first. The generation job records that exact committed transcript revision and never reads mutable working content.
7. Marking a protocol reviewed creates or selects the exact committed revision being reviewed and stores its identity. Later working edits do not alter it; the presentation state becomes `changed since review`.

On startup, abandoned `running` or `cancelling` processing jobs become `interrupted`. Staged output is never shown as ready. A final artifact without matching visible revision metadata is quarantined unless the persisted job contains sufficient checksum/path evidence to finish the exact interrupted commit safely. Working artifacts are loaded only when their SQLite checksum metadata agrees; a failed autosave keeps the last verified working document visible and reports the failure.

### Durable source-import protocol

The first production import implementation uses one explicit sequence:

1. One SQLite transaction creates the meeting, pending source-artifact record, and queued import job after the user confirms project, meeting, and source.
2. A blocking worker opens the external source read-only and copies it in bounded chunks to `working/imports/`, hashing the bytes in the same pass and persisting throttled byte progress.
3. The temporary file is flushed and synced. Its checksum, byte count, provisional media type, and intended final relative path are recorded on the job.
4. A checksum already attached to another committed source pauses the job at `duplicate_confirmation`. The complete temporary copy is retained until the user explicitly imports another copy or cancels.
5. The temporary file is renamed to the source’s final managed path and the recordings directory is synced where the platform supports directory syncing.
6. One SQLite transaction marks the source committed, changes the meeting to `source_ready`, completes the job, and clears the retained external source path.

SQLite remains authoritative for whether a source is visible as committed. A final managed file alone never advances lifecycle. The managed file is authoritative for the committed source bytes once its metadata transaction exists. The external source path is retained locally only while an import may need retry; it is never included in ordinary logs or user-facing diagnostics and is cleared after successful commit.

Startup reconciliation applies only rules supported by persisted evidence:

- abandoned `running` or `cancelling` imports become `interrupted`;
- queued intent that never began remains available to continue;
- incomplete temporary copies are removed, except a validated copy awaiting duplicate confirmation;
- a final file with persisted size/checksum metadata is verified and its interrupted database commit is completed;
- a final file that cannot be verified is moved under `working/recovery/` and the job remains non-complete;
- a legacy pending meeting without a retained source path remains visible and asks the user to choose its source again;
- a committed database source is never downgraded merely because an earlier job state was observed.

The first implementation classifies supported media from a validated filename extension. FFprobe becomes the content-based media authority when normalisation is integrated; the UI must not imply that extension classification is a full media probe.

Startup reconciliation checks paths, incomplete writes, reference consistency, and interrupted jobs without hashing every large recording. Checksum verification happens on artifact access and in an explicit/on-demand full-integrity pass.

App-managed working storage must not become opaque. Settings and documentation expose its location; exports go to explicit user-selected destinations. Preserve future boundaries for backup/restore and a portable project bundle. A basic backup/restore mechanism is considered during v0.1 hardening.

## Jobs and responsiveness

A job is a piece of background work such as importing media, transcribing audio, or generating a protocol. Jobs are durable records rather than animations owned by one screen, so closing a view does not erase what is happening and a restart can explain interrupted work honestly.

The Rust job manager owns a bounded queue. Model-heavy work is never run on the UI thread or inside a long-lived Tauri command response.

Job states are independent of meeting lifecycle. The persisted vocabulary is `queued`, `running`, `cancelling`, `failed`, `cancelled`, `interrupted`, and `completed`. Import stage detail remains explicit—such as copying, validating, duplicate confirmation, and finalising—without becoming a general workflow engine.

- Persist intent and state before starting a process.
- Emit throttled progress events; the UI can re-query authoritative state at any time.
- Capture PID/process-group identity where safe, bounded stdout/stderr, and runtime/version snapshots.
- Cancel cooperatively first, then terminate the process tree after a grace period.
- On launch, mark abandoned running jobs `interrupted`, inspect artifacts, and offer safe retry/resume.
- Concurrency defaults to one model-heavy job; imports/exports may use a separate small lane.
- Resumability means restarting from a durable stage in v0.1, not checkpointing arbitrary inference internals.

Do not build a universal workflow engine. Begin with the smallest shared job envelope that supports import, transcription, and generation, while keeping each pipeline's stages explicit.

## Efficiency and responsiveness

- Media processing, inference, migrations, and large file operations never run on the UI thread or in a long-lived command response.
- Convert process output into bounded, throttled or batched progress events. The UI re-queries authoritative state rather than reconstructing it from a high-volume event stream.
- Ordinary navigation, selection, typing, and editing should generally respond within approximately 100 ms even during background model work.
- Keep startup lightweight and lazy-load secondary views or data where this provides a measured benefit.
- Stream or incrementally process large files where practical; avoid retaining full media or very large transcripts in duplicate memory buffers.
- Keep process output and application logs bounded. Ordinary logs never contain audio, transcript, or protocol bodies.
- Profile before low-level optimisation. Architectural isolation of heavy work is the first performance tool.

## Runtime abstractions

LocaLog needs specialised local tools, but the product should not be shaped around one of them. A narrow port describes what the application needs; an adapter translates that request for a particular tool. This makes it possible to change runtimes—or implement different platform mechanics—without rewriting the meeting workflow.

Stable ports should describe only proven product capabilities:

- `MediaNormalizer`: probe and produce supported PCM working audio
- `Transcriber`: model discovery, transcribe, cancel, bounded progress
- `ProtocolGenerator`: model discovery, generate/revise, cancel, bounded progress
- `Exporter`: supported formats and deterministic export

For Phase 0, discovery needs only availability, installed or user-provided models, runtime version, and actionable diagnostics. Do not design broad capability negotiation or a public plugin SDK until at least two real implementations demonstrate a common need. A configured model path is never assumed valid. Provider output is validated before it becomes a revision.

Protocol styles are named professional presets containing structured instructions and output expectations. They may contribute controlled instructions to an internal LLM request, but the primary meeting workflow does not expose arbitrary prompt engineering. A future advanced style editor may expose these instructions in a controlled form.

Use argument arrays, never shell-interpolated commands. Environment variables are allowlisted; working directories are controlled. Loopback HTTP providers get timeouts and response-size limits. External runtime auto-start is opt-in unless LocaLog ships and supervises that runtime.

Phase 0 only discovers already installed or user-provided models. It does not download or manage models. Ollama may be required for development spikes and early technical-preview builds, but that does not settle the first public build's runtime or distribution model.

## Portability boundaries

Keep these behind traits/modules:

- app-data and user document pickers
- file reveal/open operations
- process groups and termination
- sleep prevention for long jobs
- microphone/system-audio capture and permissions
- hardware acceleration/model compatibility
- code signing, quarantine, and sidecar packaging

System audio on macOS may require Screen Recording permission and different APIs than Windows/Linux. Recording remains Phase 2, but the one-to-many recording model prevents a schema rewrite.

## Privacy and security

- No telemetry or network dependency in the core path.
- Content-free structured logs by default; optional diagnostic bundle requires preview/consent.
- Strict Tauri capability allowlist and narrow command surface.
- Validate media size/type by probing, not extension alone.
- Protect against path traversal, symlinks, hostile filenames, oversized output, and process hangs.
- Store no secrets unless a future provider requires them; then use OS credential storage.
- Document that Ollama or another separately installed provider has its own configuration and privacy boundary.

## Failure recovery

Imported originals are immutable. Derived artifacts carry provenance and completion markers. Partial files stay in `working/` and are never mistaken for finished output. Migration startup creates a backup/recovery point. Destructive cleanup is explicit and refuses paths outside the managed root.

Provenance supports repeatability of inputs, not a promise of byte-identical inference output. Where available, a completed job records provider/runtime version, immutable model digest or checksum, resolved transcription/generation settings, style revision, vocabulary revision, normalized input checksums, and application version.

## Architecture validation spikes

Run focused spikes after the minimal scaffold and fake shell, in this order:

1. SQLite/filesystem staged-write and crash-reconciliation test, including the transcript-authority decision.
2. Tauri process supervision, bounded progress events, and cancellation with a fake long-running sidecar.
3. `ffmpeg` + `whisper.cpp` import/transcription on short synthetic German and English fixtures; capture timestamps, resource use, and cancellation behaviour on the baseline machine.
4. Ollama discovery/generation spike plus a fake provider used in automated tests.
5. Markdown editing/autosave spike with a long synthetic document and keyboard/accessibility review.

Each spike records what was tested, relevant measurements, risks, packaging/licence implications where applicable, a keep/change decision, and whether its code should be retained, rewritten, or discarded. Record the result in `docs/DECISIONS.md` or a focused ADR; spike code never becomes production architecture by accident.
