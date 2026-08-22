# The running header and footer

What repeats at the top and bottom of every printed page, how somebody sets it up,
and what each of the three exports can honestly carry.

Written after a survey of the tools this audience already uses, and after
measuring what the two export paths actually do rather than what the code says
they do. The measurements are marked as such.

## The finding that decides the design

Every tool a professional office might already have converged on the same
authoring unit, independently:

| tool | what the person manipulates |
| --- | --- |
| Microsoft Word | a line typed into the header area; dynamic values inserted **into that line** from a menu of pictures, never as syntax |
| Google Docs | the same, with a deliberately tiny dynamic vocabulary — page number and page count and little else |
| LaTeX / fancyhdr | `\lhead{Seite \thepage}` — a line per slot, token inline |
| wkhtmltopdf | `--header-left "Seite [page] von [topage]"` — a line per slot, token inline |
| Chrome print | `headerTemplate`, an HTML fragment with five documented magic classes |

Nobody makes a person assemble a list of atoms. Everybody lets them write one
line and put a token in the middle of it.

LocaLog today is closest to `fancyhdr`: two bands times three slots, declared
away from the page, no preview until you export. The storage is right. **What is
wrong is that a slot cannot hold a sentence.**

## Why that is a small change and not a redesign

A slot already stores `FurnitureField[]` — an ordered list of runs, where a run is
either a token (`{ kind: 'projectName' }`) or literal text (`{ kind: 'text',
value }`). That is exactly a line with tokens inline. Three things hide it:

1. `resolveRow` joins the runs with `' · '` instead of concatenating them, so
   `Projekt: ` + `«project»` comes out `Projekt: · Neubau Halle 4`.
2. `resolveField` calls `.trim()` on a text run, so a run cannot carry the space
   that would sit between it and the next token.
3. `FurnitureEditor.svelte` presents the runs as chips with an `Add…` select,
   which is a list-of-atoms interface over a line-of-text model.

**No schema change, no migration.** Existing furniture keeps working: a row of
tokens with no text between them renders the same as it does now, because the
separator becomes something the person types rather than something imposed.

## The design

**Storage** — unchanged. `PageFurniture { header, footer, skipFirstPage }`, each
band with `left`, `centre`, `right`, each slot a `FurnitureField[]`.

**Resolution** — `resolveRow` concatenates runs verbatim. Text runs keep their own
spacing; the person types `Seite ` and `­ von ` themselves.

**Unresolvable tokens.** A token that *cannot* be answered in a given output is
different from one that answers with nothing. Today both come out empty and the
neighbours survive, so the natural footer — the word `Seite` beside a page
number — prints `Seite · 3` in Word and the bare word `Seite` on every page of the
PDF. The rule instead:

> If any token in a slot cannot be resolved for an output, that slot is omitted
> entirely for that output.

So the PDF loses the page-number slot rather than printing a fragment, and the
other two slots are unaffected. It is explainable in one sentence to the person
setting it up, which is the test.

**Authoring** — each slot is one line. Typing goes into it directly. A token is
inserted at the caret and behaves as a single object: it selects and deletes
whole, and it reads as its label (`«Seitenzahl»`) rather than as syntax. Six such
lines, laid out as they sit on the page: three across the top, three across the
bottom.

**A preview.** Every tool surveyed either shows the header in place on the page
(Word, Google Docs) or shows nothing at all until export (fancyhdr, Pandoc).
LocaLog can do better cheaply, because the editor already draws page boundaries:
the resolved header and footer belong in the gap between pages, where they are
already drawn as a summary. That gap is the preview.

## What each export can carry

Measured, not recalled.

**Word** (`docx.ts`) writes real `w:hdr` / `w:ftr` parts with a centre tab at 4819
twips and a right tab at 9638 — the same three-slot construction Word's own
default Header style uses, so the output is idiomatic. Page numbers are genuine
`PAGE` / `NUMPAGES` field codes, so Word counts them itself.

