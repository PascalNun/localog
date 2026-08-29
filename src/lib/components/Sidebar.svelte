<script lang="ts">
  import type { ActiveJob, AppRoute, ProjectSummary } from '../workflow/types';
  import Icon from './Icon.svelte';
  import SidebarResizeHandle from './SidebarResizeHandle.svelte';
  import { clock } from '../time';
  import { formatBytes } from '../bytes';
  import { t } from '../i18n';

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
    return job.stage.toLowerCase().includes('speaker') ? $t.sidebar.separatingSpeakers : job.stage;
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
        <span id="projects-label">{$t.sidebar.projects}</span>
        <button
          class="icon-button compact"
          aria-label={$t.sidebar.createProject}
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
      title={themeChoice === 'auto' ? $t.sidebar.themeFollowingShort : `Always ${themeChoice}`}
      ><Icon name={THEME_ICON[themeChoice]} /></button
    >
  </div>
  <SidebarResizeHandle {width} {onResize} {onResizeEnd} />
</aside>
