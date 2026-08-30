#!/usr/bin/env bash
set -euo pipefail

# Build the media tools LocaLog ships, small rather than stock.
#
# The application uses FFmpeg for three things: read what a file is, turn anything
# into 16 kHz mono PCM, and store a recording as Opus. A stock build is tens of
# megabytes of encoders, filters, muxers and network protocols that are never
# called. Configured down to what is used it is a few megabytes, which is less to
# ship, far less to audit, and stops carrying advisories for code no execution ever
# reaches.
#
# It also settles the licence by construction rather than by argument. Nothing here
# enables the GPL-only components, so this build is LGPL-2.1-or-later, and LocaLog
# invokes it as a separate executable rather than linking it. What remains owed is
# ordinary and is handled by `--enable-version3` plus shipping the licence texts:
# be able to supply the source for this exact build, which the pinned revision and
# the configure line below together describe.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sidecar_dir="$repo_root/src-tauri/binaries"
source_dir="${FFMPEG_SOURCE_DIR:-}"
source_ref="${FFMPEG_REF:-n7.1.1}"
target_triple="${TAURI_TARGET_TRIPLE:-$(rustc -vV | sed -n 's/^host: //p')}"

if [[ -z "$target_triple" ]]; then
  echo "Could not determine the Rust target triple. Set TAURI_TARGET_TRIPLE." >&2
  exit 1
fi

# Windows takes a prebuilt rather than building from source.
#
# The configure-and-make below wants a Unix toolchain: MSYS2, a compiler it
# recognises, and a static libopus it can find. None of that is on a Windows
# machine by default, and standing it up would be more moving parts than the
# thing it produces.
#
# What is given up by taking somebody else's build is real and worth naming. The
# source build here is deliberately minimal — `--disable-everything` and then only
# the demuxers, decoders and encoders this application actually calls — which is a
# few megabytes and a small surface to audit. A general build is neither. It also
# means Windows runs a different FFmpeg version from macOS and Linux, because no
# 7.x Windows build is published; the same recording will be decoded by 8.1 there
# and 7.1.1 elsewhere.
#
# The LGPL build rather than the GPL one, matching `--disable-gpl` below: the same
# licence position on every platform is worth more than the extra codecs, none of
# which this application asks for.
#
# Pinned by release tag and verified by checksum, because this is somebody else's
# binary going inside something people are asked to trust.
if [[ "$target_triple" == *windows* ]]; then
  release="${FFMPEG_WINDOWS_RELEASE:-autobuild-2026-08-30-13-12}"
  archive="${FFMPEG_WINDOWS_ARCHIVE:-ffmpeg-n8.1.2-50-g1a748fe2cd-win64-lgpl-8.1}"
  expected="${FFMPEG_WINDOWS_SHA256:-ca713b37c6fcbc94df1bf0409a1c89c04435e8a49fce0df85e75cf08ce3f904b}"
  url="https://github.com/BtbN/FFmpeg-Builds/releases/download/$release/$archive.zip"

  work="$(mktemp -d)"
  trap 'rm -rf "$work"' EXIT
  echo "Fetching a prebuilt FFmpeg for Windows ($archive)…"
  curl --fail --location --silent --show-error --output "$work/ffmpeg.zip" "$url"

  actual="$(powershell -NoProfile -Command \
    "(Get-FileHash -Algorithm SHA256 '$(cygpath -w "$work/ffmpeg.zip" 2>/dev/null || echo "$work/ffmpeg.zip")').Hash.ToLower()" |
    tr -d '\r')"
  if [[ "$actual" != "$expected" ]]; then
    echo "The prebuilt FFmpeg did not match its checksum." >&2
    echo "  expected $expected" >&2
    echo "  actual   $actual" >&2
    echo "Refusing to ship a binary that is not the one that was reviewed." >&2
    exit 1
  fi

  powershell -NoProfile -Command \
    "Expand-Archive -Path '$(cygpath -w "$work/ffmpeg.zip" 2>/dev/null || echo "$work/ffmpeg.zip")' -DestinationPath '$(cygpath -w "$work" 2>/dev/null || echo "$work")' -Force"

  mkdir -p "$sidecar_dir/licences"
  for tool in ffmpeg ffprobe; do
    built="$work/$archive/bin/$tool.exe"
    if [[ ! -f "$built" ]]; then
      echo "The prebuilt archive did not contain $tool.exe." >&2
      exit 1
    fi
    cp "$built" "$sidecar_dir/localog-$tool-$target_triple.exe"
    echo "Wrote $sidecar_dir/localog-$tool-$target_triple.exe"
  done
  # The obligation that comes with shipping it, met the same way the source build
  # meets it: the licence travels with the binary.
  if [[ -f "$work/$archive/LICENSE.txt" ]]; then
    cp "$work/$archive/LICENSE.txt" "$sidecar_dir/licences/ffmpeg-LICENSE.txt"
  fi
  echo "FFmpeg $archive, prebuilt (LGPL), from $url" > "$sidecar_dir/licences/ffmpeg-BUILD.txt"
  exit 0
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
  source_dir="$temporary_source/ffmpeg"
  echo "Cloning FFmpeg $source_ref into a temporary build directory…"
  git clone --depth 1 --branch "$source_ref" \
    https://git.ffmpeg.org/ffmpeg.git "$source_dir"
