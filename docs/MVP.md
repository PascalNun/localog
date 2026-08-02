# v0.1 MVP

This document defines the smallest complete version that can prove LocaLog’s central promise. “MVP” does not mean a disposable or visually unfinished application. It means a deliberately bounded workflow whose interface, privacy, and data handling are already trustworthy enough to evaluate honestly.

## Objective

Prove a private, resilient imported-audio workflow on macOS:

`project/meeting storage → local transcription → transcript review → local protocol generation → Markdown editing/export`

v0.1 is not complete merely because each engine can run; the workflow, recovery, and user control are part of the proof.

Interface quality is part of the proof, not secondary polish. The shell and vertical slice must make local processing feel calm, immediate, intuitive, and trustworthy while preserving professional review control.

macOS is the first release and performance baseline because it is the current development environment. The shared application is intended for Windows and Linux as well; platform-specific differences remain adapters, and later packaging must not require a redesign of the core workflow.

## Included

- macOS desktop build for a documented minimum OS/hardware target
- Create, rename, archive, and list projects and meetings
- Import one audio/video source into a meeting
- Media probing and local audio normalisation
- Local `whisper.cpp` transcription with Fast/Balanced/Accurate presets
- Transcript segments with timestamps, editable text, and manual speaker labels
- Global and project vocabulary; resolved into a meeting job snapshot
- Reusable/editable protocol styles
- One validated local protocol-provider adapter plus a fake adapter for tests
- Editable Markdown protocol with autosave and revision retention
- Markdown and plain-text export with collision-safe filenames
- Durable progress, cancellation, retry, interrupted-job recovery, and missing-runtime states
- Settings for storage, transcription, models, appearance, and advanced diagnostics
- Light/dark UI tokens and the primary shell/workflow states
- A coherent design system using Barlow as the locally bundled primary application typeface and covering typography, spacing/layout, sidebar/workspace behaviour, common interaction states, contextual inspectors, and key-screen visual acceptance criteria
- No telemetry, account, or cloud service

## Excluded

- Built-in microphone or system-audio recording
- Automatic diarisation
- DOCX/PDF export and template designer
- Traceability from protocol statements to transcript/audio
- Cross-device sync, collaboration, calendar, bots, mobile
- Automatic model downloads/manager unless explicitly approved after the spike
- In-app chat or general-purpose prompting

## Implementation sequence

Phase 0A, the minimal Phase 0B scaffold, and the Phase 0C fake workflow shell are complete. All five Phase 0D architecture spikes have recorded keep/change decisions. The next implementation milestone is Phase 1A: the first durable project/meeting vertical slice using fake runtimes.

### Phase 0A — documentation decisions

1. Record approved decisions, remaining questions, storage/lifecycle corrections, efficiency constraints, and the role of the visual direction.
2. Present the resulting baseline and proposed minimal scaffold for review.

### Phase 0B — minimal repository scaffold

1. Initialise the local Tauri/Svelte/TypeScript/Rust repository without unnecessary crates or packages.
2. Add formatting, linting, basic tests, `.gitignore`, contribution/privacy rules, and synthetic fixture boundaries.
3. Record the provisional macOS 13+/Apple Silicon baseline and M1/8 GB test hardware.

### Phase 0C — visual shell and fake workflow

1. Derive and document light/dark tokens, typography, spacing/grid, navigation/workspace behaviour, control states, inspector rules, and visual acceptance criteria.
2. Build the persistent sidebar, start state, project overview, new-meeting import flow, transcript-review workspace, protocol-editor workspace, and settings view.
3. Exercise importing, transcription progress, cancellation, failure, retry, transcript-ready, and protocol-draft-ready states through the intended typed boundary using fake jobs and synthetic data.
4. Omit Record, avatars/accounts, sharing, permanent AI chat, and generic dashboard patterns.

The shell uses fake content but is not disposable: its navigation, design tokens, hierarchy, interaction behaviour, accessibility foundation, and responsive layout should be suitable for the vertical slice. Transcript review and protocol editing may be behaviourally bounded here, but they must become purpose-built workspaces rather than placeholder forms during the vertical slice.

### Phase 0D — bounded validation

