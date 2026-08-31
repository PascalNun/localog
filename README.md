# LocaLog

### Local-first AI for turning meeting recordings into protocols.

LocaLog is a desktop application for people who need a written record after a meeting, but cannot send the conversation to a cloud AI service.

You import an audio or video recording, review the local transcript, and use a local language model to prepare a protocol draft. You can edit the draft before exporting it. Projects and meetings keep the recording, transcript, protocol, and exports together.

The protocol is the point of the product. Transcription is the reviewable source that makes a reliable protocol possible.

<p>
  <img src="docs/assets/screenshots/localog-start-light.png" alt="LocaLog light start screen" width="49%" />
  <img src="docs/assets/screenshots/localog-start-dark.png" alt="LocaLog dark start screen" width="49%" />
</p>

_The current shell uses synthetic project data. These screenshots show the visual direction in light and dark mode, not a finished release._

## Download

The first alpha is on the [releases page](../../releases/latest), for macOS,
Windows and Linux. It is unsigned, so each system will warn you before opening
it; the release notes say what to click and why. Read the untested list there
before you rely on the output.

## Why LocaLog exists

Meeting recordings can contain personal information, internal decisions, client details, and material that should not leave an organisation's controlled environment. LocaLog is designed around that:

- meeting content stays on the device through the core workflow;
- no LocaLog account, cloud workspace, telemetry, or hosted AI service is required;
- generated text is visible and editable before it becomes an exported record;
- projects and meetings provide context for every recording and document;
- the interface is designed as a writing and review tool, not a chatbot or model dashboard.

The project is open source under [GPL-3.0-or-later](LICENSE). It builds and packages on macOS, Windows and Linux. macOS is the platform the work is validated on: the other two compile and bundle in CI, and nobody has run them yet. Operating-system-specific work stays at the edges of the application.

## The workflow

```text
Project → Meeting → Import → Transcribe → Review → Generate → Edit → Export
```

Importing a recording is the path that works. Recording a meeting inside the application is built on all three systems now, through AVAudioEngine on macOS, WASAPI loopback on Windows and an ALSA monitor source on Linux. Only the macOS backend has ever captured audio, and that was in the spike it came from rather than from inside the application.

## Current state

LocaLog is an early working prototype. It is useful for testing the workflow and architecture, but it is not ready for production use.

The repository currently contains:

- a Tauri desktop shell with Svelte and TypeScript on the frontend and Rust at the core;
- a warm light/dark visual system using locally bundled Barlow typography;
- project and meeting storage in SQLite with versioned transcript and protocol artifacts;
- durable import, transcription, and generation jobs with cancellation, retry, and restart recovery;
- local media probing and normalisation through supervised FFmpeg processes;
- a whisper.cpp boundary for local transcription, with consent-gated verified model downloads;
- a bundled llama.cpp server for writing protocols, and an Ollama provider for anyone already running one;
- transcript review, Markdown editing, autosave, revision history, and Markdown/plain-text export;
- synthetic fixtures, evaluation harnesses, and isolated architecture spikes.

Seven sidecars ship inside the application: whisper.cpp, FFmpeg and ffprobe, the llama.cpp server, speaker diarisation and embedding, and the recorder. A locally supplied whisper.cpp executable and a user-managed Ollama server are both still accepted. Signing and notarisation, M1/8 GB performance validation, accessibility auditing, and backup/restore are open.

The main unresolved product question is protocol quality: whether the generated document is complete, factually supported, and useful after light editing.

## Documentation

Start with the [documentation guide](docs/README.md). It explains which document to read for a particular question and distinguishes product goals, accepted decisions, current implementation status, and experimental evidence.

- [Product](docs/PRODUCT.md) — what LocaLog is for, who it serves, and what belongs in the first version
- [Experience and UX](docs/UX.md) — how the application should behave and feel
- [Visual direction](docs/VISUAL_DIRECTION.md) — typography, colour, spacing, and visual character
- [MVP](docs/MVP.md) — the first complete workflow and its boundaries
- [Architecture](docs/ARCHITECTURE.md) — how the application stores data and runs local work
- [Decisions](docs/DECISIONS.md) — accepted choices, open questions, and the evidence behind them
- [Current plan](docs/PLAN.md) — what is true now and what should happen next
- [Roadmap](docs/ROADMAP.md) — later possibilities, not promises for the current release

The `spikes/` folders contain isolated studies. They are evidence and test oracles, not production modules. The local-only `eval/` folder may contain real evaluation material and must never be committed or shared.

## Running the current build

The browser shell is useful for visual and workflow development. It uses synthetic data. The native shell exercises the real storage and process boundaries when the required local runtimes are available.

Prerequisites:

- Node.js 22.12 or newer
- npm
- Rust 1.97.1 through rustup

```sh
npm install
npm run dev
```

To run the Tauri shell:

```sh
npm run tauri dev
```

For a packaged build, which compiles every sidecar first and then bundles them:

```sh
npm run tauri:build
```

Expect this to take a while the first time; FFmpeg and llama.cpp are built from source. Model files are never bundled. They download when the feature that needs them is first used, with the size shown and, where a licence requires it, the terms shown before anything is fetched.

The complete checks are documented in [CONTRIBUTING.md](CONTRIBUTING.md).

## Contributing

The project is public while its foundations are still being proven. Before changing behaviour, read the product and architecture documents and check the current plan. Keep changes small, explain important trade-offs, and do not add real meeting material, credentials, model files, or runtime binaries to the repository.

## Licence

LocaLog is free software licensed under the [GNU General Public License v3.0 or later](LICENSE). You may use, study, modify, and redistribute it, including commercially, under the licence terms. Third-party dependencies, fonts, runtimes, models, and other assets retain their own licences.

Copyright © 2026 Pascal Nünninghoff.
