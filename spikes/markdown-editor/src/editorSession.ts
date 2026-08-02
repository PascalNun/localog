export type AutosavePhase = 'clean' | 'waiting' | 'saving' | 'failed';

export interface WorkingDraftSave {
  documentId: string;
  baseRevision: number;
  sequence: number;
  markdown: string;
}

export interface WorkingDraftSaveResult {
  sequence: number;
  savedAt: string;
}

export interface WorkingDraftPort {
  saveWorking(draft: WorkingDraftSave): Promise<WorkingDraftSaveResult>;
}

export interface EditorSessionState {
  phase: AutosavePhase;
  isDirty: boolean;
  sequence: number;
  acknowledgedSequence: number;
  lastSavedAt: string | null;
  error: string | null;
}

export interface EditorSessionOptions {
  documentId: string;
  baseRevision: number;
  initialMarkdown: string;
  port: WorkingDraftPort;
  debounceMs?: number;
}

export class AutosaveFailure extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = 'AutosaveFailure';
  }
}

export class EditorSession {
  readonly documentId: string;
  readonly baseRevision: number;

  private readonly port: WorkingDraftPort;
  private readonly debounceMs: number;
  private markdownValue: string;
  private sequenceValue = 0;
  private acknowledgedSequenceValue = 0;
  private phaseValue: AutosavePhase = 'clean';
  private lastSavedAtValue: string | null = null;
  private errorValue: string | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private inFlight: Promise<void> | null = null;
  private disposed = false;
  private readonly listeners = new Set<(state: EditorSessionState) => void>();

  constructor(options: EditorSessionOptions) {
    if (!options.documentId.trim()) throw new Error('documentId must not be empty');
    if (!Number.isSafeInteger(options.baseRevision) || options.baseRevision < 0) {
      throw new Error('baseRevision must be a non-negative safe integer');
    }

    this.documentId = options.documentId;
    this.baseRevision = options.baseRevision;
    this.markdownValue = options.initialMarkdown;
    this.port = options.port;
    this.debounceMs = options.debounceMs ?? 500;
    if (!Number.isFinite(this.debounceMs) || this.debounceMs < 0) {
      throw new Error('debounceMs must be non-negative');
    }
  }

  get markdown(): string {
    return this.markdownValue;
  }

  get state(): EditorSessionState {
    return {
      phase: this.phaseValue,
      isDirty: this.sequenceValue > this.acknowledgedSequenceValue,
      sequence: this.sequenceValue,
      acknowledgedSequence: this.acknowledgedSequenceValue,
      lastSavedAt: this.lastSavedAtValue,
      error: this.errorValue,
    };
  }

  subscribe(listener: (state: EditorSessionState) => void): () => void {
    this.assertActive();
    this.listeners.add(listener);
    listener(this.state);
    return () => this.listeners.delete(listener);
  }

  applyEdit(markdown: string): void {
    this.assertActive();
    if (markdown === this.markdownValue) return;

    this.markdownValue = markdown;
    this.sequenceValue += 1;
    this.errorValue = null;
    this.phaseValue = this.inFlight ? 'saving' : 'waiting';
    this.clearTimer();
    if (!this.inFlight) this.scheduleSave();
    this.emit();
  }

  async flush(): Promise<void> {
    this.assertActive();
    this.clearTimer();

    while (this.sequenceValue > this.acknowledgedSequenceValue) {
      if (this.inFlight) {
        await this.inFlight;
      } else {
        await this.saveSnapshot();
      }
      this.clearTimer();
    }
  }

  async retry(): Promise<void> {
    this.assertActive();
    if (this.phaseValue !== 'failed') return;
    this.errorValue = null;
    this.phaseValue = 'waiting';
    this.emit();
    await this.flush();
  }

  dispose(): void {
    this.clearTimer();
    this.listeners.clear();
    this.disposed = true;
  }

  private scheduleSave(): void {
    this.timer = setTimeout(() => {
      this.timer = null;
      void this.saveSnapshot().catch(() => {
        // Failure is represented in observable state and retried only by an explicit action.
      });
    }, this.debounceMs);
  }

  private async saveSnapshot(): Promise<void> {
    if (this.inFlight) return this.inFlight;
    if (this.sequenceValue <= this.acknowledgedSequenceValue) {
      this.phaseValue = 'clean';
      this.emit();
      return;
    }

    const draft: WorkingDraftSave = {
      documentId: this.documentId,
      baseRevision: this.baseRevision,
      sequence: this.sequenceValue,
      markdown: this.markdownValue,
    };

    this.phaseValue = 'saving';
    this.errorValue = null;
    this.emit();

    const operation = this.persist(draft);
    this.inFlight = operation;

    try {
      await operation;
    } finally {
      this.inFlight = null;
    }

    if (this.sequenceValue > this.acknowledgedSequenceValue) {
      this.phaseValue = 'waiting';
      this.scheduleSave();
    } else {
      this.phaseValue = 'clean';
    }
    this.emit();
  }

  private async persist(draft: WorkingDraftSave): Promise<void> {
    try {
      const result = await this.port.saveWorking(draft);
      if (result.sequence !== draft.sequence) {
        throw new Error(
          `Autosave acknowledgement ${result.sequence} did not match request ${draft.sequence}`,
        );
      }
      if (!result.savedAt.trim()) throw new Error('Autosave acknowledgement omitted savedAt');

      this.acknowledgedSequenceValue = Math.max(this.acknowledgedSequenceValue, result.sequence);
      this.lastSavedAtValue = result.savedAt;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown autosave failure';
      this.phaseValue = 'failed';
      this.errorValue = message;
      this.emit();
      throw new AutosaveFailure(message, { cause: error });
    }
  }

  private clearTimer(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
  }

  private emit(): void {
    const state = this.state;
    for (const listener of this.listeners) listener(state);
  }

  private assertActive(): void {
    if (this.disposed) throw new Error('EditorSession has been disposed');
  }
}

export function exportMarkdown(markdown: string): string {
  return markdown;
}

export function exportPlainText(markdown: string): string {
  return markdown
    .replace(/^[ \t]*```[^\n]*$/gm, '')
    .replace(/^[ \t]{0,3}#{1,6}[ \t]+/gm, '')
    .replace(/^[ \t]{0,3}>[ \t]?/gm, '')
    .replace(/^[ \t]*[-*+]\s+\[[ xX]\]\s+/gm, '')
    .replace(/^[ \t]*[-*+]\s+/gm, '')
    .replace(/^[ \t]*\d+[.)]\s+/gm, '')
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/(\*\*|__)(.*?)\1/g, '$2')
    .replace(/(?<!\*)\*([^*\n]+)\*(?!\*)/g, '$1')
    .replace(/(?<!_)_([^_\n]+)_(?!_)/g, '$1')
    .replace(/`([^`\n]+)`/g, '$1')
    .replace(/\n{3,}/g, '\n\n')
    .trimEnd();
}
