# Evaluation material (never committed)

A local-only place for real meeting recordings and the protocols a person actually wrote from them,
used to judge whether LocaLog's generated protocols are good enough.

Everything in this directory except this file and `.gitignore` is ignored by git. That is deliberate:
`CONTRIBUTING.md` and `MVP.md` both forbid real recordings, transcripts, names, client information or
protocols from entering fixtures, tests, screenshots or issues. Synthetic fixtures for tests belong in
`fixtures/` or beside their spike; nothing here is a test fixture.

Suggested shape, one directory per meeting:

```text
eval/<short-label>/
├── audio.<ext>        the recording
├── reference.md       the protocol a person wrote from it
└── notes.md           what makes this one useful, and anything the reference gets wrong
```

Keep the labels neutral. A directory name is still a name.

## What leaves the machine

Running this material through LocaLog is entirely local: import, transcription and generation all
happen on the device, which is the point of the product.

Reading it in an assistant conversation is not local. Content pasted into or read out of this
directory during a chat is sent to a model provider. Judging quality does not require that: the
metrics in an evaluation run — section coverage, length, whether decisions and actions were captured,
whether anything was invented — can be computed and compared locally, and the verdict on wording is
better made by the person who wrote the reference anyway.

Prefer, in order: a meeting with no client or personal content; a redacted copy; a synthetic
re-enactment. Use a genuinely sensitive recording only where the whole loop stays on the device.
