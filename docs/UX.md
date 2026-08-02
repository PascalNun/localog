# Authoritative UX and interaction specification

This document describes LocaLog from the user’s point of view. It is detailed because quiet, dependable software depends on decisions about ordinary moments: where work belongs, what happens next, how saving is communicated, and what remains safe when a process fails.

This document is the primary interaction contract for LocaLog. Product and architecture documents define promise, scope, and technical boundaries. If an implementation convenience conflicts with this contract, the contract wins unless the product decision is explicitly changed.

## Product experience contract

LocaLog is a project-based professional meeting-documentation application. It is not a generic transcription utility, file bucket, chatbot, model playground, or dashboard.

The product succeeds only when private local processing becomes a calm, immediate, intuitive, and trustworthy desktop workflow. Interface quality is equal in importance to local-first architecture and reliable data handling.

Every screen and interaction should be judged by whether it helps the user reach a useful protocol with less effort and greater confidence. The interface should follow the user’s task rather than expose the internal shape of the processing pipeline. Technical capability that adds friction, delay, or uncertainty without improving the result does not improve the product.

The user must always be able to understand:

1. which project and meeting are active;
2. which stable meeting stage has been reached;
3. whether a local job is running;
4. what action is expected next;
5. what remains safe if work fails or is cancelled.

## Object hierarchy and invariants

```text
Project
└── Meeting
    ├── Recording or imported source
    ├── Transcript and revisions
    ├── Protocol draft and revisions
    └── Exports
```

- The meeting is the central unit of work.
- Every meeting belongs to exactly one project.
- Every imported or recorded source belongs to exactly one meeting.
- Recordings, transcripts, protocols, and exports never become context-free top-level libraries.
- There is no Inbox and no unassigned-recording state.
- A dropped or selected file may be held as a temporary OS reference while placement is confirmed, but it is not copied into durable managed storage until a project and meeting have been established.
- Imported original media is never silently modified or deleted.
- A meeting may eventually own multiple synchronized sources, although v0.1 imports one source.

## Persistent application structure

The normal window contains:

1. a persistent slim left sidebar;
2. one dominant main workspace;
3. an optional contextual inspector only when the current task benefits from it.

Do not reserve a permanent empty third column. The main workspace remains visually dominant.

### Sidebar information architecture

```text
LocaLog

Projects
  Project list
  + New Project

Library
  Protocol Styles
  Vocabulary

Settings
```

`Export Templates` appears only when functional. Other later areas are also omitted rather than shown as disabled navigation.

The sidebar never contains Inbox, Recordings, Transcripts, Protocols, account/avatar controls, sharing/team controls, chat, or model-manager navigation.

### Navigation rules

- Selecting a project replaces the main workspace with that project's overview while the sidebar remains stable.
- Selecting a meeting opens one coherent meeting workspace; source, transcript, protocol, and exports are stages within it.
- Selecting Protocol Styles or Vocabulary opens a full library workspace.
- Selecting Settings opens a full settings workspace, not a modal.
- The selected project remains visible in the sidebar while one of its meetings is open.
- Breadcrumbs or a compact context line identify the current project and meeting in dense workspaces.
- Browser-like back/forward behaviour and restoration of the last useful selection are desirable. Route implementation is not part of the product contract.
- On narrow windows, the sidebar may become a dismissible overlay. Keyboard focus returns to its opener when closed.
- On desktop, the sidebar defaults to 248 px and may be resized between 216 px and 360 px. The divider supports pointer and keyboard adjustment, double-click or Enter restores the default, and the chosen width is stored locally. The narrow-window overlay ignores the desktop width.
- Disabled placeholders are avoided. If a destination is not useful yet, hide it.

## Primary workflow

```text
Choose or create project
→ Import audio/video
→ Confirm minimal meeting information
→ Import and normalise locally
→ Confirm resolved transcription choices
→ Transcribe locally
→ Review transcript
→ Generate protocol draft
→ Edit and review
→ Export
```

Entry from an existing project begins with Import. Entry from the Start page or a dropped file first asks the user to choose or create the destination project. The experience may use a compact staged form or sheet, but it must not create an Inbox or force an elaborate wizard.

The application creates the meeting and assigns the source together when the user confirms setup. Copying into managed storage then begins as a transient import job. A failed or cancelled import must not leave a durable orphan source.

