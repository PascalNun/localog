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
  import RecordingReviewView from './lib/components/RecordingReviewView.svelte';
  import RecordingView from './lib/components/RecordingView.svelte';
  import TranscriptView from './lib/components/TranscriptView.svelte';
  import Icon from './lib/components/Icon.svelte';
  import { DEFAULT_APPEARANCE, EMPTY_FURNITURE } from './lib/workflow/types';
  import { buildDocx } from './lib/protocol/docx';
  import { printProtocol } from './lib/protocol/print';
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
    MeetingSummary,
    ProjectSummary,
    ProtocolDensity,
    ProtocolDraft,
    RecordingStatus,
    ExportTemplate,
    SetAsideSection,
    NewMeetingInput,
    NewProjectInput,
    ProtocolProviderStatus,
    RecordingReview,
    SpeakerSeparationStatus,
    WorkflowSnapshot,
    TranscriptionCapability,
    TranscriptionPreset,
    TranscriptionRuntimeStatus,
    AppliedCorrection,
  } from './lib/workflow/types';

  /// Whether this is the desktop shell rather than a browser preview. A browser can
  /// fall back to a download; the desktop shell blocks it, so a failure there has to
  /// be said out loud rather than quietly retried.
  const isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  // The shell already depends on the boundary that future Rust-backed adapters will implement.
  const workspaceStore = createNativeWorkspaceStore();
  const bridge = new FakeWorkflowBridge({ workspaceStore });
  let snapshot: WorkflowSnapshot | null = null;
  let route: AppRoute = { name: 'start' };
  /// What the person chose, which is not the same as what is on screen.
  ///
  /// Most systems switch themselves between light and dark during the day, and an
  /// application that picked one at launch and kept it is out of step by the evening.
  /// So "auto" is a real choice and the default, and following the system means
  /// listening rather than reading once.
  let themeChoice: 'auto' | 'light' | 'dark' = 'auto';

  /// Whether anything is being recorded, known everywhere rather than only on the
  /// screen that started it.
  ///
  /// The written direction is plain about this: a recording in progress must be
  /// unmistakable at a glance, because hiding it would be dishonest. It was visible
  /// only on the recording screen, so navigating away — to another project, to
  /// settings — left a live recorder with nothing anywhere saying so.
  ///
  /// One poller for the whole application rather than one per screen. It costs
  /// nothing on the other side: the command reads a number the reader thread has
  /// already put down and asks the operating system nothing. It runs every second
  /// while something is being recorded and every ten otherwise, which is often
  /// enough to notice a recorder that has died on its own.
  let recording: RecordingStatus = {
    available: false,
    recording: false,
    meetingId: null,
    seconds: 0,
    systemPeak: 0,
    microphonePeak: 0,
    stoppedUnexpectedly: false,
  };
  let recordingTimer: ReturnType<typeof setTimeout> | null = null;

  async function pollRecording() {
    // Any pending tick is dropped first: this is called directly after starting and
    // stopping as well as on its own schedule, and two timers would double the rate
    // every time somebody pressed record.
    if (recordingTimer) clearTimeout(recordingTimer);
    try {
      recording = await bridge.recordingStatus();
    } catch {
      // A status that cannot be read is not a reason to stop asking.
    }
    recordingTimer = setTimeout(() => void pollRecording(), recording.recording ? 1000 : 10_000);
  }

  onMount(() => {
    void pollRecording();
    return () => {
      if (recordingTimer) clearTimeout(recordingTimer);
    };
  });

  $: recordingMeeting = recording.recording
    ? (snapshot?.meetings.find((meeting) => meeting.id === recording.meetingId) ?? null)
    : null;

  /// Saved ways of presenting a protocol, read once and kept as they change.
  let exportTemplates: ExportTemplate[] = [];
  let templatesRead = false;
  $: if (!templatesRead && (route.name === 'export-templates' || route.name === 'protocol')) {
    templatesRead = true;
    void bridge
      .exportTemplates()
      .then((found) => {
        exportTemplates = found;
      })
      .catch(() => {
        exportTemplates = [];
      });
  }

  /// Sections taken out of the open protocol and kept in case they are wanted back.
  /// Read when a protocol is opened rather than held in the workspace snapshot,
  /// because it belongs to one meeting's working draft and nothing else reads it.
  let protocolSetAside: SetAsideSection[] = [];
  let setAsideFor = '';
  $: if (route.name === 'protocol' && route.meetingId !== setAsideFor) {
    setAsideFor = route.meetingId;
    protocolSetAside = [];
    void bridge
      .protocolSetAside(route.meetingId)
      .then((held) => {
        protocolSetAside = held;
      })
      .catch(() => {
        protocolSetAside = [];
      });
  }
  let theme: 'light' | 'dark' = 'light';
  let sidebarOpen = false;
  let sidebarWidth = DEFAULT_SIDEBAR_WIDTH;
  let lastHandledJobId: string | null = null;
  let announcement = '';
  let startupError = '';
  let locationRestored = false;
  let runtimeStatus: TranscriptionRuntimeStatus = {
    executablePath: null,
    modelPath: null,
    executableFound: false,
    modelFound: false,
    runtimeVersion: null,
    modelDigest: null,
    modelByteCount: null,
  };
  let runtimeError: string | null = null;
  let speakerStatus: SpeakerSeparationStatus = {
    modelsInstalled: false,
    runtimeConfigured: false,
    runtimeHealthy: false,
    runtimeVersion: null,
    runtimePath: null,
    downloadBytes: 0,
  };
  let speakerError: string | null = null;
  let capability: TranscriptionCapability = { selectedPreset: 'balanced', presets: [] };
  // modelId → percent, present only while a download is in flight.
  let downloading: Record<string, number> = {};
  let modelError: string | null = null;
  let providerError: string | null = null;

  const presetNames: Record<TranscriptionPreset, string> = {
    fast: 'Fast',
    balanced: 'Balanced',
    accurate: 'Accurate',
  };

  function presetDisplayName(preset: TranscriptionPreset) {
    return presetNames[preset] ?? 'Not selected';
  }

  function withoutModel(entries: Record<string, number>, modelId: string) {
    const next = { ...entries };
    delete next[modelId];
    return next;
  }
  let providerStatus: ProtocolProviderStatus = {
    endpoint: 'http://127.0.0.1:11434',
    serverReachable: false,
    runtimeVersion: null,
    models: [],
    selectedModel: null,
    selectedModelDigest: null,
    selectedModelReady: false,
    message: 'Ollama has not been checked yet.',
    machineMemoryGb: null,
  };

  // Route-derived context keeps project and meeting selection in one predictable place.
  $: meetingId = 'meetingId' in route ? route.meetingId : null;
  $: meeting = meetingId
    ? (snapshot?.meetings.find((candidate) => candidate.id === meetingId) ?? null)
    : null;
  // The sidebar names the meeting the work belongs to, which is the one thing the
  // panel inside that meeting never has to say.
  $: activeJobMeeting =
    snapshot?.meetings.find((entry) => entry.id === snapshot?.activeJob?.meetingId)?.title ?? null;

  $: currentProjectId = meeting?.projectId ?? ('projectId' in route ? route.projectId : null);
  $: project = currentProjectId
    ? (snapshot?.projects.find((candidate) => candidate.id === currentProjectId) ?? null)
    : null;
  $: protocol = meeting && snapshot ? (snapshot.protocols[meeting.id] ?? null) : null;
  // Resolve from the meeting's own style so transcript review can show it before generation.
  $: protocolStyle = snapshot
    ? (snapshot.styles.find(
        (candidate) => candidate.id === (protocol?.styleId ?? meeting?.styleId),
      ) ?? null)
    : null;

  // A recording dropped on the window is an import wherever the person happens to
  // be looking. Held here rather than in the import step because the commonest
  // moment to drop one is before that step exists — on the start screen, with
  // nothing open yet.
  onMount(() => {
    return bridge.subscribeFileDrops((event) => {
      if (event.kind === 'over') {
        draggingFile = true;
      } else if (event.kind === 'leave') {
        draggingFile = false;
      } else {
        draggingFile = false;
        const usable = event.paths.find((path) => isRecording(path));
        if (!usable) {
          droppedRefusal = event.paths[0]
            ? `LocaLog cannot read a ${extensionOf(event.paths[0]).toUpperCase()} file. Drop an audio or video recording.`
            : '';
          return;
        }
        droppedRefusal = '';
        droppedRecording = usable;
        if (route.name !== 'new-meeting') {
          navigate({ name: 'new-meeting', projectId: currentProjectId });
        }
      }
    });
  });

  onMount(() => {
    // Native overlay spacing belongs only to Tauri on macOS, never to browser previews or other OSes.
    document.documentElement.dataset.windowChrome = resolveWindowChrome(
      navigator.userAgent,
      '__TAURI_INTERNALS__' in window,
    );

    sidebarWidth = parseStoredSidebarWidth(localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY));

    const saved = localStorage.getItem('localog-theme');
    themeChoice = saved === 'light' || saved === 'dark' ? saved : 'auto';
    const system = window.matchMedia('(prefers-color-scheme: dark)');
    const settle = () => {
      theme = themeChoice === 'auto' ? (system.matches ? 'dark' : 'light') : themeChoice;
      applyTheme();
    };
    settle();
    // Following the system means following it, not sampling it once at launch.
    system.addEventListener('change', settle);
    followSystem = settle;
    bridge.getTranscriptionRuntimeStatus().then((status) => (runtimeStatus = status));
    bridge.getSpeakerSeparationStatus().then((status) => (speakerStatus = status));
    bridge.getProtocolProviderStatus().then((status) => (providerStatus = status));
    bridge
      .getTranscriptionCapability()
      .then((next) => (capability = next))
      .catch((error) => (modelError = error instanceof Error ? error.message : String(error)));

    const stopModelEvents = bridge.subscribeModelEvents({
      onProgress: ({ modelId, percent }) => {
        downloading = { ...downloading, [modelId]: percent };
      },
      onChanged: (next) => {
        capability = next;
        // Only clear rows that are now installed; another download may still be running.
        downloading = Object.fromEntries(
          Object.entries(downloading).filter(
            ([id]) => !next.presets.some((preset) => preset.modelId === id && preset.installed),
          ),
        );
        // A newly installed model can make transcription possible.
        bridge.getTranscriptionRuntimeStatus().then((status) => (runtimeStatus = status));
      },
      onError: ({ modelId, message }) => {
        downloading = withoutModel(downloading, modelId);
        modelError = message;
      },
    });
    const stopSpeakerEvents = bridge.subscribeSpeakerEvents((status) => {
      speakerStatus = status;
      downloading = withoutModel(downloading, 'speaker-separation');
    });

    const stopWorkspace = bridge.subscribe(
      (nextSnapshot) => {
        snapshot = nextSnapshot;
        if (!locationRestored && nextSnapshot.activeMeetingId && nextSnapshot.activeRoute) {
          const restoredMeeting = nextSnapshot.meetings.find(
            (candidate) => candidate.id === nextSnapshot.activeMeetingId,
          );
          // A fresh transcript invalidates the old protocol as the current stage.
          // Reopen review rather than showing a document generated from stale text.
          const restoredRoute =
            nextSnapshot.activeRoute === 'protocol' &&
            !['protocol_draft', 'reviewed'].includes(restoredMeeting?.lifecycle ?? '')
              ? 'transcript'
              : nextSnapshot.activeRoute;
          route = {
            name: restoredRoute,
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

    return () => {
      stopModelEvents();
      stopSpeakerEvents();
      stopWorkspace();
    };
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
      return `${formatBytes(job.progressBytes)} of ${formatBytes(job.totalBytes)} copied`;
    }
    return `${job.progress}%`;
  }

  function formatBytes(bytes: number) {
    if (bytes < 1_000_000) return `${Math.round(bytes / 1_000)} KB`;
    return `${(bytes / 1_000_000).toFixed(bytes >= 10_000_000 ? 0 : 1)} MB`;
  }

  // What the import stage reads. Named rather than complete: a person can check a
  // list, and anything not on it is refused by name instead of ignored.
  const RECORDING_TYPES = [
    'mp3',
    'm4a',
    'wav',
    'aac',
    'flac',
    'ogg',
    'opus',
    'wma',
    'aiff',
    'aif',
    'mp4',
    'mov',
    'm4v',
    'mkv',
    'avi',
    'webm',
  ];
  let draggingFile = false;
  let droppedRecording: string | null = null;
  let droppedRefusal = '';

  function extensionOf(path: string): string {
    return path.split('.').pop()?.toLowerCase() ?? '';
  }
  function isRecording(path: string): boolean {
    return RECORDING_TYPES.includes(extensionOf(path));
  }

  let recordingReview: RecordingReview | null = null;
  let reviewedMeeting = '';
  // Fetched when the screen is opened rather than kept in the workspace snapshot:
  // a waveform is a few thousand numbers and only this screen wants them.
  $: if (route.name === 'recording-review' && route.meetingId !== reviewedMeeting) {
    reviewedMeeting = route.meetingId;
    recordingReview = null;
    const wanted = route.meetingId;
    void bridge.getRecordingReview(wanted).then((found) => {
      // Not a loop: this block's condition reads `route` and `reviewedMeeting`, and
      // `reviewedMeeting` was set above, so assigning the review cannot retrigger it.
      // The guard is for a second meeting opened while the first was still loading.
      // eslint-disable-next-line svelte/infinite-reactive-loop
      if (reviewedMeeting === wanted) recordingReview = found;
    });
  }

  function navigate(nextRoute: AppRoute) {
    route = nextRoute;
    // Reviewing a recording is a step somebody is in the middle of, not a place
    // to be returned to on the next launch: they would land in an editor for a
    // meeting they had finished editing.
    if (
      nextRoute.name === 'meeting' ||
      nextRoute.name === 'transcript' ||
      nextRoute.name === 'protocol'
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

  /// Re-settles the theme when the system changes, while the choice is auto.
  let followSystem: (() => void) | null = null;

  /// The sidebar's one-key cycle through the same three states, which names the one
  /// it is in rather than only the one it would move to.
  function toggleTheme() {
    chooseTheme(themeChoice === 'auto' ? 'light' : themeChoice === 'light' ? 'dark' : 'auto');
  }

  /// Choose the theme, including letting the system choose it.
  ///
  /// This was a button that cycled through the three, labelled only with the one it
  /// would move to next — so somebody on automatic was never told they were on it,
  /// and clicking twice to get back to it looked like the control was stuck. Three
  /// named states, one of them showing as chosen, says what the application is doing.
  function chooseTheme(choice: 'auto' | 'light' | 'dark') {
    themeChoice = choice;
    if (choice === 'auto') {
      localStorage.removeItem('localog-theme');
      followSystem?.();
      return;
    }
    theme = choice;
    localStorage.setItem('localog-theme', choice);
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

  async function createMeeting(input: NewMeetingInput, forRecording = false) {
    const created = await bridge.createMeeting(input);
    if (forRecording) {
      // Nothing to import: the recorder writes the source. Straight to it, because
      // somebody who chose "Record a meeting" is standing in front of the meeting.
      navigate({ name: 'recording', meetingId: created.id });
      return;
    }
    await bridge.importRecording(created.id);
    navigate({ name: 'meeting', meetingId: created.id });
  }

  function cancelNewProject(returnToImport: boolean) {
    navigate(returnToImport ? { name: 'new-meeting', projectId: null } : { name: 'start' });
  }

  function cancelNewMeeting(projectId: string | null) {
    navigate(projectId ? { name: 'project', projectId } : { name: 'start' });
  }

  /// What a header or footer field can be filled in from.
  ///
  /// Read at the moment of export rather than stored, because a protocol marked
  /// reviewed after the header was set should say so on the page.
  function documentFacts(
    project: ProjectSummary | undefined,
    forMeeting: MeetingSummary,
    protocol: ProtocolDraft,
  ) {
    return {
      projectName: project?.name ?? '',
      meetingTitle: forMeeting.title,
      meetingDate: forMeeting.occurredAt,
      documentType: 'Protocol',
      protocolStatus:
        protocol.reviewState === 'reviewed'
          ? 'Reviewed'
          : protocol.reviewState === 'changed_since_review'
            ? 'Changed since review'
            : 'Draft',
    };
  }

  async function exportProtocol(format: 'pdf' | 'docx' | 'markdown' | 'text') {
    if (!meeting || !snapshot) return;
    const protocol = snapshot.protocols[meeting.id];
    if (!protocol) return;

    // The PDF is the document printed, not a second rendering of it, so it needs
    // no file dialog of its own: the print dialog is where a page size and a
    // destination are chosen, and on macOS that includes saving as PDF.
    if (format === 'pdf') {
      const project = snapshot.projects.find((candidate) => candidate.id === meeting.projectId);
      try {
        await printProtocol(
          {
            title: meeting.title,
            subtitle: [project?.name, meeting.occurredAt].filter(Boolean).join(' · '),
            markdown: protocol.markdown,
            appearance: project?.appearance ?? DEFAULT_APPEARANCE,
            furniture: project?.furniture ?? EMPTY_FURNITURE,
            facts: documentFacts(project, meeting, protocol),
          },
          bridge.nativePrint(),
        );
      } catch (cause) {
        announcement = `PDF export failed: ${cause instanceof Error ? cause.message : String(cause)}`;
      }
      return;
    }
    // Word is built here too, from the same blocks the screen and the PDF use,
    // and only the writing of it happens in Rust.
    if (format === 'docx') {
      const project = snapshot.projects.find((candidate) => candidate.id === meeting.projectId);
      try {
        const saved = await bridge.exportProtocolBytes(
          buildDocx({
            title: meeting.title,
            subtitle: [project?.name, meeting.occurredAt].filter(Boolean).join(' · '),
            markdown: protocol.markdown,
            appearance: project?.appearance ?? DEFAULT_APPEARANCE,
            furniture: project?.furniture ?? EMPTY_FURNITURE,
            facts: documentFacts(project, meeting, protocol),
          }),
          meeting.title,
          'docx',
          'Word document',
        );
        if (saved) announcement = 'Word export saved';
        else if (!isDesktop) announcement = 'Word export needs the desktop application.';
      } catch (cause) {
        announcement = `Word export failed: ${cause instanceof Error ? cause.message : String(cause)}`;
      }
      return;
    }

    const name = format === 'markdown' ? 'Markdown' : 'Plain text';
    // Saying what went wrong rather than falling quietly through to a browser
    // download the desktop shell blocks. Exporting was silently doing nothing for
    // exactly that reason: the save dialog needed a permission it had not been
    // granted, the call threw, and the fallback could not run either.
    try {
      const nativeExported = await bridge.exportProtocol(meeting.id, format, meeting.title);
      if (nativeExported) {
        announcement = `${name} export saved`;
        return;
      }
      // A cancelled dialog is a choice, not a failure, and says nothing.
      if (isDesktop) return;
    } catch (cause) {
      announcement = `${name} export failed: ${cause instanceof Error ? cause.message : String(cause)}`;
      return;
    }
    const content =
      format === 'markdown'
        ? protocol.markdown
        : protocol.markdown.replace(/^#{1,6}\s+/gm, '').replace(/[*_`]/g, '');
    // Browser development keeps a download fallback; Tauri uses the native verified file boundary.
    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
    const href = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = href;
    anchor.download = `${meeting.title.toLowerCase().replace(/[^a-z0-9]+/g, '-')}.${format === 'markdown' ? 'md' : 'txt'}`;
    anchor.click();
    URL.revokeObjectURL(href);
    announcement = `${format === 'markdown' ? 'Markdown' : 'Plain text'} export prepared`;
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
      {activeJobMeeting}
      {recording}
      recordingMeeting={recordingMeeting?.title ?? null}
      width={sidebarWidth}
      open={sidebarOpen}
      {themeChoice}
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
          aria-label={themeChoice === 'auto'
            ? 'Following the system theme. Switch to always light.'
            : themeChoice === 'light'
              ? 'Always light. Switch to always dark.'
              : 'Always dark. Switch to following the system.'}
          title={themeChoice === 'auto' ? 'Following the system' : `Always ${themeChoice}`}
          onclick={toggleTheme}
          ><Icon
            name={themeChoice === 'auto' ? 'monitor' : themeChoice === 'light' ? 'sun' : 'moon'}
          /></button
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
          forRecording={route.name === 'new-meeting' && route.forRecording === true}
          projects={snapshot.projects}
          initialProjectId={route.projectId}
          styles={snapshot.styles}
          onCancel={() => cancelNewMeeting(route.name === 'new-meeting' ? route.projectId : null)}
          onCreateProject={() => navigate({ name: 'new-project', returnToImport: true })}
          onSelectNativeSource={workspaceStore?.selectMediaSource.bind(workspaceStore)}
          onCreate={(input: NewMeetingInput) =>
            createMeeting(input, route.name === 'new-meeting' && route.forRecording === true)}
          {draggingFile}
          {droppedRecording}
          {droppedRefusal}
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
          presetLabel={presetDisplayName(capability.selectedPreset)}
          job={snapshot.jobs.find((job) => job.meetingId === meeting.id) ?? snapshot.activeJob}
          onNavigate={navigate}
          onTranscribe={(speakers) => bridge.startTranscription(meeting.id, speakers)}
          {speakerStatus}
          speakerPreparing={downloading['speaker-separation'] !== undefined}
          speakerDownloadPercent={downloading['speaker-separation'] ?? 0}
          onPrepareSpeakerModels={async () => {
            speakerError = null;
            downloading = { ...downloading, 'speaker-separation': 0 };
            try {
              await bridge.downloadSpeakerModels();
            } catch (error) {
              downloading = withoutModel(downloading, 'speaker-separation');
              speakerError = error instanceof Error ? error.message : String(error);
            }
          }}
          onCancel={() => bridge.cancelActiveJob(meeting.id)}
          onRetry={() => bridge.retryActiveJob(meeting.id)}
          onUpdateLanguage={(language) => bridge.updateMeetingLanguage(meeting.id, language)}
          onConfirmDuplicate={() => bridge.confirmDuplicateImport(meeting.id)}
          onReselectSource={() => bridge.reselectImportSource(meeting.id)}
          onRename={(title) => bridge.updateMeetingTitle(meeting.id, title)}
        />
      {:else if route.name === 'recording' && meeting}
        <RecordingView
          {meeting}
          onNavigate={navigate}
          onStatus={() => bridge.recordingStatus()}
          onStart={async (meetingId: string) => {
            await bridge.startRecording(meetingId);
            // Asked for at once rather than at the next idle tick: a recording that
            // takes ten seconds to admit it is running is its own small dishonesty.
            await pollRecording();
          }}
          onStop={async () => {
            await bridge.stopRecording();
            await pollRecording();
          }}
        />
      {:else if route.name === 'recording-review' && meeting}
        <RecordingReviewView
          {meeting}
          review={recordingReview}
          onNavigate={navigate}
          onSave={(edits) => bridge.setRecordingEdits(meeting.id, edits)}
          onContinue={() => navigate({ name: 'meeting', meetingId: meeting.id })}
        />
      {:else if route.name === 'transcript' && project && meeting}
        <TranscriptView
          {protocolStyle}
          onLoadAudio={(meetingId: string) => bridge.getMeetingAudio(meetingId)}
          onFindIntroductions={(meetingId: string) => bridge.findIntroductions(meetingId)}
          projectHasNames={Boolean(
            snapshot?.vocabulary.some((entry) => entry.projectId === project?.id),
          )}
          onFindNameCandidates={(meetingId: string) => bridge.findNameCandidates(meetingId)}
          onPreviewCorrection={(meetingId: string, wrong: string, right: string) =>
            bridge.previewCorrection(meetingId, wrong, right)}
          onApplyCorrection={(meetingId: string, correction: AppliedCorrection) =>
            bridge.applyCorrection(meetingId, correction)}
          {project}
          {meeting}
          transcript={snapshot.transcripts[meeting.id] ?? null}
          job={snapshot.jobs.find((job) => job.meetingId === meeting.id) ?? snapshot.activeJob}
          onNavigate={navigate}
          onGenerate={() => bridge.generateProtocol(meeting.id)}
          onCancel={() => bridge.cancelActiveJob(meeting.id)}
          onRetry={() => bridge.retryActiveJob(meeting.id)}
          onRerunTranscription={() => bridge.startTranscription(meeting.id)}
          onUpdateLanguage={(language) => bridge.updateMeetingLanguage(meeting.id, language)}
          onDeleteSegment={(segmentId) => bridge.deleteTranscriptSegment(meeting.id, segmentId)}
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
          onSetAppearance={(appearance) => bridge.setProjectAppearance(project.id, appearance)}
          onSetFurniture={(furniture) => bridge.setProjectFurniture(project.id, furniture)}
          onPreviewReplacement={(text: string, wrong: string, right: string) =>
            bridge.previewNameReplacement(text, wrong, right)}
          templates={exportTemplates}
          onApplyTemplate={(templateId: string) =>
            bridge.applyExportTemplate(project.id, templateId)}
          onSaveTemplate={async (name: string) => {
            exportTemplates = await bridge.saveExportTemplate(
              name,
              `Saved from ${project.name}.`,
              project.appearance,
              project.furniture,
            );
          }}
          setAside={protocolSetAside}
          onSectionsChanged={async (markdown: string, stash: SetAsideSection[]) => {
            await bridge.setProtocolSections(meeting.id, markdown, stash);
            protocolSetAside = stash;
          }}
          onRefine={(passage: string, instruction: string) =>
            bridge.refinePassage(meeting.id, passage, instruction)}
        />
      {:else if route.name === 'styles' || route.name === 'vocabulary' || route.name === 'export-templates'}
        <LibraryView
          kind={route.name}
          styles={snapshot.styles}
          vocabulary={snapshot.vocabulary}
          projects={snapshot.projects}
          onOpenStyle={(styleId: string) => bridge.protocolStyleDetail(styleId)}
          onDuplicateStyle={(styleId: string, name: string) =>
            bridge.duplicateProtocolStyle(styleId, name)}
          onUpdateStyle={(
            styleId: string,
            name: string,
            description: string,
            instructions: string[],
            density: ProtocolDensity,
          ) => bridge.updateProtocolStyle(styleId, name, description, instructions, density)}
          onDeleteStyle={(styleId: string) => bridge.deleteProtocolStyle(styleId)}
          templates={exportTemplates}
          onDeleteTemplate={async (templateId: string) => {
            exportTemplates = await bridge.deleteExportTemplate(templateId);
          }}
          onSaveTerm={(entry) => bridge.saveVocabularyEntry(entry)}
          onDeleteTerm={(entryId) => bridge.deleteVocabularyEntry(entryId)}
        />
      {:else if route.name === 'settings'}
        <SettingsView
          {theme}
          {themeChoice}
          {runtimeStatus}
          {runtimeError}
          {speakerStatus}
          {speakerError}
          {providerStatus}
          {capability}
          {downloading}
          {modelError}
          {providerError}
          nextJobOutcome={snapshot.nextJobOutcome}
          onChooseTheme={chooseTheme}
          onSetNextJobOutcome={(outcome: FakeJobOutcome) => bridge.setNextJobOutcome(outcome)}
          onSelectPreset={async (preset: TranscriptionPreset) => {
            try {
              capability = await bridge.setTranscriptionPreset(preset);
              modelError = null;
              runtimeStatus = await bridge.getTranscriptionRuntimeStatus();
            } catch (error) {
              modelError = error instanceof Error ? error.message : String(error);
            }
          }}
          onDownloadModel={async (modelId: string) => {
            modelError = null;
            // Show the download as started before the first progress event arrives.
            downloading = { ...downloading, [modelId]: 0 };
            try {
              await bridge.downloadTranscriptionModel(modelId);
            } catch (error) {
              downloading = withoutModel(downloading, modelId);
              modelError = error instanceof Error ? error.message : String(error);
            }
          }}
          onCancelDownload={async (modelId: string) => {
            await bridge.cancelTranscriptionDownload(modelId);
            downloading = withoutModel(downloading, modelId);
          }}
          onRemoveModel={async (modelId: string) => {
            try {
              capability = await bridge.removeTranscriptionModel(modelId);
              modelError = null;
              runtimeStatus = await bridge.getTranscriptionRuntimeStatus();
            } catch (error) {
              modelError = error instanceof Error ? error.message : String(error);
            }
          }}
          onConfigureRuntime={async (executablePath) => {
            try {
              runtimeStatus = await bridge.configureTranscriptionRuntime(executablePath);
              runtimeError = null;
            } catch (error) {
              runtimeError = error instanceof Error ? error.message : String(error);
            }
          }}
          onRefreshSpeaker={async () => {
            try {
              speakerStatus = await bridge.getSpeakerSeparationStatus();
              speakerError = null;
            } catch (error) {
              speakerError = error instanceof Error ? error.message : String(error);
            }
          }}
          onDownloadSpeaker={async () => {
            speakerError = null;
            downloading = { ...downloading, 'speaker-separation': 0 };
            try {
              await bridge.downloadSpeakerModels();
              // The browser preview has no native downloader; complete the fake path
              // immediately so its progress state cannot remain stuck at 0%.
              if (!workspaceStore) {
                speakerStatus = {
                  ...speakerStatus,
                  modelsInstalled: true,
                  runtimeConfigured: true,
                  runtimeHealthy: true,
                  runtimeVersion: 'Synthetic runtime',
                  runtimePath: speakerStatus.runtimePath,
                };
                downloading = withoutModel(downloading, 'speaker-separation');
              }
            } catch (error) {
              downloading = withoutModel(downloading, 'speaker-separation');
              speakerError = error instanceof Error ? error.message : String(error);
            }
          }}
          onRefreshProvider={async () => {
            try {
              providerStatus = await bridge.getProtocolProviderStatus();
              providerError = null;
            } catch (error) {
              providerError = error instanceof Error ? error.message : String(error);
            }
          }}
          onConfigureProvider={async (model) => {
            try {
              providerStatus = await bridge.configureProtocolProvider(model);
              providerError = null;
            } catch (error) {
              providerError = error instanceof Error ? error.message : String(error);
            }
          }}
        />
      {:else}
        <StartView onNavigate={navigate} />
      {/if}
    </div>
  {:else if startupError}
    <main class="startup-failure" id="main-content">
      <p class="eyebrow">Workspace</p>
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
