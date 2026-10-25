#!/usr/bin/env python3
"""Install a locally built standalone test app and its sidecars in the user's home."""
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parent.parent


def main():
    if sys.platform != 'linux':
        raise SystemExit('This installer is for Linux.')
    binary = ROOT / 'src-tauri/target/debug/localog'
    if not binary.is_file():
        raise SystemExit('Build first: npx tauri build --debug --no-bundle --config src-tauri/tauri.linux-bundle.conf.json')
    target = next(line.removeprefix('host: ') for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
    import json
    config = json.loads((ROOT / 'src-tauri/tauri.linux-bundle.conf.json').read_text())
    sources = {}
    for logical in config['bundle']['externalBin']:
        source = ROOT / 'src-tauri' / (logical + '-' + target)
        if not source.is_file():
            source = ROOT / 'src-tauri' / logical
        if not source.is_file() or not os.access(source, os.X_OK):
            raise SystemExit(f'Missing executable {logical}; build the sidecars first.')
        sources[Path(logical).name] = source
    parent = Path.home() / '.local/opt'
    parent.mkdir(parents=True, exist_ok=True)
    destination = parent / 'localog-testing'
    # Copy fully before replacing this install. Keep the previous install for rollback.
    with tempfile.TemporaryDirectory(prefix='.localog-install-', dir=parent) as temporary:
        staged = Path(temporary) / 'app'
        staged.mkdir()
        shutil.copy2(binary, staged / 'localog')
        for name, source in sources.items():
            shutil.copy2(source, staged / name)
        shutil.copytree(ROOT / 'src-tauri/binaries/licences', staged / 'licences')
        shutil.copy2(ROOT / 'LICENSE', staged / 'LICENSE')
        shutil.copy2(ROOT / 'src-tauri/icons/128x128.png', staged / 'icon.png')
        previous = parent / 'localog-testing-previous'
        if destination.exists():
            if previous.exists():
                shutil.rmtree(previous)
            destination.rename(previous)
        staged.rename(destination)
    launcher = Path.home() / '.local/bin/localog-test'
    launcher.parent.mkdir(parents=True, exist_ok=True)
    launcher.write_text('#!/usr/bin/env bash\nexec ' + shlex.quote(str(destination / 'localog')) + ' "$@"\n')
    launcher.chmod(0o755)
    data = Path(os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share')))
    desktop = data / 'applications/localog-testing.desktop'
    desktop.parent.mkdir(parents=True, exist_ok=True)
    # Reuse the desktop Exec quoting boundary, including literal percent signs.
    import importlib.util
    spec = importlib.util.spec_from_file_location('launcher_installer', ROOT / 'scripts/install-linux-dev-launcher.py')
    if spec is None or spec.loader is None:
        raise SystemExit('Could not load the launcher installer.')
    installer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(installer)
    desktop.write_text('\n'.join([
        '[Desktop Entry]', 'Type=Application', 'Name=LocaLog (Test Build)',
        'Comment=Standalone local test build; no development server required',
        'Exec=' + installer.desktop_quote(str(launcher)), 'Icon=' + str(destination / 'icon.png'),
        'Terminal=false', 'Categories=Office;', 'StartupNotify=true', '',
    ]))
    print(f'Installed {destination}\nOpen LocaLog (Test Build), or run {launcher}.')


if __name__ == '__main__':
    main()
