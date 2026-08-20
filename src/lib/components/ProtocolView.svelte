<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type {
    AppRoute,
    MeetingSummary,
    ProjectSummary,
    ProtocolDraft,
    ProtocolStyle,
  } from '../workflow/types';
  import Icon from './Icon.svelte';
  import StageRail from './StageRail.svelte';
  import { fromElement, toMarkdown } from '../protocol/html';
  import { renderMarkdown } from '../protocol/markdown';
  import {
    APPEARANCE_CHOICES,
    PAGE_CONTENT_PIXELS,
    appearanceStyle,
    pageBreaks,
  } from '../protocol/appearance';
  import {
    FURNITURE_FIELDS,
    fieldLabel,
    furnitureIsEmpty,
    needsPageNumbers,
  } from '../protocol/furniture';
  import { diffWords, isUnchanged, type Change } from '../protocol/diff';
  import {
    appendSection,
    moveSection,
    newSection,
    readSections,
    removeSection,
  } from '../protocol/sections';
  import type {
    DocumentAppearance,
    FurnitureField,
    FurnitureRow,
    PageFurniture,
    RefinedPassage,
    SetAsideSection,
  } from '../workflow/types';

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  export let protocol: ProtocolDraft;
  export let style: ProtocolStyle;
  export let onNavigate: (route: AppRoute) => void;
  export let onSave: (markdown: string) => Promise<void>;
  export let onCreateRevision: () => Promise<void>;
  export let onMarkReviewed: () => Promise<void>;
  export let onRestoreRevision: (revisionId: string) => Promise<void>;
  export let onExport: (format: 'pdf' | 'docx' | 'markdown' | 'text') => void;
  export let onSetAppearance: (appearance: DocumentAppearance) => Promise<void> = async () =>
    undefined;
  export let onSetFurniture: (furniture: PageFurniture) => Promise<void> = async () => undefined;
  export let onSectionsChanged: (
    markdown: string,
    setAside: SetAsideSection[],
  ) => Promise<void> = async () => undefined;
  export let setAside: SetAsideSection[] = [];
  export let onRefine: (
    passage: string,
    instruction: string,
  ) => Promise<RefinedPassage> = async () => {
    throw new Error('Rewriting is not available here.');
  };

  let markdown = protocol.markdown;
  let saveState: 'saved' | 'saving' | 'failed' = protocol.saveState;
  let inspectorOpen = true;
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
  $: if (markdown !== renderedFrom && !typingInDocument) {
    rendered = renderMarkdown(markdown);
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

  function blockOf(node: Node | null): string {
    let at: Node | null = node;
    while (at && at !== documentSurface) {
      if (at.nodeType === 1) {
        const tag = (at as Element).tagName.toLowerCase();
        if (['h1', 'h2', 'h3', 'h4', 'p', 'li', 'blockquote', 'td', 'th'].includes(tag)) return tag;
      }
      at = at.parentNode;
    }
    return 'p';
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
    return () => document.removeEventListener('selectionchange', readSelection);
  });

  $: if (view !== 'document') {
    selectionBox = null;
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
    { id: 'clarity', label: 'Improve clarity', instruction: 'Make this clearer to read.' },
    { id: 'shorter', label: 'Shorten', instruction: 'Say this in fewer words.' },
    {
      id: 'formal',
      label: 'Make more formal',
      instruction: 'Make the register more formal, as a professional minute would be written.',
    },
    {
      id: 'plain',
      label: 'Make plainer',
      instruction: 'Make the wording plainer and more direct, without losing precision.',
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
      refineError = cause instanceof Error ? cause.message : String(cause);
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
    let at: Node | null = node;
    while (at && at !== documentSurface) {
      if (at.nodeType === 1 && (at as Element).tagName === 'LI') return at as HTMLElement;
      at = at.parentNode;
    }
    return null;
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
    let at: Node | null = node;
    while (at && at !== documentSurface) {
      if (at.nodeType === 1) {
        const element = at as Element;
        if (element.tagName === 'TD' || element.tagName === 'TH') {
          return element as HTMLTableCellElement;
        }
      }
      at = at.parentNode;
    }
    return null;
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

  function tableCommand(
    command:
      'row-above' | 'row-below' | 'row-delete' | 'column-left' | 'column-right' | 'column-delete',
  ) {
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
    let block: Node | null = range.startContainer;
    while (block && block.parentNode !== documentSurface) block = block.parentNode;
    if (block) (block as Element).after(table);
    else documentSurface.append(table);

    putCaretIn(head.cells[0] ?? table);
    readDocument();
    readSelection();
  }

  /// The parts the protocol is made of, read from its own headings.
  ///
  /// Nothing is stored to make the list: a protocol already says where its sections
  /// are. A second list kept alongside would be a second truth to keep in agreement
  /// with the first, and the first would win.
  $: sections = readSections(markdown);

  let draggingSection: number | null = null;

  async function commitSections(next: string, stash: SetAsideSection[]) {
    remember();
    showAgain(next);
    await onSectionsChanged(next, stash);
  }

  async function dropSection(onto: number) {
    const from = draggingSection;
    draggingSection = null;
    if (from === null || from === onto) return;
    await commitSections(moveSection(markdown, from, onto), setAside);
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
    await commitSections(appendSection(markdown, newSection(markdown, 'New section')), setAside);
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

  /// Only where the measure is the printed one.
  ///
  /// Any other page width sets the text to a different column from the paper, so the
  /// lines fall differently and a break drawn here would be a break nowhere. Better
  /// to say why than to draw a wrong one.
  $: pagesCanBeShown = appearance.pageWidth === 'a4';

  function measurePages() {
    if (!documentSurface || !showPages || !pagesCanBeShown) {
      pageEdges = [];
      return;
    }
    const style = getComputedStyle(documentSurface);
    const top = parseFloat(style.paddingTop) || 0;
    const blocks = Array.from(documentSurface.children).map((child) => {
      const element = child as HTMLElement;
      return {
        top: element.offsetTop - top,
        height: element.offsetHeight,
        // The two the print stylesheet refuses to split.
        unbreakable: /^(H1|H2|H3|H4|TABLE)$/.test(element.tagName),
      };
    });
    // A page holds the same text however large the screen shows it, so the zoom
    // stretches the picture rather than changing what fits.
    pageEdges = pageBreaks(blocks, PAGE_CONTENT_PIXELS * textScale);
  }

  // Whenever the document, its setting or its scale changes the pagination moves.
  $: if (showPages && (rendered || textScale || appearance)) {
    queueMicrotask(measurePages);
  }
  $: if (!showPages) pageEdges = [];

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
    { id: 'document', label: 'Document' },
    { id: 'transcript', label: 'Transcript' },
    { id: 'history', label: 'History' },
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
    ? 'Nothing repeated on the page'
    : [describeRow(furniture.header), describeRow(furniture.footer)]
        .filter((part) => part !== '')
        .join(' · ');

  function describeRow(row: FurnitureRow) {
    return [...row.left, ...row.centre, ...row.right].map(fieldLabel).join(', ');
  }

  async function addField(where: 'header' | 'footer', slot: keyof FurnitureRow, kind: string) {
    if (!kind) return;
    const field = (kind === 'text' ? { kind: 'text', value: '' } : { kind }) as FurnitureField;
    await onSetFurniture({
      ...furniture,
      [where]: { ...furniture[where], [slot]: [...furniture[where][slot], field] },
    });
  }

  async function removeField(where: 'header' | 'footer', slot: keyof FurnitureRow, at: number) {
    await onSetFurniture({
      ...furniture,
      [where]: {
        ...furniture[where],
        [slot]: furniture[where][slot].filter((_, index) => index !== at),
      },
    });
  }

  async function setFieldText(
    where: 'header' | 'footer',
    slot: keyof FurnitureRow,
    at: number,
    value: string,
  ) {
    await onSetFurniture({
      ...furniture,
      [where]: {
        ...furniture[where],
        [slot]: furniture[where][slot].map((field, index) =>
          index === at && field.kind === 'text' ? { kind: 'text', value } : field,
        ),
      },
    });
  }

  const FURNITURE_SLOTS: { id: keyof FurnitureRow; label: string }[] = [
    { id: 'left', label: 'Left' },
    { id: 'centre', label: 'Centre' },
    { id: 'right', label: 'Right' },
  ];

  async function changeAppearance<K extends keyof DocumentAppearance>(
    key: K,
    value: DocumentAppearance[K],
  ) {
    await onSetAppearance({ ...appearance, [key]: value });
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

  $: statusLabel =
    protocol.reviewState === 'changed_since_review'
      ? 'Changed since review'
      : protocol.reviewState === 'reviewed'
        ? 'Reviewed'
        : 'Draft';

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
      return;
    }
    findNextInDocument();
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

  /// Replace every occurrence, on the stored form.
  ///
  /// The case that asks for this is one name misspelt through a whole protocol, so
  /// it matches without regard to case and writes exactly what was typed. Done on
  /// the Markdown and re-rendered, which is why it works the same in both views.
  function replaceAll() {
    if (findQuery.trim() === '' || matchCount === 0) return;
    remember();
    const pattern = new RegExp(findQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
    markdown = markdown.replace(pattern, replaceQuery);
    renderedFrom = '';
    scheduleSave();
    lastReplaced = matchCount;
  }

  let replaceQuery = '';
  let lastReplaced = 0;

  async function createRevision() {
    await onCreateRevision();
    markdown = protocol.markdown;
    saveState = protocol.saveState;
  }

  async function markReviewed() {
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

<main class="workspace stage-workspace" id="main-content">
  <header class="workspace-header meeting-header protocol-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> {meeting.title}</p>
      <h1 tabindex="-1">Protocol editor</h1>
      <p>
        {statusLabel} · Markdown backed
      </p>
    </div>
    <button
      class="secondary-action inspector-toggle"
      onclick={() => (inspectorOpen = !inspectorOpen)}>Document details</button
    >
  </header>

  <StageRail meetingId={meeting.id} lifecycle={meeting.lifecycle} {onNavigate} />
  <div class:without-inspector={!inspectorOpen} class="context-layout protocol-layout">
    <div class="protocol-main" style={documentStyle}>
      <div class="editor-toolbar">
        <div class="editor-tools" aria-label="Editor tools">
          <button class="text-action" onclick={() => editorCommand('undo')}
            ><Icon name="undo" size={16} /><span class="sr-only">Undo</span></button
          >
          <button class="text-action" onclick={() => editorCommand('redo')}
            ><Icon name="redo" size={16} /><span class="sr-only">Redo</span></button
          >
          <button class="text-action" onclick={() => (findOpen = !findOpen)}
            ><Icon name="search" size={15} /> Find</button
          >
          <span class="format-divider" aria-hidden="true"></span>
          <button class="text-action" aria-label="Zoom out" onclick={() => zoom(-1)}>−</button>
          <span class="zoom-reading">{zoomLabel}</span>
          <button class="text-action" aria-label="Zoom in" onclick={() => zoom(1)}>+</button>
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
              aria-label="Document menu"
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
                    showPages = !showPages;
                    moreOpen = false;
                  }}
                  disabled={view !== 'document' || !pagesCanBeShown}
                  title={pagesCanBeShown
                    ? ''
                    : 'Set the page width to the A4 text column to see where the pages end.'}
                  ><Icon name="rule" size={15} />
                  {showPages ? 'Hide page breaks' : 'Show page breaks'}</button
                >
                <button
                  role="menuitem"
                  onclick={() => {
                    insertTable();
                    moreOpen = false;
                  }}
                  disabled={view !== 'document'}
                  ><Icon name="table" size={15} /> Insert table</button
                >
                <button
                  role="menuitem"
                  onclick={() => {
                    format('insertHorizontalRule');
                    moreOpen = false;
                  }}
                  disabled={view !== 'document'}
                  ><Icon name="rule" size={15} /> Insert divider</button
                >
                <button
                  role="menuitem"
                  onclick={() => {
                    format('removeFormat');
                    moreOpen = false;
                  }}
                  disabled={view !== 'document'}
                  ><Icon name="close" size={15} /> Clear formatting</button
                >
              </div>
            {/if}
          </div>
        </div>
      </div>
      {#if findOpen}<div class="editor-find">
          <label
            ><span class="sr-only">Find in protocol</span><input
              bind:value={findQuery}
              placeholder="Find in protocol"
              onkeydown={(event) => event.key === 'Enter' && findNext()}
            /></label
          >
          <button class="secondary-action" onclick={findNext} disabled={matchCount === 0}
            >Next</button
          >
          <label
            ><span class="sr-only">Replace with</span><input
              bind:value={replaceQuery}
              placeholder="Replace with"
              onkeydown={(event) => event.key === 'Enter' && replaceAll()}
            /></label
          >
          <button class="secondary-action" onclick={replaceAll} disabled={matchCount === 0}
            >Replace all</button
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
      {#if view === 'document'}
        {#if tableBox}
          <div
            class="table-toolbar"
            role="toolbar"
            aria-label="Table"
            style={`top: ${tableBox.top}px; left: ${tableBox.left}px`}
          >
            <span class="table-toolbar-name">Table</span>
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              title="Add a row above"
              onclick={() => tableCommand('row-above')}
              ><Icon name="row-add-above" size={15} /><span class="sr-only">Add a row above</span
              ></button
            >
            <button
              class="text-action"
              title="Add a row below"
              onclick={() => tableCommand('row-below')}
              ><Icon name="row-add-below" size={15} /><span class="sr-only">Add a row below</span
              ></button
            >
            <button
              class="text-action"
              title="Delete this row"
              onclick={() => tableCommand('row-delete')}
              ><Icon name="row-remove" size={15} /><span class="sr-only">Delete this row</span
              ></button
            >
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              title="Add a column to the left"
              onclick={() => tableCommand('column-left')}
              ><Icon name="column-add-left" size={15} /><span class="sr-only"
                >Add a column to the left</span
              ></button
            >
            <button
              class="text-action"
              title="Add a column to the right"
              onclick={() => tableCommand('column-right')}
              ><Icon name="column-add-right" size={15} /><span class="sr-only"
                >Add a column to the right</span
              ></button
            >
            <button
              class="text-action"
              title="Delete this column"
              onclick={() => tableCommand('column-delete')}
              ><Icon name="column-remove" size={15} /><span class="sr-only">Delete this column</span
              ></button
            >
          </div>
        {/if}
        {#if selectionBox}
          <div
            class="selection-toolbar"
            role="toolbar"
            aria-label="Formatting"
            style={`top: ${selectionBox.top}px; left: ${selectionBox.left}px`}
          >
            <label class="block-choice">
              <span class="sr-only">Block type</span>
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
              title="Bold"
              onclick={() => format('bold')}><Icon name="bold" size={15} /></button
            >
            <button
              class="text-action"
              class:chosen={marks.italic}
              title="Italic"
              onclick={() => format('italic')}><Icon name="italic" size={15} /></button
            >
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              title="Bulleted list"
              onclick={() => format('insertUnorderedList')}
              ><Icon name="list-bulleted" size={15} /></button
            >
            <button
              class="text-action"
              title="Numbered list"
              onclick={() => format('insertOrderedList')}
              ><Icon name="list-numbered" size={15} /></button
            >
            <button
              class="text-action"
              title="Quotation"
              onclick={() => format('formatBlock', 'blockquote')}
              ><Icon name="quote" size={15} /></button
            >
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              class:chosen={refineOpen}
              title="Ask the model to say this differently"
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
                  >Custom instruction…</button
                >
                {#if customOpen}
                  <div class="refine-custom">
                    <label>
                      <span class="sr-only">What should change?</span>
                      <input
                        bind:value={customInstruction}
                        placeholder="What should change?"
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
          <section class="proposal" aria-label="Proposed rewrite">
            <div class="proposal-heading">
              <p class="eyebrow">Proposed change</p>
              <p>
                Nothing has been changed yet. Read it, then keep it or leave it — a local model
                rewrites well and is not to be taken on trust.
              </p>
            </div>
            {#if isUnchanged(waiting.changes)}
              <p class="proposal-same">The model returned the passage unchanged.</p>
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
                <p class="eyebrow">A second pass thinks these facts moved</p>
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
              <p class="proposal-checked">A second pass found no fact moved. It misses things.</p>
            {/if}
            <div class="proposal-actions">
              <button class="primary-action" onclick={acceptProposal}>Use this</button>
              <button class="secondary-action" onclick={discardProposal}>Leave it</button>
            </div>
          </section>
        {/if}
        <div class="document-stack">
          {#if showPages && pageEdges.length > 0}
            <div class="page-edges" aria-hidden="true">
              {#each pageEdges as edge, at (at)}
                <div class="page-edge" style={`top: ${edge}px`}>
                  {#if !furnitureIsEmpty(furniture)}
                    <span class="page-edge-furniture">{furnitureSummary}</span>
                  {/if}
                  <span class="page-edge-label">Page {at + 2}</span>
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
            aria-label="Protocol"
            style={`--zoom: ${textScale}`}
            oninput={readDocument}
            onpaste={handlePaste}
            onkeydown={handleKeydown}
          >
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
          ><span class="sr-only">Protocol Markdown</span><textarea
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
      <aside class="context-inspector protocol-inspector" aria-label="Protocol details">
        <div class="inspector-heading">
          <div class="inspector-tabs" role="tablist" aria-label="Protocol details">
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
            aria-label="Close inspector"
            onclick={() => (inspectorOpen = false)}><Icon name="close" size={16} /></button
          >
        </div>
        {#if inspectorTab === 'document'}
          <div class="inspector-section">
            <p class="eyebrow">Status</p>
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
                onclick={createRevision}><Icon name="check" size={16} /> Create revision</button
              >{/if}
            {#if protocol.reviewState !== 'reviewed'}<button
                class="secondary-action full-width"
                onclick={markReviewed}><Icon name="check" size={16} /> Mark reviewed</button
              >{/if}
          </div>
          <div class="inspector-section">
            <p class="eyebrow">Style</p>
            <h3>{style.name}</h3>
            <p>{style.description}</p>
          </div>
          <div class="inspector-section">
            <p class="eyebrow">Sections</p>
            {#if sections.length === 0}
              <p class="section-none">
                This protocol has no headings yet, so there is nothing to list.
              </p>
            {:else}
              <ul class="section-list">
                {#each sections as section, index (section.from)}
                  <li
                    class:dragging={draggingSection === index}
                    draggable="true"
                    ondragstart={() => (draggingSection = index)}
                    ondragover={(event) => event.preventDefault()}
                    ondrop={() => void dropSection(index)}
                    ondragend={() => (draggingSection = null)}
                  >
                    <span class="section-grip" aria-hidden="true">⠿</span>
                    <button class="section-name" onclick={() => goToSection(index)}
                      >{section.title}</button
                    >
                    <button
                      class="icon-button compact"
                      title="Set this section aside"
                      aria-label={`Set aside ${section.title}`}
                      onclick={() => void setSectionAside(index)}
                      ><Icon name="close" size={14} /></button
                    >
                  </li>
                {/each}
              </ul>
            {/if}
            {#if setAside.length > 0}
              <p class="section-stash-label">Set aside</p>
              <ul class="section-list stashed">
                {#each setAside as held, index (held.title + index)}
                  <li>
                    <span class="section-grip" aria-hidden="true"></span>
                    <span class="section-name">{held.title}</span>
                    <button
                      class="icon-button compact"
                      title="Put this section back"
                      aria-label={`Put back ${held.title}`}
                      onclick={() => void bringSectionBack(index)}
                      ><Icon name="plus" size={14} /></button
                    >
                  </li>
                {/each}
              </ul>
            {/if}
            <button class="inspector-control" onclick={() => void addSection()}>
              <Icon name="plus" size={16} />
              <span>Add section</span>
              <span></span>
            </button>
            <p class="section-note">
              A section set aside leaves the document, so what you read is still exactly what is
              exported. It is kept here and can be put back.
            </p>
          </div>
          <div class="inspector-section">
            <p class="eyebrow">Appearance</p>
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
              <span>Edit appearance</span>
              <Icon name={appearanceOpen ? 'chevron-down' : 'chevron'} size={15} />
            </button>
            {#if appearanceOpen}
              <div class="appearance-fields">
                <label>
                  <span>Font</span>
                  <select
                    value={appearance.font}
                    onchange={(event) =>
                      void changeAppearance(
                        'font',
                        event.currentTarget.value as DocumentAppearance['font'],
                      )}
                  >
                    {#each APPEARANCE_CHOICES.font as choice (choice.value)}
                      <option value={choice.value}>{choice.label}</option>
                    {/each}
                  </select>
                </label>
                <label>
                  <span>Body size</span>
                  <select
                    value={appearance.bodySize}
                    onchange={(event) =>
                      void changeAppearance('bodySize', Number(event.currentTarget.value))}
                  >
                    {#each APPEARANCE_CHOICES.bodySize as choice (choice.value)}
                      <option value={choice.value}>{choice.label}</option>
                    {/each}
                  </select>
                </label>
                <label>
                  <span>Heading scale</span>
                  <select
                    value={appearance.headingScale}
                    onchange={(event) =>
                      void changeAppearance(
                        'headingScale',
                        event.currentTarget.value as DocumentAppearance['headingScale'],
                      )}
                  >
                    {#each APPEARANCE_CHOICES.headingScale as choice (choice.value)}
                      <option value={choice.value}>{choice.label}</option>
                    {/each}
                  </select>
                </label>
                <label>
                  <span>Line spacing</span>
                  <select
                    value={appearance.lineSpacing}
                    onchange={(event) =>
                      void changeAppearance(
                        'lineSpacing',
                        event.currentTarget.value as DocumentAppearance['lineSpacing'],
                      )}
                  >
                    {#each APPEARANCE_CHOICES.lineSpacing as choice (choice.value)}
                      <option value={choice.value}>{choice.label}</option>
                    {/each}
                  </select>
                </label>
                <label>
                  <span>Page width</span>
                  <select
                    value={appearance.pageWidth}
                    onchange={(event) =>
                      void changeAppearance(
                        'pageWidth',
                        event.currentTarget.value as DocumentAppearance['pageWidth'],
                      )}
                  >
                    {#each APPEARANCE_CHOICES.pageWidth as choice (choice.value)}
                      <option value={choice.value}>{choice.label}</option>
                    {/each}
                  </select>
                </label>
                <p class="appearance-note">
                  Applies to every protocol in {project.name}, so a firm's documents look alike. It
                  changes how the protocol is set, never what it says — that is the style above.
                </p>
              </div>
            {/if}
          </div>
          <div class="inspector-section">
            <p class="eyebrow">Header &amp; footer</p>
            <h3>{furnitureSummary}</h3>
            <button
              class="inspector-control"
              aria-expanded={furnitureOpen}
              onclick={() => (furnitureOpen = !furnitureOpen)}
            >
              <Icon name="rule" size={16} />
              <span>Edit header &amp; footer</span>
              <Icon name={furnitureOpen ? 'chevron-down' : 'chevron'} size={15} />
            </button>
            {#if furnitureOpen}
              <div class="furniture-editor">
                {#each [{ id: 'header', label: 'Header' }, { id: 'footer', label: 'Footer' }] as band (band.id)}
                  {@const where = band.id as 'header' | 'footer'}
                  <div class="furniture-band">
                    <p class="eyebrow">{band.label}</p>
                    {#each FURNITURE_SLOTS as slot (slot.id)}
                      <div class="furniture-slot">
                        <span class="furniture-slot-name">{slot.label}</span>
                        <div class="furniture-chips">
                          {#each furniture[where][slot.id] as field, at (at)}
                            <span class="furniture-chip">
                              {#if field.kind === 'text'}
                                <input
                                  value={field.value}
                                  placeholder="Your text"
                                  onchange={(event) =>
                                    void setFieldText(
                                      where,
                                      slot.id,
                                      at,
                                      event.currentTarget.value,
                                    )}
                                />
                              {:else}
                                {fieldLabel(field)}
                              {/if}
                              <button
                                class="furniture-remove"
                                aria-label={`Remove ${fieldLabel(field)}`}
                                onclick={() => void removeField(where, slot.id, at)}>×</button
                              >
                            </span>
                          {/each}
                          <select
                            value=""
                            aria-label={`Add to ${band.label} ${slot.label}`}
                            onchange={(event) => {
                              void addField(where, slot.id, event.currentTarget.value);
                              event.currentTarget.value = '';
                            }}
                          >
                            <option value="">Add…</option>
                            {#each FURNITURE_FIELDS as choice (choice.kind)}
                              <option value={choice.kind}>{choice.label}</option>
                            {/each}
                          </select>
                        </div>
                      </div>
                    {/each}
                  </div>
                {/each}
                {#if needsPageNumbers(furniture)}
                  <p class="appearance-note">
                    Word counts the pages itself, so the number is right there. The PDF is printed
                    by the browser, which will not say what page it is on — the page number is left
                    out of that one rather than printed wrongly on every sheet.
                  </p>
                {/if}
                <p class="appearance-note">
                  Applies to every protocol in {project.name}. It repeats on the printed page and is
                  not part of the document you are editing.
                </p>
              </div>
            {/if}
          </div>
          <div class="inspector-section">
            <p class="eyebrow">Export</p>
            <div class="export-actions">
              <button class="primary-action full-width" onclick={() => onExport('pdf')}
                ><Icon name="download" size={16} /> Export PDF</button
              ><button class="secondary-action full-width" onclick={() => onExport('docx')}
                >Export Word</button
              ><button class="secondary-action full-width" onclick={() => onExport('markdown')}
                >Export Markdown</button
              ><button class="secondary-action full-width" onclick={() => onExport('text')}
                >Export plain text</button
              >
            </div>
            <p class="export-note">
              The PDF is printed from the document you are reading, set the way this project sets
              its protocols — choose "Save as PDF" in the print dialog.
            </p>
          </div>
        {:else if inspectorTab === 'transcript'}
          <div class="inspector-section">
            <p class="eyebrow">Source</p>
            <h3>{meeting.title}</h3>
            <p>
              This protocol was written from the reviewed transcript of this meeting. Nothing in the
              document is linked back to a passage yet; that is still to come.
            </p>
            <button
              class="text-action"
              onclick={() => onNavigate({ name: 'transcript', meetingId: meeting.id })}
              >Open reviewed transcript <Icon name="arrow" size={15} /></button
            >
          </div>
          {#if evidence}
            <div class="inspector-section">
              <p class="eyebrow">What to check</p>
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
            <p class="eyebrow">Revisions</p>
            <div class="revision-list">
              {#each [...protocol.revisions].reverse() as revision (revision.id)}
                <div>
                  <span
                    >Revision {revision.ordinal}<small
                      >{revision.status} · {revisionMoment(revision.createdAtMs)}</small
                    ></span
                  >
                  {#if revision.id === protocol.revisionId}
                    <span class="revision-current">Current</span>
                  {:else}
                    <button class="text-action" onclick={() => restoreRevision(revision.id)}
                      >Restore</button
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