## Start and empty state

When no project or meeting is selected, the main workspace uses a calm, spacious hero composition that makes one next action unmistakable.

Required content:

- one clear **Import recording** action;
- an appropriate central drop target;
- a concise local-processing statement;
- an explanation that the source becomes part of a project meeting.

If a file is dropped before a project is selected, open the lightweight placement flow before copying the file. Preserve the temporary file reference only as long as needed to complete or cancel that flow.

Do not show Record until built-in recording works. Do not show model selectors, onboarding cards, generic dashed web-upload panels, bright blue buttons, AI imagery, or account controls.

## Project creation and overview

### New project

The minimum new-project form asks for a name. Optional metadata—client, project number, description, defaults—uses progressive disclosure and can be added later.

Creating a project from an import flow returns directly to meeting setup with the new project selected. It must not strand the chosen source or force the user to restart import.

### Project overview

The overview contains:

- project name and optional metadata;
- meetings ordered by date, newest first;
- a restrained **New meeting** action;
- access to project defaults and project vocabulary;
- a useful empty state when no meetings exist.

Date and editable title are the primary meeting-row information. Secondary information may include lifecycle, duration, participants, transcript/protocol readiness, and latest safe action.

Meeting titles may initially derive from the imported filename, meeting date, or user input. Later local suggestions are optional. The title always remains editable.

The overview is a document index, not a metric dashboard. Do not introduce analytics cards, completion charts, AI scores, or model statistics.

## New meeting and import setup

The setup is lightweight and uses project defaults. Required information is limited to what is needed to place and process the meeting safely.

### Professional fields

- project;
- selected source;
- editable meeting title;
- date/time where useful;
- meeting/content language;
- participants, optional in v0.1;
- resolved vocabulary;
- protocol style;
- understandable transcription preset.

Inherited project values are visibly identified and directly overridable. Interface language and meeting/content language remain separate concepts.

The normal transcription choices are **Fast**, **Balanced**, and **Accurate**, with a sensible default. Exact Whisper model names, runtime, model paths, quantisation, chunking, and similar details remain in Advanced settings.

The resolved writing provider/model may be shown quietly, but model selection must not dominate meeting creation. The user's goal is a protocol, not operation of an inference runtime.

### Setup behaviour

- Entry from a project preselects that project.
- Entry from Start or drop begins with project placement.
- Project defaults prefill language, vocabulary, participants, protocol style, and processing preset.
- The user can review what will happen without understanding low-level settings.
- Confirmation creates the meeting, assigns the source, and begins the import job.
- After import/normalisation succeeds, the meeting becomes source-ready and exposes **Transcribe**. The initial shell may keep this as an explicit action so resolved professional settings remain reviewable.
- Advanced controls are collapsed and never required for the default path.

## Defaults and overrides

Configuration resolves as:

```text
Global defaults < Project defaults < Meeting overrides
```

| Scope   | Examples                                                                                              | Presentation                      |
| ------- | ----------------------------------------------------------------------------------------------------- | --------------------------------- |
| Global  | Interface appearance, transcription preset, writing provider/model, export type, storage behaviour    | Settings                          |
| Project | Meeting language, vocabulary, recurring participants, protocol style, metadata, later export template | Project settings/context          |
| Meeting | Title/date, participant changes, language, vocabulary/style overrides, processing preset              | New meeting and meeting workspace |

The UI shows the resolved professional value and its source—such as “Project default”—without exposing prompt assembly or runtime internals. A job snapshots the resolved settings when it starts. Later setting changes do not silently alter the running job or its provenance.

## Vocabulary ownership and behaviour

Vocabulary is a first-class structured library object, not a training claim or free-form prompt.

### Global or office vocabulary

Managed in Library → Vocabulary. Suitable entries include common abbreviations, technical terms, recurring organisations, and standard professional wording.

### Project vocabulary

Managed from the project context or project settings. Suitable entries include the project name, organisations, consultants, participants, building parts, addresses, places, products, and project-specific abbreviations.

Project vocabulary applies by default to the project's meetings. Users should not repeatedly select it unless they want a meeting-specific override. Each entry may carry preferred spelling, category, aliases, note, scope, and enabled state.

