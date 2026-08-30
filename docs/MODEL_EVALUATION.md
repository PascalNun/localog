# Model evaluation

This is the evidence behind the current protocol-generation direction. It describes what was measured, what the measurements mean, and where they stop being reliable.

## The recording stops before the meeting does

Found 17 August 2026, and it corrects the baseline every measurement in this document rests on.

The recording ends at 81 minutes 43 seconds, in the middle of a discussion of balcony dividing walls and the planting carried over from the competition. That is where section 7.5 of the written protocol ends. Everything after it in that document — the appointment with the district office, the section on specialist planning and working arrangements, the seventeen agreed next steps, and the dates — was said after the recording stopped.

**About 22% of the written protocol describes a meeting the recording does not contain.**

Three consequences, and the first two invalidate work already done here:

- **Coverage against the written protocol has been measured against a document that covers more than the source.** Every "the draft omits this subject" finding needs re-reading with that in mind; one such finding was made and withdrawn on 17 August for a related reason, and the withdrawal did not go far enough.
- **The seventeen actions the written protocol records were never all available.** A draft producing four or seven of them is not recording a quarter of the meeting's actions; it is recording what was agreed before the recording ended. How many that is has not been counted.
- **Nothing tells a person their recording stopped early.** This one is a product finding rather than an evaluation one. A protocol generated from a partial recording is silently partial, reads as complete, and has no way to say otherwise. The transcript's last words being mid-subject is visible to anybody who looks and is looked at by nobody.

It also explains something that had been read as a model failure: drafts that "thin out towards the end" are following a recording that thins out towards the end.

## A prompt cannot supply what the input lacks

Measured 17 August 2026, and worth recording because the wrong fix was tried first and looked plausible.

Every draft named a speaker label as a person — "Speaker 1" listed twice in the participants list, once under electrical planning and once under fire safety. The transcript carries no speaker separation, so every segment holds that label, which is an absence of evidence about who spoke rather than evidence of one speaker. A model reading it cannot tell those apart.

**The style instruction did not work.** Adding "a speaker label is not a person's name; describe them by role where no name was said" and measuring at three seeds: the fault persisted at two of them, in new formats — `**Elektroplanung (AVENTOR):** Speaker 1` at one, `- Speaker 1 (Elektroplanung)` at another — and the third had never shown it, so it evidenced nothing either way. Figures were unaffected at 30 to 31 of 39, so the instruction was not harmful, merely useless: it supplies no information the model was missing.

**Not sending the field does work**, deterministically: where every segment carries the same label, the label is dropped and the field is not serialised, so the string is not in the model's input and cannot be written out. Labels that do tell speakers apart are kept.

Verified at the seed that showed the fault most clearly: three labels named as people before, none after, and figures unchanged at 30 of 39. The participants list now describes an unnamed person by their role — "Elektroplanung (Ansprechpartner für alle Fragen zum Thema Elektrik)" — which is what the style asked for all along and could not get while a label was there to use instead.

The asymmetry that let one seed settle this is worth keeping in mind next to the rule about three seeds. A fault _absent_ at one seed proves nothing — the seed may never have had it. A fault _present_ at one seed proves the fix does not reliably work. Scores need repetition; a fault reproducing does not.

### Comparing across a change you made yourself

Noticed 17 August 2026 while the runs were still going, which is the only reason it is worth recording rather than withdrawing.

The style's own length instruction was being measured by running three seeds without it and comparing against earlier drafts. Between those earlier drafts and this experiment, speaker labels stopped being sent — a separate fix, made the same morning. Comparing the two would have measured two changes and attributed both to one, and the numbers moved enough to make a confident wrong answer easy: 19 headings against 16, four table rows against two.

The baseline arm is being re-run with labels already gone, so both arms differ in one thing.

Worth stating as a rule alongside the others here, because it is not the same mistake as a single seed: **a baseline expires when the code changes.** Every measurement in this document names a date for that reason, and a comparison whose arms come from different days is a comparison of the days.

## The reference meeting

The main evaluation used an 81-minute German construction-project meeting:

- about 4,909 seconds of audio;
- 788 transcript segments in the original evaluation;
- roughly 74,000 transcript characters;
- a human protocol of about 18,000 characters;
- 30 headings and 17 recorded actions in the human reference.

