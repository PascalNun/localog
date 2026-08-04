# Transcription and speaker experience — design proposal

Status: **Accepted direction (2026-08-04).** Its decisions are recorded in [DECISIONS.md](DECISIONS.md) (D-028, D-029) and its rules folded into [PRODUCT.md](PRODUCT.md), [MVP.md](MVP.md), and [UX.md](UX.md). Implementation remains gated on the validation spikes in the Validation plan below. This document redesigns how a user reaches a transcript in LocaLog: it replaces the developer-facing “point at a whisper.cpp executable and model file” settings with a seamless model choice, and introduces automatic speaker separation.

It exists because the current path-configuration flow is validation scaffolding, not the product. A professional user should choose a quality, not operate a runtime.

## What changes

1. **The whisper.cpp binary is bundled and invisible.** No “executable path” setting exists. Ever.
2. **Models are chosen, not located.** The user picks **Fast / Balanced / Accurate**; the exact model (Tiny/Base/Medium) is an Advanced detail. If the chosen model is not on the machine, the app downloads it with explicit consent and honest progress.
3. **Speakers are separated automatically.** Transcripts arrive with distinct speakers (`Speaker 1`, `Speaker 2`, …) that the user can rename and correct. Automatic diarisation becomes a v0.1 capability, not a deferred one.

## Decisions this proposal changes

These are deliberate reversals of currently written decisions. They must be recorded in `DECISIONS.md` if accepted.

| Ref                                                                     | Was                                                        | Becomes                                                                                                                                                             |
| ----------------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| D-014 / `UX.md` exclusion “automatic model downloads”                   | Discover installed/user-provided models only; no downloads | Consent-gated, on-demand download of **app-managed, checksummed** transcription/diarisation models. Still no silent/background acquisition.                         |
| `UX.md` “automatic diarisation is not required” + MVP/PRODUCT non-goals | Manual speaker labels only; `Speaker 1` constant           | Automatic speaker diarisation is a v0.1 capability via an ONNX (non-Python) diariser. Speakers remain **editable and provisional**, never authoritative identities. |
| D-022 runtime distribution                                              | User-provided whisper.cpp binary + model                   | whisper.cpp binary **bundled as a signed sidecar**; models downloaded on demand.                                                                                    |

What does **not** change: no model-manager appears in the sidebar (`UX.md:71` stands — management lives inside Settings); presets-first with internals hidden (`UX.md:162, 349` already agree); the defaults chain `global < project < meeting` (D-010); “never invent progress / never present partial work as complete”.

## Principles carried over

- The user chooses an **outcome** (quality, speakers), never a runtime.
- Downloads are **explicit and consented**, show real size and progress, and are cancellable and resumable-by-restart. Nothing downloads silently.
- Every model file is **verified against a known checksum** before it is trusted, and lives in app-managed storage with recorded provenance.
- Diarisation is **honest**: it is imperfect, so its output is presented as an editable starting point, with no avatars or confident identity claims.
- Heavy work stays off the UI thread; on the 8 GB baseline, transcription and diarisation run **sequentially**, never concurrently.

## Transcription runtime: bundled, invisible

- whisper.cpp ships as a **Tauri sidecar** (MIT-licensed; it resolves its own libraries — verified). The application resolves the binary internally; there is no executable path in Settings.
- FFmpeg/FFprobe follow the same bundling track (a redistributable, audio-only, checksummed build — see `DECISIONS.md` licensing note). Until that build exists, dev uses the installed tools.
- The Transcription settings section shows **capability**, not paths: which quality presets are ready, which need a download, and the active default.

## Model management: choose a quality, download on demand

### Preset ↔ model mapping

| Preset                 | Model (Advanced) | Approx. size | Character                      |
| ---------------------- | ---------------- | ------------ | ------------------------------ |
| **Fast**               | `tiny` / `base`  | ~75–150 MB   | Quick drafts, low memory       |
| **Balanced** (default) | `small` / `base` | ~150–460 MB  | Everyday meetings              |
| **Accurate**           | `medium`         | ~1.5 GB      | Best quality; heaviest on 8 GB |

Exact assignments are tuned after the on-baseline measurement (see Validation). The user sees the preset; the model name/size is visible only under an **Advanced** disclosure, per `UX.md`.

### Where the choice lives

- **Settings → Transcription** is the management home: presets with a ready/needs-download status, a **Download** action per model, downloaded size, and a remove-to-reclaim-space action. This is management inside Settings, not sidebar navigation.
- **Selection follows the defaults chain** (D-010): a global default preset, overridable per project, overridable per meeting. Which models are _present on the machine_ is a global fact; which preset a meeting _uses_ resolves through the chain and is snapshotted into the job at start.

### Download flow and states

Triggered either proactively in Settings, or inline the first time a meeting needs a model it does not have (“Balanced needs a 148 MB model — Download and transcribe”). Never automatic.

