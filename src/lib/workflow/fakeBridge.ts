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
  TranscriptSegment,
  WorkflowBridge,
  WorkflowSnapshot,
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
    startSeconds: 12,
    startLabel: '00:00:12',
    speaker: 'Speaker 1',
    text: 'Let’s confirm the two options we want to carry into the next review.',
    needsReview: false,
  },
  {
    id: 'segment-2',
    startSeconds: 31,
    startLabel: '00:00:31',
    speaker: 'Speaker 2',
    text: 'The lighter assembly is preferable, provided the junction detail remains serviceable.',
    needsReview: false,
  },
  {
    id: 'segment-3',
    startSeconds: 57,
    startLabel: '00:00:57',
    speaker: 'Speaker 1',
    text: 'We still need the acoustic note and the updated cost range before deciding.',
    needsReview: true,
  },
  {
    id: 'segment-4',
    startSeconds: 81,
    startLabel: '00:01:21',
    speaker: 'Speaker 3',
    text: 'I will circulate both items before Thursday afternoon.',
    needsReview: false,
  },
];

const sampleProtocol: ProtocolDraft = {
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

- Prepare the acoustic note and updated cost range before Thursday afternoon.
- Carry both viable options into the next review.
`,
  savedAt: '10:42',
  styleId: 'style-formal',
};

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
  private snapshot: WorkflowSnapshot = {
    projects: [
      {
        id: 'project-harbor-canopy',
        name: 'Harbor Canopy Study',
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
      'meeting-envelope-options': structuredClone(sampleTranscript),
      'meeting-access-review': structuredClone(sampleTranscript),
    },
    protocols: { 'meeting-envelope-options': structuredClone(sampleProtocol) },
    styles: [
      {
        id: 'style-formal',
        name: 'Formal minutes',
        description: 'Structured record of discussion, decisions, and actions.',
        language: 'English',
      },
      {
        id: 'style-working-note',
        name: 'Internal working note',
        description: 'Concise working record for an internal project team.',
        language: 'English',
      },
      {
        id: 'style-decision-log',
        name: 'Technical decision log',
        description: 'Emphasises alternatives, constraints, and explicit decisions.',
        language: 'English',
      },
    ],
    vocabulary: [
      {
        id: 'vocab-1',
        term: 'serviceability',
        category: 'Technical term',
        scope: 'Global',
        projectId: null,
      },
      {
        id: 'vocab-2',
        term: 'junction detail',
        category: 'Building part',
        scope: 'Project',
        projectId: 'project-harbor-canopy',
      },
      {
        id: 'vocab-3',
        term: 'acoustic note',
        category: 'Document',
        scope: 'Project',
        projectId: 'project-harbor-canopy',
      },
    ],
    jobs: [],
    activeJob: null,
    nextJobOutcome: 'success',
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

  async startTranscription(meetingId: string): Promise<void> {
    this.startJob(meetingId, 'transcription');
  }

  async generateProtocol(meetingId: string): Promise<void> {
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

  async updateTranscriptSegment(meetingId: string, segmentId: string, text: string): Promise<void> {
    const segment = this.snapshot.transcripts[meetingId]?.find(
      (candidate) => candidate.id === segmentId,
    );
    if (!segment) return;
    segment.text = text;
    segment.needsReview = false;
    this.emit();
  }

  async updateSpeaker(meetingId: string, speaker: string, replacement: string): Promise<void> {
    const nextSpeaker = replacement.trim();
    if (!nextSpeaker) return;
    for (const segment of this.snapshot.transcripts[meetingId] ?? []) {
      if (segment.speaker === speaker) segment.speaker = nextSpeaker;
    }
    this.emit();
  }

  async updateProtocol(meetingId: string, markdown: string): Promise<void> {
    const protocol = this.snapshot.protocols[meetingId];
    if (!protocol) return;
    protocol.markdown = markdown;
    protocol.savedAt = new Intl.DateTimeFormat('en', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    }).format(new Date());
    const meeting = this.findMeeting(meetingId);
    if (meeting.lifecycle === 'reviewed') meeting.lifecycle = 'protocol_draft';
    this.emit();
  }

  async markReviewed(meetingId: string): Promise<void> {
    const meeting = this.findMeeting(meetingId);
    if (meeting.lifecycle === 'protocol_draft') meeting.lifecycle = 'reviewed';
    this.emit();
  }

  async setNextJobOutcome(outcome: FakeJobOutcome): Promise<void> {
    this.snapshot.nextJobOutcome = outcome;
    this.emit();
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
      this.snapshot.transcripts[job.meetingId] = structuredClone(sampleTranscript);
    }
    if (job.kind === 'generation') {
      this.snapshot.protocols[job.meetingId] = {
        ...structuredClone(sampleProtocol),
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
