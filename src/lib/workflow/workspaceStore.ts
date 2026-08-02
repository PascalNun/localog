import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  ActiveJob,
  MeetingSummary,
  NewMeetingInput,
  NewProjectInput,
  ProjectSummary,
  SourceSelection,
} from './types';

export interface WorkspaceData {
  projects: ProjectSummary[];
  meetings: MeetingSummary[];
  jobs: ActiveJob[];
}

/**
 * Narrow persistence port for the hierarchy proven in Phase 1A.
 * Processing artifacts remain owned by the fake workflow until their Rust commit boundaries exist.
 */
export interface WorkspaceStore {
  loadWorkspace(): Promise<WorkspaceData>;
  createProject(input: NewProjectInput): Promise<ProjectSummary>;
  createMeeting(input: NewMeetingInput): Promise<MeetingSummary>;
  updateMeetingTitle(meetingId: string, title: string): Promise<void>;
  selectMediaSource(): Promise<SourceSelection | null>;
  startImport(meetingId: string): Promise<void>;
  cancelImport(meetingId: string): Promise<void>;
  retryImport(meetingId: string, allowDuplicate: boolean): Promise<void>;
  replaceImportSource(meetingId: string, source: SourceSelection): Promise<void>;
  subscribe(listener: (workspace: WorkspaceData) => void): Promise<UnlistenFn>;
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

  subscribe(listener: (workspace: WorkspaceData) => void): Promise<UnlistenFn> {
    return listen<WorkspaceData>('workspace://changed', (event) => listener(event.payload));
  }
}

export function createNativeWorkspaceStore(): WorkspaceStore | undefined {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return undefined;
  return new TauriWorkspaceStore();
}
