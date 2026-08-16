import type {
  ActiveJob,
  FakeJobOutcome,
  JobKind,
  MeetingLifecycle,
  MeetingSummary,
  NewMeetingInput,
  NewProjectInput,
  ProjectSummary,
  ProtocolDraft,
  TranscriptDocument,
  TranscriptSegment,
  WorkflowBridge,
  WorkflowSnapshot,
  TranscriptionRuntimeStatus,
  SpeakerSeparationStatus,
  VocabularyDraft,
  FileDropEvent,
  SpeakerRequest,
  RecordingEdits,
  RecordingReview,
  NameCandidate,
  CorrectionMatch,
  AppliedCorrection,
} from './types';
import type { WorkspaceStore } from './workspaceStore';

interface FakeBridgeOptions {
  tickMs?: number;
  progressStep?: number;
  workspaceStore?: WorkspaceStore | undefined;
}

const initialMeetings: MeetingSummary[] = [
  {
    id: 'meeting-envelope-options',
    projectId: 'project-harbor-canopy',
    title: 'Envelope options',
    occurredAt: '2026-07-29',
    durationLabel: '48 min',
    lifecycle: 'protocol_draft',
    language: 'English',
    sourceName: 'synthetic-envelope-review.m4a',
    sourceByteCount: 48_200_000,
    sourceMediaType: 'audio/mp4',
    styleId: 'style-formal',
  },
  {
    id: 'meeting-access-review',
    projectId: 'project-harbor-canopy',
    title: 'Access review',
    occurredAt: '2026-07-22',
    durationLabel: '1 h 06 min',
    lifecycle: 'transcript_ready',
    language: 'English',
    sourceName: 'synthetic-access-review.wav',
    sourceByteCount: 72_400_000,
    sourceMediaType: 'audio/wav',
    styleId: 'style-formal',
  },
  {
    id: 'meeting-kickoff',
    projectId: 'project-harbor-canopy',
    title: 'Project kick-off',
    occurredAt: '2026-07-08',
    durationLabel: '34 min',
    lifecycle: 'source_ready',
    language: 'English',
    sourceName: 'synthetic-kickoff.mp3',
    sourceByteCount: 33_800_000,
    sourceMediaType: 'audio/mpeg',
    styleId: 'style-working-note',
  },
];

const sampleTranscript: TranscriptSegment[] = [
  {
    id: 'segment-1',
    startMs: 12_000,
    endMs: 30_000,
    speaker: 'Speaker 1',
    text: 'Let’s confirm the two options we want to carry into the next review.',
    needsReview: false,
  },
  {
    id: 'segment-2',
    startMs: 31_000,
    endMs: 56_000,
    speaker: 'Speaker 2',
    text: 'The lighter assembly is preferable, provided the junction detail remains serviceable.',
    needsReview: false,
  },
  {
    id: 'segment-3',
    startMs: 57_000,
    endMs: 80_000,
    speaker: 'Speaker 1',
    text: 'We still need the acoustic note and the updated cost range before deciding.',
    needsReview: true,
  },
  {
    id: 'segment-4',
    startMs: 81_000,
    endMs: 102_000,
    speaker: 'Speaker 3',
    text: 'I will circulate both items before Thursday afternoon.',
    needsReview: false,
  },
];

const sampleProtocol: ProtocolDraft = {
  meetingId: 'meeting-envelope-options',
  revisionId: 'protocol-demo-1',
  transcriptRevisionId: 'transcript-demo-1',
  markdown: `# Meeting protocol

## Purpose

Review the current envelope options and identify the information required for a decision.

## Discussion

- The lighter assembly remains the preferred direction.
- Serviceability at the primary junction must be confirmed.
- Acoustic advice and an updated cost range are still outstanding.

## Decisions

No final assembly decision was made in this meeting.

## Actions

| Task | Responsible |
| --- | --- |
| Prepare the acoustic note and updated cost range before Thursday | Priya |
| Carry both viable options into the next review | |
`,
  styleId: 'style-formal',
  reviewState: 'draft',
  isDirty: false,
  saveState: 'saved',
  savedAtMs: Date.now(),
  revisions: [{ id: 'protocol-demo-1', ordinal: 1, status: 'draft', createdAtMs: Date.now() }],
  // The preview shows the evidence a real run records, because a demonstration
  // that omits it demonstrates a different product.
  evidence: {
    quantitiesStated: 4,
    quantitiesAccounted: 3,
    quantitiesInvented: [],
    tasksUnowned: ['Carry both viable options into the next review'],
    charactersSpoken: 6120,
    charactersWritten: 780,
  },
};

