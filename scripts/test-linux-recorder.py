#!/usr/bin/env python3
"""Explicit desktop integration check; uses only synthetic virtual audio sources.

Requires PipeWire-Pulse/PulseAudio, pactl, paplay and a built recorder. No default
device is changed, no microphone is opened and nothing plays through speakers.
"""

import array
import json
import math
import os
from pathlib import Path
import signal
import selectors
import subprocess
import sys
import tempfile
import time
import wave


def run(*args, **kwargs):
    return subprocess.run(args, check=True, capture_output=True, text=True,
                          timeout=10, **kwargs).stdout.strip()


def tone(path, frequency):
    samples = array.array("h")
    for frame in range(288_000):
        sample = round(8_000 * math.sin(2 * math.pi * frequency * frame / 48_000))
        samples.extend((sample, sample))
    if sys.byteorder != "little":
        samples.byteswap()
    with wave.open(str(path), "wb") as output:
        output.setparams((2, 2, 48_000, 0, "NONE", "not compressed"))
        output.writeframes(samples.tobytes())


def verify(path, wanted, unwanted):
    with wave.open(str(path), "rb") as source:
        assert source.getparams()[:3] == (2, 2, 48_000), source.getparams()
        frames = source.getnframes()
        assert 48_000 <= frames <= 12 * 48_000, frames
        samples = array.array("h", source.readframes(frames))
    if sys.byteorder != "little":
        samples.byteswap()
    mono = samples[::2]

    def magnitude(frequency):
        # Compare short windows: independent sound-server clocks can resample or
        # shift phase, which cancels a single coherent sum across the whole file.
        total = 0
        for start in range(0, len(mono), 4096):
            window = mono[start:start + 4096]
            real = sum(s * math.cos(2 * math.pi * frequency * i / 48_000)
                       for i, s in enumerate(window))
            imag = sum(s * math.sin(2 * math.pi * frequency * i / 48_000)
                       for i, s in enumerate(window))
            total += math.hypot(real, imag)
        return total / len(mono)

    # Six seconds of the generated tone must survive, not merely a recognisable
    # fragment. Integrated energy also catches dropped packets amid silence.
    expected_energy = 6 * 48_000 * (8_000 ** 2 / 2)
    energy_ratio = sum(sample * sample for sample in mono) / expected_energy
    assert 0.9 < energy_ratio < 1.1, (path.name, "audio lost or duplicated", energy_ratio)
    level = magnitude(wanted)
    assert level > 500, (path.name, "missing synthetic tone", level)
    assert level > 10 * magnitude(unwanted), (path.name, "tracks were mixed")
    return frames / 48_000


def main():
    root = Path(__file__).resolve().parent.parent
    recorder = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else (
        root / "sidecars/record-meeting/target/debug/localog-record-meeting")
    defaults = (run("pactl", "get-default-sink"), run("pactl", "get-default-source"))
    modules = []
    children = []
    sink = f"localog-test-{os.getpid()}"
    try:
        for name in (sink, sink + "-mic"):
            modules.append(run("pactl", "load-module", "module-null-sink",
                               f"sink_name={name}", "rate=48000", "channels=2"))
        env = dict(os.environ, PULSE_SINK=sink, PULSE_SOURCE=sink + "-mic.monitor")
        readiness = json.loads(run(str(recorder), "--check", env=env))
        assert readiness == {"systemAudio": "granted", "microphone": "granted"}, readiness
        with tempfile.TemporaryDirectory(prefix="localog-recorder-test-") as temporary:
            work = Path(temporary)
            tone(work / "call.wav", 440)
            tone(work / "room.wav", 880)
            recording = subprocess.Popen(
                [str(recorder), "--system", str(work / "system.wav"),
                 "--microphone", str(work / "microphone.wav")],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            children.append(recording)
            # Wait for the recorder's first real level line, not a guessed startup
            # delay: stream negotiation can take longer on a busy desktop.
            with selectors.DefaultSelector() as ready:
                ready.register(recording.stdout, selectors.EVENT_READ)
                assert ready.select(timeout=8), "Recorder never reported ready streams"
                first_level = recording.stdout.readline()
            playback = []
            for device, filename in ((sink, "call.wav"), (sink + "-mic", "room.wav")):
                child = subprocess.Popen(
                    ["paplay", "--client-name=" + device, "--stream-name=" + device,
                     "--latency-msec=20", "--raw", "--rate=48000", "--channels=2",
                     "--format=s16le", "--device", device], stdin=subprocess.PIPE)
                children.append(child)
                with wave.open(str(work / filename), "rb") as source:
                    playback.append((child, source.readframes(source.getnframes())))
            started_playback = time.monotonic()
            for frame in range(300):
                for child, audio in playback:
                    child.stdin.write(audio[frame * 3840:(frame + 1) * 3840])
                    child.stdin.flush()
                time.sleep(max(0, started_playback + (frame + 1) * 0.02 - time.monotonic()))
            for child, _ in playback:
                child.stdin.close()
            for child in children[1:]:
                assert child.wait(timeout=12) == 0
            time.sleep(1.2)
            started = time.monotonic()
            recording.send_signal(signal.SIGTERM)
            stdout, stderr = recording.communicate(timeout=3)
            assert recording.returncode == 0, stderr
            assert time.monotonic() - started < 2, "Recorder did not stop promptly"
            levels = [json.loads(line) for line in (first_level + stdout).splitlines()]
            assert any(line["systemPeak"] > 0.1 for line in levels), levels
            assert any(line["microphonePeak"] > 0.1 for line in levels), levels
            system_duration = verify(work / "system.wav", 440, 880)
            microphone_duration = verify(work / "microphone.wav", 880, 440)
            assert abs(system_duration - microphone_duration) < 0.2, (system_duration, microphone_duration)
        # A monitor selected as the microphone must not duplicate the system track.
        env["PULSE_SOURCE"] = sink + ".monitor"
        assert json.loads(run(str(recorder), "--check", env=env))["microphone"] == "denied"
        # A disconnected server must give an honest, bounded preflight response.
        env["PULSE_SERVER"] = "unix:/tmp/localog-no-such-pulse-server"
        assert json.loads(run(str(recorder), "--check", env=env))["unavailable"]
        print("PASS: separate synthetic tracks, valid WAVs, levels, prompt stop, "
              "duplicate-monitor guard and unavailable-server preflight")
    finally:
        for child in children:
            if child.poll() is None:
                child.kill()
                child.wait()
        for module in reversed(modules):
            run("pactl", "unload-module", module)
        assert defaults == (run("pactl", "get-default-sink"),
                            run("pactl", "get-default-source")), "Desktop defaults changed"


if __name__ == "__main__":
    main()
