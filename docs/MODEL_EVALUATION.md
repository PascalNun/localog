# Model evaluation

This is the evidence behind the current protocol-generation direction. It describes what was measured, what the measurements mean, and where they stop being reliable.

## The reference meeting

The main evaluation used an 81-minute German construction-project meeting:

- about 4,909 seconds of audio;
- 788 transcript segments in the original evaluation;
- roughly 74,000 transcript characters;
- a human protocol of about 18,000 characters;
- 30 headings and 17 recorded actions in the human reference.

The recording and reference remain local-only. This document records measurements, not names or meeting content.

All runs described here used an Apple M1 Pro with 16 GB RAM. That machine is stronger than the approved M1/8 GB baseline.

## Transcription

The measured whisper models behaved roughly as follows:

| Model  | Approximate time for the meeting |
| ------ | -------------------------------: |
| base   |                      1.5 minutes |
| medium |                        6 minutes |

Vocabulary made the important difference in this domain. Proper names and unusual project terms were the words most likely to be wrong, while ordinary professional terminology was usually already recognised correctly.

## Protocol generation

Several local models were tested with different context limits:

- `qwen2.5-coder` produced short drafts and could not hold the full meeting comfortably;
- `gemma2:9b` failed with a small context window;
- `gemma4:12b` was too slow or memory-heavy for the available machine;
- `qwen3.5:4b` with a long context produced a full-length German draft in about 10 minutes 51 seconds;
- the same model with project vocabulary produced a shorter, more useful draft in about 6 minutes 03 seconds, with improved proper names.

The exact output is not the acceptance criterion. The important questions are whether the protocol keeps decisions, actions, figures, and attribution, and whether a person can correct it without rewriting the meeting from scratch.

## What went wrong in early runs

The evaluation exposed several engineering problems:

- a 30-second HTTP deadline was too short for a local model;
- output limits discarded otherwise useful long drafts;
- input and output budgets were not separated clearly;
- first-token delays looked like a hung process;
- reasoning models sometimes returned content in a separate `thinking` field;
- a 128 KB response cap was too small;
- asking the model to write each topic independently caused it to rewrite almost the entire transcript.

Those findings led to longer bounded requests, context-limit discovery, sectioned generation, and the current investigation into structured evidence and mechanical checks.

## Facts and coverage

Plain code found nineteen quantities in the transcript. One early generated protocol accounted for only one. This is a useful baseline because it does not depend on the model judging its own work.

Coverage alone is not enough. One topic-by-topic run covered 23 of 24 figures but produced roughly 74,000 characters—almost the length of the transcript. A professional protocol is not a transcript with headings.

## Speaker separation

The real-meeting diarisation run produced 753 segments across eight clusters when a known speaker count was used. Without that constraint, the same recording produced 86 clusters. The number is encouraging but not proof that the labels correspond to real people.

The isolated synthetic diarisation study found 88.2% frame accuracy on a 23.5-second three-speaker fixture, with 259 MB peak memory and 46 MB of model files. The fixture was clean, short, and non-overlapping; the embedding model was trained on Chinese. Long, noisy, overlapping, multilingual, and M1/8 GB tests remain necessary.

## Memory

Long context is expensive. One measured Qwen configuration used about 4.7 GB resident memory at a 40,960-token context on the 16 GB machine. Larger models and long contexts can push that machine into swap, so the M1/8 GB measurement is a release gate rather than an optimisation detail.

## What this evidence does and does not prove

It proves that the local boundaries are technically viable and that vocabulary and context handling matter. It does not yet prove professional protocol quality, broad multilingual quality, distributable runtime choices, or acceptable performance on the weakest target machine.
