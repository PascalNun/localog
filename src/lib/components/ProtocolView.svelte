<script lang="ts">
  import { onDestroy } from 'svelte';
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
    { label: 'Text', tag: 'p', title: 'Ordinary paragraph' },
    { label: 'H1', tag: 'h1', title: 'Top-level heading' },
    { label: 'H2', tag: 'h2', title: 'Section heading' },
    { label: 'H3', tag: 'h3', title: 'Sub-section heading' },
  ];

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
        <div class="choice-row" role="group" aria-label="How to view the protocol">
          <button
            class="choice"
            class:chosen={view === 'document'}
            aria-pressed={view === 'document'}
            onclick={() => (view = 'document')}><Icon name="document" size={15} /> Document</button
          >
          <button
            class="choice"
            class:chosen={view === 'markdown'}
            aria-pressed={view === 'markdown'}
            onclick={() => (view = 'markdown')}>Markdown</button
          >
        </div>
        <div class="editor-tools" aria-label="Editor tools">
          <button class="text-action" onclick={() => editorCommand('undo')}>Undo</button>
          <button class="text-action" onclick={() => editorCommand('redo')}>Redo</button>
          <button class="text-action" onclick={() => (findOpen = !findOpen)}>Find</button>
          <button
            class="text-action"
            aria-label="Decrease text size"
            onclick={() => (textScale = Math.max(0.85, textScale - 0.1))}>A−</button
          >
          <button
            class="text-action"
            aria-label="Increase text size"
            onclick={() => (textScale = Math.min(1.4, textScale + 0.1))}>A+</button
          >
        </div>
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
        <div class="format-bar" role="toolbar" aria-label="Formatting">
          {#each BLOCK_FORMATS as block (block.tag)}
            <button
              class="text-action"
              title={block.title}
              onclick={() => format('formatBlock', block.tag)}>{block.label}</button
            >
          {/each}
          <span class="format-divider" aria-hidden="true"></span>
          <button class="text-action" title="Bold" onclick={() => format('bold')}
            ><strong>B</strong></button
          >
          <button class="text-action" title="Italic" onclick={() => format('italic')}
            ><em>I</em></button
          >
          <span class="format-divider" aria-hidden="true"></span>
          <button
            class="text-action"
            title="Bulleted list"
            onclick={() => format('insertUnorderedList')}>List</button
          >
          <button
            class="text-action"
            title="Numbered list"
            onclick={() => format('insertOrderedList')}>1. List</button
          >
          <button
            class="text-action"
            title="Quotation"
            onclick={() => format('formatBlock', 'blockquote')}>Quote</button
          >
        </div>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        <div
          bind:this={documentSurface}
          class="protocol-document editable"
          contenteditable="true"
          role="textbox"
          tabindex="0"
          aria-multiline="true"
          aria-label="Protocol"
          style={`font-size: ${textScale}rem`}
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
          <div>
            <p class="eyebrow">Document</p>
            <h2>Protocol</h2>
          </div>
          <button
            class="icon-button compact"
            aria-label="Close inspector"
            onclick={() => (inspectorOpen = false)}><Icon name="close" size={16} /></button
          >
        </div>
        <div class="inspector-section">
          <p class="eyebrow">Style</p>
          <h3>{style.name}</h3>
          <p>{style.description}</p>
        </div>
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
          {#if protocol.isDirty}<button class="secondary-action full-width" onclick={createRevision}
              ><Icon name="check" size={16} /> Create revision</button
            >{/if}
          {#if protocol.reviewState !== 'reviewed'}<button
              class="secondary-action full-width"
              onclick={markReviewed}><Icon name="check" size={16} /> Mark reviewed</button
            >{/if}
        </div>
        {#if evidence}
          <div class="inspector-section">
            <p class="eyebrow">What to check</p>
            <h3>{evidence.quantitiesAccounted} of {evidence.quantitiesStated} figures kept</h3>
            <p>
              The meeting stated {evidence.quantitiesStated} figures and this draft repeats {evidence.quantitiesAccounted}
              of them. How many belong here is a matter of the style you chose, so this is something to
              look at rather than a score.
            </p>
            {#if evidence.quantitiesInvented.length > 0}
              <p class="evidence-warning">
                <Icon name="warning" size={15} />
                <span
                  >{evidence.quantitiesInvented.length === 1
                    ? 'One figure appears here that the meeting did not state'
                    : `${evidence.quantitiesInvented.length} figures appear here that the meeting did not state`}:
                  {evidence.quantitiesInvented.join(', ')}. Worth confirming against the recording.</span
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
        <div class="inspector-section">
          <p class="eyebrow">Revision history</p>
          <div class="revision-list">
            {#each protocol.revisions as revision (revision.id)}
              <div>
                <span>Revision {revision.ordinal}<small>{revision.status}</small></span>
                {#if revision.id !== protocol.revisionId}<button
                    class="text-action"
                    onclick={() => restoreRevision(revision.id)}>Restore</button
                  >{/if}
              </div>
            {/each}
          </div>
        </div>
        <div class="inspector-section">
          <p class="eyebrow">Source</p>
          <button
            class="text-action"
            onclick={() => onNavigate({ name: 'transcript', meetingId: meeting.id })}
            >Open reviewed transcript <Icon name="arrow" size={15} /></button
          >
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
        </div>
        <p class="export-note">
          The PDF is A4, printed from the document you are reading — choose "Save as PDF" in the
          print dialog.
        </p>
        <p class="refinement-note">
          Nothing here rewrites your text for you. The draft is yours to edit, and every revision is
          kept.
        </p>
      </aside>
    {/if}
  </div>
</main>
