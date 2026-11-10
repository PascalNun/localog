#!/usr/bin/env python3
"""Explicit, pinned download for the local Ming evaluation, not app setup."""

import argparse
import json
import shutil
from pathlib import Path

from build import digest

MODEL = "mlx-community/Ming-3.0-tts-0.5B-bf16"
REVISION = "283b3c342c6fb69a7a87832c97f9799567efe05d"
WEIGHTS = {
    "model.safetensors": "16a7086c992a6eab7d855df90c9c2edbb0f4f3a702b3cd697f6011e198d0c8b4",
    "campplus.safetensors": "a479586bd8f8f96cdb414eaa2fa07c6cd54d6d1636562e2d75a0211cb3b0a6fd",
    "tokenizer.json": "87d54ca3b44ea2f822ba39a0a45cb1b61644bd6606d9d3536dea75367bffb06d",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    # Leave room for the OS, swap and the recording; never fill a nearly full disk.
    if not (args.out / "model.safetensors").exists() and shutil.disk_usage(args.out).free < 6 * 1024**3:
        parser.error("At least 6 GiB free is required before downloading the 3 GB model")
    from huggingface_hub import snapshot_download

    snapshot_download(MODEL, revision=REVISION, local_dir=args.out, max_workers=2,
                      allow_patterns=["*.json", "*.safetensors", "*.txt", "LICENSE", "README.md"])
    checksums = {}
    for name, expected in WEIGHTS.items():
        actual = digest(args.out / name)
        if actual != expected:
            raise ValueError(f"Checksum mismatch: {name}")
        checksums[name] = actual
    for path in sorted(args.out.iterdir()):
        if path.is_file() and path.name != "verified-model.json" and path.name not in checksums:
            checksums[path.name] = digest(path)
    (args.out / "verified-model.json").write_text(json.dumps({
        "repository": MODEL, "revision": REVISION, "sha256": checksums,
    }, indent=2) + "\n")
    print(f"Verified local model: {args.out.resolve()}", flush=True)


if __name__ == "__main__":
    main()