The recording and reference remain local-only. This document records measurements, not names or meeting content.

All runs described here used an Apple M1 Pro with 16 GB RAM. That machine is stronger than the approved M1/8 GB baseline.

## Transcription

The measured whisper models behaved roughly as follows:

| Model  | Approximate time for the meeting |
| ------ | -------------------------------: |
| base   |                      1.5 minutes |
| medium |                        6 minutes |

Vocabulary made the important difference in this domain. Proper names and unusual project terms were the words most likely to be wrong, while ordinary professional terminology was usually already recognised correctly.

### Every comparison below was run on a transcript that used neither

Found on 16 August 2026, by reading a generated protocol against the one the owner wrote for the same meeting rather than against another draft.

The transcript all the model comparisons in this document are based on was produced with the `balanced` preset, which maps to whisper `base`, and with an empty vocabulary — the workspace holds zero vocabulary entries. Counted in that transcript, for terms the written protocol spells consistently:

| What the term is                    | Wrong spellings  | Times wrong | Ever right                            |
| ----------------------------------- | ---------------- | ----------: | ------------------------------------- |
| The building system supplier        | two variants     |          13 | never                                 |
| The housing form under discussion   | one variant      |          40 | never                                 |
| The funding body's abbreviation     | one variant      |           4 | 3 times, mixed                        |
| Five participant surnames           | one variant each |           8 | two of five, mixed                    |
| The word for structural engineering | —                |           — | **absent from all 72,837 characters** |

The last row is the one that matters. The word never appears, so the protocol model filed the structural engineer under a discipline he does not practise. No protocol model can recover a term it was never given, and the generated draft reproduced these spellings faithfully — which is the correct behaviour, not a fault.

This means the model comparisons in this document measure protocol models fairly against each other, but understate all of them. Terminology errors that were read as generation faults are transcription faults. The two levers were known and documented in this very section before the comparisons were run; neither was pulled.

### Pulling both levers

Same audio, same fourteen terms counted, 16 August 2026. Thirty proper nouns passed as whisper's initial prompt with `--carry-initial-prompt`, and then also the larger model.

| Transcription         | Terms with a correct spelling | Terms with no wrong spelling left |  Time |
| --------------------- | ----------------------------: | --------------------------------: | ----: |
| `base`, no vocabulary |                       3 of 14 |                           0 of 14 | 2 min |
| `base` + vocabulary   |                      10 of 14 |                           8 of 14 | 2 min |
| `medium` + vocabulary |                  **13 of 14** |                      **11 of 14** | 7 min |

Transcript length barely moved across all three — 72,837 to 74,114 characters. This is not more words, it is the same words spelled correctly.

The two levers compose rather than overlap. Two of the participant surnames did not improve at all from the vocabulary on `base` and were correct immediately on `medium`: biasing a model towards a spelling only helps if it can resolve the sounds in the first place. Conversely `medium` alone would not have known which of several plausible spellings a project uses.

One term resists every combination, and one pair of three-letter abbreviations differing by a single consonant still mixes. Both are arguments for making correction easy in the transcript view, not for expecting transcription to be perfect.

Seven minutes against two, on an eighty-two minute meeting, for that difference in terminology. Transcription and generation never run at once, so the larger model does not compete with the language model for memory.

### What that leaves for the person to correct

Run through the shipping extractor rather than a script, 16 August 2026. It offers a word only when the transcriber was unsure of it every time it was heard.

|                                                    | Original transcript | Both levers pulled |
| -------------------------------------------------- | ------------------: | -----------------: |
| Segments flagged as containing something uncertain |          322 of 675 |         192 of 793 |
| Candidates offered                                 |                   6 |                  4 |
| **Mis-heard names among them**                     |               **2** |              **0** |

Every candidate remaining on the corrected transcript is a correctly spelled word the transcriber happened to be unsure of, including a hyphenated compound spelled exactly as the written protocol spells it — one the original transcript never managed.

So on a well-transcribed meeting the panel has nothing to correct and says so. It reports what the transcriber was unsure of, not what is wrong.

This is the claim that survived the speaker-label control. That a corrected transcript produces a transcript with no names left to correct is direct. That it produces a better _protocol_ was tested and is not the case — see below.

### A correctly spelled transcript does not produce a better protocol

