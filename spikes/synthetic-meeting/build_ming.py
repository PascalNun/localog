#!/usr/bin/env python3
"""Render the invented meeting with a verified local Ming model on Apple Silicon.

Run --audition first. Rendering never downloads models or opens app data. Named
synthetic voices, per-turn seeds and hash-keyed checkpoints keep identities and
restarts explicit without introducing a production TTS adapter.
"""

import argparse
import gc
import hashlib
import importlib.metadata
import json
import shutil
import time
import wave
from pathlib import Path

from build import RATE, SPEAKERS, assemble_meeting, digest, frames, read_turns, run
from prepare_ming import MODEL, REVISION

VOICES = {"M": "US_Emma", "J": "US_James", "L": "UK_Charlotte", "T": "UK_George"}
OPTIONS = {"lang_code": "de", "cfg_scale": 2.0, "sigma": 0.25,
           "temperature": 0.0, "ddpm_steps": 10, "max_decode_steps": 512, "stream": True}
AUDITION_TURNS = [0, 1, 2, 3, 8, 9, 10, 11]


def verify_model(path):
    provenance = json.loads((path / "verified-model.json").read_text())
    if provenance["repository"] != MODEL or provenance["revision"] != REVISION:
        raise ValueError("Run prepare_ming.py for the pinned checkpoint first")
    for name, expected in provenance["sha256"].items():
        if digest(path / name) != expected:
            raise ValueError(f"Model file changed: {name}")
    return provenance


