# Roadmap after v0.1

The sequence is outcome-based, not a release promise.

LocaLog is intended for macOS, Windows, and Linux. macOS receives the first complete build because it is the current development and validation environment. Cross-platform architecture is a present requirement; packaged Windows and Linux releases follow once the central workflow and their platform adapters have been validated.

## Phase 2 — recording and richer review

- Microphone recording with interruption/recovery handling
- macOS system-audio capture, then platform equivalents
- Multiple synchronized recording sources per meeting
- Automatic speaker diarisation and better manual speaker tools
- Audio waveform and faster timestamp navigation
- Protocol-to-transcript/audio traceability
- DOCX export and basic templates

## Phase 3 — portability and libraries

- Windows and Linux packaged builds
- Provider/model management that remains optional and local
- Export template library and PDF conversion
- Search across local projects and meetings
- Polished project archive/import bundles and portability beyond the basic backup/restore considered during v0.1 hardening
- Richer vocabulary import, suggestions, and conflict resolution
- Structured task/decision views derived from protocols, always reviewable

## Phase 4 — advanced local workflows

- Optional live transcription where hardware permits
- User-approved local automation and batch processing
- Pluggable providers/runtimes with a stable extension contract
- More precise source citations and confidence/review tooling
- Optional organisation policies for managed deployments

## Explicitly uncommitted

Cloud sync, accounts, collaboration, shared workspaces, calendar integration, meeting bots, mobile apps, and hosted inference are not implied by this roadmap. Each would require a separate product/privacy decision and must not become an incidental dependency.
