<script lang="ts">
  import type { ActiveJob, AppRoute, ProjectSummary } from '../workflow/types';
  import Icon from './Icon.svelte';
  import SidebarResizeHandle from './SidebarResizeHandle.svelte';
  import { clock } from '../time';
  import { tick } from 'svelte';
  import { errorMessage } from '../errors';
  import { formatBytes } from '../bytes';
  import { jobErrorTitle, stageText, t } from '../i18n';

  export let projects: ProjectSummary[];
  export let route: AppRoute;
  export let currentProjectId: string | null;
  export let activeJob: ActiveJob | null;
  /// Which meeting the work belongs to. The sidebar exists to answer that when the
  /// meeting is no longer on screen; the panel inside the meeting never needs to.
  export let activeJobMeeting: string | null = null;
  /// A recording in progress, wherever somebody happens to be.
  export let recording: { recording: boolean; seconds: number; meetingId: string | null } = {
    recording: false,
    seconds: 0,
    meetingId: null,
  };
  export let recordingMeeting: string | null = null;

  export let width: number;
  export let open = false;
  export let onNavigate: (route: AppRoute) => void;
  export let onClose: () => void;
  export let onArchiveProject: (projectId: string) => Promise<void> = async () => undefined;
  export let onRenameProject: (projectId: string, name: string) => Promise<void> = async () =>
    undefined;
  let projectMenu = '';
  let archiveBusy = false;
  let archiveError = '';
  let renamingProject = '';
  let nameDraft = '';
  let renameError = '';
  let renameInput: HTMLInputElement | null = null;
  function floatingMenu(node: HTMLElement) {
    // Keep the menu outside the scrolling/transformed sidebar on every webview.
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }
  async function openProjectMenu(projectId: string, event?: MouseEvent) {
    projectMenu = projectMenu === projectId ? '' : projectId;
    archiveError = '';
    await tick();
    if (!projectMenu) return;
    const menu = document.getElementById(`project-menu-${projectMenu}`);
    const anchor = document
      .getElementById(`project-actions-${projectMenu}`)
      ?.getBoundingClientRect();
    if (!menu || !anchor) return;
    const bounds = menu.getBoundingClientRect();
    const atPointer = event && (event.clientX !== 0 || event.clientY !== 0);
    const left = atPointer ? event.clientX : anchor.right + 6;
    const top = atPointer ? event.clientY : anchor.top;
    menu.style.left = `${Math.max(8, Math.min(left, window.innerWidth - bounds.width - 8))}px`;
    menu.style.top = `${Math.max(8, Math.min(top, window.innerHeight - bounds.height - 8))}px`;
    document.getElementById(`open-project-${projectMenu}`)?.focus();
  }

  /// Renaming happens in the row it belongs to, because a menu that opens a
  /// dialog for one word is more ceremony than the sidebar uses anywhere else.
  async function beginRename(project: ProjectSummary) {
    projectMenu = '';
    archiveError = '';
    renameError = '';
    renamingProject = project.id;
    nameDraft = project.name;
    await tick();
    renameInput?.focus();
    renameInput?.select();
  }

  async function saveRename(projectId: string) {
    const next = nameDraft.trim();
    if (!next) {
      renamingProject = '';
      return;
    }
    renameError = '';
    try {
      await onRenameProject(projectId, next);
      renamingProject = '';
    } catch (cause) {
      renameError = errorMessage(cause);
    }
  }

  function cancelRename() {
    renamingProject = '';
    renameError = '';
  }
  async function archiveProject(projectId: string) {
    archiveBusy = true;
    archiveError = '';
    try {
      await onArchiveProject(projectId);
      projectMenu = '';
    } catch (cause) {
      archiveError = errorMessage(cause);
    } finally {
      archiveBusy = false;
    }
  }
  function closeProjectMenu() {
    const previous = projectMenu;
    projectMenu = '';
    document.getElementById(`project-actions-${previous}`)?.focus();
  }
  export let onToggleTheme: () => void;
  /// What was chosen, as against what is on screen. "Auto" looks like whichever the
  /// system is showing, so the control has to say which of the two it is.
  export let themeChoice: 'auto' | 'light' | 'dark' = 'auto';

  /// The icon names the state the control is in, not the one it would move to.
  ///
  /// It showed a moon in light mode and a sun in dark: the theme you would get by
  /// clicking. That reads as a label for the current state and is the opposite of
  /// one, and it left automatic — which is the state most people are in — with no
  /// appearance of its own at all. Following the system was signalled by dimming
  /// the icon to 55%, which reads as "switched off" rather than as "automatic",
  /// and by a tooltip nobody sees unless they go looking.
  const THEME_ICON = { auto: 'monitor', light: 'sun', dark: 'moon' } as const;
  export let onResize: (width: number) => void;
  export let onResizeEnd: (width: number) => void;

  $: operationalJob = activeJob && !['completed'].includes(activeJob.state) ? activeJob : null;
  $: jobNeedsAttention =
    operationalJob?.state === 'failed' ||
    operationalJob?.state === 'interrupted' ||
    operationalJob?.state === 'cancelled' ||
    operationalJob?.requiresDuplicateConfirmation;
  // Two lines, two different facts. Naming the work here said the same thing as
  // the stage beneath it — "Transcribing" over "Transcribing · 43%" — and the same
  // words again in the panel inside the meeting, three times on one screen.
  //
  // So this line answers what the panel cannot: which meeting the work belongs to.
  // That is the question someone has once they have walked away from it, which is
  // the only time this corner of the screen is the one they are reading.
  $: jobHeading = jobNeedsAttention
    ? operationalJob?.requiresDuplicateConfirmation
      ? $t.sidebar.importNeedsDecision
      : $t.sidebar.needsAttention
    : (activeJobMeeting ?? workHeading(operationalJob?.kind));

  function workHeading(kind: string | undefined): string {
    switch (kind) {
      case 'import':
        return $t.sidebar.importingRecording;
      case 'transcription':
        return $t.sidebar.transcribing;
      case 'generation':
        return $t.sidebar.writingProtocol;
      default:
        return $t.sidebar.working;
    }
  }

  function jobDetail(job: ActiveJob) {
    if (jobNeedsAttention || job.state === 'queued')
      return job.error ? jobErrorTitle(job.error.code, job.kind) : stageText(job.stage);
    // Without a meeting name above it, the stage would be the only line, so the
    // work is named here instead of being lost.
    if (!activeJobMeeting) return `${stageLabel(job)} · ${progressLabel(job)}`;
    if (job.kind === 'import' && job.totalBytes !== null) {
      return `${formatBytes(job.progressBytes)} of ${formatBytes(job.totalBytes)}`;
    }
    return `${stageLabel(job)} · ${progressLabel(job)}`;
  }

  function stageLabel(job: ActiveJob) {
    return job.stage.toLowerCase().includes('speaker')
      ? $t.sidebar.separatingSpeakers
      : stageText(job.stage);
  }

  function progressLabel(job: ActiveJob) {
    return job.stage.toLowerCase().includes('speaker')
      ? $t.sidebar.workingEllipsis
      : `${job.progress}%`;
  }

  function navigate(nextRoute: AppRoute) {
    onNavigate(nextRoute);
    onClose();
  }
