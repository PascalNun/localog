# Later possibilities

The roadmap keeps useful ideas visible without turning them into promises for v0.1. The current product remains the imported-recording workflow described in [MVP.md](MVP.md).

## After the first reliable workflow

### Recording — arrived, on macOS

Microphone and system-audio recording are in the application, and surviving a kill without losing audio was proved in the study before it was built. The reasoning for holding it back was right about the cost: system audio took five attempts and failed all five for the same reason, because macOS hands an unauthorised tap silence rather than an error.

What is still ahead here: a Linux and a Windows recorder, whether two tracks drift apart over a long meeting, and multiple synchronised sources.

### Better speaker review

Diarisation can separate voices but cannot know their names. Later work may add project participants, fast keyboard assignment, rename/reassign/merge tools, and carefully controlled local suggestions. A stored voice profile would be sensitive personal data: it would need to be local, visible, project-scoped, individually deletable, and excluded from exports.

### Less manual vocabulary work

Participants, project documents, and previous protocols could suggest vocabulary entries. Suggestions would always require confirmation. The prompt budget means the library must be prioritised rather than allowed to grow without limit.

### Portability and distribution

Windows and Linux packaged builds, installer updates, backup/restore, and a portable project bundle are future work. They must preserve the same local-first workflow rather than introduce a second product shape.

The sidecars themselves are no longer future work: six of them ship inside the macOS application. Signing them with a real identity is, and it is the last thing between the current bundle and giving it to somebody. `PLAN.md` records what the other two platforms actually cost, which is less than this section assumed when it was written.

### Export and retrieval

DOCX and PDF export are built, both written from the same blocks the screen renders, so there is one document rather than one per destination. Basic templates exist as a separate concept and are due to become presets in the appearance panel instead.

Still ahead: search across local projects, and structured views for decisions and actions.

## Deliberately later

- collaboration and sharing;
- accounts and cloud sync;
- calendar integrations and live bots;
- mobile applications;
- semantic search as a broad knowledge layer;
- a public provider/plugin ecosystem;
- hosted processing or organisation-controlled remote processing.

A mobile or iPad application is a larger job than it looks, and larger than a Windows or Linux port. This architecture spawns sidecars, and iOS does not let a sandboxed application execute another binary. Whisper and the diariser would have to be linked in and called as libraries rather than run as commands, which is a second build of the engine rather than a port of this one. Worth knowing before it is counted alongside the desktop platforms.

Remote processing is not automatically incompatible with local-first, but it changes the promise from “the content stays on this device” to something weaker. It would require explicit consent, clear destination language, and a separate product decision.

## How to use this roadmap

An idea moves into the current plan only when it solves a demonstrated problem in the current workflow, has an accepted boundary, and can be evaluated without making the application harder to understand. Until then, it belongs here and should not appear as an active control in the interface.
