#!/usr/bin/env bash
# Build every sidecar this platform can build.
#
# Four of the five are portable: they take a target triple and select their own
# accelerator from it, so the same script produces a Metal build on macOS and a
# CPU build elsewhere. The recorder is the exception and is meant to be — it is
# Core Audio and AVFoundation in one Swift file, and there is no honest way to
# build it off Darwin.
#
# Skipping it rather than failing is what makes a Linux build possible at all.
# Nothing above the recorder assumes it exists: `recording.rs` reports
# `recorderMissing` when the binary is absent, the interface says so in eight
# languages, and importing a recording — which is the path that actually works
# today — never touches it.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"

# Two recorders, and the split is the capability rather than the language. macOS
# taps a single application's audio through Core Audio, which is what makes a
# recording of the call rather than of everything the machine is playing. Windows
# and Linux have no equivalent, so theirs captures the output as a whole — one
# crate for both, because WASAPI loopback and a PipeWire monitor source are the
# same shape once something has abstracted them.
if [[ "$(uname -s)" == "Darwin" ]]; then
  bash "$here/scripts/build-recorder-sidecar.sh"
else
  bash "$here/scripts/build-recorder-portable-sidecar.sh"
fi

bash "$here/scripts/build-whisper-sidecar.sh"
bash "$here/scripts/build-llama-sidecar.sh"
bash "$here/scripts/build-sherpa-sidecar.sh"
bash "$here/scripts/build-embedding-sidecar.sh"
bash "$here/scripts/build-ffmpeg-sidecar.sh"
