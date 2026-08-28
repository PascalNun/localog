<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type {
    AppRoute,
    MeetingSummary,
    ProjectSummary,
    ProtocolDraft,
    ProtocolStyle,
  } from '../workflow/types';
  import Icon, { type IconName } from './Icon.svelte';
  import StageRail from './StageRail.svelte';
  import AppearanceFields from './AppearanceFields.svelte';
  import FurnitureEditor from './FurnitureEditor.svelte';
  import SectionList from './SectionList.svelte';
  import { fromElement, toMarkdown } from '../protocol/html';
  import { renderMarkdown } from '../protocol/markdown';
  import {
    APPEARANCE_CHOICES,
    PAGE_CONTENT_PIXELS,
    appearanceStyle,
    pageStarts,
  } from '../protocol/appearance';
  import { fieldLabel, furnitureIsEmpty, resolveRow } from '../protocol/furniture';
  import { diffWords, isUnchanged, type Change } from '../protocol/diff';
  import { documentFacts } from '../protocol/document';
  import { findInSource } from '../protocol/source';
  import { clockFromMillis } from '../time';
  import { errorMessage } from '../errors';
  import {
    appendSection,
    moveSection,
    newSection,
    readSections,
    removeSection,
  } from '../protocol/sections';
  import type {
    DocumentAppearance,
    FurnitureRow,
    PageFurniture,
    AppearancePreset,
    ExportFormat,
    NameReplacement,
    RefinedPassage,
    SetAsideSection,
    TranscriptSegment,
  } from '../workflow/types';
  import { t } from '../i18n';

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  export let protocol: ProtocolDraft;
  export let style: ProtocolStyle;
  export let onNavigate: (route: AppRoute) => void;
  export let onSave: (markdown: string) => Promise<void>;
  export let onCreateRevision: () => Promise<void>;
  export let onMarkReviewed: () => Promise<void>;
  export let onRestoreRevision: (revisionId: string) => Promise<void>;
  /// The page starts go with a PDF: print.ts cuts the sheet at them rather than
  /// asking the browser to paginate, because the browser it would ask does not
  /// repeat a running header across pages at all.
  export let onExport: (format: ExportFormat, pageStarts?: number[]) => void;
  export let onSetAppearance: (appearance: DocumentAppearance) => Promise<void> = async () =>
    undefined;
  export let onSetFurniture: (furniture: PageFurniture) => Promise<void> = async () => undefined;
  export let onSectionsChanged: (
    markdown: string,
    setAside: SetAsideSection[],
  ) => Promise<void> = async () => undefined;
  export let setAside: SetAsideSection[] = [];
  export let presets: AppearancePreset[] = [];
  export let onApplyPreset: (presetId: string) => Promise<void> = async () => undefined;
  export let onSavePreset: (name: string) => Promise<void> = async () => undefined;
  export let onDeletePreset: (presetId: string) => Promise<void> = async () => undefined;
  /// Which export this person reaches for, chosen once in Settings.
  ///
  /// It decides which button is the loud one and which comes first, and nothing
  /// else: every format stays one click away, because a default that hid the
  /// others would be a worse setting than none.
  export let defaultExport: ExportFormat = 'pdf';
  /// The transcript this protocol was written from, for checking a passage against
  /// what was actually said.
  export let transcript: { segments: TranscriptSegment[] } | null = null;
  export let onPreviewReplacement: (
    text: string,
    wrong: string,
    right: string,
  ) => Promise<NameReplacement> = async () => {
    throw new Error($t.protocol.replaceUnavailable);
  };
  export let onRefine: (
    passage: string,
    instruction: string,
  ) => Promise<RefinedPassage> = async () => {
    throw new Error($t.protocol.rewriteUnavailable);
  };

  let markdown = protocol.markdown;
  let saveState: 'saved' | 'saving' | 'failed' = protocol.saveState;
  /// Open beside the document where there is room, closed where the drawer would
  /// cover it.
  ///
  /// Below 900px the inspector is a drawer laid over the workspace rather than a
  /// column beside it, so starting open means arriving at a document with its middle
  /// hidden — the find bar and the right-hand third of every line were behind it.
  /// The direction is plain that the document must remain usable, so at that size it
  /// is opened deliberately rather than by default.
  let inspectorOpen = typeof window === 'undefined' || window.innerWidth > 900;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let editor: HTMLTextAreaElement;
  let findQuery = '';
  let findOpen = false;
  let textScale = 1;

  /// Which way the protocol is being worked on.
  ///
  /// The document is what somebody is making; the Markdown is how it is stored.
  /// Both are editable and both save to the same place, so the choice is only
  /// about which one somebody would rather look at.
  let view: 'document' | 'markdown' = 'document';
  let documentSurface: HTMLElement | null = null;

  /// The rendered document, held apart from what is being typed.
  ///
  /// Re-rendering while somebody types would replace the element under their caret
  /// on every keystroke and throw them to the top of the document. So the HTML is
  /// rebuilt only when the Markdown changed somewhere other than here — a revision
  /// restored, or an edit made in the Markdown view.
  let typingInDocument = false;
  let renderedFrom = '';
  let rendered = '';
  // A change guard: `renderedFrom` is written so the *next* evaluation can compare
  // against it. The rule sees a write nothing reads in this block and cannot model that.
  $: if (markdown !== renderedFrom && !typingInDocument) {
    rendered = renderMarkdown(markdown);
    // eslint-disable-next-line no-useless-assignment
    renderedFrom = markdown;
  }

  /// Read the document back after an edit.
  ///
  /// Markdown remains the stored form, so what a browser leaves in an editable
  /// region — divs for paragraphs, `b` for `strong`, non-breaking spaces holding
  /// the caret — is read back into it rather than kept.
  function readDocument() {
    if (!documentSurface) return;
    rememberAfterTyping();
    typingInDocument = true;
    markdown = toMarkdown(fromElement(documentSurface));
    renderedFrom = markdown;
    scheduleSave();
    if (showPages) queueMicrotask(measurePages);
    // Released after the save is scheduled, so the re-render above does not fire
    // against the surface somebody is still typing in.
    queueMicrotask(() => {
      typingInDocument = false;
    });
  }

  /// The formatting a protocol needs, and nothing further.
  ///
  /// `execCommand` is deprecated and is still the only thing every engine
  /// implements for this. What it produces is not trusted: whatever it leaves
  /// behind is read back through the same reader as everything else.
  function format(command: string, value?: string) {
    if (!documentSurface) return;
    remember();
    documentSurface.focus();
    document.execCommand(command, false, value);
    readDocument();
  }

  const BLOCK_FORMATS = [
    { label: 'Paragraph', tag: 'p' },
    { label: 'Heading 1', tag: 'h1' },
    { label: 'Heading 2', tag: 'h2' },
    { label: 'Heading 3', tag: 'h3' },
  ];

  /// What the cursor is standing in, and where it is on screen.
  ///
  /// The concept asks for a toolbar that appears at the selection and names the
  /// heading level — which means the editor has to know what block the caret is in,
  /// something `execCommand` never tells anybody. It is read from the selection
  /// instead, on every selection change, which is the one thing the browser will say
  /// reliably.
  let selectionBox: { top: number; left: number } | null = null;
  let currentBlock = 'p';
  let marks = { bold: false, italic: false };
  let moreOpen = false;

  /**
   * The nearest ancestor matching `selector`, provided it is inside the document.
   *
   * Three functions walked up to `documentSurface` by hand, differing only in what
   * they tested for on the way. The platform does this walk, and this same file
   * already asks it to a few lines away, with `.closest('table')` and
   * `.closest('tr')`.
   *
   * The hand-written loops stopped *before* `documentSurface` where `.closest()`
   * would consider it. That difference cannot bite: `documentSurface` is a div,
   * and none of the selectors used here matches one.
   */
  function closestIn(node: Node | null, selector: string): HTMLElement | null {
    const from = node?.nodeType === 1 ? (node as Element) : (node?.parentElement ?? null);
    const found = from?.closest(selector) ?? null;
    return found && documentSurface?.contains(found) ? (found as HTMLElement) : null;
  }

  /**
   * The block a node sits in, counted from the document's own children.
   *
   * Not expressible as a selector — it is "whichever child of the surface this is
   * inside", whatever that child happens to be — so it keeps its own loop, written
   * once instead of the two identical copies it was.
   */
  function topBlockOf(node: Node | null): Node | null {
    const block = topBlockOf(node);
    return block;
  }

  function blockOf(node: Node | null): string {
    return closestIn(node, 'h1,h2,h3,h4,p,li,blockquote,td,th')?.tagName.toLowerCase() ?? 'p';
  }

  function readSelection() {
    if (view !== 'document' || !documentSurface) {
      selectionBox = null;
      return;
    }
    const selection = window.getSelection();
    const range = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
    if (!range || !documentSurface.contains(range.commonAncestorContainer)) {
      selectionBox = null;
      return;
    }
    currentBlock = blockOf(range.startContainer);

    // Table controls follow the caret rather than a selection: nobody selects text
    // in order to add a row.
    const cell = cellAt(range.startContainer);
    const table = cell?.closest('table');
    if (table) {
      const rect = table.getBoundingClientRect();
      tableBox = { top: rect.top, left: rect.left };
    } else {
      tableBox = null;
    }
    marks = {
      bold: document.queryCommandState('bold'),
      italic: document.queryCommandState('italic'),
    };

    // Only when something is actually selected. A caret sitting in a word is not a
    // selection, and a toolbar that follows the caret is a toolbar in the way.
    if (range.collapsed) {
      selectionBox = null;
      if (!refineBusy) {
        refineOpen = false;
        customOpen = false;
      }
      return;
    }
    const rect = range.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) {
      selectionBox = null;
      return;
    }
    selectionBox = { top: rect.top, left: rect.left + rect.width / 2 };
  }

  onMount(() => {
    document.addEventListener('selectionchange', readSelection);
    window.addEventListener('keydown', handleShortcut);
    return () => {
      document.removeEventListener('selectionchange', readSelection);
      window.removeEventListener('keydown', handleShortcut);
    };
  });

  /// The keys a writing tool is expected to answer to.
  ///
  /// Find is the one somebody reaches for without thinking, and it was only ever
  /// reachable by finding the button first. Print is the other: every document
  /// application answers to it, and this one was leaving it to a webview that
  /// cannot print.
  function handleShortcut(event: KeyboardEvent) {
    const held = event.metaKey || event.ctrlKey;
    if (held && event.key.toLowerCase() === 'f') {
      event.preventDefault();
      findOpen = true;
      queueMicrotask(() => findField?.focus());
      return;
    }
    if (held && event.key.toLowerCase() === 'p') {
      // The key everybody presses for a PDF, doing what the export menu does.
      // Left to the webview it would print the application rather than the
      // protocol — and on macOS window.print() does nothing at all here, so it
      // would look broken rather than wrong.
      event.preventDefault();
      onExport('pdf', pageStartsNow());
      return;
    }
    if (event.key === 'Escape' && findOpen) {
      findOpen = false;
    }
  }

  let findField: HTMLInputElement | null = null;

  /// Whether the page has moved at all.
  ///
  /// The find bar is only frosted once there is something behind it to frost. At the
  /// top of a document it sits on the page with nothing passing under it, and a
  /// blur there is an effect for its own sake — so it fades in on the first scroll
  /// and fades out again at the top.
  ///
  /// Only a boolean changes, and only when the threshold is crossed, so the handler
  /// costs one comparison per scroll event and never reads layout.
  let workspace: HTMLElement | null = null;
  let pageMoved = false;

  function readScroll() {
    const moved = (workspace?.scrollTop ?? 0) > 6;
    if (moved !== pageMoved) pageMoved = moved;
  }

  // Clearing state on the way out of the document view. Nothing reads these
  // afterwards, which is what "cleared" means rather than a wasted write.
  $: if (view !== 'document') {
    selectionBox = null;
    // eslint-disable-next-line no-useless-assignment
    tableBox = null;
    moreOpen = false;
  }

  /// Asking the model to say a passage differently.
  ///
  /// Contextual and momentary, never a conversation: the selection goes to the
  /// model with an instruction, the answer replaces it, and undo takes it back like
  /// any other edit. Nothing runs unless somebody asks, and the passage travels
  /// alone — no transcript, no meeting — because the job is to rephrase what is
  /// there, and anything else the model could see is something it could add.
  let refineOpen = false;
  let refineBusy = '';
  let refineError = '';
  let customOpen = false;
  let customInstruction = '';
  /// A rewrite waiting to be looked at.
  ///
  /// Nothing is applied until somebody has seen what changed. Checking a rewrite
  /// afterwards catches some of what a small local model gets wrong and not all of
  /// it — "KW 38" becoming "Woche 38" loses nothing a checker can count — so the
  /// answer is not a better checker but a change nobody has to take on trust.
  let proposal: {
    range: Range;
    passage: string;
    revised: string;
    changes: Change[];
    missingFigures: string[];
    noticedChanges: string[];
    checked: boolean;
  } | null = null;

  const REFINEMENTS = [
    {
      id: 'clarity',
      label: $t.protocol.improveClarity,
      instruction: $t.protocol.improveClarityInstruction,
    },
    { id: 'shorter', label: $t.protocol.shorten, instruction: $t.protocol.shortenInstruction },
    {
      id: 'formal',
      label: $t.protocol.makeFormal,
      instruction: $t.protocol.makeFormalInstruction,
    },
    {
      id: 'plain',
      label: $t.protocol.makePlainer,
      instruction: $t.protocol.makePlainerInstruction,
    },
  ];

  async function refine(instruction: string, label: string) {
    if (!documentSurface) return;
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return;
    const range = selection.getRangeAt(0);
    const passage = range.toString();
    if (passage.trim() === '') return;

    refineBusy = label;
    refineError = '';
    try {
      const revised = await onRefine(passage, instruction);
      // The range is kept rather than the text: it is where the change goes if it
      // is wanted, and a selection does not survive a panel opening.
      proposal = {
        range: range.cloneRange(),
        passage,
        revised: revised.text,
        changes: diffWords(passage, revised.text),
        missingFigures: revised.missingFigures,
        noticedChanges: revised.noticedChanges,
        checked: revised.checked,
      };
    } catch (cause) {
      refineError = errorMessage(cause);
    } finally {
      refineBusy = '';
      refineOpen = false;
      customOpen = false;
      customInstruction = '';
    }
  }

  function acceptProposal() {
    if (!proposal || !documentSurface) return;
    remember();
    documentSurface.focus();
    const selection = window.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(proposal.range);
    // Applied through the selection so that undo owns it, rather than rebuilding
    // the document underneath the person who asked.
    document.execCommand('insertText', false, proposal.revised);
    proposal = null;
    readDocument();
  }

  function discardProposal() {
    proposal = null;
  }

  /// Paste, put through the document's own vocabulary on the way in.
  ///
  /// What arrives on the clipboard from Word or a browser is a thicket: styled
  /// spans, class names, `mso-` attributes, fonts and colours. The reader that
  /// takes this document back to Markdown already knows how to ignore all of it, so
  /// the cleanest sanitiser available is a round trip through it — parse what came,
  /// read it to Markdown, render that back. Anything the protocol vocabulary does
  /// not have cannot survive the journey, and the words inside it always do.
  function handlePaste(event: ClipboardEvent) {
    const clipboard = event.clipboardData;
    if (!clipboard) return;
    remember();
    const html = clipboard.getData('text/html');
    const plain = clipboard.getData('text/plain');
    if (!html && !plain) return;

    event.preventDefault();
    if (html) {
      const parsed = new DOMParser().parseFromString(html, 'text/html');
      const markdown = toMarkdown(fromElement(parsed.body));
      document.execCommand('insertHTML', false, renderMarkdown(markdown));
    } else {
      // Plain text is already clean; inserted as text so newlines become blocks
      // rather than one long line.
      document.execCommand('insertText', false, plain);
    }
    readDocument();
  }

  /// The reflexes anybody has in a list, which a bare editable region does not have.
  ///
  /// Tab and Shift-Tab move an item in and out; Enter on an empty item leaves the
  /// list rather than making a fourth empty bullet. Their absence is most of what
  /// makes an editor feel broken rather than merely limited.
  function handleKeydown(event: KeyboardEvent) {
    if (view !== 'document') return;

    if (event.key === 'Tab') {
      const inList = blockOf(window.getSelection()?.anchorNode ?? null) === 'li';
      if (!inList) return;
      event.preventDefault();
      document.execCommand(event.shiftKey ? 'outdent' : 'indent');
      readDocument();
      return;
    }

    if (event.key === 'Enter' && !event.shiftKey) {
      const selection = window.getSelection();
      const item = itemAt(selection?.anchorNode ?? null);
      if (item && item.textContent?.trim() === '') {
        event.preventDefault();
        document.execCommand('outdent');
        readDocument();
      }
    }
  }

  function itemAt(node: Node | null): HTMLElement | null {
    return closestIn(node, 'li');
  }

  /// Tables, which are the one thing `execCommand` has no answer for at all.
  ///
  /// The formal style ends in an actions table, so this is the most-used structure
  /// in a protocol and was the least editable: the cells could be typed into and
  /// nothing else. Adding a row is the operation somebody actually wants — one more
  /// action agreed, one struck — so these work on the document directly and the
  /// result is read back to Markdown like every other edit.
  let tableBox: { top: number; left: number } | null = null;

  function cellAt(node: Node | null): HTMLTableCellElement | null {
    return closestIn(node, 'td,th') as HTMLTableCellElement | null;
  }

  function currentCell(): HTMLTableCellElement | null {
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return null;
    return cellAt(selection.getRangeAt(0).startContainer);
  }

  /// A cell that can be clicked into even when it is empty.
  function blankCell(tag: 'td' | 'th'): HTMLTableCellElement {
    const cell = document.createElement(tag);
    cell.append(document.createElement('br'));
    return cell;
  }

  function putCaretIn(cell: HTMLElement) {
    const range = document.createRange();
    range.selectNodeContents(cell);
    range.collapse(true);
    const selection = window.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);
  }

  type TableCommand =
    'row-above' | 'row-below' | 'row-delete' | 'column-left' | 'column-right' | 'column-delete';

  /**
   * The table toolbar, as data.
   *
   * Six buttons that differed only in a command, an icon and a label, and each
   * label was written twice — once as the tooltip and once for a screen reader —
   * so twelve strings had to be kept saying the same thing by hand. Typed against
   * TableCommand, so a button for a command that does not exist will not compile.
   */
  const TABLE_COMMANDS: { command: TableCommand; icon: IconName; label: string }[] = [
    { command: 'row-above', icon: 'row-add-above', label: $t.protocol.addRowAbove },
    { command: 'row-below', icon: 'row-add-below', label: $t.protocol.addRowBelow },
    { command: 'row-delete', icon: 'row-remove', label: $t.protocol.deleteRow },
    { command: 'column-left', icon: 'column-add-left', label: $t.protocol.addColumnLeft },
    { command: 'column-right', icon: 'column-add-right', label: $t.protocol.addColumnRight },
    { command: 'column-delete', icon: 'column-remove', label: $t.protocol.deleteColumn },
  ];

  /** Where the row buttons end and the column buttons begin. */
  const TABLE_GROUP_BREAK = 3;

  function tableCommand(command: TableCommand) {
    const cell = currentCell();
    const row = cell?.closest('tr');
    const table = cell?.closest('table');
    if (!cell || !row || !table) return;
    const column = cell.cellIndex;
    const rows = Array.from(table.rows);

    switch (command) {
      case 'row-above':
      case 'row-below': {
        const fresh = document.createElement('tr');
        for (let index = 0; index < row.cells.length; index += 1) fresh.append(blankCell('td'));
        // A new row never goes above the header, because the first row is the
        // header when this is read back to Markdown.
        const isHeaderRow = row.parentElement?.tagName === 'THEAD';
        if (command === 'row-above' && !isHeaderRow) row.before(fresh);
        else row.after(fresh);
        // An inserted row must land in the body even when the header was the anchor.
        if (isHeaderRow && fresh.parentElement?.tagName === 'THEAD') {
          const body = table.tBodies[0] ?? table.createTBody();
          body.prepend(fresh);
        }
        putCaretIn(fresh.cells[0] ?? fresh);
        break;
      }
      case 'row-delete': {
        if (rows.length <= 1) {
          table.remove();
          break;
        }
        const next = rows[rows.indexOf(row) + 1] ?? rows[rows.indexOf(row) - 1];
        row.remove();
        if (next) putCaretIn(next.cells[Math.min(column, next.cells.length - 1)] ?? next);
        break;
      }
      case 'column-left':
      case 'column-right': {
        const at = command === 'column-left' ? column : column + 1;
        for (const each of rows) {
          const tag = each.parentElement?.tagName === 'THEAD' ? 'th' : 'td';
          const fresh = blankCell(tag);
          const reference = each.cells[at];
          if (reference) reference.before(fresh);
          else each.append(fresh);
        }
        putCaretIn(row.cells[at] ?? cell);
        break;
      }
      case 'column-delete': {
        if ((rows[0]?.cells.length ?? 0) <= 1) {
          table.remove();
          break;
        }
        for (const each of rows) each.cells[column]?.remove();
        const landing = row.cells[Math.min(column, row.cells.length - 1)];
        if (landing) putCaretIn(landing);
        break;
      }
    }

    readDocument();
    readSelection();
  }

  /// A new table: a header row and one row under it, which is the shape a protocol
  /// uses. Two columns, because the actions table is a task and who owns it.
  function insertTable() {
    if (!documentSurface) return;
    remember();
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return;
    const range = selection.getRangeAt(0);
    if (!documentSurface.contains(range.commonAncestorContainer)) return;

    const table = document.createElement('table');
    const head = table.createTHead().insertRow();
    head.append(blankCell('th'), blankCell('th'));
    const body = table.createTBody().insertRow();
    body.append(blankCell('td'), blankCell('td'));

    // Placed after whatever block the caret is in, never inside a paragraph.
    const block = topBlockOf(range.startContainer);
    // The document surface is a contenteditable and Svelte does not own its contents.
    // Inserting the node is the whole mechanism; going through Svelte's template
    // would re-render the surface and move the caret.
    if (block) (block as Element).after(table);
    // eslint-disable-next-line svelte/no-dom-manipulating
    else documentSurface.append(table);

    putCaretIn(head.cells[0] ?? table);
    readDocument();
    readSelection();
  }

  /// Saved ways of presenting a protocol.
  ///
  /// Applying one sets the appearance and the header and footer together, because
  /// they are two halves of one look and a preset that set only half would be a
  /// preset nobody could trust.
  let savingPreset = false;
  let presetName = '';
  let presetsOpen = false;
  $: presetSummary =
    presets.length === 0 ? $t.protocol.noneSaved : $t.protocol.savedCount(presets.length);

  const EXPORT_LABELS: Record<ExportFormat, string> = {
    pdf: $t.protocol.exportPdf,
    docx: $t.protocol.exportWord,
    markdown: $t.protocol.exportMarkdown,
    text: $t.protocol.exportPlainText,
  };
  /// The chosen one first, the rest in their usual order behind it.
  const EXPORT_ORDER: ExportFormat[] = ['pdf', 'docx', 'markdown', 'text'];
  $: exportOrder = [defaultExport, ...EXPORT_ORDER.filter((each) => each !== defaultExport)];

  async function savePreset() {
    const name = presetName.trim();
    if (name === '') return;
    await onSavePreset(name);
    presetName = '';
    savingPreset = false;
  }

  /// Where a passage of the protocol appears in the transcript.
  ///
  /// A search, not a provenance link, and the difference matters enough to say in
  /// the panel: nothing records which segment produced which sentence, and a
  /// protocol legitimately paraphrases, gathers one subject from four places, and
  /// states things the transcript only implies. Looking for the words is honest and
  /// is what somebody checking a draft against the recording actually wants.
  let lookingUp = '';
  $: sourceHits =
    lookingUp.trim() === '' || !transcript ? [] : findInSource(lookingUp, transcript.segments);

  /// Take whatever is selected in the document, or the block the caret is in.
  function lookUpSelection() {
    const selection = window.getSelection();
    const chosen = selection?.toString().trim() ?? '';
    if (chosen !== '') {
      lookingUp = chosen;
      inspectorTab = 'transcript';
      return;
    }
    const node = selection?.anchorNode ?? null;
    let block: Node | null = node;
    while (block && block.parentNode !== documentSurface) block = block.parentNode;
    const text = (block as HTMLElement | null)?.textContent?.trim() ?? '';
    if (text !== '') {
      lookingUp = text;
      inspectorTab = 'transcript';
    }
  }

  /// The parts the protocol is made of, read from its own headings.
  ///
  /// Nothing is stored to make the list: a protocol already says where its sections
  /// are. A second list kept alongside would be a second truth to keep in agreement
  /// with the first, and the first would win.
  $: sections = readSections(markdown);

  async function commitSections(next: string, stash: SetAsideSection[]) {
    remember();
    showAgain(next);
    await onSectionsChanged(next, stash);
  }

  /// Take a section out without throwing it away.
  ///
  /// It leaves the document, because the document must remain exactly what every
  /// export produces — a section kept in the file but hidden from the page would
  /// make the screen and the PDF differ, which is the one thing this editor exists
  /// not to do. It is kept beside the draft instead, and can be put back.
  async function setSectionAside(index: number) {
    const section = sections[index];
    if (!section) return;
    const { markdown: without, removed } = removeSection(markdown, index);
    await commitSections(without, [...setAside, { title: section.title, markdown: removed }]);
  }

  async function bringSectionBack(index: number) {
    const held = setAside[index];
    if (!held) return;
    await commitSections(
      appendSection(markdown, held.markdown),
      setAside.filter((_, at) => at !== index),
    );
  }

  async function addSection() {
    await commitSections(
      appendSection(markdown, newSection(markdown, $t.protocol.newSection)),
      setAside,
    );
  }

  /// Put the caret at a section, so the list is a way through the document.
  function goToSection(index: number) {
    if (view !== 'document' || !documentSurface) return;
    const headings = Array.from(
      documentSurface.querySelectorAll(`h${sections[index]?.level ?? 2}`),
    );
    const wanted = headings.find(
      (heading) => heading.textContent?.trim() === sections[index]?.title,
    );
    (wanted ?? headings[index])?.scrollIntoView({ block: 'center' });
  }

  /// Where the printed pages would end, drawn over the document.
  ///
  /// Not by cutting the document into pages: one editable region has to stay one
  /// editable region, or a selection cannot cross a page and the whole round trip to
  /// Markdown comes apart. The boundaries are measured and drawn over the top
  /// instead, which is a picture of the pagination rather than the pagination
  /// itself — and honest about being an estimate.
  ///
  /// Off unless asked for, because the concept wants no permanent page breaks: this
  /// is for checking what falls where, not for living in.
  let showPages = false;
  let pageEdges: number[] = [];

  /// Every page width now, because the paper follows the measure.
  ///
  /// This was A4 only, and for a good reason at the time: any other width set the
  /// text to a different column from the paper, so the lines fell differently and a
  /// break drawn here was a break nowhere. print.ts now derives the paper's side
  /// margin from the document's own measure, so the two columns are the same width
  /// and a break measured here is the break the printer makes.
  ///
  /// It stays an estimate at the widest settings, where the margin is clamped off
  /// the unprintable edge of the paper and the columns part company again — which
  /// is what the note under the document says either way.

  /// How much room to open between the foot of one page and the head of the next.
  const PAGE_GAP = 34;

  function clearPageGaps() {
    if (!documentSurface) return;
    for (const child of Array.from(documentSurface.children)) {
      const element = child as HTMLElement;
      if (element.dataset.pageStart) {
        delete element.dataset.pageStart;
        element.style.paddingTop = '';
      }
    }
  }

  /**
   * Where the pages begin, on the document as it is laid out now.
   *
   * Measured whether or not the breaks are being shown, because the export wants
   * them too: print.ts cuts the sheet at these indices rather than asking the
   * browser to paginate, and the browser it would be asking does not repeat a
   * running header at all.
   *
   * Indices into the document's own blocks, which are the blocks renderBlocks
   * produces — so an index here is the same block there.
   */
  function pageStartsNow(): number[] {
    if (!documentSurface) return [];
    const style = getComputedStyle(documentSurface);
    const top = parseFloat(style.paddingTop) || 0;
    const children = Array.from(documentSurface.children) as HTMLElement[];
    // A page holds the same text however large the screen shows it, so the zoom
    // stretches the picture rather than changing what fits.
    const gap = PAGE_GAP * textScale;

    // Measured on the paper rather than on the screen. The gaps measurePages opens
    // between the pages are a picture of where one ends and occupy nothing on the
    // page itself, so they come back out before anything is asked about what fits.
    let opened = 0;
    const blocks = children.map((element) => {
      const own = element.dataset.pageStart === 'true' ? gap : 0;
      const measured = {
        top: element.offsetTop - top - opened,
        height: element.offsetHeight - own,
        // The two the print stylesheet refuses to split.
        unbreakable: /^(H1|H2|H3|H4|TABLE)$/.test(element.tagName),
      };
      opened += own;
      return measured;
    });

    return pageStarts(blocks, PAGE_CONTENT_PIXELS * textScale);
  }

  function measurePages() {
    if (!documentSurface || !showPages) {
      clearPageGaps();
      pageEdges = [];
      return;
    }
    const gap = PAGE_GAP * textScale;
    const children = Array.from(documentSurface.children) as HTMLElement[];
    const starts = pageStartsNow();
    const beginsAPage = new Set(starts);
    children.forEach((element, index) => {
      if (beginsAPage.has(index)) {
        element.dataset.pageStart = 'true';
        // Padding rather than margin: a margin here would collapse into the one a
        // heading already carries, and the gap would come out the size of whichever
        // was larger instead of the size asked for.
        element.style.paddingTop = `${gap}px`;
      } else if (element.dataset.pageStart) {
        delete element.dataset.pageStart;
        element.style.paddingTop = '';
      }
    });

    // Read after the room was opened, and in the stack's own coordinates — the same
    // ones the overlay is positioned in — so the edge lands on the gap rather than
    // a document padding's distance above it.
    pageEdges = starts.map((index) => children[index]?.offsetTop ?? 0);
  }

  // Whenever the document, its setting or its scale changes the pagination moves.
  $: if (showPages && (rendered || textScale || appearance)) {
    queueMicrotask(measurePages);
  }
  $: if (!showPages) pageEdges = [];
  /// The band as it will print, from the same facts the exporters read.
  ///
  /// Shown in the gap between the pages rather than described in words above it.
  /// Every tool surveyed either shows the header in place on the page or shows
  /// nothing at all until export; this costs almost nothing, because the gap is
  /// already drawn and was already carrying a list of field names.
  $: facts = documentFacts(project, meeting, protocol);
  $: pageCount = pageEdges.length + 1;
  $: bandAt = (at: number, band: 'header' | 'footer') => {
    const row = furniture[band];
    const page = band === 'footer' ? at + 1 : at + 2;
    const marker = { number: String(page), ofCount: `${page} / ${pageCount}` };
    return {
      left: resolveRow(row.left, facts, marker),
      centre: resolveRow(row.centre, facts, marker),
      right: resolveRow(row.right, facts, marker),
    };
  };

  /// Two lines of furniture want more room between the sheets than a bare edge.
  $: pageGap = (furnitureIsEmpty(furniture) ? PAGE_GAP : PAGE_GAP + 26) * textScale;

  /// Zoom, which is how large the document looks and not how large it prints.
  ///
  /// The two were the same control, which meant somebody making the text bigger to
  /// read it was changing the document. The exported size lives in the document's
  /// own appearance and is not this.
  /// How the document is set, and the panel for changing it.
  ///
  /// Held by the project, because the reason anybody changes it is that a firm's
  /// protocols should look alike — so it is said in the panel rather than left to
  /// be discovered when the next protocol comes out the same.
  let appearanceOpen = false;

  /// Which face of the inspector is showing.
  ///
  /// Three jobs that share one column: what the document is, where it came from,
  /// and what it has been. Tabs rather than one long panel, because the panel had
  /// grown to seven sections and the ones somebody needs while writing were below
  /// the ones they need once a week.
  let inspectorTab: 'document' | 'transcript' | 'history' = 'document';
  const INSPECTOR_TABS = [
    { id: 'document', label: $t.protocol.tabDocument },
    { id: 'transcript', label: $t.protocol.tabTranscript },
    { id: 'history', label: $t.protocol.tabHistory },
  ] as const;

  function revisionMoment(at: number) {
    return new Date(at).toLocaleString(undefined, {
      day: 'numeric',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
  $: appearance = project.appearance;
  $: documentStyle = appearanceStyle(appearance);

  /// What repeats at the top and bottom of every printed page.
  ///
  /// Fields rather than typed text, so that a page number can be counted and a date
  /// comes from the meeting. Typed text is one of the fields, which is how anything
  /// the list does not cover still gets in.
  let furnitureOpen = false;
  $: furniture = project.furniture;
  $: furnitureSummary = furnitureIsEmpty(furniture)
    ? $t.protocol.nothingRepeated
    : [describeRow(furniture.header), describeRow(furniture.footer)]
        .filter((part) => part !== '')
        .join(' · ');

  function describeRow(row: FurnitureRow) {
    return [...row.left, ...row.centre, ...row.right].map(fieldLabel).join(', ');
  }

  const ZOOM_STEPS = [0.8, 0.9, 1, 1.1, 1.25, 1.5];
  $: zoomLabel = `${Math.round(textScale * 100)}%`;
  function zoom(direction: -1 | 1) {
    const at = ZOOM_STEPS.indexOf(textScale);
    const next = at === -1 ? 2 : Math.min(ZOOM_STEPS.length - 1, Math.max(0, at + direction));
    textScale = ZOOM_STEPS[next] ?? 1;
  }

  // Evidence for the reader, not a verdict on the draft. A protocol longer than
  // the meeting it records is the failure a figure count cannot see, so length is
  // stated beside the figures rather than left to be noticed.
  $: evidence = protocol.evidence ?? null;
  $: lengthAgainstRecording = evidence
    ? `${Math.round(evidence.charactersWritten / 1000)}k characters written from ${Math.round(
        evidence.charactersSpoken / 1000,
      )}k spoken.` +
      (evidence.charactersWritten > evidence.charactersSpoken / 2
        ? ' That is long for a record of a meeting.'
        : '')
    : '';

  /// The status as the inspector says it, in the interface's language.
  ///
  /// Deliberately not `reviewStateLabel`, which is still used by `documentFacts`
  /// for the header printed on the page. That one is part of the document rather
  /// than part of the application, and which language it should be in — the
  /// interface's, or the meeting's — is the decision still open on the board
  /// beside the ISO date.
  $: statusLabel =
    protocol.reviewState === 'reviewed'
      ? $t.protocol.statusReviewed
      : protocol.reviewState === 'changed_since_review'
        ? $t.protocol.statusChanged
        : $t.protocol.statusDraft;

  /// Let the document be as long as it is, and let the page do the scrolling.
  ///
  /// A textarea is a fixed box with its own scrollbar, so the editor scrolled inside
  /// a page that also scrolled: two bars for one document, and no way to see how long
  /// the protocol actually is. A textarea cannot grow by itself, so its height is set
  /// from its own content — reset to auto first, because scrollHeight never shrinks
  /// below the height already set.
  function growToFit(area: HTMLTextAreaElement | null) {
    if (!area) return;
    area.style.height = 'auto';
    area.style.height = `${area.scrollHeight}px`;
  }

  // On load, when the draft is replaced by another revision, and when the text size
  // changes — each of which changes how tall the same words are.
  $: if (editor && markdown !== undefined && textScale) {
    queueMicrotask(() => growToFit(editor));
  }

  function scheduleSave() {
    saveState = 'saving';
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      try {
        await onSave(markdown);
        saveState = 'saved';
      } catch {
        saveState = 'failed';
      }
    }, 420);
  }

  /// Undo that knows what a heading is.
  ///
  /// The browser's own undo covers typing in an editable region and nothing else.
  /// Adding a table row, removing a column, replacing a name through the document
  /// and accepting a rewrite are all done to the document directly, and none of
  /// them ever reached that stack — so undo silently skipped past the very
  /// operations somebody is most likely to want back.
  ///
  /// The history is kept over the Markdown, which is the document. That costs a
  /// re-render on undo, and buys an undo that means the same thing everywhere.
  let history: string[] = [];
  let future: string[] = [];
  const MOST_REMEMBERED = 60;

  /// Remember the document as it is now, before something changes it.
  function remember() {
    if (history[history.length - 1] === markdown) return;
    history = [...history, markdown].slice(-MOST_REMEMBERED);
    future = [];
  }

  /// Typing is remembered in pauses rather than per keystroke, so that undo steps
  /// back a phrase at a time as it does in a word processor.
  let restingTimer: ReturnType<typeof setTimeout> | null = null;
  function rememberAfterTyping() {
    if (restingTimer) clearTimeout(restingTimer);
    const before = markdown;
    restingTimer = setTimeout(() => {
      if (history[history.length - 1] !== before) {
        history = [...history, before].slice(-MOST_REMEMBERED);
        future = [];
      }
    }, 600);
  }

  function stepBack() {
    const previous = history[history.length - 1];
    if (previous === undefined) return;
    history = history.slice(0, -1);
    future = [...future, markdown];
    showAgain(previous);
  }

  function stepForward() {
    const next = future[future.length - 1];
    if (next === undefined) return;
    future = future.slice(0, -1);
    history = [...history, markdown];
    showAgain(next);
  }

  /// Put a remembered document back on screen.
  ///
  /// The surface is written to directly rather than left to the reactive render.
  /// Operations like adding a table row change the document without going through
  /// the rendered string, so after stepping back that string can be *identical* to
  /// what it already was — and nothing identical is ever reapplied. Undo then
  /// appeared to do nothing at all, which is how this was found.
  function showAgain(text: string) {
    markdown = text;
    rendered = renderMarkdown(text);
    renderedFrom = text;
    // The same contenteditable. Svelte will not re-render a surface it does not
    // control, which is why restoring a revision once appeared to do nothing at all.
    // eslint-disable-next-line svelte/no-dom-manipulating
    if (documentSurface) documentSurface.innerHTML = rendered;
    scheduleSave();
  }

  function editorCommand(command: 'undo' | 'redo') {
    if (view === 'markdown') {
      // A textarea's own undo is good and knows the caret; leave it alone.
      editor?.focus();
      document.execCommand(command);
      markdown = editor?.value ?? markdown;
      scheduleSave();
      return;
    }
    if (command === 'undo') stepBack();
    else stepForward();
  }

  /// How many times the search appears, counted on the stored form.
  ///
  /// The Markdown is the document, so counting there counts once for both views
  /// rather than once per view — and it is the number somebody wants before
  /// replacing a name through a protocol.
  $: matchCount =
    findQuery.trim() === '' ? 0 : markdown.toLowerCase().split(findQuery.toLowerCase()).length - 1;

  function findNext() {
    if (!findQuery) return;
    if (view === 'markdown') {
      const from = editor?.selectionEnd ?? 0;
      const lowerText = markdown.toLowerCase();
      const lowerQuery = findQuery.toLowerCase();
      let index = lowerText.indexOf(lowerQuery, from);
      if (index < 0) index = lowerText.indexOf(lowerQuery);
      if (index < 0) return;
      editor?.focus();
      editor?.setSelectionRange(index, index + findQuery.length);
      // A textarea that grows to its content does not scroll itself, so the page
      // has to be told where the match went.
      bringIntoView(index);
      return;
    }
    findNextInDocument();
  }

  /// Put a match in the middle of the window rather than wherever it happened to be.
  ///
  /// The editor is as tall as its text and the page does the scrolling, so nothing
  /// moves by itself when the selection changes.
  function bringIntoView(index: number) {
    if (!editor) return;
    const before = editor.value.slice(0, index).split('\n').length - 1;
    const lineHeight = parseFloat(getComputedStyle(editor).lineHeight) || 20;
    const target = editor.offsetTop + before * lineHeight;
    editor
      .closest('.workspace')
      ?.scrollTo({ top: Math.max(0, target - window.innerHeight / 2), behavior: 'smooth' });
  }

  /// The next occurrence in the rendered document.
  ///
  /// Walked over the text nodes rather than searched in the HTML, because the HTML
  /// contains tag names and a search for "table" would otherwise find the table
  /// rather than the word.
  function findNextInDocument() {
    if (!documentSurface) return;
    const walker = document.createTreeWalker(documentSurface, NodeFilter.SHOW_TEXT);
    const selection = window.getSelection();
    const from = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
    const needle = findQuery.toLowerCase();

    // Two passes: after the caret, then from the top, so the search wraps.
    for (const afterCaret of [true, false]) {
      walker.currentNode = documentSurface;
      let node = walker.nextNode();
      let reached = !afterCaret;
      while (node) {
        if (afterCaret && from && node === from.endContainer) {
          reached = true;
          const at = (node.nodeValue ?? '').toLowerCase().indexOf(needle, from.endOffset);
          if (at >= 0) return select(node, at);
        } else if (reached) {
          const at = (node.nodeValue ?? '').toLowerCase().indexOf(needle);
          if (at >= 0) return select(node, at);
        }
        node = walker.nextNode();
      }
    }
  }

  function select(node: Node, at: number) {
    const range = document.createRange();
    range.setStart(node, at);
    range.setEnd(node, at + findQuery.length);
    const selection = window.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);
    (node.parentElement ?? documentSurface)?.scrollIntoView({ block: 'center' });
  }

  /// Replace a name through the protocol, shown before it happens.
  ///
  /// The case that asks for this is a firm or a person named wrongly throughout, and
  /// a literal replace is not enough for it: German writes the interior of a compound
  /// in lower case, so `Klinker` hides inside `klinkerfassade` and a plain replace
  /// walks past it. The rule that finds both is the one the transcript corrections
  /// already use, and it is called rather than copied — a second copy would be a
  /// second answer.
  ///
  /// Shown first, then kept or not, for the same reason a rewrite is: a change made
  /// in forty places at once is not one somebody should meet after the fact.
  let replacement: NameReplacement | null = null;
  let replaceError = '';
  let replaceBusy = false;

  async function previewReplace() {
    if (findQuery.trim() === '') return;
    replaceBusy = true;
    replaceError = '';
    try {
      replacement = await onPreviewReplacement(markdown, findQuery, replaceQuery);
      if (replacement.matches.length === 0) replaceError = $t.protocol.nameNotFound;
    } catch (cause) {
      replacement = null;
      replaceError = errorMessage(cause);
    } finally {
      replaceBusy = false;
    }
  }

  function keepReplacement() {
    if (!replacement) return;
    remember();
    lastReplaced = replacement.matches.length;
    showAgain(replacement.markdown);
    replacement = null;
  }

  let replaceQuery = '';
  let lastReplaced = 0;

  async function createRevision() {
    await onCreateRevision();
    markdown = protocol.markdown;
    saveState = protocol.saveState;
  }

  async function markProtocolReviewed() {
    await onMarkReviewed();
    markdown = protocol.markdown;
    saveState = protocol.saveState;
  }

  async function restoreRevision(revisionId: string) {
    await onRestoreRevision(revisionId);
    markdown = protocol.markdown;
    saveState = protocol.saveState;
  }

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
    if (restingTimer) clearTimeout(restingTimer);
  });
