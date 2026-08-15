import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import type {
  ActiveJob,
  MeetingSummary,
  NewMeetingInput,
  NewProjectInput,
  ProjectSummary,
  ProtocolDraft,
  ProtocolProviderStatus,
  ProtocolStyle,
  SourceSelection,
  SpeakerRequest,
  TranscriptDocument,
  MeetingAudio,
  RecordingEdits,
  RecordingReview,
  TranscriptionCapability,
  TranscriptionPreset,
  TranscriptionRuntimeStatus,
  SpeakerSeparationStatus,
  ModelDownloadError,
  ModelDownloadProgress,
  VocabularyDraft,
  VocabularyEntry,
  FileDropEvent,
} from './types';

export interface WorkspaceData {
  projects: ProjectSummary[];
  meetings: MeetingSummary[];
  jobs: ActiveJob[];
  transcripts: Record<string, TranscriptDocument>;
  protocols: Record<string, ProtocolDraft>;
  styles: ProtocolStyle[];
  vocabulary: VocabularyEntry[];
  activeMeetingId: string | null;
  activeRoute: 'meeting' | 'transcript' | 'protocol' | null;
}

/**
 * Narrow persistence port for the durable workflow. Heavy file and model work
 * remains behind Rust commands while the browser fallback stays deterministic.
 */
export interface WorkspaceStore {
  loadWorkspace(): Promise<WorkspaceData>;
  createProject(input: NewProjectInput): Promise<ProjectSummary>;
  createMeeting(input: NewMeetingInput): Promise<MeetingSummary>;
  updateMeetingTitle(meetingId: string, title: string): Promise<void>;
  updateMeetingLanguage?: (meetingId: string, language: string) => Promise<void>;
  selectMediaSource(): Promise<SourceSelection | null>;
  startImport(meetingId: string): Promise<void>;
  cancelImport(meetingId: string): Promise<void>;
  retryImport(meetingId: string, allowDuplicate: boolean): Promise<void>;
  replaceImportSource(meetingId: string, source: SourceSelection): Promise<void>;
  startTranscription(
    meetingId: string,
    failRequested: boolean,
    speakers?: SpeakerRequest,
  ): Promise<void>;
  startGeneration(meetingId: string, failRequested: boolean): Promise<void>;
  cancelProcessing(meetingId: string): Promise<void>;
  retryProcessing(meetingId: string): Promise<void>;
  updateTranscriptSegment(
    meetingId: string,
    segmentId: string,
    text: string,
  ): Promise<WorkspaceData>;
  renameTranscriptSpeaker(
    meetingId: string,
    speaker: string,
    replacement: string,
  ): Promise<WorkspaceData>;
  subscribeFileDrops(handler: (event: FileDropEvent) => void): () => void;
  saveVocabularyEntry(entry: VocabularyDraft): Promise<WorkspaceData>;
  deleteVocabularyEntry(entryId: string): Promise<WorkspaceData>;
  autosaveProtocol(meetingId: string, markdown: string): Promise<WorkspaceData>;
  createProtocolRevision(meetingId: string): Promise<WorkspaceData>;
  markProtocolReviewed(meetingId: string): Promise<WorkspaceData>;
  restoreProtocolRevision(meetingId: string, revisionId: string): Promise<WorkspaceData>;
  saveWorkspaceLocation(
    meetingId: string,
    route: 'meeting' | 'transcript' | 'protocol',
  ): Promise<void>;
  subscribe(listener: (workspace: WorkspaceData) => void): Promise<UnlistenFn>;
  getTranscriptionRuntimeStatus?: () => Promise<TranscriptionRuntimeStatus>;
  configureTranscriptionRuntime?: (executablePath: string) => Promise<TranscriptionRuntimeStatus>;
  getSpeakerSeparationStatus?: () => Promise<SpeakerSeparationStatus>;
  configureSpeakerRuntime?: (executablePath: string) => Promise<SpeakerSeparationStatus>;
  downloadSpeakerModels?: () => Promise<void>;
  subscribeSpeakerEvents?: (handler: (status: SpeakerSeparationStatus) => void) => () => void;
  getMeetingAudio?: (meetingId: string) => Promise<MeetingAudio | null>;
  getTranscriptionCapability?: () => Promise<TranscriptionCapability>;
  setTranscriptionPreset?: (preset: TranscriptionPreset) => Promise<TranscriptionCapability>;
  downloadTranscriptionModel?: (modelId: string) => Promise<void>;
  cancelTranscriptionDownload?: (modelId: string) => Promise<void>;
  removeTranscriptionModel?: (modelId: string) => Promise<TranscriptionCapability>;
  subscribeModelEvents?: (handlers: {
    onProgress: (progress: ModelDownloadProgress) => void;
    onChanged: (capability: TranscriptionCapability) => void;
    onError: (error: ModelDownloadError) => void;
  }) => () => void;
  exportProtocol?: (
    meetingId: string,
    format: 'markdown' | 'text',
    title: string,
  ) => Promise<boolean>;
  getProtocolProviderStatus?: () => Promise<ProtocolProviderStatus>;
  configureProtocolProvider?: (model: string | null) => Promise<ProtocolProviderStatus>;
}

