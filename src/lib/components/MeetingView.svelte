<script lang="ts">
  import type { ActiveJob, AppRoute, MeetingSummary, ProjectSummary } from '../workflow/types';
  import Icon from './Icon.svelte';
  import ProgressPanel from './ProgressPanel.svelte';
  import StageRail from './StageRail.svelte';

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  export let job: ActiveJob | null;
  export let onNavigate: (route: AppRoute) => void;
  export let onTranscribe: () => Promise<void>;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;
  export let onConfirmDuplicate: () => Promise<void>;
  export let onReselectSource: () => Promise<void>;
  export let onRename: (title: string) => Promise<void>;

  let editingTitle = false;
  let titleDraft = meeting.title;
  $: relevantJob = job?.meetingId === meeting.id ? job : null;
  $: transcriptionUnavailable = Boolean(relevantJob && relevantJob.state !== 'completed');

  async function saveTitle() {
    await onRename(titleDraft);
    editingTitle = false;
  }

  function formatBytes(bytes: number | null) {
    if (bytes === null) return 'Stored locally';
    if (bytes < 1_000_000) return `${Math.round(bytes / 1_000)} KB`;
    return `${(bytes / 1_000_000).toFixed(bytes >= 10_000_000 ? 0 : 1)} MB`;
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header meeting-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> Meeting</p>
      <div class="editable-title">
        {#if editingTitle}<input
            bind:value={titleDraft}
            aria-label="Meeting title"
            onkeydown={(event) => event.key === 'Enter' && saveTitle()}
          /> <button class="text-action" onclick={saveTitle}>Save</button>{:else}<h1 tabindex="-1">
            {meeting.title}
          </h1>
          <button
            class="icon-button compact"
            aria-label="Edit meeting title"
            onclick={() => (editingTitle = true)}>✎</button
          >{/if}
      </div>
      <p>
        {meeting.occurredAt} · {meeting.language} · {meeting.durationLabel ?? 'Duration pending'}
      </p>
    </div>
  </header>

  <StageRail meetingId={meeting.id} lifecycle={meeting.lifecycle} {onNavigate} />
  {#if relevantJob && relevantJob.state !== 'completed'}<ProgressPanel
      job={relevantJob}
      {onCancel}
      {onRetry}
      {onConfirmDuplicate}
      {onReselectSource}
    />{/if}

  <section class="meeting-stage">
    {#if meeting.lifecycle === 'draft'}
      <div class="stage-message">
        <p class="eyebrow">Source import</p>
        <h2>Your original remains unchanged</h2>
        <p>
          {#if relevantJob?.state === 'interrupted'}LocaLog was closed before the managed copy was
            committed. The meeting remains in Draft and the import can be retried safely.{:else if relevantJob?.state === 'cancelled'}The
            managed copy was cancelled. The meeting remains in Draft and the external file was not
            modified.{:else if relevantJob?.state === 'failed'}The managed copy could not be
            committed. The meeting remains in Draft and the external file was not modified.{:else}LocaLog
            is copying this source into private managed storage. It will become ready only after the
            copy has been validated and committed.{/if}
        </p>
      </div>
    {:else if meeting.lifecycle === 'source_ready'}
      <div class="stage-message">
        <p class="eyebrow">Source ready</p>
        <h2>Ready to transcribe locally</h2>
        <p>
          {#if meeting.sourceByteCount !== null}<strong>{meeting.sourceName}</strong> is safely stored
            with this meeting. The external original was not modified.{:else}<strong
              >{meeting.sourceName}</strong
            > is assigned to this synthetic browser meeting. No real media file was copied.{/if}
        </p>
        <dl class="resolved-settings">
          <div>
            <dt>Managed source</dt>
            <dd>
              {meeting.sourceByteCount === null
                ? 'Synthetic fixture'
                : formatBytes(meeting.sourceByteCount)}<small
                >{meeting.sourceMediaType ?? 'Browser preview'}</small
              >
            </dd>
          </div>
          <div>
            <dt>Language</dt>
            <dd>{meeting.language}<small>Meeting setting</small></dd>
          </div>
          <div>
            <dt>Preset</dt>
            <dd>Balanced<small>Global default</small></dd>
          </div>
          <div>
            <dt>Vocabulary</dt>
            <dd>Global + project<small>Project default</small></dd>
          </div>
        </dl>
        <button class="primary-action" onclick={onTranscribe} disabled={transcriptionUnavailable}
          >{transcriptionUnavailable ? 'Use the job controls above' : 'Transcribe'}
          <Icon name="arrow" /></button
        >
      </div>
    {:else if meeting.lifecycle === 'transcript_ready'}
      <div class="stage-message">
        <p class="eyebrow">Transcript ready</p>
        <h2>Review before generation</h2>
        <p>The timestamped transcript is ready for corrections and manual speaker mapping.</p>
        <button
          class="primary-action"
          onclick={() => onNavigate({ name: 'transcript', meetingId: meeting.id })}
          >Review transcript <Icon name="arrow" /></button
        >
      </div>
    {:else}
      <div class="stage-message">
        <p class="eyebrow">Protocol available</p>
        <h2>Continue in the document editor</h2>
        <p>The transcript remains available alongside the current protocol revision.</p>
        <button
          class="primary-action"
          onclick={() => onNavigate({ name: 'protocol', meetingId: meeting.id })}
          >Open protocol <Icon name="arrow" /></button
        >
      </div>
    {/if}
  </section>
</main>
