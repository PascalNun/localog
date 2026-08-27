# Current plan

This is the short answer to “where is LocaLog now, and what should happen next?” It is intentionally a current-status document, not a diary of every experiment.

The product and architecture documents describe the destination. The decision log records choices. This document describes what the code can honestly claim today.

Last reviewed: 17 August 2026.

## What 16 August established

**Transcription was the bottleneck, and it is largely fixed.** Every model comparison in this project had been run on a transcript made with the smallest whisper model and an empty vocabulary. Thirty proper nouns plus the larger model take fourteen counted terms from three spelled correctly to thirteen, and leave nothing for a person to correct. It does **not** produce a better protocol — measured, four runs, no difference. It fixes the document somebody circulates, which is a narrower claim and the true one.

**The context window is free to choose above 16,384.** Fifteen runs across five widths: quality does not move, time does — 16,384 costs about 2.4× the wall-clock of 40,960. 8,192 fails at every seed because the window cannot hold the folded notes and a whole protocol at once. The floor and the unknown-memory fallback were 4,096 and 8,192; both are now 16,384.

**A protocol has to fit in one answer, which caps meetings at roughly 1.8× the reference one.** Independent of the machine. Now said before the work starts rather than after three identical failures.

**Writing by topic controls length and coverage but not structure.** 74,000 characters became 26,100 with per-topic budgets, at 31 of 35 figures — the best coverage measured. The document is still organised by when things were said, with three separate facade sections. Structure is a global decision and per-section rules cannot make it.

**On method, at the cost of four withdrawn claims:** every conclusion drawn from comparing two protocols at one seed was noise — figures range 23 to 31 at a _fixed_ setting. Every direct count held. When something fails, measure the constraint rather than reasoning about the cause.

**On guards:** a hard cap set equal to the soft check it accompanies makes the correction unreachable, and a per-section fatal check aborted a twenty-three-minute run over 198 characters. Limits must be proportionate to what actually goes wrong.

## Found by using it, 17 August 2026

The owner ran the application and reported these. Ordered by whether the thing is
broken, then wrong, then missing.

### Broken — it does something other than what it says

1. ~~**"Record a meeting" led to a form whose button did nothing.**~~ Fixed. The
   button's `disabled` was changed and the submit guard was not, so it enabled and
   silently returned.
2. ~~**Export Markdown does nothing.**~~ Fixed. The capability granted
   `dialog:allow-open` and not `dialog:allow-save`, so the save dialog never opened,
   the call threw, and the code fell through to a browser download the desktop shell
   blocks. A failure now says so rather than trying a fallback that cannot work.

### Wrong — it says something that is not true

3. ~~**Settings recommends a model measured to be worse.**~~ Fixed, and the cause was
   not the catalogue — that was already ordered by what was measured. The picker asked
   `navigator.deviceMemory`, which WebKit does not implement and this shell is WebKit,
   so every macOS machine reported nothing, nothing was read as the weakest supported
   machine, and everything larger was filtered out. The backend already knew the answer
   exactly and now reports it.
4. **The protocol editor's text is narrower than its frame.** A layout fault, small
   and visible every time somebody edits.

### Missing — the thing it should do and does not

5. **The protocol editor should read as a word processor, not a text box.** Either a
   rendered view with a formatting bar — headings, lists, emphasis — or a switch
   between that and the Markdown source. Markdown stays the stored form; what is
   missing is that somebody editing a professional document should not have to know
   what `##` means.
6. **Exporting needs more than Markdown.** The protocol is the product and Markdown
   is not what a professional sends. Wanted, in the owner's order:

   - **DOCX**, because it is what a client edits and returns.
   - **PDF**, on a stated page size — A4 first, since the audience is German. Likely
     easiest as Markdown to HTML to print, rather than a PDF library: the styling then
     comes from the same stylesheet the editor uses, and page size is one CSS rule.
   - **A header and footer somebody can set**, edited rather than configured — the
     firm's name, the project, a page number. HTML is a plausible form for it, so the
     same content can serve the PDF and whatever follows.

   None of these is started. The order matters: DOCX and PDF are the deliverable, the
   header and footer are what make it look like the firm's document rather than the
   application's.

7. **The recording screen does not match the reference layout.** The interaction was
   built from the written direction; the reference images are the standard and it has
   not been held against them.
8. ~~**The protocol editor scrolls inside a page that also scrolls.**~~ Fixed, needs
   looking at. The editor was a fixed box with its own scrollbar inside a scrolling
   workspace: two bars for one document, and no way to see how long the protocol is.
   The editor now grows to its content and only the page scrolls.
9. ~~**The interface jumps when moving between Source, Transcript and Protocol.**~~
   Fixed. Source kept the standard page padding while the other two had been made
   denser, so every move between the stages shifted the whole page sideways — 23px at
   1280 wide, 20px at 1600. The three stages of one meeting now share one column by
   construction rather than by coincidence, so it cannot drift apart again. Measured
   before and after at both widths. Scroll position was suspected first and cleared:
   it already resets correctly on every stage change.
10. ~~**The `+` beside Projects does not line up with the meeting counts below it,
    and the one beside "New project" sits inside the project names.**~~ Fixed, and
    measured rather than eyeballed: a glyph centred in a hit target is not where the
    hit target is, so both are now aligned on their ink — 219.9 against 220 on the
    right, 27.6 against 28 on the left.

11. ~~**Recording could never start.**~~ Fixed. The row written when a recording
    begins used a kind and a state the table's CHECK constraints forbade, so every
    attempt failed and the person was told the workspace was unreachable.
12. ~~**The theme control never admitted that following the system was one of its
    states.**~~ Fixed twice: once in Settings, and then in the sidebar, where the
    same fault had been left standing. The small button showed a moon in light mode
    and a sun in dark — the theme a click would _produce_, which reads as a label
    for the current state and is the opposite of one. Automatic, which is the state
    most people are in, had no appearance at all: it was signalled by dimming the
    icon to 55%, which reads as "switched off", and by a tooltip nobody sees unless
    they go looking. Each of the three states now wears its own icon — a monitor, a
    sun, a moon — at full strength. Automatic was implemented and worked, but the control cycled
    through three states while naming only the one it would move to next, so somebody
    on automatic was never told. It is now three named states with the one in force
    shown, and it says which theme the Mac is set to.
13. **The Protocol styles page does nothing.** It lists the three styles and stops:
    they cannot be opened, read, edited, copied or added to. The page states that
    styles are presets rather than prompts and then offers no way to hold one.

14. ~~**A small mark beside the wordmark.**~~ Done, 20 August. The same waveform the
    application icon and the start page already use, cropped to its middle five bars
    so that it reads at the height of a word, and sized in `em` against the wordmark
    so the two stay one thing at any size.

### Open questions, which are the owner's

- **Should the editor show the document as pages?** An A4 page layout would make the
  editor and the PDF export the same thing seen twice, and would give the header and
  footer somewhere to live. It is also a much larger change than a flowing document,
  and it only makes sense alongside the rendered editor rather than before it.
- **Which comes first: the rendered editor, or the export formats?** They meet in the
  same place — a rendered document is what a PDF is printed from — so doing the
  editor first may make the exports nearly free, or may delay them by a week.

## Read against another harness, 24 August 2026

turnstonelabs/turnstone was read to see whether its approach had anything for this
one. It is an orchestration platform for tool-using agents, with a formal
definition of "harness" and numbered falsifiable claims. Most of it prices risks
this application does not run — adversarial tool output, prompt injection through
retrieved pages, cluster dispatch — and its own vocabulary would be a loss here,
where every object already has a plainer name.

**In its terms this application already is a harness**, and in one place ahead of
it: `beyond_one_answer` refuses before the first model call, which is the
closed-form halt condition the document says you generally cannot have. Its rule
that a learned judge may never be the verifier is one `check_rewrite` already
states better — "Never a gate."

One habit was worth taking, and four defects came out of taking its determinism
rule seriously. All four are fixed:

- Every protocol's stored provenance named two settings it was not generated with:
  8,192 context and 2,048 output tokens against a run resolving
  `affordable_context` (floor 16,384) and `output_tokens_for`. This project
  measured 8,192 as failing at every seed.
- Retries two and three asked byte-for-byte the same question — fixed correction
  text, pinned seed — so one of three attempts was free at best. The attempt
  number moves the seed now.
- `adherence.md` has never been a table: ten literals wrote a backslash and an n.
- The harness ran one seed and overwrote its own output, so the three-seed rule
  could not be followed with the tool provided. It runs every seed and prints the
  spread.

### Still open, found in the same pass

1. **A short meeting gets no retry at all.** `plan_sections` returning one section
   routes to `generate_in_one_pass`, which is one call, one parse, one validation,
   and `?` — no fold-back, no fail-open. `generate_from_sections` has all three,
   and commit 479d1f5 added them there with the measurement in its message: about
   one draw in five arrives without a table. The same fault is still live on the
   path short meetings take.
2. **The retry budget does not cover a bad draw.** `attempt(...)?` sends a model
   error straight out, so `ATTEMPTS_PER_STEP` protects only against a failed check,
   not against the malformed answer `parse_structured` exists for.
3. **`with_correction_or_keep` can lose the answer it exists to keep** — an error
   on a later attempt discards a complete protocol that arrived on an earlier one,
   at the step whose own comment calls it the most expensive to lose.
