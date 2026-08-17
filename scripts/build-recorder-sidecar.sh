#!/usr/bin/env bash
# Build the meeting recorder as a Tauri sidecar.
#
# One Swift file, no package manifest, following the study it came from. What matters
# here is not the compile but the two things around it.
#
# The Info.plist is *linked into the binary* rather than placed beside it. Capturing
# audio is gated on a usage description, and a bare executable without one is handed
# silence rather than an error — which is how the study spent a run recording 235
# seconds of nothing before anybody noticed.
#
# The signature identity is stable, because macOS remembers the Screen & System Audio
# Recording permission against an identity. An ad-hoc signature that changes on every
# build asks the person to grant it again every time, and a permission somebody has to
# keep re-granting is one they will eventually grant to the wrong thing.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
spike="$here/spikes/meeting-recording"
target="${1:-aarch64-apple-darwin}"
out="$here/src-tauri/binaries/localog-record-meeting-$target"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This recorder is the macOS one. Linux and Windows need their own." >&2
  exit 1
fi

mkdir -p "$here/src-tauri/binaries"
swiftc -O -o "$out" "$spike/Sources/main.swift" \
  -framework AVFoundation -framework CoreAudio -framework Foundation \
  -Xlinker -sectcreate -Xlinker __TEXT -Xlinker __info_plist -Xlinker "$spike/Info.plist"

codesign --force --sign - --identifier app.localog.record-meeting "$out"

echo "built $out"
echo
echo "System audio also needs the person to grant Screen & System Audio Recording to"
echo "LocaLog itself. Until they do, macOS hands this binary silence rather than"
echo "refusing it, so the recorder asks first and declines rather than recording nothing."
