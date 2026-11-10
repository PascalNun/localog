#!/usr/bin/env python3
"""Measure shipped embedding + clustering against synthetic speaker ground truth.

This is an adapter-level evaluation, not a Tauri UI or job-recovery test. Its
intervals are utterance windows, not phonetic speech activity, so it does not
report a standard diarisation error rate (DER).
"""

import argparse
import json
import platform
import re
import subprocess
import time
from pathlib import Path

from build import digest


def validate_intervals(segments):
    previous = 0
    for segment in segments:
        start, end = segment["startMs"], segment["endMs"]
        if start < previous or end <= start:
            raise ValueError("Expected sorted, non-overlapping, positive intervals")
        previous = end


def score(reference, predicted):
    """One-to-one label mapping, weighted by reference-window intersection.

    A split into many clusters must not earn perfect credit just because every
    fragment has a pure majority. At most one cluster can map to each reference
    speaker. With four speakers an exact subset dynamic program is tiny.
    """
    validate_intervals(reference)
    validate_intervals(predicted)
    speakers = sorted({segment["speaker"] for segment in reference})
    labels = sorted({segment["speaker"] for segment in predicted})
    matrix = {label: [0] * len(speakers) for label in labels}
    mixed = 0
    substantial_crossings = 0
    ceiling = 0
    for prediction in predicted:
        heard = set()
        per_speaker = [0] * len(speakers)
        for truth in reference:
            overlap = max(0, min(truth["endMs"], prediction["endMs"]) - max(truth["startMs"], prediction["startMs"]))
            if overlap:
                matrix[prediction["speaker"]][speakers.index(truth["speaker"])] += overlap
                per_speaker[speakers.index(truth["speaker"])] += overlap
                heard.add(truth["speaker"])
        mixed += len(heard) > 1
        ceiling += max(per_speaker, default=0)
        substantial_crossings += sum(duration >= 250 for duration in per_speaker) > 1
    states = {0: (0, {})}
    for label in labels:
        following = dict(states)
        for mask, (value, mapping) in states.items():
            for index, speaker in enumerate(speakers):
                if not mask & (1 << index):
                    new_mask = mask | (1 << index)
                    candidate = value + matrix[label][index]
                    if candidate > following.get(new_mask, (-1, {}))[0]:
                        following[new_mask] = (candidate, {**mapping, label: speaker})
        states = following
    correct, mapping = max(states.values(), key=lambda state: state[0])
    total = sum(s["endMs"] - s["startMs"] for s in reference)
    covered = sum(sum(row) for row in matrix.values())
    predicted_duration = sum(s["endMs"] - s["startMs"] for s in predicted)
    by_speaker = {}
    for index, speaker in enumerate(speakers):
        duration = sum(s["endMs"] - s["startMs"] for s in reference if s["speaker"] == speaker)
        credited = sum(matrix[label][index] for label, name in mapping.items() if name == speaker)
        by_speaker[speaker] = {"windowSeconds": duration / 1000, "correctPercent": 100 * credited / duration}
    return {"referenceSpeakers": len(speakers), "predictedSpeakers": len(labels),
            "correctReferenceWindowPercent": 100 * correct / total,
            "confusedWindowSeconds": (covered - correct) / 1000,
            "missedWindowSeconds": (total - covered) / 1000,
            "outsideReferenceWindowSeconds": (predicted_duration - covered) / 1000,
            "mixedSpeakerSegments": mixed, "mixedOver250MsSegments": substantial_crossings,
            "singleLabelPerSegmentCeilingPercent": 100 * ceiling / total,
            "mapping": mapping, "bySpeaker": by_speaker,
            "matrixSeconds": {label: {speaker: row[index] / 1000 for index, speaker in enumerate(speakers)}
                              for label, row in matrix.items()}}


def measured(arguments, log):
    started = time.monotonic()
    timing = ["/usr/bin/time", "-l" if platform.system() == "Darwin" else "-v"]
    with log.open("w") as errors:
        subprocess.run(timing + [str(arg) for arg in arguments], check=True,
                       stdout=subprocess.DEVNULL, stderr=errors, timeout=1800)
    detail = log.read_text(errors="replace")
    memory = re.search(r"(\d+)\s+maximum resident set size", detail)
    return {"seconds": time.monotonic() - started,
            "peakResidentBytes": int(memory[1]) if memory else None}