Vocabulary may assist transcription context and protocol generation where supported. The UI must not promise fine-tuning, training, or perfect spelling.

## Protocol-style ownership and behaviour

Protocol styles are first-class reusable professional document presets.

- Global styles live in Library → Protocol Styles.
- A project may define a default style.
- A meeting may override it.
- Styles may have German and English variants without coupling them to interface localisation.

Examples include short internal working note, formal minutes, task list, client summary, and technical decision log.

The normal workflow presents style names, descriptions, and expected document outcomes—not raw prompts. Internally a style may contribute structured instructions. A future advanced editor may expose those instructions in a controlled form.

## Stable meeting lifecycle and transient jobs

Persistence and domain logic keep two independent axes.

### Stable meeting lifecycle

- `draft`: meeting exists; source import has not committed successfully;
- `source_ready`: original and required working source are committed;
- `transcript_ready`: a committed transcript revision exists;
- `protocol_draft`: a committed editable protocol revision exists;
- `reviewed`: the current protocol revision has been explicitly reviewed;
- `archived`: the meeting is retained but removed from active work.

Editing a reviewed protocol must not silently leave it reviewed. Exact reviewed/changed-since-review semantics remain an approval question.

Export is a related record, not a lifecycle replacement. A meeting can have multiple exports while remaining protocol-draft or reviewed.

### Transient job state

- `queued`;
- `running`;
- `cancelling`;
- `failed`;
- `interrupted`;
- `completed`, with a success or cancellation outcome.

Job kind and stage explain whether import, transcription, or generation is active. A UI presentation state combines stable lifecycle, active/latest job, and recoverable error without persisting fake meeting states such as `transcribing`.

### Lifecycle/job presentation matrix

| Stable lifecycle | Job condition                    | Main presentation                                                    | Expected action             |
| ---------------- | -------------------------------- | -------------------------------------------------------------------- | --------------------------- |
| Draft            | No job                           | Source/setup summary                                                 | Import or repair source     |
| Draft            | Import queued/running            | Honest import/normalisation progress                                 | Cancel                      |
| Draft            | Import failed/interrupted        | Error plus source safety/placement explanation                       | Retry or change source      |
| Source ready     | No job                           | Resolved transcription choices                                       | Transcribe                  |
| Source ready     | Transcription queued/running     | Transcript stage progress; navigation remains available              | Cancel                      |
| Source ready     | Transcription failed/interrupted | Source remains safe; actionable diagnostics                          | Retry or change settings    |
| Transcript ready | No job                           | Transcript review workspace                                          | Generate protocol           |
| Transcript ready | Generation queued/running        | Transcript remains readable/editable where safe; generation progress | Cancel                      |
| Transcript ready | Generation failed/interrupted    | Transcript remains safe and available                                | Retry or change settings    |
| Protocol draft   | No job                           | Protocol editor and export                                           | Edit, mark reviewed, export |
| Reviewed         | No job                           | Reviewed status plus document/export                                 | Export or deliberately edit |
| Archived         | No active job                    | Read-only/restore-oriented summary                                   | Restore to active work      |

A cancelled job returns to the latest stable lifecycle and never deletes a completed artifact. A killed app marks abandoned work interrupted and must not present partial output as complete.

## Coherent meeting workspace

Selecting a meeting opens one workspace that changes with stable lifecycle and active jobs. It is not split into unrelated top-level screens.

A restrained internal stage indicator may use:

```text
Source    Transcript    Protocol
```

This indicator is navigation/presentation, not the persisted lifecycle model. The workspace preserves access to earlier stable artifacts where useful—for example, the transcript remains available while protocol generation runs.

## Transcript review

Transcript review is a primary professional workspace.

Required v0.1 behaviour:

- readable segments in time order;
- timestamps that seek the source audio;
- a compact audio transport with play/pause and seeking;
- editable transcript text;
- generic or manually assigned speaker labels;
- speaker renaming/mapping without implying automatic diarisation;
- review and correction of unclear terms;
- vocabulary suggestions where useful and honest;
- visible autosave state and recovery;
- a clear **Generate protocol** action;
- access to meeting context and resolved style/vocabulary.

Automatic diarisation is not required. Until it exists, synthetic examples and UI language use `Speaker 1`, `Speaker 2`, or user-assigned participant names only after explicit mapping. Do not display confident participant avatars or inferred speaker identities.

