# One embedding per transcript segment

This study asks whether speaker separation needs the segmentation model it
currently runs.

The shipped pass hands the diariser audio and lets a pyannote segmentation model
find where speakers change — over a recording whose boundaries transcription has
already established. Everything awkward about that pass follows from it: the
condensed working file, the silence between samples, the 300 ms gap that turned
out to be shorter than the diariser's own `min_duration_off`, the merged runs of
126 segments, and the eight minutes each speaker count costs.

sherpa-onnx's C API exposes a speaker embedding extractor that takes the embedding
model alone. So the pass could instead be: take a couple of seconds from the middle
of each known segment, compute one vector, cluster the vectors.

`embed-segments` does the embedding. `spikes/speaker-embedding/` holds nothing else,
because clustering a few hundred short vectors is arithmetic.

## What was measured

An M1 Pro with 16 GB, the same 3D-Speaker embedding model the application already
installs, and two recordings: the synthetic three-speaker German fixture from
`spikes/speaker-diarisation/`, which has recorded ground truth, and the 81.8-minute
reference meeting of 675 segments, which has none.

### It reproduces known ground truth exactly

Clustering the fixture's six turns into three groups returns the ground truth,
each cluster one speaker. The vectors separate with room to spare:

|                    | Cosine similarity |
| ------------------ | ----------------- |
| Same speaker       | +0.754 to +0.810  |
| Different speakers | +0.092 to +0.281  |

Merging from six clusters to two, the similarity falls off a cliff exactly where
the speakers run out — `0.810, 0.790, 0.777, **0.255**, 0.148`.

### It is one to two orders of magnitude faster

|                                | Time on the reference meeting |
| ------------------------------ | ----------------------------: |
| Diariser, whole recording      |                        1810 s |
| Diariser, sampled condensation |                         498 s |
| **Embedding 675 segments**     |                    **23.3 s** |
| **Clustering 675 vectors**     |                     **5.9 s** |

Clustering is not a second pass over audio, so every speaker count can be tried at
once. The eight-minutes-per-count that made the number worth arguing about
disappears.

### It agrees with the diariser, slightly more than the diariser's own two runs agree

At eleven clusters, the embedding result agrees with the diariser's sampled run on
**91.9 %** of segments. For scale, the diariser's own whole-recording and sampled
runs agree with each other on 88.6 %. The shapes match closely too — segments per
speaker of `388 120 102 17 17 11 10 7 1 1 1` against the diariser's
`381 121 100 25 16 11 8 7 4 2`.

### A speaker count falls out of it

Pairwise similarity on the real meeting is bimodal: a peak near +0.1 where
different people are compared, a trough around +0.3, and a second peak at +0.5 to
+0.6 where somebody is compared with themselves. So a count can be read off by
merging until the similarity drops through a floor:

| Floor | Speakers found |
| ----: | -------------: |
|  0.14 |              6 |
|  0.16 |              7 |
|  0.18 |              9 |
|  0.20 |         **12** |
|  0.25 |             15 |

An earlier version of this table was off by one at every row. The Python counted
the groups remaining _after_ the merge it had just refused for being below the
floor, rather than before it; porting the same algorithm to Rust found it. The
figures above come from the Rust, whose grouping reproduces the Python's exactly —
`388 120 102 17 17 11 10 7 1 1 1` at eleven voices.

The diariser's own automatic mode gives 67 on the same audio, and 86 on the whole
recording, so reading the count off the merge similarity is a different order of
answer. Whether it is the _right_ answer is not established: this recording's true
count is unknown, and the floor that produces a plausible number here is a constant
fitted to one meeting.

## What this does not establish

The reference meeting has no known speaker count, so eleven being plausible is not
eleven being correct, and a floor calibrated against one meeting is a guess with a
decimal point. Several recordings with known participants are needed before any
floor is chosen, and that evidence does not exist yet.

The fixture is clean, short, synthetic and non-overlapping.

One embedding per segment cannot catch a speaker changing inside a segment, which
pyannote can in principle. Segments here average 7.3 seconds and the sample comes
from the middle. The diariser's measured behaviour — those unbroken runs of 126
segments — suggests it is not catching them in practice either, but that is an
argument from its failure rather than evidence for ours.

The dominant speaker holding 56-58 % of segments appears in all three methods.
Either it is real or it is a property of the embedding model, and this study cannot
tell which. That model is trained on Chinese, which remains a poor match for the
first audience and remains unexamined.

## Running it

```bash
SHERPA_ONNX_LIB=/path/to/sherpa-onnx/lib cargo build --release
./target/release/embed-segments <embedding-model.onnx> <audio.wav> <segments.json> > vectors.json
```

The audio must be 16 kHz mono 16-bit PCM — the application's working audio already
is. `segments.json` may be a LocaLog transcript document or a bare array of
`{"startMs": …, "endMs": …}`.

`SHERPA_ONNX_LIB` must point at the `lib` directory of a sherpa-onnx shared
distribution, the one holding `libsherpa-onnx-c-api`. The build refuses without it
rather than guessing at a path.

Nothing here is production code. It exists so the decision about restructuring the
application rests on measurement.
