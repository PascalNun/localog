# A thirty-minute meeting with known speakers

This fixture gives speaker separation a longer, labelled baseline. Four invented
people plan a community repair workshop in German. They discuss space, access,
equipment, money and responsibilities, correct earlier statements, and leave
some decisions open. Short replies appear between longer turns.

The [first measured results](RESULTS.md) show why speaker attribution is still
under evaluation, including similar TTS voices and transcript segments that
cross speaker changes.

The script is written for this repository. No real meeting, participant or client
material is used. The generated audio stays in the ignored `eval/` directory;
Apple's installed voice assets are not part of the repository or its licence.

## Generate the recording

On a Mac with FFmpeg and the four listed German voices installed:

```sh
python3 spikes/synthetic-meeting/build.py --out eval/synthetic-meeting-30m
```

The generator uses `say`, not a cloud service. It makes exactly 30:00 of mono
16 kHz PCM audio and an M4A copy suitable for importing into LocaLog. A small,
recorded tempo adjustment preserves voice pitch and fits the conversation to
the requested duration. It does not fill the time with a long silent ending.

The output also contains:

- `reference.md`: the script with speaker names and timestamps;
- `reference.json` and `reference.rttm`: machine-readable turn boundaries;
- `manifest.json`: voices, rates, system version, tempo factor and checksums;
- `parts/`: local intermediate renders, retained to make interrupted builds cheap
  to restart.

The generator refuses to overwrite a completed fixture. New OS or voice versions
may sound different; provenance and the recorded checksums identify a render,
rather than promising byte-identical synthesis on another machine.

For a manual app check, import the WAV or M4A, select German, request speaker
separation with four speakers and compare the result with the reference. Keep
the test project separate from any real project. Automatic count estimation is
a second check, not the source of the expected answer.

## Measure the current adapters

`evaluate.py` runs the shipped embedding executable and compiles a tiny wrapper
around the application's actual Rust clustering module. It first uses known
turn boundaries, then transcribes the entire recording with whisper.cpp and
repeats the check using those transcript boundaries. Each pass reports both a
supplied count of four and the current automatic count estimate.

```sh
python3 spikes/synthetic-meeting/evaluate.py \
  --fixture eval/synthetic-meeting-30m \
  --out eval/synthetic-meeting-30m-run1 \
  --embedding-executable src-tauri/binaries/localog-speaker-embedding-aarch64-apple-darwin \
  --embedding-model /path/to/speaker-embedding.onnx \
  --whisper-executable src-tauri/binaries/localog-whisper-aarch64-apple-darwin \
  --whisper-model /path/to/ggml-base.bin
```

No weights are downloaded and the app's database is not opened. Native runtime
builds and installed models are explicit inputs. Rendering requires macOS;
the scorer itself uses only Python's standard library, and the clustering
wrapper needs Rust. Use the corresponding native executables on other systems.

`results.json` records hashes, wall time, peak resident memory on macOS and
assignment results. Predicted segments and runtime diagnostics stay alongside
it. The output directory must be new so results cannot silently reuse a stale
transcript. The test deliberately keeps the current application's neutral
`Speaker 1` fallback for segments too short to embed, and reports how many
segments received that fallback.

## What the numbers mean

Labels are matched one-to-one against the four reference speakers, independently
for each run. Renumbering a correct speaker does not count as an error; splitting
one person into many clusters does. Correct assignment is weighted by the time
where predicted windows intersect reference utterance windows. Missed windows,
confused windows and predictions outside reference windows are reported
separately. Per-speaker results make a dominant voice less likely to hide a
missing participant.

The scorer also computes the best possible attribution if each predicted segment
can have only one speaker. This separates the cost of transcript boundaries that
cross a speaker change from the classification errors themselves. It counts
crossings both with any positive overlap and with at least 250 ms from each of
two speakers; the latter is a diagnostic convention, not a scoring collar.

These are **utterance-window measurements, not standard diarisation error rate
(DER)**. Reference turns contain sentence pauses and are not a phonetic
speech-activity annotation. The fixture has no overlap, room noise, microphone
variation, accents or spontaneous human speech. TTS voices may also share a
synthesis engine. A good result establishes a clean synthetic baseline, not
readiness for real meetings.

