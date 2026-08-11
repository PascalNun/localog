# Bundled local runtimes

This directory contains generated, target-specific sidecars for release builds.
The sidecars are deliberately not committed to the repository: they are native
artifacts produced by the release workflow and signed together with the Tauri
application.

The expected file name is:

```text
localog-speaker-diarization-<rust-target-triple>
```

For example, an Apple Silicon build uses
`localog-speaker-diarization-aarch64-apple-darwin`. Tauri strips the target
suffix when it places the sidecar in the packaged application, so the Rust
runtime resolver can find the same logical name in development and release.

Run `npm run build:sidecar` from the repository root to build the reviewed
sherpa-onnx revision for the current Rust target. The script verifies the
source commit before compiling. A release build must run that command
before `npm run tauri:build`; the release-only Tauri config adds the sidecar to
the bundle while keeping ordinary development and Rust test builds usable
without a native artifact in the checkout.
