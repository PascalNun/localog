# Bundled local runtimes

This directory contains generated, target-specific sidecars for release builds.
The sidecars are deliberately not committed to the repository: they are native
artifacts produced by the release workflow and signed together with the Tauri
application.

The expected file names are:

```text
localog-whisper-<rust-target-triple>
localog-speaker-diarization-<rust-target-triple>
localog-speaker-embedding-<rust-target-triple>
```

For example, an Apple Silicon build uses
`localog-speaker-diarization-aarch64-apple-darwin`. Tauri strips the target
suffix when it places the sidecar in the packaged application, so the Rust
runtime resolver can find the same logical name in development and release.

Run `npm run build:sidecar` from the repository root to build the reviewed
revisions for the current Rust target: whisper.cpp for transcription, and
sherpa-onnx for the two halves of optional speaker separation. Each script
verifies the source commit before compiling, because the runtimes were validated
at those revisions and not at whatever the branch has since become.

`localog-speaker-embedding` is LocaLog's own executable rather than an upstream
one. It links the sherpa-onnx C API from the same pinned revision as the
diariser, statically, so it carries no dependency on the machine that built it —
a dylib beside a sidecar would be another artifact to place, sign and notarise,
and a dynamically linked binary runs where it was built and fails on somebody
else's computer. Built on an Apple Silicon machine on 15 August 2026 it is a
14.5 MB executable linking nothing outside `/usr/lib` and `/System/Library`, and
it reproduces the reference meeting's grouping exactly.

The Rust resolver looks for these names before anything on the machine's PATH,
so a packaged build uses the runtime it was signed with rather than one that
happens to be installed. The upstream names are still accepted, so a contributor
with whisper.cpp already installed does not have to build a sidecar first. A release build must run that command
before `npm run tauri:build`; the release-only Tauri config adds the sidecar to
the bundle while keeping ordinary development and Rust test builds usable
without a native artifact in the checkout.
