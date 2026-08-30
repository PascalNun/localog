<script lang="ts">
  import type {
    AppRoute,
    MeetingLifecycle,
    MeetingSummary,
    ProjectSummary,
  } from '../workflow/types';
  import Icon from './Icon.svelte';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';
  import { formatMeetingDate } from '../protocol/document';

  export let project: ProjectSummary;
  export let meetings: MeetingSummary[];
  export let onNavigate: (route: AppRoute) => void;
  export let onDeleteMeeting: (meetingId: string) => Promise<void> = async () => undefined;
  export let onArchiveProject: (projectId: string) => Promise<void> = async () => undefined;
  export let onArchiveMeeting: (meetingId: string) => Promise<void> = async () => undefined;

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
      deleteError = errorMessage(cause);
    } finally {
      confirming = '';
    }
  }

  async function archive(meeting: MeetingSummary) {
    deleteError = '';
    try {
      await onArchiveMeeting(meeting.id);
    } catch (cause) {
      deleteError = errorMessage(cause);
    } finally {
      confirming = '';
    }
  }

  // Reactive rather than constant: it was a constant when there was only one
  // language to write it in.
  $: lifecycleLabel = {
    draft: $t.lifecycle.draft,
    source_ready: $t.lifecycle.sourceReady,
    transcript_ready: $t.lifecycle.transcriptReady,
    protocol_draft: $t.lifecycle.protocolDraft,
    reviewed: $t.lifecycle.reviewed,
    archived: $t.lifecycle.archived,
  } satisfies Record<MeetingLifecycle, string>;

  function openMeeting(meeting: MeetingSummary) {
    if (meeting.lifecycle === 'transcript_ready')
      onNavigate({ name: 'transcript', meetingId: meeting.id });
    else if (['protocol_draft', 'reviewed'].includes(meeting.lifecycle))
      onNavigate({ name: 'protocol', meetingId: meeting.id });
    else onNavigate({ name: 'meeting', meetingId: meeting.id });
  }

  // Was its own `Intl` call pinned to `'en'`, so a German window listed its meetings
  // in English. One formatter now, and it reads the language like everything else.
  $: formatDate = (value: string) => formatMeetingDate(value, $t, 'short');
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header project-header">
    <div>
      <p class="eyebrow">{$t.project.eyebrow}</p>
      <h1 tabindex="-1">{project.name}</h1>
      <p>{project.description}</p>
    </div>
    <div class="project-header-actions">
      <!-- Archiving is not deleting: everything under the project stays, and it
           comes back from Settings whenever somebody wants it. So it is a quiet
           action beside the loud one rather than behind a confirmation. -->
      <button class="quiet-action" onclick={() => void onArchiveProject(project.id)}>
        {$t.project.archiveProject}
      </button>
      <button
        class="primary-action"
        onclick={() => onNavigate({ name: 'new-meeting', projectId: project.id })}
      >
        {$t.project.newMeeting}
        <Icon name="plus" size={16} />
      </button>
    </div>
  </header>

  <section class="document-section" aria-labelledby="meetings-heading">
    <div class="section-heading-row">
      <h2 id="meetings-heading">{$t.project.meetings}</h2>
      <span class="meta">{$t.project.newestFirst} · {meetings.length}</span>
    </div>

    {#if meetings.length}
      <div class="meeting-index">
        <!--
          Shaped like a row, because it is naming a row's columns.

          The four labels used to sit directly in the header, which shares its
          grid with `.meeting-row` — two columns. Four labels in two columns
          wrapped onto three lines, and had done since the header was written;
          English words are short enough that it read as an odd choice rather
          than as a fault. German is about a third longer and made it obvious.
          The inner element carries the same four columns the row's button does,
          so the labels now sit above the things they name.
        -->
        <div class="meeting-index-header" aria-hidden="true">
          <div class="meeting-index-columns">
            <span>{$t.project.columnDate}</span><span>{$t.project.columnMeeting}</span><span
              >{$t.project.columnDuration}</span
            ><span>{$t.project.columnStatus}</span>
          </div>
          <span></span>
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
                <!-- Archiving offered beside deleting, because most of what somebody
                     reaches for the cross to do is get it out of the list, and that
                     does not have to be permanent. -->
                <button class="text-action" onclick={() => void archive(meeting)}
                  >{$t.project.archive}</button
                >
                <button class="text-action" onclick={() => void remove(meeting)}
                  >{$t.project.delete}</button
                >
                <button class="text-action" onclick={() => (confirming = '')}
                  >{$t.project.keep}</button
                >
              </span>
            {:else}
              <button
                class="icon-button compact meeting-delete"
                title={$t.project.deleteMeeting(meeting.title)}
                aria-label={$t.project.deleteMeeting(meeting.title)}
                onclick={() => (confirming = meeting.id)}><Icon name="close" size={15} /></button
              >
            {/if}
          </div>
        {/each}
        {#if confirming}
          <p class="meeting-delete-note">
            {$t.project.deleteWarning}
          </p>
        {/if}
        {#if deleteError}<p class="setting-error" role="alert">{deleteError}</p>{/if}
      </div>
    {:else}
      <div class="empty-inline">
        <h3>{$t.project.noMeetings}</h3>
        <p>{$t.project.noMeetingsDetail}</p>
        <button
          class="text-action"
          onclick={() => onNavigate({ name: 'new-meeting', projectId: project.id })}
          >{$t.project.importRecording} <Icon name="arrow" /></button
        >
      </div>
    {/if}
  </section>
</main>
