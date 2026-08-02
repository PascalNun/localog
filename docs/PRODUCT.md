# Product definition

This document describes why LocaLog exists and what kind of product it should become. It starts from professional meeting work rather than from AI capabilities.

In LocaLog, a **protocol** is the structured written record or minutes of a meeting. It is not a networking term.

## Problem

Professional meetings produce decisions, responsibilities, unresolved questions, and technical detail that must become a reliable written record. Their recordings can also contain confidential conversations, personal information, internal decisions, client details, or other material that should not leave the organisation’s controlled environment. Manual protocols are slow; generic cloud meeting assistants are often unsuitable for this sensitive project content and tend to optimise for summaries rather than controlled documentation.

## Product promise

LocaLog is an open-source, local-first AI desktop application for creating structured, editable meeting protocols from audio and video on the user’s device. The protocol is the intended result. Local transcription and transcript review provide controlled source material for a local language model to create a draft, which the user can then refine and export.

This path combines project context, deliberate human review, and a carefully designed writing workflow; no LocaLog account or cloud service is required for the core experience. Transcription is an essential capability, but it serves the protocol rather than becoming a second, equally weighted product destination.

The privacy focus follows from this architecture: the normal workflow should not require sensitive meeting content to be uploaded to a third-party transcription or language-model service. Local-first describes where the work happens and who remains in control of the data, not merely a deployment preference.

Its differentiator is not local inference alone. LocaLog connects the full documentation sequence—from source preservation and transcription to correction, controlled generation, revision, and export—without separating privacy, professional structure, and interface quality into secondary concerns.

Interface quality is a core product requirement and principal differentiator, equal in importance to local-first architecture and reliable data handling. The product succeeds only when local processing becomes a calm, immediate, intuitive, and trustworthy professional desktop workflow; working transcription and generation engines alone are insufficient.

## User-centred principle

LocaLog is evaluated from the user’s point of view: can someone move from a meeting recording to a useful, trustworthy protocol clearly, quickly, and with control? Models, architecture, and feature breadth are means to that outcome, not measures of success by themselves.

Product and implementation choices should reduce unnecessary effort, waiting, uncertainty, and cognitive load throughout that task. When the technically easiest option would materially weaken the workflow, the trade-off must be raised and reviewed. User focus does not override privacy, data integrity, or human review; those safeguards are part of what makes the product useful and trustworthy.

## Initial users

The first proving context is architecture and planning work, including architects, engineers, project managers, consultants, construction teams, and public-sector project teams. The product model should remain useful to other professional project teams; architecture-specific language belongs in presets and vocabulary, not hard-coded application logic.

The application is intended for macOS, Windows, and Linux. macOS is the first development and validation environment, not the final limit of the product.

## Product model

```text
Project
└── Meeting
    ├── Recording(s)
    ├── Transcript and revisions
    ├── Protocol draft and revisions
    └── Export(s)
```

Every recording belongs to a meeting and every meeting belongs to a project. A meeting may eventually contain separate microphone and system-audio sources, so `Recording` is one-to-many even though v0.1 imports one source.

Microphone and system-audio recording remain specified for Phase 2, but the Record action is omitted from the first functional MVP shell until recording actually works.

## Core capabilities

- Create projects and meetings with inherited defaults.
- Import audio or video, preserve the source, and normalise working audio locally.
- Transcribe locally with human-friendly quality presets.
- Review transcript text, timestamps, speaker labels, and unclear terms.
- Apply global and project vocabulary containing names, acronyms, organisations, places, and technical terms.
- Generate an editable protocol with a user-selectable local model and reusable protocol style.
- Save the canonical protocol as Markdown plus structured metadata.
- Export Markdown and plain text in v0.1; add DOCX next.

## Defaults and overrides

Configuration resolves in this order:

`global default < project default < meeting override`

The resolved values should be snapshotted when a processing job starts so a later settings change cannot silently alter a running result or its provenance. Relevant settings include meeting/content language, transcription preset, vocabulary sets, participants, protocol style, writing provider/model, and export template. The goal is repeatability of inputs, not a promise of byte-identical model output.

Interface language and meeting/content language are independent. The language selected for transcription and protocol generation never implicitly changes the application interface language.

## Vocabulary

Vocabulary is a library object, not a free-form prompt box. Entries should support term, preferred spelling, category, aliases, optional note, scope, and enabled state. v0.1 uses vocabulary as transcription context where supported and as protocol-generation context; it does not claim to fine-tune models.

## Protocol styles

A style is a named professional preset containing structured instructions and output expectations. Styles are selected as professional outcomes, not presented as arbitrary per-meeting prompt engineering. An advanced style editor may eventually expose the underlying instructions in a controlled form. Initial examples:

- Internal working note
- Formal minutes
- Task list
- Client summary
- Technical decision log
- German and English variants

Styles are content, not model settings. Temperature, context size, quantisation, and chunking remain advanced implementation choices.

## Experience quality

- The first shell establishes the real navigation, spacing, typography, hierarchy, light/dark tokens, and interaction behaviour; it is not a disposable wrapper around backend work.
- Routine navigation, selection, typing, and editing should feel immediate while local processing runs.
- Progressive disclosure and sensible defaults keep technical settings out of the normal workflow.
- Transcript review and protocol editing are serious professional workspaces, not placeholder forms in the completed vertical slice.
- Accessibility, keyboard use, focus, text scaling, empty/progress/error states, and recovery communication are part of product quality.
- Material implementation trade-offs that weaken calmness, clarity, or professional quality require review rather than silently choosing the technically easiest UI.

## Trust and privacy

- The generated protocol is visibly labelled a draft until the user marks it reviewed.
- The source recording and transcript are never deleted as an implicit side effect of generation or export.
- No telemetry, analytics SDK, remote crash reporting, cloud sync, or automatic upload.
- Network-capable local runtimes must be documented and explicitly configured; LocaLog binds or connects to loopback only where it controls the endpoint.
- Logs must exclude transcript/protocol bodies and redact user paths by default.

## Non-goals for v0.1

- Generic chatbot or permanent prompt field
- Cloud accounts, sync, team collaboration, or sharing
- Calendar integrations or meeting bots
- Mobile clients
- Live transcription
- Automatic speaker diarisation
- System-audio or microphone recording
- Model marketplace or training/fine-tuning
- Final-authority or compliance claims
- Rich project management, task tracking, or search across an organisation

## Success signal

With a synthetic or consented real-world recording, a user can complete the core workflow without the app becoming unresponsive, without data leaving the machine through LocaLog, and with enough review control to produce a useful Markdown protocol.
