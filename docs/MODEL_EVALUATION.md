# Model evaluation

What has actually been run, on what hardware, and what came out. Every row is a real run against real
meeting audio, not a claim from a model card. Update it when a run finishes, including the failures —
a model that could not complete is as useful to know about as one that did.

The material itself lives in the local-only `eval/` directory and is never committed.

## Test case

An 81-minute German construction project meeting, with a protocol and a set of notes written
afterwards by a person. Those documents are the reference the generated output is compared against.

Nothing here identifies the meeting, its participants or the client. Terms are referred to by their
role — "client firm", "participant surname" — because the measurements are what this file is for and
the actual strings are exactly what must not leave the machine.

| Property                 | Value                                      |
| ------------------------ | ------------------------------------------ |
| Audio                    | 81 min (4,909 s), mono                     |
| Transcript               | 788 segments, ~74,000 characters           |
| Estimated tokens         | ~24,500                                    |
| Human reference protocol | 18,212 characters, 30 headings, 17 actions |
| Human reference notes    | 1,695 words, denser and less formal        |

## Hardware

All runs on the development machine: **Apple M1 Pro, 16 GB**. This is _not_ the M1/8 GB baseline in
D-015, so timings here are optimistic and memory findings are conservative — anything that strains
16 GB cannot run on 8 GB.

## Transcription

| Model  | Size   | Time for 81 min | Notes                              |
| ------ | ------ | --------------- | ---------------------------------- |
| medium | 1.5 GB | ~6 min          | Used for all generation runs below |
| base   | 148 MB | ~1.5 min        | Used for contract validation only  |

### Vocabulary makes a measurable difference

whisper.cpp accepts an initial prompt (`--prompt`), and `--carry-initial-prompt` applies it to every
window rather than only the first 30 seconds. Supplying the project's own terminology — firms, people,
recurring subjects — corrected terms that were otherwise wrong throughout:

| Term                             | Without vocabulary              | With vocabulary |
| -------------------------------- | ------------------------------- | --------------- |
| Client firm (an unusual acronym) | 0 — heard as a well-known brand | **33**          |
| That wrong brand name            | 20                              | **0**           |
| Participant surname, Slavic      | 0 — heard as a different name   | **2**           |
| Participant surname, Greek       | 0 — split into two words        | **1**           |
| Participant surname, hyphenated  | 0 — hyphen dropped              | **2**           |

This is the evidence behind treating vocabulary as a real feature rather than an intention. Note the
cap: the initial prompt is limited to roughly 224 tokens, so a project's vocabulary has to be
prioritised rather than dumped in.

## Generation

| Model            | Size   | Context used | Result                        | Time   |
| ---------------- | ------ | ------------ | ----------------------------- | ------ |
| qwen2.5-coder:7b | 4.7 GB | 8K           | 3,698 chars — quarter length  | 9m53s  |
| qwen2.5-coder:7b | 4.7 GB | 16K          | 4,210 chars — still short     | 6m23s  |
| qwen2.5-coder:7b | 4.7 GB | 32K          | rejected: missing section     | —      |
| gemma2:9b        | 5.4 GB | 8K           | failed — window too small     | —      |
| gemma4:12b       | 7.6 GB | 16K–49K      | failed, seven attempts        | —      |
| gemma4:12b       | 7.6 GB | 16K + vocab  | failed: >30 min for one pass  | 34m+   |
| gemma4:12b       | 7.6 GB | 40K + vocab  | 12,492 chars — quiet machine  | 15m56s |
| **qwen3.5:4b**   | 3.4 GB | 40K          | **22,211 chars, full length** | 10m51s |
| **qwen3.5:4b**   | 3.4 GB | 40K + vocab  | **15,610 chars**              | 6m03s  |

Term accuracy in the generated protocol, with and without vocabulary:

| Term in output            | Without vocabulary | With vocabulary |
| ------------------------- | ------------------ | --------------- |
| Client firm, correct      | 0                  | **11**          |
| Wrong brand name          | 15                 | **0**           |
| Wrong surname, Slavic     | 3                  | 0               |
| Wrong surname, Greek      | 2                  | 0               |
| Wrong surname, hyphenated | 2                  | 0               |

