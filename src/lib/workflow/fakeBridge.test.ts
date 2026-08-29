import { DEFAULT_APPEARANCE, EMPTY_FURNITURE } from './types';
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

/**
 * A stand-in for the real store, carrying only the methods a test asks about.
 *
 * Every method on WorkspaceStore is required, and deliberately so: a method left
 * off the real store is not a compile error but a feature that quietly answers
 * from the demo data instead of the database. Nine of them did, for a while. A
 * test double is the one place a partial store is honest, so it says so with a
 * cast here rather than by making the interface optional for everybody.
 */
function mockStore(overrides: Partial<WorkspaceStore> = {}): WorkspaceStore {
  return {
    loadWorkspace: vi.fn<WorkspaceStore['loadWorkspace']>().mockResolvedValue(emptyWorkspace),
    createProject: vi.fn<WorkspaceStore['createProject']>(),
    createMeeting: vi.fn<WorkspaceStore['createMeeting']>(),
    updateMeetingTitle: vi.fn<WorkspaceStore['updateMeetingTitle']>(),
    selectMediaSource: vi.fn<WorkspaceStore['selectMediaSource']>(),
    startImport: vi.fn<WorkspaceStore['startImport']>(),
    cancelImport: vi.fn<WorkspaceStore['cancelImport']>(),
    protocolStyleDetail: vi.fn<WorkspaceStore['protocolStyleDetail']>(),
    recordingStatus: vi.fn<WorkspaceStore['recordingStatus']>().mockResolvedValue({
      available: false,
      recording: false,
      meetingId: null,
      seconds: 0,
      systemPeak: 0,
      microphonePeak: 0,
      stoppedUnexpectedly: false,
      notes: [],
    }),
    startRecording: vi.fn<WorkspaceStore['startRecording']>(),
    stopRecording: vi.fn<WorkspaceStore['stopRecording']>().mockResolvedValue(emptyWorkspace),
    findIntroductions: vi.fn<WorkspaceStore['findIntroductions']>().mockResolvedValue([]),
    findNameCandidates: vi.fn<WorkspaceStore['findNameCandidates']>().mockResolvedValue([]),
    previewCorrection: vi.fn<WorkspaceStore['previewCorrection']>().mockResolvedValue([]),
    applyCorrection: vi
      .fn<WorkspaceStore['applyCorrection']>()
      .mockResolvedValue({ workspace: emptyWorkspace, changed: 0 }),
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
  } as WorkspaceStore;
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
    await bridge.markProtocolReviewed('meeting-envelope-options');
    await bridge.autosaveProtocol('meeting-envelope-options', '# Revised protocol');
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
      appearance: DEFAULT_APPEARANCE,
      furniture: EMPTY_FURNITURE,
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
      names: [],
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
      appearance: DEFAULT_APPEARANCE,
      furniture: EMPTY_FURNITURE,
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
      stage: 'interrupted',
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

  /**
   * Presets, which nothing covered until the concept was renamed.
   *
   * The rename touched a struct, four Tauri commands, four bridge methods, four
   * component props and a table that deliberately kept its old name, and the
   * suite went green throughout because none of it was exercised. Both compilers
   * agree the names line up; only a test says the behaviour does.
   */
  /**
   * Archiving in the preview, which is where a real bug lived.
   *
   * The first version removed the project from the archived list and then, in
   * the next statement, looked in that same list for the project it was meant to
   * be putting back. It type-checked, it read correctly, and it silently lost the
   * project: archiving worked, bringing it back did nothing at all. Only using it
   * showed that, so this is here to keep it shown.
   */
  describe('archiving', () => {
    it('takes a project out of the list and puts the same one back', async () => {
      const bridge = new FakeWorkflowBridge({});
      const before = await bridge.getSnapshot();
      const project = before.projects[0];
      expect(project).toBeDefined();

      await bridge.setProjectArchived(project!.id, true);
      const hidden = await bridge.getSnapshot();
      expect(hidden.projects.some((each) => each.id === project!.id)).toBe(false);
      expect(hidden.projects.length).toBe(before.projects.length - 1);

      const away = await bridge.archivedWork();
      expect(away.projects.map((each) => each.id)).toEqual([project!.id]);

      await bridge.setProjectArchived(project!.id, false);
      const back = await bridge.getSnapshot();
      expect(back.projects.some((each) => each.id === project!.id)).toBe(true);
      expect(back.projects.length).toBe(before.projects.length);
      expect((await bridge.archivedWork()).projects).toHaveLength(0);
    });

    it('does the same for one meeting without touching its project', async () => {
      const bridge = new FakeWorkflowBridge({});
      const before = await bridge.getSnapshot();
      const meeting = before.meetings[0];
      expect(meeting).toBeDefined();

      await bridge.setMeetingArchived(meeting!.id, true);
      const hidden = await bridge.getSnapshot();
      expect(hidden.meetings.some((each) => each.id === meeting!.id)).toBe(false);
      // The project it belongs to is untouched.
      expect(hidden.projects.length).toBe(before.projects.length);

      await bridge.setMeetingArchived(meeting!.id, false);
      const back = await bridge.getSnapshot();
      expect(back.meetings.some((each) => each.id === meeting!.id)).toBe(true);
      expect(back.meetings.length).toBe(before.meetings.length);
    });
  });

  describe('appearance presets', () => {
    it('lists what shipped, saves a new one, and applies it to a project', async () => {
      const bridge = new FakeWorkflowBridge({});

      const shipped = await bridge.appearancePresets();
      expect(shipped.length).toBeGreaterThan(0);
      // Everything seeded is built in, and a built-in cannot be removed.
      expect(shipped.every((preset) => preset.builtIn)).toBe(true);

      const saved = await bridge.saveAppearancePreset(
        'House style',
        'What this office uses',
        { ...DEFAULT_APPEARANCE, bodySize: 12 },
        EMPTY_FURNITURE,
      );
      const mine = saved.find((preset) => preset.name === 'House style');
      expect(mine).toBeDefined();
      expect(mine?.builtIn).toBe(false);
      expect(mine?.appearance.bodySize).toBe(12);

      // Applying sets the project, because a preset belongs to the project rather
      // than to the protocol somebody happened to have open when they saved it.
      const before = await bridge.getSnapshot();
      const project = before.projects[0];
      expect(project).toBeDefined();
      await bridge.applyAppearancePreset(project!.id, mine!.id);

      const after = await bridge.getSnapshot();
      expect(after.projects.find((each) => each.id === project!.id)?.appearance.bodySize).toBe(12);
    });

    it('removes one that was saved and leaves the shipped ones alone', async () => {
      const bridge = new FakeWorkflowBridge({});
      const saved = await bridge.saveAppearancePreset(
        'Throwaway',
        '',
        DEFAULT_APPEARANCE,
        EMPTY_FURNITURE,
      );
      const mine = saved.find((preset) => preset.name === 'Throwaway');
      expect(mine).toBeDefined();

      const after = await bridge.deleteAppearancePreset(mine!.id);
      expect(after.find((preset) => preset.id === mine!.id)).toBeUndefined();
      expect(after.some((preset) => preset.builtIn)).toBe(true);
    });
  });
});