function sampleTranscriptDocument(meetingId: string): TranscriptDocument {
  return {
    schemaVersion: 1,
    meetingId,
    revisionId: `transcript-demo-${meetingId}`,
    language: 'English',
    speakerResolution: 'resolved',
    segments: structuredClone(sampleTranscript),
    baseRevisionId: `transcript-demo-${meetingId}`,
    isDirty: false,
    saveState: 'saved',
    savedAtMs: Date.now(),
  };
}

const jobStages: Record<JobKind, string[]> = {
  import: ['Checking source', 'Copying original', 'Preparing working audio'],
  transcription: ['Preparing local model', 'Transcribing audio', 'Writing transcript revision'],
  generation: ['Preparing reviewed transcript', 'Generating draft', 'Writing protocol revision'],
};

const completionLifecycle: Record<JobKind, MeetingLifecycle> = {
  import: 'source_ready',
  transcription: 'transcript_ready',
  generation: 'protocol_draft',
};

function cloneSnapshot(snapshot: WorkflowSnapshot): WorkflowSnapshot {
  // Adapter state crosses the boundary by value so consumers cannot mutate its authority.
  return structuredClone(snapshot);
}

/**
 * Deterministic, in-memory implementation of the workflow boundary used by the Phase 0 shell.
 * It exercises UI states without touching files, models, or confidential meeting data.
 */
export class FakeWorkflowBridge implements WorkflowBridge {
  /** Edits made in the preview, which last as long as the page does. */
  private recordingEdits = new Map<string, RecordingEdits>();
  private snapshot: WorkflowSnapshot = {
    projects: [
      {
        id: 'project-harbor-canopy',
        name: 'Halle 4 Study',
        description: 'Synthetic demonstration project for the Phase 0 shell.',
        meetingCount: initialMeetings.length,
        defaultLanguage: 'English',
        defaultStyleId: 'style-formal',
      },
      {
        id: 'project-material-lab',
        name: 'Material Lab',
        description: 'Reusable studies and internal working sessions.',
        meetingCount: 0,
        defaultLanguage: 'English',
        defaultStyleId: 'style-working-note',
      },
    ],
    meetings: structuredClone(initialMeetings),
    transcripts: {
      'meeting-envelope-options': sampleTranscriptDocument('meeting-envelope-options'),
      'meeting-access-review': sampleTranscriptDocument('meeting-access-review'),
    },
    protocols: { 'meeting-envelope-options': structuredClone(sampleProtocol) },
    styles: [
      {
        id: 'style-formal',
        name: 'Formal minutes',
        description: 'Structured record of discussion, decisions, and actions.',
        language: 'Meeting language',
        density: 'comprehensive',
      },
      {
        id: 'style-working-note',
        name: 'Internal working note',
        description: 'Concise working record for an internal project team.',
        language: 'Meeting language',
        density: 'concise',
      },
      {
        id: 'style-decision-log',
        name: 'Technical decision log',
        description: 'Emphasises alternatives, constraints, and explicit decisions.',
        language: 'Meeting language',
        density: 'terse',
      },
    ],
    vocabulary: [
      {
        id: 'vocab-1',
        term: 'serviceability',
        category: 'Technical term',
        scope: 'Global',
        projectId: null,
        enabled: true,
      },
      {
        id: 'vocab-2',
        term: 'junction detail',
        category: 'Building part',
        scope: 'Project',
        projectId: 'project-harbor-canopy',
        enabled: true,
      },
      {
        id: 'vocab-3',
        term: 'acoustic note',
        category: 'Document',
        scope: 'Project',
        projectId: 'project-harbor-canopy',
        enabled: true,
      },
    ],
    jobs: [],
    activeJob: null,
    nextJobOutcome: 'success',
    activeMeetingId: null,
    activeRoute: null,
  };

  private readonly listeners = new Set<(snapshot: WorkflowSnapshot) => void>();
  private readonly tickMs: number;
  private readonly progressStep: number;
  private readonly workspaceStore: WorkspaceStore | null;
  private readonly ready: Promise<void>;
  // One supervised timer is intentional here; this demo is not a general workflow engine.
  private timer: ReturnType<typeof setInterval> | null = null;
  private meetingSequence = 1;
  private projectSequence = 1;

