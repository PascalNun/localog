# Later possibilities

The roadmap keeps useful ideas visible without turning them into promises for v0.1. The current product remains the imported-recording workflow described in [MVP.md](MVP.md).

## After the first reliable workflow

### Recording

Microphone recording, system-audio capture, interruption recovery, and multiple synchronised sources may follow the imported-audio path. Recording is deliberately later because permissions and platform behaviour are substantial work, and the product must first prove the protocol workflow.

### Better speaker review

Diarisation can separate voices but cannot know their names. Later work may add project participants, fast keyboard assignment, rename/reassign/merge tools, and carefully controlled local suggestions. A stored voice profile would be sensitive personal data: it would need to be local, visible, project-scoped, individually deletable, and excluded from exports.

### Less manual vocabulary work

Participants, project documents, and previous protocols could suggest vocabulary entries. Suggestions would always require confirmation. The prompt budget means the library must be prioritised rather than allowed to grow without limit.

### Portability and distribution

Windows and Linux packaged builds, signed sidecars, installer updates, backup/restore, and a portable project bundle are future work. They must preserve the same local-first workflow rather than introduce a second product shape.

### Export and retrieval

DOCX/PDF export, basic templates, search across local projects, and structured views for decisions and actions may become useful after Markdown editing and review are stable.

## Deliberately later

- collaboration and sharing;
- accounts and cloud sync;
- calendar integrations and live bots;
- mobile applications;
- semantic search as a broad knowledge layer;
- a public provider/plugin ecosystem;
- hosted processing or organisation-controlled remote processing.

Remote processing is not automatically incompatible with local-first, but it changes the promise from “the content stays on this device” to something weaker. It would require explicit consent, clear destination language, and a separate product decision.

## How to use this roadmap

An idea moves into the current plan only when it solves a demonstrated problem in the current workflow, has an accepted boundary, and can be evaluated without making the application harder to understand. Until then, it belongs here and should not appear as an active control in the interface.
