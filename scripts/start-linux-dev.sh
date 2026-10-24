#!/usr/bin/env bash
# Start the real workspace, using the checkout's front-end and native watchers.
set -euo pipefail
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
cd -- "$repo_root"
for tool in npm cargo; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "LocaLog development needs $tool. See docs/LINUX_DEVELOPMENT.md." >&2
    exit 1
  fi
done
if [[ ! -d node_modules ]]; then
  echo 'Run npm ci in the LocaLog checkout first.' >&2
  exit 1
fi
exec npm run tauri dev
