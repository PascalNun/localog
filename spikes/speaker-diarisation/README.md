# Speaker separation study

This study tests the accepted v0.1 direction of using an ONNX diariser instead of a Python/PyTorch stack. It ran on an M1 Pro with 16 GB RAM, not the M1/8 GB baseline.

## What was measured

- sherpa-onnx v1.13.4 on macOS arm64;
- a 23.5-second synthetic German fixture with three clean voices;
- a segmentation model and a 3D-Speaker embedding model;
- known speaker count and automatic speaker-count detection.

The fixture ran in 7.24 seconds, used about 259 MB peak memory, and required about 46 MB of model files. Frame accuracy was 88.2% when the speaker count was supplied, and automatic detection found all three speakers across the tested thresholds.

## What that means

The approach is technically viable and relatively small. It is not evidence of production speaker quality. The fixture has no room noise, reverberation, crosstalk, overlapping speech, or long-duration behaviour, and the embedding model was trained on Chinese.

Before accepting the feature for normal use, compare a multilingual or German-suited model, test overlap and long recordings, measure the M1/8 GB machine, and validate the alignment between diariser turns and transcript segments with real listening.

Downloaded macOS binaries also need ad-hoc signing during development or they can be killed without a useful diagnostic. Shipped binaries will be signed with the application.
