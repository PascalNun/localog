# Decisions

This is the record of choices that shape LocaLog. It is not a backlog and it is not a list of every implementation detail.

Status words:

- **Accepted** — the direction is chosen.
- **Proposed** — a direction is being tested.
- **Open** — a decision is still needed.
- **Deferred** — deliberately postponed.
- **Superseded** — retained for history, but no longer current.

## Accepted decisions

### Product and workflow

1. LocaLog is a desktop-first, local-first application. The core workflow requires no account, cloud service, telemetry, or hosted AI provider.
2. The product hierarchy is `Project → Meeting → recordings, transcripts, protocols, exports`. There is no separate inbox as the centre of the product.
3. Imported audio and video are the first input. Microphone and system-audio recording remain a later phase and the Record action stays out of the first working shell.
4. The protocol is the outcome. Transcription is a reviewable source, not the final product.
5. Generated protocols remain drafts. Revisions are retained and a review belongs to one exact immutable revision.
6. Interface language and meeting/content language are separate settings.
7. Protocol styles are named professional presets with structured instructions and output expectations. They are not arbitrary prompt fields for each meeting.
8. The interface is a core product requirement equal to privacy and data reliability. It should feel calm, immediate, clear, and professional.
9. Barlow is the locally bundled primary application typeface. The reference images are directional, not a list of literal controls.

### Technology and data

10. The frontend uses Svelte with TypeScript.
11. Tauri is the desktop shell.
12. Rust owns application logic, storage, background work, and local runtime integration.
13. The first implementation is a small modular monolith. No universal workflow engine, public provider SDK, export crate, or large crate graph is needed yet.
14. SQLite is authoritative for identity, relationships, lifecycle state, revision metadata, job state, and artifact path/checksum records.
15. An immutable versioned artifact file is authoritative for the content of that committed revision. The file becomes visible only after durable writing and the corresponding database transaction.
16. Working autosave state is separate from immutable revisions. Imported originals are never silently modified.
17. Structured JSON is the canonical committed transcript representation. Any database projection is derived and rebuildable, not a second editable copy.
18. Markdown is the canonical protocol source. Markdown and plain text are the first exports.
19. IDs are opaque UUIDv7 values. Stored timestamps use UTC Unix milliseconds; meeting dates remain explicit professional dates.
20. Heavy work is supervised and kept off the interface thread. Progress, logs, process output, and retries are bounded. One heavy local task runs at a time until measurements show that a scheduler is needed.

### Local runtimes and models

21. FFmpeg/FFprobe are the first media boundary.
22. whisper.cpp is the first transcription runtime direction and should eventually be shipped as a signed sidecar.
23. sherpa-onnx is the accepted v0.1 direction for optional automatic speaker separation. Its executable is built as a target-specific Tauri sidecar and discovered automatically; its verified model files are prepared on first use with explicit consent. Labels remain provisional and editable.
24. Ollama is accepted for development spikes and early technical previews through a narrow loopback-only provider boundary. It is not the final public distribution decision.
25. D-014’s original “discover installed models only” rule is superseded. Known transcription and diarisation models may be downloaded on demand with explicit user consent, HTTPS verification, checksums, atomic installation, and no silent acquisition. A model marketplace is not part of the product.
26. Reproducibility means recording provenance and resolved inputs, not promising byte-identical model output.

### Scope and distribution

27. App-managed working storage is approved for v0.1, with documented locations and explicit exports. Backup/restore remains part of v0.1 hardening and a portable project bundle remains a later possibility.
28. macOS 13+ on Apple Silicon is the provisional first platform. An M1 Mac with 8 GB RAM is the weakest representative test machine. Windows and Linux remain intended platforms.
29. LocaLog is licensed under GPL-3.0-or-later. Third-party runtimes, models, fonts, binaries, and assets retain their own licences and need separate distribution review.
30. The repository is public as `PascalNun/localog`. Issues and project management should follow real implementation needs rather than reproduce a speculative backlog.
31. Protocol model choice is a global preference in Settings. The normal workflow reuses that choice; optional language-specific profiles and per-meeting overrides remain advanced possibilities. Every job records the exact resolved model and settings used.

## Open questions

These are the questions that still affect product behaviour, distribution, or long-term architecture.

1. What self-contained local runtime, if any, should replace or accompany Ollama for public protocol generation?
2. What is the minimum backup/restore experience for v0.1 hardening, and when should a portable project bundle become an acceptance criterion?
3. Which interface locales ship first? Meeting language is now chosen independently through project defaults and per-meeting overrides; the first interface locale is still English.
4. Should speaker separation be on by default when its runtime is available, or remain an explicit optional step?
5. Should language detection be offered as an advisory preflight, and what confidence threshold would make it useful without overriding the selected language?
6. Will macOS distribution use a direct notarised build, the Mac App Store, or both? The answer affects sandboxing and sidecars.
7. Which transcript content should remain canonical if a future structured transcript view and Markdown editing surface both exist? The current answer is one structured JSON transcript artifact, with derived views rather than two editable sources.

## Deferred decisions

- integrated recording;
- collaboration, sharing, accounts, cloud sync, calendars, live bots, and mobile applications;
- semantic search across projects;
- DOCX/PDF export and a rich template designer;
- a public provider/plugin SDK and broad capability negotiation;
- a portable project-bundle format beyond basic backup/restore;
- organisation-controlled remote processing.

## How decisions are made

A decision belongs here when it changes the product promise, data safety, user workflow, distribution model, or a boundary that future work will depend on. A spike can provide evidence, but evidence is not automatically a decision.

When a decision changes, update the accepted summary and the affected product or architecture document in the same change. Keep the old reasoning only when it helps explain why the decision changed.

## Evidence that shaped the current direction

- The storage study established the file-before-database visibility rule, separate working state, checksum reconciliation, and recovery behaviour.
- The process study established argument-array invocation, bounded output, process-group cancellation, and a single heavy-work lane.
- The media study established the normalised mono/16 kHz boundary, provenance requirements, and why Python/PyTorch Whisper is not the public distribution baseline.
- The local provider study established the narrow loopback Ollama boundary, exact installed-model checks, cancellation, bounded responses, and provenance.
- The Markdown study established separate autosave and immutable revisions, sequenced saves, and deterministic plain-text export.
- The speaker study made sherpa-onnx viable as a candidate, while also showing that synthetic accuracy is not enough to accept real meeting quality.
- The model evaluation showed that vocabulary helps proper nouns, long context is expensive, and protocol quality—not model size alone—is the critical question.

The detailed measurements remain in the research and spike documents. They should be read as evidence for the decisions above, not as additional requirements.
