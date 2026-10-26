#!/usr/bin/env python3
"""Install a per-user launcher for this checkout; no runtime or model installation."""
import argparse
import os
import shlex
from pathlib import Path


def desktop_quote(value):
    # Desktop Exec is not a shell command. Literal percent signs must also be escaped.
    return '"' + value.replace('\\', '\\\\').replace('"', '\\"').replace('`', '\\`').replace('$', '\\$').replace('%', '%%') + '"'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bin-dir', type=Path, default=Path.home() / '.local/bin')
    parser.add_argument('--data-dir', type=Path, default=Path(os.environ.get('XDG_DATA_HOME', Path.home() / '.local/share')))
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    binary = args.bin_dir.expanduser().resolve() / 'localog-dev'
    desktop = args.data_dir.expanduser().resolve() / 'applications/localog-development.desktop'
    binary.parent.mkdir(parents=True, exist_ok=True)
    desktop.parent.mkdir(parents=True, exist_ok=True)
    binary.write_text('#!/usr/bin/env bash\nexec bash ' + shlex.quote(str(repo / 'scripts/start-linux-dev.sh')) + ' "$@"\n')
    binary.chmod(0o755)
    desktop.write_text('\n'.join([
        '[Desktop Entry]', 'Type=Application', 'Name=LocaLog (Development)',
        'Comment=Test the local LocaLog checkout with your saved workspace',
        'Exec=' + desktop_quote(str(binary)),
        'Icon=' + str(repo / 'src-tauri/icons/128x128.png'),
        'Terminal=true', 'Categories=Office;', 'StartupNotify=false', '',
    ]))
    print(f'Installed {binary}\nInstalled {desktop}\nOpen LocaLog (Development) in your application launcher.')


if __name__ == '__main__':
    main()