class TauriWorkspaceStore implements WorkspaceStore {
  loadWorkspace(): Promise<WorkspaceData> {
    return invoke<WorkspaceData>('load_workspace');
  }

  createProject(input: NewProjectInput): Promise<ProjectSummary> {
    return invoke<ProjectSummary>('create_project', { input });
  }

  createMeeting(input: NewMeetingInput): Promise<MeetingSummary> {
    return invoke<MeetingSummary>('create_meeting', { input });
  }

  updateMeetingTitle(meetingId: string, title: string): Promise<void> {
    return invoke('update_meeting_title', { meetingId, title });
  }

  updateMeetingLanguage(meetingId: string, language: string): Promise<void> {
    return invoke('update_meeting_language', { meetingId, language });
  }

  async selectMediaSource(): Promise<SourceSelection | null> {
    const path = await open({
      multiple: false,
      directory: false,
      title: 'Choose a meeting recording',
      filters: [
        {
          name: 'Audio and video',
          extensions: [
            'wav',
            'mp3',
            'm4a',
            'aac',
            'flac',
            'ogg',
            'opus',
            'mp4',
            'mov',
            'mkv',
            'webm',
          ],
        },
      ],
    });
    if (!path) return null;
    const name = path.split(/[\\/]/).pop();
    return name ? { name, path } : null;
  }

  startImport(meetingId: string): Promise<void> {
    return invoke('start_import', { meetingId });
  }

  cancelImport(meetingId: string): Promise<void> {
    return invoke('cancel_import', { meetingId });
  }

  retryImport(meetingId: string, allowDuplicate: boolean): Promise<void> {
    return invoke('retry_import', { meetingId, allowDuplicate });
  }

  replaceImportSource(meetingId: string, source: SourceSelection): Promise<void> {
    return invoke('replace_import_source', {
      meetingId,
      sourceName: source.name,
      sourcePath: source.path,
    });
  }

  startTranscription(
    meetingId: string,
    failRequested: boolean,
    speakers: SpeakerRequest = 'together',
  ): Promise<void> {
    return invoke('start_transcription', {
      meetingId,
      failRequested,
      separateSpeakers: speakers !== 'together',
      expectedSpeakers: typeof speakers === 'number' ? speakers : null,
    });
  }

  startGeneration(meetingId: string, failRequested: boolean): Promise<void> {
    return invoke('start_generation', { meetingId, failRequested });
  }

  cancelProcessing(meetingId: string): Promise<void> {
    return invoke('cancel_processing', { meetingId });
  }

  retryProcessing(meetingId: string): Promise<void> {
    return invoke('retry_processing', { meetingId });
  }

  updateTranscriptSegment(
    meetingId: string,
    segmentId: string,
    text: string,
  ): Promise<WorkspaceData> {
    return invoke('update_transcript_segment', { meetingId, segmentId, text });
  }

  renameTranscriptSpeaker(
    meetingId: string,
    speaker: string,
    replacement: string,
  ): Promise<WorkspaceData> {
    return invoke('rename_transcript_speaker', { meetingId, speaker, replacement });
  }