Settled 16 August 2026, after a control run removed the confound that made it look otherwise. `gemma4:12b` at 40,960, speaker labels flattened in both conditions so that only the spelling differs:

| Transcript                       |          Seed 7 |        Seed 101 |
| -------------------------------- | --------------: | --------------: |
| Original, labels flattened       | table, 28 of 35 | table, 30 of 35 |
| Corrected, `medium` + vocabulary | table, 30 of 39 | table, 28 of 39 |

Four runs, four action tables, and figure ratios of 80, 86, 77 and 72 per cent — overlapping, with the corrected transcript no better and possibly slightly worse.

The first comparison had suggested otherwise, but the corrected transcripts were also unlabelled while the original carried fifty-four speaker labels, and this document had already measured that removing labels changes the figures kept. Holding the labels constant removes the difference.

In hindsight the very first draft read against the written protocol said as much: every quantity in it was correct while every proper noun was wrong. The model was already extracting the right facts from a transcript full of mis-heard names.

**This relocates what the transcription work is for rather than diminishing it.** The vocabulary and the larger model do not help a model understand a meeting. They fix the document a person has to circulate, where a client's name spelt wrongly and a structural engineer filed under a discipline he does not practise are unacceptable however many figures survived. That is a narrower claim than the one first made here, and unlike the first one it holds.

### What this project's measurements are worth, by kind

Worth stating plainly, because four separate conclusions were withdrawn on 16 August 2026 and they failed the same way.

Every measure derived from **comparing two protocols** turned out to be noise at one draw each: whether the action table appears, how many figures are kept, whether the context matters, whether speaker labels matter, whether corrected spelling matters. Figures range 23 to 31 across seeds at a fixed setting, which is wider than any difference between settings.

Every measure that is a **direct count** held: terms spelled correctly against a written reference, occurrences a correction changes, candidates an extractor offers, segments a transcriber flagged.

The rule that follows: a protocol comparison needs several seeds before it is written down, and a claim that can be made as a direct count should be.

### Reading a draft against the written protocol

The first such comparison, 16 August 2026, gemma4:12b at seed 7. Details naming the project are in `eval/COMPARISON.md`, which is not tracked.

What survived the comparison well: every quantity checked was correct, including areas, grid dimensions, room counts before and after a reduction, and a funding ceiling. The draft also reproduced the meeting's judgement of six design approaches — which were rejected, which were kept, and why — from the words alone.

What failed:

1. **No actions table.** The written protocol closes with seventeen rows of task and owner. The draft has none. This is the part of a protocol a reader acts on.
2. **It ends a third early**, mid-subject, and the last thing in the file is the model's own JSON scaffolding leaking into the document.
3. **Every proper noun is wrong**, for the transcription reasons above.

Faults 1 and 2 are one fault: the draft ran out before finishing, and nothing marked the place where it stopped. A reader cannot tell a protocol that ended from one that was cut off.

This is the first evidence about whether milestone 1 is met, and it says no — for reasons that are specific and addressable rather than diffuse.

### The acceptance test, 17 August 2026

The first draft produced with everything from 16 August in place — the corrected transcript, the action-table check, the tidying pass, the marked gaps — read against the written protocol. `gemma4:12b`, seed 7, 793 seconds, 30 of 39 figures.

**All three faults found in the first comparison are fixed.** Every name is right, including a structural engineer who had been filed under a discipline he does not practise. The actions table exists and its four rows are real actions from the meeting, one with an owner. The draft ends properly, with dates, rather than in the model's own scaffolding.

**A fourth fault sits underneath them, and its size is not yet established.** The first attempt to measure it was wrong and is recorded here rather than quietly fixed. Ten subjects from the written protocol were searched for in the draft, nine were absent, and that was written down as "the draft silently omits subjects" — without checking whether the transcript contained them. Six of the nine are not in the transcript under any spelling tried, so the model could not have written them and was right not to. Checking a draft against a reference without checking the source is the same error this document has recorded four times already.

What survives the correction, across two seeds:

| Subject                                         | In the transcript            | In the draft          |
| ----------------------------------------------- | ---------------------------- | --------------------- |
| The district office and the appointment with it | yes                          | **no, at both seeds** |
| The soil survey                                 | not under any spelling tried | —                     |
| Bundling questions through one person           | not under any spelling tried | —                     |
| The 90 cm access width                          | no                           | —                     |
| The pilot standard, dates                       | yes                          | yes                   |