1. Storage and recovery spike.
2. Tauri process supervision, progress-event, and cancellation spike.
3. Media normalisation/transcription sidecar spike.
4. Local protocol-provider/discovery spike.
5. Markdown editing/autosave spike.

Each spike records what was tested, measurements, risks, a keep/change decision, a documentation update, and whether its code is retained, rewritten, or discarded. No spike becomes production architecture by accident.

### Phase 1A — vertical slice with fakes

1. Project/meeting lifecycle and repository layer.
2. Full UI workflow using synthetic fixtures and fake runtimes.
3. Durable jobs, cancellation, restart recovery, and error states.

Implement only the recovery needed to protect this slice; do not delay it with a general persistence or workflow framework.

Implementation note, 2026-08-02: step 1 has begun with the production SQLite schema, project/meeting/pending-source transaction, restart-safe listing and title updates, and a narrow Tauri workspace store. Fake processing remains session-only and does not advance durable lifecycle until the corresponding source or revision commit exists.

### Phase 1B — real local pipeline

1. Import/normalise adapter.
2. `whisper.cpp` adapter and transcript persistence/review.
3. Vocabulary resolution.
4. Protocol style and selected provider integration.
5. Protocol editor, revisions, Markdown/TXT export.

### Phase 1C — hardening

1. Packaging, permissions, signing/notarisation plan.
2. Accessibility and keyboard pass.
3. Crash, disk-full, missing binary/model, cancellation, and long-recording tests.
4. Privacy/log audit and release checklist.

## Acceptance criteria

### Core workflow

- A user can create a project and meeting, import a supported synthetic file, and complete the workflow without terminal use.
- The source is copied into managed storage and remains unchanged.
- Restarting the app preserves projects, settings, transcript edits, protocol edits, and job history.
- The transcript exposes timestamps and supports text/speaker corrections before generation.
- Generation records the exact style, vocabulary, provider, model identifier, and relevant settings used.
- Exported Markdown round-trips the protocol text; plain text removes Markdown predictably.

### Responsiveness and jobs

- Navigation and editing remain responsive during processing on the documented baseline machine.
- Ordinary navigation, selection, typing, and editing interactions generally complete within approximately 100 ms during background work.
- Progress never blocks the UI; job state can be reconstructed from storage after relaunch.
- Cancellation stops the supervised process within a defined, measured timeout and retains the latest stable artifacts.
- A killed app does not present partial output as complete on restart.

### Privacy and safety

- Automated tests fail if the core workflow initiates non-loopback network access.
- Default logs contain no transcript/protocol body and no unredacted managed document path.
- Imported hostile filenames cannot escape the managed root or alter command arguments.
- Unsupported/corrupt media, disk-full, missing model/runtime, provider timeout, and invalid output have actionable UI states.

### Accessibility and visual quality

- Full core workflow is keyboard reachable with visible focus.
- Light and dark themes meet contrast targets for essential text and controls.
- Text scaling and reduced-motion settings do not make the workflow unusable.
- The shell follows `docs/VISUAL_DIRECTION.md` and is reviewed against its documented visual acceptance criteria.
- The shell establishes reviewed light/dark tokens, typography, spacing/grid, sidebar/workspace behaviour, common interaction states, inspector rules, and screen-specific visual criteria before detailed UI implementation.
- Transcript review and protocol editing support their professional tasks without falling back to generic forms, dashboards, permanent chat, or model-manager patterns.

## Test strategy

- **Domain unit tests:** hierarchy invariants, state transitions, settings resolution, vocabulary merging, naming and validation.
- **Storage tests:** migrations, repository round trips, atomic file writes, reconciliation, path containment, revision retention.
- **Adapter contract tests:** fake and real-provider response normalisation, cancellation, timeouts, malformed output.
- **Integration tests:** synthetic media through import/normalisation/transcription on supported CI or a scheduled macOS lane; database plus filesystem recovery.
- **UI component tests:** meeting states, errors, autosave status, keyboard/focus behaviour.
- **End-to-end tests:** happy path with fake engines; cancel/retry; app restart mid-job; missing runtime/model; edit/export persistence.
- **Manual release checks:** representative German/English synthetic recordings, long-duration performance, signing/permissions, dark mode, VoiceOver.

No real client recording, transcript, or protocol may enter fixtures, CI, screenshots, or issue reports.