4. **The evaluation harness hand-types the fourteen style instructions** that
   migration 9 writes into every workspace. Byte-identical today; nothing checks it.

## What to do next, in order — 23 August 2026

Supersedes the 18 August ordering below, which is kept for its findings. Written
after the owner printed a real protocol and found the running header absent, and
after a cleanup pass that turned up why.

The principle this time: **make the thing work before making it nice, and make it
authorable before making it designable.** There is no point polishing how somebody
arranges a header that does not print.

**Where this list stands, 25 August 2026.** 1, 2 and 3 were carried out and the
reasoning held: the header now prints on every page because we cut the pages, and
the page number came with it. 4 has not been done — `export_templates` and
`save_export_template` are still commands and templates still hold their own page.
5 is deferred by the owner, and the header/footer grid and the letterhead logo wait
with it. The language decision below is still a decision.

### 1. Bind ⌘P to the export, not the browser's print

Fifteen minutes, and it removes a daily friction: the shortcut every document
application has, doing what the menu item already does. `Ctrl+P` elsewhere.

### 2. Paginate the print sheet ourselves

**The header does not repeat, and this is why.** The PDF path draws the bands with
`position: fixed`, which repeats per page in Chromium and does not in WKWebView —
which is what macOS prints through. Measured on the owner's own export: the header
printed once on page 1, the footer once on page 3, and page 2 had neither. The
mechanism the whole design rested on does not work on the target platform.

The answer is to stop asking the browser to repeat anything and build the sheet as
explicit page boxes, each holding its own header, its slice of the document, and
its own footer. The editor already computes where the pages break — `pageStarts`,
written for the visible separation between pages — and the same measurement drives
this.

Four things fall out of one change:

- the header and the footer appear on every page, in any engine
- **page numbers become possible in the PDF**, because we are the thing paginating
  and therefore know both the number and the count
- the band stops colliding with the first line, because it is inside its own page
  box rather than fixed over the text column
- `skipFirstPage`, which is stored and honoured by nothing, becomes trivial

### 3. Make a header slot a line somebody writes

Half done. The model already carries a line — text runs and value tokens in order —
and resolution now concatenates them, so `Seite 3 von 12` is expressible. What
remains is the interface: the panel still offers chips and an `Add…` select, which
is a list-of-atoms interface over a line-of-text model, and the spaces somebody
types are invisible inside a chip.

### 4. Fold export templates into the appearance panel as presets

A project already has an appearance and furniture. A template is only those two
under a name, so that another project can use them — which is a preset, and does
not earn a page in the sidebar beside Protocol styles and Names & terms. **Save as
preset** and **Use preset**, where somebody is already looking at the settings.
Deletes a concept and a page.

### 5. The band as a grid, and the logo

The design is written down in `HEADER_FOOTER.md`. It needs 2 first: a logo makes
the band taller, and until the band has a page box of its own that is a collision.

### Also, and needing a decision rather than an implementation

The values are English and the date is ISO: a German protocol's header currently
reads `Protocol · 2026-08-10 · Draft`. German and English with an English fallback
is the cheap version; all twelve meeting languages is a different piece of work.

## What to do next, in order

Rewritten 18 August 2026, after a day of using the application. The previous ordering
was written before recording had ever been tried and is superseded.

The principle: **prove what is claimed, then finish what the product hands over, then
make it authorable, then the small frictions.** Each item is finishable on its own —
none of them leaves the application half-changed if the next one never happens.

### 0. Prove a recording works from end to end

Recording could not start at all until today, and fixing the statement that failed is
not the same as recording working. Nobody has yet recorded a meeting in this
application and transcribed it. The system-audio tap depends on a permission macOS
grants silently: an application that does not have it is handed silence rather than an
error, so the failure looks exactly like a quiet room. Until one real recording becomes
a transcript, half of milestone 1 is unproven.

Cheap, and it gates everything else — there is no point polishing a recording screen
for a path that does not run.

**Underway, 18 August.** Three things are now known, and two of them were faults:

- The recorder itself works. Run for a minute it writes both tracks, reports a level
  every second, stops on `SIGTERM` and leaves no orphan behind.
- The system tap works, and this was worth proving rather than assuming, because an
  application without the permission is handed silence rather than an error. Played a
  tone at four percent, the system peak went from `0.000` to `0.005` for exactly as
  long as the tone lasted.
- **The FFmpeg LocaLog ships cannot combine the two tracks.** It is configured down to
  the filters the application uses — deliberately, and that is the right instinct —
  but the list was written before the recorder existed and does not include `amix`,
  which is precisely what combining a recording asks for. On this machine the failure
  was invisible: a full FFmpeg is installed and was found instead. On a machine that
  has only what LocaLog ships, every recording would have failed at the last step,
  after the meeting. Fixed by adding the one filter to the build and rebuilding.

The lesson is narrow and worth keeping: **the shipped build's filter list is a contract
with the code, and nothing checked it.** A test now does.

A recording now goes the whole way in a test: two tracks in, one combined file
committed with a checksum, at a path that exists. What is still unproven is the last
inch — clicking Record in the window and getting a meeting that transcribes. That
needs the macOS system-audio permission dialog, which only appears to somebody sitting
at the machine, so it is the owner's to confirm and not something a test can stand in
for.

### 1. Stop the interface jumping between Source, Transcript and Protocol

An hour or two. The three stages are coherent to edit in; getting between them throws
the page around. It is the most-repeated irritation in a day of use, and it makes the
next item pleasant rather than annoying to work on.

### 2. Let a protocol style be opened and read — done, 18 August

Small, and it removes a page that currently states a promise and offers nothing. Today
a style cannot be opened at all: the type the interface receives carries a name, a
description, a language and a density, and not one of the instructions that actually
shape the document. Somebody choosing between three styles is choosing between three
sentences.

This step is reading only — open a style, see what it asks for, see its density and its
required sections. Authoring is step 5, and is much larger.

Built. A style opens where it sits and shows what it actually asks the model for, in
the order it asks: fourteen instructions for the formal style, against the one
sentence the list used to carry. What the density setting means is said in words
rather than as a label. The page also says plainly that editing is not built yet, and
what will and will not be editable when it is, so the boundary is learned before
somebody goes looking for it.

### 3. The document arc — the protocol is what this is all for

The largest piece, and four steps that share one foundation, so they are worth doing in
this order rather than separately:

- **a. One Markdown-to-HTML renderer**, used by the editor and by every export. Written
  once. Everything below depends on it and nothing below duplicates it.
- **b. A rendered editor** with a formatting bar — headings, lists, emphasis, tables —
  and a switch to the Markdown source for anybody who wants it. Markdown stays the
  stored form. What is missing today is that somebody editing a professional document
  should not have to know what `##` means.
- **c. PDF on A4**, by printing the rendered document with `@page` rules. Nearly free
  once (a) and (b) exist, and it needs no PDF library and no new dependency.

**Done, 19 August: (a), (c), and the reading half of (b).** The renderer is written
rather than installed — a protocol's Markdown comes from prompts this project wrote,
a general library would carry raw HTML passthrough, and the same renderer is read by
the editor and by every export, so what it does has to be knowable. The protocol now
shows as a document with the Markdown one click away, at A4's text measure rather
than the window's, so the screen and the page break in the same places. The PDF is
that same document printed: an A4 sheet built beside the application, with the
heading-at-the-foot-of-a-page and split-table-row faults ruled out, and taken down
again afterwards.

**And (b) in full, later the same day.** The document is now typed into directly,
with a formatting bar for headings, bold, italic, both kinds of list and quotations.
Markdown stays the stored form, so every edit is read back into it — which matters
more than it sounds, because what a browser leaves in an editable region is not the
HTML this project wrote: `div`s where paragraphs were asked for, `b` where `strong`
went in, `span`s carrying styles, and non-breaking spaces holding the caret at the
end of a line. The reader knows only the vocabulary a protocol has and reduces
anything else to the words inside it, because losing a stray `span` is right and
losing the words in it never is.

The property that had to hold: **a document nobody edited does not change when the
editor merely opens it.** A test renders a protocol to HTML, reads it back, and
requires the same Markdown out — otherwise opening the document view would rewrite
protocols by itself. Verified in the running application in both directions: typed
text arrives in the stored Markdown, and the H2 button turns a paragraph into a `##`
heading there.

- **d. A header and footer somebody can edit**, in the same rendered surface — the
  firm's name, the project, a page number.

### 3b. The editor as a word processor

**Superseded, 19 August.** The owner wrote a full concept for the editor; it is
recorded in [EDITOR.md](EDITOR.md) along with a reading of it against what is built.
The nine items below were a guess at the same thing and are kept because most of them
appear in the concept too, but EDITOR.md is the document to work from.

### The earlier feature list

Written 19 August, after the owner used the first version. The document view can be
typed into and carries headings, bold, italic, lists and quotations. That is a start
and not a word processor, and the gap is large enough to be worth listing rather than
discovering one complaint at a time.

Ordered by how often somebody editing a real protocol would reach for it. Everything
here writes back to Markdown, because Markdown stays the stored form.

**Editing the things a protocol is made of**

1. **Tables.** The formal style ends in an actions table, so this is the most-used
   structure in the document and currently the least editable — the cells can be
   typed into and nothing else. Wanted: add and remove a row, add and remove a
   column, move a row up or down. A row is the unit somebody actually manipulates:
   one more action, one action struck.
