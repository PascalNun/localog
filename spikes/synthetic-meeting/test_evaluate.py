"""Small risk-boundary tests for scoring, independent of TTS and model weights."""

import unittest

from evaluate import predict, score, validate_intervals


def interval(start, end, speaker):
    return {"startMs": start, "endMs": end, "speaker": speaker}


class ScoringTests(unittest.TestCase):
    def setUp(self):
        self.truth = [interval(0, 1000, "A"), interval(1500, 2500, "B")]

    def test_cluster_numbers_are_arbitrary(self):
        result = score(self.truth, [interval(0, 1000, "8"), interval(1500, 2500, "2")])
        self.assertEqual(result["correctReferenceWindowPercent"], 100)

    def test_overclustering_does_not_earn_perfect_credit(self):
        truth = [interval(0, 1000, "A"), interval(1000, 2000, "A")]
        result = score(truth, [interval(0, 1000, "1"), interval(1000, 2000, "2")])
        self.assertEqual(result["correctReferenceWindowPercent"], 50)

    def test_merged_speaker_and_silence_are_separate_errors(self):
        result = score(self.truth, [interval(0, 2500, "1")])
        self.assertEqual(result["correctReferenceWindowPercent"], 50)
        self.assertEqual(result["confusedWindowSeconds"], 1)
        self.assertEqual(result["outsideReferenceWindowSeconds"], 0.5)
        self.assertEqual(result["mixedSpeakerSegments"], 1)
        self.assertEqual(result["mixedOver250MsSegments"], 1)
        self.assertEqual(result["singleLabelPerSegmentCeilingPercent"], 50)

    def test_missing_speech_is_not_counted_as_confusion(self):
        result = score(self.truth, [interval(0, 500, "1")])
        self.assertEqual(result["missedWindowSeconds"], 1.5)
        self.assertEqual(result["confusedWindowSeconds"], 0)

    def test_invalid_intervals_fail_instead_of_double_counting(self):
        with self.assertRaises(ValueError):
            validate_intervals([interval(0, 200, "A"), interval(100, 300, "B")])
        with self.assertRaises(ValueError):
            validate_intervals([interval(0, 0, "A")])

    def test_short_segment_matches_current_application_fallback(self):
        result = predict(self.truth, {"indices": [1], "known": [2]}, "known")
        self.assertEqual([s["speaker"] for s in result], ["0", "2"])


if __name__ == "__main__":
    unittest.main()
