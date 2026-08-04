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
  startMs: number;
  endMs: number;
  speaker: string;
  text: string;
  needsReview: boolean;
}

export interface TranscriptDocument {
  schemaVersion: number;
  meetingId: string;
  revisionId: string;
  language: string;
  segments: TranscriptSegment[];
  baseRevisionId: string;
  isDirty: boolean;
  saveState: 'saved' | 'failed';
  savedAtMs: number;
}

export interface ProtocolRevisionSummary {
  id: string;
  ordinal: number;
  status: 'draft' | 'reviewed';
  createdAtMs: number;
}

export interface ProtocolDraft {
  meetingId: string;
  revisionId: string;
  transcriptRevisionId: string;
  markdown: string;
  styleId: string;
  reviewState: 'draft' | 'reviewed' | 'changed_since_review';
  isDirty: boolean;
  saveState: 'saved' | 'failed';
  savedAtMs: number;
  revisions: ProtocolRevisionSummary[];
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
  transcripts: Record<string, TranscriptDocument>;
  protocols: Record<string, ProtocolDraft>;
  styles: ProtocolStyle[];
  vocabulary: VocabularyEntry[];
  jobs: ActiveJob[];
  activeJob: ActiveJob | null;
  nextJobOutcome: FakeJobOutcome;
  activeMeetingId: string | null;
  activeRoute: 'meeting' | 'transcript' | 'protocol' | null;
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

export interface TranscriptionRuntimeStatus {
  executablePath: string | null;
  modelPath: string | null;
  executableFound: boolean;
  modelFound: boolean;
  runtimeVersion: string | null;
  modelDigest: string | null;
  modelByteCount: number | null;
}

// The user chooses a quality; the exact model stays an Advanced detail.
export type TranscriptionPreset = 'fast' | 'balanced' | 'accurate';

export interface TranscriptionPresetStatus {
  preset: TranscriptionPreset;
  modelId: string;
  byteCount: number;
  installed: boolean;
}

export interface TranscriptionCapability {
  selectedPreset: TranscriptionPreset;
  presets: TranscriptionPresetStatus[];
}

export interface ModelDownloadProgress {
  modelId: string;
  percent: number;
}

export interface ModelDownloadError {
  modelId: string;
  message: string;
}

export interface MeetingAudio {
  /** A webview-playable URL for the meeting's working audio. */
  source: string;
  durationMs: number | null;
}

export interface ProtocolProviderModel {
  name: string;
  size: number;
  digest: string;
}

export interface ProtocolProviderStatus {
  endpoint: string;
  serverReachable: boolean;
  runtimeVersion: string | null;
  models: ProtocolProviderModel[];
  selectedModel: string | null;
  selectedModelDigest: string | null;
  selectedModelReady: boolean;
  message: string;
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
  createProtocolRevision(meetingId: string): Promise<void>;
  markReviewed(meetingId: string): Promise<void>;
  restoreProtocolRevision(meetingId: string, revisionId: string): Promise<void>;
  saveWorkspaceLocation(
    meetingId: string,
    route: 'meeting' | 'transcript' | 'protocol',
  ): Promise<void>;
  setNextJobOutcome(outcome: FakeJobOutcome): Promise<void>;
  getTranscriptionRuntimeStatus(): Promise<TranscriptionRuntimeStatus>;
  configureTranscriptionRuntime(executablePath: string): Promise<TranscriptionRuntimeStatus>;
  /** Working audio for transcript review, or null until it exists. */
  getMeetingAudio(meetingId: string): Promise<MeetingAudio | null>;
  getTranscriptionCapability(): Promise<TranscriptionCapability>;
  setTranscriptionPreset(preset: TranscriptionPreset): Promise<TranscriptionCapability>;
  downloadTranscriptionModel(modelId: string): Promise<void>;
  cancelTranscriptionDownload(modelId: string): Promise<void>;
  removeTranscriptionModel(modelId: string): Promise<TranscriptionCapability>;
  /** Live download progress, completion, and failure. Returns an unsubscribe function. */
  subscribeModelEvents(handlers: {
    onProgress: (progress: ModelDownloadProgress) => void;
    onChanged: (capability: TranscriptionCapability) => void;
    onError: (error: ModelDownloadError) => void;
  }): () => void;
  exportProtocol(meetingId: string, format: 'markdown' | 'text', title: string): Promise<boolean>;
  getProtocolProviderStatus(): Promise<ProtocolProviderStatus>;
  configureProtocolProvider(model: string | null): Promise<ProtocolProviderStatus>;
}
