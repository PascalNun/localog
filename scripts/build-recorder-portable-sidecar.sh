#!/usr/bin/env bash
set -euo pipefail

# Build the recorder for the platforms that are not macOS.
#
# macOS keeps its own, in Swift: Core Audio process taps capture a single
# application's output, which neither Windows nor Linux offers, and that recorder
# is written, signed against a stable identity for the permission macOS remembers,
# and working. Replacing it with something more portable and less capable would be
# a loss.
#
# This one records what the speakers are playing as a whole. On Windows that is
# WASAPI loopback on a rendering device; on Linux it is the monitor source
# PipeWire and PulseAudio publish beside each sink. One crate covers both, which
# is why this is one sidecar rather than two.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
crate_dir="$repo_root/sidecars/record-meeting"
sidecar_dir="$repo_root/src-tauri/binaries"
target_triple="${TAURI_TARGET_TRIPLE:-$(rustc -vV | sed -n 's/^host: //p')}"

if [[ -z "$target_triple" ]]; then
  echo "Could not determine the Rust target triple. Set TAURI_TARGET_TRIPLE." >&2
  exit 1
fi

rustup target add "$target_triple" >/dev/null 2>&1 || true

cargo build --release \
  --manifest-path "$crate_dir/Cargo.toml" \
  --target "$target_triple"

built="$crate_dir/target/$target_triple/release/localog-record-meeting"
destination="$sidecar_dir/localog-record-meeting-$target_triple"
if [[ -f "$built.exe" ]]; then
  built="$built.exe"
  destination="$destination.exe"
fi
if [[ ! -f "$built" ]]; then
  echo "The recorder build completed without producing a binary." >&2
  exit 1
fi

mkdir -p "$sidecar_dir"
cp "$built" "$destination"
chmod +x "$destination" 2>/dev/null || true
echo "Wrote $destination"
