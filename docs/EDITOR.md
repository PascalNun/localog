# The protocol editor

The owner's concept for the editor, written 19 August 2026, recorded here as given.
Section 20 onwards is this project's reading of it: what already exists, what the
concept implies that is not obvious, and the one decision that shapes the rest.

## Purpose

The Protocol Editor is the main working environment after LocaLog has generated a
protocol draft. It should feel like a calm professional writing tool rather than a
Markdown editor, AI chat interface, or full desktop-publishing application.

The editor should provide enough document functionality to produce a polished
professional protocol while deliberately avoiding the complexity of Microsoft Word,
InDesign, or a general-purpose publishing system.

The guiding principle is: **the document remains central. Additional functionality
appears contextually only when needed.**

## 1. Overall layout

The existing LocaLog application structure remains unchanged.

**Left sidebar.** The persistent application sidebar remains visible and provides
access to Projects, Protocol Styles, Vocabulary / Names & Terms, and Settings. The
editor must not introduce a second application navigation system.

**Main workspace.** The centre of the screen contains the protocol document. The
document should visually dominate the interface and use a comfortable readable width
rather than filling the complete application window. The editor should feel
document-like, but it does not need to simulate individual A4 sheets while writing.

Preferred behaviour: centred document column; generous horizontal margins; clear
typographic hierarchy; no permanent visible page breaks; subtle indication of
document width; smooth scrolling through one continuous document.

Pagination is primarily an export concern.

**Right inspector.** Contains document-specific functions, in tabs: Document,
Transcript, History. The inspector is contextual and may be collapsed. The main
document must remain usable when the inspector is hidden.

## 2. Editing model

The protocol is stored internally as Markdown-backed structured content. The user
should not need to think about Markdown during normal use. Normal editing should
behave like a lightweight word processor.

Supported content types for the first complete editor: paragraph; heading 1, 2, 3;
bold; italic; bulleted list; numbered list; quote; divider; simple table, if it can
be implemented cleanly.

Do not initially support: arbitrary text colours; arbitrary per-character fonts; text
boxes; multi-column layouts; floating images; complex page geometry;
desktop-publishing tools; paragraph background styling; advanced typographic effects.

The purpose is professional protocol editing, not general document production.

## 3. Formatting interaction

Formatting should not require a large permanent ribbon.

**Default editor toolbar.** The permanent toolbar should remain minimal: Undo, Redo,
Find, Zoom, optional document menu. The current document save state may also appear
subtly in this area — "Saved just now", or "Saving…". Do not overload this bar with
all formatting functions.

**Contextual selection toolbar.** When text is selected, a small floating toolbar
appears near the selection: Bold, Italic, Heading level, Bulleted list, Numbered
list, Quote, Link, More. The More menu may contain: Insert table, Divider, Clear
formatting, Markdown view. The contextual toolbar disappears when the selection is
cleared. This keeps the normal editor visually quiet.

## 4. Markdown view

Markdown remains useful as an advanced representation but should not be one of the
primary visible modes. Normal users should remain in the document editor.

Markdown view may be accessible through the document menu, the contextual More menu,
a keyboard shortcut, or advanced settings.

Switching between document and Markdown representations must preserve the document
deterministically. Markdown must remain the canonical document representation, or
have a clearly defined canonical relationship with the editor state.

## 5. Document typography

LocaLog should support document-wide typography settings, controlled at the
document-style level rather than through arbitrary local font changes.

- **Font family.** Default Barlow. The user may choose another supported font. The
  first implementation should preferably use a controlled font selector rather than
  expose every low-level typographic property.
- **Body size.** Presets: 10 pt, 11 pt, 12 pt, 13 pt.
- **Heading scale.** Compact, Standard, Large.
- **Line spacing.** Compact, Comfortable, Spacious.
- **Document width.** Narrow, Standard, Wide, A4-like.

These settings apply consistently to the complete protocol. Avoid creating situations
where different paragraphs accidentally use different fonts and unrelated sizes.

Document appearance should normally be edited through the right-hand Document
inspector. Selecting "Edit appearance" should preferably transform or expand the
inspector rather than opening a large blocking modal. A temporary preview panel is
acceptable if necessary, but the central document should remain visible whenever
possible.

