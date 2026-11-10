# First long synthetic speaker check — 26 October 2026

The fixture is usable for repeatable development tests. The current speaker
pipeline does not separate its four voices reliably. That is a result about
this recording, not an accuracy estimate for human meetings.

## Recording and setup

The recording is exactly 30:00, with 120 turns and approximately 4,800 German
words. Four invented participants discuss a workshop renovation. It uses the
installed Anna Premium, Eddy, Sandy and Reed voices. A pitch-preserving tempo
factor of 1.2762 fits the rendered speech to thirty minutes, with short gaps
between turns. There is no overlap, background noise or real personal data.

The test ran on an M1 Pro with 16 GB RAM and macOS 27.0.1. It used the freshly
built macOS sidecars, the installed multilingual Whisper base model and the
installed speaker embedding model. No models were downloaded. Model SHA-256:

- Whisper base: `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`
- Speaker embedding: `1a331345f04805badbb495c775a6ddffcdd1a732567d5ec8b3d5749e3c7a5e4b`

The evaluation uses the shipped embedding executable and includes the actual
`src-tauri/src/clustering.rs` source, with the unchanged automatic floor of
0.20. Embedding uses eight threads; Whisper uses its default four. Known and
automatic counts are read from the same embeddings, not separate inference runs.

## Results

Correct attribution is measured over reference utterance windows, after an
optimal one-to-one label match. These percentages are **not standard DER** and
do not measure transcription accuracy.

| Boundaries         | Count supplied | Voices produced | Correct reference-window time |
| ------------------ | -------------- | --------------: | ----------------------------: |
| Known TTS turns    | Four           |               4 |                        74.87% |
| Known TTS turns    | Automatic      |               2 |                        51.71% |
| Whisper transcript | Four           |               4 |                        71.26% |
| Whisper transcript | Automatic      |               2 |                        49.15% |

On the first run, Whisper transcribed all thirty minutes in 65.7 seconds and
produced 301 segments. Embedding took 9.0 seconds for the reference turns and
18.3 seconds for the Whisper segments. Clustering took less than 0.2 seconds.
Peak resident memory was about 0.92 GiB and 0.95 GiB respectively for the
embedding passes, and 0.58 GiB for Whisper. These are individual process peaks,
not whole-application memory, swap behaviour or evidence for the M1/8 GB baseline.

A second complete run returned the same speaker counts and attribution
percentages. Whisper took 42.4 seconds; the two embedding passes took 8.2 and
18.4 seconds. Treat timing as these observed runs, not a guaranteed runtime.
Six scorer tests and eight existing Rust clustering tests passed; the older
study-dependent Rust test remains explicitly ignored. FFprobe confirmed both
audio copies are thirty minutes long. Formatting and whitespace checks passed.

## What failed

**Two synthetic voices are very close in embedding space.** Jonas/Eddy and
Timur/Reed merge even when the true boundaries are supplied. Forcing four groups
splits Mara/Anna instead of separating that pair. Mean cosine similarity for
Eddy versus Reed is about 0.757, close to within-voice means of 0.781 and 0.796
(the within-voice figures include self-pairs). The installed identifiers put
Eddy, Sandy and Reed in Apple's Eloquence family. Different voice names do not
establish independent human-like voice identities. This fixture cannot, by
itself, distinguish a synthesis-family limitation from a model limitation.

**Transcript boundaries do not reliably mark speaker changes.** Eighty-one of
301 Whisper segments intersect more than one reference speaker. Sixty-two
contain at least 250 ms from each of two speakers. Even a perfect classifier
restricted to one speaker per segment could reach only 93.34% on these windows.
That is a measurable limitation of the current segment-level approach.

**Short turns expose the neutral-label fallback.** Ten reference turns are too
short for an embedding and receive the current application's `Speaker 1`
fallback. That label does not mean the first participant was identified. All
Whisper segments were long enough to embed, partly because short replies were
absorbed into larger segments.

The test does not check overlap, accents, microphone changes, room acoustics,
human speech, the review UI, durable jobs or cancellation.

## Keep/change decision

Keep the script, generator, reference timings and scorer as evaluation tools.
Keep the generated recording and diagnostic artifacts local. None of the spike
code becomes application code, and this test changes no runtime settings.

Do not tune the automatic threshold against this single fixture. First add an
independently voiced control, then consented human audio with known speakers.
On that material, compare embedding models and measure speaker-change handling
separately from clustering. Any production change should show whether it fixes
those errors without losing short replies or increasing correction work.

The immediate product-facing check is to import this recording, request four
speakers and compare several transitions with `reference.md`. Use it to inspect
renaming and correction as well as the automatic output; do not present the
speaker labels as verified attribution.