This is an adapter-level check. It does not exercise Tauri commands, durable
jobs, cancellation, recovery or the review interface. Those need separate tests.

The scoring tests run without voices, media or models:

```sh
python3 -m unittest discover -s spikes/synthetic-meeting -p 'test_*.py' -v
```

Keep this code as an evaluation fixture, not application architecture. Do not
tune the clustering threshold against this single recording.

## Ming 3.0 on Apple Silicon

The [measured Ming results](MING_RESULTS.md) cover the completed recording,
spoken-content comparison and unresolved speaker-attribution failures.

The second renderer uses Ming 3.0's dense 0.5B model through MLX-Audio, isolated
from the application. Setup is explicit; rendering itself is offline. It needs
Python 3.12, FFmpeg and an Apple Silicon Mac with a working Metal device. The
pinned MLX wheel requires macOS 26 or later; this does not change LocaLog's
platform requirements.

```sh
python3.12 -m venv eval/ming-tts/venv
eval/ming-tts/venv/bin/pip install --no-cache-dir -r spikes/synthetic-meeting/requirements-ming.txt

HF_HOME="$PWD/eval/ming-tts/hub" HF_HUB_DISABLE_TELEMETRY=1 HF_HUB_DISABLE_XET=1 \
  eval/ming-tts/venv/bin/python spikes/synthetic-meeting/prepare_ming.py \
  --out eval/ming-tts/model

HF_HUB_OFFLINE=1 HF_HUB_DISABLE_TELEMETRY=1 \
  eval/ming-tts/venv/bin/python spikes/synthetic-meeting/build_ming.py \
  --model eval/ming-tts/model --out eval/synthetic-meeting-ming --audition
```

The downloader pins the checkpoint revision and verifies the published SHA-256
checksums for the weights and tokenizer. It downloads approximately 3 GB and
requires at least 6 GiB free before starting. All model files remain ignored.
The generator verifies the local files again before loading them, refuses to mix
settings within one render, and stops between turns if free space falls below
3 GiB. Its Metal cache is bounded; it does not raise the system's GPU allowance.

The audition contains two turns each from Mara/US_Emma, Jonas/US_James,
Lea/UK_Charlotte and Timur/UK_George. These are English preset identities asked
to speak German, not native German speaker references. Check pronunciation,
identity and delivery before assuming that the model suits the recording.
No real voice is cloned. MLX-Audio 0.6.2 includes the upstream Ming prompt and
sampling fixes; known stopping and speaker-conditioning limitations still need
watching.

Remove `--audition` to render the whole script in the same output directory.
Completed, checksummed utterances are reused, including the audition takes.
Interrupted turns have no completion metadata and are rendered again. Per-turn
seeds, generation settings, model hashes and package versions are recorded. The
original Apple-TTS recording is preserved for comparison.

The final output has the same WAV, M4A and reference format as the first fixture,
so `evaluate.py` can measure it with unchanged transcription and speaker models.
Native 44.1 kHz utterance files are retained in `parts/` for listening; the
canonical meeting is 16 kHz to match the application's normalised input.

Reference text and a high speaker score do not establish naturalness or spoken
content accuracy. Listen to the audition and check the generated speech against
the script. This remains development-only test tooling, not a TTS feature in
LocaLog.

After transcribing an audition or full recording with the local Whisper runtime,
check recognised content separately:

```sh
python3 spikes/synthetic-meeting/check_speech.py \
  --reference eval/synthetic-meeting-ming/reference.json \
  --whisper-json eval/synthetic-meeting-ming-run1/whisper.json \
  --out eval/synthetic-meeting-ming-run1/speech-content.json
```

Use `audition-reference.json` for the short sample. This word-edit measurement
ignores case and punctuation but counts spelling and number-format differences.
It is an ASR-based check, not a human judgement of naturalness, and must not be
reported as the model's official benchmark result.

To run the adapter evaluation on the audition, pass `--audio-name audition.wav`
and `--reference-name audition-reference.json` to `evaluate.py`. Its defaults
continue to select the complete thirty-minute recording.
