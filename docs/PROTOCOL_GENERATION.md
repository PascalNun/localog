# Making protocol generation trustworthy

This document explains the current generation problem and the direction being tested. It is an engineering investigation, not a promise that every part is already in the production path.

## The problem

A meeting protocol asks a model to do several difficult things at once:

1. understand a long transcript;
2. decide what mattered;
3. attribute statements and actions;
4. write a coherent professional document;
5. avoid inventing anything;
6. fit inside the model’s context and output limits.

The current small-model path can produce a plausible document, but plausibility is not enough. A protocol can read well while quietly omitting a decision, losing a number, or attributing an action to the wrong person.

## What the experiments taught us

The long German reference meeting contained roughly 73,000 transcript characters and nineteen quantities that could be found without a language model. One early protocol accounted for only one of those quantities.

Writing one section per topic improved figure coverage but produced a document almost as long as the transcript. The model was no longer forced to decide what to leave out; it simply rewrote the meeting.

The useful lesson is not “ask for shorter prose” more emphatically. Compression is a judgement, and an instruction cannot reliably replace a constraint enforced by the context window.

## The direction being tested

Separate reading from writing without throwing away the source.

The useful intermediate is a record whose entries point back to transcript segments:

```text
MeetingRecord
  topics         title + source segments
  decisions      statement + source segments + certainty
  actions        task + owner + due date + source segments
  open questions question + source segments
  figures        quantity + source segments
```

The exact shape is still being tested. The important property is that a later step can ask what an item came from, what was not claimed, and what needs another attempt.

## Proposed passes

1. **Read.** Scan manageable transcript sections and extract structured findings, not polished prose.
2. **Consolidate.** Merge repeated topics and actions in plain code. Ask the model only about genuine ambiguities.
3. **Compose.** Write one coherent protocol from the evidence and the selected professional style.
4. **Check.** Compare the protocol with what the transcript actually contains.
5. **Retry narrowly.** Re-ask one failed pass or section instead of throwing away a complete run.

This is not a universal workflow engine. The number of passes, their limits, and their stopping rules remain explicit application code.

## What can be checked without a model

Some checks are mechanical and should stay mechanical:

- quantities and dates found by pattern;
- statements that cite no transcript segment;
- actions without an owner or an explicit unassigned marker;
- missing or duplicated required sections;
- output that is empty, truncated, or larger than a sensible bound.

These checks do not decide whether a protocol is good. They expose omissions and unsupported claims before a person relies on the draft.

## What remains human work

A model must not approve its own protocol. A final “looks good” verdict from the same model would create false confidence rather than trust.

The person reviewing the document remains responsible for deciding whether the writing is accurate, useful, and appropriate. The application’s job is to make that review easier and more informed.

## Current status

Sectioned generation, fact scanning, subject discovery, subject grouping, and unclaimed-segment reporting have been explored. The production provider still uses the simpler validated generation path while the structured approach is measured and connected carefully.

The next proof is a complete German run whose quality is judged against the human reference on completeness, correctness, attribution, length, and editing effort. An English run should follow.