## 6. Header and footer

Supported because professional protocols frequently require repeated project
information. They are document/export properties rather than normal editable body
content.

**Header.** Left, centre and right content. Possible dynamic fields: project name,
meeting title, project number, meeting date, client, protocol status, custom text.
Optional settings: show on first page; hide on first page; different first page.

**Footer.** Project name, document type, date, page number, page count, custom text.
For example: `Nordenstadt | Protocol | Page 3 of 6`.

Settings live in the Document inspector under a collapsible "Header & Footer"
section. Selecting it may open an expanded side editor or lightweight side sheet.
Avoid forcing users into a large modal unless the configuration genuinely needs more
room. A small live preview is useful.

## 7. Protocol style

The selected Protocol Style remains separate from visual formatting. A Protocol Style
controls content-generation behaviour — Formal Minutes, Internal Working Note,
Decision Log, Task Summary — and may define expected document structure and language.
Document Appearance controls how the generated document is visually presented.

**These concepts must not be conflated.** For example: Protocol Style "Formal
Minutes"; Document Appearance "Barlow, 11 pt, Standard headings".

## 8. AI-assisted editing

AI functionality should be integrated contextually rather than through a permanent
chat box. LocaLog is not a chat-first application.

**Selected-text refinement.** Rewrite, improve clarity, shorten, make more formal,
summarise, adjust tone, custom instruction. Reached through the contextual toolbar, a
context menu, or `⌘K`.

**Custom instruction** opens a small temporary command field — "What should change?"
— for example "Make this point shorter and retain the technical terminology". After
the operation completes, the input disappears.

**Whole-document refinement.** The Document inspector may also provide a restrained
"Refine protocol" command. This should not be presented as a permanent AI
conversation.

## 9. Transcript relationship

The protocol must retain a clear relationship to its transcript. The Transcript
inspector tab should eventually allow the user to view relevant transcript passages,
jump to timestamps, compare protocol content with source material, and later inspect
source references.

For the first editor implementation it is sufficient to provide quick navigation back
to the transcript, meeting/transcript identity, and a later extension point for
source-linked protocol sections. The editor should not lose project or meeting
context.

## 10. Autosave

Normal editing is protected through continuous autosave. Visible states: Unsaved,
Saving…, Saved, Save failed.

Autosave must not create a formal revision after every keystroke. The editor must
remain responsive while saving occurs. Closing or navigating away should not silently
lose changes.

## 11. Revision history

LocaLog uses a restrained revision system rather than full document-version-management
software.

A formal revision may be created when a protocol is initially generated, regenerated,
explicitly created by the user, marked reviewed, or when an older revision is
restored. Normal typing remains part of the current autosaved working state.

**Review belongs to one exact revision.** Visible states: Draft, Reviewed, Changed
since review. If a reviewed protocol is later edited, the reviewed historical
revision remains preserved, the current state becomes "Changed since review", and the
user may review the new state again.

The History tab may show a simple chronological timeline: draft generated, edited,
reviewed, changed since review, restored. The detailed history remains secondary and
does not clutter normal editing.

## 12. Mark reviewed

The Document inspector contains the current document status. For a draft: status
"Draft", action "Mark reviewed". After review: "Reviewed". After later editing:
"Changed since review".

This behaviour should remain visually clear but restrained. Review is a professional
workflow feature and should not dominate the editing surface.

## 13. Find, undo and redo

The editor must provide reliable standard editing behaviour: undo, redo, find, find
next / previous, keyboard shortcuts, standard text selection, copy and paste,
keyboard navigation. These should behave as users expect from a native desktop
writing tool.

## 14. Zoom and text scaling

The user should be able to change the editor display scale without changing the
actual exported typography. Suggested controls: zoom out, current zoom percentage,
zoom in — `− 100% +`. **This is separate from the document's actual body font size.**

## 15. Export

Access to export from the Document inspector. Initial formats: Markdown, plain text.
Later: DOCX, PDF. Export settings may eventually use the selected document
appearance, header/footer, page format, and export template.

The editor should not become a print-layout environment merely because export
functionality exists.

## 16. Inspector behaviour

