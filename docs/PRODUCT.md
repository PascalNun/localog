# Product

## The idea

LocaLog helps a person turn a meeting recording into a useful written protocol without sending the conversation to a hosted AI service.

The finished protocol is the purpose of the product. Transcription matters because it gives the person something they can review before the model turns the meeting into a document. The result is not meant to be a mysterious answer from an AI system. It is a draft with a visible source, a history, and a human decision at the end.

## Who it is for

LocaLog is for professionals who regularly need to turn meetings into reliable written records: project teams, consultants, planners, architects, researchers, and organisations that handle information which should remain inside their own environment.

The first audience is German-speaking project teams, but the product is not a German-language application. Interface language and meeting language are separate concepts. The same workflow should work in German, English, and other languages when the local models support them.

## Why local matters

Meeting recordings can contain personal information, internal decisions, client details, and material subject to professional or contractual duties. LocaLog is privacy-focused for a practical reason: the core workflow is designed so the content stays on the user's device.

There is no required account, LocaLog cloud, telemetry, or third-party AI service. A future organisation-controlled runtime may be considered, but it would require a separate decision and a plain explanation before any meeting data moved elsewhere.

Local processing is not the whole promise. A private application that is confusing, slow, or careless with revisions would still fail its users. Privacy, data safety, and interface quality belong together.

## The product model

LocaLog keeps work in a simple hierarchy:

```text
Project
└── Meeting
    ├── recording(s)
    ├── transcript revisions
    ├── protocol revisions
    └── exports
```

Projects provide a home for related meetings, vocabulary, and professional defaults. A meeting is the unit of work. An imported recording never becomes an unassigned file floating outside that context.

Stable meeting progress and temporary processing are different things. A meeting may be ready for transcript review while a later generation job is running or has failed. The database keeps those two axes separate so the interface can describe the situation honestly.

## The first complete workflow

```text
Create a project → create a meeting → import a recording
→ prepare and transcribe it locally → review the transcript
→ generate a protocol draft locally → edit and review it
→ export Markdown or plain text
```

The user should be able to stop between each step, understand what has happened, correct mistakes, and return after a restart without losing work.

## Product principles

### The protocol is the outcome

LocaLog is not primarily a transcript viewer and not a general-purpose chatbot. The workflow is shaped around creating a useful protocol with as little unnecessary work as possible.

### Human review stays visible

A generated protocol is a draft. The application keeps revisions, makes the source transcript available, and never silently presents model output as an authoritative final record.

### Local processing should feel calm

Heavy work belongs in the background. Navigation, selection, typing, and editing should remain immediate while transcription or generation runs. Progress should explain what is happening without flooding the interface with technical output.

### Context is more valuable than a file list

The project and meeting structure gives every recording, transcript, vocabulary term, protocol, and export a place. This is what makes the result useful later, not merely impressive at the moment it is generated.

### Technical detail should be available, not dominant

People choose a quality outcome such as “Fast”, “Balanced”, or “Accurate”. Runtime paths, model identifiers, memory limits, and diagnostics belong behind progressive disclosure until the user actually needs them.

### The interface is a core feature

The product should feel like a carefully made desktop writing and productivity tool: quiet surfaces, clear hierarchy, useful empty states, serious editing, honest failures, and dependable recovery. It should not look like a model manager, a generic SaaS dashboard, or an AI playground.

## Scope of the first version

The first complete version focuses on imported audio and video. It includes local transcription, transcript review, local protocol generation, Markdown editing, and Markdown/plain-text export.

Recording from the microphone or system audio remains part of the long-term product direction, but the Record action is not shown until it works reliably on the relevant platforms.

The first version does not include accounts, cloud sync, collaboration, calendar integration, live meeting bots, mobile applications, DOCX/PDF export, semantic search, automatic finalisation, a public provider SDK, or automatic model downloads from arbitrary sources.

Known local models may be downloaded with the user's consent when that is part of the accepted runtime direction. The application must not download anything silently.

## Storage and portability

App-managed storage is the approved working model for v0.1. It is not permission to hide professional data inside an opaque container.

LocaLog must keep the data location documented, provide explicit user-selected exports, and preserve a path to backup/restore and a portable project bundle. Basic backup/restore belongs in v0.1 hardening even if a polished bundle arrives later.

Imported originals are never modified. Committed revisions are immutable and recoverable. Working autosaves are separate from those revisions.

## Success criteria

The first success criterion is not a benchmark score. It is a real, consented or synthetic meeting that can move through the complete workflow and produce a protocol a professional accepts after light editing.

That workflow must be:

- local by default and clear about any exception;
- responsive while heavy work runs;
- recoverable after cancellation, failure, or restart;
- traceable enough that a person can understand which transcript and settings produced a draft;
- useful in both light and dark themes and with keyboard access;
- honest about uncertainty instead of hiding it behind confident language.