def predict(segments, grouping, mode):
    # Match the current app's fallback: an unembedded short segment gets Speaker 1.
    # Count these explicitly rather than presenting the fallback as an observation.
    labels = dict(zip(grouping["indices"], grouping[mode]))
    return [{"startMs": segment["startMs"], "endMs": segment["endMs"],
             "speaker": str(labels.get(index, 0))} for index, segment in enumerate(segments)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--audio-name", default="werkstatt-am-park-30min.wav")
    parser.add_argument("--reference-name", default="reference.json")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--embedding-executable", type=Path, required=True)
    parser.add_argument("--embedding-model", type=Path, required=True)
    parser.add_argument("--whisper-executable", type=Path, required=True)
    parser.add_argument("--whisper-model", type=Path, required=True)
    parser.add_argument("--threads", type=int, default=8)
    args = parser.parse_args()
    args.out = args.out.resolve()
    if args.out.exists():
        parser.error("Choose a new evaluation directory to avoid reusing stale results")
    args.out.mkdir(parents=True)
    audio = args.fixture.resolve() / args.audio_name
    reference_path = args.fixture.resolve() / args.reference_name
    reference = json.loads(reference_path.read_text())["segments"]
    validate_intervals(reference)
    root = Path(__file__).resolve().parents[2]
    cluster = args.out / "cluster"
    subprocess.run(["rustc", "--edition=2021", "-O", str(Path(__file__).with_name("cluster.rs")),
                    "-o", str(cluster)], check=True)
    results = {"system": platform.platform(), "audioSha256": digest(audio),
               "referenceSha256": digest(reference_path), "embeddingThreads": args.threads,
               "whisperThreads": "runtime default",
               "modelFiles": {"embedding": args.embedding_model.name, "whisper": args.whisper_model.name},
               "models": {"embedding": digest(args.embedding_model), "whisper": digest(args.whisper_model)},
               "runtimes": {"embedding": digest(args.embedding_executable), "whisper": digest(args.whisper_executable)},
               "clusteringSha256": digest(root / "src-tauri/src/clustering.rs"),
               "evaluationSha256": digest(Path(__file__)),
               "scope": "Shipped sidecars and exact app clustering; not the Tauri command/job/storage path",
               "metric": "Utterance-window attribution, one-to-one optimal label mapping, no collar; NOT standard DER",
               "runs": {}}

    def evaluate_boundaries(name, segments):
        validate_intervals(segments)
        timings = args.out / f"{name}-segments.json"
        timings.write_text(json.dumps(segments, ensure_ascii=False) + "\n")
        vectors = args.out / f"{name}-vectors.bin"
        print(f"Embedding {name}: {len(segments)} segments", flush=True)
        measurement = measured([args.embedding_executable.resolve(), "--model", args.embedding_model.resolve(),
                                "--audio", audio, "--segments", timings, "--out", vectors,
                                "--threads", args.threads], args.out / f"{name}-embedding.log")
        started = time.monotonic()
        grouping = json.loads(subprocess.check_output([str(cluster), str(vectors), str(len({s['speaker'] for s in reference}))], text=True))
        measurement["clusteringSeconds"] = time.monotonic() - started
        measurement["embeddedSegments"] = len(grouping["indices"])
        measurement["unembeddedSegments"] = len(segments) - len(grouping["indices"])
        for mode in ["known", "automatic"]:
            predictions = predict(segments, grouping, mode)
            result = {**measurement, **score(reference, predictions)}
            (args.out / f"{name}-{mode}-predicted.json").write_text(json.dumps(predictions, indent=2) + "\n")
            results["runs"][f"{name}-{mode}"] = result
            print(f"{name}/{mode}: {result['predictedSpeakers']} voices, {result['correctReferenceWindowPercent']:.2f}% correct reference-window time", flush=True)
        (args.out / "results.json").write_text(json.dumps(results, indent=2) + "\n")

    evaluate_boundaries("reference", reference)
    print("Transcribing the fixture with the installed base model (German)", flush=True)
    transcript = args.out / "whisper"
    results["transcription"] = measured([args.whisper_executable.resolve(), "-m", args.whisper_model.resolve(),
        "-f", audio, "--output-json-full", "--output-file", transcript,
        "--language", "de", "--print-progress"], args.out / "whisper.log")
    # The app leaves whisper.cpp's thread/default decoding settings unchanged too.
    whisper = json.loads(transcript.with_suffix(".json").read_bytes())
    segments = [{"startMs": s["offsets"]["from"], "endMs": s["offsets"]["to"], "text": s["text"]}
                for s in whisper["transcription"] if s["text"].strip()]
    evaluate_boundaries("whisper", segments)
    print(f"Results: {args.out / 'results.json'}", flush=True)


if __name__ == "__main__":
    main()
