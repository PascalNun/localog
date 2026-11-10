#!/usr/bin/env python3
"""Check spoken content using local Whisper output, separately from voice labels.

This is an ASR-based intelligibility check, not a naturalness or factual-quality
score. Written numbers versus spoken number words still count as differences.
"""

import argparse
import json
import re
import unicodedata
from pathlib import Path

from build import digest


def words(text):
    return re.findall(r"\w+", unicodedata.normalize("NFKC", text).casefold())


def word_error_rate(expected, actual):
    reference, hypothesis = words(expected), words(actual)
    if not reference:
        raise ValueError("A non-empty reference is required")
    previous = list(range(len(hypothesis) + 1))
    for row, word in enumerate(reference, 1):
        current = [row]
        for column, candidate in enumerate(hypothesis, 1):
            current.append(min(current[-1] + 1, previous[column] + 1,
                               previous[column - 1] + (word != candidate)))
        previous = current
    return {"referenceWords": len(reference), "recognisedWords": len(hypothesis),
            "wordEdits": previous[-1], "wordErrorPercent": 100 * previous[-1] / len(reference)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--whisper-json", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    reference = json.loads(args.reference.read_text())
    transcription = json.loads(args.whisper_json.read_bytes())
    expected = " ".join(segment["text"] for segment in reference["segments"])
    actual = " ".join(segment["text"] for segment in transcription["transcription"])
    result = {**word_error_rate(expected, actual), "referenceSha256": digest(args.reference),
              "whisperJsonSha256": digest(args.whisper_json), "scorerSha256": digest(Path(__file__)),
              "scope": "ASR-based content check; not human listening or a naturalness score"}
    args.out.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
