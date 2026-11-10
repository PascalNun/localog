# Ming 3.0 recording — 27 October 2026

Ming runs locally on the M1 Pro and has produced the complete thirty-minute
meeting. The speech has substantially better agreement with the script through
the same Whisper recogniser than the Apple-TTS version. Speaker attribution is
still unreliable. These are separate findings.

## What ran

The test uses `mlx-community/Ming-3.0-tts-0.5B-bf16`, pinned to revision
`283b3c342c6fb69a7a87832c97f9799567efe05d`. The downloader verified the published
SHA-256 checksums, and the renderer verified the local files again before loading.
The main weights hash is
`16a7086c992a6eab7d855df90c9c2edbb0f4f3a702b3cd697f6011e198d0c8b4`.

The isolated Python 3.12 environment uses MLX-Audio 0.6.2, MLX/MLX-Metal 0.32.4,
Transformers 5.21.0 and SoundFile 0.14.0. The machine is an Apple M1 Pro with
16 GB RAM and macOS 27.0.1. BF16 Metal computation and full model inference both
worked. No NVIDIA runtime, cloud inference or running TTS server is involved.
Downloads were explicit; rendering and the evaluation ran offline.

The four intended speakers use US_Emma, US_James, UK_Charlotte and UK_George,
with German requested explicitly. These are English preset identities, not native
German references or clones of real participants. We have not independently
verified that each identity stays perceptually consistent throughout the audio.

## The recording

The unchanged fictional script has 120 turns and approximately 4,800 words.
Native utterances total 32.80 minutes. A pitch-preserving tempo factor of
1.1231, with short gaps between turns, produces exactly 30:00 without dropping
scripted turns or adding a long silent ending.

The canonical WAV is mono, 16 kHz, PCM16; an M4A copy is also available. Native
44.1 kHz utterances, seeds, generation settings, hashes and reference timings
remain in the local output folder. The old Apple-TTS recording is preserved.
Neither model weights nor generated media are committed.

The short audition took 126.3 seconds of synthesis for 142.5 seconds of utterance
audio, before its inter-turn gaps. The full render accumulated 41.83 minutes of
per-turn generation time. Speed varied markedly during the longer session, so
that total is not a controlled hardware benchmark or a promised runtime.
MLX's reported peak allocation stayed around 4.42 GB. This is not whole-process
or whole-system memory, and it does not validate the M1/8 GB baseline.

## Spoken content

Both full recordings were checked against the same script with the installed
multilingual Whisper base model and the same word-edit scorer:

| Recording | Reference words | Word edits | Word-error percentage |
| --------- | --------------: | ---------: | --------------------: |
| Apple TTS |           4,791 |      1,048 |                21.87% |
| Ming 3.0  |           4,791 |        185 |                 3.86% |

The scorer normalises case and punctuation, but number formats and spelling
differences still count. Whisper can introduce errors itself. This is a useful
recognition-based intelligibility check, **not a naturalness rating or an
official model benchmark**. The four-voice audition is available for listening.

## Speaker attribution

The embedding model and actual Rust clustering code were unchanged. Labels are
matched one-to-one against the intended speaker roles, weighted by reference
utterance-window time. These percentages are not standard DER or human-meeting
accuracy.

| Boundaries         | Count supplied | Voices produced | Reference-window agreement |
| ------------------ | -------------- | --------------: | -------------------------: |
| Known TTS turns    | Four           |               4 |                     53.95% |
| Known TTS turns    | Automatic      |               3 |                     53.95% |
| Whisper transcript | Four           |               4 |                     73.60% |
| Whisper transcript | Automatic      |               2 |                     52.14% |

The audition's eight known turns grouped perfectly when four speakers were
supplied. **That did not generalise to the full recording.** On the full Whisper
path, Lea receives only 0.13% agreement while the other roles exceed 93%.
The average therefore hides an almost entirely unresolved participant; 73.60%
must not be presented as dependable attribution. The older Apple fixture scored
71.26% on that path, with automatic mode also finding two voices.

The known-turn pass merges most of the two female roles together and most of
the two male roles together, using extra groups for outliers rather than the
intended identities. This cannot be explained by transcript boundaries alone.
It does not establish whether the main cause is synthetic voice similarity or
drift, the embedding model, or clustering; independent listening and another
embedding model are needed to separate those questions.

Whisper produced 486 segments in 43.8 seconds. Embedding took 8.5 seconds for
the known turns and 26.3 seconds for Whisper's segments; clustering took less
than 0.4 seconds. Twelve reference turns were too short to embed and received
the app's existing `Speaker 1` fallback. No Whisper segments were skipped.
Eighty-five transcript segments intersect more than one intended speaker;
59 contain at least 250 ms from each of two speakers. A perfect single-label
classifier on these boundaries could reach 97.73%, leaving substantial error
beyond the boundary limitation.

## Keep/change decision

Keep the Ming setup and renderer as isolated evaluation tools. Keep this recording
as an additional test case, not as proof of reliable speaker identities or a
replacement for labelled human speech. No application dependencies, settings,
data or production behaviour changed, and no clustering threshold was tuned.

Next, listen to repeated passages from each role and check identity consistency.
Then compare an independent speaker embedding model on the same audio before
changing clustering. Validate any resulting change with consented human speech
whose speakers are known. Do not save automatic labels as verified identities.

Fourteen fixture-tool tests and eight Rust clustering tests passed; the older
study-dependent test remains ignored. Dependency checks, Python compilation,
Markdown formatting and whitespace checks passed. FFprobe confirmed 1,800
seconds for both final audio copies. The temporary idle-sleep prevention
assertion ended with the renderer.
