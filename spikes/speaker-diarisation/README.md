# Speaker diarisation spike

Validates the approach accepted in direction by **D-029**: automatic speaker separation through an
ONNX diariser rather than a Python/PyTorch stack.

Run on 2026-08-05, on the M1 Pro / 16 GB development machine — **not** the M1 / 8 GB baseline.

## What was tested

- `sherpa-onnx` v1.13.4, prebuilt macOS arm64 binaries (`osx-arm64-shared-no-tts`, 24 MB download).
- Segmentation model `sherpa-onnx-pyannote-segmentation-3-0` (7.2 MB).
- Speaker-embedding model `3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k` (39 MB).
- A synthetic German meeting with three speakers, generated from macOS `say` voices, with exact
  ground truth recorded at generation time. Content is invented; no real meeting material is used.
  See `build-fixture.sh` and `fixtures/ground-truth.json`.
- Both modes: a known speaker count, and automatic speaker-count detection across thresholds.

## Measurements

Fixture: 23.5 s, three speakers, six turns, 16 kHz mono PCM — the same audio contract LocaLog already
normalises to, so no extra conversion is required.

| Measure                                   | Result                                     |
| ----------------------------------------- | ------------------------------------------ |
| Wall time for 23.5 s of audio             | 7.24 s (≈3.2× faster than real time)       |
| Peak resident memory                      | 259 MB                                     |
| Added model footprint                     | 46 MB (7.2 MB + 39 MB)                     |
| Frame accuracy at 10 ms, 3 clusters given | 88.2 % correct, 5.5 % unlabelled           |
| Speakers found automatically              | 3 of 3, at every threshold from 0.4 to 0.8 |

Every one of the three speakers was separated and consistently identified. Six of seven predicted
turns mapped to the correct speaker; one 0.56 s span was misattributed. Automatic speaker-count
detection was stable across the whole threshold range tested, so LocaLog would not need to ask the
user how many people were in the meeting, nor tune a threshold per recording.

## Packaging finding

The downloaded binaries are killed immediately by macOS on Apple Silicon (`SIGKILL`, exit 137, no
output) until they are ad-hoc signed:

```sh
codesign --force -s - lib/*.dylib bin/sherpa-onnx-offline-speaker-diarization
```

Anything LocaLog ships will be signed as part of the application, so this affects development
convenience rather than distribution — but it is exactly the failure mode that looks like a crash
with no diagnostic, so it is worth knowing.

## Assessment

The approach looks viable and notably cheaper than expected. 46 MB of models and 259 MB of peak
memory is a small addition next to a 148 MB–1.5 GB transcription model, and running faster than real
time means diarisation adds a modest fraction to the time a user already waits for transcription.
The ONNX route also avoids the Python/PyTorch footprint that was rejected for transcription.

## Limits of this evidence

These results are the easy case, and should not be read as production quality:

- **Synthetic speech is unrealistically favourable.** TTS voices are clean, evenly spaced, and far
  more acoustically distinct than real people in a room. No background noise, no reverberation, no
  crosstalk.
- **No overlapping speech was tested**, which is common in real meetings and is where diarisation
  usually degrades.
- **The embedding model is trained on Chinese** and was used on German because it is the documented
  example. It still worked well, but a multilingual or German-suited model should be compared before
  anything is selected.
- **One short fixture.** No long-recording behaviour, no memory growth over an hour of audio.
- **Not measured on the M1 / 8 GB baseline**, so the timing above is optimistic.
- **Alignment is not solved here.** Diariser turns and whisper segments do not share boundaries;
  mapping one onto the other is the remaining design work and was not attempted in this spike.

## Suggested keep/change

- **Keep** sherpa-onnx as the candidate diariser, and keep it behind an application port rather than
  letting it into the domain model.
- **Keep** the fixture and its ground truth as a test oracle; regenerate rather than commit audio.
- **Change before implementation:** compare a multilingual or German-suited embedding model; test
  overlapping speech and a long recording; repeat on the M1 / 8 GB baseline.
- **Do not import this spike's invocation into production.** The production adapter goes through the
  existing supervised-process boundary, with cancellation and bounded output like the whisper.cpp
  adapter.