2. **A heading control that shows what the cursor is in.** Today the buttons apply a
   heading; they never say whether you are standing in one. A word processor's style
   box reads the selection.
3. **Lists that behave.** Tab and Shift-Tab to indent and outdent, Enter on an empty
   item to leave the list. These are the reflexes anybody has, and their absence is
   what makes an editor feel broken rather than limited.
4. **Undo and redo that know about the document.** The buttons drive the browser's
   own undo, which was written for the Markdown box and does not know what a
   heading is.

**Getting text in and out**

5. **Paste that keeps structure and drops decoration.** Pasting from Word or a
   browser currently arrives as whatever HTML the source had; the reader on the way
   back keeps the words and loses everything it does not recognise, which is right
   but happens after the fact. Cleaning the paste as it lands is what makes it
   predictable.
6. **Find and replace.** Find exists for the Markdown view. Replace does not exist
   at all, and a protocol full of one misspelled name is exactly the case.

**The page**

7. **A page size that is a setting rather than an assumption.** A4 is written into
   the stylesheet. It should be chosen — A4 or Letter at least — and the choice
   should reach the screen, the PDF and the Word file together.
8. **A header and footer somebody can edit.** The firm's name, the project, a page
   number. Word carries page numbers natively; the print path takes them from the
   dialog, which is a limit worth stating rather than hiding.
9. **Page breaks visible in the editor**, so that what is about to print is not a
   surprise. This is where the earlier question about showing pages comes back, and
   it is worth answering after 7 and 8 rather than before.

**Not on this list, deliberately**

Images, footnotes, comments, tracked changes, multiple columns. Each is a real
feature of a word processor and none of them is a thing a meeting protocol has.

### 4. DOCX — done, 19 August

Independent of step 3 and does not wait for it. A `.docx` is a zip holding one XML
document, and the mapping from the same document structure is direct. It goes after
the arc because the arc gives it a settled structure to map from, not because it
matters less: it is what a client edits and returns.

Built, and with no dependency after all. The zip writer is about a hundred lines,
because a Word document may store its parts uncompressed and a protocol is tens of
kilobytes — the compression would have saved nothing worth a dependency. The document
is assembled from the same blocks and the same runs the screen and the PDF use, which
is what the inline parser was rewritten as a scanner for: two parsers that must agree
eventually do not.

Headings, bold, italic, real bulleted and numbered lists, and the actions table as a
table with a header row that repeats across pages. A4 with the same margins as the
PDF. Verified without trusting this project's own code: `file` reports Microsoft Word
2007+, `unzip -t` passes every part, all four XML parts are well-formed, and macOS
`textutil` — Apple's Word reader, not ours — reads the whole protocol back with the
umlauts, the m² and the table intact. **Word itself has not opened it**, which is the
one check left, and needs a machine with Word on it.

### 5. Let a protocol style be authored — done, 20 August

- Duplicate a shipped style; edit its name, its description, its instructions and its
  density; delete one you made. A style that shipped cannot be edited, so a protocol
  written last year can be written the same way again — it is copied first.
- A style in use cannot be deleted, and the refusal names what is using it.

**Fidelity is not shown as locked. It is not there to lock.** The seven rules that
decide whether a protocol is _true_ rather than how it _reads_ were sitting in each
style's own instruction list, beside "use numbered headings" — one a matter of style
and the other not. They now live in the code and are added to every style as a
protocol is written, so authoring a style cannot reach them: not because the
interface forbids it, but because they are not in the thing being edited. Migration
20 takes them out of every style, including edited ones, since they were never the
style's to keep.

They are still shown, under "Always, in every style", with the reason — a document
reporting a decision nobody made is not a differently-styled protocol but a wrong
one. A test asserts they are absent from all three shipped styles and present in what
reaches the model, because "unauthorable by construction" is only worth anything if
both halves hold.

**And the part that had to follow it.** The actions-table check used to ask whether
the style was called `style-formal`. That was true while one style existed and became
wrong the moment a style could be copied: a duplicate has a new id, so a copy of the
formal style would have quietly stopped requiring the table its original demands, and
nothing would have said so. Adding duplication without this would have introduced
that fault.

`required_sections` was the field meant for this and could not do it — it holds
English section names while the protocol is written in the meeting's language, so
matching "Actions" against "Aufgaben" needs something this application does not have,
and it was therefore never checked anywhere. A table needs no translating.

Styles now carry **structural expectations**: things that can be checked in whatever
language the protocol is written in. One exists, because one can be checked. A
duplicate carries its original's, and a test requires exactly that — along with the
opposite, that a copy of a style with no table does not acquire one.

Writing that test immediately found a second fault: the duplication statement never
supplied `updated_at_ms`, which is `NOT NULL`, so duplicating any style would have
failed outright the first time somebody tried it.

### 6. The recording screen — built from the written direction, 21 August

There is no reference image for it. `docs/assets/ui-reference/` holds six studies —
start in both themes, the project view, the new-meeting flow, transcript review and
the protocol editor — and none of a recording. The owner asked for it to be built
from the written direction instead, which is specific enough to work from.

What the direction actually demands, and what it got:

- **"Recording is part of the interface, not an instrument panel bolted onto it."**
  The mark is the same waveform as the start page and the application icon, drawn live
  from what is arriving — one stroke per second of the recent past rather than a
  meter. No filled bars, no pulsing red, no clipping indicator borrowed from a DAW.
- **"A recording in progress must be unmistakable at a glance, because hiding it would
  be dishonest."** This was the one that was not met. It was unmistakable only on the
  screen that started it; navigating to another project or to settings left a live
  recorder with nothing anywhere saying so. The sidebar now carries it wherever
  somebody is, in the same quiet vocabulary as every other status — a dot that
  breathes slowly, steady for anybody who has asked for less movement — and clicking
  it goes back.
- **"Whether the people in the room have agreed is the responsibility of the person
  recording them."** Said once, plainly, in the lead. Not a checkbox, not a consent
  gate: a product cannot witness an agreement it was not present for, and one that
  pretends to would be manufacturing a record of it.

The screen also stopped polling on its own. It now reads the status the application
already takes once a second, because two pollers asking the same question on different
schedules is two answers — the screen and the sidebar could disagree by up to a second,
and the recorder was asked twice as often for nothing.

**Still unproven, and only the owner can prove it:** pressing Record in the real window
and getting a meeting that transcribes. That needs the macOS system-audio permission
dialog, which appears only to somebody sitting at the machine.

### The two questions that were the owner's, answered

**Should the editor show pages?** Not yet — but at a page's width. A flowing document
set to A4's text measure gives the thing a page layout is actually wanted for: what is
edited and what prints break at the same place. Pagination inside an editable surface
is genuinely hard, and its remaining value is at export time, where the print step
already supplies it. Worth reconsidering once the header and footer exist, since those
are the parts that really want a page to sit on.

**Rendered editor before the exports, or after?** Before. A PDF is a rendered document
printed; doing the editor first makes the PDF nearly free, and doing the PDF first
means writing the renderer twice. DOCX does not depend on it either way.

### Parked, with reasons

Writing by topic. Two things are now known about it and one is not. It **does** control
length — 74,000 characters became 26,100 with per-topic budgets — and coverage held at 31
of 35 figures, the best measured. It **does not** produce a document organised by subject:
twenty-nine headings with three separate facade sections and the meeting's closing at
position fifteen, because each topic is written independently in transcript order and
structure is a global decision. What is **not** known is whether it removes the dependence
on a wide context, which is the reason it was tried: two runs at 8,192 tokens failed with
the answer cut off, and the implementation never split an oversized topic — a comment
claims it does and no code does — so a single topic's passages can fill the window on
their own. Measuring that needs instrumenting the path rather than inferring from the
failures, which is what the second cause candidate (the topic pass's own group-judging
payload) makes clear. Parked until there is an idea for structure that per-section rules
cannot supply — it controls length and coverage and produces a document ordered by when things were said. The model that proposes corrections for the two or three ragged words a meeting leaves. The visual pass over the interface, which is a different job from the interaction and was deliberately not done with it.

## The direction

The first meaningful product path remains:

```text
Project → Meeting → Imported recording → Local transcription
→ Transcript review → Local protocol generation → Markdown editing
→ Markdown/plain-text export
```

The next milestone is not another general framework. It is a protocol that a professional can accept after light editing, produced locally, with enough evidence to understand what the system did and where it may be uncertain.

## What is in place

### Product shell — Done

The Tauri shell and browser preview have the real navigation structure, warm light and dark themes, locally bundled Barlow typography, a resizable sidebar, contextual inspectors, keyboard-visible focus, reduced-motion handling, and the main empty, project, meeting, transcript, protocol, library, and settings screens.

The visual shell is still evolving, but it is no longer a disposable mockup.

### Storage and recovery — Done for the current vertical slice

SQLite stores identity, relationships, lifecycle state, revision metadata, jobs, and artifact paths/checksums. Committed transcript and protocol content lives in immutable versioned files. Working autosaves are separate. Imported originals are never silently changed.

Migrations, staged writes, checksums, interrupted jobs, cancellation, retry, and restart reconciliation are covered by the Rust tests.

### Import and media preparation — Done for the current vertical slice

