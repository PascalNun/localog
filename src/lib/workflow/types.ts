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
  | { name: 'recording-review'; meetingId: string }
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
  /** Words the transcription model itself was unsure of. Absent on older transcripts. */
  uncertainWords?: string[];
}

export interface TranscriptDocument {
  schemaVersion: number;
  meetingId: string;
  revisionId: string;
  language: string;
  speakerResolution: 'unknown' | 'unavailable' | 'failed' | 'resolved';
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

/**
 * What the run that wrote a draft found out about its own result, established
 * without a model and without a reader.
 *
 * Shown as evidence to look at, never as a verdict. A machine judgement placed in
 * front of a person asks them to read less carefully, and reading carefully is the
 * only check in this product that reliably works.
 */
/**
 * What somebody asked for about speakers. Three answers rather than a number that
 * might be missing: leaving them together is a choice, not an absence of one, and
 * the separation pass must not run because the models happen to be installed.
 *
 * `'estimate'` is only offered where the runtime can work the number out, which
 * the diariser could not — it answered by re-reading the audio, so every count
 * cost another pass and had to be decided in advance.
 */
export type SpeakerRequest = 'together' | 'estimate' | number;

export interface ProtocolEvidence {
  quantitiesStated: number;
  quantitiesAccounted: number;
  quantitiesInvented: string[];
  /** Tasks recorded with nobody against them. Absent from drafts written before
   * this was measured, so treat an undefined value as "not looked at". */
  tasksUnowned?: string[];
  charactersSpoken: number;
  charactersWritten: number;
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
  /** Absent for drafts written before this was recorded. */
  evidence?: ProtocolEvidence | null;
}

export type ProtocolDensity = 'comprehensive' | 'concise' | 'terse';

export interface ProtocolStyle {
  id: string;
  name: string;
  description: string;
  language: string;
  density: ProtocolDensity;
}

/** A file being dragged over the window, or let go of. */
export type FileDropEvent =
  { kind: 'over' } | { kind: 'leave' } | { kind: 'dropped'; paths: string[] };

export interface VocabularyDraft {
  id: string | null;
  term: string;
  category: string;
  scope: 'Global' | 'Project';
  projectId: string | null;
  enabled: boolean;
}

export interface VocabularyEntry {
  id: string;
  term: string;
  category: string;
  scope: 'Global' | 'Project';
  projectId: string | null;
  enabled: boolean;
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

/** Readiness of optional speaker separation; it never blocks transcription. */
export interface SpeakerSeparationStatus {
  modelsInstalled: boolean;
  runtimeConfigured: boolean;
  runtimeHealthy: boolean;
  runtimeVersion: string | null;
  runtimePath: string | null;
  downloadBytes: number;
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

/** A stretch of a recording, in milliseconds from its start. */
export interface RecordingSpan {
  fromMs: number;
  toMs: number;
}

/**
 * What somebody decided to leave out of a recording. Held as trims plus removals
 * because that is how a person describes it, and because the review screen shows
 * them back as separate, undoable decisions.
 */
export interface RecordingEdits {
  startMs: number;
  endMs?: number | null;
  removed?: RecordingSpan[];
}

/** Everything the review screen needs before a recording is transcribed. */
export interface RecordingReview {
  durationMs: number;
  /** Peaks from zero to one, for drawing. */
  waveform: number[];
  edits: RecordingEdits;
  keptDurationMs: number;
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
  startTranscription(meetingId: string, speakers?: SpeakerRequest): Promise<void>;
  generateProtocol(meetingId: string): Promise<void>;
  cancelActiveJob(meetingId: string): Promise<void>;
  retryActiveJob(meetingId: string): Promise<void>;
  confirmDuplicateImport(meetingId: string): Promise<void>;
  reselectImportSource(meetingId: string): Promise<void>;
  updateMeetingTitle(meetingId: string, title: string): Promise<void>;
  updateMeetingLanguage(meetingId: string, language: string): Promise<void>;
  updateTranscriptSegment(meetingId: string, segmentId: string, text: string): Promise<void>;
  deleteTranscriptSegment(meetingId: string, segmentId: string): Promise<void>;
  updateSpeaker(meetingId: string, speaker: string, replacement: string): Promise<void>;
  /** Files dropped onto the window. Returns an unsubscribe function. */
  subscribeFileDrops(handler: (event: FileDropEvent) => void): () => void;
  saveVocabularyEntry(entry: VocabularyDraft): Promise<void>;
  deleteVocabularyEntry(entryId: string): Promise<void>;
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
  getSpeakerSeparationStatus(): Promise<SpeakerSeparationStatus>;
  configureSpeakerRuntime(executablePath: string): Promise<SpeakerSeparationStatus>;
  downloadSpeakerModels(): Promise<void>;
  subscribeSpeakerEvents(handler: (status: SpeakerSeparationStatus) => void): () => void;
  /** Working audio for transcript review, or null until it exists. */
  getMeetingAudio(meetingId: string): Promise<MeetingAudio | null>;
  getRecordingReview(meetingId: string): Promise<RecordingReview | null>;
  setRecordingEdits(meetingId: string, edits: RecordingEdits): Promise<void>;
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
