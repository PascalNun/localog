# Model evaluation

What has actually been run, on what hardware, and what came out. Every row is a real run against real
meeting audio, not a claim from a model card. Update it when a run finishes, including the failures —
a model that could not complete is as useful to know about as one that did.

The material itself lives in the local-only `eval/` directory and is never committed.

## Test case

An 81-minute German construction project meeting (`Projektbesprechung`), with a protocol and a set of
notes written afterwards by a person. Those documents are the reference the generated output is
compared against.

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

| Term            | Without vocabulary   | With vocabulary |
| --------------- | -------------------- | --------------- |
| NORVEK          | 0 (heard as "Norwegen") | **33**          |
| "Norwegen" (wrong) | 20                   | **0**           |
| Mustermann      | 0 ("Musterman")      | **2**           |
| Beispielhuber   | 0 ("Beispiel-Huber")    | **1**           |
| Muster-Weber     | 0 ("Musterweber")      | **2**           |

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
| **qwen3.5:4b**   | 3.4 GB | 40K          | **22,211 chars, full length** | 10m51s |
| **qwen3.5:4b**   | 3.4 GB | 40K + vocab  | **15,610 chars**              | 6m03s  |

Term accuracy in the generated protocol, with and without vocabulary:

| Term in output  | Without vocabulary | With vocabulary |
| --------------- | ------------------ | --------------- |
| NORVEK          | 0                  | **11**          |
| "Norwegen" (wrong) | 15                 | **0**           |
| "Musterman"     | 3                  | 0               |
| "Beispiel-Huber"   | 2                  | 0               |
| "Musterweber"     | 2                  | 0               |

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
- **Proper nouns are the whole win _in this domain_.** Standard German professional terminology transcribed correctly
  with no help at all: Fassade (43 occurrences), Grundriss (28), Treppenhaus (12), Erschließung (10),
  plus Laubengang, Wohnungsmix, Tragwerk, Bauphysik, Barrierefreiheit and Stahlbeton. Every term
  vocabulary actually corrected was a company name or a surname. Since the prompt is capped at about
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

- **gemma4:12b is not viable on this hardware.** Retried after the reasoning, budget and timeout
  defects were all fixed, so those confounds are gone. It still failed, with a single condensing pass
  exceeding thirty minutes before returning anything, against six minutes for a complete protocol from
  qwen3.5:4b. The earlier memory observation stands: a 12B model at long context does not have room
  to work on 16 GB. This says nothing about how well it writes, only that it cannot finish here.
- **The mixture-of-experts question is still untested.** gemma4:26b holds 25.2 B parameters but
  activates only 3.8 B per token, and Ollama memory-maps weights, so the resident working set may be
  far smaller than the 18 GB file. That would make it behave quite unlike a dense model of the same
  size. The download was started and then stopped because it was competing with a running test for
  bandwidth; most of the data is on disk and Ollama resumes, so this is a matter of finishing rather
  than starting over.
- What does a model do on an English meeting? No English reference pair exists yet.
- Nothing here has been measured on the M1/8 GB baseline.