  constructor(options: FakeBridgeOptions = {}) {
    this.tickMs = options.tickMs ?? 420;
    this.progressStep = options.progressStep ?? 11;
    this.workspaceStore = options.workspaceStore ?? null;
    if (this.workspaceStore) {
      // Native startup must not flash synthetic projects before SQLite has loaded.
      this.snapshot.projects = [];
      this.snapshot.meetings = [];
      this.snapshot.transcripts = {};
      this.snapshot.protocols = {};
      this.ready = this.loadDurableWorkspace(this.workspaceStore);
      void this.workspaceStore.subscribe((workspace) => {
        this.applyDurableWorkspace(workspace);
        this.emit();
      });
    } else {
      this.ready = Promise.resolve();
    }
  }

  async getSnapshot(): Promise<WorkflowSnapshot> {
    await this.ready;
    return cloneSnapshot(this.snapshot);
  }

  subscribe(
    listener: (snapshot: WorkflowSnapshot) => void,
    onError?: (message: string) => void,
  ): () => void {
    this.listeners.add(listener);
    if (this.workspaceStore) {
      void this.ready
        .then(() => {
          if (this.listeners.has(listener)) listener(cloneSnapshot(this.snapshot));
        })
        .catch((error: unknown) => onError?.(errorMessage(error)));
    } else {
      listener(cloneSnapshot(this.snapshot));
    }
    return () => this.listeners.delete(listener);
  }

  async createProject(input: NewProjectInput): Promise<ProjectSummary> {
    await this.ready;
    const project: ProjectSummary = this.workspaceStore
      ? await this.workspaceStore.createProject(input)
      : {
          id: `project-demo-${this.projectSequence++}`,
          name: input.name.trim() || 'Untitled project',
          description: input.description.trim(),
          meetingCount: 0,
          defaultLanguage: input.defaultLanguage,
          defaultStyleId: 'style-formal',
        };
    this.snapshot.projects = [...this.snapshot.projects, project];
    this.emit();
    return structuredClone(project);
  }

  async createMeeting(input: NewMeetingInput): Promise<MeetingSummary> {
    await this.ready;
    const meeting: MeetingSummary = this.workspaceStore
      ? await this.workspaceStore.createMeeting(input)
      : {
          id: `meeting-demo-${this.meetingSequence++}`,
          projectId: input.projectId,
          title: input.title.trim() || this.titleFromSource(input.sourceName),
          occurredAt: input.occurredAt,
          durationLabel: null,
          lifecycle: 'draft',
          language: input.language,
          sourceName: input.sourceName,
          sourceByteCount: null,
          sourceMediaType: null,
          styleId: input.styleId,
        };

    this.snapshot.meetings = [meeting, ...this.snapshot.meetings];
    const project = this.snapshot.projects.find((candidate) => candidate.id === input.projectId);
    if (project) project.meetingCount += 1;
    this.emit();
    return structuredClone(meeting);
  }

  async importRecording(meetingId: string): Promise<void> {
    if (this.workspaceStore) {
      await this.workspaceStore.startImport(meetingId);
      return;
    }
    this.startJob(meetingId, 'import');
  }

  async startTranscription(
    meetingId: string,
    speakers: SpeakerRequest = 'together',
  ): Promise<void> {
    if (this.workspaceStore) {
      const failRequested = this.snapshot.nextJobOutcome === 'failure';
      this.snapshot.nextJobOutcome = 'success';
      await this.workspaceStore.startTranscription(meetingId, failRequested, speakers);
      return;
    }
    this.startJob(meetingId, 'transcription');
  }

  async generateProtocol(meetingId: string): Promise<void> {
    if (this.workspaceStore) {
      const failRequested = this.snapshot.nextJobOutcome === 'failure';
      this.snapshot.nextJobOutcome = 'success';
      await this.workspaceStore.startGeneration(meetingId, failRequested);
      return;
    }
    this.startJob(meetingId, 'generation');
  }

  async cancelActiveJob(meetingId: string): Promise<void> {
    const job = this.workspaceStore
      ? this.snapshot.jobs.find((candidate) => candidate.meetingId === meetingId)
      : this.snapshot.activeJob;
    if (!job || !['queued', 'running'].includes(job.state)) return;
    if (this.workspaceStore && job.kind === 'import') {
      await this.workspaceStore.cancelImport(job.meetingId);
      return;
    }
    if (this.workspaceStore) {
      await this.workspaceStore.cancelProcessing(job.meetingId);
      return;
    }
    job.state = 'cancelling';
    job.stage = 'Stopping local process safely';
    this.emit();
  }