The right inspector should avoid becoming permanently overloaded. Use progressive
disclosure. An example Document tab: Status (Draft / Mark reviewed); Protocol Style
(Formal Minutes); Appearance (Barlow, 11 pt, Standard headings / Edit appearance);
Header & Footer (Project / Date / Page number / Edit); Export.

Each section may expand when required. The inspector itself may be collapsed to
maximise writing space.

## 17. Avoid modal-heavy interaction

LocaLog's general design principle is a stable central workspace.

Prefer: inspector expansion; contextual toolbar; small menus; lightweight side
sheets; a temporary command palette.

Avoid: frequent large modal dialogs; full-screen configuration screens for minor
document changes; multiple overlapping windows; permanent floating tool palettes.

Large modal dialogs should only be used when the user is performing an infrequent
task that genuinely requires focused configuration.

## 18. Visual direction

Consistent with the existing LocaLog references. Use Barlow for the UI; warm cream
light mode; warm charcoal dark mode; restrained accent colour; thin separators;
generous whitespace; clear typographic hierarchy; subtle selection states; minimal
visual noise. **The document itself should be the strongest visual element.**

Avoid: generic SaaS dashboard styling; bright blue actions; excessive cards; heavy
toolbar chrome; large coloured buttons; AI-themed visual effects; unnecessary avatars
or account controls.

## 19. Design principle

The editor should feel capable without appearing complicated. A new user should be
able to open a generated protocol and immediately edit the text without learning the
interface. More advanced functionality should reveal itself only when requested.

The desired experience is: simple writing surface → contextual editing tools →
document controls in the inspector → deeper options only when necessary.

LocaLog should approach the usefulness of a lightweight word processor while
retaining the calmness and clarity of the rest of the application. The editor is not
intended to replace Word or InDesign. It is intended to make editing, reviewing and
exporting professional meeting protocols unusually straightforward.

---

# Reading the concept against what is built

Added 19 August 2026 by this project, not by the owner.

## What already exists

- The document is edited directly, not as Markdown, with Markdown kept as the stored
  form. A round-trip test requires that a protocol nobody edited does not change when
  the editor merely opens it — §4's "deterministically" made into a test.
- Headings, bold, italic, both lists and quotations, applied from a toolbar.
- One renderer shared by the screen, the PDF and the Word file, so §15's "export
  settings use the document appearance" has one place to read from rather than three.
- Export as Markdown, plain text, **DOCX and PDF** — §15 lists the last two as
  "later"; they are done.
- Autosave with Saving / Saved / Save failed, and revisions separate from typing
  (§10, §11), including "Changed since review".
- Find, undo and redo (§13) — but see the gaps.
- Text scaling (§14) — but it is not yet separate from the exported size, which is
  precisely what §14 asks for.

## What the concept asks for and does not exist

Ordered as the concept orders it, not by size.

1. **The permanent ribbon should not be permanent** (§3). What exists is the opposite
   of what is asked for: a fixed row of formatting buttons, with the document/Markdown
   switch given equal prominence to the document itself. Wanted: a minimal permanent
   bar — undo, redo, find, zoom, save state — and a floating toolbar that appears on
   selection and leaves when it goes.
2. **Markdown should stop being a primary mode** (§4). It is currently one of two
   equal choices at the top of the editor. It belongs behind a menu.
3. ~~**Document typography is not settable at all**~~ (§5). **Done, 20 August.** All
   five settings exist — font, body size, heading scale, line spacing, page width —
   and all five reach the screen, the PDF and the Word file, because one module
   decides what "11 pt, standard headings, comfortable" means and the three surfaces
   read it rather than each interpreting it. A test requires that a project set to
   Georgia at 13pt produces a Word file naming Georgia at 26 half-points, since a
   setting that changes the screen and not the export is worse than no setting: the
   document somebody approved would not be the document their client opens.

   Held by the **project**, not by each protocol, on the grounds that the reason
   anybody sets it is that a firm's documents should look alike. The panel says so
   in a line rather than leaving it to be discovered. If that is the wrong unit it
   is a small change.

   Verified in the running application: Barlow → Georgia, 11 pt → 13 pt (14.67 →
   17.33 px), leading 23.8 → 32.1, measure 675 → 555 px, and heading 1 26.6 → 41.7 px.

