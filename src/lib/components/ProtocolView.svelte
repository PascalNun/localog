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

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  export let protocol: ProtocolDraft;
  export let style: ProtocolStyle;
  export let onNavigate: (route: AppRoute) => void;
  export let onSave: (markdown: string) => Promise<void>;
  export let onMarkReviewed: () => Promise<void>;
  export let onExport: (format: 'markdown' | 'text') => void;

  let markdown = protocol.markdown;
  let saveState: 'saved' | 'saving' = 'saved';
  let inspectorOpen = true;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  function scheduleSave() {
    saveState = 'saving';
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      await onSave(markdown);
      saveState = 'saved';
    }, 420);
  }

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
  });
</script>

<main class="workspace dense-workspace" id="main-content">
  <header class="workspace-header meeting-header protocol-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> {meeting.title}</p>
      <h1 tabindex="-1">Protocol editor</h1>
      <p>
        {meeting.lifecycle === 'reviewed' ? 'Reviewed revision' : 'Editable draft'} · Markdown backed
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
        <span>Markdown source</span><span class:busy={saveState === 'saving'} class="save-state"
          >{saveState === 'saving' ? 'Saving locally…' : `Saved ${protocol.savedAt}`}</span
        >
      </div>
      <label class="protocol-editor"
        ><span class="sr-only">Protocol Markdown</span><textarea
          bind:value={markdown}
          oninput={scheduleSave}
          spellcheck="true"></textarea></label
      >
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
          <h3>{meeting.lifecycle === 'reviewed' ? 'Reviewed' : 'Draft'}</h3>
          <p>
            {meeting.lifecycle === 'reviewed'
              ? 'Editing this revision returns it to draft.'
              : 'Generated content remains reviewable and editable.'}
          </p>
          {#if meeting.lifecycle !== 'reviewed'}<button
              class="secondary-action full-width"
              onclick={onMarkReviewed}><Icon name="check" size={16} /> Mark reviewed</button
            >{/if}
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
            <button class="primary-action full-width" onclick={() => onExport('markdown')}
              ><Icon name="download" size={16} /> Export Markdown</button
            ><button class="secondary-action full-width" onclick={() => onExport('text')}
              >Export plain text</button
            >
          </div>
        </div>
        <p class="refinement-note">
          Contextual AI refinement remains out of this shell until the editor/provider spikes prove
          safe revision behavior.
        </p>
      </aside>
    {/if}
  </div>
</main>
