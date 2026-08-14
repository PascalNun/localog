# Bundled local runtimes

This directory contains generated, target-specific sidecars for release builds.
The sidecars are deliberately not committed to the repository: they are native
artifacts produced by the release workflow and signed together with the Tauri
application.

The expected file names are:

```text
localog-whisper-<rust-target-triple>
localog-speaker-diarization-<rust-target-triple>
```

For example, an Apple Silicon build uses
`localog-speaker-diarization-aarch64-apple-darwin`. Tauri strips the target
suffix when it places the sidecar in the packaged application, so the Rust
runtime resolver can find the same logical name in development and release.

Run `npm run build:sidecar` from the repository root to build both reviewed
revisions for the current Rust target: whisper.cpp for transcription and
sherpa-onnx for optional speaker separation. Each script verifies the source
commit before compiling, because the runtimes were validated at those revisions
and not at whatever the branch has since become.

The Rust resolver looks for these names before anything on the machine's PATH,
so a packaged build uses the runtime it was signed with rather than one that
happens to be installed. The upstream names are still accepted, so a contributor
with whisper.cpp already installed does not have to build a sidecar first. A release build must run that command
before `npm run tauri:build`; the release-only Tauri config adds the sidecar to
the bundle while keeping ordinary development and Rust test builds usable
without a native artifact in the checkout.
