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
  import { APPEARANCE_CHOICES, appearanceStyle } from '../protocol/appearance';
  import {
    FURNITURE_FIELDS,
    fieldLabel,
    furnitureIsEmpty,
    needsPageNumbers,
  } from '../protocol/furniture';
  import type {
    DocumentAppearance,
    FurnitureField,
    FurnitureRow,
    PageFurniture,
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
    typingInDocument = true;
    markdown = toMarkdown(fromElement(documentSurface));
    renderedFrom = markdown;
    scheduleSave();
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
    marks = {
      bold: document.queryCommandState('bold'),
      italic: document.queryCommandState('italic'),
    };

    // Only when something is actually selected. A caret sitting in a word is not a
    // selection, and a toolbar that follows the caret is a toolbar in the way.
    if (range.collapsed) {
      selectionBox = null;
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
    moreOpen = false;
  }

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

  function editorCommand(command: 'undo' | 'redo') {
    editor.focus();
    document.execCommand(command);
    markdown = editor.value;
    scheduleSave();
  }

  function findNext() {
    if (!findQuery) return;
    const from = editor.selectionEnd;
    const lowerText = markdown.toLowerCase();
    const lowerQuery = findQuery.toLowerCase();
    let index = lowerText.indexOf(lowerQuery, from);
    if (index < 0) index = lowerText.indexOf(lowerQuery);
    if (index < 0) return;
    editor.focus();
    editor.setSelectionRange(index, index + findQuery.length);
  }

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
    <div class="protocol-main">
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
                  }}>{view === 'document' ? 'Markdown view' : 'Document view'}</button
                >
                <button
                  role="menuitem"
                  onclick={() => {
                    format('insertHorizontalRule');
                    moreOpen = false;
                  }}
                  disabled={view !== 'document'}>Insert divider</button
                >
                <button
                  role="menuitem"
                  onclick={() => {
                    format('removeFormat');
                    moreOpen = false;
                  }}
                  disabled={view !== 'document'}>Clear formatting</button
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
          <button class="secondary-action" onclick={findNext}>Next</button>
        </div>{/if}
      {#if view === 'document'}
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
              onclick={() => format('bold')}><strong>B</strong></button
            >
            <button
              class="text-action"
              class:chosen={marks.italic}
              title="Italic"
              onclick={() => format('italic')}><em>I</em></button
            >
            <span class="format-divider" aria-hidden="true"></span>
            <button
              class="text-action"
              title="Bulleted list"
              onclick={() => format('insertUnorderedList')}>•</button
            >
            <button
              class="text-action"
              title="Numbered list"
              onclick={() => format('insertOrderedList')}>1.</button
            >
            <button
              class="text-action"
              title="Quotation"
              onclick={() => format('formatBlock', 'blockquote')}>&rdquo;</button
            >
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
          style={`${documentStyle}; --zoom: ${textScale}`}
          oninput={readDocument}
        >
          {@html rendered}
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
            <p class="eyebrow">Appearance</p>
            <h3>
              {APPEARANCE_CHOICES.font.find((choice) => choice.value === appearance.font)?.label} ·
              {appearance.bodySize} pt
            </h3>
            <button
              class="text-action inspector-disclosure"
              aria-expanded={appearanceOpen}
              onclick={() => (appearanceOpen = !appearanceOpen)}
              >{appearanceOpen ? 'Done' : 'Edit appearance'}</button
            >
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
              class="text-action inspector-disclosure"
              aria-expanded={furnitureOpen}
              onclick={() => (furnitureOpen = !furnitureOpen)}
              >{furnitureOpen ? 'Done' : 'Edit header & footer'}</button
            >
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