Supported audio/video files can be copied into managed storage, probed, normalised to working audio, checksummed, cancelled, and recovered after interruption. The original file remains untouched.

### Transcription — Partial

The application has a supervised whisper.cpp boundary, structured JSON parsing, timestamps, uncertainty markers, vocabulary prompts, provenance, model presets, and consent-gated verified model downloads.

Real local runs and a long German evaluation exist. A release-only Tauri configuration and reproducible
sidecar builds now define the distribution path for both whisper.cpp and sherpa-onnx, each pinned to
the revision its behaviour was validated against, and the resolver prefers the shipped runtime over
anything on the machine. The sidecars have not yet been built and run on a clean machine, and signed
artifacts and the M1/8 GB baseline still need validation.

### Speaker separation — Partial and provisional

The application contains diariser output parsing, time-overlap alignment, editable speaker labels, managed
diarisation models, a bundled-runtime discovery boundary, and a first-use preparation action in the
meeting flow. The quality evidence is limited to a short synthetic study and one development-machine
evaluation.

Separation runs only when somebody offers a number of people. Clustering by similarity alone was
measured at eighty-six speakers on a meeting of about eleven, because a voice drifts across eighty
minutes of videoconference, and the models stay installed after the first use — so the pass would
otherwise keep running unasked and keep producing that.

Speaker labels must remain provisional. They are not confirmed identities.

#### The pass now listens to samples rather than the whole recording — measured, kept

It ran for about half an hour on an eighty-one minute meeting, which is longer than transcription
and generation together.

Skipping silence does not fix that: only ten per cent of the reference recording is silent, so
working on speech alone saves about a ninth.

Sampling does. Separation runs after transcription, so the segments are already known, and placing a
voice needs a couple of seconds of it rather than a whole utterance. Two seconds from the middle of
each of the reference meeting's 675 segments, joined by silence, is 25.6 minutes of audio in place of
81.8.

Measured on the reference meeting, both runs asking for eleven speakers:

|                              | Whole recording |      Sampled |
| ---------------------------- | --------------: | -----------: |
| Time                         |          1810 s |    **498 s** |
| Turns                        |             291 |          128 |
| Speakers landing on segments |               8 |           10 |
| Largest speaker's share      |            58 % |         56 % |
| Longest unbroken run         |    126 segments | 126 segments |

The two agree on 88.6 % of segments. Neither is ground truth — see below — so agreement is the
question, and the shapes match: the same dominant speaker, the same 126-segment run. Sampling
resolves a slightly longer tail rather than a shorter one. It is three and a half times faster and
is now the path, with the whole recording kept as the fallback when the condensation cannot be built.

The condensation is byte arithmetic over the working audio, not an ffmpeg call. Both ffmpeg routes
were tried and rejected: a filter graph of one `atrim` per sample had not finished after ten
minutes, and the concat demuxer rounds each out point up to a packet boundary, which drifted 36
seconds across the meeting and would have read the last turns back against the wrong audio.

#### What the number of speakers means — open

The count is treated as a fact and is structurally a guess. Thirty people are invited and fifteen
speak; somebody unexpected joins; two people share one microphone. The owner of this project
attended the reference meeting and cannot say whether ten spoke or eleven.

Asking for a count is also the wrong shape of question for the case that needs it. Four people around
a table and the user knows; twenty on a site call with subcontractors dialling in and they do not,
and that is where separation would earn its place.

What the count is worth was measured by sweeping it on the sampled audio, comparing each run against
the eleven-speaker one:

| Asked for | Labels used | Segments per label, largest first       | Agreement |
| --------: | ----------: | --------------------------------------- | --------: |
|         6 |           6 | `385 121 100 43 19 7`                   |      96 % |
|        10 |           9 | `385 121 100 25 16 11 8 7 2`            |      99 % |
|        11 |          10 | `381 121 100 25 16 11 8 7 4 2`          |         — |
|        14 |          10 | `381 121 100 25 16 11 8 7 4 2`          |     100 % |
|        20 |          14 | `378 108 69 31 25 16 13 11 7 4 4 4 3 2` |      92 % |

**The answer barely moves between 6 and 14.** The count a user agonises over mostly does not matter,
and where it does — asking twenty when about ten spoke — the damage is invisible: the top speaker
holds, while the second and third are carved up, 121 to 108 and 100 to 69.

Three ways to avoid asking were measured and none works:

- **The diariser's own automatic mode.** Clustering by distance threshold gives 67 labels on the
  condensed audio and 86 on the whole recording.
- **A quick precheck.** A sparse condensation of every fourth segment — 6.4 minutes, about two
  minutes a run — was swept from 4 to 18. The number of labels simply tracks what is asked for, with
  no plateau anywhere. It measures the sample, not the meeting.
- **Plateau detection on the full condensation.** Eleven and fourteen return identical output, which
  looked like a signal, but twenty returns fifteen. A real estimator would keep answering ten for
  anything above ten. Two agreeing points inside a narrow window are a coincidence, not a method.

#### The pipeline was the wrong shape — replaced, and the count is now an answer

Every problem above traces to one thing: the pass runs pyannote **segmentation** to find where
speakers change, over audio whose boundaries transcription already established. The condensation, the
silence between samples, the 300 ms that turned out to be shorter than the diariser's own
`min_duration_off`, the merged runs of 126 segments, and the eight minutes each count costs are all
consequences of rediscovering what was already known.

sherpa-onnx's C API exposes a speaker embedding extractor that takes the **embedding model alone**:
give it audio, it returns a vector. So the pass should be — two seconds of each transcript segment,
one embedding per segment, cluster the vectors. That removes the segmentation model, removes the
condensation and everything built around it, and makes clustering free, because grouping a few
hundred vectors takes milliseconds rather than minutes.

Free clustering dissolves the count question rather than answering it. Every k can be tried at once
and the affinity matrix's eigengap read directly, which is the actual method for estimating how many
speakers there are; counting non-empty clusters, which is all the CLI permits, is a crude proxy for
it. The user can be shown what was found instead of asked for what they do not know.

What survives from the sampling work is its central finding — two seconds of a segment is enough to
place a voice — which is exactly what makes per-segment embedding cheap. What becomes unnecessary is
the plumbing around it.

That is now built. `localog-speaker-embedding` is a supervised sidecar like the others, built from
the same pinned sherpa-onnx revision as the diariser and linked statically so it carries no
dependency on the machine that built it. It writes one vector per segment as a small versioned
binary file rather than through a pipe, because a meeting's worth is megabytes. The grouping happens
in the application and is checked against the study: on the reference meeting it reproduces it
exactly, `388 120 102 17 17 11 10 7 1 1 1` at eleven voices.

**The interface now offers three answers rather than two.** Leave the speakers together, separate
them into a stated number, or separate them and let LocaLog work out how many. The third was not
offerable before: the diariser answered by re-reading the audio, so a count had to be settled in
advance by somebody who often does not know it. Leaving them together remains a choice somebody
made rather than an absence of one, and the pass does not run because the models happen to be
installed.

The estimate reads the count off where the merging stops joining a person to themselves. The floor
is fitted to one fixture with ground truth and one meeting without, so it is offered as an estimate
that can be replaced with a number, never asserted.

The diariser remains as the fallback where the embedding sidecar is not installed. There is one
genuine loss against it — pyannote can in principle catch a speaker change inside a single transcript
segment, and one embedding per segment cannot. Segments average 7.3 seconds here and the sample is
taken from the middle, and the 126-segment merged runs suggest the older pipeline was not catching
them in practice either.

Not yet built and run on a clean machine, which is the same gap the other two sidecars have.

#### Still unanswered

The embedding model is trained on Chinese, which is a poor match for the first audience. Whether a
different one does better is unresearched.

Underneath all of it: speaker separation exists to improve attribution in the protocol, and that has
never been measured. A protocol generated with speakers and one without, from the same meeting, would
settle whether the pass earns its place at all.

Neither run above can be scored for accuracy, because the reference meeting has no known speaker
count. If a wrong count degrades the result badly and nobody can supply a right one, the honest
conclusion is that automatic separation is not ready to be trusted for attribution, and the useful
feature is one that helps a person label speakers rather than one that claims to know.

### Protocol generation — Partial; the main quality work

The Ollama provider is narrow, loopback-only, cancellable, bounded, provenance-aware, and restricted to already available models. Generation is sectioned for long transcripts and has style and vocabulary inputs.

Generation records what it found about its own result: how many stated quantities the protocol keeps,
which figures it states that the meeting did not, and its length against the transcript. Those numbers
now reach the reader, beside the draft they describe.

They are presented as evidence to look at, never as a verdict. Only one of them is wrong under every
style — a figure the draft states that the meeting did not — and that is the only one shown as a
warning. How much a draft keeps is what its style asked for, and a machine judgement placed in front
of a person asks them to read less carefully, which is the one check in this product that reliably
works.

Dividing a meeting into subjects is compiled for evaluation only, which is its honest status while the
first question below is open. Writing subject by subject was measured and rejected — it produced a
document longer than the transcript — and running the pass merely to index a finished protocol would
add about seven minutes to a twelve-minute run for a diagnostic. The evidence stays runnable through
the evaluation harness; the shipped library carries nothing it does not call.

The generated protocol is not yet proven complete or reliable enough for professional use.

### Editing and export — Done for the current vertical slice