The screen may use a dismissible inspector for speakers and unclear terms. At narrower widths it becomes a drawer or inline section so transcript reading remains dominant.

Search and filtering are useful but must not delay the core correction/generation path. A detailed waveform is later scope; a functional time-based transport is sufficient for v0.1.

## Protocol editor

The protocol editor is a professional writing workspace backed by canonical Markdown.

Required behaviour:

- edit protocol content directly;
- autosave working state separately from committed immutable revisions;
- show saved/unsaved/recovered state quietly and clearly;
- expose current protocol style information;
- show draft, reviewed, or changed-since-review status;
- provide Markdown and plain-text export;
- provide access back to the reviewed transcript;
- retain revision/provenance information without overwhelming the document.

The document remains visually dominant. Formatting support should be conservative and Markdown-safe. A full toolbar, section visibility controls, sharing, and rich editor features are not automatic MVP requirements.

### Contextual refinement

LocaLog is not chat-first. Optional AI-assisted refinement is explicitly invoked and contextual, for example:

- select text and choose Rewrite;
- choose Refine protocol;
- open a temporary command field;
- enter a short instruction such as “make this section shorter.”

The instruction field disappears when the refinement task ends. There is no permanent bottom prompt or general-purpose chat history. Refinement produces a reviewable draft/revision and never silently finalises the protocol.

## Settings behaviour

Settings replaces the large workspace while preserving the sidebar. Primary settings never use a cramped modal.

Initial categories:

- General;
- Models;
- Transcription;
- Storage;
- Privacy;
- Appearance;
- Advanced, only for diagnostics and justified technical controls.

Product-language labels appear before runtime terminology. Models and Transcription show usable defaults first; exact paths, runtime details, context windows, and similar controls use progressive disclosure.

Storage explains the app-managed data location and provides reveal/open guidance where safe. Backup/restore UI remains subject to the v0.1 hardening decision. Privacy explains local processing and the separate privacy boundary of an externally installed loopback provider.

Interface language and meeting/content language are separate settings. German is not hard-coded as the application interface language.

## Contextual inspector rules

An inspector is appropriate for:

- transcript speakers and unclear terms;
- protocol style, revision context, and export;
- project metadata/defaults;
- advanced import options.

Rules:

- only appear when it supports the active task;
- remain dismissible;
- never leave an empty permanent column;
- never make the inspector wider or more visually dominant than the main work;
- become a drawer or inline disclosure on compact windows;
- preserve keyboard order and return focus when closed.

## Built-in recording in Phase 2

Built-in recording remains part of the product architecture but is absent from the active v0.1 interface until it works reliably.

The later recording flow must:

- choose or create the project and meeting context before durable capture begins;
- support microphone input;
- support system/desktop audio where the platform and permissions permit it;
- store sources locally and assign them directly to the meeting;
- handle permission denial, interruption, partial capture, cancellation, and restart recovery honestly;
- preserve the one-meeting-to-many-sources model without creating a top-level recording library.

The Phase 2 recording design requires a separate platform and permission review. Earlier exploratory controls do not pre-approve its interaction design.

## Visible professional choices and hidden technical details

### Show in the normal workflow

- project and meeting;
- editable title and date;
- participants;
- meeting/content language;
- Fast/Balanced/Accurate transcription preset;
- protocol style;
- vocabulary and inheritance;
- lifecycle and job progress;
- local-processing status;
- export format.

### Hide by default

- temperature and token limits;
- quantisation;
- context-window internals;
- chunking;
- low-level model paths;
- raw provider configuration;
- prompt assembly;
- process stdout/stderr;
- hardware tuning controls.

These details may appear in Advanced settings where justified and validated.

Local-first operation is a product invariant, not an on/off mode, so the interface must not use a permanent green “local mode active” badge. Persistent operational status is reserved for information that can actually change and require attention: local runtime or model availability, active processing, cancellation, failure, or missing setup. Privacy explanations belong in appropriate trust and onboarding context rather than masquerading as live status.

## Progress, failure, cancellation, and recovery

