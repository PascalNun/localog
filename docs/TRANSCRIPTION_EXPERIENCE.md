# Transcription experience

Transcription is a means to the protocol, but it is also the first place where the user can correct the record. The application should make it understandable and recoverable without asking the person to become a runtime administrator.

## The normal path

The user chooses a quality outcome such as Fast, Balanced, or Accurate. The application eventually chooses the appropriate local model and explains its cost. Exact runtime paths and model identifiers belong in advanced settings or diagnostics.

The diariser now has a release-sidecar path and the meeting flow can prepare its verified model files
on first use. The transcription executable still accepts a manually supplied whisper.cpp binary in the
development build; that remains temporary scaffolding while its sidecar is built and signed.

## What the application records

For each transcript, LocaLog records the input and the conditions that shaped it:

- runtime and provider version;
- model identifier, size, and digest where available;
- language and resolved transcription settings;
- vocabulary revision and prompt;
- normalised input checksum;
- application version and timing information.

This is provenance, not a claim that two model runs will produce byte-identical text.

## Vocabulary

Vocabulary is a small, structured library of terms, preferred spellings, aliases, categories, and notes. Global entries can be shared across projects; project entries take precedence.

The list is prioritised rather than sent wholesale. Proper names and unusual terms deserve the limited prompt space before terminology the model already knows. A transcript or protocol should record which vocabulary shaped it.

## Uncertainty

The transcript may mark words the model itself considered uncertain. The review surface should name those words, make them searchable, and let editing settle the warning. Uncertainty is an invitation to look, not a judgement that the text is wrong.

The first threshold was measured on German audio. English validation remains outstanding.

## Speakers

Speaker separation is optional and provisional. A diariser reports voice turns; the transcriber reports text turns. Their time boundaries do not match, so LocaLog joins them by overlap and leaves labels editable.

Every transcript records the outcome of that optional pass. “Resolved” means usable diariser turns
were aligned to the text; “failed” means the configured pass produced no usable output; “unavailable”
means the runtime or its models were not ready. In the latter two cases the neutral `Speaker 1`
label is a fallback, not a claim that there was only one speaker. Older transcripts may report that
the result is unknown because that metadata did not exist yet. The review inspector says which case
applies and explains whether preparation is needed before a rerun. If the number of speakers is known, it is supplied on
the meeting’s transcription step for that run; it is not a global setting.

The meeting language is an explicit input, inherited from the project by default and editable before
or after a transcript exists. If it was wrong, the user can correct it and choose “Rerun transcription”.
The existing transcript stays visible until the replacement has been validated and committed. Automatic
language detection may become an advisory preflight later, but it must never silently override the
selected language.

The current ONNX direction is promising, but real-room speech, overlapping voices, multilingual quality, long recordings, and the M1/8 GB baseline still need evidence. The release sidecar is built from a pinned sherpa-onnx source revision and discovered automatically; signing, platform validation, and the corresponding whisper/FFmpeg packaging remain.

## Recovery and responsiveness

Normalised audio is a regenerable cache. The imported original remains untouched. Transcription runs away from the interface thread, reports bounded progress, can be cancelled, and leaves the last stable revision available after failure or restart.

The review surface must remain usable while work runs. A person should be able to read or correct an existing transcript rather than watch a blocked application window.

## Distribution still to solve

The public application should bundle or resolve its runtimes without asking ordinary users to browse for executable files. The release configuration now carries the diariser sidecar; a signed whisper.cpp sidecar, a redistributable FFmpeg strategy, model licence review, and platform validation on macOS, Windows, and Linux remain required.
