# Roadmap after v0.1

The sequence is outcome-based, not a release commitment.

LocaLog is intended for macOS, Windows, and Linux. macOS receives the first complete build because it is the current development and validation environment. Cross-platform architecture is a present requirement; packaged Windows and Linux releases follow once the central workflow and their platform adapters have been validated.

## Phase 2 — recording and richer review

**Recording is wanted earlier than this ordering implies.** It was placed after the import workflow
on the reasoning that importing proves the pipeline and recording only adds a source. That
understates it: a product people reach for at the start of a meeting is a different product from one
they remember afterwards, and every import begins with someone having already solved the recording
problem another way. Treat it as the first thing after the current work rather than as the opening
of a later phase.

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
  person can be recognised across that project's meetings and named once instead of every time. This
  is the strongest convenience available and the intended direction: solving the problem with
  technology rather than asking the user to repeat themselves. It is also the most sensitive, because
  a stored voice profile is personal data belonging to the people in the meeting. It is therefore
  bound by the handling rules in `PRODUCT.md` — local, project-scoped, visible, individually
  deletable, excluded from exports, and never a silent side effect. The remaining design work is how
  a profile is created, shown, and deleted, not whether the capability is wanted.

## Filling vocabulary without making the user type it

Vocabulary measurably improves transcription (see [MODEL_EVALUATION.md](MODEL_EVALUATION.md)), but a
user who must maintain a word list by hand will not maintain one. Directions to weigh:

- **Participants become vocabulary automatically.** A name typed into a meeting is exactly the kind
  of term transcription gets wrong, and the user has already typed it. Nothing should have to be
  entered twice.
- **Seed a project from a document it already has.** An existing protocol contains every firm,
  person and recurring term the project uses. Offering to extract candidate vocabulary from one, for
  the user to confirm, turns list-building into a single review step.
- **Vocabulary sets per professional field.** Architecture, medical, legal and similar fields each
  have their own recurring terminology, and a user could choose the set that matches their work
  rather than starting empty.

One measured constraint shapes all of these: whisper's initial prompt is capped at roughly 224
tokens, so vocabulary must be **prioritised rather than accumulated**. Evidence from the first real
meeting shows what deserves that budget. Standard German professional terminology was already
transcribed correctly without any help — Fassade, Grundriss, Treppenhaus, Erschließung, Laubengang,
Tragwerk, Barrierefreiheit and Stahlbeton all appeared correctly. Every term vocabulary actually
fixed was a **proper noun**: a company name, a participant's surname.

So the ordering should follow specificity rather than volume: this meeting's participants first, then
the project's own names and places, then genuinely ambiguous abbreviations of the field, and only
then general terminology. A field-specific set is still worth having, but it earns its place through
acronyms and unusual usage, not through vocabulary the model already knows.

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

## Later — borrowing a firm's own machine

Everything runs on the device the meeting was imported to. A firm that already owns a capable server
could lend it to the work instead: the same application, the same files, but transcription and
protocol generation carried out on a machine down the corridor rather than on a laptop.

**The architecture barely resists it.** The application already talks to its protocol provider over
HTTP on loopback, and to its runtimes as supervised processes. Pointing the provider at another host
is a small change to a setting. That is exactly why the question is not a technical one.

**What changes is the sentence the product is built on.** Today it is "meeting content never leaves
this device". On a firm's server it becomes "never leaves this organisation's network", which is a
different promise and a weaker one — true and defensible for a firm that owns its infrastructure,
and not the same thing at all. It cannot be quietly widened. Anyone whose recording is about to
travel must be told where it is going, in those words, before it goes.

**The case for it is real even so.** A meeting takes six minutes to transcribe and six to write up on
an M1; a server with room for a larger model could do better on both, and could hold a model too
large for any laptop. A firm that has already bought the machine is not asked to buy anything.

**The case against doing it soon is stronger.** The current work is making the thing fast and small
enough that ordinary hardware suffices, and every improvement there reduces the reason to reach for a
server. A capability that removes pressure to be efficient is worth deferring precisely while
efficiency is still improving.

What would have to be settled first:

