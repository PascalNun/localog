#!/usr/bin/env python3
"""Build a local, timed four-voice meeting with macOS speech and FFmpeg.

No downloads or network calls. Speech audio is deliberately not published: the
installed Apple voices retain their own terms. The invented script is shareable.
"""

import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import wave
from pathlib import Path

RATE = 16_000
SPEAKERS = {
    "M": {"name": "Mara", "role": "Moderation", "voice": "Anna (Premium)", "rate": 170},
    "J": {"name": "Jonas", "role": "Bauplanung", "voice": "Eddy (German (Germany))", "rate": 176},
    "L": {"name": "Lea", "role": "Budget", "voice": "Sandy (German (Germany))", "rate": 168},
    "T": {"name": "Timur", "role": "Zugänglichkeit", "voice": "Reed (German (Germany))", "rate": 173},
}


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read_turns(path):
    turns = []
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        speaker, text = line.split("|", 1)
        if speaker not in SPEAKERS or not text.strip():
            raise ValueError(f"Invalid turn on line {number}")
        turns.append({"speaker": speaker, "text": text.strip()})
    if {turn["speaker"] for turn in turns} != set(SPEAKERS):
        raise ValueError("The fixture must exercise all four voices")
    return turns


def frames(path):
    with wave.open(str(path), "rb") as audio:
        if (audio.getnchannels(), audio.getsampwidth(), audio.getframerate()) != (1, 2, RATE):
            raise ValueError("Expected PCM16 mono 16 kHz")
        return audio.getnframes()


def run(arguments):
    subprocess.run([str(arg) for arg in arguments], check=True, stdout=subprocess.DEVNULL)


def stamp(milliseconds):
    seconds = milliseconds // 1000
    return f"{seconds // 60:02}:{seconds % 60:02}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--seconds", type=int, default=1800)
    parser.add_argument("--script", type=Path, default=Path(__file__).with_name("meeting-de.txt"))
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("Rendering uses installed macOS voices; scoring is platform independent")
    if not shutil.which("ffmpeg") or args.seconds < 1:
        parser.error("FFmpeg and a positive duration are required")
    args.out = args.out.resolve()
    manifest_path = args.out / "manifest.json"
    if manifest_path.exists():
        parser.error("A completed fixture exists here; choose a new output directory")
    turns = read_turns(args.script)
    parts = args.out / "parts"
    parts.mkdir(parents=True, exist_ok=True)

    # Hash-keyed intermediates permit restarting an interrupted synthesis without
    # confusing a changed script, voice or speaking rate with an earlier render.
    source_files = []
    for index, turn in enumerate(turns):
        speaker = SPEAKERS[turn["speaker"]]
        identity = json.dumps([turn, speaker, platform.platform()], ensure_ascii=False)
        key = hashlib.sha256(identity.encode()).hexdigest()[:16]
        source = parts / f"{index:03}-{key}.wav"
        if not source.exists():
            aiff = source.with_suffix(".aiff")
            run(["/usr/bin/say", "-v", speaker["voice"], "-r", speaker["rate"], "-o", aiff, turn["text"]])
            # Trim only the edges, never sentence pauses inside an utterance.
            trim = "silenceremove=start_periods=1:start_threshold=-48dB"
            run(["ffmpeg", "-nostdin", "-v", "error", "-y", "-i", aiff,
                 "-af", f"{trim},areverse,{trim},areverse", "-ar", RATE,
                 "-ac", 1, "-c:a", "pcm_s16le", source])
        source_files.append(source)
        if (index + 1) % 8 == 0:
            print(f"Synthesised {index + 1}/{len(turns)} turns", flush=True)

    assemble_meeting(turns, source_files, args.out, args.seconds, SPEAKERS, {
        "synthesis": "macOS say",
        "scriptSha256": digest(args.script),
        "generatorSha256": digest(Path(__file__)),
        "limitations": ["Synthetic TTS, not human speech", "No overlap or microphone noise",
                        "Reference windows include natural sentence pauses; not speech-activity ground truth",
                        "Apple voice assets remain governed by their terms; audio is local-only"],
    })