  async retryActiveJob(meetingId: string): Promise<void> {
    const job = this.workspaceStore
      ? this.snapshot.jobs.find((candidate) => candidate.meetingId === meetingId)
      : this.snapshot.activeJob;
    if (!job || !['queued', 'failed', 'cancelled', 'interrupted'].includes(job.state)) return;
    if (this.workspaceStore && job.kind === 'import') {
      await this.workspaceStore.retryImport(job.meetingId, false);
      return;
    }
    if (this.workspaceStore) {
      await this.workspaceStore.retryProcessing(job.meetingId);
      return;
    }
    this.snapshot.nextJobOutcome = 'success';
    this.startJob(job.meetingId, job.kind, job.attempt + 1);
  }

  async confirmDuplicateImport(meetingId: string): Promise<void> {
    if (!this.workspaceStore) return;
    await this.workspaceStore.retryImport(meetingId, true);
  }

  async reselectImportSource(meetingId: string): Promise<void> {
    if (!this.workspaceStore) return;
    const source = await this.workspaceStore.selectMediaSource();
    if (!source) return;
    await this.workspaceStore.replaceImportSource(meetingId, source);
  }

  async updateMeetingTitle(meetingId: string, title: string): Promise<void> {
    await this.ready;
    const meeting = this.findMeeting(meetingId);
    const nextTitle = title.trim();
    if (!nextTitle) return;
    if (this.workspaceStore) await this.workspaceStore.updateMeetingTitle(meetingId, nextTitle);
    meeting.title = nextTitle;
    this.emit();
  }

  async updateMeetingLanguage(meetingId: string, language: string): Promise<void> {
    const nextLanguage = language.trim();
    if (!nextLanguage) throw new Error('Choose a meeting language.');
    if (this.workspaceStore?.updateMeetingLanguage) {
      await this.workspaceStore.updateMeetingLanguage(meetingId, nextLanguage);
      return;
    }
    this.findMeeting(meetingId).language = nextLanguage;
    const transcript = this.snapshot.transcripts[meetingId];
    if (transcript) transcript.language = nextLanguage;
    this.emit();
  }

