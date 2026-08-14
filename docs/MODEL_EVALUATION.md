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

The reference meeting has **no known speaker count**. The owner attended, believes they were not
recorded, and spoke little; the true number is ten or eleven and nobody can settle which. Earlier
notes here treated eleven as fact. It was an estimate. Nothing below should be read as an accuracy
figure against ground truth.

Asked for eleven speakers, the whole recording produced 291 turns under eleven labels, of which only
eight are the majority speaker of any transcript segment.

Without a count, the same recording produced 86 clusters — one voice drifting across eighty minutes
of videoconference becomes many.

### Sampling instead of replaying the whole recording

Separation runs after transcription, so the segments are known, and placing a voice needs a couple of
seconds rather than a whole utterance. Two seconds from the middle of each of the 675 segments,
joined by 300 ms of silence, is 25.6 minutes of audio in place of 81.8.

Both runs asked for eleven speakers, on an M1 Pro with 16 GB:

|                                     | Whole recording |      Sampled |
| ----------------------------------- | --------------: | -----------: |
| Audio embedded                      |        81.8 min |     25.6 min |
| Time                                |          1810 s |        498 s |
| Turns                               |             291 |          128 |
| Speakers landing on segments        |               8 |           10 |
| Largest speaker's share of segments |            58 % |         56 % |
| Largest speaker's share of time     |            62 % |            — |
| Longest unbroken run                |    126 segments | 126 segments |

The two agree on 596 of 673 comparable segments, 88.6 %, after matching labels between the runs —
numbering differs because it follows whoever speaks first.

Since neither is ground truth, agreement is what can be measured, and the shapes match closely: the
same dominant speaker at 56-58 %, the same 126-segment unbroken run. That run and that dominance are
therefore properties of the diariser on this audio, not artifacts of condensing it. Sampling
resolves a slightly longer tail, not a shorter one.

### What the speaker count is worth

Swept on the sampled audio, each run compared against the eleven-speaker one after matching labels
between them:

| Asked for | Labels used | Segments per label, largest first       | Agreement |
| --------: | ----------: | --------------------------------------- | --------: |
|         6 |           6 | `385 121 100 43 19 7`                   |      96 % |
|        10 |           9 | `385 121 100 25 16 11 8 7 2`            |      99 % |
|        11 |          10 | `381 121 100 25 16 11 8 7 4 2`          |         — |
|        14 |          10 | `381 121 100 25 16 11 8 7 4 2`          |     100 % |
|        20 |          14 | `378 108 69 31 25 16 13 11 7 4 4 4 3 2` |      92 % |

Eleven and fourteen produce identical output. Between 6 and 14 the answer barely moves, so a wrong
count in that range costs almost nothing. At twenty the top speaker is untouched while the second
and third are split — 121 to 108, 100 to 69 — which is the shape of damage nobody would notice.

Three ways of not asking were measured, and none of them works:

| Method                                                  | Result                               |
| ------------------------------------------------------- | ------------------------------------ |
| Distance threshold, whole recording                     | 86 labels                            |
| Distance threshold, condensed                           | 67 labels                            |
| Sparse precheck, every 4th segment, 6.4 min, swept 4-18 | no plateau; labels track the request |
| Plateau on the full condensation                        | holds 11 to 14, breaks at 20         |

The precheck measures its sample rather than the meeting. The plateau is two agreeing points in a
narrow window, not an estimator: a real one would keep answering ten for any request above ten.

### How the samples are cut

By copying byte ranges out of the 16 kHz mono working audio, not by asking ffmpeg. Two ffmpeg routes
were measured and rejected:

- a filter graph of one `atrim` per sample splits the decoded stream once per sample and reads it
  through for each; at 753 samples it had not finished after ten minutes;
- the concat demuxer builds the file in under four seconds but rounds each out point up to a packet
  boundary, measuring 1767.7 s where 1731.6 s was planned — about 48 ms per sample and accumulating,
  which would have read the meeting's last turns back against audio 36 seconds away.

Byte copying is exact by construction and takes under a second. A test condenses a recording whose
every frame records its own millisecond and checks each sample begins and ends where planned; both
ffmpeg routes fail it.

The isolated synthetic diarisation study found 88.2% frame accuracy on a 23.5-second three-speaker fixture, with 259 MB peak memory and 46 MB of model files. The fixture was clean, short, and non-overlapping; the embedding model was trained on Chinese. Long, noisy, overlapping, multilingual, and M1/8 GB tests remain necessary.

## Memory

Long context is expensive. One measured Qwen configuration used about 4.7 GB resident memory at a 40,960-token context on the 16 GB machine. Larger models and long contexts can push that machine into swap, so the M1/8 GB measurement is a release gate rather than an optimisation detail.

## What this evidence does and does not prove

It proves that the local boundaries are technically viable and that vocabulary and context handling matter. It does not yet prove professional protocol quality, broad multilingual quality, distributable runtime choices, or acceptable performance on the weakest target machine.
