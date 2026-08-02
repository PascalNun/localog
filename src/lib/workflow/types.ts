// Stable document readiness and transient execution are deliberately separate domain axes.
export type MeetingLifecycle =
  'draft' | 'source_ready' | 'transcript_ready' | 'protocol_draft' | 'reviewed' | 'archived';

export type JobState =
  'queued' | 'running' | 'cancelling' | 'failed' | 'cancelled' | 'interrupted' | 'completed';
export type JobOutcome = 'succeeded' | 'cancelled';
export type JobKind = 'import' | 'transcription' | 'generation';
export type FakeJobOutcome = 'success' | 'failure';

export type AppRoute =
  | { name: 'start' }
  | { name: 'project'; projectId: string }
  | { name: 'new-project'; returnToImport: boolean }
  | { name: 'new-meeting'; projectId: string | null }
  | { name: 'meeting'; meetingId: string }
  | { name: 'transcript'; meetingId: string }
  | { name: 'protocol'; meetingId: string }
  | { name: 'styles' }
  | { name: 'vocabulary' }
  | { name: 'settings' };

export interface ProjectSummary {
  id: string;
  name: string;
  description: string;
  meetingCount: number;
  defaultLanguage: string;
  defaultStyleId: string;
}

export interface MeetingSummary {
  id: string;
  projectId: string;
  title: string;
  occurredAt: string;
  durationLabel: string | null;
  lifecycle: MeetingLifecycle;
  language: string;
  sourceName: string | null;
  sourceByteCount: number | null;
  sourceMediaType: string | null;
  styleId: string;
}

export interface TranscriptSegment {
  id: string;
  startSeconds: number;
  startLabel: string;
  speaker: string;
  text: string;
  needsReview: boolean;
}

export interface ProtocolDraft {
  markdown: string;
  savedAt: string;
  styleId: string;
}

export interface ProtocolStyle {
  id: string;
  name: string;
  description: string;
  language: string;
}

export interface VocabularyEntry {
  id: string;
  term: string;
  category: string;
  scope: 'Global' | 'Project';
  projectId: string | null;
}

export interface ActiveJob {
  id: string;
  meetingId: string;
  kind: JobKind;
  state: JobState;
  outcome: JobOutcome | null;
  progress: number;
  progressBytes: number;
  totalBytes: number | null;
  stage: string;
  attempt: number;
  error: { code: string; title: string; detail: string } | null;
  requiresDuplicateConfirmation: boolean;
}

export interface WorkflowSnapshot {
  projects: ProjectSummary[];
  meetings: MeetingSummary[];
  transcripts: Record<string, TranscriptSegment[]>;
  protocols: Record<string, ProtocolDraft>;
  styles: ProtocolStyle[];
  vocabulary: VocabularyEntry[];
  jobs: ActiveJob[];
  activeJob: ActiveJob | null;
  nextJobOutcome: FakeJobOutcome;
}

export interface NewProjectInput {
  name: string;
  description: string;
  defaultLanguage: string;
}

export interface NewMeetingInput {
  projectId: string;
  title: string;
  occurredAt: string;
  language: string;
  sourceName: string;
  sourcePath: string | null;
  styleId: string;
}

export interface SourceSelection {
  name: string;
  path: string;
}

// UI code depends on this contract; fake and real adapters must preserve the same semantics.
export interface WorkflowBridge {
  getSnapshot(): Promise<WorkflowSnapshot>;
  subscribe(
    listener: (snapshot: WorkflowSnapshot) => void,
    onError?: (message: string) => void,
  ): () => void;
  createProject(input: NewProjectInput): Promise<ProjectSummary>;
  createMeeting(input: NewMeetingInput): Promise<MeetingSummary>;
  importRecording(meetingId: string): Promise<void>;
  startTranscription(meetingId: string): Promise<void>;
  generateProtocol(meetingId: string): Promise<void>;
  cancelActiveJob(meetingId: string): Promise<void>;
  retryActiveJob(meetingId: string): Promise<void>;
  confirmDuplicateImport(meetingId: string): Promise<void>;
  reselectImportSource(meetingId: string): Promise<void>;
  updateMeetingTitle(meetingId: string, title: string): Promise<void>;
  updateTranscriptSegment(meetingId: string, segmentId: string, text: string): Promise<void>;
  updateSpeaker(meetingId: string, speaker: string, replacement: string): Promise<void>;
  updateProtocol(meetingId: string, markdown: string): Promise<void>;
  markReviewed(meetingId: string): Promise<void>;
  setNextJobOutcome(outcome: FakeJobOutcome): Promise<void>;
}
