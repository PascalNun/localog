#!/usr/bin/env bash
set -euo pipefail

# Build the one native executable LocaLog needs for optional speaker separation.
# This is a release/developer command, never an action performed by the app at
# runtime. The source revision is pinned so the shipped binary can be audited.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_dir="$repo_root/src-tauri/binaries"
source_dir="${SHERPA_ONNX_SOURCE_DIR:-}"
source_ref="${SHERPA_ONNX_REF:-v1.12.20}"
source_commit="${SHERPA_ONNX_COMMIT:-5ce3d6d93a5f4fe11657bf11a6bf3a022eeef22f}"
target_triple="${TAURI_TARGET_TRIPLE:-$(rustc -vV | sed -n 's/^host: //p')}"

if [[ -z "$target_triple" ]]; then
  echo "Could not determine the Rust target triple. Set TAURI_TARGET_TRIPLE." >&2
  exit 1
fi

temporary_source=""
cleanup() {
  if [[ -n "$temporary_source" ]]; then
    rm -rf "$temporary_source"
  fi
}
trap cleanup EXIT

if [[ -z "$source_dir" ]]; then
  temporary_source="$(mktemp -d)"
  source_dir="$temporary_source/sherpa-onnx"
  echo "Cloning sherpa-onnx $source_ref into a temporary build directory…"
  git clone --depth 1 --branch "$source_ref" \
    https://github.com/k2-fsa/sherpa-onnx.git "$source_dir"
fi

if [[ ! -f "$source_dir/CMakeLists.txt" ]]; then
  echo "SHERPA_ONNX_SOURCE_DIR must point to a sherpa-onnx source checkout." >&2
  exit 1
fi

actual_commit="$(git -C "$source_dir" rev-parse HEAD 2>/dev/null || true)"
if [[ "$actual_commit" != "$source_commit" ]]; then
  echo "Expected sherpa-onnx commit $source_commit, found ${actual_commit:-unknown}." >&2
  echo "Set SHERPA_ONNX_COMMIT explicitly only when updating the reviewed runtime revision." >&2
  exit 1
fi

build_dir="$source_dir/build-localog-$target_triple"
cmake -S "$source_dir" -B "$build_dir" \
  -DCMAKE_BUILD_TYPE=Release \
  -DSHERPA_ONNX_ENABLE_BINARY=ON \
  -DSHERPA_ONNX_ENABLE_SPEAKER_DIARIZATION=ON \
  -DSHERPA_ONNX_ENABLE_TTS=OFF \
  -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF \
  -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF \
  -DSHERPA_ONNX_BUILD_C_API_EXAMPLES=OFF
cmake --build "$build_dir" --config Release \
  --target sherpa-onnx-offline-speaker-diarization \
  --parallel "${CMAKE_BUILD_PARALLEL_LEVEL:-2}"

binary="$(find "$build_dir" -type f \
  \( -name sherpa-onnx-offline-speaker-diarization -o \
     -name sherpa-onnx-offline-speaker-diarization.exe \) \
  -print -quit)"
if [[ -z "$binary" ]]; then
  echo "The sherpa-onnx build completed without producing the diarisation executable." >&2
  exit 1
fi

mkdir -p "$sidecar_dir"
destination="$sidecar_dir/localog-speaker-diarization-$target_triple"
if [[ "$binary" == *.exe ]]; then
  destination="$destination.exe"
fi
cp "$binary" "$destination"
chmod +x "$destination"
echo "Wrote $destination"
