#!/usr/bin/env bash
# Build all seven sidecars for the host platform. The recorder selects its
# platform backend below; the inference and media scripts select their own
# accelerator, source build or reviewed prebuilt for the target.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"

# Two recorders, and the split is the capability rather than the language. macOS
# taps a single application's audio through Core Audio, which is what makes a
# recording of the call rather than of everything the machine is playing. Windows
# and Linux currently capture the selected output as a whole. Their backend
# adapters share one two-track command and progress contract.
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