4. ~~**Header and footer do not exist**~~ (§6). **Done, 20 August.** Three slots in
   each of the two bands, filled with fields rather than typed text — project name,
   meeting title, meeting date, document type, status, page number, page n of m, and
   custom text for anything the list does not cover. Held by the project, beside the
   appearance and for the same reason.

   Word gets real header and footer parts with its own `PAGE` and `NUMPAGES` field
   codes, so the page number is counted rather than written. The print path repeats
   the furniture on every sheet using fixed positioning, which is the only way a
   browser will do it — and it **cannot** supply a page number, because nothing in
   the page knows what page it is on. That field is left out of the PDF rather than
   printed as "Page 1" on every sheet, and the editor says so where somebody adds it.

5. **The inspector has no tabs** — ~~done, 20 August.~~ Document, Transcript and
   History, with the evidence panel moved to Transcript because comparing what the
   protocol says against what the meeting said is the job §9 gives that tab.
6. ~~**Tables cannot be edited**~~ (§2). **Done, 20 August.** A row above, a row
   below, a row removed; a column either side, a column removed; and a new table from
   the document menu. Controls appear when the caret enters a table rather than when
   text is selected, because nobody selects text in order to add a row.

   This is the one part of the concept `execCommand` has no answer for at all, and so
   the first real test of the decision to own the operations rather than adopt an
   engine. They are about a hundred and twenty lines against the document, and the
   result is read back to Markdown like every other edit. Removing the last row or
   the last column removes the table, which is what somebody emptying a table means.

   Verified in the running application by shape and by what was stored: 3r×2c → 4r×2c
   → 4r×3c → 5r×3c → 5r×2c → 4r×2c, with text typed into a new row arriving in the
   Markdown in the right cell.

7. ~~**AI-assisted editing does not exist**~~ (§8). **Done, 20 August**, for
   selected-text refinement: improve clarity, shorten, make more formal, make
   plainer, and a custom instruction. Contextual and momentary as the concept asks —
   a menu on the selection toolbar, gone when the selection goes, and never a
   conversation. The passage travels to the model **alone**: no transcript, no
   meeting, no vocabulary, because the job is to say the same thing differently and
   anything else the model could see is something it could add.

   **Measured against the real model before being trusted, and it does not hold.**
   Nine rewrites of three German passages on `qwen3.5:4b`: form survived 9 of 9 — a
   list item came back a list item, a heading a heading — but **facts changed in
   three of twenty-four**. `2. Obergeschoss` became `Obergeschoss (Etage II)`; `KW
38` became `Woche 38`, twice. The instruction to reproduce every figure exactly
   is there and is not enough.

   The first answer was to check the result: look for every number of the passage in
   what came back. That catches the floor number and **not** `KW` becoming `Woche`,
   because there the number survived and only the abbreviation changed. A checker
   good enough to catch that is a checker that understands German — which is the
   thing being checked.

   So the answer is not a better checker. **A rewrite is a proposal, not an action.**
   It is shown as a word-level difference — struck through where words went, marked
   where they came — and nothing is applied until somebody has looked at it and said
   so. The figure check is still run and named in the panel, but it is now a hint on
   a change somebody is already reading rather than the only thing standing between
   a model and the record.

   **A second model pass was then measured too**, because the owner asked whether
   one would catch what a token check cannot. Eight pairs — four with a fact
   genuinely altered, four merely reworded — put to a checking prompt:

   | Asked for                      | Caught | False alarms |
   | ------------------------------ | ------ | ------------ |
   | `qwen3.5:4b`, boolean + schema | 1 of 3 | 0 of 5       |
   | `gemma4:12b`, boolean + schema | 0 of 3 | 0 of 5       |
   | `qwen3.5:4b`, list the changes | 3 of 4 | 4 of 4       |
   | `gemma4:12b`, list the changes | 3 of 4 | 1 of 4       |

   Three things come out of that. **The boolean was the fault, not the idea**:
   `gemma4:12b` wrote "The date was changed from September to October" and returned
   `factChanged: false` in the same answer. Asked instead to list what differs, it
   found the invented "einstimmig genehmigt", the altered floor and the changed
   month. **Size decides usefulness**: the 4B model finds them too and flags
   everything else as well, which is the same as finding nothing. **And its one
   false alarm is arguable** — it objected that "hat zugesagt, zu nennen" became
   "nennt", which is a commitment turning into an act, and is a fair objection.

   `ministral-3:8b` (8.9B) was then measured too, and matched the 11.9B exactly —
   three of four, one arguable objection. So the line goes at **7B**, between a size
   measured useless and a size measured useful, rather than at a round number
   somebody liked. A test pins it to those three readings.

   Even so it missed `KW 38` → `Woche 38`, as everything has. **Built, 20 August**,
   as another hint on the proposal panel where a capable model is installed — never
   a gate, and never in place of reading the difference. On a real passage it named
   the floor abbreviation and the date reformat that the figure check also caught,
   plus three rewordings it treats as facts and a person would not.

   Two faults surfaced only by running it against a real runtime. The capability
   check read `parameterSize` while Ollama says `parameter_size`, so **every model
   arrived sizeless and was quietly judged too small** — which looks exactly like
   "no capable model installed". And the rewrite came back with the floor and the
   date in bold, which the passage never had and the instruction forbade; emphasis
   the passage did not have is now removed. Both have tests.

   Two other answers were considered and rejected. **Stricter instructions**: the
   prompt already says to reproduce every figure exactly, and the model altered them
   anyway; asking harder does not make a four-billion-parameter model obey.
   **Checking the rewrite against the transcript**: a protocol legitimately
   paraphrases the meeting, so "does this sentence match the source" has no clean
   answer and would be slow and full of false alarms. That machinery exists where it
   belongs — on generation, where the evidence panel counts which stated figures the
   draft kept.

