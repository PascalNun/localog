#!/usr/bin/env python3
"""Stage a Flatpak manifest from the Debian bundle built by the Linux release job."""
import argparse
from pathlib import Path
import shutil
import subprocess

APP_ID = 'io.github.PascalNun.LocaLog'
TOOLS = ['record-meeting', 'whisper', 'llama-server', 'speaker-diarization', 'speaker-embedding', 'ffmpeg', 'ffprobe']


def prepare(package, output):
    if not package.is_file() or package.suffix != '.deb':
        raise ValueError('Supply the .deb produced by the LocaLog Linux build.')
    if output.exists():
        raise ValueError('Choose a new staging directory; existing files will not be overwritten.')
    # Verify that the archive includes the complete runtime set before staging it.
    # bsdtar can read the ar container and the compressed data tar on Linux/macOS.
    members = subprocess.check_output(['ar', 't', str(package)], text=True).splitlines()
    data = [name for name in members if name.startswith('data.tar.')]
    if len(data) != 1:
        raise ValueError('The Debian bundle must contain exactly one data archive.')
    payload = subprocess.run(['ar', 'p', str(package), data[0]], stdout=subprocess.PIPE, check=True).stdout
    entries = subprocess.run(['bsdtar', '-tf', '-'], input=payload, stdout=subprocess.PIPE, check=True).stdout.decode().splitlines()
    entries = {entry.removeprefix('./') for entry in entries}
    expected = {'usr/bin/localog'} | {f'usr/bin/localog-{tool}' for tool in TOOLS}
    missing = expected - entries
    if missing:
        raise ValueError('Incomplete bundle; missing: ' + ', '.join(sorted(missing)))
    source = Path(__file__).resolve().parent.parent / 'packaging/flatpak'
    output.mkdir(parents=True)
    for path in source.iterdir():
        if path.is_file():
            shutil.copy2(path, output / path.name)
    shutil.copy2(package, output / 'localog.deb')
    shutil.copy2(source.parent.parent / 'src-tauri/icons/128x128.png', output / 'localog.png')
    return output / f'{APP_ID}.json'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    try:
        manifest = prepare(args.package.resolve(), args.output.resolve())
    except (ValueError, subprocess.CalledProcessError, FileNotFoundError) as error:
        parser.exit(1, f'{error}\n')
    print(manifest)


if __name__ == '__main__':
    main()
