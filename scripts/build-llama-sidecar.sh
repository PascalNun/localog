#!/usr/bin/env bash
set -euo pipefail

# Build the protocol-generation runtime LocaLog ships with, so that nobody
# installing the application is asked to install a second one.
#
# This is the same decision transcription already made and the same shape:
# whisper.cpp is bundled and its models are downloaded on demand. Generation
# asked people to install Ollama and run `ollama pull` in a terminal, which is a
# lot to ask of somebody whose job is writing up meetings — and it made the two
# halves of one product answer the same question differently.
#
# `llama-server` rather than the CLI, because the application already speaks HTTP
# to a local generation server. Keeping that shape means the retry logic, the
# correction passes and the stall detection survive the change.
#
# The revision is pinned for the reason whisper's is: the contract was validated
# against this build, not against whatever is current on the day.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_dir="$repo_root/src-tauri/binaries"
source_dir="${LLAMA_CPP_SOURCE_DIR:-}"
source_ref="${LLAMA_CPP_REF:-v0.3.0}"
source_commit="${LLAMA_CPP_COMMIT:-c1d0e7a004015f23bc0233470b747b596f29b264}"
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
  source_dir="$temporary_source/llama.cpp"
  echo "Cloning llama.cpp $source_ref into a temporary build directory…"
  git clone --depth 1 --branch "$source_ref" \
    https://github.com/ggml-org/llama.cpp.git "$source_dir"
fi

if [[ ! -f "$source_dir/CMakeLists.txt" ]]; then
  echo "LLAMA_CPP_SOURCE_DIR must point to a llama.cpp source checkout." >&2
  exit 1
fi

actual_commit="$(git -C "$source_dir" rev-parse HEAD 2>/dev/null || true)"
if [[ "$actual_commit" != "$source_commit" ]]; then
  echo "Expected llama.cpp commit $source_commit, found ${actual_commit:-unknown}." >&2
  echo "Set LLAMA_CPP_COMMIT explicitly only when updating the reviewed runtime revision." >&2
  exit 1
fi

# Metal is the difference between minutes and hours on Apple Silicon, and is off
# by default in a plain CMake configure.
metal_flag="-DGGML_METAL=OFF"
if [[ "$target_triple" == *apple-darwin ]]; then
  metal_flag="-DGGML_METAL=ON"
fi

build_dir="$source_dir/build-localog-$target_triple"
# Two things off deliberately, and both default to ON.
#
# LLAMA_CURL: the server links libcurl to fetch models itself, which is both a
# dependency the packaged binary should not carry and a second way to download a
# model — and downloading a model, with its licence shown and recorded, is the
# application's job rather than the runtime's.
#
# LLAMA_OPENSSL: the server offers HTTPS, which costs a link against whatever
# OpenSSL the build machine happens to have. The first build of this took
# Homebrew's, at a path no other Mac has, and the check at the end of this script
# is what caught it. Nothing here needs TLS: the only thing that ever talks to
# this server is the application on the same machine, over the loopback.
cmake -S "$source_dir" -B "$build_dir" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DLLAMA_BUILD_TESTS=OFF \
  -DLLAMA_BUILD_EXAMPLES=OFF \
  -DLLAMA_BUILD_TOOLS=ON \
  -DLLAMA_BUILD_SERVER=ON \
  -DLLAMA_CURL=OFF \
  -DLLAMA_OPENSSL=OFF \
  "$metal_flag"
cmake --build "$build_dir" --config Release \
  --target llama-server \
  --parallel "${CMAKE_BUILD_PARALLEL_LEVEL:-2}"

binary="$(find "$build_dir" -type f \( -name llama-server -o -name llama-server.exe \) -print -quit)"
if [[ -z "$binary" ]]; then
  echo "The llama.cpp build completed without producing llama-server." >&2
  exit 1
fi

mkdir -p "$sidecar_dir"
destination="$sidecar_dir/localog-llama-server-$target_triple"
if [[ "$binary" == *.exe ]]; then
  destination="$destination.exe"
fi
cp "$binary" "$destination"
chmod +x "$destination"
echo "Wrote $destination"

# Statically, so the packaged application carries no dependency on a library that
# happens to be installed on the machine that built it.
if command -v otool >/dev/null 2>&1; then
  external="$(otool -L "$destination" | tail -n +2 |
    grep -vE '/usr/lib/|/System/Library/' || true)"
  if [[ -n "$external" ]]; then
    echo "Warning: the sidecar links libraries outside the system:" >&2
    echo "$external" >&2
    echo "A packaged build would fail on a machine without them." >&2
  fi
fi