fi

if [[ ! -f "$source_dir/configure" ]]; then
  echo "FFMPEG_SOURCE_DIR must point to an FFmpeg source checkout." >&2
  exit 1
fi

build_dir="$source_dir/build-localog-$target_triple"
mkdir -p "$build_dir"
cd "$build_dir"

# What a meeting recording can plausibly arrive as, and nothing else. Adding a
# format here is a deliberate act: it is code that ships and is signed.
demuxers="mov,matroska,wav,mp3,ogg,flac,aiff,aac,m4v,asf,avi,caf,w64"
decoders="aac,aac_latm,mp3,mp3float,flac,vorbis,opus,pcm_s16le,pcm_s16be,pcm_s24le,pcm_f32le,pcm_alaw,pcm_mulaw,alac,ac3,wmav2"
encoders="pcm_s16le,libopus"
muxers="wav,opus,ogg"

# Statically, so the result carries no dependency on the machine that built it.
#
# `--pkg-config-flags=--static` is not enough on its own and was measured not to
# be: a package manager ships libopus as both a dylib and an archive in one
# directory, and the linker takes the dylib every time. The reliable fix is to put
# a directory holding only the archive first on the search path, because the
# linker walks -L directories in order and takes the first match it finds.
opus_archive="$(pkg-config --variable=libdir opus)/libopus.a"
if [[ ! -f "$opus_archive" ]]; then
  echo "No static libopus at $opus_archive." >&2
  echo "Install one, or the sidecar will depend on a library the target may not have." >&2
  exit 1
fi
static_first="$build_dir/static-first"
mkdir -p "$static_first"
cp "$opus_archive" "$static_first/"

../configure \
  --prefix="$build_dir/out" \
  --pkg-config-flags=--static \
  --extra-ldflags="-L$static_first" \
  --disable-everything \
  --disable-doc \
  --disable-htmlpages --disable-manpages --disable-podpages --disable-txtpages \
  --disable-network \
  --disable-autodetect \
  --disable-programs \
  --enable-ffmpeg --enable-ffprobe \
  --disable-gpl --disable-nonfree \
  --enable-version3 \
  --enable-libopus \
  --enable-protocol=file,pipe \
  --enable-demuxer="$demuxers" \
  --enable-decoder="$decoders" \
  --enable-encoder="$encoders" \
  --enable-muxer="$muxers" \
  --enable-parser=aac,mpegaudio,flac,vorbis,opus,ac3 \
  --enable-filter=aresample,aformat,anull,atrim,aselect,anullsrc,concat,volume,amix \
  --enable-bsf=null

make -j"${MAKE_PARALLEL_LEVEL:-4}"

mkdir -p "$sidecar_dir"
for tool in ffmpeg ffprobe; do
  built="$build_dir/$tool"
  if [[ ! -x "$built" ]]; then
    echo "The FFmpeg build completed without producing $tool." >&2
    exit 1
  fi
  destination="$sidecar_dir/localog-$tool-$target_triple"
  if [[ "$target_triple" == *windows* ]]; then
    destination="$destination.exe"
    built="$built.exe"
  fi
  cp "$built" "$destination"
  chmod +x "$destination"
  echo "Wrote $destination ($(du -h "$destination" | cut -f1))"
done

# The licence texts travel with the binaries, which is most of what is owed for
# shipping somebody else's LGPL software.
mkdir -p "$sidecar_dir/licences"
for file in COPYING.LGPLv2.1 COPYING.LGPLv3 LICENSE.md CREDITS; do
  if [[ -f "$source_dir/$file" ]]; then
    cp "$source_dir/$file" "$sidecar_dir/licences/ffmpeg-$file"
  fi
done
echo "FFmpeg $source_ref, configured as above" > "$sidecar_dir/licences/ffmpeg-BUILD.txt"
echo "Licence texts in $sidecar_dir/licences"

if command -v otool >/dev/null 2>&1; then
  for tool in ffmpeg ffprobe; do
    binary="$sidecar_dir/localog-$tool-$target_triple"
    external="$(otool -L "$binary" | tail -n +2 |
      grep -vE '/usr/lib/|/System/Library/' || true)"
    if [[ -n "$external" ]]; then
      echo "Warning: localog-$tool links libraries outside the system:" >&2
      echo "$external" >&2
      echo "A packaged build would fail on a machine without them." >&2
    fi
  done
fi