So there is at least one subject discussed and not recorded, consistently at both seeds. Whether that is one omission or many needs a reading rather than a word search, and the two subjects absent from the transcript are their own question: the written protocol records them, so either they were said in a form transcription lost entirely, or the author knew them from outside the recording.

**What reading three drafts found, which counting did not.**

All three carry an action table, so the check works. Figures held at 26 to 30 of 39. Seed 101 is a credible protocol: six numbered sections, 18,772 characters against a written 17,879, seven real actions, and specific detail — a 12 m² room too narrow for its meter cabinets, the 1.50 m clearance they need, the access balcony at one building not running the whole way round.

Three faults a count cannot see:

- **The same speaker label appears twice as two different people**, once under electrical planning and once under fire safety. The transcript it was written from has no speaker separation, so every segment is "Speaker 1" and the model faithfully lists that label under two roles. A reader meets a participants list naming the same person as two disciplines.
- **Words corrupted in the middle**: "wurdeerückt" for "wurde eingerückt". Not the model's scaffolding, which the tidying pass removes, but a word damaged inside a sentence.
- **Every owner reads "Nicht angegeben".** This is correct: the style forbids guessing an owner the source does not give. It is also the clearest measured difference from the written protocol, where a person assigns owners from knowing who does what. A protocol that never attributes an action is honest and half as useful, and no change to the model fixes it.

The lengths are worth keeping in view while that is settled: 9,591 characters at seed 7 and 18,772 at seed 101, against a written protocol of 17,879. Length varies by a factor of two between draws and the same subject is missing from both, so length is not what decides coverage.

## Protocol generation

### The first pass, and why its verdicts have expired

Several local models were tried early, against the generation path as it then was:

- `qwen2.5-coder` produced short drafts and could not hold the full meeting comfortably;
- `gemma2:9b` failed with a small context window;
- `gemma4:12b` was too slow or memory-heavy for the available machine;
- `qwen3.5:4b` with a long context produced a full-length German draft in about 10 minutes 51 seconds;
- the same model with project vocabulary produced a shorter, more useful draft in about 6 minutes 03 seconds, with improved proper names.

**Those verdicts describe a pipeline that no longer exists, and should not be read as
current.** They were measured before the generation path acquired sectioning for long
transcripts, before context limits were discovered from the provider rather than
assumed, and before the output ceiling and request deadline were corrected — the
faults listed under "What went wrong in early runs" below. A model judged too slow or
too heavy was being asked to hold a whole meeting in one request against limits that
were wrong.

`gemma4:12b` is the clear case. It was set aside here as too slow or memory-heavy;
measured against the current path it is the most accurate and by far the most stable
of the installed models, and the owner's own experience of using it agreed before any
of this was measured. The rejection was not a mistake at the time. It expired, and
nothing in this document said so until it was measured again.

That is worth generalising: a model verdict is only true of the pipeline that produced
it, and every entry in the list above predates three separate corrections to that
pipeline.

The exact output is not the acceptance criterion. The important questions are whether the protocol keeps decisions, actions, figures, and attribution, and whether a person can correct it without rewriting the meeting from scratch.

## What went wrong in early runs

The evaluation exposed several engineering problems:

- a 30-second HTTP deadline was too short for a local model;
- output limits discarded otherwise useful long drafts;
- input and output budgets were not separated clearly;
- first-token delays looked like a hung process;
- reasoning models sometimes returned content in a separate `thinking` field;
- a 128 KB response cap was too small;
- asking the model to write each topic independently caused it to rewrite almost the entire transcript.

Those findings led to longer bounded requests, context-limit discovery, sectioned generation, and the current investigation into structured evidence and mechanical checks.

## Facts and coverage

Plain code found nineteen quantities in the transcript. One early generated protocol accounted for only one. This is a useful baseline because it does not depend on the model judging its own work.

Coverage alone is not enough. One topic-by-topic run covered 23 of 24 figures but produced roughly 74,000 characters—almost the length of the transcript. A professional protocol is not a transcript with headings.

## Does attributing speech to speakers improve the protocol?

