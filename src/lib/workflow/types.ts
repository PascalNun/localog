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
  | { name: 'new-meeting'; projectId: string | null; forRecording?: boolean }
  | { name: 'meeting'; meetingId: string }
  | { name: 'recording-review'; meetingId: string }
  | { name: 'recording'; meetingId: string }
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
  appearance: DocumentAppearance;
  furniture: PageFurniture;
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

/**
 * A word the transcriber never got right, offered as a possible name.
 *
 * Not every candidate is a name — the filter keeps words the transcriber was unsure
 * of every time it heard them, which catches mis-heard proper nouns along with some
 * ordinary words it simply fumbled. A short list somebody scans beats a long one they
 * abandon, and anything missed is caught next meeting.
 */
export interface NameCandidate {
  /** The spelling as the transcript has it. */
  heard: string;
  /** How many times it occurs. */
  occurrences: number;
  /** One place it appears, so it can be recognised without hunting for it. */
  context: string;
}

/**
 * Somebody who said their own name near the start of a meeting.
 *
 * The spelling is the transcript's, however wrong — that is what makes the list
 * useful. Somebody who was there recognises "Person A" at once, and the
 * wrong spelling is what a correction has to match to find it.
 */
export interface Introduction {
  heard: string;
  /** What they said they do, in their own words. */
  role: string;
  /** The sentence they said it in. */
  context: string;
}

/** One file inside a backup, with what it should be when it comes back. */
export interface BackupFile {
  path: string;
  byteCount: number;
  sha256: string;
}

/**
 * What a backup says about itself.
 *
 * Read before restoring so somebody can be shown what they are about to replace
 * their work with. The counts are of what the backup holds, not of what is here
 * now — the whole point is that those differ.
 */
export interface BackupManifest {
  format: number;
  createdAtMs: number;
  applicationVersion: string;
  database: BackupFile;
  files: BackupFile[];
  projectCount: number;
  meetingCount: number;
  /** Always true today. Said in the manifest so it can be shown, not assumed. */
  excludesModels: boolean;
  folderName: string;
}

/** What a restore did. */
export interface RestoreOutcome {
  projectCount: number;
  meetingCount: number;
  /** Where the replaced workspace was kept. Nothing is deleted. */
  previousWorkspace: string;
}

/**
 * What this machine will let the recorder capture, asked before a meeting.
 *
 * The values are the recorder's own words rather than a union defined here. Two
 * more recorders have to be written, for Linux and for Windows, and a closed set
 * on this side would have to be widened for each by somebody who cannot see what
 * the new recorder answers.
 */
export interface RecordingPermissions {
  /** "granted", or "not-granted" — never-asked and refused look the same. */
  systemAudio: string;
  /** "granted", "denied", "restricted", or "undetermined". */
  microphone: string;
  /** Set when the question could not be put at all, which is not a refusal. */
  unavailable?: string | null;
}

/** What a recording in progress is doing. */
export interface RecordingStatus {
  /** Whether this machine has a recorder at all. */
  available: boolean;
  recording: boolean;
  meetingId: string | null;
  seconds: number;
  /** Loudest sample in the last second, 0 to 1. */
  systemPeak: number;
  microphonePeak: number;
  /** The recorder stopped without being asked to. */
  stoppedUnexpectedly: boolean;
}

/** One place a correction would apply. */
export interface CorrectionMatch {
  segmentId: string;
  startMs: number;
  /** The sentence around it, because some wrong spellings are ordinary words. */
  context: string;
}

/**
 * What a correction did, which can differ from what was asked of it: a place whose
 * text has moved since the review is skipped rather than corrected blindly.
 */