| State                     | Presentation                                                      | Action                                     |
| ------------------------- | ----------------------------------------------------------------- | ------------------------------------------ |
| Not present               | Preset marked “Download (148 MB)”                                 | Download (explicit)                        |
| Downloading               | Determinate byte progress; cancellable                            | Cancel                                     |
| Verifying                 | “Checking download…”                                              | —                                          |
| Ready                     | “Ready”                                                           | Use / Remove                               |
| Verify failed             | “The download was incomplete or corrupt.” Partial file discarded. | Retry                                      |
| Download failed / offline | “No connection to the model source.”                              | Retry; work offline with any present model |
| Disk full                 | Checked before download; “Not enough space (needs 148 MB).”       | Free space / choose smaller preset         |

Models download to `<app-data>/models/` (the storage layout already reserves this), are checksum-verified before first use, and record provenance (source URL, digest, size, app version). The existing job envelope, staged-write, and restart-reconciliation rules apply — an interrupted download never appears as a ready model.

## Speaker diarisation

### Approach: ONNX, not Python

Diarisation runs through **sherpa-onnx** (ONNX Runtime, C++), **not** pyannote/whisperX. LocaLog already rejected Python/PyTorch Whisper as too heavy (`DECISIONS.md` media spike); a Python diarisation stack would reintroduce exactly that footprint. sherpa-onnx keeps diarisation bundleable and consistent with the lightweight, local-first ethos. Its models (a segmentation model and a speaker-embedding model) download on demand like transcription models.

**Honest cost, stated up front:** diarisation adds two more model files (order of tens–low-hundreds of MB), additional RAM, and extra processing time. On the 8 GB baseline it must run **after** transcription, not alongside it. This is the price of the “speakers in v0.1” decision and is accepted deliberately.

### Pipeline

Diarisation is a stage of the transcription job, not a new user action:

```text
normalise → transcribe (whisper) → diarise (sherpa-onnx) → assign each segment its dominant speaker → commit
```

The `Transcriber` port is unchanged; diarisation is a separate internal capability whose result populates the existing per-segment `speaker` field. No schema change — the seam was already there (`speaker` per segment; manual speaker tools in review). If diarisation fails or its model is absent, transcription still commits with a single `Speaker 1` and a quiet “speakers not separated” note; a transcript is never lost because diarisation failed.

### Review experience

Transcript review gains first-class speaker tools:

- **Rename a speaker everywhere** (`Speaker 2` → “Anna Berger”) in one action.
- **Reassign a segment** to a different speaker when diarisation misattributed it.
- **Merge** two labels the model split, or **split** one it wrongly joined.
- Speakers are visibly **auto-detected and editable** — a quiet affordance, never a confident avatar or photo. Colour is a secondary cue only, never the sole signal.
- Optional participant mapping ties a detected speaker to a named meeting participant (relates to `UX.md` open question 1).

### Required spike before commit

Consistent with the project’s spike discipline, diarisation is **proposed, not accepted**, until an isolated spike validates: sherpa-onnx model contract and licensing; diarisation quality on synthetic multi-speaker German/English audio; whisper↔diariser segment alignment; memory and time on the **M1/8 GB** baseline; and cancellation. No diarisation code enters production before that keep/change record — exactly as whisper.cpp was validated.

## Failure and edge states (designed, not incidental)

- **Offline, model absent:** transcription is blocked with a clear reason and a retry; any already-present model still works. No silent failure.
- **Partial/corrupt download:** discarded, never trusted; retry offered.
- **Diariser unavailable/failed:** transcript still commits with a single speaker and an honest note.
- **Low disk:** checked before any download starts.
- **Model removed while selected:** the meeting’s resolved preset shows “needs download” again; prior transcripts are untouched.

## Open questions

1. Exact preset→model assignments, pending the on-baseline measurement.
2. Bundle _one_ smallest model after all, so the very first transcription works fully offline? (You chose pure download-on-demand; revisit only if first-run offline matters.)
3. Model source/CDN and its integrity/pinning story (Hugging Face vs a LocaLog-hosted mirror), and the licence review for redistributing model files.
4. Should diarisation be defeatable per meeting (a “single speaker / don’t separate” option) for one-on-one recordings where it only adds cost?
5. Where exactly the inline “needs a model” prompt appears in the source-ready → transcribe moment.

## Validation plan

1. **Model-download boundary spike:** consented download → checksum verify → app-managed storage → restart reconciliation, with offline/partial/disk-full states. Reuses the existing job + staged-write discipline.
2. **Diarisation spike:** sherpa-onnx contract, quality, alignment, and M1/8 GB memory/time; keep/change record in `DECISIONS.md`.
3. **On-baseline preset measurement:** finalise the preset→model table on real 8 GB hardware.
4. Only then: fold accepted rules into `UX.md`, record reversals in `DECISIONS.md`, and implement behind the existing ports.