The question underneath every hour spent on speaker separation, asked for the first
time on 15 August 2026. Three protocols from the reference meeting, one model
(`qwen3.5:4b`), the shipped formal-minutes style, the same seed and temperature,
differing only in the speaker labels the generator was given:

| Labels                                        | Distinct |  Time | Characters | Figures kept | Invented |
| --------------------------------------------- | -------: | ----: | ---------: | -----------: | -------: |
| none — every segment `Speaker 1`              |        1 | 599 s |     18,643 | **24 of 35** |        4 |
| grouped — the embedding pass                  |       12 | 488 s |     15,965 |     20 of 35 |        3 |
| scattered — a diarisation run without a count |       54 | 308 s |     13,000 |     23 of 35 |        2 |

**On these measures the labels do not help.** The unlabelled run kept the most
figures; the grouped run kept the fewest and was shorter. The differences are small
and this is one model on one meeting, so the honest reading is not "labels hurt" but
"no benefit is visible here, and the burden is on the next measurement to find one".

More telling is that the labels barely reach the page. Across all three drafts the
string `Speaker N` appears **once**. The generator is handed twelve or fifty-four
distinct speakers and writes a protocol that attributes almost nothing — so
attribution is not happening whether or not separation works, which makes the
quality of the separation the second question rather than the first.

### Reading the protocols, rather than counting them

The measurements in this section are structural — headings, tables, figures kept.
That is the instrument this project already found gameable once, when a document
longer than its own transcript scored 23 of 24. So the drafts were read.

**`gemma4:12b` writes a protocol a professional could edit rather than rewrite.**
It opens with the participants grouped by discipline, each with their name and role
and the speaker label they were given. It numbers its sections and sub-sections
descriptively. It carries specific figures into the prose — areas, spans,
dimensions, deadlines — rather than rounding them away. It ends with the table of
next steps the style asks for, an owner against each row, and writes `Nicht
angegeben` where the meeting named nobody: the instruction never to invent an owner,
followed exactly.

It is the baseline as of 16 August 2026, on that reading and on the figures.

Two faults were visible only by reading:

- The model types a **literal backslash-n** as text, three to nine times per
  protocol, landing mid-sentence in a document somebody hands to a client. The JSON
  is unescaped long before that point, so these are two characters the model wrote.
  They are now repaired into line breaks.
- The **unowned-tasks evidence is defeated by the model behaving well.** It looks
  for an empty cell, and a model told never to invent an owner writes the absence in
  words, so the check reports nothing on precisely the rows it exists to raise. What
  was tried and reverted is recorded in `facts.rs`.

Neither would have appeared in any count.

### Is the model too small, or the prompt wrong?

Asked on 15 August 2026 by running the same transcript and the same shipped style
through the installed models, counting structure rather than reading prose. Then
asked again at two further seeds, which changed the answer.

| Model           | Seed |  Time | Headings | Tables | Figures kept |
| --------------- | ---: | ----: | -------: | -----: | -----------: |
| `granite4.1:8b` |    7 | 555 s |       23 |      1 |     22 of 35 |
| `granite4.1:8b` |  101 | 558 s |       26 |      1 |     19 of 35 |
| `granite4.1:8b` |  202 | 526 s |       16 |      1 |  **6 of 35** |
| `gemma4:12b`    |    7 | 827 s |        3 |      0 |     31 of 35 |
| `gemma4:12b`    |  101 | 841 s |       21 |      1 |     29 of 35 |
| `gemma4:12b`    |  202 | 816 s |       25 |      1 |     27 of 35 |
| `qwen3.5:4b`    |    7 | 317 s |       10 |      0 |     24 of 35 |

**The instructions are followable.** Both larger models produce the table of next
steps at most seeds, which `qwen3.5:4b` never produced in five runs. The prompt is
not the fault, and prompt engineering is not where the next improvement lies.

**Single-run comparisons between models are worthless here, and one was nearly acted
on.** The first sweep used one seed and produced two conclusions that do not
survive: that `gemma4:12b` does not produce tables — it does, at two of three seeds
— and that the two models trade structure against completeness, which was an
artifact of the one seed each was measured at.

**What does survive is a difference in reliability.** `granite4.1:8b` keeps 22, 19
and 6 of 35 stated figures on identical input: a run that loses five sixths of the
figures a meeting stated is not a tool for producing a record, and nothing in the
output announces which run it was. `gemma4:12b` keeps 27, 29 and 31 — a spread of
four against granite's sixteen, and better at its worst than granite at its best.