def write_audition(out, turns, sources, speakers):
    cursor = 0
    segments = []
    with wave.open(str(out / "audition.wav"), "wb") as audio:
        audio.setparams((1, 2, RATE, 0, "NONE", "not compressed"))
        for turn, source in zip(turns, sources):
            start = cursor
            with wave.open(str(source), "rb") as clip:
                while chunk := clip.readframes(RATE):
                    audio.writeframesraw(chunk)
                cursor += clip.getnframes()
            segments.append({"startMs": round(start * 1000 / RATE),
                             "endMs": round(cursor * 1000 / RATE), **turn})
            audio.writeframesraw(b"\0" * RATE)  # half a second between voices
            cursor += RATE // 2
    (out / "audition-reference.json").write_text(json.dumps({"speakers": speakers, "segments": segments},
                                                           ensure_ascii=False, indent=2) + "\n")
    print(f"Audition: {out / 'audition.wav'} ({cursor / RATE:.1f} s)", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--script", type=Path, default=Path(__file__).with_name("meeting-de.txt"))
    parser.add_argument("--audition", action="store_true", help="Render two turns per speaker only")
    parser.add_argument("--seconds", type=int, default=1800)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()
    args.out = args.out.resolve()
    if (args.out / "manifest.json").exists():
        parser.error("A completed fixture exists here; choose a new output directory")
    args.out.mkdir(parents=True, exist_ok=True)
    parts = args.out / "parts"
    parts.mkdir(exist_ok=True)
    provenance = verify_model(args.model)
    turns = read_turns(args.script)
    indices = AUDITION_TURNS if args.audition else list(range(len(turns)))
    speakers = {key: {"name": value["name"], "role": value["role"], "voice": VOICES[key]}
                for key, value in SPEAKERS.items()}
    packages = {name: importlib.metadata.version(name) for name in
                ["mlx-audio", "mlx", "mlx-metal", "transformers", "numpy", "soundfile"]}
    runtime_source = Path(importlib.metadata.distribution("mlx-audio").locate_file(
        "mlx_audio/tts/models/bailingmm/bailingmm.py"))
    identity = {"modelRevision": REVISION, "modelSha256": provenance["sha256"],
                "packages": packages, "runtimeSourceSha256": digest(runtime_source),
                "rendererSha256": digest(Path(__file__)),
                "options": OPTIONS, "seed": args.seed}
    existing_identity = args.out / "render-settings.json"
    if existing_identity.exists() and json.loads(existing_identity.read_text()) != identity:
        parser.error("Settings changed; use a new output directory rather than mixing renders")
    existing_identity.write_text(json.dumps(identity, indent=2) + "\n")

    import mlx.core as mx
    import numpy as np
    import soundfile as sf
    from mlx_audio.tts.utils import load_model

    if not mx.metal.is_available():
        parser.error("This evaluation requires a working Apple Silicon Metal device")
    # Bound caches rather than raising the system's GPU wired-memory allowance.
    mx.set_cache_limit(256 * 1024**2)
    mx.set_memory_limit(6 * 1024**3)
    print("Loading the verified local Ming checkpoint", flush=True)
    model = load_model(args.model.resolve(), strict=True)
    source_files = []
    render_log = []
    for position, index in enumerate(indices):
        turn = turns[index]
        if shutil.disk_usage(args.out).free < 3 * 1024**3:
            raise RuntimeError("Stopping with checkpoints intact: less than 3 GiB disk space remains")
        seed = args.seed + index
        key = hashlib.sha256(json.dumps([identity, turn, VOICES[turn["speaker"]], seed],
                                        ensure_ascii=False, sort_keys=True).encode()).hexdigest()[:16]
        source = parts / f"{index:03}-{key}.wav"
        metadata = source.with_suffix(".json")
        if source.exists() and metadata.exists():
            record = json.loads(metadata.read_text())
            if digest(source) != record["pcmSha256"]:
                raise ValueError(f"Checkpoint checksum mismatch: turn {index + 1}")
        else:
            print(f"Rendering {position + 1}/{len(indices)} (script turn {index + 1}): {speakers[turn['speaker']]['name']}", flush=True)
            mx.random.seed(seed)
            started = time.monotonic()
            native = parts / f"{index:03}-{key}-native.wav"
            with sf.SoundFile(native, "w", samplerate=model.sample_rate, channels=1, subtype="PCM_16") as clip:
                for fragment, result in enumerate(model.generate(text=turn["text"],
                        voice=VOICES[turn["speaker"]], **OPTIONS), 1):
                    samples = np.asarray(result.audio)
                    if not samples.size or not np.isfinite(samples).all():
                        raise ValueError("The model produced empty or non-finite audio")
                    clip.write(samples)
                    print(f"  fragment {fragment}: {len(samples) / model.sample_rate:.1f} s audio", flush=True)
                    del samples, result
            elapsed = time.monotonic() - started
            # Resample for the exact same diarisation input format as the first
            # fixture. Keep native samples too, for listening at full bandwidth.
            trim = "silenceremove=start_periods=1:start_threshold=-48dB"
            temporary = source.with_suffix(".tmp.wav")
            run(["ffmpeg", "-nostdin", "-v", "error", "-y", "-i", native,
                 "-af", f"{trim},areverse,{trim},areverse", "-ar", RATE,
                 "-ac", 1, "-c:a", "pcm_s16le", temporary])
            duration = frames(temporary) / RATE
            if not 0.2 <= duration <= max(8, len(turn["text"].split()) * 1.5):
                raise ValueError(f"Implausible duration for turn {index + 1}: {duration:.1f} s")
            temporary.replace(source)
            record = {"index": index, "speaker": turn["speaker"], "voice": VOICES[turn["speaker"]],
                      "seed": seed, "seconds": elapsed, "audioSeconds": duration,
                      "pcmSha256": digest(source), "nativeSha256": digest(native),
                      "peakMlxBytes": mx.get_peak_memory()}
            metadata.write_text(json.dumps(record, indent=2) + "\n")
            print(f"  done: {duration:.1f} s speech in {elapsed:.1f} s", flush=True)
            gc.collect()
            mx.clear_cache()
        render_log.append(record)
        source_files.append(source)
    (args.out / "render-log.json").write_text(json.dumps(render_log, indent=2) + "\n")
    audition_sources = source_files if args.audition else [source_files[index] for index in AUDITION_TURNS]
    write_audition(args.out, [turns[index] for index in AUDITION_TURNS], audition_sources, speakers)
    if args.audition:
        return
    assemble_meeting(turns, source_files, args.out, args.seconds, speakers, {
        "synthesis": "Ming-3.0-tts-0.5B via MLX-Audio", "model": provenance,
        "packages": packages, "resolvedSettings": OPTIONS, "baseSeed": args.seed,
        "renderSeconds": sum(record["seconds"] for record in render_log),
        "peakMlxBytes": max(record["peakMlxBytes"] for record in render_log),
        "scriptSha256": digest(args.script), "generatorSha256": digest(Path(__file__)),
        "limitations": ["Synthetic TTS, not human speech", "No overlap or microphone noise",
                        "Reference windows include sentence pauses; not speech-activity ground truth",
                        "English preset voice identities speaking German; native accent is not guaranteed",
                        "Naturalness needs human listening; ASR and clustering do not establish it"],
    })


if __name__ == "__main__":
    main()
