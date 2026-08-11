# Experience and interaction

LocaLog should feel like a quiet professional desktop tool. The person using it is trying to make a useful record of a meeting, not operate an AI system.

The interface is therefore part of the product’s reliability. A calm layout, honest status language, predictable focus, and recoverable editing matter as much as the local model.

## The experience in one sentence

At every moment, the user should know which meeting they are working on, what the application has already done, what needs attention, and what will happen next.

## Navigation

The slim project sidebar stays visible on larger windows. It contains:

- the LocaLog wordmark;
- projects and their meeting counts;
- a clear way to create a project;
- the small library area for protocol styles and vocabulary;
- settings;
- one quiet status line for background work.

The central workspace holds the current task. The sidebar can be resized, and on compact windows it becomes an overlay with an obvious open/close control. On macOS the surfaces extend beneath the native title-bar controls; the application must not draw its own fake traffic lights.

The interface should not reserve a permanent empty inspector. An inspector appears only when the current task benefits from extra context and can be dismissed without losing work.

## The first screen

The start state is light, open, and deliberately sparse. It should answer one question: how do I begin?

The primary action is **Import recording**. Recording itself is not shown until it works. The page explains in plain language that the recording will be transcribed and turned into a reviewable protocol on the device.

The sound mark in the middle of the page is a provisional identity element, not a promise that the application is recording.

## Projects and meetings

Creating a project is a short form. A project can be named, renamed, and eventually archived without making the person configure models or storage first.

The project view is an index, not a dashboard. It should show the meetings that belong to the project, their stable stage, and the next useful action. Empty projects should explain what belongs there and offer one clear next step.

A meeting starts in `draft`. Once the source has been safely imported it becomes `source_ready`. Later stable stages are `transcript_ready`, `protocol_draft`, `reviewed`, and `archived`.

Temporary work is shown separately. A meeting can be transcript-ready while a protocol job is running, or still have a usable transcript after generation fails. The UI combines the two axes into a sentence; it never replaces the stable lifecycle with a progress label.

## New meeting and import

The new-meeting flow asks only for what the application cannot infer:

1. the project;
2. the meeting title and date;
3. the recording.

The recording can be selected with a file picker or dropped onto the import surface. The original is copied into managed storage and never modified. The interface says what is being copied, what has been checked, and what the user can do next.

Unsupported files, duplicate content, missing permissions, disk-full conditions, and cancellation need specific messages. “Something went wrong” is not enough when the application knows what happened.

## Transcript review

Transcript review is a reading and correction surface, not a form with a large text box.

It should provide:

- timestamped segments;
- editable text with a visible save state;
- audio playback and seeking when the working audio is available;
- a way to follow playback without forcing it;
- search and navigation between segments;
- clear marking of words the transcription model was unsure about;
- editable, provisional speaker labels;
- keyboard movement between segments;
- a focused inspector for uncertainty, speakers, and source context.

The first useful location is the first thing that needs attention, not necessarily the top of an 800-segment transcript. Editing a segment settles its uncertainty. A speaker label is never presented as a confirmed identity unless a person has named it.

## Protocol generation

Before generation, the user should be able to see what will be used: the selected professional style, the meeting language, relevant vocabulary, and the reviewed transcript revision.

The normal interface talks about outcomes, not runtimes. “Balanced transcription” and “Formal minutes” belong in the main workflow. Exact model names, paths, context limits, and provider diagnostics belong in settings or an advanced area.

Protocol model choice is made once in Settings and then reused. The Settings view presents a short, curated catalogue with a recommendation based on the available machine and installed models. A person can choose a different model there, but the normal meeting and protocol flow does not ask for a model every time. The catalogue marks models that are installed, planned, or still awaiting German and English evaluation.

Generation is a background job. The interface should make it clear that the transcript remains available and that the user can navigate while the draft is being prepared.

## Protocol editing

The protocol editor is a writing surface with a calm margin, readable line length, native undo/redo, find, text scaling, and a clear autosave state. It should make the document feel owned by the person editing it, not by the generator that produced the first draft.

Generated text is a draft. Review applies to one exact revision. If the person edits after reviewing, the interface says **changed since review** and preserves the reviewed revision.

Export is explicit. Markdown and plain text are offered from the editor, and an existing destination is never silently overwritten.

## Background work and status

The persistent sidebar status answers “what is happening?” without pulling the user away from the current task.

It distinguishes:

- waiting in a queue;
- running with a meaningful stage;
- waiting for a user decision;
- cancelling safely;
- failed with a recovery action;
- interrupted and ready to resume;
- completed.

The status line should say what the machine is doing in the words a reader would use: “Preparing the recording”, “Writing the transcript”, “Finding what was discussed”, or “Draft ready”. It should not spend its most visible line repeating that everything is local, because that is an invariant explained in the trust surface.

Progress is bounded and throttled. A long step should include a moving detail where possible, such as a passage count or the number of subjects found. Raw subprocess output never appears in the interface.

Only one heavy local task runs at a time in the current design. If an action is refused because another task is using the heavy lane, the reason is shown where the user is already looking.

## Failure and recovery

Failure is a normal state, not an exceptional dialog. Every failed job should answer:

1. what failed;
2. what work is still safe;
3. whether retrying is sensible;
4. what the person needs to decide or change.

After a restart, partial files are not presented as finished artifacts. Interrupted jobs are named, stable revisions remain available, and working autosaves are reconciled with the last database-acknowledged state.

## Progressive disclosure

The normal path should work without technical knowledge. Advanced settings may explain:

- the selected quality and its storage cost;
- installed model and runtime details;
- provider readiness;
- storage location;
- diagnostics needed to solve a real problem.

Advanced controls should never be the only way to discover the next normal action, and they should not dominate the first-run experience.

## Accessibility and responsiveness

The complete workflow must be keyboard reachable. Focus is visible and moves to the new workspace or the first relevant control after navigation. Buttons and fields have meaningful accessible names. Text scaling, reduced motion, screen-reader announcements, and compact widths are considered part of the design, not a later compliance pass.

Ordinary navigation, selection, typing, and editing should generally feel immediate—roughly within 100 ms—even while background work is running. Large transcripts and media must not be copied into multiple unnecessary UI buffers.

## Visual acceptance questions

For each major screen, review:

- Does the current project, meeting, and stage remain clear?
- Is there one obvious next action?
- Is the screen calm without becoming empty or vague?
- Can a person recover from a failure without opening a technical panel?
- Does the light/dark version preserve hierarchy and contrast?
- Does keyboard focus remain visible and sensible?
- Does the layout remain useful when the window becomes compact?

The supplied reference images guide the visual character and spacing. They do not turn avatars, sharing controls, recording buttons, or generic dashboard elements into requirements.

## Open UX questions

- Should archive and restore be available in v0.1, and should permanent deletion remain absent?
- What minimum backup/restore actions belong in Settings?
- Should speaker labels be renamed only in the transcript, or also through a project participant list?
- How should the first-run meeting language be selected without coupling it to interface language?
- What is the most useful way to show generation evidence without making the editor feel like a diagnostic console?