So `gemma4:12b` is the candidate, on both accuracy and stability, at roughly 1.5×
granite's time and 2.6× `qwen3.5:4b`'s. The endorsement `qwen3.5:4b` carries earlier
in this document predates any structural measurement and should not be relied on.

Generation is reproducible: the same seed, transcript and style return the same
document to the heading. That is worth having — evidence about a draft means little
if the draft cannot be reproduced — but it also means a "repeat" at a fixed seed
tests nothing, which is how the first sweep came to be believed.

One meeting, one style, German, three seeds.

### The European candidate, measured

`ministral-3:8b` is what the catalogue offers as its European option and had never
been run. Three seeds, same transcript, same style:

| Seed |  Time | What it produced                                                                                                                                                                | Figures kept |
| ---: | ----: | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -----------: |
|    7 | 866 s | A protocol: 12 KB, participants by organisation, 48 headings, the table with four owned rows                                                                                    |     28 of 35 |
|  101 | 438 s | **211 bytes.** A code fence, a heading, and two `[nicht im Transkript genannt]` placeholders, which the style forbids                                                           |      0 of 35 |
|  202 | 838 s | **A JSON document**, opening ` ```json ` and holding `metadata`, `participants`, `organisations` — asked for markdown inside a JSON field, it returned JSON inside a JSON field |     28 of 35 |

One usable protocol in three. `gemma4:12b` opens `## Projektbeteiligte` and produces
eleven kilobytes of protocol at every seed.

**The strongest case yet for reading rather than counting.** The JSON document scored
28 of 35 — better than Gemma at the same seed — because every figure is present as
text. By every structural measure it looked competitive, and it is not a protocol at
all. This project already learned that once, when a draft longer than its own
transcript scored 23 of 24; the same instrument failed the same way here.

Two of the three runs also produced JSON the application could not parse until the
repair described in `provider.rs` was added: the model writes markdown's line breaks
and real newlines inside a JSON string. That is a fault of the model rather than of
the transcript, and it destroyed a quarter-hour of work each time until it was
handled.

So Gemma remains the baseline and the European option is not yet a substitute. What
would change that is a model that respects the output contract, not a better figure
count.

### Two failures the same runs exposed, which matter more

**No draft produced a table of next steps.** The style says so explicitly and twice —
"End with a table of agreed next steps with two columns, the task and the
responsible party", and "The table of next steps must list every action that was
agreed". Three runs, zero tables.

That has a direct consequence for the evidence shown beside a draft: the
unowned-tasks check reads table rows, so on this model it can never fire. The check
is not wrong. It measures a structure the model does not produce, which is a
different problem and a worse one.

**The unlabelled run produced no headings at all** — 98 bullet lines and not one
heading, against a style asking for numbered sections with descriptive headings. The
labelled runs produced 12 and 25. So the labels did change the shape of the output
markedly, just not in the way they were meant to.

### What this changes

Making speaker separation more accurate is not the most valuable work available. A
protocol that ignores the participants, omits the table it was asked for, and in one
configuration has no sections at all is failing at things a reader notices long
before they notice a misattributed sentence.

It does not show that separation is worthless: recording microphone and system audio
on separate tracks would attribute the room against the remote participants for
free, and a person renaming a handful of labels in review gets attribution no model
will match. But it does mean the next protocol-quality work should be about
instruction adherence, and the speaker count and embedding model can wait.

Measured on an M1 Pro with 16 GB. One model, one meeting, one style: `granite4.1:8b`
and `gemma4:12b` are installed and untested here, and a model that does follow the
style would change every line of this section.

## Speaker separation

The reference meeting has **no known speaker count**. The owner attended, believes they were not
recorded, and spoke little; the true number is ten or eleven and nobody can settle which. Earlier
notes here treated eleven as fact. It was an estimate. Nothing below should be read as an accuracy
figure against ground truth.

Asked for eleven speakers, the whole recording produced 291 turns under eleven labels, of which only
eight are the majority speaker of any transcript segment.

Without a count, the same recording produced 86 clusters — one voice drifting across eighty minutes
of videoconference becomes many.

### Sampling instead of replaying the whole recording