### What each result taught

- **A coding model writes short.** qwen2.5-coder produced a correct skeleton at a quarter of the
  reference length regardless of how much context it was given. Widening the window fixed the
  mechanics, not the brevity.
- **Context window is a hard constraint, not a preference.** gemma2:9b has an 8,192-token limit. A
  reference-length protocol is about 6,000 tokens, leaving under 2,000 for the source it must be
  written from. No amount of prompting can fix that.
- **Model size must be judged against the machine, not the disk.** gemma4:12b at long context drove
  the 16 GB machine to 6 GB of swap. Some of those failures were later traced to a different cause
  (see below), so this is a contributing factor rather than a proven verdict.
- **Small models with very large windows changed the picture.** qwen3.5:4b is 3.4 GB with a 256K
  context: it fits an 8 GB machine _and_ holds an entire meeting in one pass, which removes the
  compression that shortened earlier output.
- **Proper nouns are the whole win _in this domain_.** Standard German building terminology — the
  vocabulary of any textbook on the subject — transcribed correctly with no help at all: the most
  common such term appeared 43 times and was never wrong, and nine others behaved the same way. Every
  term vocabulary actually corrected was a company name or a surname. Since the prompt is capped at about
  224 tokens, a vocabulary of general field terminology would spend that budget on words the model
  already knows.

  This is one meeting in one field, and the rule should not be treated as general. German
  construction terminology is common enough in training data to be transcribed correctly unaided;
  other professions may not be. Medicine has drug names and abbreviations that are both unusual and
  easily confused, and law gives ordinary words specific meanings. The ordering rule needs testing in
  a second field before it is relied on, and the product should let a field's vocabulary set decide
  rather than hard-coding an assumption drawn from architecture.

- **Vocabulary shortens as well as corrects.** The vocabulary run produced 15,610 characters against
  22,211 without, closer to the reference. A cleaner transcript appears to reduce the model's
  need to hedge and restate.

## Bugs this exercise found

Nine defects, none of which unit tests could have caught, all only visible against a real recording:

1. A 30-second global HTTP deadline applied to generation, which cannot finish in 30 seconds.
2. Notes merged in fixed pairs without checking the pair fits the model's window.
3. A fixed output cap that cut answers off mid-JSON.
4. Sections sized against the input window while ignoring the answer budget.
5. A 120-second first-byte deadline, shorter than large-prompt processing takes.
6. Truncation reported as a JSON parse error rather than as truncation.
7. A 128 KB response cap, smaller than a full protocol.
8. Intermediate notes bounded by the _protocol's_ length preference while being told to be exhaustive.
9. **Reasoning models answer in a separate `thinking` field.** Both qwen3.5 and gemma4 returned an
   empty response and put their JSON there. This probably explains several failures first attributed
   to memory pressure, and it was fixed only after gemma4 had already been set aside.

## Marking the words the model was unsure of

whisper's `--output-json-full` reports a probability for every token, which is the raw material for
telling a reader where the transcript is weak. Turning that into a useful question took two attempts.

**The obvious rule flags the wrong words.** Marking any token below 0.40 flagged nine words across
four minutes of the real German meeting: `wäre`, `oder`, `Hier`, `acht`, `Nee`, `da`. These are
ordinary function words, and a low score on them means the model was choosing between two harmless
alternatives. A review pass that asks about those teaches people to ignore it.

**Rarity separates the signal.** A common word is a single token; a rare one — a company, a surname,
a technical compound — has to be assembled from several. Requiring at least two non-punctuation
pieces, and ignoring punctuation scores entirely, changes what surfaces:

| Rule                            | Words flagged in 35 segments | What they were                        |
| ------------------------------- | ---------------------------- | ------------------------------------- |
| `p < 0.40`, any token           | 7                            | Mostly function words                 |
| **`p < 0.40`, ≥ 2 word pieces** | **2**                        | **the client firm, and one compound** |
| `p < 0.50`, ≥ 2 word pieces     | 5                            | Adds correctly-heard compounds        |
| `p < 0.60`, ≥ 2 word pieces     | 15                           | Mostly correct German compounds       |