8. **The Transcript tab does not exist** (§9).
9. **A History timeline does not exist** (§11), though the revisions behind it do.
10. **Zoom is not separate from body size** (§14).

## Speed, measured rather than assumed

The owner restated on 19 August that the program being **lightning fast** is a key
requirement, not a preference, and that writing something in Rust rather than
TypeScript is the right answer whenever it is faster.

That bears directly on the editor, because the editor does the most work per
keystroke of anything in the application: it reads the whole document out of the DOM
and writes the whole thing back to Markdown on every change. So it was measured
before anything was decided about it.

On a synthetic protocol of **39,456 characters** — longer than the real ones seen so
far — on this machine:

| Step                             | Cost      |
| -------------------------------- | --------- |
| `readBlocks` (Markdown → blocks) | 0.14 ms   |
| `renderMarkdown` (blocks → HTML) | 0.69 ms   |
| `toMarkdown` (tree → Markdown)   | 0.31 ms   |
| Walking the live DOM, in the app | ~1.8 ms\* |

\* measured at 0.27 ms over 6,024 characters and scaled; the walk is linear in nodes.

So a keystroke in a very long protocol costs roughly **2 ms**, against the 16.7 ms a
frame allows. Re-rendering the HTML is not on that path at all — it is deliberately
skipped while somebody is typing, because replacing the element under the caret would
throw them to the top of the document.

**Conclusion: this path is not a bottleneck and should not be optimised.** Writing it
in Rust and crossing the bridge twice per keystroke would very likely be slower, not
faster, because the cost here is the DOM walk rather than the parsing, and the DOM is
only reachable from the webview. The number to watch is the DOM walk: if protocols
reach a few hundred thousand characters, or if the walk shows up in a profile, the
fix is to read only the block that changed rather than to move languages.

## Paste and list keys — done, 20 August

**Paste** now goes through the document's own vocabulary on the way in. What arrives
from Word is a thicket — styled spans, `MsoListParagraph`, `mso-` attributes, fonts
and colours — and the reader that takes this document back to Markdown already knows
how to ignore all of it. So the sanitiser is a round trip through it: parse what
came, read it to Markdown, render that back. Anything the protocol vocabulary does
not have cannot survive the journey, and the words inside it always do. Measured on
real Word markup: every word kept, bold and lists kept, zero `style` and zero `class`
attributes left behind.

**Tab and Shift-Tab** move a list item in and out, and Enter on an empty item leaves
the list instead of making another empty bullet. Measured: depth 1 → 2 → 1.

## Find, replace and undo — done, 20 August