</script>

<main
  bind:this={workspace}
  class="workspace stage-workspace"
  id="main-content"
  onscroll={readScroll}
>
  <header class="workspace-header meeting-header protocol-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> {meeting.title}</p>
      <h1 tabindex="-1">{$t.protocol.editor}</h1>
      <p>
        {statusLabel} · {$t.protocol.markdownBacked}
      </p>
    </div>
    <button
      class="secondary-action inspector-toggle"
      onclick={() => (inspectorOpen = !inspectorOpen)}>{$t.protocol.documentDetails}</button
    >
  </header>

  <StageRail meetingId={meeting.id} lifecycle={meeting.lifecycle} {onNavigate} />
  <div class:without-inspector={!inspectorOpen} class="context-layout protocol-layout">
    <div class="protocol-main" style={documentStyle}>
      <div class="editor-toolbar">
        <div class="editor-tools" aria-label={$t.protocol.editorTools}>
          <button class="text-action" onclick={() => editorCommand('undo')}
            ><Icon name="undo" size={16} /><span class="sr-only">Undo</span></button
          >
          <button class="text-action" onclick={() => editorCommand('redo')}
            ><Icon name="redo" size={16} /><span class="sr-only">Redo</span></button
          >
          <button class="text-action" onclick={() => (findOpen = !findOpen)}
            ><Icon name="search" size={15} /> {$t.protocol.find}</button
          >
          <span class="format-divider" aria-hidden="true"></span>
          <button class="text-action" aria-label={$t.protocol.zoomOut} onclick={() => zoom(-1)}
            >−</button
          >
          <span class="zoom-reading">{zoomLabel}</span>
          <button class="text-action" aria-label={$t.protocol.zoomIn} onclick={() => zoom(1)}
            >+</button
          >
          <span class="format-divider" aria-hidden="true"></span>
          <!-- Both of these lived behind the ⋯ menu, where the person who wrote the
               application could not find them either. -->
          <button
            class="text-action"
            title={$t.protocol.insertTable}
            disabled={view !== 'document'}
            onclick={() => insertTable()}
            ><Icon name="table" size={15} /><span class="sr-only">{$t.protocol.insertTable}</span
            ></button
          >
          <button
            class="text-action"
            class:chosen={showPages}
            aria-pressed={showPages}
            title={showPages ? 'Hide page breaks' : 'Show page breaks'}
            disabled={view !== 'document'}
            onclick={() => (showPages = !showPages)}
            ><Icon name="rule" size={15} /><span class="sr-only"
              >{showPages ? 'Hide page breaks' : 'Show page breaks'}</span
            ></button
          >
        </div>
        <div class="editor-trailing">
          <span
            class:busy={saveState === 'saving'}
            class:error={saveState === 'failed'}
            class="save-state"
            >{saveState === 'saving'
              ? 'Saving…'
              : saveState === 'failed'
                ? 'Autosave failed'
                : protocol.isDirty
                  ? 'Working edits saved'
                  : 'Revision saved'}</span
          >
          <div class="editor-menu">
            <button
              class="text-action"
              aria-haspopup="true"
              aria-expanded={moreOpen}
              aria-label={$t.protocol.documentMenu}
              onclick={() => (moreOpen = !moreOpen)}>⋯</button
            >
            {#if moreOpen}
              <div class="editor-menu-sheet" role="menu">
                <button
                  role="menuitem"
                  onclick={() => {
                    view = view === 'document' ? 'markdown' : 'document';
                    moreOpen = false;
                  }}
                  ><Icon name="document" size={15} />
                  {view === 'document' ? 'Markdown view' : 'Document view'}</button
                >
                <button
                  role="menuitem"
                  onclick={() => {
                    format('insertHorizontalRule');
                    moreOpen = false;
                  }}
                  disabled={view !== 'document'}
                  ><Icon name="rule" size={15} /> {$t.protocol.insertDivider}</button
                >
                <button
                  role="menuitem"
                  onclick={() => {
                    format('removeFormat');
                    moreOpen = false;
                  }}
                  disabled={view !== 'document'}
                  ><Icon name="close" size={15} /> {$t.protocol.clearFormatting}</button
                >
              </div>
            {/if}
          </div>
        </div>
      </div>
      {#if findOpen}<div class:floating={pageMoved} class="editor-find">
          <label
            ><span class="sr-only">{$t.protocol.findInProtocol}</span><input
              bind:this={findField}
              bind:value={findQuery}
              placeholder={$t.protocol.findInProtocol}
              onkeydown={(event) => event.key === 'Enter' && findNext()}
            /></label
          >
          <button class="secondary-action" onclick={findNext} disabled={matchCount === 0}
            >Next</button
          >
          <label
            ><span class="sr-only">{$t.protocol.replaceWith}</span><input
              bind:value={replaceQuery}
              placeholder={$t.protocol.replaceWith}
              onkeydown={(event) => event.key === 'Enter' && void previewReplace()}
            /></label
          >
          <button
            class="secondary-action"
            onclick={() => void previewReplace()}
            disabled={findQuery.trim() === '' || replaceBusy}
            >{replaceBusy ? 'Looking…' : 'Replace all'}</button
          >
          <span class="find-count"
            >{findQuery.trim() === ''
              ? ''
              : matchCount === 0
                ? 'Not found'
                : `${matchCount} ${matchCount === 1 ? 'match' : 'matches'}`}{lastReplaced > 0
              ? ` · replaced ${lastReplaced}`
              : ''}</span
          >
        </div>{/if}
      {#if replaceError}<p class="setting-error" role="alert">{replaceError}</p>{/if}
      {#if replacement && replacement.matches.length > 0}
        {@const waiting = replacement}
        <section class="proposal" aria-label={$t.protocol.proposedReplacement}>
          <div class="proposal-heading">
            <p class="eyebrow">
              {waiting.matches.length}
              {waiting.matches.length === 1 ? 'change' : 'changes'}, not yet made
            </p>
            <p>
              A capitalised name is looked for inside compounds as well, which is where a plain
              replace misses it. Read them, then keep them or leave them.
            </p>
          </div>
          <ul class="replacement-list">
            {#each waiting.matches.slice(0, 12) as match, at (at)}
              <li>
                <span class="replacement-line">Line {match.line}</span>
                <span class="replacement-context">{match.context}</span>
                <span class="replacement-change">{match.matched} → {match.replacement}</span>
              </li>
            {/each}
          </ul>
          {#if waiting.matches.length > 12}
            <p class="proposal-same">
              and {waiting.matches.length - 12} more, all of the same two forms.
            </p>
          {/if}
          <div class="proposal-actions">
            <button class="primary-action" onclick={keepReplacement}
              >{$t.protocol.makeChanges}</button
            >
            <button class="secondary-action" onclick={() => (replacement = null)}
              >{$t.protocol.leaveIt}</button
            >
          </div>
        </section>
      {/if}
      {#if view === 'document'}
        {#if tableBox}
          <div
            class="table-toolbar"
            role="toolbar"
            aria-label={$t.protocol.table}
            style={`top: ${tableBox.top}px; left: ${tableBox.left}px`}
          >
            <span class="table-toolbar-name">{$t.protocol.table}</span>
            <span class="format-divider" aria-hidden="true"></span>
            {#each TABLE_COMMANDS as entry, at (entry.command)}
              {#if at === TABLE_GROUP_BREAK}
                <span class="format-divider" aria-hidden="true"></span>
              {/if}
              <button
                class="text-action"
                title={entry.label}
                onclick={() => tableCommand(entry.command)}
                ><Icon name={entry.icon} size={15} /><span class="sr-only">{entry.label}</span
                ></button
              >
            {/each}
          </div>
        {/if}
        {#if selectionBox}
          <div
            class="selection-toolbar"
            role="toolbar"
            aria-label={$t.protocol.formatting}
            style={`top: ${selectionBox.top}px; left: ${selectionBox.left}px`}
          >
            <label class="block-choice">
              <span class="sr-only">{$t.protocol.blockType}</span>
              <select
                value={BLOCK_FORMATS.some((block) => block.tag === currentBlock)
                  ? currentBlock
                  : 'p'}
                onchange={(event) => format('formatBlock', event.currentTarget.value)}
              >
                {#each BLOCK_FORMATS as block (block.tag)}
                  <option value={block.tag}>{block.label}</option>
                {/each}
              </select>
            </label>
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              class:chosen={marks.bold}
              title={$t.protocol.bold}
              onclick={() => format('bold')}><Icon name="bold" size={15} /></button
            >
            <button
              class="text-action"
              class:chosen={marks.italic}
              title={$t.protocol.italic}
              onclick={() => format('italic')}><Icon name="italic" size={15} /></button
            >
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              title={$t.protocol.bulletedList}
              onclick={() => format('insertUnorderedList')}
              ><Icon name="list-bulleted" size={15} /></button
            >
            <button
              class="text-action"
              title={$t.protocol.numberedList}
              onclick={() => format('insertOrderedList')}
              ><Icon name="list-numbered" size={15} /></button
            >
            <button
              class="text-action"
              title={$t.protocol.quotation}
              onclick={() => format('formatBlock', 'blockquote')}
              ><Icon name="quote" size={15} /></button
            >
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              class:chosen={refineOpen}
              title={$t.protocol.askModel}
              aria-haspopup="true"
              aria-expanded={refineOpen}
              onclick={() => (refineOpen = !refineOpen)}
              disabled={refineBusy !== ''}
              >{refineBusy === '' ? 'Rewrite' : `${refineBusy}…`}</button
            >
            {#if refineOpen}
              <div class="refine-sheet" role="menu">
                {#each REFINEMENTS as choice (choice.id)}
                  <button
                    role="menuitem"
                    onclick={() => void refine(choice.instruction, choice.label)}
                    >{choice.label}</button
                  >
                {/each}
                <button role="menuitem" onclick={() => (customOpen = !customOpen)}
                  >{$t.protocol.customInstruction}</button
                >
                {#if customOpen}
                  <div class="refine-custom">
                    <label>
                      <span class="sr-only">{$t.protocol.whatShouldChange}</span>
                      <input
                        bind:value={customInstruction}
                        placeholder={$t.protocol.whatShouldChange}
                        onkeydown={(event) => {
                          if (event.key === 'Enter' && customInstruction.trim() !== '') {
                            void refine(customInstruction.trim(), 'Rewriting');
                          }
                        }}
                      />
                    </label>
                  </div>
                {/if}
                <p class="refine-note">
                  The passage goes to your local model on its own. Numbers, names and dates are to
                  come back unchanged — check them, and undo if they did not.
                </p>
              </div>
            {/if}
          </div>
        {/if}
        {#if refineError}
          <p class="setting-error" role="alert">{refineError}</p>
        {/if}
        {#if proposal}
          {@const waiting = proposal}
          <section class="proposal" aria-label={$t.protocol.proposedRewrite}>
            <div class="proposal-heading">
              <p class="eyebrow">{$t.protocol.proposedChange}</p>
              <p>
                Nothing has been changed yet. Read it, then keep it or leave it — a local model
                rewrites well and is not to be taken on trust.
              </p>
            </div>
            {#if isUnchanged(waiting.changes)}
              <p class="proposal-same">{$t.protocol.unchanged}</p>
            {:else}
              <p class="proposal-diff">
                {#each waiting.changes as change, at (at)}
                  {#if change.kind === 'same'}<span>{change.text}</span
                    >{:else if change.kind === 'removed'}<del>{change.text}</del>{:else}<ins
                      >{change.text}</ins
                    >{/if}
                {/each}
              </p>
            {/if}
            {#if waiting.missingFigures.length > 0}
              <p class="refine-lost" role="alert">
                <Icon name="warning" size={15} />
                <span
                  >{waiting.missingFigures.length === 1
                    ? 'A figure the passage stated is missing from this rewrite'
                    : `${waiting.missingFigures.length} figures the passage stated are missing from this rewrite`}:
                  {waiting.missingFigures.join(', ')}.</span
                >
              </p>
            {/if}
            {#if waiting.noticedChanges.length > 0}
              <div class="proposal-noticed">
                <p class="eyebrow">{$t.protocol.factsMoved}</p>
                <ul>
                  {#each waiting.noticedChanges as noticed, at (at)}
                    <li>{noticed}</li>
                  {/each}
                </ul>
                <p>
                  Asked of your own model, and it is wrong in both directions: it misses changes and
                  it queries wording that is fine. Worth a look, not a verdict.
                </p>
              </div>
            {:else if waiting.checked}
              <p class="proposal-checked">{$t.protocol.noFactMoved}</p>
            {/if}
            <div class="proposal-actions">
              <button class="primary-action" onclick={acceptProposal}>{$t.protocol.useThis}</button>
              <button class="secondary-action" onclick={discardProposal}
                >{$t.protocol.leaveIt}</button
              >
            </div>
          </section>
        {/if}
        <div class="document-stack">
          {#if showPages && pageEdges.length > 0}
            <div class="page-edges" aria-hidden="true">
              {#each pageEdges as edge, at (at)}
                <div class="page-edge" style={`top: ${edge}px; height: ${pageGap}px`}>
                  {#if !furnitureIsEmpty(furniture)}
                    {@const foot = bandAt(at, 'footer')}
                    <div class="page-edge-band">
                      <span>{foot.left}</span><span>{foot.centre}</span><span>{foot.right}</span>
                    </div>
                  {/if}
                  <span class="page-edge-label">Page {at + 2}</span>
                  {#if !furnitureIsEmpty(furniture)}
                    {@const head = bandAt(at, 'header')}
                    <div class="page-edge-band">
                      <span>{head.left}</span><span>{head.centre}</span><span>{head.right}</span>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div
            bind:this={documentSurface}
            class="protocol-document editable"
            contenteditable="true"
            role="textbox"
            tabindex="0"
            aria-multiline="true"
            aria-label={$t.protocol.protocolLabel}
            style={`--zoom: ${textScale}`}
            oninput={readDocument}
            onpaste={handlePaste}
            onkeydown={handleKeydown}
          >
            <!-- renderMarkdown escapes every run through escapeHtml, and safeLink allowlists
                 https, http and mailto, so a javascript: destination never reaches an href.
                 Checked rather than assumed. -->
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html rendered}
          </div>
          {#if showPages}
            <p class="page-note">
              Where the pages would end, measured the way the print stylesheet sets them: a heading
              or a table moves down whole rather than splitting, prose does not. The printer settles
              the last line or two, so treat this as within a line rather than exact.
            </p>
          {/if}
        </div>
      {:else}
        <label class="protocol-editor"
          ><span class="sr-only">{$t.protocol.protocolMarkdown}</span><textarea
            bind:this={editor}
            bind:value={markdown}
            oninput={(event) => {
              growToFit(event.currentTarget);
              scheduleSave();
            }}
            style={`font-size: ${textScale}rem`}
            spellcheck="true"></textarea></label
        >
      {/if}
    </div>

    {#if inspectorOpen}
      <aside class="context-inspector protocol-inspector" aria-label={$t.protocol.protocolDetails}>
        <div class="inspector-heading">
          <div class="inspector-tabs" role="tablist" aria-label={$t.protocol.protocolDetails}>
            {#each INSPECTOR_TABS as tab (tab.id)}
              <button
                role="tab"
                class="inspector-tab"
                class:chosen={inspectorTab === tab.id}
                aria-selected={inspectorTab === tab.id}
                onclick={() => (inspectorTab = tab.id)}>{tab.label}</button
              >
            {/each}
          </div>
          <button
            class="icon-button compact"
            aria-label={$t.protocol.closeInspector}
            onclick={() => (inspectorOpen = false)}><Icon name="close" size={16} /></button
          >
        </div>
        {#if inspectorTab === 'document'}
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.status}</p>
            <h3>{statusLabel}</h3>
            <p>
              {protocol.reviewState === 'changed_since_review'
                ? 'The reviewed revision is preserved. These working edits have not been reviewed.'
                : protocol.reviewState === 'reviewed'
                  ? 'This exact immutable revision was marked reviewed.'
                  : 'Generated content remains reviewable and editable.'}
            </p>
            {#if protocol.isDirty}<button
                class="secondary-action full-width"
                onclick={createRevision}
                ><Icon name="check" size={16} /> {$t.protocol.createRevision}</button
              >{/if}
            {#if protocol.reviewState !== 'reviewed'}<button
                class="secondary-action full-width"
                onclick={markProtocolReviewed}
                ><Icon name="check" size={16} /> {$t.protocol.markReviewed}</button
              >{/if}
          </div>
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.style}</p>
            <h3>{style.name}</h3>
            <p>{style.description}</p>
          </div>
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.sections}</p>
            <SectionList
              {sections}
              {setAside}
              onMove={async (from, to) => {
                await commitSections(moveSection(markdown, from, to), setAside);
              }}
              onSetAside={setSectionAside}
              onBringBack={bringSectionBack}
              onAdd={addSection}
              onGoTo={goToSection}
            />
          </div>
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.appearance}</p>
            <h3>
              {APPEARANCE_CHOICES.font.find((choice) => choice.value === appearance.font)?.label} ·
              {appearance.bodySize} pt
            </h3>
            <button
              class="inspector-control"
              aria-expanded={appearanceOpen}
              onclick={() => (appearanceOpen = !appearanceOpen)}
            >
              <Icon name="document" size={16} />
              <span>{$t.protocol.editAppearance}</span>
              <Icon name={appearanceOpen ? 'chevron-down' : 'chevron'} size={15} />
            </button>
            {#if appearanceOpen}
              <AppearanceFields
                {appearance}
                projectName={project.name}
                onChange={onSetAppearance}
              />
            {/if}
          </div>
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.headerFooter}</p>
            <h3>{furnitureSummary}</h3>
            <button
              class="inspector-control"
              aria-expanded={furnitureOpen}
              onclick={() => (furnitureOpen = !furnitureOpen)}
            >
              <Icon name="rule" size={16} />
              <span>{$t.protocol.editHeaderFooter}</span>
              <Icon name={furnitureOpen ? 'chevron-down' : 'chevron'} size={15} />
            </button>
            {#if furnitureOpen}
              <FurnitureEditor {furniture} projectName={project.name} onChange={onSetFurniture} />
            {/if}
          </div>
          <!--
            Presets: the appearance above and the header and footer above that, kept
            under a name so another project can be set the same way.

            This used to be "export templates", with a page in the sidebar beside
            Protocol styles and Names & terms, and its controls down in Export where
            they were nowhere near the settings they capture. A template held an
            appearance and a furniture and nothing else, which is those two panels
            under a name — a preset, and not a concept that earns a page of its own.
            Folding it in here removes the page, the nav entry and the word, and puts
            using and saving next to the thing being saved.
          -->
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.presets}</p>
            <h3>{presetSummary}</h3>
            <button
              class="inspector-control"
              aria-expanded={presetsOpen}
              onclick={() => (presetsOpen = !presetsOpen)}
            >
              <Icon name="folder" size={16} />
              <span>{$t.protocol.useOrSavePreset}</span>
              <Icon name={presetsOpen ? 'chevron-down' : 'chevron'} size={15} />
            </button>
            {#if presetsOpen}
              {#if presets.length > 0}
                <ul class="preset-list">
                  {#each presets as preset (preset.id)}
                    <li>
                      <div class="preset-name">
                        <strong>{preset.name}</strong>
                        <!-- What applying it will actually do. The page this replaced
                             showed the same line, and a name alone would have been less
                             than the page it replaced. -->
                        <small
                          >{APPEARANCE_CHOICES.font.find(
                            (choice) => choice.value === preset.appearance.font,
                          )?.label} · {preset.appearance.bodySize} pt · {APPEARANCE_CHOICES.pageWidth.find(
                            (choice) => choice.value === preset.appearance.pageWidth,
                          )?.label}</small
                        >
                      </div>
                      <button class="text-action" onclick={() => void onApplyPreset(preset.id)}>
                        {$t.protocol.use}
                      </button>
                      <!-- A shipped preset can be used and copied, never removed. -->
                      {#if !preset.builtIn}
                        <button
                          class="text-action is-danger"
                          onclick={() => void onDeletePreset(preset.id)}
                        >
                          {$t.protocol.remove}
                        </button>
                      {/if}
                    </li>
                  {/each}
                </ul>
              {/if}
              {#if savingPreset}
                <div class="preset-save">
                  <label>
                    <span class="sr-only">{$t.protocol.nameForPreset}</span>
                    <input
                      bind:value={presetName}
                      placeholder={$t.protocol.nameThisPreset}
                      onkeydown={(event) => event.key === 'Enter' && void savePreset()}
                    />
                  </label>
                  <button class="secondary-action" onclick={() => void savePreset()}
                    >{$t.protocol.save}</button
                  >
                  <button class="text-action" onclick={() => (savingPreset = false)}
                    >{$t.protocol.cancel}</button
                  >
                </div>
              {:else}
                <button class="text-action preset-save-open" onclick={() => (savingPreset = true)}
                  >{$t.protocol.saveAsPreset}</button
                >
              {/if}
            {/if}
          </div>
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.export}</p>
            <div class="export-actions">
              {#each exportOrder as format (format)}
                <button
                  class:primary-action={format === defaultExport}
                  class:secondary-action={format !== defaultExport}
                  class="full-width"
                  onclick={() =>
                    format === 'pdf' ? onExport('pdf', pageStartsNow()) : onExport(format)}
                >
                  {#if format === defaultExport}<Icon name="download" size={16} />{/if}
                  {EXPORT_LABELS[format]}
                </button>
              {/each}
            </div>
            <p class="export-note">
              The PDF is printed from the document you are reading, set the way this project sets
              its protocols — choose "Save as PDF" in the print dialog.
            </p>
          </div>
        {:else if inspectorTab === 'transcript'}
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.source}</p>
            <h3>{meeting.title}</h3>
            <p>
              Written from the reviewed transcript of this meeting. Nothing records which passage
              produced which sentence, so what follows looks for the words rather than claiming to
              know — a paraphrase will find nothing, which is the honest answer.
            </p>
            <button class="inspector-control" onclick={lookUpSelection}>
              <Icon name="search" size={16} />
              <span>{$t.protocol.findSelectedPassage}</span>
              <span></span>
            </button>
            {#if lookingUp.trim() !== ''}
              <p class="source-query">{$t.protocol.lookingFor} <em>{lookingUp.slice(0, 90)}</em></p>
              {#if sourceHits.length === 0}
                <p class="source-none">
                  None of these words appear together in the transcript. That usually means the
                  draft has put it in its own words, which it is entitled to do — the recording is
                  the place to check it.
                </p>
              {:else}
                <ul class="source-hits">
                  {#each sourceHits as hit (hit.segmentId)}
                    <li>
                      <span class="source-when">{clockFromMillis(hit.startMs)}</span>
                      <span class="source-said"
                        ><strong>{hit.speaker}</strong>
                        {hit.text}</span
                      >
                    </li>
                  {/each}
                </ul>
              {/if}
            {/if}
            <button
              class="text-action"
              onclick={() => onNavigate({ name: 'transcript', meetingId: meeting.id })}
              >{$t.protocol.openReviewedTranscript} <Icon name="arrow" size={15} /></button
            >
          </div>
          {#if evidence}
            <div class="inspector-section">
              <p class="eyebrow">{$t.protocol.whatToCheck}</p>
              <h3>{evidence.quantitiesAccounted} of {evidence.quantitiesStated} figures kept</h3>
              <p>
                The meeting stated {evidence.quantitiesStated} figures and this draft repeats {evidence.quantitiesAccounted}
                of them. How many belong here is a matter of the style you chose, so this is something
                to look at rather than a score.
              </p>
              {#if evidence.quantitiesInvented.length > 0}
                <p class="evidence-warning">
                  <Icon name="warning" size={15} />
                  <span
                    >{evidence.quantitiesInvented.length === 1
                      ? 'One figure appears here that the meeting did not state'
                      : `${evidence.quantitiesInvented.length} figures appear here that the meeting did not state`}:
                    {evidence.quantitiesInvented.join(', ')}. Worth confirming against the
                    recording.</span
                  >
                </p>
              {/if}
              {#if evidence.tasksUnowned && evidence.tasksUnowned.length > 0}
                <p class="evidence-unowned">
                  {evidence.tasksUnowned.length === 1
                    ? 'One task here has nobody against it'
                    : `${evidence.tasksUnowned.length} tasks here have nobody against them`}:
                  {evidence.tasksUnowned.join('; ')}. The draft leaves an owner out rather than
                  guessing at one, so this may be exactly what the meeting decided — and it is far
                  cheaper to put a name to it now than at the next meeting.
                </p>
              {/if}
              <p class="evidence-length">
                {lengthAgainstRecording}
              </p>
            </div>
          {/if}
        {:else}
          <div class="inspector-section">
            <p class="eyebrow">{$t.protocol.revisions}</p>
            <div class="revision-list">
              {#each [...protocol.revisions].reverse() as revision (revision.id)}
                <div>
                  <span
                    >Revision {revision.ordinal}<small
                      >{revision.status} · {revisionMoment(revision.createdAtMs)}</small
                    ></span
                  >
                  {#if revision.id === protocol.revisionId}
                    <span class="revision-current">{$t.protocol.current}</span>
                  {:else}
                    <button class="text-action" onclick={() => restoreRevision(revision.id)}
                      >{$t.protocol.restore}</button
                    >
                  {/if}
                </div>
              {/each}
            </div>
            <p class="refinement-note">
              Typing is kept as working edits and does not make a revision. A revision is made when
              a draft is generated, when you ask for one, when you mark a protocol reviewed, and
              when an older one is restored — so this list stays short enough to read.
            </p>
          </div>
        {/if}
        <p class="refinement-note">
          Nothing here rewrites your text for you. The draft is yours to edit, and every revision is
          kept.
        </p>
      </aside>
    {/if}
  </div>
</main>