- Heavy work never blocks navigation, typing, selection, audio control, or document editing.
- Progress is determinate only when measurable. Do not invent time remaining or “AI magic” animation.
- High-frequency runtime output is throttled or batched before reaching the frontend.
- Cancellation is explicit and reports when safe shutdown is still in progress.
- Every failure state explains what remains safe, what failed, and the next useful action.
- Missing runtime/model, unsupported/corrupt media, insufficient disk space, provider timeout, process crash, invalid output, permission denial, and export collision receive designed states.
- Diagnostic detail sits behind disclosure and excludes content/user paths by default.
- Closing the window or relaunching must not make partial output appear complete.
- Destructive actions use archive/trash semantics where practical and confirm only material irreversible effects.

## Responsive, keyboard, and accessibility behaviour

- Ordinary navigation, selection, typing, and editing should generally respond within approximately 100 ms during background work.
- All core actions are keyboard reachable with a visible `focus-visible` treatment.
- Essential actions never exist only on hover.
- Controls have accessible names; status changes use non-disruptive announcements.
- Text scaling to 200% must preserve the core workflow without clipped actions or overlapping columns.
- Reduced-motion preferences remove nonessential transitions.
- Colour is never the only status signal.
- Main interactive targets should normally be at least 40 × 40 px.
- At compact widths, secondary metadata reduces before core actions, the sidebar becomes an overlay, and inspectors become drawers/inline disclosures.
- The main workspace—not the whole window chrome—owns the primary scroll region where practical.

## Visual and design-system foundation

The goal is a consistent visual character across every real state, not a collection of individually styled screens.

### Typography

Barlow is the approved primary application typeface for navigation, controls, labels, metadata, headings, transcript text, and initial protocol editing. It is bundled locally; no runtime font request is permitted.

Initial font stack:

```css
"Barlow", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif
```

Initial hierarchy:

| Role            | Size / line height                   | Weight            | Use                              |
| --------------- | ------------------------------------ | ----------------- | -------------------------------- |
| Display         | `clamp(2.75rem, 5vw, 4.8rem)` / 0.98 | 400               | Start hero only                  |
| Workspace title | `2rem` / 1.1                         | 500               | Project, meeting, settings title |
| Section heading | `1.25rem` / 1.25                     | 500               | Major workspace section          |
| Body            | `0.98rem` / 1.55                     | 400               | Reading/editing                  |
| UI              | `0.875rem` / 1.35                    | 500               | Controls/navigation              |
| Meta            | `0.75rem` / 1.4                      | 500               | Secondary information            |
| Label           | `0.6875rem` / 1.3                    | 600 with tracking | Eyebrows/group labels            |

Load only the required 400, 500, and 600 styles initially. Record font source, licence, version, selected assets, and checksums before committing them. No complementary document typeface is approved; add one only for a demonstrated reading or interaction need.

### Light and dark tokens

| Semantic token   | Light     | Dark      | Role                        |
| ---------------- | --------- | --------- | --------------------------- |
| Canvas           | `#f5f2ec` | `#1a1917` | Window background           |
| Sidebar          | `#eeeae3` | `#1e1d1a` | Persistent navigation plane |
| Workspace        | `#fbf9f5` | `#23211e` | Dominant work surface       |
| Raised surface   | `#fffefa` | `#2a2824` | Contextual controls only    |
| Subtle selection | `#eeeae4` | `#302e29` | Quiet selected/hover state  |
| Primary ink      | `#282621` | `#f3ede2` | Essential text              |
| Muted ink        | `#6e6961` | `#b8b0a5` | Secondary text              |
| Fine line        | `#ddd7ce` | `#403d37` | Dividers/borders            |
| Primary action   | `#34322d` | `#eee5d6` | Restrained primary control  |
| Action ink       | `#faf8f3` | `#211f1c` | Text on primary control     |
| Success          | `#4f825d` | `#76ad80` | Safe/completed state        |
| Warning          | `#a66e25` | `#d2a05d` | Review/caution              |
| Danger           | `#a14f48` | `#d47b72` | Failure/destructive state   |
| Focus            | `#776b54` | `#d3bc8b` | Keyboard focus ring         |

Dark mode is designed, not inverted: warm charcoal replaces pure black; primary text remains warm; surfaces differ subtly; no glow, glass, gradients, or neon effects are used. Contrast and visible focus take precedence over preserving exploratory colour values.