  async deleteTranscriptSegment(meetingId: string, segmentId: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(
        await this.workspaceStore.deleteTranscriptSegment(meetingId, segmentId),
      );
      this.emit();
      return;
    }
    const document = this.snapshot.transcripts[meetingId];
    // The last one stays, as it does in the application: a transcript of nothing
    // is not a document somebody meant to make.
    if (!document || document.segments.length <= 1) return;
    document.segments = document.segments.filter((candidate) => candidate.id !== segmentId);
    document.isDirty = true;
    document.savedAtMs = Date.now();
    this.emit();
  }

  async updateTranscriptSegment(meetingId: string, segmentId: string, text: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(
        await this.workspaceStore.updateTranscriptSegment(meetingId, segmentId, text),
      );
      this.emit();
      return;
    }
    const document = this.snapshot.transcripts[meetingId];
    const segment = document?.segments.find((candidate) => candidate.id === segmentId);
    if (!segment) return;
    segment.text = text;
    segment.needsReview = false;
    if (document) {
      document.isDirty = true;
      document.savedAtMs = Date.now();
    }
    this.emit();
  }

  async updateSpeaker(meetingId: string, speaker: string, replacement: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(
        await this.workspaceStore.renameTranscriptSpeaker(meetingId, speaker, replacement),
      );
      this.emit();
      return;
    }
    const nextSpeaker = replacement.trim();
    if (!nextSpeaker) return;
    const document = this.snapshot.transcripts[meetingId];
    for (const segment of document?.segments ?? []) {
      if (segment.speaker === speaker) segment.speaker = nextSpeaker;
    }
    if (document) document.isDirty = true;
    this.emit();
  }

  /// The browser preview has no native window, so nothing is ever dropped on it.
  /// Delegated like every other call that needs the real application. Returning a
  /// no-op unconditionally is why dropping a recording did nothing: the interface
  /// talks to this bridge, not to the store behind it, so a method that forgets to
  /// pass the call through is indistinguishable from a feature that was never
  /// built. In the browser preview there is no native window to drop onto.
  subscribeFileDrops(handler: (event: FileDropEvent) => void): () => void {
    return this.workspaceStore?.subscribeFileDrops(handler) ?? (() => undefined);
  }

  /**
   * Stand-in candidates until the extractor exists in Rust.
   *
   * Shaped like the real thing measured on a long meeting: a handful of words, some
   * of them names worth keeping and some of them the transcriber simply fumbling.
   */
  async findNameCandidates(_meetingId: string): Promise<NameCandidate[]> {
    return [
      {
        heard: 'Prüfstelle',
        occurrences: 7,
        context: '…confirmed with Prüfstelle before the review…',
      },
      {
        heard: 'Junktion',
        occurrences: 4,
        context: '…the Junktion detail remains serviceable…',
      },
      {
        heard: 'Fachplanung',
        occurrences: 3,
        context: '…Fachplanung will circulate both items…',
      },
      { heard: 'ansonst', occurrences: 2, context: '…und ansonst bleibt es dabei…' },
      { heard: 'kanopie', occurrences: 2, context: '…die kanopie über dem Eingang…' },
    ];
  }

  async previewCorrection(
    meetingId: string,
    wrong: string,
    _right: string,
  ): Promise<CorrectionMatch[]> {
    const document = this.snapshot.transcripts[meetingId];
    if (!document) return [];
    return document.segments
      .filter((segment: TranscriptSegment) => segment.text.includes(wrong))
      .map((segment: TranscriptSegment) => ({
        segmentId: segment.id,
        startMs: segment.startMs,
        context: segment.text,
      }));
  }

  async applyCorrection(meetingId: string, correction: AppliedCorrection): Promise<void> {
    const document = this.snapshot.transcripts[meetingId];
    if (!document) return;
    for (const segment of document.segments) {
      if (correction.keptSegmentIds.length && !correction.keptSegmentIds.includes(segment.id)) {
        continue;
      }
      segment.text = segment.text.split(correction.wrong).join(correction.right);
    }
    if (correction.remember) {
      await this.saveVocabularyEntry({
        id: null,
        term: correction.right,
        category: 'Person',
        scope: 'Project',
        projectId: this.snapshot.projects[0]?.id ?? null,
        enabled: true,
      });
    }
    this.emit();
  }

  async saveVocabularyEntry(draft: VocabularyDraft): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(await this.workspaceStore.saveVocabularyEntry(draft));
      this.emit();
      return;
    }
    const term = draft.term.trim();
    if (!term) return;
    const projectId = draft.scope === 'Project' ? draft.projectId : null;
    const existing = this.snapshot.vocabulary.find((entry) => entry.id === draft.id);
    if (existing) {
      Object.assign(existing, {
        term,
        category: draft.category,
        scope: draft.scope,
        projectId,
        enabled: draft.enabled,
      });
    } else {
      this.snapshot.vocabulary = [
        ...this.snapshot.vocabulary,
        {
          id: `vocab-${this.snapshot.vocabulary.length + 1}`,
          term,
          category: draft.category,
          scope: draft.scope,
          projectId,
          enabled: draft.enabled,
        },
      ];
    }
    this.emit();
  }

  async deleteVocabularyEntry(entryId: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(await this.workspaceStore.deleteVocabularyEntry(entryId));
      this.emit();
      return;
    }
    this.snapshot.vocabulary = this.snapshot.vocabulary.filter((entry) => entry.id !== entryId);
    this.emit();
  }

  async updateProtocol(meetingId: string, markdown: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(await this.workspaceStore.autosaveProtocol(meetingId, markdown));
      this.emit();
      return;
    }
    const protocol = this.snapshot.protocols[meetingId];
    if (!protocol) return;
    protocol.markdown = markdown;
    protocol.savedAtMs = Date.now();
    protocol.isDirty = true;
    if (protocol.reviewState === 'reviewed') protocol.reviewState = 'changed_since_review';
    this.emit();
  }

  async createProtocolRevision(meetingId: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(await this.workspaceStore.createProtocolRevision(meetingId));
      this.emit();
      return;
    }
    const protocol = this.snapshot.protocols[meetingId];
    if (!protocol) return;
    protocol.revisionId = `protocol-demo-${Date.now()}`;
    protocol.isDirty = false;
    protocol.reviewState = 'draft';
    protocol.revisions.unshift({
      id: protocol.revisionId,
      ordinal: protocol.revisions.length + 1,
      status: 'draft',
      createdAtMs: Date.now(),
    });
    this.findMeeting(meetingId).lifecycle = 'protocol_draft';
    this.emit();
  }

  async markReviewed(meetingId: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(await this.workspaceStore.markProtocolReviewed(meetingId));
      this.emit();
      return;
    }
    await this.createProtocolRevision(meetingId);
    const meeting = this.findMeeting(meetingId);
    const protocol = this.snapshot.protocols[meetingId];
    if (protocol) {
      protocol.reviewState = 'reviewed';
      const revision = protocol.revisions.find((item) => item.id === protocol.revisionId);
      if (revision) revision.status = 'reviewed';
    }
    meeting.lifecycle = 'reviewed';
    this.emit();
  }

  async restoreProtocolRevision(meetingId: string, revisionId: string): Promise<void> {
    if (this.workspaceStore) {
      this.applyDurableWorkspace(
        await this.workspaceStore.restoreProtocolRevision(meetingId, revisionId),
      );
      this.emit();
      return;
    }
    const protocol = this.snapshot.protocols[meetingId];
    if (!protocol?.revisions.some((revision) => revision.id === revisionId)) return;
    protocol.revisionId = `protocol-demo-restored-${Date.now()}`;
    protocol.reviewState = 'draft';
    protocol.isDirty = false;
    protocol.revisions.unshift({
      id: protocol.revisionId,
      ordinal: protocol.revisions.length + 1,
      status: 'draft',
      createdAtMs: Date.now(),
    });
    this.findMeeting(meetingId).lifecycle = 'protocol_draft';
    this.emit();
  }

  async saveWorkspaceLocation(
    meetingId: string,
    route: 'meeting' | 'transcript' | 'protocol',
  ): Promise<void> {
    this.snapshot.activeMeetingId = meetingId;
    this.snapshot.activeRoute = route;
    if (this.workspaceStore) await this.workspaceStore.saveWorkspaceLocation(meetingId, route);
  }

  async setNextJobOutcome(outcome: FakeJobOutcome): Promise<void> {
    this.snapshot.nextJobOutcome = outcome;
    this.emit();
  }

  async getTranscriptionRuntimeStatus(): Promise<TranscriptionRuntimeStatus> {
    if (this.workspaceStore?.getTranscriptionRuntimeStatus) {
      return this.workspaceStore.getTranscriptionRuntimeStatus();
    }
    return {
      executablePath: null,
      modelPath: null,
      executableFound: false,
      modelFound: false,
      runtimeVersion: null,
      modelDigest: null,
      modelByteCount: null,
    };
  }

  async configureTranscriptionRuntime(executablePath: string): Promise<TranscriptionRuntimeStatus> {
    if (this.workspaceStore?.configureTranscriptionRuntime) {
      return this.workspaceStore.configureTranscriptionRuntime(executablePath);
    }
    return this.getTranscriptionRuntimeStatus();
  }

  async getSpeakerSeparationStatus(): Promise<SpeakerSeparationStatus> {
    if (this.workspaceStore?.getSpeakerSeparationStatus) {
      return this.workspaceStore.getSpeakerSeparationStatus();
    }
    return {
      modelsInstalled: false,
      runtimeConfigured: false,
      runtimeHealthy: false,
      runtimeVersion: null,
      runtimePath: null,
      downloadBytes: 0,
    };
  }

  async configureSpeakerRuntime(executablePath: string): Promise<SpeakerSeparationStatus> {
    if (this.workspaceStore?.configureSpeakerRuntime) {
      return this.workspaceStore.configureSpeakerRuntime(executablePath);
    }
    return this.getSpeakerSeparationStatus();
  }

  async downloadSpeakerModels(): Promise<void> {
    await this.workspaceStore?.downloadSpeakerModels?.();
  }

  subscribeSpeakerEvents(handler: (status: SpeakerSeparationStatus) => void): () => void {
    return this.workspaceStore?.subscribeSpeakerEvents?.(handler) ?? (() => {});
  }

  // A recording's shape, invented for the preview: a quiet start, speech in the
  // middle with the pauses a meeting really has, and a quiet end.
  async getRecordingReview(meetingId: string): Promise<RecordingReview | null> {
    const durationMs = 74 * 60 * 1000;
    const buckets = 1200;
    const waveform = Array.from({ length: buckets }, (_, index) => {
      const through = index / buckets;
      if (through < 0.03 || through > 0.96) return 0.01;
      // Speech, with the gaps between turns and a quieter passage two thirds in.
      const turn = 0.55 + 0.35 * Math.abs(Math.sin(index * 0.7));
      const gap = Math.sin(index * 0.21) > 0.82 ? 0.08 : 1;
      const quieter = through > 0.62 && through < 0.68 ? 0.35 : 1;
      return Math.min(1, turn * gap * quieter);
    });
    return {
      durationMs,
      waveform,
      edits: this.recordingEdits.get(meetingId) ?? { startMs: 0 },
      keptDurationMs: durationMs,
    };
  }

  async setRecordingEdits(meetingId: string, edits: RecordingEdits): Promise<void> {
    this.recordingEdits.set(meetingId, edits);
  }

  async getMeetingAudio(meetingId: string): Promise<import('./types').MeetingAudio | null> {
    // The browser preview has no managed media, so no audio is offered.
    return this.workspaceStore?.getMeetingAudio?.(meetingId) ?? null;
  }

  async getTranscriptionCapability(): Promise<import('./types').TranscriptionCapability> {
    if (this.workspaceStore?.getTranscriptionCapability) {
      return this.workspaceStore.getTranscriptionCapability();
    }
    // The browser preview has no managed storage, so nothing is installed.
    return {
      selectedPreset: 'balanced',
      presets: [
        { preset: 'fast', modelId: 'tiny', byteCount: 77_691_713, installed: false },
        { preset: 'balanced', modelId: 'base', byteCount: 147_951_465, installed: false },
        { preset: 'accurate', modelId: 'medium', byteCount: 1_533_763_059, installed: false },
      ],
    };
  }

  async setTranscriptionPreset(
    preset: import('./types').TranscriptionPreset,
  ): Promise<import('./types').TranscriptionCapability> {
    if (this.workspaceStore?.setTranscriptionPreset) {
      return this.workspaceStore.setTranscriptionPreset(preset);
    }
    const capability = await this.getTranscriptionCapability();
    return { ...capability, selectedPreset: preset };
  }

  async downloadTranscriptionModel(modelId: string): Promise<void> {
    await this.workspaceStore?.downloadTranscriptionModel?.(modelId);
  }

  async cancelTranscriptionDownload(modelId: string): Promise<void> {
    await this.workspaceStore?.cancelTranscriptionDownload?.(modelId);
  }

  async removeTranscriptionModel(
    modelId: string,
  ): Promise<import('./types').TranscriptionCapability> {
    if (this.workspaceStore?.removeTranscriptionModel) {
      return this.workspaceStore.removeTranscriptionModel(modelId);
    }
    return this.getTranscriptionCapability();
  }

  subscribeModelEvents(handlers: {
    onProgress: (progress: import('./types').ModelDownloadProgress) => void;
    onChanged: (capability: import('./types').TranscriptionCapability) => void;
    onError: (error: import('./types').ModelDownloadError) => void;
  }): () => void {
    return this.workspaceStore?.subscribeModelEvents?.(handlers) ?? (() => {});
  }

  async exportProtocol(
    meetingId: string,
    format: 'markdown' | 'text',
    title: string,
  ): Promise<boolean> {
    if (this.workspaceStore?.exportProtocol) {
      return this.workspaceStore.exportProtocol(meetingId, format, title);
    }
    return false;
  }

  async getProtocolProviderStatus(): Promise<import('./types').ProtocolProviderStatus> {
    if (this.workspaceStore?.getProtocolProviderStatus) {
      return this.workspaceStore.getProtocolProviderStatus();
    }
    return {
      endpoint: 'http://127.0.0.1:11434',
      serverReachable: false,
      runtimeVersion: null,
      models: [],
      selectedModel: null,
      selectedModelDigest: null,
      selectedModelReady: false,
      message: 'Ollama is available as an optional local provider for development.',
    };
  }

  async configureProtocolProvider(
    model: string | null,
  ): Promise<import('./types').ProtocolProviderStatus> {
    if (this.workspaceStore?.configureProtocolProvider) {
      return this.workspaceStore.configureProtocolProvider(model);
    }
    void model;
    return this.getProtocolProviderStatus();
  }

  private startJob(meetingId: string, kind: JobKind, attempt = 1): void {
    if (this.timer) clearInterval(this.timer);
    this.findMeeting(meetingId);
    this.snapshot.activeJob = {
      id: `job-${kind}-${Date.now()}`,
      meetingId,
      kind,
      state: 'queued',
      outcome: null,
      progress: 0,
      progressBytes: 0,
      totalBytes: null,
      stage: 'Queued locally',
      attempt,
      error: null,
      requiresDuplicateConfirmation: false,
    };
    this.emit();
    this.timer = setInterval(() => this.advanceJob(), this.tickMs);
  }

  private async loadDurableWorkspace(store: WorkspaceStore): Promise<void> {
    const workspace = await store.loadWorkspace();
    this.applyDurableWorkspace(workspace);
  }

  private applyDurableWorkspace(workspace: Awaited<ReturnType<WorkspaceStore['loadWorkspace']>>) {
    this.snapshot.projects = workspace.projects;
    this.snapshot.meetings = workspace.meetings;
    this.snapshot.jobs = workspace.jobs;
    this.snapshot.activeJob =
      workspace.jobs.find((job) => ['queued', 'running', 'cancelling'].includes(job.state)) ??
      workspace.jobs[0] ??
      null;
    this.snapshot.transcripts = workspace.transcripts;
    this.snapshot.protocols = workspace.protocols;
    this.snapshot.styles = workspace.styles;
    this.snapshot.vocabulary = workspace.vocabulary;
    this.snapshot.activeMeetingId = workspace.activeMeetingId;
    this.snapshot.activeRoute = workspace.activeRoute;
  }

  private advanceJob(): void {
    const job = this.snapshot.activeJob;
    if (!job) return this.stopTimer();

    if (job.state === 'cancelling') {
      job.state = 'completed';
      job.outcome = 'cancelled';
      job.stage = 'Cancelled — latest stable work retained';
      this.stopTimer();
      this.emit();
      return;
    }

    if (job.state === 'queued') job.state = 'running';
    job.progress = Math.min(100, job.progress + this.progressStep);
    const stages = jobStages[job.kind];
    const stageIndex = Math.min(stages.length - 1, Math.floor(job.progress / 34));
    job.stage = stages[stageIndex] ?? 'Working locally';

    if (this.snapshot.nextJobOutcome === 'failure' && job.progress >= 45) {
      job.state = 'failed';
      job.stage = `${this.labelForJob(job.kind)} stopped`;
      job.error = {
        code: 'synthetic_failure',
        title: `${this.labelForJob(job.kind)} could not finish`,
        detail:
          'The synthetic runtime stopped as requested. The source and latest stable work are still safe.',
      };
      // Make failure one-shot so Retry demonstrates recovery without another settings change.
      this.snapshot.nextJobOutcome = 'success';
      this.stopTimer();
      this.emit();
      return;
    }

    if (job.progress >= 100) {
      job.state = 'completed';
      job.outcome = 'succeeded';
      job.stage = `${this.labelForJob(job.kind)} complete`;
      this.completeJob(job);
      this.stopTimer();
    }
    this.emit();
  }

  private completeJob(job: ActiveJob): void {
    const meeting = this.findMeeting(job.meetingId);
    // Fake lifecycle advances after success but remains session-only until artifact commits exist.
    meeting.lifecycle = completionLifecycle[job.kind];
    if (job.kind === 'import') meeting.durationLabel = '42 min';
    if (job.kind === 'transcription') {
      const transcript = sampleTranscriptDocument(job.meetingId);
      transcript.language = meeting.language;
      this.snapshot.transcripts[job.meetingId] = transcript;
    }
    if (job.kind === 'generation') {
      this.snapshot.protocols[job.meetingId] = {
        ...structuredClone(sampleProtocol),
        meetingId: job.meetingId,
        styleId: meeting.styleId,
      };
    }
  }

  private findMeeting(meetingId: string): MeetingSummary {
    const meeting = this.snapshot.meetings.find((candidate) => candidate.id === meetingId);
    if (!meeting) throw new Error(`Unknown synthetic meeting: ${meetingId}`);
    return meeting;
  }

  private titleFromSource(sourceName: string): string {
    return sourceName
      .replace(/\.[^.]+$/, '')
      .replace(/[-_]+/g, ' ')
      .replace(/^\w/, (character) => character.toUpperCase());
  }

  private labelForJob(kind: JobKind): string {
    if (kind === 'import') return 'Import';
    if (kind === 'transcription') return 'Transcription';
    return 'Protocol generation';
  }

  private emit(): void {
    const snapshot = cloneSnapshot(this.snapshot);
    for (const listener of this.listeners) listener(snapshot);
  }

  private stopTimer(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }
}

function errorMessage(error: unknown): string {
  return typeof error === 'string'
    ? error
    : error instanceof Error
      ? error.message
      : 'LocaLog could not prepare its local workspace.';
}
