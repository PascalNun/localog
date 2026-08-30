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

if [[ "$(uname -s)" == "Darwin" ]]; then
  bash "$here/scripts/build-recorder-sidecar.sh"
else
  echo "Not Darwin: skipping the recorder. This build will import recordings but not make them."
fi

bash "$here/scripts/build-whisper-sidecar.sh"
bash "$here/scripts/build-sherpa-sidecar.sh"
bash "$here/scripts/build-embedding-sidecar.sh"
bash "$here/scripts/build-ffmpeg-sidecar.sh"