**PDF** (`print.ts`, through the browser) draws the bands as `position: fixed`
elements. Richer than Word in principle — anything CSS renders — but with two
limits, one of them currently a defect (below). Page numbers are impossible by
the technique: `counter(page)` has no value in flowed content. Measured: it
renders `0 / 0`.

**Markdown and plain text** carry no furniture, and no title or subtitle either.
They have no pages, so running furniture is meaningless in them. The honest
position is that they are out of scope, and that the `.md` file is not the same
document as the PDF and the `.docx`.

**The floor both real outputs share**, and therefore what may be offered without
qualification: two bands, one line each, three slots, holding text and the tokens
for project name, meeting title, meeting date, document type and status, in one
size and colour. Above that floor, exactly one cheap step keeps them in step —
bold or italic per run, and a hairline rule under the band. `docx.ts` already
emits both shapes and each has an exact CSS twin.

## Rejected: a box the person types HTML into

The owner raised it, and it deserves a straight answer.

It is genuinely attractive: unbounded, so no letterhead is ever unreachable, and
it matches how somebody who writes Markdown already thinks. `print.ts` builds its
sheet with `innerHTML`, so the PDF side would be nearly free.

It is rejected because **Word cannot consume it.** A `.docx` header is OOXML, not
HTML; there is no path from one to the other short of a converter. Pandoc — the
most code-forward tool surveyed — has no syntax for Word headers at all and tells
you to open the reference `.docx` *in Word* and set it there. So an HTML box buys
a rich PDF header and no Word header, which breaks the one property the whole
design exists to hold: that the two outputs agree.

The secondary reason is that the escape-hatch versions of this in other tools are
notorious for failing silently — a template that renders at font-size 0 unless you
set it explicitly, images dropped unless base64, the band clipped away entirely if
the page margins are not also set. That is a support burden this product's
audience should never meet.

What is actually wanted from HTML — *put a value in the middle of a sentence* — is
the design above, without the markup.

## Defects to fix first

These are independent of the authoring model, and several of them are why the
feature feels unfinished. Verified in the code unless marked.

1. **The PDF ignores the project's appearance.** `print.ts` sets
   `--document-font`, `--document-size`, `--document-leading` and
   `--document-measure` on the print root, and the 165 lines of `@media print`
   CSS reference none of them and set no `font-family` at all. A project set to
   Georgia 13 pt gets Georgia 13 pt in Word and the interface's own font at
   10.5 pt in the PDF. `EDITOR.md §5` states all five settings reach all three
   surfaces; that is currently untrue.
2. **The band prints inside the text column.** `position: fixed` resolves against
   the page area, and nothing reserves space for it, so the header lands on the
   first lines of the page. Measured in Chromium: a header string printed on the
   ascenders of the masthead title.
3. **`skipFirstPage` is dead.** Declared in `types.ts`, defaulted, present in a
   test fixture, read by no exporter. In Word it is small (`w:titlePg` plus a
   first-page reference). In the print path it is impossible by construction — a
   fixed element repeats on every sheet and there is no selector for "not page
   one". Decide it: implement it Word-only and say so, or remove the field.
4. **The values are English and ISO-dated.** `documentType` is hard-coded
   `'Protocol'`, status comes from `reviewStateLabel` as `Draft` / `Reviewed` /
   `Changed since review`, and the date is passed through as the stored
   `2026-07-29`. A German protocol header currently reads
   `Protocol · 2026-07-29 · Draft`. The date needs a format somebody chooses;
   `29.07.2026` and `2026-07-29` are not interchangeable in a German document.

## One lead worth measuring before betting on it

CSS page margin boxes — `@page { @bottom-center { content: counter(page) } }` —
**work in Chromium**, printing `1 von 2` correctly in the paper margin. That would
give the PDF real page numbers *and* fix defect 2, because a margin box lives in
the margin rather than the text column, and it would change what the header may
promise.

But macOS prints through WKWebView, not Chromium: `bridge.nativePrint()` reaches
`print_window`, which is `NSPrintOperation`, because `window.print()` does nothing
in that webview. An attempt to measure WebKit produced 1.1 million pages for a
two-line document, so the harness was wrong and **this is unverified on the actual
target**. Measure it there before designing around it.