The lowest-scoring word in the whole excerpt, at **0.138**, was the client's company name misheard.
That is exactly the word worth asking a reader about, and it is the same class of error the
vocabulary feature exists to prevent. Raising the threshold to 0.50 begins flagging ordinary German
compounds — the kind any building text contains — all of which were transcribed correctly.

Also worth noting: punctuation carries its own probability, and a doubtful comma was inflating the
piece count of ordinary words such as `da.` and `wäre.`. Excluding punctuation from both the score
and the count is what makes the rule hold.

The shipped parser was then run over that same real whisper output and produced exactly this:

```
2 of 35 segments flagged
    102.0s  [<client firm, misheard>]
    204.5s  [<one ordinary German compound>]
```

That check is kept as an ignored test, since it needs meeting audio that never lives in this
repository:

```
LOCALOG_EVAL_WHISPER_JSON=/path/to/out.json \
  cargo test --lib uncertain_words_against_real_output -- --ignored --nocapture
```

Measured on the four-minute excerpt beginning at 10:00. Single excerpt, one language, one recording:
the threshold is evidence-based rather than proven, and should be revisited against English audio.

## Diarisation

The spike used synthetic speech and was far too easy. On the real meeting it fails.

| Case                                  | Speakers found | Speed          |
| ------------------------------------- | -------------- | -------------- |
| Spike, 23 s, three synthetic voices   | 3 of 3         | 3.2x real time |
| Real meeting, 81 min, about 11 people | **86**         | **0.51x**      |
| Real meeting, 10 min excerpt          | 12             | 0.51x          |

Two separate problems.

**Over-clustering grows with length.** Ten minutes of the same recording yields twelve speakers;
the full eighty-one minutes yields eighty-six. The embedding of one person drifts across a long
videoconference — different microphones, connection quality, and compression — so the clustering
splits a single voice into many. One cluster holds 58% of the speech while eighty-five share a long
tail, most with a single short turn.

Directions worth trying, cheapest first:

- Supply the expected speaker count when it is known. Participants are already a planned meeting
  field, so this is free information the product will have anyway.
- Raise the clustering threshold so voices merge more readily, at the risk of merging two people.
- Diarise in windows and match clusters across window boundaries, which addresses drift directly but
  is the most work.

**Speed responds to configuration.** Both networks default to `num_threads = 1`, and a CoreML
provider is available. Enabling six threads and CoreML took the ten-minute excerpt from 308.9 s to
188.6 s, a 1.64x improvement for no cost, which would bring the full meeting from about 45 minutes to
about 26. Diarisation nonetheless remains the slowest stage of the pipeline by a wide margin, against
roughly six minutes for transcription.

**Requesting a known speaker count works.** With eleven clusters requested the excerpt produced seven
speakers rather than the eighty-six seen with a threshold alone. Participants are already a planned
meeting field, so the product will have that number without asking for it twice.

**The two passes are partly redundant.** Transcription and diarisation each segment the audio
independently, yet whisper already reports where speech is and where segments begin and end.
Extracting embeddings only for whisper's existing segments would skip the second segmentation pass,
skip silence, and remove the alignment problem entirely, since the boundaries would match by
construction. This build ships no standalone embedding tool, so it would require the sherpa-onnx C
API through a binding — trading the supervised-process boundary for a linked library. Recorded as an
option rather than a plan.

### Diariser models

Both are fetched as bare files, so no archive handling is needed and no compression dependency was
added. Checksums were verified against copies obtained independently through the project's own
release archive.

| Model                           | Size    | Purpose                                     |
| ------------------------------- | ------- | ------------------------------------------- |
| `pyannote-segmentation-3.0`     | 5.99 MB | Finds where speech and voices change        |
| `3D-Speaker eres2net` embedding | 39.6 MB | Describes each voice so they can be grouped |

## What a model actually costs in memory

Measured on the development machine with Ollama 0.30.10, by loading a model and reading the
`llama-server` runner's resident set. This matters because LocaLog's baseline is an 8 GB Mac, and
because the number that decides viability is not the file size.

**Context is not free.** The same 3.4 GB model spans 3.6 GB to 7.3 GB resident depending only on how
much context it is given:

