# LocaLog documentation

LocaLog is being designed around one clear outcome: helping people turn meeting recordings into useful, reviewed protocols with local AI. Transcription provides the source; the editable written record is the destination. The interface, local processing, data safety, and professional workflow are equally important parts of reaching it.

These documents move from intention to implementation so that the project can be understood without beginning in source code.

You do not need to be a software developer to read them. Technical terms are used where precision matters, but each document begins with the purpose behind the machinery.

## A useful reading order

1. **[Product definition](PRODUCT.md)** explains the problem LocaLog addresses, who it is for, and what it deliberately does not try to become.
2. **[Experience and interaction](UX.md)** describes the application from the user’s point of view: projects, meetings, review, editing, progress, failure, and recovery.
3. **[Visual direction](VISUAL_DIRECTION.md)** defines the restrained, editorial character of the interface and the design rules behind it.
4. **[v0.1 scope](MVP.md)** turns the product idea into a bounded first release and an ordered implementation plan.
5. **[Technical architecture](ARCHITECTURE.md)** explains how the interface, application core, files, database, background work, and local AI tools fit together.
6. **[Decisions](DECISIONS.md)** records accepted choices, evidence, remaining risks, and questions that still require an answer.
7. **[Working plan](PLAN.md)** tracks what is actually built today and what happens next. It is the living document; the others describe the goals being worked toward.
8. **[Polishing plan](POLISH.md)** describes what stands between a working pipeline and something a professional would use every week.
9. **[Model evaluation](MODEL_EVALUATION.md)** records which models have actually been run against real meeting audio, what came out, and what failed.
10. **[Transcription and speaker experience](TRANSCRIPTION_EXPERIENCE.md)** specifies the seamless model choice and automatic speaker separation (bundled runtime, on-demand models, diarisation).
11. **[Roadmap](ROADMAP.md)** keeps later possibilities visible without treating them as decided.

## A few recurring terms

- **Local-first** — LocaLog’s normal meeting workflow is designed to run on the user’s computer. It does not require a LocaLog account or cloud service, and it does not silently upload meeting content.
- **Protocol** — The structured written record or minutes of a meeting. In this project, “protocol” is a document—not a networking term.
- **Runtime** — A local program that performs specialised work, such as FFmpeg for media processing or an inference engine for transcription and language generation.
- **Adapter** — A small translation layer between LocaLog and a particular operating system or runtime. Adapters keep product rules independent from macOS, Windows, Linux, FFmpeg, Ollama, or one specific model.
- **Artifact** — A file produced or preserved by the application, such as an imported recording, a committed transcript revision, or a protocol revision.
- **Canonical** — The one authoritative version of content. LocaLog avoids keeping two independently editable “master” copies of the same transcript or protocol.
- **Provenance** — The information needed to understand how a result was produced: application and runtime versions, model identity, settings, style and vocabulary revisions, and input checksums where available. Provenance supports repeatable inputs; it does not guarantee identical model output.
- **Lifecycle and job state** — A meeting’s durable stage—such as transcript ready—is different from temporary background activity—such as a transcription job currently running or failing. Keeping them separate prevents an interrupted process from corrupting the meeting’s stable state.
- **Spike** — A deliberately isolated technical study used to answer a risky question with evidence. Spike code does not become production architecture automatically.

## Platform direction

macOS is the first development, performance, and packaging environment because that is where the project is currently being built. LocaLog is intended as a cross-platform desktop application for macOS, Windows, and Linux.

Shared product and application logic must remain platform-independent. Differences in process handling, data locations, permissions, recording, acceleration, signing, and packaging are kept behind operating-system-specific adapters.

## Status language

The decision documents use four labels:

- **Accepted** — the current direction, supported by a product decision or completed evidence.
- **Proposed** — a serious working hypothesis that still needs validation.
- **Approval required** — a choice that changes product behaviour, data safety, distribution, or long-term architecture.
- **Deferred** — intentionally not being decided or implemented in the current phase.

The repository is public while the application is still early. “Documented” does not mean “implemented,” and “validated in a spike” does not mean “ready for distribution.” The [README](../README.md) gives the current implementation status.
