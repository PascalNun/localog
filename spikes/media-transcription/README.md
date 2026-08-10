# Media and transcription study

This isolated crate tested the local media contract using runtimes already installed on the development machine. No runtime, model, or recording is downloaded or committed by the study.

## What it tested

- structured FFprobe inspection;
- safe normalisation to mono 16 kHz PCM audio;
- cancellation of a real FFmpeg process group;
- progress parsing and bounded diagnostics;
- explicit runtime and model discovery;
- timestamped transcript parsing and validation;
- provenance, checksums, and timings.

The study used FFmpeg/FFprobe, the installed Python Whisper command, and user-provided model files to validate the contract. Python/PyTorch is not the public distribution choice; whisper.cpp remains the application direction.

Run the portable checks with:

```sh
cargo test
```

Installed-runtime checks are explicit because they require local tools and models:

```sh
cargo test --test installed_runtime -- --ignored --nocapture
```