| `num_ctx` | qwen3.5:4b resident | Note                                   |
| --------- | ------------------- | -------------------------------------- |
| 4,096     | 3.61 GB             | Close to the file size                 |
| 16,384    | 3.89 GB             |                                        |
| 40,960    | **4.70 GB**         | The context LocaLog currently asks for |
| 131,072   | 7.33 GB             | Would not fit an 8 GB machine at all   |

That works out at roughly **30 KB of KV cache per token**. An 81-minute meeting is about 24,500
tokens of transcript, so the transcript alone accounts for around 0.7 GB before the model has
written anything. On an 8 GB machine, weights plus context plus the operating system plus LocaLog
itself is the real budget, and the context window is the part that is easiest to get wrong.

The practical consequence: **the model tier and the context length have to be chosen together.**
Recommending a model on file size alone would understate what it needs by more than a gigabyte.

### gemma4:12b, measured on a quiet machine

The earlier verdict was withdrawn as premature because a large download had been competing for
bandwidth and memory. Retested with nothing else running:

| `num_ctx` | Resident | Processor | First response to a short prompt |
| --------- | -------- | --------- | -------------------------------- |
| 16,384    | 7.96 GB  | 100% GPU  | 7 s                              |
| 40,960    | 8.26 GB  | 100% GPU  | 5 s                              |

So the model loads fully onto the GPU and answers a short prompt promptly. This does **not** yet
disprove the thirty-minute failure, which happened on a 24,500-token prompt where the cost is in
processing the input rather than loading the weights — that run is a separate measurement.

What it does establish firmly: at 8.26 GB resident, **gemma4:12b cannot run on the 8 GB baseline at
all**, whatever its quality. It can only ever be an option for larger machines.

## Candidates not yet tested

Recorded so they are not lost between sessions. Sizes and context from ollama.com; nothing here has
been run.

| Candidate         | Size   | Context | Why it is on the list                                                   |
| ----------------- | ------ | ------- | ----------------------------------------------------------------------- |
| **mistral-nemo**  | ~7 GB  | 128K    | European lab, Apache 2.0, historically strong in German and French      |
| mistral-small3.2  | ~14 GB | 128K    | Larger Mistral; likely too big for 16 GB on current evidence            |
| **gemma4:e2b**    | 7.2 GB | 128K    | 2.3 B _effective_ parameters, so far cheaper to compute than gemma4:12b |
| gemma4:e4b        | 9.6 GB | 128K    | 4.5 B effective                                                         |
| **granite4.1:3b** | 2.1 GB | 128K    | IBM, Apache 2.0, German among explicitly listed languages               |
| granite4.1:8b     | 5.3 GB | 128K    | Same family, more capacity                                              |

**Mistral matters for a reason beyond quality.** LocaLog's users are European professional firms, and
a European model developed under EU rules is easier to justify to a client asking where the technology
came from — even though nothing leaves the device, so no transfer question arises technically. This is
a procurement and trust argument rather than a privacy one, and it is worth testing on those grounds
alone. Its Apache 2.0 licence also matters if a model is ever fine-tuned, which Gemma's own terms
would complicate.

## Open questions

- ~~**gemma4:12b is unproven, not disproven.**~~ **Settled.** Run on a genuinely quiet machine at
  40K context it completed in **15m56s and produced 12,492 characters** — so the earlier failures
  really were contention, and withdrawing that verdict was right. But the clean result does not
  favour it: against qwen3.5:4b's 15,610 characters in 6m03s it is **two and a half times slower and
  a fifth shorter**, and at 8.26 GB resident it cannot run on the 8 GB baseline at all. It is not a
  candidate. This is a better outcome than the original guess, because it is now known rather than
  assumed.
- **The mixture-of-experts question is still untested.** gemma4:26b holds 25.2 B parameters but
  activates only 3.8 B per token, and Ollama memory-maps weights, so the resident working set may be
  far smaller than the 18 GB file. That would make it behave quite unlike a dense model of the same
  size. The download was started and then stopped because it was competing with a running test for
  bandwidth; most of the data is on disk and Ollama resumes, so this is a matter of finishing rather
  than starting over.
- What does a model do on an English meeting? No English reference pair exists yet.
- Nothing here has been measured on the M1/8 GB baseline.