export interface AppliedCorrection {
  wrong: string;
  right: string;
  /** Segment ids of the occurrences to change. Empty means every one of them. */
  keptSegmentIds: string[];
  /** Whether to remember the spelling for the project's future meetings. */
  remember: boolean;
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
 * What somebody asked for about speakers. Three answers rather than a number that
 * might be missing: leaving them together is a choice, not an absence of one, and
 * the separation pass must not run because the models happen to be installed.
 *
 * `'estimate'` is only offered where the runtime can work the number out, which
 * the diariser could not — it answered by re-reading the audio, so every count
 * cost another pass and had to be decided in advance.
 */
export type SpeakerRequest = 'together' | 'estimate' | number;

/**
 * What the run that wrote a draft found out about its own result, established
 * without a model and without a reader.
 *
 * Shown as evidence to look at, never as a verdict. A machine judgement placed in
 * front of a person asks them to read less carefully, and reading carefully is the
 * only check in this product that reliably works.
 */
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

/**
 * How a project's protocols are set — as opposed to what they say.
 *
 * Deliberately separate from the protocol style: a style decides what belongs in the
 * document, this decides how it looks. Every value is one of a short list rather than
 * a number, so a protocol cannot end up at 11.5pt Helvetica by accident.
 */
export interface DocumentAppearance {
  font: 'barlow' | 'georgia' | 'times-new-roman' | 'arial' | 'calibri';
  /** Body size in points, as it will print. */
  bodySize: number;
  headingScale: 'compact' | 'standard' | 'large';
  lineSpacing: 'compact' | 'comfortable' | 'spacious';
  pageWidth: 'narrow' | 'standard' | 'wide' | 'a4';
}

/**
 * What repeats at the top and bottom of every printed page.
 *
 * A document property rather than body content: it is not part of what the meeting
 * said. Each slot is a list of fields rather than free text, so that "Page 3 of 6"
 * can be counted rather than typed; anything the list does not cover goes in as a
 * `text` field.
 */
export type FurnitureField =
  | { kind: 'projectName' }
  | { kind: 'meetingTitle' }
  | { kind: 'meetingDate' }
  | { kind: 'documentType' }
  | { kind: 'protocolStatus' }
  | { kind: 'pageNumber' }
  | { kind: 'pageOfCount' }
  | { kind: 'text'; value: string };

export interface FurnitureRow {
  left: FurnitureField[];
  centre: FurnitureField[];
  right: FurnitureField[];
}

export interface PageFurniture {
  header: FurnitureRow;
  footer: FurnitureRow;
  /** A title page usually carries its own heading and wants nothing repeated on it. */
  skipFirstPage: boolean;
}

/**
 * A rewritten passage, and what checking it found.
 *
 * `missingFigures` are numbers that were in the passage and are not in what came
 * back. Measured rather than trusted: on a real German passage the small local model
 * altered a fact in three of twenty-four rewrites, and a protocol whose figures
 * drift is worse than one nobody rewrote.
 */
export interface RefinedPassage {
  text: string;
  missingFigures: string[];
  /**
   * What a second model pass thought the rewrite changed about the facts.
   *
   * Empty when it found nothing and also when no model was asked; `checked` says
   * which. A hint on a change somebody is already reading, never a verdict — it
   * misses things too.
   */
  noticedChanges: string[];
  checked: boolean;
}

/**
 * A saved way of presenting a protocol.
 *
 * The typography and the running header and footer, named. Not the protocol style:
 * that decides what the document says, this decides how it is set.
 */
export interface AppearancePreset {
  id: string;
  name: string;
  description: string;
  appearance: DocumentAppearance;
  furniture: PageFurniture;
  /** One that shipped, which can be used and copied but not overwritten. */
  builtIn: boolean;
}

export interface NameMatch {
  line: number;
  context: string;
  matched: string;
  replacement: string;
}

export interface NameReplacement {
  matches: NameMatch[];
  markdown: string;
}

/** A section taken out of a protocol and kept in case it is wanted back. */
export interface SetAsideSection {
  title: string;
  /** The whole block, heading and all, exactly as it was. */
  markdown: string;
}

export const EMPTY_FURNITURE: PageFurniture = {
  header: { left: [], centre: [], right: [] },
  footer: { left: [], centre: [], right: [] },
  skipFirstPage: false,
};

export const DEFAULT_APPEARANCE: DocumentAppearance = {
  font: 'barlow',
  bodySize: 11,
  headingScale: 'standard',
  lineSpacing: 'comfortable',
  pageWidth: 'a4',
};

/**
 * What is known about speaker separation before anything has been asked.
 *
 * Every field says no, which is the safe reading: an interface that assumed the
 * models were there would offer separation and fail at the point of use. Three
 * components and the bridge each wrote this object out.
 */
export const SPEAKER_SEPARATION_UNREADY: SpeakerSeparationStatus = {
  modelsInstalled: false,
  runtimeConfigured: false,
  runtimeHealthy: false,
  runtimeVersion: null,
  runtimePath: null,
  downloadBytes: 0,
};

/**
 * The parts of a protocol style a person may change.
 *
 * One object rather than four positional arguments, because `name` and
 * `description` are both strings and sit next to each other: swapping them at a
 * call site compiles, and produces a style whose name is its description. The
 * fidelity rules are deliberately absent — they are not the author's to edit.
 */
export interface StyleEdit {
  name: string;
  description: string;
  instructions: string[];
  density: ProtocolDensity;
}

/**
 * What each transcription preset is called, and what choosing it means.
 *
 * The three names were spelled out twice — on the settings screen where a preset
 * is chosen, and on the meeting screen where the chosen one is reported — so
 * renaming one left the other calling it by the old word.
 */
export const PRESET_LABELS: Record<TranscriptionPreset, { name: string; detail: string }> = {
  fast: { name: 'Fast', detail: 'Quick drafts, lightest on memory' },
  balanced: { name: 'Balanced', detail: 'Everyday meetings' },
  accurate: { name: 'Accurate', detail: 'Best quality, slowest' },
};

export interface ProtocolStyle {
  id: string;
  name: string;
  description: string;
  language: string;
  density: ProtocolDensity;
}

/**
 * A protocol style read in full, rather than the three sentences the list carries.
 *
 * `instructions` is what the style actually asks the model for, in the order it asks.
 * `asShipped` says the style is still the one that came with the application.
 */
export interface ProtocolStyleDetail {
  id: string;
  name: string;
  description: string;
  density: ProtocolDensity;
  /** What this style asks for, which is the part that can be changed. */
  instructions: string[];
  /**
   * What is actually checked when a protocol is written.
   *
   * Replaced `requiredSections`, which held English section names while the protocol
   * is written in the meeting's language and could therefore never be checked.
   * Showing it as "sections it must produce" claimed a guarantee this application
   * does not make.
   */
  checks: string[];
  asShipped: boolean;
  /**
   * The rules every style carries and none may change.
   *
   * They are not stored with the style: they live in the code and are added to every
   * protocol as it is written, so editing a style cannot reach them. Shown here so
   * they read as a promise rather than as something quietly enforced.
   */
  fidelity: string[];
  /** The shipped styles are copied rather than edited. */
  editable: boolean;
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
  /**
   * What the machine actually has, read by the backend. Null where it cannot be
   * established — never from the browser, which cannot tell in this shell.
   */
  machineMemoryGb: number | null;
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
  startImport(meetingId: string): Promise<void>;
  startTranscription(meetingId: string, speakers?: SpeakerRequest): Promise<void>;
  startGeneration(meetingId: string): Promise<void>;
  cancelActiveJob(meetingId: string): Promise<void>;
  retryActiveJob(meetingId: string): Promise<void>;
  confirmDuplicateImport(meetingId: string): Promise<void>;
  reselectImportSource(meetingId: string): Promise<void>;
  updateMeetingTitle(meetingId: string, title: string): Promise<void>;
  updateMeetingLanguage(meetingId: string, language: string): Promise<void>;
  updateTranscriptSegment(meetingId: string, segmentId: string, text: string): Promise<void>;
  deleteTranscriptSegment(meetingId: string, segmentId: string): Promise<void>;
  renameTranscriptSpeaker(meetingId: string, speaker: string, replacement: string): Promise<void>;
  /** Files dropped onto the window. Returns an unsubscribe function. */
  subscribeFileDrops(handler: (event: FileDropEvent) => void): () => void;
  protocolStyleDetail(styleId: string): Promise<ProtocolStyleDetail>;
  duplicateProtocolStyle(styleId: string, name: string): Promise<void>;
  updateProtocolStyle(styleId: string, edit: StyleEdit): Promise<void>;
  deleteProtocolStyle(styleId: string): Promise<void>;
  /** Delete a meeting, its recordings, its transcript and its protocols. */
  deleteMeeting(meetingId: string): Promise<void>;
  setProjectAppearance(projectId: string, appearance: DocumentAppearance): Promise<void>;
  setProjectFurniture(projectId: string, furniture: PageFurniture): Promise<void>;
  /**
   * What replacing a name through a protocol would do.
   *
   * Uses the same rule as the transcript corrections, so a capitalised name is found
   * in its compound form too — German writes the interior of a compound in lower
   * case, and a literal replace walks past it.
   */
  previewNameReplacement(text: string, wrong: string, right: string): Promise<NameReplacement>;
  appearancePresets(): Promise<AppearancePreset[]>;
  saveAppearancePreset(
    name: string,
    description: string,
    appearance: DocumentAppearance,
    furniture: PageFurniture,
  ): Promise<AppearancePreset[]>;
  deleteAppearancePreset(presetId: string): Promise<AppearancePreset[]>;
  applyAppearancePreset(projectId: string, presetId: string): Promise<void>;
  /** Sections taken out of a protocol and kept in case they are wanted back. */
  protocolSetAside(meetingId: string): Promise<SetAsideSection[]>;
  /** The document and the stash together, because they are one change. */
  setProtocolSections(
    meetingId: string,
    markdown: string,
    setAside: SetAsideSection[],
  ): Promise<void>;
  /** Rewrite one passage as asked, returning the new text without storing it. */
  refinePassage(meetingId: string, passage: string, instruction: string): Promise<RefinedPassage>;
  /** Copy the workspace into a folder. Returns what was written. */
  createBackup(parent: string, folderName: string): Promise<BackupManifest>;
  /** What a folder claims to be, without reading its files. */
  inspectBackup(folder: string): Promise<BackupManifest>;
  /** Put a verified backup back, keeping what it replaced. */
  restoreBackup(folder: string): Promise<RestoreOutcome>;
  /** What the machine will allow, asked when the record screen opens. */
  recordingPermissions(): Promise<RecordingPermissions>;
  /** Open the System Settings pane where a recording permission is granted. */
  openPrivacySettings(pane: 'screen' | 'microphone'): Promise<void>;
  /** What a recording in progress is doing. Cheap; polled while the screen is open. */
  recordingStatus(): Promise<RecordingStatus>;
  startRecording(meetingId: string): Promise<void>;
  stopRecording(): Promise<void>;
  /**
   * Who introduced themselves at the start of a meeting. Model work of about a
   * minute, so it runs when somebody asks for it.
   */
  findIntroductions(meetingId: string): Promise<Introduction[]>;
  /** Words the transcriber was never sure of, most likely first. */
  findNameCandidates(meetingId: string): Promise<NameCandidate[]>;
  /** Every place a correction would apply, with enough sentence to judge it. */
  previewCorrection(meetingId: string, wrong: string, right: string): Promise<CorrectionMatch[]>;
  /**
   * Apply the kept occurrences and remember the spelling.
   *
   * One action, two outcomes: this transcript is repaired, and the next meeting is
   * transcribed correctly because the term joins the project's names.
   */
  applyCorrection(meetingId: string, correction: AppliedCorrection): Promise<number>;
  saveVocabularyEntry(entry: VocabularyDraft): Promise<void>;
  deleteVocabularyEntry(entryId: string): Promise<void>;
  autosaveProtocol(meetingId: string, markdown: string): Promise<void>;
  createProtocolRevision(meetingId: string): Promise<void>;
  markProtocolReviewed(meetingId: string): Promise<void>;
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
  /** The platform's own print panel, where the webview's does nothing. */
  nativePrint(): (() => Promise<void>) | undefined;
  /** Save a document the interface built — a Word file, for now. */
  exportProtocolBytes(
    contents: Uint8Array,
    title: string,
    extension: string,
    formatName: string,
  ): Promise<boolean>;
  getProtocolProviderStatus(): Promise<ProtocolProviderStatus>;
  configureProtocolProvider(model: string | null): Promise<ProtocolProviderStatus>;
}