The protocol editor supports Markdown editing, autosave, undo/redo, find, text scaling, review state, immutable revisions, restoration, and explicit Markdown/plain-text export.

A transcript line can now be removed as well as rewritten — for the throat clearing, the crosstalk
and the thirty seconds of somebody's dog. Without a confirmation, deliberately: rewriting a line is
exactly as permanent as removing one and asks nobody's permission, and the committed revision is the
way back from both. The last line cannot be removed, in the application and in the interface.

A recording can be trimmed before it is transcribed. Drag across the timeline to select a stretch,
then start there, end there, or remove it; every cut is listed and undoable one at a time or all at
once. The cuts are held as a description of what to keep and applied when the working audio is
built, so the recording is never modified and none of it is final until then. What is removed is
veiled on the timeline rather than taken off it, because seeing what you cut is what makes putting
it back feel possible.

Editing before transcription is what keeps that simple: no transcript exists yet whose timestamps
would have to be reconciled with a timeline that just got shorter.

The recording timeline is usable without a pointer, which it was not when first built: it takes
focus, the arrow keys move a visible caret along it, holding shift takes the selection with them the
way selection works in a text field, and `Home`, `End` and `Escape` do what they say. Alt gives a
finer step and shift a coarser one, so crossing an eighty-minute meeting is a few keys rather than
a hundred. It announces itself as a slider whose value is the selection, so what a screen reader
says and what the screen shows are the same thing.

The editor still needs long-document, accessibility, and real-background-load validation.

### Libraries and settings — Partial

Vocabulary is editable and resolved into job provenance. The shipped professional styles are structured and versioned, but the style library is not yet fully editable. Language concepts remain separate by design.

The meeting-language flow is now wired through project defaults, per-meeting overrides, transcription
runtime language codes, and protocol-generation inputs. The normal UI offers common languages while
still allowing a language name outside the convenience list. Interface-language selection remains a
separate future setting.

Meeting and transcript review now expose the language as a correction point. A user can change it and
explicitly rerun transcription; the current result remains in place until the new job commits, while
the new run receives its own immutable revision and provenance. Automatic language detection is not
used as a silent replacement for the user's choice. It should first be tested as an advisory preflight,
because a guess is not a safe reason to change a professional record.

Speaker differentiation is now visible as an optional, progressive-disclosure setting. The existing
diariser boundary reports model/runtime readiness, accepts an expected speaker count for each
transcription run, and keeps labels
editable in transcript review. It remains provisional until a distributable runtime and broader
multilingual quality evidence exist.

## What is not yet true

Checked against the code on 25 August 2026. Three entries that stood here were no
longer true and have been replaced by what is; the rest were confirmed by reading
the thing they describe rather than by remembering it.

- ~~Project and meeting archive actions are not exposed in the interface.~~ Done on
  26 August 2026, and worth recording why it was mis-sized here: both tables had
  carried `archived_at_ms` since the beginning and every list already filtered on it,
  but nothing could ever write one, so the filter was doing nothing. What was missing
  was the write side, somewhere to see what had been put away, and the way back — not
  only the buttons.
- ~~Basic backup and restore are not implemented.~~ Done on 26 August 2026. A backup
  is a folder holding the database and the managed audio, with a checksum for every
  file. The database is copied by SQLite's `VACUUM INTO` rather than by the
  filesystem, because the newest writes live in the write-ahead log and copying the
  one file yields a backup silently missing them. Models are excluded and the
  manifest says so. Restoring verifies everything before it moves anything, and moves
  the current workspace aside rather than deleting it.
- The final public protocol-generation runtime is undecided; Ollama is for development
  and early technical previews.
- Real English end-to-end quality evidence is still missing.
- **The interface is English only.** Meetings can be transcribed in twelve
  languages and the protocol is written in the meeting's language, but every word
  of the application around them is English — in a product written for German
  offices. A settings row said "Interface language: English" with no way to change
  it, which was worse than saying nothing, and was removed on 27 August 2026 rather
  than wired to a menu: the work is translating every string, not adding a control.
  It should be sized properly before it is started. This is criterion 8 in
  `MVP.md` and the only one of the ten that is open and actionable.
- The application is not signed. It is ad-hoc signed, which runs here and is refused
  on anybody else's Mac, and there is no Developer ID on this machine. This is the
  whole of what stands between the current bundle and handing it to somebody. The
  owner intends to enrol; it is deliberately last, because it is the one remaining
  task that buys nothing for Windows or Linux.
- Whether the two recorded tracks drift apart over a long meeting is unanswered. A
  short run cannot separate drift from a fixed start offset, so it needs a long one.
- Windows and Linux have no packaged release. What the code costs them is now
  measured rather than assumed, and it is small — see below.
- The M1/8 GB baseline has not been measured and will not be until such a machine is
  available. It is not a task while there is no hardware to run it on; MVP.md
  criterion 9 stands unmet and openly so.

### What the other platforms actually cost, measured 25 August 2026

Three files in the Rust side carry a platform branch, and every one degrades
honestly rather than failing:

| what                                      | macOS                                             | elsewhere                                                                                 |
| ----------------------------------------- | ------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| memory detection (`machine.rs`)           | `sysctl hw.memsize`                               | Linux reads `/proc/meminfo`; Windows returns nothing, and a conservative constant is used |
| diarisation accelerator (`media.rs`)      | Core ML                                           | CPU: slower, correct                                                                      |
| free disk before a download (`models.rs`) | `statvfs`                                         | `#[cfg(not(unix))]` returns nothing, so Windows alone loses the check                     |
| the print dialog (`lib.rs`)               | Tauri's `window.print()`, which is cross-platform | the same call                                                                             |

Four of the five sidecar scripts are portable, and the whisper one selects Metal on
the target triple, which is correct rather than a special case. Sidecar discovery
assumes no platform. Model download is Rust and checksums. Cutting the print sheet
into page boxes ourselves _removed_ a WebKit dependency rather than adding one.

**One thing is genuinely macOS-only: the recorder.** Core Audio process taps and
AVFoundation in one Swift file, and `build-recorder-sidecar.sh` refuses to run off
Darwin. Linux needs a PipeWire recorder and Windows a WASAPI-loopback one. The
contract they have to meet is already the right shape and already written down —
`--system <path.wav> --microphone <path.wav>`, two files out — and nothing above
the recorder knows how the bytes were captured.

An iPad is a different question from a port. This architecture spawns sidecars, and
iOS does not permit a sandboxed application to execute another binary. Whisper and
the diariser would have to be linked in and called as libraries. That is a second
build of the engine, not a port of this one, and it should not sit on a list beside
Windows and Linux as though it were the same size of job.

## Next milestones

### 1. Make protocol generation trustworthy

This ends when a representative German meeting produces a protocol that a professional accepts after light editing.

Work in this order:

1. Decide whether the current sectioned approach is enough or whether the structured evidence record should become the production intermediate.
2. Keep the useful transcript-to-segment links and unclaimed-segment reporting.
3. Quantities kept, figures invented, length against the recording, and tasks recorded with nobody
   against them are measured and shown. Unsupported statements still need a check that can be
   relied on, and the obvious ones need a model to judge its own work, which this project has
   already found unreliable.

   Missing sections cannot be checked as things stand. A style's `required_sections` are stored as
   English literals while the protocol is written in the meeting's language, and they are
   deliberately kept out of the prompt — sending them once took the reference protocol from 17,393
   characters to 2,747. Matching "Actions" against "Aufgaben" needs something the application does
   not have.

   The action table is now checked structurally instead, which sidesteps the language entirely: a
   markdown delimiter row looks the same in German. Measured across nine runs, about one draw in
   five omitted the table having been told twice to produce one, and the rejection is fed back
   through the existing correcting retry.

### 1b. What a style is, before anybody can author one

Decision 7 says a person authors their own styles. Before that is built, the object needs taking
apart, because what it currently holds is four different things under one name.

Sorting the shipped style's fourteen instructions:

| Kind          | Count | Example                                                   |
| ------------- | ----: | --------------------------------------------------------- |
| **Structure** |     6 | "End with a table of agreed next steps"                   |
| **Fidelity**  |     5 | "Never invent a decision, an action, an owner, or a date" |
| **Voice**     |     1 | "Write discussion as calm, factual prose"                 |
| Density       |     1 | "Write at whatever length the material requires"          |
| Language      |     1 | "Write the entire protocol in the meeting's language"     |

**One instruction of fourteen concerns how language is used**, which is the thing the owner means
by style — direct or lofty, plain or formal. The rest is document shape and rules about telling the
truth, both wearing the name "style".

That suggests four axes rather than one, and they do not all belong to the author:

- **Fidelity is invariant and not authorable.** "Never invent a decision", "reproduce every figure
  exactly", "never leave a placeholder", "cover every topic". A person authoring a house style is
  choosing how their firm writes, not whether the record may be wrong. These belong to the
  application and should be applied to every style, including authored ones.
- **Structure is authorable and checkable.** Numbered sections, participants first, ends with a
  table of tasks and owners. Declared as structure rather than as prose, each one becomes a check
  that plugs into the correcting retry — which is exactly how the action table now works, and it
  needed no translation to do it.
- **Density is already a separate setting** and should stay one. `ProtocolDensity` exists, is
  applied by `with_density`, and is not stylistic: it is how much of the meeting a reader wants.