</script>

{#if open}
  <button class="sidebar-scrim" aria-label={$t.sidebar.closeNavigation} onclick={onClose}></button>
{/if}

<svelte:window
  onresize={() => (projectMenu = '')}
  onkeydown={(event) => {
    if (event.key === 'Escape' && projectMenu) closeProjectMenu();
  }}
  onclick={(event) => {
    if (
      !(event.target instanceof Element) ||
      (!event.target.closest('.sidebar-project-row') && !event.target.closest('.project-row-menu'))
    )
      projectMenu = '';
  }}
/>

<aside class:open class="sidebar" aria-label={$t.sidebar.primaryNavigation}>
  <button class="wordmark" onclick={() => navigate({ name: 'start' })}>
    <!-- The application's own mark, the same seven bars as its icon and its start
         page, cropped to the middle five so it reads at the height of a word. -->
    <svg class="wordmark-mark" viewBox="0 0 62 44" aria-hidden="true">
      <line x1="5" y1="16" x2="5" y2="28" />
      <line x1="17" y1="7" x2="17" y2="37" />
      <line x1="29" y1="2" x2="29" y2="42" />
      <line x1="41" y1="9" x2="41" y2="35" />
      <line x1="53" y1="17" x2="53" y2="27" />
    </svg>
    <span>LocaLog</span>
  </button>

  <nav class="sidebar-nav">
    <section aria-labelledby="projects-label">
      <div class="nav-section-heading">
        <button
          id="projects-label"
          class="nav-section-link"
          class:active={route.name === 'projects'}
          title={$t.sidebar.allProjects}
          onclick={() => navigate({ name: 'projects' })}>{$t.sidebar.projects}</button
        >
        <button
          class="icon-button compact"
          aria-label={$t.sidebar.createProject}
          onclick={() => navigate({ name: 'new-project', returnToImport: false })}
          ><Icon name="plus" size={15} /></button
        >
      </div>
      <div class="project-links">
        {#each projects as project (project.id)}
          <div class="sidebar-project-row" class:active={currentProjectId === project.id}>
            {#if renamingProject === project.id}
              <form
                class="project-rename"
                onsubmit={(event) => {
                  event.preventDefault();
                  void saveRename(project.id);
                }}
              >
                <input
                  bind:this={renameInput}
                  bind:value={nameDraft}
                  aria-label={$t.project.renameProject}
                  onkeydown={(event) => {
                    if (event.key === 'Escape') cancelRename();
                  }}
                />
              </form>
            {:else}
              <button
                class:active={currentProjectId === project.id}
                class="project-link"
                oncontextmenu={(event) => {
                  event.preventDefault();
                  void openProjectMenu(project.id, event);
                }}
                onclick={() => {
                  projectMenu = '';
                  renamingProject = '';
                  navigate({ name: 'project', projectId: project.id });
                }}
              >
                <span>{project.name}</span><span class="count">{project.meetingCount}</span>
              </button>
            {/if}
            <button
              id={`project-actions-${project.id}`}
              class="icon-button compact project-actions"
              aria-label={`${$t.project.actions}: ${project.name}`}
              title={$t.project.actions}
              aria-controls={projectMenu === project.id ? `project-menu-${project.id}` : undefined}
              aria-haspopup="menu"
              aria-expanded={projectMenu === project.id}
              onclick={() => void openProjectMenu(project.id)}
              ><svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true"
                ><circle cx="3" cy="8" r="1" fill="currentColor" /><circle
                  cx="8"
                  cy="8"
                  r="1"
                  fill="currentColor"
                /><circle cx="13" cy="8" r="1" fill="currentColor" /></svg
              ></button
            >
            {#if projectMenu === project.id}
              <div
                id={`project-menu-${project.id}`}
                use:floatingMenu
                class="project-row-menu"
                role="menu"
                aria-label={project.name}
              >
                <button
                  id={`open-project-${project.id}`}
                  role="menuitem"
                  class="text-action"
                  onclick={() => {
                    projectMenu = '';
                    navigate({ name: 'project', projectId: project.id });
                  }}>{$t.project.openProject}</button
                >
                <button
                  id={`rename-project-${project.id}`}
                  role="menuitem"
                  class="text-action"
                  onclick={() => void beginRename(project)}>{$t.project.rename}</button
                >
                <button
                  id={`edit-project-${project.id}`}
                  role="menuitem"
                  class="text-action"
                  onclick={() => {
                    projectMenu = '';
                    navigate({ name: 'project', projectId: project.id, edit: true });
                  }}>{$t.project.editDetails}</button
                >
                <button
                  id={`archive-project-${project.id}`}
                  role="menuitem"
                  class="text-action"
                  title={$t.settings.archivedLead}
                  disabled={archiveBusy}
                  onclick={() => void archiveProject(project.id)}
                  >{$t.project.archiveProject}</button
                >
                {#if archiveError}<p class="setting-error" role="alert">{archiveError}</p>{/if}
              </div>
            {/if}
          </div>
          {#if renameError && renamingProject === project.id}
            <p class="sidebar-error" role="alert">{renameError}</p>
          {/if}
        {/each}
      </div>
      <button
        class="text-action sidebar-action"
        onclick={() => navigate({ name: 'new-project', returnToImport: false })}
        ><Icon name="plus" size={15} /> {$t.sidebar.newProject}</button
      >
    </section>

    <section aria-labelledby="library-label">
      <div class="nav-section-heading"><span id="library-label">{$t.sidebar.library}</span></div>
      <button
        class:active={route.name === 'styles'}
        class="nav-link"
        onclick={() => navigate({ name: 'styles' })}
        ><Icon name="document" size={17} /> {$t.sidebar.protocolStyles}</button
      >
      <button
        class:active={route.name === 'vocabulary'}
        class="nav-link"
        onclick={() => navigate({ name: 'vocabulary' })}
        ><Icon name="book" size={17} /> {$t.sidebar.namesAndTerms}</button
      >
    </section>

    <section aria-labelledby="settings-label">
      <div class="nav-section-heading"><span id="settings-label">{$t.sidebar.settings}</span></div>
      <button
        class:active={route.name === 'settings'}
        class="nav-link"
        onclick={() => navigate({ name: 'settings' })}
        ><Icon name="settings" size={17} /> {$t.sidebar.settings}</button
      >
    </section>
  </nav>

  <div class:has-status={operationalJob || recording.recording} class="sidebar-footer">
    {#if recording.recording}
      <!-- Not a compliance device and not an alarm: a recording that is running is
           simply said, in the same quiet vocabulary as everything else, wherever the
           person happens to be. Hiding it would be the dishonest choice. -->
      <button
        class="local-status is-recording"
        aria-live="polite"
        onclick={() =>
          recording.meetingId && navigate({ name: 'recording', meetingId: recording.meetingId })}
      >
        <span class="recording-dot"></span>
        <span
          ><strong>{$t.sidebar.recording}</strong><small
            >{recordingMeeting ? `${recordingMeeting} · ` : ''}{clock(recording.seconds)}</small
          ></span
        >
      </button>
    {/if}
    {#if operationalJob}
      <!-- Somewhere to go. This said that something needed attention and gave no way
           to reach it: the meeting it belongs to was the one thing a reader wanted
           and the one thing it would not offer. -->
      <button
        class:attention={jobNeedsAttention}
        class="local-status"
        aria-live="polite"
        title={jobNeedsAttention
          ? $t.sidebar.openMeetingNeedingAttention
          : $t.sidebar.openThisMeeting}
        onclick={() =>
          operationalJob && navigate({ name: 'meeting', meetingId: operationalJob.meetingId })}
      >
        <span
          class:processing-dot={!jobNeedsAttention && operationalJob.state !== 'queued'}
          class:status-dot={jobNeedsAttention || operationalJob.state === 'queued'}
        ></span>
        <span><strong>{jobHeading}</strong><small>{jobDetail(operationalJob)}</small></span>
      </button>
    {/if}
    <button
      class="icon-button"
      onclick={onToggleTheme}
      aria-label={themeChoice === 'auto'
        ? $t.sidebar.themeFollowingSystem
        : themeChoice === 'light'
          ? $t.sidebar.themeAlwaysLight
          : $t.sidebar.themeAlwaysDark}
      title={themeChoice === 'auto'
        ? $t.sidebar.themeFollowingShort
        : themeChoice === 'light'
          ? $t.sidebar.themeAlwaysLightShort
          : $t.sidebar.themeAlwaysDarkShort}><Icon name={THEME_ICON[themeChoice]} /></button
    >
  </div>
  <SidebarResizeHandle {width} {onResize} {onResizeEnd} />
</aside>