- **Consent, per meeting, in plain words.** Not a setting configured once and forgotten. The people
  recorded consented to a recording, and where it is processed is part of what they consented to.
- **Proof of what the server is.** A hostname in a settings field is not evidence that a machine
  belongs to the firm rather than to a hosting provider. Without that, "local" becomes a word the
  product uses and the user cannot check.
- **What happens when it is unreachable.** Falling back to the laptop silently would be wrong in both
  directions: it hides where the work went, and it may be slower than the user was told.
- **Whether anything is left behind.** A server that keeps a copy of a transcript has turned a
  local-first product into a filing system nobody agreed to.

Related to [fitting into an organisation's own systems](#later--fitting-into-an-organisations-own-systems),
and to be decided alongside it rather than separately: both are about a firm's infrastructure, and
both trade a promise that is currently simple for one that needs explaining.

## Later — adapting a model, rather than training one

Whether LocaLog should have its own model comes up naturally, and the honest answer today is no —
but for reasons that could change, so they are worth writing down rather than settling by instinct.

**The errors measured so far are not the errors training fixes.** Names came out wrong, and
vocabulary fixed that. Output came out too short, and the model choice and the style specification
fixed that. Attribution goes to the wrong organisation, and speaker separation is what addresses
that. None of those is a gap in what the model knows; each is a gap in what it was told. A
fine-tuned model would have inherited every one of them.

**What training would genuinely buy is house style.** A firm's protocols have conventions — how a
decision is phrased, what counts as an action, how much of the discussion survives — that are
tedious to express as instructions and obvious from examples. That is exactly what fine-tuning is
good at, and it is the one argument for it worth taking seriously.

**But the training data is the most confidential data in the product.** Adaptation needs pairs of
transcript and the protocol a person actually wrote. Those pairs are precisely the material that
must never leave a device. Any plan that involves collecting them centrally to train a shared
LocaLog model contradicts the reason the product exists, and should be rejected on that basis alone
rather than on cost.

That leaves one shape that is consistent with the product: **adaptation that happens on the firm's
own machine, from the firm's own past protocols, and never leaves it.** A cloud competitor cannot
credibly offer that, because the documents would have to be uploaded. It is a real differentiator
rather than a technical vanity.

Order of attempts, cheapest first — each one may make the next unnecessary:

1. **Examples in the prompt, not weights.** Give the model two or three of the firm's own past
   protocols as in-context examples. This needs no training at all, works today, and directly tests
   whether style adaptation helps enough to be worth pursuing. The cost is context window, which
   `MODEL_EVALUATION.md` shows is the scarcest resource on the baseline machine.
2. **Retrieval over the firm's archive**, so the examples chosen resemble the meeting at hand.
3. **A local LoRA**, trained on the firm's machine, kept beside their workspace, never uploaded.
   Only worth building if 1 and 2 leave a visible gap.

Pre-training is not a candidate at any point. Full fine-tuning is not a candidate while a LoRA is
untried.

**What has to be true before step 3 is even assessed:**

- Steps 1 and 2 have been tried and measurably fall short.
- The remaining errors are style errors, not factual ones. Training makes a model sound more like
  the reference; it does not make it more accurate, and reaching for it to fix wrong facts would be
  a mistake.
- Enough pairs exist in one firm to train on — realistically dozens, not the single reference pair
  the evaluation currently rests on.
- The base model's licence permits it. This is a present-day constraint on model choice, not a
  future one: Apache 2.0 (Mistral, Qwen, Granite) keeps this door open, and more restrictive terms
  close it. It is one of the reasons the candidate list is weighted the way it is.

**Domain-specialised models** — legal, notarial, medical — are a different question and are worth
testing on their merits rather than assumed to help. LocaLog's task is not domain reasoning; it is
structuring spoken language into a document. A model tuned on legal prose may be worse at
conversational German, which is what a meeting actually contains. Test before believing.

## Explicitly uncommitted

Cloud sync, accounts, collaboration, shared workspaces, calendar integration, meeting bots, and hosted inference are not implied by this roadmap. A phone application is a stated long-term goal above, but no part of v0.1 depends on it. Each would require a separate product/privacy decision and must not become an incidental dependency.
