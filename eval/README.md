# Evaluation material (never committed)

This directory is for local quality evaluation: recordings, the protocols people actually wrote, and notes about what a generated protocol gets right or wrong.

Everything here except this README and `.gitignore` is ignored by Git. That is deliberate. Real meeting material is useful for honest evaluation, but it is not publication material and it is not a test fixture.

Suggested shape, one neutral directory per meeting:

```text
eval/<short-label>/
├── audio.<ext>        the recording
├── reference.md       the protocol a person wrote
└── notes.md            why the example is useful
```

A directory name can still reveal information, so keep labels generic.

## What leaves the machine

Running the material through the local application is local. Reading it in an assistant conversation is not: content shown to a hosted model leaves the device. Do not use an assistant to inspect sensitive evaluation material merely because it is convenient.

Useful measures can be computed locally: whether decisions and actions were captured, whether quantities survived, whether unsupported claims appeared, how long the output is, and how much editing it required. The person who wrote the reference remains the best judge of whether the wording is good.

Prefer, in order: a meeting with no client or personal content, a redacted copy, or a synthetic re-enactment. Use a genuinely sensitive recording only when the whole evaluation loop stays on the device.

## The denylist

`eval/redacted-names.txt` holds the names that were removed from the source on
30 August 2026, one per line. `src/lib/redactedNames.test.ts` reads it and fails
if any of them reappears in a tracked file.

It lives here, and not beside the test, for the same reason everything else here
does: it is real material. A clone without this file skips the check and says so,
which is correct — a machine with no reference meeting has nothing to protect.

Add to it whenever new material is evaluated. The stem is enough; matching is
case-insensitive and by substring, so a surname catches its inflections.
