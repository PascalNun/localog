"""Media assembly checks; no model, network or speech voices required."""

import array
import json
import math
import shutil
import tempfile
import unittest
import wave
from pathlib import Path

from build import RATE, SPEAKERS, assemble_meeting, frames
from build_ming import AUDITION_TURNS, VOICES


class BuildTests(unittest.TestCase):
    def test_audition_repeats_each_speaker_and_preserves_script_indices(self):
        from build import read_turns

        turns = read_turns(Path(__file__).with_name("meeting-de.txt"))
        self.assertEqual(len(set(VOICES.values())), 4)
        for speaker in SPEAKERS:
            self.assertEqual(sum(turns[i]["speaker"] == speaker for i in AUDITION_TURNS), 2)

    @unittest.skipUnless(shutil.which("ffmpeg"), "requires FFmpeg")
    def test_assembly_is_exact_and_timings_follow_rendered_samples(self):
        with tempfile.TemporaryDirectory(prefix="localog-synthetic-test-") as directory:
            out = Path(directory)
            sources = []
            turns = []
            for index, speaker in enumerate(SPEAKERS):
                source = out / f"source-{index}.wav"
                samples = array.array("h", [round(4000 * math.sin(2 * math.pi * 300 * frame / RATE))
                                            for frame in range(RATE)])
                with wave.open(str(source), "wb") as audio:
                    audio.setparams((1, 2, RATE, 0, "NONE", "not compressed"))
                    audio.writeframes(samples.tobytes())
                sources.append(source)
                turns.append({"speaker": speaker, "text": "Synthetic test."})
            assemble_meeting(turns, sources, out, 7, SPEAKERS, {"synthesis": "test tone"})
            self.assertEqual(frames(out / "werkstatt-am-park-30min.wav"), 7 * RATE)
            reference = json.loads((out / "reference.json").read_text())["segments"]
            previous = 0
            for actual, expected in zip(reference, turns):
                self.assertGreater(actual["startMs"], previous)
                self.assertGreater(actual["endMs"], actual["startMs"])
                self.assertEqual(actual["speaker"], expected["speaker"])
                previous = actual["endMs"]
            self.assertLess(previous, 7000)
            with self.assertRaisesRegex(ValueError, "overwrite"):
                assemble_meeting(turns, sources, out, 7, SPEAKERS, {})

    def test_assembly_rejects_missing_sources(self):
        with tempfile.TemporaryDirectory(prefix="localog-synthetic-test-") as directory:
            with self.assertRaisesRegex(ValueError, "Each turn"):
                assemble_meeting([{"speaker": "M", "text": "Test"}], [], Path(directory), 7, SPEAKERS, {})


if __name__ == "__main__":
    unittest.main()