  /// Dropping a recording onto the window is how most people expect to start,
  /// and it is also the only way that never asks them to find a file twice. The
  /// webview reports the drag as it happens so the target can show that it will
  /// accept the file, rather than only reacting once it has been let go.
  subscribeFileDrops(handler: (event: FileDropEvent) => void): () => void {
    let stop: UnlistenFn | null = null;
    let cancelled = false;
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === 'enter' || event.payload.type === 'over') {
          handler({ kind: 'over' });
        } else if (event.payload.type === 'leave') {
          handler({ kind: 'leave' });
        } else if (event.payload.type === 'drop') {
          handler({ kind: 'dropped', paths: event.payload.paths });
        }
      })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else stop = unlisten;
      });
    return () => {
      cancelled = true;
      stop?.();
    };
  }

  saveVocabularyEntry(entry: VocabularyDraft): Promise<WorkspaceData> {
    return invoke('save_vocabulary_entry', { entry });
  }

  deleteVocabularyEntry(entryId: string): Promise<WorkspaceData> {
    return invoke('delete_vocabulary_entry', { entryId });
  }

  autosaveProtocol(meetingId: string, markdown: string): Promise<WorkspaceData> {
    return invoke('autosave_protocol', { meetingId, markdown });
  }

  createProtocolRevision(meetingId: string): Promise<WorkspaceData> {
    return invoke('create_protocol_revision', { meetingId });
  }

  markProtocolReviewed(meetingId: string): Promise<WorkspaceData> {
    return invoke('mark_protocol_reviewed', { meetingId });
  }

  restoreProtocolRevision(meetingId: string, revisionId: string): Promise<WorkspaceData> {
    return invoke('restore_protocol_revision', { meetingId, revisionId });
  }

  saveWorkspaceLocation(
    meetingId: string,
    route: 'meeting' | 'transcript' | 'protocol',
  ): Promise<void> {
    return invoke('save_workspace_location', { meetingId, route });
  }

  subscribe(listener: (workspace: WorkspaceData) => void): Promise<UnlistenFn> {
    return listen<WorkspaceData>('workspace://changed', (event) => listener(event.payload));
  }

  getTranscriptionRuntimeStatus(): Promise<TranscriptionRuntimeStatus> {
    return invoke<TranscriptionRuntimeStatus>('transcription_runtime_status');
  }

  configureTranscriptionRuntime(executablePath: string): Promise<TranscriptionRuntimeStatus> {
    return invoke<TranscriptionRuntimeStatus>('configure_transcription_runtime', {
      executablePath,
    });
  }

  getSpeakerSeparationStatus(): Promise<SpeakerSeparationStatus> {
    return invoke<SpeakerSeparationStatus>('speaker_separation_status');
  }

  configureSpeakerRuntime(executablePath: string): Promise<SpeakerSeparationStatus> {
    return invoke<SpeakerSeparationStatus>('configure_speaker_runtime', { executablePath });
  }

  downloadSpeakerModels(): Promise<void> {
    return invoke('download_speaker_models');
  }

  subscribeSpeakerEvents(handler: (status: SpeakerSeparationStatus) => void): () => void {
    let stop: UnlistenFn | null = null;
    let cancelled = false;
    void listen<SpeakerSeparationStatus>('speakers://changed', (event) =>
      handler(event.payload),
    ).then((unlisten) => {
      if (cancelled) unlisten();
      else stop = unlisten;
    });
    return () => {
      cancelled = true;
      stop?.();
    };
  }

  async getRecordingReview(meetingId: string): Promise<RecordingReview | null> {
    return invoke('recording_review', { meetingId, buckets: 2000 });
  }

  async setRecordingEdits(meetingId: string, edits: RecordingEdits): Promise<void> {
    return invoke('set_recording_edits', { meetingId, edits });
  }

  async getMeetingAudio(meetingId: string): Promise<MeetingAudio | null> {
    const found = await invoke<{ path: string; durationMs: number | null } | null>(
      'meeting_audio',
      {
        meetingId,
      },
    );
    if (!found) return null;
    // The asset protocol streams the file with range requests instead of
    // holding a long recording in memory.
    return { source: convertFileSrc(found.path), durationMs: found.durationMs };
  }

  getTranscriptionCapability(): Promise<TranscriptionCapability> {
    return invoke<TranscriptionCapability>('transcription_capability');
  }

  setTranscriptionPreset(preset: TranscriptionPreset): Promise<TranscriptionCapability> {
    return invoke<TranscriptionCapability>('set_transcription_preset', { preset });
  }

  downloadTranscriptionModel(modelId: string): Promise<void> {
    return invoke('download_transcription_model', { modelId });
  }

  cancelTranscriptionDownload(modelId: string): Promise<void> {
    return invoke('cancel_transcription_download', { modelId });
  }

  removeTranscriptionModel(modelId: string): Promise<TranscriptionCapability> {
    return invoke<TranscriptionCapability>('remove_transcription_model', { modelId });
  }

  subscribeModelEvents(handlers: {
    onProgress: (progress: ModelDownloadProgress) => void;
    onChanged: (capability: TranscriptionCapability) => void;
    onError: (error: ModelDownloadError) => void;
  }): () => void {
    const pending = [
      listen<ModelDownloadProgress>('model://progress', (event) =>
        handlers.onProgress(event.payload),
      ),
      listen<TranscriptionCapability>('model://changed', (event) =>
        handlers.onChanged(event.payload),
      ),
      listen<ModelDownloadError>('model://error', (event) => handlers.onError(event.payload)),
    ];
    let stopped = false;
    const unlisteners: UnlistenFn[] = [];
    for (const promise of pending) {
      void promise.then((unlisten) => {
        // Listeners resolve asynchronously; honour an unsubscribe that arrived first.
        if (stopped) unlisten();
        else unlisteners.push(unlisten);
      });
    }
    return () => {
      stopped = true;
      for (const unlisten of unlisteners) unlisten();
    };
  }

  async exportProtocol(
    meetingId: string,
    format: 'markdown' | 'text',
    title: string,
  ): Promise<boolean> {
    const extension = format === 'markdown' ? 'md' : 'txt';
    const safeTitle =
      title
        .trim()
        .replace(/[^a-z0-9]+/gi, '-')
        .replace(/^-|-$/g, '') || 'protocol';
    const destination = await save({
      title: `Export ${title}`,
      defaultPath: `${safeTitle}.${extension}`,
      filters: [
        { name: format === 'markdown' ? 'Markdown' : 'Plain text', extensions: [extension] },
      ],
    });
    if (!destination) return false;
    await invoke('export_protocol', { meetingId, format, destination });
    return true;
  }

  getProtocolProviderStatus(): Promise<ProtocolProviderStatus> {
    return invoke<ProtocolProviderStatus>('protocol_provider_status');
  }

  configureProtocolProvider(model: string | null): Promise<ProtocolProviderStatus> {
    return invoke<ProtocolProviderStatus>('configure_protocol_provider', { model });
  }
}

export function createNativeWorkspaceStore(): WorkspaceStore | undefined {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return undefined;
  return new TauriWorkspaceStore();
}