def assemble_meeting(turns, source_files, out, seconds, speakers, provenance):
    """Assemble measured utterances, preserving timings for either TTS engine."""
    if len(turns) != len(source_files) or not turns:
        raise ValueError("Each turn needs exactly one rendered source")
    if (out / "manifest.json").exists():
        raise ValueError("Refusing to overwrite a completed fixture")
    parts = out / "parts"
    parts.mkdir(parents=True, exist_ok=True)
    # A small uniform tempo adjustment makes this exactly thirty minutes rather
    # than padding the end with silence. Pitch is preserved. Record the factor.
    gaps = [round(RATE * (0.23 + (index % 5) * 0.08)) for index in range(len(turns))]
    target_frames = seconds * RATE
    speech_budget = target_frames - sum(gaps) - RATE  # one second before the first turn
    tempo = sum(frames(path) for path in source_files) / speech_budget
    if not 0.7 <= tempo <= 1.3:
        raise ValueError(f"Tempo {tempo:.3f} is too extreme; adjust the source speaking rates")
    adjusted = []
    for index, source in enumerate(source_files):
        target = parts / f"timed-{index:03}.wav"
        run(["ffmpeg", "-nostdin", "-v", "error", "-y", "-i", source,
             "-af", f"atempo={tempo:.10f}", "-c:a", "pcm_s16le", target])
        adjusted.append(target)
    total = RATE + sum(frames(path) for path in adjusted) + sum(gaps)
    # FFmpeg's filter rounds each utterance. Absorb that subsecond residual in
    # the distributed inter-turn gaps, not by truncating the final speaker.
    residual = target_frames - total
    for index in range(len(gaps)):
        correction = round(residual / (len(gaps) - index))
        gaps[index] += correction
        residual -= correction
        if gaps[index] < 0:
            raise ValueError("Tempo rounding exceeded the available gap")

    output = out / "werkstatt-am-park-30min.wav"
    segments = []
    cursor = RATE
    with wave.open(str(output), "wb") as meeting:
        meeting.setparams((1, 2, RATE, 0, "NONE", "not compressed"))
        meeting.writeframes(b"\0" * RATE * 2)
        for turn, source, gap in zip(turns, adjusted, gaps):
            start = cursor
            with wave.open(str(source), "rb") as audio:
                while chunk := audio.readframes(RATE):
                    meeting.writeframesraw(chunk)
                cursor += audio.getnframes()
            # Field order matches the shipped embedding sidecar's input format.
            segments.append({"startMs": round(start * 1000 / RATE),
                             "endMs": round(cursor * 1000 / RATE), **turn})
            meeting.writeframesraw(b"\0" * gap * 2)
            cursor += gap
    assert frames(output) == target_frames
    preview = output.with_suffix(".m4a")
    run(["ffmpeg", "-nostdin", "-v", "error", "-y", "-i", output,
         "-c:a", "aac", "-b:a", "96k", "-movflags", "+faststart", preview])
    (out / "reference.json").write_text(json.dumps({"language": "de", "speakers": speakers,
                                                       "segments": segments}, ensure_ascii=False, indent=2) + "\n")
    (out / "reference.rttm").write_text("".join(
        f"SPEAKER werkstatt 1 {s['startMs'] / 1000:.3f} {(s['endMs'] - s['startMs']) / 1000:.3f} <NA> <NA> {s['speaker']} <NA> <NA>\n"
        for s in segments))
    (out / "reference.md").write_text("# Werkstatt am Park\n\nFictional German meeting; four synthetic voices.\n\n" + "\n\n".join(
        f"**{stamp(s['startMs'])} — {speakers[s['speaker']]['name']}**\n\n{s['text']}" for s in segments) + "\n")
    manifest = {"synthetic": True, "durationSeconds": seconds, "sampleRate": RATE,
                "turns": len(turns), "words": sum(len(t["text"].split()) for t in turns),
                "speakers": speakers, "tempoFactor": tempo, "system": platform.platform(),
                "assemblySha256": digest(Path(__file__)),
                "ffmpegVersion": subprocess.check_output(["ffmpeg", "-version"], text=True).splitlines()[0],
                "checksums": {path.name: digest(path) for path in [output, preview, out / "reference.json"]},
                **provenance}
    (out / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(f"Ready: {output}\n{len(turns)} turns, {seconds} seconds, tempo {tempo:.4f}", flush=True)


if __name__ == "__main__":
    main()
