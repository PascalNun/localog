# Product quality notes

This document collects the small decisions that make the difference between a working pipeline and a tool someone would trust every week.

It is not a second roadmap. Work belongs in [PLAN.md](PLAN.md); this document explains the quality bar behind that work.

## Do not make the user configure the machine

The normal path should ask for a quality outcome, not an executable path or a model name. The application should measure what the machine can reasonably handle, recommend a choice, explain the disk and time cost, and keep advanced details available without putting them in front of everyone.

Model choice is a persistent Settings preference, not a per-protocol interruption. A curated catalogue can show a recommended model and alternatives for stronger machines, while the regular workflow stays focused on the meeting and its protocol.

Nothing is downloaded without consent. A model or runtime is never fetched as a hidden side effect of importing a meeting.

Until runtimes are bundled, the current development build still exposes configuration controls. Those controls are scaffolding, not the desired public experience.

## The machine should not be allowed to surprise the user

Before starting expensive work, LocaLog should eventually:

- avoid offering models that cannot fit the available memory;
- use the model’s real context limit rather than assuming one;
- check disk space before downloading;
- give a rough expectation of time and storage cost;
- report pathological slowness instead of looking frozen;
- keep one heavy local job active at a time;
- prefer a smaller model that finishes to a larger model that swaps indefinitely.

The current single heavy-work lane is a useful safety boundary. Memory recommendations and first-run setup are still planned.

## One place says what is happening

The sidebar status line should remain the single, persistent answer to “what is happening right now?” It should include downloads and other heavy work, not only meeting jobs.

The wording should describe the task in the user’s language. “Finding what was discussed” is better than “running topic extraction”. A long stage should say where it has reached, not merely repeat its name.

Progress should teach the user something useful where possible: a passage count, a completed stage, or a decision that needs attention. It should never mirror raw process output.

## The pipeline should feel joined up

- Import should make the next transcription step clear.
- Transcription should land the user at the first useful review point.
- Review should explain what generation will use.
- Generation should not replace a previous draft without preserving it.
- Editing should feel independent of the model that made the first draft.
- Export should be explicit and collision-safe.

The joins between stages matter more than adding another isolated capability.

## Waiting well

A real meeting can take minutes to transcribe and generate. The interface cannot remove that time, but it can make the wait feel intentional:

- say what is happening;
- show honest movement;
- keep navigation available;
- preserve the transcript and previous protocol while generation runs;
- make cancellation and retry understandable;
- show useful intermediate work when doing so helps a person notice an omission early.

## Language quality

The interface speaks to the person, not to the implementation. Prefer “Saving the transcript” to “committing the transcript revision”. Prefer “The selected model is not ready” to a raw provider error code.

Technical provenance remains available where it helps trust, but it does not become the headline of ordinary workflow copy.

## What to keep resisting

- generic dashboard layouts;
- permanent chat fields;
- bright default-primary buttons;
- decorative AI imagery and “magic” animations;
- hidden essential actions;
- broad settings before the normal workflow works;
- code that is cleverer than the problem requires;
- comments that describe every line instead of explaining the important reason behind a boundary.
