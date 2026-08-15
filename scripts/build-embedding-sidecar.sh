#!/usr/bin/env bash
set -euo pipefail

# Build the speaker-embedding sidecar LocaLog ships.
#
# This is a release/developer command and never something the app does at run
# time. It builds sherpa-onnx from a pinned revision, then links our own small
# executable against it, because the shipped binary should be auditable back to a
# known source rather than to whatever was on the build machine.
#
# Statically, so the result carries no dependency on this machine. A dynamically
# linked sidecar runs where it was built and fails on somebody else's computer,
# and a dylib shipped beside an externalBin is another artifact to place, sign and
# notarise.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_dir="$repo_root/src-tauri/binaries"
crate_dir="$repo_root/sidecars/speaker-embedding"
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

# The same pin as the diarisation sidecar, so both trace to one reviewed revision.
actual_commit="$(git -C "$source_dir" rev-parse HEAD 2>/dev/null || true)"
if [[ "$actual_commit" != "$source_commit" ]]; then
  echo "Expected sherpa-onnx commit $source_commit, found ${actual_commit:-unknown}." >&2
  echo "Set SHERPA_ONNX_COMMIT explicitly only when updating the reviewed runtime revision." >&2
  exit 1
fi

# Only what a speaker embedding needs. Every option left on is code that ships,
# is signed, and has to be accounted for.
build_dir="$source_dir/build-localog-embedding-$target_triple"
cmake -S "$source_dir" -B "$build_dir" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DSHERPA_ONNX_ENABLE_BINARY=OFF \
  -DSHERPA_ONNX_ENABLE_C_API=ON \
  -DSHERPA_ONNX_ENABLE_SPEAKER_DIARIZATION=ON \
  -DSHERPA_ONNX_ENABLE_TTS=OFF \
  -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF \
  -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF \
  -DSHERPA_ONNX_BUILD_C_API_EXAMPLES=OFF \
  -DSHERPA_ONNX_ENABLE_PYTHON=OFF
cmake --build "$build_dir" --config Release \
  --parallel "${CMAKE_BUILD_PARALLEL_LEVEL:-2}"

# Gather every archive the build produced into one directory for the linker.
lib_dir="$build_dir/localog-lib"
rm -rf "$lib_dir"
mkdir -p "$lib_dir"
find "$build_dir" -name '*.a' -exec cp {} "$lib_dir/" \;
if [[ ! -f "$lib_dir/libsherpa-onnx-c-api.a" ]]; then
  echo "The sherpa-onnx build produced no static C API library." >&2
  echo "Found: $(ls "$lib_dir" 2>/dev/null | tr '\n' ' ')" >&2
  exit 1
fi

SHERPA_ONNX_LIB="$lib_dir" cargo build --release \
  --manifest-path "$crate_dir/Cargo.toml" \
  --target "$target_triple"

binary="$crate_dir/target/$target_triple/release/localog-speaker-embedding"
if [[ ! -f "$binary" ]]; then
  binary="$crate_dir/target/release/localog-speaker-embedding"
fi
if [[ ! -f "$binary" ]]; then
  echo "The sidecar build completed without producing an executable." >&2
  exit 1
fi

mkdir -p "$sidecar_dir"
destination="$sidecar_dir/localog-speaker-embedding-$target_triple"
if [[ "$target_triple" == *windows* ]]; then
  destination="$destination.exe"
  binary="$binary.exe"
fi
cp "$binary" "$destination"
chmod +x "$destination"
echo "Wrote $destination"

# The point of building statically is that this list stays empty of anything the
# target machine might not have.
if command -v otool >/dev/null 2>&1; then
  external="$(otool -L "$destination" | tail -n +2 |
    grep -vE '/usr/lib/|/System/Library/' || true)"
  if [[ -n "$external" ]]; then
    echo "Warning: the sidecar links libraries outside the system:" >&2
    echo "$external" >&2
    echo "A packaged build would fail on a machine without them." >&2
  fi
fi
