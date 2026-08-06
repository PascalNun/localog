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
| gemma4:12b       | 7.6 GB | 16K–49K      | failed in five attempts       | —      |
| **qwen3.5:4b**   | 3.4 GB | 40K          | **22,211 chars, full length** | 10m51s |
| **qwen3.5:4b**   | 3.4 GB | 40K + vocab  | **15,610 chars**              | 6m03s  |

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

## Open questions

- Does gemma4:12b work now that reasoning is disabled? It was abandoned before that fix.
- Is a mixture-of-experts model such as gemma4:26b (3.8 B active of 25.2 B) usable on 16 GB? Weights
  are memory-mapped, so the resident working set may be far smaller than the file.
- What does a model do on an English meeting? No English reference pair exists yet.
- Nothing here has been measured on the M1/8 GB baseline.
