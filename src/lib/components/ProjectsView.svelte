<script lang="ts">
  import type {
    AppRoute,
    ArchivedWork,
    MeetingSummary,
    ProjectDetailsInput,
    ProjectSummary,
    ProtocolStyle,
  } from '../workflow/types';
  import { meetingLanguageLabel } from '../workflow/languages';
  import { formatMeetingDate } from '../protocol/document';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';
  import Icon from './Icon.svelte';
  import ProjectDetailsEditor from './ProjectDetailsEditor.svelte';

  export let projects: ProjectSummary[];
  export let meetings: MeetingSummary[];
  export let styles: ProtocolStyle[];
  export let onNavigate: (route: AppRoute) => void;
  export let onUpdateProject: (projectId: string, input: ProjectDetailsInput) => Promise<void>;
  export let onArchivedWork: () => Promise<ArchivedWork> = async () => ({
    projects: [],
    meetings: [],
  });
  export let onUnarchiveProject: (projectId: string) => Promise<void> = async () => undefined;

  let editing = '';
  let showArchived = false;
  let archived: ArchivedWork = { projects: [], meetings: [] };
  let archivedLoaded = false;
  let archivedLoading = false;
  let archivedError = '';
  let unarchiving = '';

  $: formatDate = (value: string) => formatMeetingDate(value, $t, 'short');
  $: styleNames = new Map(styles.map((style) => [style.id, style.name]));
  /// One pass rather than a filter and sort per row, and reactive so a meeting
  /// committed while the list is open updates the date it belongs to.
  $: lastMeetingByProject = meetings.reduce((latest, meeting) => {
    const known = latest.get(meeting.projectId);
    if (!known || meeting.occurredAt > known) latest.set(meeting.projectId, meeting.occurredAt);
    return latest;
  }, new Map<string, string>());

  async function toggleArchived() {
    showArchived = !showArchived;
    if (!showArchived || archivedLoaded) return;
    archivedLoading = true;
    archivedError = '';
    try {
      archived = await onArchivedWork();
      archivedLoaded = true;
    } catch (cause) {
      archivedError = errorMessage(cause);
    } finally {
      archivedLoading = false;
    }
  }

  async function bringBack(projectId: string) {
    unarchiving = projectId;
    archivedError = '';
    try {
      await onUnarchiveProject(projectId);
      archived = await onArchivedWork();
    } catch (cause) {
      archivedError = errorMessage(cause);
    } finally {
      unarchiving = '';
    }
  }

  async function saveDetails(projectId: string, input: ProjectDetailsInput) {
    // The editor owns the failure message; only a successful save closes it.
    await onUpdateProject(projectId, input);
    editing = '';
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header">
    <div>
      <p class="eyebrow">{$t.projects.eyebrow}</p>
      <h1 tabindex="-1">{$t.projects.title}</h1>
      <p>{$t.projects.lead}</p>
    </div>
    <div class="project-header-actions">
      <button
        class="primary-action"
        onclick={() => onNavigate({ name: 'new-project', returnToImport: false })}
      >
        {$t.sidebar.createProject}
        <Icon name="plus" size={16} />
      </button>
    </div>
  </header>

  <section class="document-section" aria-labelledby="projects-heading">
    <div class="section-heading-row">
      <h2 id="projects-heading">{$t.sidebar.projects}</h2>
      <span class="meta">{projects.length}</span>
    </div>

    {#if projects.length}
      <div class="project-index">
        <div class="project-index-header" aria-hidden="true">
          <div class="project-index-columns">
            <span>{$t.projects.columnProject}</span>
            <span>{$t.projects.columnDescription}</span>
            <span>{$t.projects.columnMeetings}</span>
            <span>{$t.projects.columnLastMeeting}</span>
            <span>{$t.projects.columnDefaults}</span>
          </div>
          <span></span>
        </div>
        {#each projects as project (project.id)}
          <div class="project-index-row">
            <button
              class="project-index-open"
              onclick={() => onNavigate({ name: 'project', projectId: project.id })}
            >
              <span class="project-index-name">{project.name}</span>
              <span>{project.description || '—'}</span>
              <span>{project.meetingCount}</span>
              <span
                >{lastMeetingByProject.has(project.id)
                  ? formatDate(lastMeetingByProject.get(project.id) ?? '')
                  : $t.projects.noMeetings}</span
              >
              <span
                >{meetingLanguageLabel(project.defaultLanguage)} · {styleNames.get(
                  project.defaultStyleId,
                ) ?? project.defaultStyleId}</span
              >
            </button>
            <button
              class="text-action project-index-edit"
              aria-expanded={editing === project.id}
              onclick={() => (editing = editing === project.id ? '' : project.id)}
              >{$t.projects.edit}</button
            >
          </div>
          {#if editing === project.id}
            <div class="project-index-editor">
              <ProjectDetailsEditor
                {project}
                {styles}
                onSave={(input) => saveDetails(project.id, input)}
                onCancel={() => (editing = '')}
              />
            </div>
          {/if}
        {/each}
      </div>
    {:else}
      <div class="empty-inline">
        <h3>{$t.projects.noProjects}</h3>
        <p>{$t.projects.noProjectsDetail}</p>
        <button
          class="text-action"
          onclick={() => onNavigate({ name: 'new-project', returnToImport: false })}
          >{$t.sidebar.createProject} <Icon name="arrow" /></button
        >
      </div>
    {/if}
  </section>

  <section class="document-section" aria-labelledby="archived-projects-heading">
    <div class="section-heading-row">
      <h2 id="archived-projects-heading">{$t.settings.archived}</h2>
      <button class="quiet-action" onclick={() => void toggleArchived()}>
        {showArchived ? $t.projects.hideArchived : $t.projects.showArchived}
      </button>
    </div>
    {#if showArchived}
      <p class="setting-hint">{$t.projects.archivedLead}</p>
      {#if archivedLoading}<p class="setting-hint">{$t.settings.working}</p>{/if}
      {#if archivedError}<p class="setting-error" role="alert">{archivedError}</p>{/if}
      {#if archived.projects.length}
        <div class="project-index">
          {#each archived.projects as project (project.id)}
            <div class="archived-project-row">
              <span>
                <strong>{project.name}</strong>
                <small>{project.description}</small>
              </span>
              <span class="count">{project.meetingCount}</span>
              <button
                class="text-action"
                disabled={unarchiving === project.id}
                onclick={() => void bringBack(project.id)}>{$t.settings.bringBack}</button
              >
            </div>
          {/each}
        </div>
      {:else if !archivedLoading}
        <p class="setting-hint">{$t.settings.nothingArchived}</p>
      {/if}
    {/if}
  </section>
</main>
