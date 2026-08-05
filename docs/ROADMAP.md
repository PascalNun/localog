# Roadmap after v0.1

The sequence is outcome-based, not a release commitment.

LocaLog is intended for macOS, Windows, and Linux. macOS receives the first complete build because it is the current development and validation environment. Cross-platform architecture is a present requirement; packaged Windows and Linux releases follow once the central workflow and their platform adapters have been validated.

## Phase 2 — recording and richer review

- Microphone recording with interruption/recovery handling
- macOS system-audio capture, then platform equivalents
- Multiple synchronized recording sources per meeting
- Richer speaker tools beyond the v0.1 diarisation accepted in D-029
- Audio waveform and faster timestamp navigation
- Protocol-to-transcript/audio traceability
- DOCX export and basic templates

## Making speakers easy to name

Diarisation separates voices; it cannot know who they are. Naming is therefore a user task, and the
goal is to make it cost almost nothing. Recorded here as directions to weigh, not as a chosen design.

A useful reframe first: attribution matters unevenly. Nobody needs every sentence of a discussion
attributed, but a decision or an action is close to useless without a name. Naming can therefore be
lazy and on demand — name the two or three people who own something, ignore the rest — rather than a
step that must be completed before the protocol can be written.

Approaches, roughly cheapest first:

- **Pick from the meeting's participants.** Participants already exist in the product model and are
  not built yet. With a participant list, naming a speaker is choosing from a short list rather than
  typing. `UX.md` already asks whether speaker mapping may create participants from the transcript
  workspace; that question becomes worth answering here.
- **Inherit recurring participants from the project.** Regular project meetings involve mostly the
  same people, so the list should arrive pre-filled rather than being retyped per meeting.
- **Fast assignment in review.** Keyboard-first assignment, assigning from a segment the user is
  already reading, and renaming everywhere at once. The existing side-panel rename is the start of
  this.
- **Suggest names from what was said.** Meetings contain their own evidence: self-introduction, and
  direct address such as “Danke, Anna”. A local model could propose a mapping. This must only ever
  propose: a wrong name silently attached to a decision is precisely the kind of invented authority
  the product refuses. Suggestions stay provisional and require confirmation.
- **Remember a voice within a project.** Diarisation already produces speaker embeddings, so the same
  person could be recognised across that project's meetings and named once instead of every time.
  This is the strongest convenience and the most sensitive: a stored voice profile is biometric data.
  It would require an explicit product and privacy decision, must be local, per project, visible, and
  deletable, and must be opt-in rather than a silent side effect of transcription. It is recorded as a
  possibility, not an intention.

## Phase 3 — portability and libraries

- Windows and Linux packaged builds
- Provider/model management that remains optional and local
- Export template library and PDF conversion
- Search across local projects and meetings
- Polished project archive/import bundles and portability beyond the basic backup/restore considered during v0.1 hardening
- Richer vocabulary import, suggestions, and conflict resolution
- Structured task/decision views derived from protocols, always reviewable

## Later — the same workflow on a phone

A long-term goal, deliberately not being investigated yet.

Recording usually happens on a phone, so the current split — capture on the phone, process on a
laptop — adds a transfer step to every meeting. Doing the whole workflow on one device would remove
it. The smaller transcription models are the reason this is plausible rather than fanciful: the
quality presets already include models small enough that on-device transcription is a reasonable
question to ask, even if protocol generation stays on a desktop at first.

Nothing about this is committed, and it must not become an incidental dependency of v0.1. It is
recorded here so that architecture choices which would make it impossible are noticed early: the
product core stays independent of desktop-only assumptions, and platform behaviour stays behind the
adapters described in the technical architecture.

## Later — a dedicated capture device

A separate long-term idea, recorded so it is not forgotten. Also not being investigated yet.

A small purpose-built recorder — closer to a dictation device than a computer — would capture meeting
audio without a laptop open on the table, which is often the socially easier thing in a client or
site meeting. It would only capture: transcription and protocol generation stay on the computer, so
the device needs no model, no accelerator, and no network.

This is a hardware product with its own economics, firmware, certification, and support burden, which
makes it a much larger commitment than a phone application. It is listed as a possibility, not a
plan. What it does imply for the software is modest and already true: imported audio remains the
first-class input path, and a meeting can own more than one source.

## Later — fitting into an organisation's own systems

A long-term goal for larger teams, recorded now and deliberately not designed yet.

Firms already have somewhere that project documents belong: a self-hosted file service such as
Nextcloud, a company drive, or a collaboration suite. A protocol that has to be exported and filed by
hand is a protocol that gets filed inconsistently. The goal is that a LocaLog project can be linked to
where the organisation already keeps that project's material, so a reviewed protocol lands in the
right place without manual copying.

This does not contradict local-first, and the distinction matters: the target is infrastructure the
organisation controls, which is exactly where this material is supposed to live. It is not a LocaLog
cloud, not a third-party inference service, and not a hosted account system. Sending meeting content
to a service the organisation does not control remains outside the product.

Requirements before any of this is designed: it stays optional and off by default, every destination
is configured explicitly by the user or their administrator, nothing is uploaded without an explicit
action, and the local workflow remains complete on its own. Whether LocaLog offers an outbound
integration, an API surface for others to call, or both is undecided.

## Phase 4 — advanced local workflows

- Optional live transcription where hardware permits
- User-approved local automation and batch processing
- Pluggable providers/runtimes with a stable extension contract
- More precise source citations and confidence/review tooling
- Optional organisation policies for managed deployments

## Explicitly uncommitted

Cloud sync, accounts, collaboration, shared workspaces, calendar integration, meeting bots, and hosted inference are not implied by this roadmap. A phone application is a stated long-term goal above, but no part of v0.1 depends on it. Each would require a separate product/privacy decision and must not become an incidental dependency.
