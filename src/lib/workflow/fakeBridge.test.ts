import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FakeWorkflowBridge } from './fakeBridge';
import type { ActiveJob, MeetingSummary, ProjectSummary } from './types';
import type { WorkspaceStore } from './workspaceStore';

const emptyWorkspace = {
  projects: [],
  meetings: [],
  jobs: [],
  transcripts: {},
  protocols: {},
  styles: [],
  vocabulary: [],
  activeMeetingId: null,
  activeRoute: null,
};

function mockStore(overrides: Partial<WorkspaceStore> = {}): WorkspaceStore {
  return {
    loadWorkspace: vi.fn<WorkspaceStore['loadWorkspace']>().mockResolvedValue(emptyWorkspace),
    createProject: vi.fn<WorkspaceStore['createProject']>(),
    createMeeting: vi.fn<WorkspaceStore['createMeeting']>(),
    updateMeetingTitle: vi.fn<WorkspaceStore['updateMeetingTitle']>(),
    selectMediaSource: vi.fn<WorkspaceStore['selectMediaSource']>(),
    startImport: vi.fn<WorkspaceStore['startImport']>(),
    cancelImport: vi.fn<WorkspaceStore['cancelImport']>(),
    findNameCandidates: vi.fn<WorkspaceStore['findNameCandidates']>().mockResolvedValue([]),
    previewCorrection: vi.fn<WorkspaceStore['previewCorrection']>().mockResolvedValue([]),
    applyCorrection: vi.fn<WorkspaceStore['applyCorrection']>().mockResolvedValue(emptyWorkspace),
    retryImport: vi.fn<WorkspaceStore['retryImport']>(),
    replaceImportSource: vi.fn<WorkspaceStore['replaceImportSource']>(),
    startTranscription: vi.fn<WorkspaceStore['startTranscription']>(),
    startGeneration: vi.fn<WorkspaceStore['startGeneration']>(),
    cancelProcessing: vi.fn<WorkspaceStore['cancelProcessing']>(),
    retryProcessing: vi.fn<WorkspaceStore['retryProcessing']>(),
    updateTranscriptSegment: vi.fn<WorkspaceStore['updateTranscriptSegment']>(),
    deleteTranscriptSegment: vi.fn<WorkspaceStore['deleteTranscriptSegment']>(),
    renameTranscriptSpeaker: vi.fn<WorkspaceStore['renameTranscriptSpeaker']>(),
    subscribeFileDrops: vi
      .fn<WorkspaceStore['subscribeFileDrops']>()
      .mockReturnValue(() => undefined),
    saveVocabularyEntry: vi.fn<WorkspaceStore['saveVocabularyEntry']>(),
    deleteVocabularyEntry: vi.fn<WorkspaceStore['deleteVocabularyEntry']>(),
    autosaveProtocol: vi.fn<WorkspaceStore['autosaveProtocol']>(),
    createProtocolRevision: vi.fn<WorkspaceStore['createProtocolRevision']>(),
    markProtocolReviewed: vi.fn<WorkspaceStore['markProtocolReviewed']>(),
    restoreProtocolRevision: vi.fn<WorkspaceStore['restoreProtocolRevision']>(),
    saveWorkspaceLocation: vi.fn<WorkspaceStore['saveWorkspaceLocation']>(),
    subscribe: vi.fn<WorkspaceStore['subscribe']>().mockResolvedValue(() => undefined),
    ...overrides,
  };
}

