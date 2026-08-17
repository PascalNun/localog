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
    ProtocolProviderStatus,
    RecordingReview,
    SpeakerSeparationStatus,
    WorkflowSnapshot,
    TranscriptionCapability,
    TranscriptionPreset,
    TranscriptionRuntimeStatus,
    AppliedCorrection,
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

    const savedTheme = localStorage.getItem('localog-theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    theme = savedTheme === 'dark' || (!savedTheme && prefersDark) ? 'dark' : 'light';
    applyTheme();
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

  async function exportProtocol(format: 'markdown' | 'text') {
    if (!meeting || !snapshot) return;
    const protocol = snapshot.protocols[meeting.id];
    if (!protocol) return;
    const nativeExported = await bridge.exportProtocol(meeting.id, format, meeting.title);
    if (nativeExported) {
      announcement = `${format === 'markdown' ? 'Markdown' : 'Plain text'} export saved`;
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
          onStart={(meetingId: string) => bridge.startRecording(meetingId)}
          onStop={() => bridge.stopRecording()}
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
        />
      {:else if route.name === 'styles' || route.name === 'vocabulary'}
        <LibraryView
          kind={route.name}
          styles={snapshot.styles}
          vocabulary={snapshot.vocabulary}
          projects={snapshot.projects}
          onSaveTerm={(entry) => bridge.saveVocabularyEntry(entry)}
          onDeleteTerm={(entryId) => bridge.deleteVocabularyEntry(entryId)}
        />
      {:else if route.name === 'settings'}
        <SettingsView
          {theme}
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
          onToggleTheme={toggleTheme}
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
