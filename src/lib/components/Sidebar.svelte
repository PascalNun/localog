<script lang="ts">
  import type { ActiveJob, AppRoute, ProjectSummary } from '../workflow/types';
  import Icon from './Icon.svelte';
  import SidebarResizeHandle from './SidebarResizeHandle.svelte';

  export let projects: ProjectSummary[];
  export let route: AppRoute;
  export let currentProjectId: string | null;
  export let activeJob: ActiveJob | null;
  /// Which meeting the work belongs to. The sidebar exists to answer that when the
  /// meeting is no longer on screen; the panel inside the meeting never needs to.
  export let activeJobMeeting: string | null = null;
  export let width: number;
  export let open = false;
  export let theme: 'light' | 'dark';
  export let onNavigate: (route: AppRoute) => void;
  export let onClose: () => void;
  export let onToggleTheme: () => void;
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
      ? 'Import needs your decision'
      : 'Needs your attention'
    : (activeJobMeeting ?? workHeading(operationalJob?.kind));

  function workHeading(kind: string | undefined): string {
    switch (kind) {
      case 'import':
        return 'Importing the recording';
      case 'transcription':
        return 'Transcribing';
      case 'generation':
        return 'Writing the protocol';
      default:
        return 'Working';
    }
  }

  function jobDetail(job: ActiveJob) {
    if (jobNeedsAttention || job.state === 'queued') return job.error?.title ?? job.stage;
    // Without a meeting name above it, the stage would be the only line, so the
    // work is named here instead of being lost.
    if (!activeJobMeeting) return `${stageLabel(job)} · ${progressLabel(job)}`;
    if (job.kind === 'import' && job.totalBytes !== null) {
      return `${formatBytes(job.progressBytes)} of ${formatBytes(job.totalBytes)}`;
    }
    return `${stageLabel(job)} · ${progressLabel(job)}`;
  }

  function stageLabel(job: ActiveJob) {
    return job.stage.toLowerCase().includes('speaker') ? 'Separating speakers' : job.stage;
  }

  function progressLabel(job: ActiveJob) {
    return job.stage.toLowerCase().includes('speaker') ? 'Working…' : `${job.progress}%`;
  }

  function formatBytes(bytes: number) {
    if (bytes < 1_000_000) return `${Math.round(bytes / 1_000)} KB`;
    return `${(bytes / 1_000_000).toFixed(bytes >= 10_000_000 ? 0 : 1)} MB`;
  }

  function navigate(nextRoute: AppRoute) {
    onNavigate(nextRoute);
    onClose();
  }
</script>

{#if open}
  <button class="sidebar-scrim" aria-label="Close navigation" onclick={onClose}></button>
{/if}

<aside class:open class="sidebar" aria-label="Primary navigation">
  <button class="wordmark" onclick={() => navigate({ name: 'start' })}>LocaLog</button>

  <nav class="sidebar-nav">
    <section aria-labelledby="projects-label">
      <div class="nav-section-heading">
        <span id="projects-label">Projects</span>
        <button
          class="icon-button compact"
          aria-label="Create project"
          onclick={() => navigate({ name: 'new-project', returnToImport: false })}
          ><Icon name="plus" size={15} /></button
        >
      </div>
      <div class="project-links">
        {#each projects as project (project.id)}
          <button
            class:active={currentProjectId === project.id}
            class="project-link"
            onclick={() => navigate({ name: 'project', projectId: project.id })}
          >
            <span>{project.name}</span><span class="count">{project.meetingCount}</span>
          </button>
        {/each}
      </div>
      <button
        class="text-action sidebar-action"
        onclick={() => navigate({ name: 'new-project', returnToImport: false })}
        ><Icon name="plus" size={15} /> New project</button
      >
    </section>

    <section aria-labelledby="library-label">
      <div class="nav-section-heading"><span id="library-label">Library</span></div>
      <button
        class:active={route.name === 'styles'}
        class="nav-link"
        onclick={() => navigate({ name: 'styles' })}
        ><Icon name="document" size={17} /> Protocol styles</button
      >
      <button
        class:active={route.name === 'vocabulary'}
        class="nav-link"
        onclick={() => navigate({ name: 'vocabulary' })}
        ><Icon name="book" size={17} /> Vocabulary</button
      >
    </section>

    <section aria-labelledby="settings-label">
      <div class="nav-section-heading"><span id="settings-label">Settings</span></div>
      <button
        class:active={route.name === 'settings'}
        class="nav-link"
        onclick={() => navigate({ name: 'settings' })}
        ><Icon name="settings" size={17} /> Settings</button
      >
    </section>
  </nav>

  <div class:has-status={operationalJob} class="sidebar-footer">
    {#if operationalJob}
      <div class:attention={jobNeedsAttention} class="local-status" aria-live="polite">
        <span
          class:processing-dot={!jobNeedsAttention && operationalJob.state !== 'queued'}
          class:status-dot={jobNeedsAttention || operationalJob.state === 'queued'}
        ></span>
        <span><strong>{jobHeading}</strong><small>{jobDetail(operationalJob)}</small></span>
      </div>
    {/if}
    <button
      class="icon-button"
      onclick={onToggleTheme}
      aria-label={`Use ${theme === 'light' ? 'dark' : 'light'} theme`}
      ><Icon name={theme === 'light' ? 'moon' : 'sun'} /></button
    >
  </div>
  <SidebarResizeHandle {width} {onResize} {onResizeEnd} />
</aside>
