# Developing on Linux

The browser preview supports interface and workflow work without an AI model. The
native application also exercises real SQLite storage, file dialogs and background
jobs. Model downloads remain explicit; starting either shell does not need one.

## Prerequisites

Install Node.js 22.12 or newer, npm and Rust through rustup. The repository's
`rust-toolchain.toml` selects the Rust version, rustfmt and Clippy for the host
platform; an Apple target is not required on Linux.

On Arch Linux / Omarchy, install the desktop build dependencies:

```sh
omarchy pkg add base-devel webkit2gtk-4.1 openssl librsvg pkgconf libpulse gst-plugins-base gst-plugins-good
```

On Debian / Ubuntu, use the desktop packages listed in
`.github/workflows/linux.yml`. Tauri 2 requires WebKitGTK **4.1**, not 4.0.
Audio playback also requires `gstreamer1.0-plugins-base` and
`gstreamer1.0-plugins-good` (included as dependencies of the Debian bundle).
These are needed by WebKit's audio player even though import/transcription use
the separate FFmpeg sidecar. Without `autoaudiosink`, WebKit can abort when the
transcript audio player opens, leaving a blank native window. On Arch, verify
`gst-inspect-1.0 autoaudiosink` and `gst-inspect-1.0 pulsesink` both succeed before
testing playback. After installing missing plugins, restart the native app.

## Run the application

From the repository root:

```sh
npm ci
npm run dev
```

Open `http://127.0.0.1:1420` for the synthetic browser preview. Stop that server
before starting the native app, which starts its own server on the same port:

```sh
npm run tauri dev
```

On a memory-constrained machine, limit Rust compilation and omit debug symbols:

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 npm run tauri dev
```

The development shell and Rust tests do not require bundled sidecars. Missing
runtimes are reported by the relevant feature. System FFmpeg/ffprobe are accepted
for media preparation; transcription, generation and speaker separation need their
own runtimes and models before those operations can work.

The frontend watcher excludes the native source/build tree and Rust build outputs
under sidecars and spikes. Tauri handles native source changes itself, so native
compilation does not need to occupy Vite's watcher or reload the browser preview.

Linux recording requires a running PulseAudio server or PipeWire's PulseAudio
compatibility server. The recorder uses [libpulse-binding](https://docs.rs/libpulse-binding/)
to capture the default output's monitor and the default microphone independently.
`PULSE_SINK` and `PULSE_SOURCE` can override those sources for a single process.
It does not select an unrelated monitor merely because it appears first.

The explicit synthetic recorder check needs `pactl` and `paplay` (Arch:
`libpulse`). It creates temporary virtual outputs, plays generated tones only
into them, checks the captured tracks and removes the outputs afterwards. It
does not change the desktop defaults or open the physical microphone:

```sh
cargo build --locked --manifest-path sidecars/record-meeting/Cargo.toml
python scripts/test-linux-recorder.py
```

Native workspace data is stored under the platform's app-data directory for
`app.localog.desktop` (normally `~/.local/share/app.localog.desktop`). Settings
can open the actual workspace location. Keep its recordings, database and models
out of the checkout and out of version control.

## Checks and packaging

Run the checks in [CONTRIBUTING.md](../CONTRIBUTING.md). Rust checks can run before
building any sidecar:

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Packaging additionally needs CMake, Ninja, NASM, patchelf, libpulse headers, and a
**static** Opus library. Arch's normal `opus` package does not provide `libopus.a`;
check `$(pkg-config --variable=libdir opus)/libopus.a` before building FFmpeg.
Do not substitute a shared library into a release intended to be self-contained.

```sh
CARGO_BUILD_JOBS=1 CMAKE_BUILD_PARALLEL_LEVEL=1 MAKE_PARALLEL_LEVEL=1 npm run tauri:build:linux
```

This builds the sidecars and explicitly loads `tauri.linux-bundle.conf.json` to
produce Linux packages. Keeping it separate from Tauri's automatically loaded
`tauri.linux.conf.json` lets normal development work before those binaries exist.

Compiling a package does not establish recording, playback, printing or model
quality on a desktop. Validate those separately with synthetic material and record
any untested paths in the current plan.

## Window frame

`tauri.linux.conf.json` is the automatically merged Linux window override for both
development and release builds. It removes the separate OS title bar. Compact
app-owned controls provide minimize, maximize/restore and close, with a draggable
top region and resize edges for floating desktops. The release-only sidecar and
packaging configuration remains in `tauri.linux-bundle.conf.json`.

Window-frame validation: native Linux launch, light/dark screenshots at 1280px
and 800px, keyboard focus and simulated Tauri calls for all three controls,
dragging and corner resizing. Actual window-manager behavior on GNOME/KDE has
not been checked on this Hyprland machine.

## CPU-compatible generation engine

A real Ministral 3B load on the Surface exposed a limit of the imported release
sidecar: `--version` worked, but the CPU backend raised SIGILL while loading the
model. GDB stopped on the GPR-to-vector `vpbroadcastq` instruction, an AVX-512 form
unavailable on this AVX2 CPU. This was not a model checksum or out-of-memory fault.

`build-llama-sidecar.sh` now sets `GGML_NATIVE=OFF`, so a release build does not
inherit newer instructions from its build host. The pinned x86 build keeps AVX2,
FMA and F16C enabled; this does not promise support for pre-AVX2 machines. It also
sets `LLAMA_BUILD_UI=OFF`: LocaLog supplies the interface, so the inference sidecar
does not need to build or fetch the server's separate web UI.

Validated on 22 October 2026 by building the exact pinned source on the Surface
and loading the verified Ministral 3B Q4_K_M file with the same 16,384-token
context setting used by LocaLog. The server became ready after 17.7 seconds and
returned `BEREIT.` for a synthetic German prompt. This is a load/completion smoke
test, not a professional-protocol quality or long-meeting benchmark.

## Tiled windows

The Linux window minimum is 360 × 320 logical pixels. A larger native minimum
can exceed a tiler's allocation (for example, a 1354 × 431 tile), causing the
compositor to compress the surface instead of letting CSS adapt. Short windows
also use tighter vertical spacing; the workspace and navigation scroll. Narrow
windows keep the existing overlay navigation and single-column layouts.

## Transcript playback

WebKitGTK can reject media through Tauri's `asset:` scheme even with the audio
plugins installed ([upstream issue](https://github.com/tauri-apps/tauri/issues/3725)).
Linux therefore streams working WAV files through an app-owned IPv4 loopback
listener. Only repository-resolved files allowed by the existing asset scope are
registered, behind opaque URLs. GET/HEAD and byte ranges support playback and
seeking with disk streaming rather than recording-sized frontend buffers. The
listener exists only for the app process; no login service is installed.