- **Voice is the remaining thing, and is the smallest part of the object today.** It is also the
  part that most needs prose rather than structure, because "calm and factual" cannot be expressed
  as a field.

**All four were carried out, 20–21 August, and the analysis held.**

- **Fidelity** left every style and became seven rules in the code, added to each protocol as it is
  written. Authoring cannot reach them because they are not in the object being edited.
- **Structure** became structural expectations — one, because one can be checked. The actions-table
  check had been asking whether the style was called `style-formal`, which duplication would have
  broken silently.
- **Density** stayed a separate setting and gained the instruction that had been sitting in the
  formal style arguing with it.
- **Voice** is what remains editable, alongside structure, which is what a person authoring a house
  style is actually choosing.

The one correction the analysis needs: it counts five fidelity instructions and there are seven,
because "separate what was decided from what remains open" and "mark uncertainty in the words the
meeting used" are also about whether the record is true rather than how it reads.

The immediate consequence is that `required_sections` should be replaced rather than repaired.
Structural expectations in a checkable form are what it was reaching for; English section names
were the wrong shape for a document written in German.

#### A contradiction that becomes live the moment density is chosen

Density should be a choice the person makes, and three options are enough — `ProtocolDensity`
already has exactly three, and their directives read well. But the shipped style also carries its
own length instruction:

> Write at whatever length the material requires. Do not compress the meeting into a summary: this
> is a record, and a reader who was absent must be able to follow what was discussed and what
> follows from it.

while `with_density` independently appends, for Terse:

> Write as briefly as the meaning allows, roughly a line per point.

Both reach the model in the same prompt. It is dormant today because one style exists, its density
is fixed at comprehensive, and nothing lets anybody choose otherwise. It bites on the first day
density becomes selectable, at two of its three settings.

The decomposition says what to do. That instruction is three things fused: a length rule, which is
density's; a "this is a record, not a summary" rule, which is fidelity's and is already carried by
"Cover every topic that was discussed"; and a justification for both. Split along those lines it can
be deleted without losing anything.

**Measured, 17 August 2026, and the answer is not to delete it.** Three seeds with the
instruction and three without, speaker labels already removed in both arms so they
differ in one thing:

| Seed       |    With |   Without |
| ---------- | ------: | --------: |
| 7          |  12,384 |    10,219 |
| 101        |  12,276 |    10,141 |
| 202        |  12,465 |    11,192 |
| **Spread** | **189** | **1,051** |

It adds between 1,273 and 2,165 characters — the direction reliable at every seed, the
magnitude not. The more useful number is the spread: **with the instruction three
drafts land within 189 characters of one another, and without it they scatter across
1,051.** It does not merely lengthen the protocol, it anchors it, and five times less
variation between draws is the most stable behaviour measured anywhere in this
project.

So the instruction earns its place at the comprehensive setting. What remains wrong
is that it lives in the style, where it will contradict the two density settings
nobody can select yet. The change is to **move its content into the comprehensive
density directive** rather than to remove it: then "do not compress this into a
summary" travels with the setting that means it, and terse stops carrying an
instruction that fights itself.

Not done here, because it needs a style revision and a migration, and it belongs with
the density-selection work rather than ahead of it.

4. Retry one failed section or pass rather than discarding the complete run.
5. Compare the result with the existing human reference on completeness, correctness, attribution, length, and editing effort—not length alone.
6. Repeat the same workflow with an English meeting or synthetic equivalent.

### 1z. Ask for the vocabulary, because it is the largest measured win in the project

Do this before any further work on protocol models. It is the cheapest improvement
found so far and it improves every model at once.

The reference meeting was transcribed with an empty vocabulary — the workspace holds
zero entries — and every model comparison in this project rests on that transcript.
Giving whisper thirty proper nouns as its initial prompt, on the _same_ model, with
nothing else changed:

| Term                                  | Without vocabulary      | With vocabulary   |
| ------------------------------------- | ----------------------- | ----------------- |
| The housing form the meeting is about | 0 right, 40 wrong       | 32 right, 4 wrong |
| The building system supplier          | 0 right, 13 wrong       | 6 right, 2 wrong  |
| The client's name                     | 0 right, 1 wrong        | 4 right, 0 wrong  |
| The word for the building envelope    | 19 right, 11 wrong      | 35 right, 6 wrong |
| The word for structural engineering   | **never occurs at all** | occurs            |

The last row is the one to act on. Without the vocabulary that word does not exist
anywhere in seventy-two thousand characters, and the generated protocol therefore
files a named structural engineer under a discipline he does not practise. No
protocol model can recover from that, and every hour spent comparing protocol models
on that transcript was measuring the transcriber.

The mechanism is built and correct: `vocabulary_prompt` prioritises proper nouns,
respects whisper's limit, and passes `--carry-initial-prompt` so the terms bias the
whole meeting rather than its first thirty seconds. **What is missing is the asking.**
`NewProjectView` mentions in one line that vocabulary "can be configured in the
project"; `NewMeetingView` says per-meeting vocabulary "is not available yet". A
person who creates a project, records a meeting and presses go gets the bad
transcript, because nothing ever asked them for the twelve words that would fix it.

So the work is a question of product, not of models: at project creation, ask for
the names — the client, the firms, the people, the project. It is the one input where
a minute of somebody's typing is worth more than any model choice this project has
measured. The exact shape of that asking is a design decision and is the owner's.

The screen is now called **Names & terms**. Vocabulary oversold it and glossary would
be wrong — a glossary carries definitions and this carries none. It is a spelling
list, and measured, almost entirely a list of proper nouns.

#### Where the application asks, and what it does with the answer

Designed 16 August 2026 with the owner. Not built.

**1. Offer candidates instead of a number nobody can act on.** whisper already records
which words it was unsure of, and the transcript view already has a panel for them.
For the reference meeting that panel says **322 to check** out of 675 segments, which
is not a task anybody starts. But the words it flags do contain the mis-heard names —
`Trakwerk`, `Klasterwohnung`, `Nukera`, `Vermessung`.

Keeping only words the transcriber was unsure of _every_ time it heard them, heard at
least twice, gives **six candidates** for an eighty-minute meeting, of which two or
three are the names that matter. That is a thirty-second job with a large payoff, in a
panel that currently asks for an impossible one. Loosening the filter to catch more
names costs precision quickly: a top-fifteen list picks up two more surnames and about
as many ordinary German compounds.

Open question for the owner: whether the flag count stays visible alongside, or the
candidates replace it. The flags are also how somebody finds passages to re-read
before trusting the protocol, which is an argument for keeping them.

**1b. The protocol model already flags names the transcriber was sure about.** Found
by accident, 16 August 2026, reading the foot of a draft:

> [Note: The term "Klinker-Nord" is used in the source text; it is unclear if this
> refers to a specific project name or location.]

That is the client's name, mis-heard. Unprompted, the model noticed the word behaves
like a name it does not recognise and told the reader instead of using it silently.

The important part is which error it caught. whisper flagged the catastrophically
mangled form of that name — `Lärgedorf-Bildes-Fropette-Reit` — and did **not** flag
the plain wrong spelling, because it was confident about it. **A transcriber's
confidence cannot flag an error it is confident about.** That is a structural blind
spot, and it is the dangerous class: a confidently wrong name is the one that reaches
a client's inbox looking correct.

So there are three sources of candidate terms, and they overlap less than expected:

1. Words the transcriber was unsure of every time — catches garbled forms.
2. The same, grouped by stem — catches names whose mis-hearing varied.
3. **The protocol model's own notes — catches names the transcriber was sure about.**

The third is weak on its own: four of thirteen drafts carried such a note, each
flagging one name. It is also free, since the model is already writing them, and it is
the only one of the three that can see this class at all. Harvesting them into the
candidate list is a small piece of work with no new model call.

**2. One correction, two jobs.** Correcting `Klaster → Cluster` should fix the current
transcript _and_ enter Names & terms so the next meeting is transcribed correctly.
Cure and prevention from the same keystroke, which is what makes the thirty seconds
worth spending.

**3. Fix the current transcript deterministically, before the protocol harness sees
it.** Measured on the reference meeting, plain substring replacement of eleven
corrected stems fixed **80 occurrences** in milliseconds — reaching roughly what a
seven-minute re-transcription with the larger model achieved:

| Term     | Before | After replacement | (`medium` + vocabulary) |
| -------- | -----: | ----------------: | ----------------------: |
| Cluster  |      0 |            **40** |                      40 |
| HOAI   |      0 |            **16** |                      35 |
| Tragwerk |      0 |             **5** |                       6 |
| Fassade  |     19 |            **30** |                      50 |

Compounds are the easy case, not the hard one, because German builds them by
concatenation: fixing the stem repaired `Clusterwohnung`, `Raumcluster` and
`Einraumcluster` without anyone listing them. Of 74 occurrences, 59 came out clean and
about **three** were genuinely still wrong — `Clusterwund`, `Clusterwohnenheit`.

No model, instant, and auditable: the change can be shown and undone, which a model
pass over the whole transcript cannot offer.

**4. The replacement must be reviewable, because some wrong spellings are real words.**
`Halle` should be `Halde`, a participant's surname — and _Halle_ is the German word
for cross. In this meeting all three occurrences are the person, so replacing blind
would have been safe; that will not always hold. Show the matches in context and let
them be deselected.

