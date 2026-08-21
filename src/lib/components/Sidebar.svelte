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
  /// A recording in progress, wherever somebody happens to be.
  export let recording: { recording: boolean; seconds: number; meetingId: string | null } = {
    recording: false,
    seconds: 0,
    meetingId: null,
  };
  export let recordingMeeting: string | null = null;

  function elapsed(seconds: number) {
    const minutes = Math.floor(seconds / 60);
    const rest = seconds % 60;
    const hours = Math.floor(minutes / 60);
    const pad = (value: number) => String(value).padStart(2, '0');
    return hours > 0 ? `${hours}:${pad(minutes % 60)}:${pad(rest)}` : `${minutes}:${pad(rest)}`;
  }
  export let width: number;
  export let open = false;
  export let onNavigate: (route: AppRoute) => void;
  export let onClose: () => void;
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
        ><Icon name="book" size={17} /> Names &amp; terms</button
      >
      <button
        class:active={route.name === 'export-templates'}
        class="nav-link"
        onclick={() => navigate({ name: 'export-templates' })}
        ><Icon name="download" size={16} /> Export templates</button
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
          ><strong>Recording</strong><small
            >{recordingMeeting ? `${recordingMeeting} · ` : ''}{elapsed(recording.seconds)}</small
          ></span
        >
      </button>
    {/if}
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
      aria-label={themeChoice === 'auto'
        ? 'Following the system theme. Switch to always light.'
        : themeChoice === 'light'
          ? 'Always light. Switch to always dark.'
          : 'Always dark. Switch to following the system.'}
      title={themeChoice === 'auto' ? 'Following the system' : `Always ${themeChoice}`}
      ><Icon name={THEME_ICON[themeChoice]} /></button
    >
  </div>
  <SidebarResizeHandle {width} {onResize} {onResizeEnd} />
</aside>