**Find** only ever worked in the Markdown box: it read a textarea's selection, and
the document view has no textarea, so the button did nothing there. It now walks the
document's text nodes — over the text rather than the HTML, so a search for "table"
finds the word and not the markup — and wraps at the end. **Replace all** is the case
that asks for this, one name misspelt through a protocol: it works on the Markdown,
so it behaves the same in both views, and says how many matches there are before
anything happens and how many were replaced after.

**Undo now knows what a heading is.** The browser's own undo covers typing in an
editable region and nothing else, so adding a table row, removing a column, replacing
a name and accepting a rewrite all went past it invisibly — undo skipped exactly the
operations somebody most wants back. The history is kept over the Markdown, which is
the document, and typing is remembered in pauses rather than per keystroke so undo
steps back a phrase at a time.

One thing that only showed up by trying it: stepping back left the screen unchanged,
because a table row had been added to the document without going through the rendered
string — so after undo that string was _identical_ to what it already was, and
nothing identical is ever reapplied. The restored document is written to the surface
directly. Measured: 3 rows → 4 → 3 → 4.

## Wanted, from looking at it — 20 August

The owner's, recorded rather than built, and none of them urgent.

1. ~~**The toolbar should be the width of the page under it.**~~ **Done, 20 August.**
   The toolbar, the find bar and the rewrite panel all measure like the page and move
   with the page-width setting. Measured at all four widths: 469, 587, 675, 763 —
   toolbar and sheet identical at each.

   Two unit traps on the way, both invisible until measured. The measure was in `em`,
   which resolves against whatever element reads it: the sheet sets its own font size
   and the toolbar does not, so one "46em" came out 675px and the other 552px. It is
   an absolute length now. And the sheet carried `margin: 0 auto` as a grid item,
   which makes an item take its content's width rather than its share of the row — so
   it sat at 648px whatever it was asked for.

2. **Show where the pages break.** With A4 chosen, the editor should divide the
   document the way a word processor does, so it is always clear what falls on which
   sheet. This is the question from earlier answered in the affirmative: pages in the
   editor, now that a page size and a header and footer exist and there is something
   for a page to mean. It stays hard — pagination in an editable surface is the
   difficult part — and it is worth doing rather than avoiding.

   The header and footer should presumably be visible on each page too, since being
   able to see them is half the reason for having them.

3. **The appearance panel is not styled to the standard of the rest.** It works and
   it looks like a form. The reference shows something quieter.

## The decision that shapes the rest

**How much editing engine to write.**

The editor today is a `contenteditable` region driven by `document.execCommand`, with
the result read back into Markdown after every change. That was the right way to find
out whether the shape works, and it has three limits that the concept runs into
directly:

- `execCommand` undo is the browser's, and it does not know what a heading is (§13
  wants undo that behaves as a native writing tool's does).
- Nothing knows what the cursor is standing in, so a heading control cannot show the
  current block (§3's "Heading level" in the contextual toolbar implies it).
- Table structure — adding a row, removing a column — has no equivalent in
  `execCommand` at all (§2).

Three ways forward, and the choice is the owner's:

- **Keep `contenteditable`, own the operations.** Read the selection to know the
  current block, and write the block and table operations directly against the DOM
  rather than through `execCommand`, keeping Markdown as the record of truth for
  undo. No dependency. Most of the work is in selection handling, which is where
  editors are genuinely difficult.
- **Adopt an editing engine** — ProseMirror or similar. It brings a real document
  model, reliable undo, table commands and selection state, all of which the concept
  asks for. It is a substantial dependency that would own the editing surface, which
  is against this project's habit of not installing what it can keep small.
- **Keep it as it is** and accept that the editor stays a lightweight text surface,
  declining §2's tables and §3's heading control.

**Settled, 19 August: the first.** The owner's instruction that the program be
lightning fast and the code minimal, and that a large runtime dependency is the wrong
shape for this project, rules out the second — an editing engine is precisely a
framework that would own the editing surface. It is also the right answer on its own
merits: the supported content types in §2 are a short and closed list — paragraph,
three headings, bold, italic, two lists, quote, divider, table — and a closed list is
the case where owning the operations is tractable.

The measurements above say the cost of owning them is affordable. What remains
genuinely hard is selection handling, which is where editors are difficult, and that
is hard in every one of the three options.
