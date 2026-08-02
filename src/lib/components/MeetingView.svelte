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
  export let onRename: (title: string) => Promise<void>;

  let editingTitle = false;
  let titleDraft = meeting.title;
  $: relevantJob = job?.meetingId === meeting.id ? job : null;
  $: transcriptionUnavailable = Boolean(relevantJob && relevantJob.state !== 'completed');

  async function saveTitle() {
    await onRename(titleDraft);
    editingTitle = false;
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
    />{/if}

  <section class="meeting-stage">
    {#if meeting.lifecycle === 'draft'}
      <div class="stage-message">
        <p class="eyebrow">Import in progress</p>
        <h2>Your original remains unchanged</h2>
        <p>
          The fake boundary exercises the same stable-meeting and transient-job separation intended
          for real local processing.
        </p>
      </div>
    {:else if meeting.lifecycle === 'source_ready'}
      <div class="stage-message">
        <p class="eyebrow">Source ready</p>
        <h2>Ready to transcribe locally</h2>
        <p>
          <strong>{meeting.sourceName}</strong> is assigned to this meeting. The Phase 0 fake has not
          read its contents.
        </p>
        <dl class="resolved-settings">
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
