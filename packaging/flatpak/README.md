# Experimental Flatpak packaging

This path is for testing, not a Flathub release. The normal Linux job still builds
Debian and AppImage packages. Its manual **flatpak** input additionally repackages
the same Debian artifact into a single-file Flatpak, separately for x86_64 and
arm64. A successful package build is not a desktop workflow validation.

The Debian bundle is built on Ubuntu 22.04, including all seven pinned sidecars.
The Flatpak uses GNOME 49 for GTK 3, WebKitGTK 4.1 and desktop audio dependencies.
Do not repackage a binary built on a newer host without checking its required ABI
against that runtime. This binary-based approach follows the [Tauri Flatpak
guide](https://v2.tauri.app/distribute/flatpak/); a Flathub submission and fully
source-based offline build remain separate work.

## Build locally

Install `flatpak`, `flatpak-builder`, `ar`, `bsdtar`, and `appstreamcli` through
your distribution. Build a complete Linux Debian bundle first; see
[Linux development](../../docs/LINUX_DEVELOPMENT.md). Models are not bundled.

```sh
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.gnome.Platform//49 org.gnome.Sdk//49
work=$(mktemp -d)
python3 scripts/prepare-flatpak.py path/to/localog.deb "$work/input"
appstreamcli validate --no-net "$work/input/io.github.PascalNun.LocaLog.metainfo.xml"
flatpak-builder --user --repo="$work/repo" "$work/build" "$work/input/io.github.PascalNun.LocaLog.json"
flatpak build-bundle "$work/repo" "$work/LocaLog.flatpak" io.github.PascalNun.LocaLog --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo
```

The preparation command refuses an incomplete bundle or an existing staging
folder. It copies inputs; it does not modify the Debian package or install an app.
`python3 scripts/test-linux-packaging.py` checks this with synthetic archives.

## Install, launch and update a test build

```sh
flatpak install --user --bundle --or-update path/to/LocaLog.flatpak
flatpak run io.github.PascalNun.LocaLog
```

For a newer test bundle, repeat the install command. There is no hosted LocaLog
Flatpak repository or automatic application-update channel yet. The GNOME runtime
comes from Flathub and can receive updates through Flatpak normally.

Flatpak has a separate workspace under
`~/.var/app/io.github.PascalNun.LocaLog/data/app.localog.desktop`. The existing
checkout's workspace stays in `~/.local/share/app.localog.desktop`. Use LocaLog's
explicit backup/restore controls to move work after validating a build; do not
silently copy databases, models or recordings during installation. Removing a
launcher or package must not silently delete either workspace.

## Permissions and release gate

The manifest grants Wayland with fallback X11, graphics and shared IPC for desktop
rendering, PulseAudio for playback/recording, and network for consented model
downloads plus loopback AI/audio connections. It forces GTK portal file selection
and grants no blanket home/host filesystem access, host process execution, or
unrestricted D-Bus access. Network permission does not restrict the app to
loopback; core meeting processing must retain its existing local-only behavior.
See [Flatpak permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html).

Before recommending this package, validate on GNOME and KDE, under Wayland and
X11, including an xdg-desktop-portal implementation:

- launch with no developer checkout, Rust, Node, Ollama, or system FFmpeg;
- import a user-selected recording through the portal; restart and retain access
  to the managed copy; cancel and retry work;
- download a small verified model, transcribe, generate, review and export;
- play and seek transcript audio through WebKit and the scoped loopback server;
- record microphone/output with PulseAudio and PipeWire-Pulse, explicitly checking
  monitor-source access rather than assuming that playback grants working capture;
- select export/backup/restore locations through portals and reopen the workspace;
- detect missing shared libraries for the main app and every sidecar; load an
  actual model on the minimum supported CPU, not only `--version`;
- update the bundle without losing workspace data; check light/dark and tiled views.

The x86 inference build requires AVX2/FMA/F16C. A universal Linux package does not
make unsupported CPUs or machines without enough memory suitable for every model.
The Surface's native development success does not establish Flatpak success.
