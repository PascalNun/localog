#!/usr/bin/env bash
set -euo pipefail

# Build the transcription runtime LocaLog ships with, so that nobody installing
# the application is asked to find an executable for it.
#
# This is a release/developer command and never something the app does at run
# time. The source revision is pinned because the transcription contract — the
# JSON shape, the per-token probabilities, the progress output — was validated
# against this build and not against whatever happens to be on a machine.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_dir="$repo_root/src-tauri/binaries"
source_dir="${WHISPER_CPP_SOURCE_DIR:-}"
source_ref="${WHISPER_CPP_REF:-v1.9.2}"
source_commit="${WHISPER_CPP_COMMIT:-306c88f4d1286aec1bf96e544632897886af5501}"
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
  source_dir="$temporary_source/whisper.cpp"
  echo "Cloning whisper.cpp $source_ref into a temporary build directory…"
  git clone --depth 1 --branch "$source_ref" \
    https://github.com/ggml-org/whisper.cpp.git "$source_dir"
fi

if [[ ! -f "$source_dir/CMakeLists.txt" ]]; then
  echo "WHISPER_CPP_SOURCE_DIR must point to a whisper.cpp source checkout." >&2
  exit 1
fi

actual_commit="$(git -C "$source_dir" rev-parse HEAD 2>/dev/null || true)"
if [[ "$actual_commit" != "$source_commit" ]]; then
  echo "Expected whisper.cpp commit $source_commit, found ${actual_commit:-unknown}." >&2
  echo "Set WHISPER_CPP_COMMIT explicitly only when updating the reviewed runtime revision." >&2
  exit 1
fi

# Metal is the reason transcription is minutes rather than hours on Apple
# Silicon, and is off by default in a plain CMake configure.
metal_flag="-DGGML_METAL=OFF"
if [[ "$target_triple" == *apple-darwin ]]; then
  metal_flag="-DGGML_METAL=ON"
fi

build_dir="$source_dir/build-localog-$target_triple"
cmake -S "$source_dir" -B "$build_dir" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DWHISPER_BUILD_TESTS=OFF \
  -DWHISPER_BUILD_EXAMPLES=ON \
  -DWHISPER_BUILD_SERVER=OFF \
  "$metal_flag"
cmake --build "$build_dir" --config Release \
  --target whisper-cli \
  --parallel "${CMAKE_BUILD_PARALLEL_LEVEL:-2}"

binary="$(find "$build_dir" -type f \( -name whisper-cli -o -name whisper-cli.exe \) -print -quit)"
if [[ -z "$binary" ]]; then
  echo "The whisper.cpp build completed without producing whisper-cli." >&2
  exit 1
fi

mkdir -p "$sidecar_dir"
destination="$sidecar_dir/localog-whisper-$target_triple"
if [[ "$binary" == *.exe ]]; then
  destination="$destination.exe"
fi
cp "$binary" "$destination"
chmod +x "$destination"
echo "Wrote $destination"

# Built statically so the packaged application carries no dependency on a
# library that happens to be installed. A dynamically linked sidecar would run
# on the machine it was built on and fail on somebody else's.
if command -v otool >/dev/null 2>&1; then
  external="$(otool -L "$destination" | tail -n +2 |
    grep -vE '/usr/lib/|/System/Library/' || true)"
  if [[ -n "$external" ]]; then
    echo "Warning: the sidecar links libraries outside the system:" >&2
    echo "$external" >&2
    echo "A packaged build would fail on a machine without them." >&2
  fi
fi
