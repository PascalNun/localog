#!/usr/bin/env bash
# One file, one binary. A study does not need a package manifest.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p build
# The Info.plist is linked into the binary, not placed beside it. A process tap
# is gated on a usage description, and a bare executable that has none is handed
# silence rather than an error - which is how this study spent a run recording
# 235 seconds of nothing.
swiftc -O -o build/record-meeting Sources/main.swift \
  -framework AVFoundation -framework CoreAudio -framework Foundation \
  -Xlinker -sectcreate -Xlinker __TEXT -Xlinker __info_plist -Xlinker Info.plist

# Ad-hoc signing, so the identity the permission is remembered against is stable.
codesign --force --sign - --identifier app.localog.spike.record-meeting build/record-meeting
echo "built build/record-meeting"