describe('FakeWorkflowBridge', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('keeps stable meeting lifecycle separate from a running job', async () => {
    const bridge = new FakeWorkflowBridge({ tickMs: 10, progressStep: 50 });
    await bridge.startTranscription('meeting-kickoff');

    let snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-kickoff')?.lifecycle).toBe(
      'source_ready',
    );
    expect(snapshot.activeJob?.state).toBe('queued');

    await vi.advanceTimersByTimeAsync(20);
    snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-kickoff')?.lifecycle).toBe(
      'transcript_ready',
    );
    expect(snapshot.activeJob?.outcome).toBe('succeeded');
  });

  it('cancels work without deleting the latest stable state', async () => {
    const bridge = new FakeWorkflowBridge({ tickMs: 10, progressStep: 20 });
    await bridge.startTranscription('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(10);
    await bridge.cancelActiveJob('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(10);

    const snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-kickoff')?.lifecycle).toBe(
      'source_ready',
    );
    expect(snapshot.activeJob?.outcome).toBe('cancelled');
  });

  it('supports a synthetic failure and safe retry', async () => {
    const bridge = new FakeWorkflowBridge({ tickMs: 10, progressStep: 50 });
    await bridge.setNextJobOutcome('failure');
    await bridge.startTranscription('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(10);
    expect((await bridge.getSnapshot()).activeJob?.state).toBe('failed');

    await bridge.retryActiveJob('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(20);
    const snapshot = await bridge.getSnapshot();
    expect(snapshot.activeJob?.outcome).toBe('succeeded');
    expect(snapshot.transcripts['meeting-kickoff']?.segments).toHaveLength(4);
  });

  it('allows the meeting language to be corrected before a fresh transcript revision', async () => {
    const bridge = new FakeWorkflowBridge({ tickMs: 10, progressStep: 100 });
    await bridge.startTranscription('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(20);

    await bridge.updateMeetingLanguage('meeting-kickoff', 'German');
    await bridge.startTranscription('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(20);

    const snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-kickoff')?.language).toBe('German');
    expect(snapshot.transcripts['meeting-kickoff']?.language).toBe('German');
  });

  it('preserves the reviewed revision when working content changes', async () => {
    const bridge = new FakeWorkflowBridge();
    await bridge.markReviewed('meeting-envelope-options');
    await bridge.updateProtocol('meeting-envelope-options', '# Revised protocol');
    const snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-envelope-options')?.lifecycle).toBe(
      'reviewed',
    );
    expect(snapshot.protocols['meeting-envelope-options']?.reviewState).toBe(
      'changed_since_review',
    );
  });

  it('loads and writes the native hierarchy through the workspace store', async () => {
    const persistedProject: ProjectSummary = {
      id: 'project-persisted',
      name: 'Persisted synthetic project',
      description: 'No real project data.',
      meetingCount: 0,
      defaultLanguage: 'English',
      defaultStyleId: 'style-formal',
    };
    const createdProject: ProjectSummary = {
      ...persistedProject,
      id: 'project-created',
      name: 'Created synthetic project',
    };
    const store = mockStore({
      loadWorkspace: vi
        .fn<WorkspaceStore['loadWorkspace']>()
        .mockResolvedValue({ ...emptyWorkspace, projects: [persistedProject] }),
      createProject: vi.fn<WorkspaceStore['createProject']>().mockResolvedValue(createdProject),
      selectMediaSource: vi
        .fn<WorkspaceStore['selectMediaSource']>()
        .mockResolvedValue({ name: 'synthetic-reselected.wav', path: '/synthetic/reselected.wav' }),
      replaceImportSource: vi
        .fn<WorkspaceStore['replaceImportSource']>()
        .mockResolvedValue(undefined),
    });
    const bridge = new FakeWorkflowBridge({ workspaceStore: store });

    expect((await bridge.getSnapshot()).projects).toEqual([persistedProject]);
    await bridge.createProject({
      name: createdProject.name,
      description: createdProject.description,
      defaultLanguage: createdProject.defaultLanguage,
    });

    expect(store.createProject).toHaveBeenCalledOnce();
    expect((await bridge.getSnapshot()).projects).toEqual([persistedProject, createdProject]);
  });

  it('reports a bounded native-workspace startup failure', async () => {
    const store = mockStore({
      loadWorkspace: vi
        .fn<WorkspaceStore['loadWorkspace']>()
        .mockRejectedValue('Storage unavailable'),
    });
    const bridge = new FakeWorkflowBridge({ workspaceStore: store });
    const onError = vi.fn();

    bridge.subscribe(() => undefined, onError);
    await vi.waitFor(() => expect(onError).toHaveBeenCalledWith('Storage unavailable'));
  });

  it('restores an interrupted native import and retries the selected meeting', async () => {
    const project: ProjectSummary = {
      id: 'project-recovery',
      name: 'Synthetic recovery project',
      description: '',
      meetingCount: 1,
      defaultLanguage: 'English',
      defaultStyleId: 'style-formal',
    };
    const meeting: MeetingSummary = {
      id: 'meeting-recovery',
      projectId: project.id,
      title: 'Synthetic interrupted import',
      occurredAt: '2026-08-02',
      durationLabel: null,
      lifecycle: 'draft',
      language: 'English',
      sourceName: 'synthetic-recovery.wav',
      sourceByteCount: null,
      sourceMediaType: null,
      styleId: 'style-formal',
    };
    const job: ActiveJob = {
      id: 'job-recovery',
      meetingId: meeting.id,
      kind: 'import',
      state: 'interrupted',
      outcome: null,
      progress: 42,
      progressBytes: 420,
      totalBytes: 1_000,
      stage: 'Import was interrupted — original unchanged',
      attempt: 1,
      error: {
        code: 'interrupted',
        title: 'Import was interrupted',
        detail: 'The meeting remains in Draft.',
      },
      requiresDuplicateConfirmation: false,
    };
    const store = mockStore({
      loadWorkspace: vi.fn<WorkspaceStore['loadWorkspace']>().mockResolvedValue({
        ...emptyWorkspace,
        projects: [project],
        meetings: [meeting],
        jobs: [job],
      }),
      selectMediaSource: vi
        .fn<WorkspaceStore['selectMediaSource']>()
        .mockResolvedValue({ name: 'synthetic-reselected.wav', path: '/synthetic/reselected.wav' }),
      retryImport: vi.fn<WorkspaceStore['retryImport']>().mockResolvedValue(undefined),
      replaceImportSource: vi
        .fn<WorkspaceStore['replaceImportSource']>()
        .mockResolvedValue(undefined),
    });
    const bridge = new FakeWorkflowBridge({ workspaceStore: store });

    const snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings[0]?.lifecycle).toBe('draft');
    expect(snapshot.activeJob).toEqual(job);

    await bridge.retryActiveJob(meeting.id);
    expect(store.retryImport).toHaveBeenCalledWith(meeting.id, false);

    await bridge.reselectImportSource(meeting.id);
    expect(store.replaceImportSource).toHaveBeenCalledWith(meeting.id, {
      name: 'synthetic-reselected.wav',
      path: '/synthetic/reselected.wav',
    });
  });
});
