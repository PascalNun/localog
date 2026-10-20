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
omarchy pkg add base-devel webkit2gtk-4.1 openssl librsvg pkgconf libpulse
```

On Debian / Ubuntu, use the desktop packages listed in
`.github/workflows/linux.yml`. Tauri 2 requires WebKitGTK **4.1**, not 4.0.

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