### Spacing and layout grid

- Base unit: 4 px; shared steps: 4, 8, 12, 16, 24, 32, 48, 64, 96 px.
- Sidebar: approximately 248 px wide; compact desktop approximately 216 px; overlay below roughly 900 px.
- Workspace gutters: 56–72 px wide, 32 px compact, 20 px at minimum supported width.
- Readable workspace maximum: approximately 1240 px where the task does not require full width.
- Inspector: 288–320 px when present.
- Transcript rows align timestamp, speaker, and text columns consistently.
- Use planes, whitespace, and fine rules before boxes. Avoid nested cards, excessive radius, pills, shadows, or filled panels.

### Common control states

Every control defines default, hover, active, `focus-visible`, disabled, loading/busy, error, and success behaviour. Primary actions use restrained charcoal/warm-light contrast, never bright blue. Secondary actions use typography or fine borders. Motion is short and functional; reduced motion removes it.

## Screen-specific design emphasis

1. The start screen establishes identity through space, typography, one clear action, and an honest local-processing statement.
2. New Meeting uses progressive disclosure and quiet controls rather than a dense configuration form.
3. Project overview is a chronological working index, not a metric dashboard.
4. Transcript review can be information-dense while preserving a clear reading hierarchy and honest speaker labels.
5. Protocol editing gives the document priority; style, provenance, review, and export belong in contextual support.
6. Dark mode uses warm charcoal planes and restrained contrast rather than inversion, glow, or card-heavy composition.

## Screen/state matrix

| Screen or context      | Primary purpose                                  | Required data                                                       | Primary action          | Secondary actions                                  | Empty state                                    | Loading/running state                                     | Failure state                                            | Completed state                        | Design emphasis               | Scope                                        |
| ---------------------- | ------------------------------------------------ | ------------------------------------------------------------------- | ----------------------- | -------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------- | -------------------------------------------------------- | -------------------------------------- | ----------------------------- | -------------------------------------------- |
| Start                  | Begin a structured meeting                       | Projects sufficient for placement                                   | Import recording        | Select recent project                              | Explain project placement and local processing | Placement/import progress appears only after confirmation | Source/placement error retains safe temporary context    | Opens meeting workspace                | Spacious hero and local trust | MVP                                          |
| New project            | Establish required meeting context               | Name; optional metadata/defaults                                    | Create project          | Cancel                                             | Not applicable                                 | Inline save state                                         | Validation/storage error                                 | Returns to originating flow/project    | Focused, brief setup          | MVP                                          |
| Project overview       | Find/create meetings in context                  | Project metadata, newest-first meetings                             | New meeting             | Edit defaults/vocabulary, open meeting             | Explain first import                           | Quiet list refresh only                                   | Preserve cached list and offer retry                     | Meeting index                          | Quiet chronological index     | MVP                                          |
| New meeting/import     | Assign source and resolved professional settings | Project, source, title/date, language, vocabulary, style, preset    | Confirm and import      | Change source/project, advanced disclosure, cancel | Source-selection guidance                      | Import/normalisation progress after creation              | Explain source safety and retry/change source            | Source-ready meeting                   | Calm staged setup             | MVP                                          |
| Meeting: source ready  | Review resolved transcription setup              | Source metadata, settings provenance                                | Transcribe              | Edit title/settings, reveal source                 | Missing-source repair state                    | Transcription progress                                    | Missing runtime/model/media diagnostics; source retained | Transcript-ready workspace             | Workflow continuity           | MVP                                          |
| Transcript review      | Correct transcript before generation             | Audio transport, timestamps, segments, speaker labels, review flags | Generate protocol       | Edit, seek, rename/map speakers, resolve terms     | Explain unavailable/empty transcript           | Generation may run without hiding stable transcript       | Transcript/generation errors retain source/transcript    | Reviewed transcript remains accessible | Dense reading hierarchy       | MVP                                          |
| Protocol editor        | Produce controlled professional document         | Markdown draft, style, provenance, save/review state                | Export or mark reviewed | Edit, return to transcript, contextual refine      | Explain missing/failed generation              | Autosave/generation status without blocking editing       | Recover working draft or retry generation/export         | Draft/reviewed protocol revision       | Document dominance            | MVP                                          |
| Protocol refinement    | Make an explicit bounded AI-assisted change      | Selected scope/current revision, temporary instruction              | Apply draft refinement  | Cancel                                             | Not shown until invoked                        | Local generation progress                                 | Original revision retained; retry/cancel                 | New reviewable draft/revision          | Contextual disclosure         | Later unless generation spike keeps it small |
| Vocabulary library     | Maintain reusable terminology                    | Global sets/entries                                                 | Add/edit entry          | Enable, disable, filter                            | Explain global vs project vocabulary           | Local save/import state                                   | Validation/storage error                                 | Updated reusable vocabulary            | Practical library utility     | MVP core, richer import later                |
| Protocol-style library | Maintain document presets                        | Style name, description, structured expectations                    | Add/edit style          | Duplicate, set defaults                            | Explain professional presets                   | Local save state                                          | Validation/storage error                                 | Updated reusable style                 | Outcome-led presets           | MVP                                          |
| Settings               | Configure product defaults and local runtimes    | General, Models, Transcription, Storage, Privacy, Appearance        | Save/apply setting      | Reveal advanced diagnostics                        | Explain missing runtime/model where relevant   | Discovery/check status                                    | Actionable diagnostics without content                   | Resolved defaults visible              | Progressive disclosure        | MVP                                          |
| Contextual inspector   | Support current task without displacing it       | Context-specific review/style/export metadata                       | Resolve current context | Dismiss                                            | Not rendered                                   | Bounded local state only                                  | Inline contextual error                                  | Returns focus to main task             | Task-specific support         | MVP where specified                          |
| Recording              | Capture mic/system audio into meeting            | Project, meeting, permissions, sources                              | Start/stop recording    | Input selection                                    | Permission/setup guidance                      | Live resilient capture                                    | Permission/interruption recovery                         | Source assigned to meeting             | Separate later design         | Phase 2                                      |