**5. Only then, a small model, on what is left.** Three ragged words per meeting is
where substitution cannot help, because the mis-hearing itself varied and there is no
consistent stem to catch. This is the one stage in the pipeline where a long context
is provably unnecessary: deciding whether `Halle` is a person or a crucifix needs one
sentence, not eighty minutes. So it is a small model, a few hundred characters of
window, and only for passages the deterministic pass could not settle — seconds of
work, not minutes.

Two constraints make it safe:

- **It proposes; it never applies.** The transcript changes only where somebody
  approved the change.
- **It is given Names & terms, and may only propose corrections built from them.** It
  may offer `Clusterwohnenheit → Clusterwohneinheit` because `Cluster` is listed. It
  cannot invent a name nobody entered. That bounds the single risk of letting a model
  near the evidence record.

**6. Say what is being done, at every step.** `docs/UX.md` already requires this —
the status answers "what is happening?" in a reader's words, with a moving detail on
long steps — and the five steps above were written without it, which is how a stage
that quietly rewrites the evidence record gets built.

This stage needs it more than most, because it changes a document the person is
holding. Concretely:

- The substitution is instant, so it needs a **result**, not a progress bar:
  "12 corrections applied in 80 places" with the places listed and undoable. A silent
  transformation of the transcript would contradict the standing rule that imported
  originals and existing exports are never silently changed.
- The model pass gets a stage in a reader's words — "Checking 3 passages that could
  not be settled" — with the count as the moving detail, not a spinner.
- Waiting for approval is already one of the states the sidebar distinguishes, and
  this stage is the clearest case of it in the application.
- Nothing here is a heavy-lane task, so it must not block or be blocked by one.

Build order is deliberate: the deterministic pass first. If a few meetings show the
leftover is consistently three-ish words, a person fixes them faster than the
suggestion could be built.

### 1a. Strengthen the harness so a bad draw is not a lost run

Generation is already sectioned: a long meeting is condensed section by section and
then synthesised. When any one step returns something unusable the whole run is
discarded, which is a quarter of an hour of somebody's machine for a fault that
affected one section.

Measured on the reference meeting, `ministral-3:8b` returned a usable protocol at one
seed of three and `gemma4:12b` at three of three — but Gemma missed the required
table at one of them. So this is not about rescuing weak models: every model has bad
draws, and the harness currently converts each one into a total loss.

Done, and worth having before the rest:

- Code fences stripped deterministically, including ` ```json `.
- An answer refused when it parses as a JSON object, or is shorter than a hundredth
  of what was said. Both of `ministral-3:8b`'s failures are caught by this, and the
  JSON one had scored 28 of 35 figures.
- Every parse of a model's answer repaired when it is nearly-JSON, rather than the
  run being lost to a newline the model wrote inside a string.

To build, in order:

1. **Retry the step that failed, not the run.** A section that comes back empty,
   fenced-as-JSON or implausibly short is one request, and the sections around it
   were fine. Retry it a small fixed number of times before failing the job.
2. **Tell the model what was wrong.** "You returned JSON; return markdown" is a
   strong correction and costs one request. This is the whole of the agentic part —
   it needs no planner and no extra pass, only the rejection fed back.
3. ~~**Keep what survived.**~~ Built. A section that fails every retry becomes a
   marked hole: the notes carry an instruction to say at that point that the content
   is unknown and not to guess at it, and the finished protocol carries a closing
   section naming each missing stretch by its position in the recording, so somebody
   can scrub to it and listen. The second of those does not depend on the model doing
   as it was told. Only a bad answer, a truncated one or a model gone quiet is
   survivable this way; a missing model or a changed runtime fails every remaining
   section identically and still fails the run, as does every section failing.
4. **Ask for less at a time**, which is the same experiment as the context question
   below and worth running once for both. Partly measured — see below.

### The context window is a parameter, not a requirement

Worth recording because it is easy to assume otherwise. `plan_sections` divides a
transcript to fit `context_tokens`, and `synthesis_budget` folds the notes until they
fit the same window. The harness already works the way a person would: read the
meeting in pieces, then bring the pieces together on far less material than the
whole. 40,960 tokens was chosen because it was measured as affordable, not because
the design needs it. Narrowing it makes more, smaller sections and more folding, and
until 16 August it was not known what else it changed.

The two reasons it was worth measuring, and how they came out:

- **Memory.** The hope was 8,192 tokens — about 1 GB of key-value cache rather than 5,
  which would bring `gemma4:12b` near 7 GB total instead of 12 and inside the 8 GB
  target. **That width does not work**: it leaves about 15,200 characters for a
  protocol that runs 11,000 to 18,000, and three seeds failed identically. The route
  to a small machine is 16,384, which works at every seed and costs about 2.4 times
  the wall-clock.
- **Weaker models.** `ministral-3:8b` wrote 12 KB of usable protocol at one seed and
  collapsed at two others from the same prompt. Whether a smaller ask is a steadier
  one for it has not been measured; the correcting retry rescued one of its two
  failures without any change of width.

The cost is more requests and so more wall-clock, which is what the measurements
below show and the only thing they show: quality does not move.

#### Measured, 16 August 2026

`gemma4:12b` on the reference meeting, three seeds at each context. Twelve runs.

| Context | Sections | Seconds (mean) | Figures, by seed | Tables |
| ------: | -------: | -------------: | ---------------- | -----: |
|  16,384 |        5 |      **2,079** | 24, 31, 26       | 3 of 3 |
|  24,576 |        3 |          1,457 | 29, 25, 27       | 3 of 3 |
|  32,768 |        2 |            877 | 29, 27, 23       | 2 of 3 |
|  40,960 |        2 |           ~840 | ~29, –, –        | 2 of 3 |

**Quality does not depend on the context. Time does.** Figures range 23 to 31 across
seeds at a fixed window, which is wider than any difference between windows; the best
single result of the whole sweep — 31 of 35 — came from 16,384, the context that had
been written off as failing. Ten of twelve runs carried the action table, spread
across every context. Headings swing from 11 to 48 at 16,384 alone.

What moves systematically is wall-clock: **16,384 costs about 2.4 times as long as
40,960**, because five sections means five times the requests. That was predicted
above before it was measured, and it is the only prediction here that survived.

So the trade is memory against time, at flat quality. An eight-gigabyte machine can
run this at a small window and pay for it in minutes rather than in the protocol — a
better bargain than it looked, and the answer that target needed.

#### What not to spend effort on, and why

Decided 16 August 2026, before any of it was built.

The context problem is the part of this work most likely to solve itself. It is not a
limit on what a model can understand; it is key-value cache memory on a small machine.
Both sides move the right way: capability per parameter keeps improving, and the 8 GB
floor is a legacy-device concern that shrinks every year the project runs — decision 28
names an M1 from 2020 as the weakest representative machine.

So the test for any design proposed to work around a small window: **does it survive a
model change?**

| Work                                                            | Survives a better model                                                     |
| --------------------------------------------------------------- | --------------------------------------------------------------------------- |
| The names glossary and the transcript corrections               | Yes — about whisper and proper nouns, nothing to do with the language model |
| Harness checks: the action table, tidying, retries, marked gaps | Yes — a better model still draws badly sometimes                            |
| Knowing the window is free to choose                            | It is knowledge, not architecture, and costs nothing to keep                |
| An index-and-write-in-parts design built _for_ small windows    | **No** — a smaller, better model makes it wasted work                       |

That last row is the decision. **Do not rebuild generation around an index in order to
fit a small context.** If it is built, it must be justified by protocol quality or by
harness reliability — reasons that outlive whatever model is current — and measured
against the current path rather than assumed better.

The small-device target does not need it either way. 16,384 tokens already produces the
same protocol for about 2.4 times the wall-clock, so a modest machine is served by
choosing a smaller window rather than by rearchitecting.

**What that is worth to the product, rather than to the benchmark.** A local
application usually degrades on a weaker machine by giving it a smaller model, and a
smaller model writes a worse protocol — measured here at 20 figures against 31. This
result says the window is not that kind of dial. The same model at a smaller window
produces the same protocol and takes longer, so a modest machine can be slower without
being worse, and there is no quality tier to explain to anybody.

Two limits keep that from being a claim yet. It holds for the **window**, not for the
model: a machine too small for `gemma4:12b` at any window still has to run something
else, and that does cost quality. And nothing here has run on eight gigabytes — every
figure in this section is from a 16 GB M1 Pro. The M1/8 GB baseline still has to be
measured on the hardware it describes.

Two earlier readings of this sweep were wrong, both from single draws at each point,
and both are recorded here rather than quietly fixed. The first said a smaller context
produced the action table; the second said figures were a flat 29 everywhere. Also
withdrawn: 16,384 does not fail — it returned `IncompleteResponse` once and then
produced protocols at all three seeds, including the best of them.

#### 8,192 is a floor, and the reason is arithmetic

Measured at three seeds after the fold was given the retry it lacked. All three failed
with `IncompleteResponse`, identically — which is what a structural limit looks like
and exactly not what a bad draw looks like, since everything else measured here varied
between seeds.

Reported by the harness's own sizing rather than inferred from the failures:

| Context | Sections | Notes fold to | Room left for the protocol   |
| ------: | -------: | ------------: | ---------------------------- |
|   8,192 |       18 |      6,254 ch | 5,069 tok — about 15,200 ch  |
|  16,384 |        8 |     14,856 ch | the full 8,192 tok requested |
|  24,576 |        4 |     32,059 ch | the full 8,192 tok requested |
|  32,768 |        3 |     49,262 ch | the full 8,192 tok requested |
|  40,960 |        2 |     66,465 ch | the full 8,192 tok requested |

Drafts of this meeting run 11,000 to 18,000 characters, so at 8,192 a long one is cut
off. **It is the only window that binds the answer**; every larger one is limited by
the requested output ceiling with room over. Retrying cannot help, which is why giving
the fold a retry did not: arithmetic does not vary between attempts.

The earlier attempt at this point was invalid rather than failing — the harness
promised the answer the whole window, the reading window came out at zero and
`plan_sections` fell to one section per segment. That was a defect in the code as much
as in the test and is fixed.

**So the floor is 16,384, and it costs time rather than quality.**

**The variation that matters is between draws, not between settings.** Both runs
missing the table are at seed 7 — the harness default, and therefore the draw behind
every early single-seed comparison in this project and the draft that was read against
the written protocol. "This draft has no actions table" became "this context produces
no actions table" on that basis.

This is the third time the same mistake has been recorded here: `granite4.1:8b`
scoring 22, 19 and 6 on three seeds is the same finding wearing different clothes.
Knowing about the trap is evidently not the same as being out of it, so the rule now
is that no comparison at a single seed is written down as a property of anything.

What remains genuinely open is why one draw in eight omits the table entirely, since
that is the failure a reader would notice first.

One distinction to keep. The first phase currently **condenses** — it writes detailed
notes keeping every point — rather than **indexing**, which is what a person does
when they scan a transcript marking where each subject lives. An index would be
cheaper again. `topics.rs` already does it and is compiled for evaluation only:
what was measured and rejected there was _writing_ subject by subject, which produced
a document longer than the transcript. Finding the subjects was never the failure.

What this cannot do is worth stating plainly. It enforces the shape of an answer and
never its substance. A model that writes 211 bytes about an eighty-minute meeting
will not be made to understand it by being asked again — the floor rises, the ceiling
does not.

### 2. Measure the approved baseline

Run the complete path on an M1 Mac with 8 GB RAM. Record elapsed time, peak memory, swap behaviour, disk use, cancellation time, and whether the interface remains usable while work runs.

The current M1 Pro/16 GB measurements are valuable development evidence, not the release baseline.

### 3. Validate runtime and speaker distribution — the next thing to do

**Three of four sidecars now build and are self-contained.** On an Apple Silicon machine,
`localog-whisper` (3.3 MB), `localog-speaker-diarization` (23.4 MB) and `localog-speaker-embedding`
(14.5 MB) each link nothing outside `/usr/lib` and `/System/Library`, so there is no library to place
beside them. `npm run build:sidecar` builds all three from pinned revisions. Whisper transcribes real
audio with Metal; the embedding sidecar reproduces the reference meeting's grouping exactly.

What to do next, in this order, because each unblocks what follows:

1. ~~Package the application.~~ **Done on 15 August 2026, and it works.** `LocaLog.app` bundles all
   five sidecars into `Contents/MacOS` with the target suffix stripped, which is the shape the
   resolver expects; each runs from inside the bundle, and the FFmpeg licence texts ship in
   `Contents/Resources`. The application is ad-hoc signed, with no team identifier — enough to run
   locally, not enough to distribute.

   The `.dmg` step fails, and not for a reason in this project: `create-dmg` drives Finder over
   AppleScript to arrange the window, and the build machine has not granted Automation permission
   for Finder, so it stops with `-1743`. The application bundle is complete either way. A machine
   with that permission, or a CI runner, produces the disk image.

2. **Answer the system-audio question inside that package.** macOS gates capture behind Screen &
   System Audio Recording and hands an unauthorised tap silence rather than an error, and it will
   not attribute a permission request from a process below a terminal. So the capture code has never
   been observed to work, which is a different claim from it being broken. A packaged application
   has its own signed identity, shows its own dialog naming itself, and is where a user meets this
   anyway. Until it is answered, nothing more should be built on the assumption that the tap works.

3. **FFmpeg**, the only runtime with no sidecar at all: the application still requires it on the
   machine. It is the most predictable item left, because the licensing turns out to be the easy part
   and the build is the work.

   Licensing is straightforward here for one reason: FFmpeg is invoked as a separate executable
   rather than linked, so this is two programs talking and not one derived work. LocaLog is
   GPL-3.0-or-later and FFmpeg's GPL components are GPL-2.0-**or-later**, which is compatible; built
   without the GPL-only components it is LGPL and simpler again. What is still owed is the ordinary
   obligation — ship the licence texts, and be able to supply the source for the exact build.

   The build should be small rather than stock. The application uses FFmpeg to probe a file, turn
   anything into 16 kHz mono PCM, and encode Opus. A stock build is tens of megabytes of encoders,
   filters and network protocols that are never called; configured with `--disable-everything` and
   only the demuxers, decoders and encoders in use it is a few megabytes — less to ship, far less to
   audit, fewer advisories arriving for code that is never reached, and it avoids the GPL-only pieces
   by construction rather than by argument.

   Two alternatives were considered and rejected. Writing the decoding is not sensible: reading MP3,
   AAC and MP4 correctly is decades of accumulated edge cases, and it is the one dependency worth
   having. Using each platform's own decoders — AVFoundation, Media Foundation — removes the binary
   but costs three implementations with different format support, and worse, the same recording would
   produce different working audio and therefore different transcripts depending on the machine. For
   a tool whose output people rely on as a record, one decoder everywhere is worth more than the
   binary it would save.

4. **The M1 / 8 GB baseline**, which needs that hardware and has never been measured.

Then, for speaker separation: **replace the embedding model**, which is trained on Chinese and has
never been revisited. sherpa-onnx publishes several, and swapping one is a file change. This used to
cost eight minutes an attempt and now costs thirty-nine seconds, so trying two or three is an
afternoon — a question that was too expensive to ask casually is now cheap, which is the clearest
dividend of moving to embeddings. Also long recordings and overlapping speech. Decide whether the optional setting becomes the default once a verified runtime
exists, and what the review interface needs for renaming, reassignment and merging labels.

**That question has now been asked, and the answer reorders this list.** Three protocols from the
reference meeting differing only in their speaker labels — none, the embedding pass's twelve, and a
scattered fifty-four — kept 24, 20 and 23 of 35 stated figures. No benefit from the labels is
visible, and across all three drafts the string `Speaker N` appears **once**: the generator is handed
the speakers and attributes almost nothing.

The same runs exposed two failures that matter more than the speaker count. **No draft produced the
table of next steps** the style asks for explicitly and twice — which means the unowned-tasks check
shown beside a draft can never fire on this model, because it reads table rows and there are none.
And the unlabelled run produced **no headings at all**, 98 bullets against a style asking for
numbered sections.

That instruction-adherence question has since been asked too, across the installed models and at
three seeds each. The instructions are followable: both larger models produce the table
`qwen3.5:4b` never produced in five runs. **So the prompt is not the fault**, and the largest lever
available is the model.

What the seeds changed is which model. On identical input `granite4.1:8b` keeps 22, 19 and 6 of 35
stated figures — a run that loses five sixths of a meeting's figures is not a tool for producing a
record, and nothing in the output says which run it was. `gemma4:12b` keeps 27, 29 and 31, better at
its worst than granite at its best, at about 1.5× the time.

**`gemma4:12b` is the candidate for the default**, and the endorsement `qwen3.5:4b` carries in the
evaluation predates any structural measurement. See `docs/MODEL_EVALUATION.md`.

### 4. Harden the product

Add archive and basic backup/restore, finish the language settings, confirm audio playback, perform the accessibility and keyboard pass, audit ordinary logs and privacy boundaries, and remove or isolate unused experimental generation code.

### 5. Package and broaden platform validation

Only after the workflow and runtime choices are credible should the project enable bundling, signing, notarisation, and packaged Windows/Linux validation.

## Running the evaluation harnesses

They take minutes to hours against real models, which makes a few things easy to get
wrong and expensive to get wrong:

- **Check what is already running before starting anything.** Two generations on a
  16 GB machine compete for memory, spill onto the CPU and produce timings that mean
  nothing — and one run was left going for nine hours while a second was started
  beside it. `ps` and `ollama ps` answer this in a second.
- **`ollama ps` is the honest view of memory**, not the model's size on disk. A 7.1 GB
  model at a 40,960-token context occupies 14 GB. Anything reporting a CPU share
  rather than 100 % GPU is swapping and its timings should be discarded.
- **Never wait on a `pgrep` for a string the waiting command itself contains.** It
  matches its own command line and waits forever, which is how the nine hours passed.
- **Vary the seed.** A repeat at a fixed seed reproduces the run rather than testing
  it, and the spread between seeds has been larger than the spread between models.
- **Read the drafts, do not only count them.** Both faults worth fixing in August
  2026 — a literal `\n` printed mid-sentence, and an evidence check defeated by a
  model behaving well — were invisible to every structural measure.

## Definition of done for a milestone

Every milestone should end with:

- what was tested;
- measurements where they matter;
- risks discovered;
- a keep/change decision;
- the documentation update made in the same change;
- a clear statement of whether code is production, provisional, experimental, or discarded.

Green unit tests alone are not enough for a milestone involving real models, long files, user experience, or distribution.
