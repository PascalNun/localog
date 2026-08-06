# Polishing plan

What stands between a working pipeline and something a professional would happily use every week.
The machinery now runs; this is about how it is met, named, and configured.

Ordered by how much friction each item removes, not by how hard it is. Items are not started unless
they appear in [PLAN.md](PLAN.md) as well — this document says _what good looks like_, the plan says
_when_.

## 1. The user should never configure a runtime

Today someone must find and select a whisper.cpp executable before anything works, and the diariser
needs three more paths. Nothing about that belongs in a professional tool.

- Runtimes are bundled and resolved internally; no executable setting exists anywhere.
- Models are chosen as a **quality**, never as a file. This already works for transcription and must
  work the same way for the diariser and for protocol generation.
- The application reads what the machine and the model can actually do — installed memory, the
  model's real context limit — and configures itself. It recommends; the user may override.
- Nothing in the normal path asks a question the application could answer for itself.

**Test of success:** a new user reaches a finished protocol without opening Advanced once.

## 1a. Guard rails: the application must not let a user start something that cannot finish

Everything below was learned the hard way during evaluation. A user should never discover these by
watching a progress bar for forty minutes.

- **Do not offer a model the machine cannot run.** Installed memory is readable on every target
  platform. A model whose weights plus working memory exceed what is available should be marked as
  such, not silently offered. Measured: a 12 B model at long context drove a 16 GB machine into
  swap, and an 18 GB mixture-of-experts model produced 0.6 tokens per second — a protocol would have
  taken over two hours.
- **Never request a context larger than the model supports.** The model reports its own limit;
  assuming a fixed one both wastes a capable model and silently truncates a smaller one.
- **Estimate before starting, honestly.** Transcription and generation each take minutes on a real
  meeting. A rough expectation set beforehand is worth more than a precise number invented afterwards.
- **Detect pathological slowness and say so.** A job producing tokens far below the expected rate is
  not progressing normally, and the user should be told that rather than left watching.
- **Check disk before downloading**, which the model manager already does, and check memory before
  loading.
- **Never run two heavy local jobs at once.** Measured: a transcription that took eleven minutes on a
  quiet machine took over fifty-six while a large download competed for bandwidth and memory. The job
  manager already admits one model-heavy job; downloads must respect the same rule.
- **Prefer a smaller model that finishes to a larger one that might not.** The default should be what
  works on the machine in front of the user, with larger options available and clearly labelled.

## 2. Names and language

The interface should use the words a project team uses, not the words the implementation uses.

- Review every visible string for implementation vocabulary. "Runtime", "adapter", "provider",
  "context window" and "job" are our words, not the user's.
- Stage names during processing should describe what is happening to the meeting, not which
  subprocess is running. "Reading the meeting in sections" is closer than "condensing transcript".
- Errors say what happened, what is still safe, and what to do next — in that order, without codes.
- Keep German and English equal in the interface's own language handling. No content language is
  privileged.

## 3. The shape of the pipeline

The sequence is right; the joins between stages are where it feels unfinished.

- **Import → transcribe** currently requires an explicit action after import completes. Decide
  whether that is deliberate (reviewing resolved settings first) or merely unfinished.
- **Transcribe → review** should land the user at the first thing needing attention, not at the top
  of an 800-segment transcript.
- **Review → generate** should make it obvious what the protocol will be generated _from_, and that
  editing the transcript first is worthwhile.
- **Generate → edit** should show what changed if a protocol is regenerated, rather than silently
  replacing the previous draft.
- Long operations need honest, moving progress at every stage. Transcription and generation both take
  minutes on real meetings; a still bar reads as a hang.

## 4. Waiting well

A real meeting takes about six minutes to transcribe and six more to generate. That is the single
biggest experience problem, and it cannot be engineered away entirely.

- Say what is happening and roughly how much is left, without inventing precision.
- Never block navigation or editing while work runs.
- Make the wait useful: the transcript should be readable and correctable while generation is queued.
- Consider whether a first draft can appear progressively rather than only at the end.

## 5. Speakers and vocabulary as ordinary work

Both are now real capabilities and both are currently invisible in the interface.

- Renaming a speaker should be one action that applies everywhere, from the segment being read.
- Reassigning, merging and splitting speakers belong in review, not in a settings screen.
- Vocabulary should be reachable from the meeting where a term was wrong, not only from a library.
- A term corrected in a transcript should be offerable as a vocabulary entry in one step.

## 6. Trust made visible

The product's central claim is that this material stays on the device. That should be evident without
being shouted.

- The protocol is visibly a draft until reviewed, and visibly changed after later editing.
- Where the data lives is discoverable in one step.
- Provenance — which model, which vocabulary, which style — is available without dominating the page.
- No permanent "local mode" badge. Status is for things that can change and need attention.

## 7. Accessibility and physical comfort

Not a late pass. These are a professional tool's table stakes.

- Full keyboard reachability with visible focus, including transcript correction.
- Text scaling to 200% without clipping or overlap.
- Reduced-motion respected.
- Colour never the only signal.
- Long reading sessions: line length, contrast, and the transcript's typography deserve attention
  since people will spend real time in them.

## Known rough edges, specifically

Concrete items already observed, small enough to fix once their area is touched:

- The playback **Follow** control's treatment is not accepted.
- Audio playback has never been confirmed by ear.
- Every transcript segment still shows a per-segment speaker column that was a constant until
  diarisation landed.
- The asset protocol is scoped to the whole application-data directory rather than the working-audio
  folder.
- Archive exists in the schema but is unreachable.
- A finished protocol draft can be discarded for missing one required heading.
