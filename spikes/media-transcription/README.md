# Media normalization and transcription spike

This crate is isolated from `src-tauri`. It validates installed-runtime contracts and must not be imported as production architecture.

## Installed-runtime scope

- FFmpeg/FFprobe 8.1.2 from Homebrew.
- OpenAI Whisper CLI 20250625 from the user's pipx environment.
- User-provided `medium.pt` and `large-v3.pt` model files already present in the Whisper cache.
- No runtime, binary, or model is downloaded by this spike.

The proposed public adapter remains `whisper.cpp`. Because no compatible installed `whisper.cpp` binary/model pair is available, this spike uses the installed Python Whisper runtime to validate normalized media, timestamps, model discovery/provenance, failure diagnostics, and output validation. It does not accept Python/PyTorch as the public distribution model.

## Acceptance checks

- Discover explicit installed runtimes and models without network access.
- Probe a synthetic video/audio container through structured FFprobe output.
- Normalize to mono 16 kHz PCM WAV without modifying the source.
- Parse bounded FFmpeg progress and reject missing/non-audio input.
- Cancel a real FFmpeg process group.
- Transcribe generated synthetic speech with the installed model.
- Validate non-empty timestamped segments without asserting byte-identical text.
- Record runtime version, model size/checksum, settings, input/output checksums, duration, and timing.

Run the portable/unit checks with `cargo test`. Run the installed-runtime checks explicitly with:

```sh
cargo test --test installed_runtime -- --ignored --nocapture
```

The keep/change decision and measurements are recorded in `docs/DECISIONS.md` after validation.
