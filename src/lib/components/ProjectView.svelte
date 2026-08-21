<script lang="ts">
  import type {
    AppRoute,
    MeetingLifecycle,
    MeetingSummary,
    ProjectSummary,
  } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let project: ProjectSummary;
  export let meetings: MeetingSummary[];
  export let onNavigate: (route: AppRoute) => void;
  export let onDeleteMeeting: (meetingId: string) => Promise<void> = async () => undefined;

  /// Deleting is asked twice, in the row itself.
  ///
  /// A meeting takes its recording, its transcript and every protocol revision with
  /// it, so it should not go on one stray click — and a dialog for something this
  /// small would be more ceremony than the rest of the interface uses.
  let confirming = '';
  let deleteError = '';

  async function remove(meeting: MeetingSummary) {
    deleteError = '';
    try {
      await onDeleteMeeting(meeting.id);
    } catch (cause) {
      deleteError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      confirming = '';
    }
  }

  const lifecycleLabel: Record<MeetingLifecycle, string> = {
    draft: 'Draft',
    source_ready: 'Ready to transcribe',
    transcript_ready: 'Transcript ready',
    protocol_draft: 'Protocol draft',
    reviewed: 'Reviewed',
    archived: 'Archived',
  };

  function openMeeting(meeting: MeetingSummary) {
    if (meeting.lifecycle === 'transcript_ready')
      onNavigate({ name: 'transcript', meetingId: meeting.id });
    else if (['protocol_draft', 'reviewed'].includes(meeting.lifecycle))
      onNavigate({ name: 'protocol', meetingId: meeting.id });
    else onNavigate({ name: 'meeting', meetingId: meeting.id });
  }

  const formatDate = (value: string) =>
    new Intl.DateTimeFormat('en', { day: '2-digit', month: 'short', year: 'numeric' }).format(
      new Date(`${value}T12:00:00`),
    );
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header project-header">
    <div>
      <p class="eyebrow">Project</p>
      <h1 tabindex="-1">{project.name}</h1>
      <p>{project.description}</p>
    </div>
    <button
      class="primary-action"
      onclick={() => onNavigate({ name: 'new-meeting', projectId: project.id })}
    >
      New meeting <Icon name="plus" size={16} />
    </button>
  </header>

  <section class="document-section" aria-labelledby="meetings-heading">
    <div class="section-heading-row">
      <h2 id="meetings-heading">Meetings</h2>
      <span class="meta">Newest first · {meetings.length} total</span>
    </div>

    {#if meetings.length}
      <div class="meeting-index">
        <div class="meeting-index-header" aria-hidden="true">
          <span>Date</span><span>Meeting</span><span>Duration</span><span>Status</span><span></span>
        </div>
        {#each meetings as meeting (meeting.id)}
          <div class="meeting-row">
            <button class="meeting-open" onclick={() => openMeeting(meeting)}>
              <span>{formatDate(meeting.occurredAt)}</span>
              <span class="meeting-name"
                ><strong>{meeting.title}</strong><small>{meeting.sourceName}</small></span
              >
              <span>{meeting.durationLabel ?? '—'}</span>
              <span class="status-label"
                ><span class={`status-dot ${meeting.lifecycle}`}></span>{lifecycleLabel[
                  meeting.lifecycle
                ]}</span
              >
            </button>
            {#if confirming === meeting.id}
              <span class="meeting-confirm">
                <button class="text-action" onclick={() => void remove(meeting)}>Delete</button>
                <button class="text-action" onclick={() => (confirming = '')}>Keep</button>
              </span>
            {:else}
              <button
                class="icon-button compact meeting-delete"
                title={`Delete ${meeting.title}`}
                aria-label={`Delete ${meeting.title}`}
                onclick={() => (confirming = meeting.id)}><Icon name="close" size={15} /></button
              >
            {/if}
          </div>
        {/each}
        {#if confirming}
          <p class="meeting-delete-note">
            Deleting a meeting removes its recording, its transcript and every protocol revision,
            from this device. It cannot be undone.
          </p>
        {/if}
        {#if deleteError}<p class="setting-error" role="alert">{deleteError}</p>{/if}
      </div>
    {:else}
      <div class="empty-inline">
        <h3>No meetings yet</h3>
        <p>Import the first recording to begin this project’s meeting record.</p>
        <button
          class="text-action"
          onclick={() => onNavigate({ name: 'new-meeting', projectId: project.id })}
          >Import recording <Icon name="arrow" /></button
        >
      </div>
    {/if}
  </section>
</main>