## Explicit exclusions for v0.1

- active microphone or system-audio recording;
- automatic diarisation as a requirement;
- Inbox or orphan source queue;
- top-level recording/transcript/protocol libraries;
- accounts, avatars, sharing, teams, collaboration, or cloud sync;
- calendar integration, meeting bots, live transcription, or mobile apps;
- permanent AI chat or general-purpose prompt field;
- public provider/plugin SDK or model-manager navigation;
- automatic model downloads;
- DOCX/PDF export or export-template designer;
- semantic organisation-wide search;
- automatic finalisation or compliance claims;
- rich formatting, section visibility, or toolbar features without an independently justified workflow need;
- decorative AI imagery, sparkles, gradients, glowing/glass surfaces, dashboard metrics, or bright default-blue actions.

## Reconciliation of earlier contradictions

- Stable meeting lifecycle and transient processing are now explicitly separate.
- Start and New Meeting omit Record until recording works; recording remains Phase 2.
- Export Templates is hidden until functional rather than shown as a disabled library destination.
- Dropped/selected files require project placement before durable copying, preserving the no-orphan invariant.
- Transcript speaker UI does not imply automatic diarisation or reliable identity inference.
- Protocol refinement is contextual and temporary; there is no permanent chat field.
- Sharing, avatars, account UI, rich toolbars, and generic dashboard elements are non-requirements.
- Project meetings are ordered chronologically with newest first, resolving the previous ambiguous wording.
- A restrained Source/Transcript/Protocol rail is presentation navigation, not a persisted state machine.
- New Meeting is a lightweight staged setup, not a mandatory multi-page wizard.

## Remaining product approval questions

1. What exact action marks a protocol reviewed, and does any content edit move it to `protocol_draft` or to a distinct changed-since-review presentation?
2. Are participants a manually managed meeting list in v0.1, and should speaker mapping be allowed to create participants from the transcript workspace?
3. Which interface locales ship initially, and how is the independent first-run meeting/content language chosen?
4. For a file dropped on Start, should project placement precede the file chooser strictly, or is retaining the temporary OS file reference through placement acceptable as specified here?
5. Which project/meeting archive and restore actions are required in v0.1, and is permanent deletion intentionally absent?
6. What minimum backup/restore controls must appear during v0.1 hardening?
7. Should contextual protocol refinement enter v0.1 only if the generation/editor spikes show it can be implemented without weakening revision clarity?