Separation runs after transcription, so the segments are known, and placing a voice needs a couple of
seconds rather than a whole utterance. Two seconds from the middle of each of the 675 segments,
joined by 300 ms of silence, is 25.6 minutes of audio in place of 81.8.

Both runs asked for eleven speakers, on an M1 Pro with 16 GB:

|                                     | Whole recording |      Sampled |
| ----------------------------------- | --------------: | -----------: |
| Audio embedded                      |        81.8 min |     25.6 min |
| Time                                |          1810 s |        498 s |
| Turns                               |             291 |          128 |
| Speakers landing on segments        |               8 |           10 |
| Largest speaker's share of segments |            58 % |         56 % |
| Largest speaker's share of time     |            62 % |            — |
| Longest unbroken run                |    126 segments | 126 segments |

The two agree on 596 of 673 comparable segments, 88.6 %, after matching labels between the runs —
numbering differs because it follows whoever speaks first.

Since neither is ground truth, agreement is what can be measured, and the shapes match closely: the
same dominant speaker at 56-58 %, the same 126-segment unbroken run. That run and that dominance are
therefore properties of the diariser on this audio, not artifacts of condensing it. Sampling
resolves a slightly longer tail, not a shorter one.

### What the speaker count is worth

Swept on the sampled audio, each run compared against the eleven-speaker one after matching labels
between them:

| Asked for | Labels used | Segments per label, largest first       | Agreement |
| --------: | ----------: | --------------------------------------- | --------: |
|         6 |           6 | `385 121 100 43 19 7`                   |      96 % |
|        10 |           9 | `385 121 100 25 16 11 8 7 2`            |      99 % |
|        11 |          10 | `381 121 100 25 16 11 8 7 4 2`          |         — |
|        14 |          10 | `381 121 100 25 16 11 8 7 4 2`          |     100 % |
|        20 |          14 | `378 108 69 31 25 16 13 11 7 4 4 4 3 2` |      92 % |

Eleven and fourteen produce identical output. Between 6 and 14 the answer barely moves, so a wrong
count in that range costs almost nothing. At twenty the top speaker is untouched while the second
and third are split — 121 to 108, 100 to 69 — which is the shape of damage nobody would notice.

Three ways of not asking were measured, and none of them works:

| Method                                                  | Result                               |
| ------------------------------------------------------- | ------------------------------------ |
| Distance threshold, whole recording                     | 86 labels                            |
| Distance threshold, condensed                           | 67 labels                            |
| Sparse precheck, every 4th segment, 6.4 min, swept 4-18 | no plateau; labels track the request |
| Plateau on the full condensation                        | holds 11 to 14, breaks at 20         |

The precheck measures its sample rather than the meeting. The plateau is two agreeing points in a
narrow window, not an estimator: a real one would keep answering ten for any request above ten.

### How the samples are cut

By copying byte ranges out of the 16 kHz mono working audio, not by asking ffmpeg. Two ffmpeg routes
were measured and rejected:

- a filter graph of one `atrim` per sample splits the decoded stream once per sample and reads it
  through for each; at 753 samples it had not finished after ten minutes;
- the concat demuxer builds the file in under four seconds but rounds each out point up to a packet
  boundary, measuring 1767.7 s where 1731.6 s was planned — about 48 ms per sample and accumulating,
  which would have read the meeting's last turns back against audio 36 seconds away.

Byte copying is exact by construction and takes under a second. A test condenses a recording whose
every frame records its own millisecond and checks each sample begins and ends where planned; both
ffmpeg routes fail it.

The isolated synthetic diarisation study found 88.2% frame accuracy on a 23.5-second three-speaker fixture, with 259 MB peak memory and 46 MB of model files. The fixture was clean, short, and non-overlapping; the embedding model was trained on Chinese. Long, noisy, overlapping, multilingual, and M1/8 GB tests remain necessary.

## Memory

Long context is expensive. One measured Qwen configuration used about 4.7 GB resident memory at a 40,960-token context on the 16 GB machine. Larger models and long contexts can push that machine into swap, so the M1/8 GB measurement is a release gate rather than an optimisation detail.

## What this evidence does and does not prove

It proves that the local boundaries are technically viable and that vocabulary and context handling matter. It does not yet prove professional protocol quality, broad multilingual quality, distributable runtime choices, or acceptable performance on the weakest target machine.
