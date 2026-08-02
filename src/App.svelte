<script lang="ts">
  import { onMount } from 'svelte';
  import LibraryView from './lib/components/LibraryView.svelte';
  import MeetingView from './lib/components/MeetingView.svelte';
  import NewMeetingView from './lib/components/NewMeetingView.svelte';
  import NewProjectView from './lib/components/NewProjectView.svelte';
  import ProjectView from './lib/components/ProjectView.svelte';
  import ProtocolView from './lib/components/ProtocolView.svelte';
  import SettingsView from './lib/components/SettingsView.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import StartView from './lib/components/StartView.svelte';
  import TranscriptView from './lib/components/TranscriptView.svelte';
  import Icon from './lib/components/Icon.svelte';
  import {
    DEFAULT_SIDEBAR_WIDTH,
    SIDEBAR_WIDTH_STORAGE_KEY,
    clampSidebarWidth,
    parseStoredSidebarWidth,
  } from './lib/layout/sidebarSizing';
  import { resolveWindowChrome } from './lib/platform/windowChrome';
  import { FakeWorkflowBridge } from './lib/workflow/fakeBridge';
  import { createNativeWorkspaceStore } from './lib/workflow/workspaceStore';
  import type {
    AppRoute,
    FakeJobOutcome,
    NewMeetingInput,
    NewProjectInput,
    WorkflowSnapshot,
  } from './lib/workflow/types';

  // The shell already depends on the boundary that future Rust-backed adapters will implement.
  const workspaceStore = createNativeWorkspaceStore();
  const bridge = new FakeWorkflowBridge({ workspaceStore });
  let snapshot: WorkflowSnapshot | null = null;
  let route: AppRoute = { name: 'start' };
  let theme: 'light' | 'dark' = 'light';
  let sidebarOpen = false;
  let sidebarWidth = DEFAULT_SIDEBAR_WIDTH;
  let lastHandledJobId: string | null = null;
  let announcement = '';
  let startupError = '';
  let locationRestored = false;

  // Route-derived context keeps project and meeting selection in one predictable place.
  $: meetingId = 'meetingId' in route ? route.meetingId : null;
  $: meeting = meetingId
    ? (snapshot?.meetings.find((candidate) => candidate.id === meetingId) ?? null)
    : null;
  $: currentProjectId = meeting?.projectId ?? ('projectId' in route ? route.projectId : null);
  $: project = currentProjectId
    ? (snapshot?.projects.find((candidate) => candidate.id === currentProjectId) ?? null)
    : null;
  $: protocol = meeting && snapshot ? (snapshot.protocols[meeting.id] ?? null) : null;
  $: protocolStyle =
    snapshot && protocol
      ? (snapshot.styles.find((candidate) => candidate.id === protocol.styleId) ??
        snapshot.styles[0] ??
        null)
      : null;

  onMount(() => {
    // Native overlay spacing belongs only to Tauri on macOS, never to browser previews or other OSes.
    document.documentElement.dataset.windowChrome = resolveWindowChrome(
      navigator.userAgent,
      '__TAURI_INTERNALS__' in window,
    );

    sidebarWidth = parseStoredSidebarWidth(localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY));

    const savedTheme = localStorage.getItem('localog-theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    theme = savedTheme === 'dark' || (!savedTheme && prefersDark) ? 'dark' : 'light';
    applyTheme();

    return bridge.subscribe(
      (nextSnapshot) => {
        snapshot = nextSnapshot;
        if (!locationRestored && nextSnapshot.activeMeetingId && nextSnapshot.activeRoute) {
          route = {
            name: nextSnapshot.activeRoute,
            meetingId: nextSnapshot.activeMeetingId,
          };
          locationRestored = true;
        } else if (!locationRestored) {
          locationRestored = true;
        }
        handleCompletedJob(nextSnapshot);
      },
      (message) => {
        startupError = message;
      },
    );
  });

  function handleCompletedJob(nextSnapshot: WorkflowSnapshot) {
    const job = nextSnapshot.activeJob;
    if (!job || job.state !== 'completed' || job.id === lastHandledJobId) return;
    lastHandledJobId = job.id;
    announcement = job.stage;
    if (job.outcome !== 'succeeded') return;
    // Background completion must not pull someone away from a different meeting or view.
    if (routeMeetingId() !== job.meetingId) return;
    if (job.kind === 'transcription') navigate({ name: 'transcript', meetingId: job.meetingId });
    if (job.kind === 'generation') navigate({ name: 'protocol', meetingId: job.meetingId });
  }

  function routeMeetingId() {
    return 'meetingId' in route ? route.meetingId : null;
  }

  function compactJobStatus(job: NonNullable<WorkflowSnapshot['activeJob']>) {
    if (job.requiresDuplicateConfirmation) return 'Needs your decision';
    if (job.state === 'queued') return 'Ready to continue';
    if (job.state === 'cancelling') return 'Cancelling safely';
    if (job.kind === 'import' && job.totalBytes !== null) {
      return `${formatBytes(job.progressBytes)} of ${formatBytes(job.totalBytes)} copied locally`;
    }
    return `${job.progress}% · running locally`;
  }

  function formatBytes(bytes: number) {
    if (bytes < 1_000_000) return `${Math.round(bytes / 1_000)} KB`;
    return `${(bytes / 1_000_000).toFixed(bytes >= 10_000_000 ? 0 : 1)} MB`;
  }

  function navigate(nextRoute: AppRoute) {
    route = nextRoute;
    if (
      'meetingId' in nextRoute &&
      ['meeting', 'transcript', 'protocol'].includes(nextRoute.name)
    ) {
      void bridge.saveWorkspaceLocation(nextRoute.meetingId, nextRoute.name);
    }
    sidebarOpen = false;
    requestAnimationFrame(() => document.querySelector<HTMLElement>('#main-content h1')?.focus());
  }

  function applyTheme() {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  }

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    localStorage.setItem('localog-theme', theme);
    applyTheme();
  }

  function resizeSidebar(width: number) {
    sidebarWidth = clampSidebarWidth(width);
  }

  function finishSidebarResize(width: number) {
    resizeSidebar(width);
    // Persist once at interaction end instead of writing synchronously for every pointer movement.
    localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, String(sidebarWidth));
  }

  async function createProject(input: NewProjectInput, returnToImport: boolean) {
    const created = await bridge.createProject(input);
    navigate(
      returnToImport
        ? { name: 'new-meeting', projectId: created.id }
        : { name: 'project', projectId: created.id },
    );
  }

  async function createMeeting(input: NewMeetingInput) {
    const created = await bridge.createMeeting(input);
    await bridge.importRecording(created.id);
    navigate({ name: 'meeting', meetingId: created.id });
  }

  function cancelNewProject(returnToImport: boolean) {
    navigate(returnToImport ? { name: 'new-meeting', projectId: null } : { name: 'start' });
  }

  function cancelNewMeeting(projectId: string | null) {
    navigate(projectId ? { name: 'project', projectId } : { name: 'start' });
  }

  function exportProtocol(format: 'markdown' | 'text') {
    if (!meeting || !snapshot) return;
    const protocol = snapshot.protocols[meeting.id];
    if (!protocol) return;
    const content =
      format === 'markdown'
        ? protocol.markdown
        : protocol.markdown.replace(/^#{1,6}\s+/gm, '').replace(/[*_`]/g, '');
    // Phase 0 uses a browser download; durable exports will move behind the Rust file boundary.
    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
    const href = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = href;
    anchor.download = `${meeting.title.toLowerCase().replace(/[^a-z0-9]+/g, '-')}.${format === 'markdown' ? 'md' : 'txt'}`;
    anchor.click();
    URL.revokeObjectURL(href);
    announcement = `${format === 'markdown' ? 'Markdown' : 'Plain text'} export prepared locally`;
  }
</script>

<svelte:head
  ><meta name="theme-color" content={theme === 'light' ? '#f5f2ec' : '#1a1917'} /></svelte:head
>

<a class="skip-link" href="#main-content">Skip to workspace</a>
<div class="app-shell" style={`--sidebar-width: ${sidebarWidth}px`}>
  <div class="window-drag-region" data-tauri-drag-region aria-hidden="true"></div>
  {#if snapshot}
    <Sidebar
      projects={snapshot.projects}
      {route}
      {currentProjectId}
      activeJob={snapshot.activeJob}
      width={sidebarWidth}
      open={sidebarOpen}
      {theme}
      onNavigate={navigate}
      onClose={() => (sidebarOpen = false)}
      onToggleTheme={toggleTheme}
      onResize={resizeSidebar}
      onResizeEnd={finishSidebarResize}
    />

    <div class="app-main">
      <div class="mobile-topbar" data-tauri-drag-region>
        <button
          class="icon-button"
          aria-label="Open navigation"
          onclick={() => (sidebarOpen = true)}><Icon name="menu" /></button
        >
        <button class="mobile-wordmark" onclick={() => navigate({ name: 'start' })}>LocaLog</button>
        <button
          class="icon-button"
          aria-label={`Use ${theme === 'light' ? 'dark' : 'light'} theme`}
          onclick={toggleTheme}><Icon name={theme === 'light' ? 'moon' : 'sun'} /></button
        >
      </div>

      {#if snapshot.activeJob && ['queued', 'running', 'cancelling'].includes(snapshot.activeJob.state) && routeMeetingId() !== snapshot.activeJob.meetingId}
        <button
          class="global-job-strip"
          onclick={() => navigate({ name: 'meeting', meetingId: snapshot!.activeJob!.meetingId })}
          ><span class="processing-dot"></span><span
            ><strong>{snapshot.activeJob.stage}</strong><small
              >{compactJobStatus(snapshot.activeJob)}</small
            ></span
          ><Icon name="arrow" /></button
        >
      {/if}

      {#if route.name === 'start'}
        <StartView onNavigate={navigate} />
      {:else if route.name === 'new-project'}
        <NewProjectView
          returnToImport={route.returnToImport}
          onCancel={() => cancelNewProject(route.name === 'new-project' && route.returnToImport)}
          onCreate={(input) =>
            createProject(input, route.name === 'new-project' && route.returnToImport)}
        />
      {:else if route.name === 'new-meeting'}
        <NewMeetingView
          projects={snapshot.projects}
          initialProjectId={route.projectId}
          styles={snapshot.styles}
          onCancel={() => cancelNewMeeting(route.name === 'new-meeting' ? route.projectId : null)}
          onCreateProject={() => navigate({ name: 'new-project', returnToImport: true })}
          onSelectNativeSource={workspaceStore?.selectMediaSource.bind(workspaceStore)}
          onCreate={createMeeting}
        />
      {:else if route.name === 'project' && project}
        <ProjectView
          {project}
          meetings={snapshot.meetings
            .filter((candidate) => candidate.projectId === project.id)
            .sort((a, b) => b.occurredAt.localeCompare(a.occurredAt))}
          onNavigate={navigate}
        />
      {:else if route.name === 'meeting' && project && meeting}
        <MeetingView
          {project}
          {meeting}
          job={snapshot.jobs.find((job) => job.meetingId === meeting.id) ?? snapshot.activeJob}
          onNavigate={navigate}
          onTranscribe={() => bridge.startTranscription(meeting.id)}
          onCancel={() => bridge.cancelActiveJob(meeting.id)}
          onRetry={() => bridge.retryActiveJob(meeting.id)}
          onConfirmDuplicate={() => bridge.confirmDuplicateImport(meeting.id)}
          onReselectSource={() => bridge.reselectImportSource(meeting.id)}
          onRename={(title) => bridge.updateMeetingTitle(meeting.id, title)}
        />
      {:else if route.name === 'transcript' && project && meeting}
        <TranscriptView
          {project}
          {meeting}
          transcript={snapshot.transcripts[meeting.id] ?? null}
          job={snapshot.jobs.find((job) => job.meetingId === meeting.id) ?? snapshot.activeJob}
          onNavigate={navigate}
          onGenerate={() => bridge.generateProtocol(meeting.id)}
          onCancel={() => bridge.cancelActiveJob(meeting.id)}
          onRetry={() => bridge.retryActiveJob(meeting.id)}
          onUpdateSegment={(segmentId, text) =>
            bridge.updateTranscriptSegment(meeting.id, segmentId, text)}
          onUpdateSpeaker={(speaker, replacement) =>
            bridge.updateSpeaker(meeting.id, speaker, replacement)}
        />
      {:else if route.name === 'protocol' && project && meeting && protocol && protocolStyle}
        <ProtocolView
          {project}
          {meeting}
          {protocol}
          style={protocolStyle}
          onNavigate={navigate}
          onSave={(markdown) => bridge.updateProtocol(meeting.id, markdown)}
          onCreateRevision={() => bridge.createProtocolRevision(meeting.id)}
          onMarkReviewed={() => bridge.markReviewed(meeting.id)}
          onRestoreRevision={(revisionId) => bridge.restoreProtocolRevision(meeting.id, revisionId)}
          onExport={exportProtocol}
        />
      {:else if route.name === 'styles' || route.name === 'vocabulary'}
        <LibraryView
          kind={route.name}
          styles={snapshot.styles}
          vocabulary={snapshot.vocabulary}
          projects={snapshot.projects}
        />
      {:else if route.name === 'settings'}
        <SettingsView
          {theme}
          nextJobOutcome={snapshot.nextJobOutcome}
          onToggleTheme={toggleTheme}
          onSetNextJobOutcome={(outcome: FakeJobOutcome) => bridge.setNextJobOutcome(outcome)}
        />
      {:else}
        <StartView onNavigate={navigate} />
      {/if}
    </div>
  {:else if startupError}
    <main class="startup-failure" id="main-content">
      <p class="eyebrow">Local workspace</p>
      <h1 tabindex="-1">Workspace could not be opened</h1>
      <p>{startupError} Your existing files have not been changed.</p>
      <button class="secondary-action" onclick={() => window.location.reload()}>Try again</button>
    </main>
  {:else}
    <div class="app-loading" aria-live="polite">
      <span class="processing-dot"></span>Preparing local workspace…
    </div>
  {/if}
</div>

<p class="sr-only" aria-live="polite">{announcement}</p>
