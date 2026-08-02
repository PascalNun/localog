import { performance } from 'node:perf_hooks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  AutosaveFailure,
  EditorSession,
  exportMarkdown,
  exportPlainText,
  type WorkingDraftPort,
  type WorkingDraftSave,
} from '../src/editorSession';

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe('EditorSession', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('coalesces rapid edits into one sequenced autosave', async () => {
    const writes: WorkingDraftSave[] = [];
    const port: WorkingDraftPort = {
      async saveWorking(draft) {
        writes.push(draft);
        return { sequence: draft.sequence, savedAt: '2026-08-02T09:00:00Z' };
      },
    };
    const session = new EditorSession({
      documentId: 'protocol-1',
      baseRevision: 3,
      initialMarkdown: '# Protocol',
      debounceMs: 500,
      port,
    });

    session.applyEdit('# Protocol\nA');
    session.applyEdit('# Protocol\nAgenda');
    session.applyEdit('# Protocol\nAgenda\n\nDecision');
    expect(session.state).toMatchObject({ phase: 'waiting', isDirty: true, sequence: 3 });

    await vi.advanceTimersByTimeAsync(499);
    expect(writes).toHaveLength(0);
    await vi.advanceTimersByTimeAsync(1);

    expect(writes).toEqual([
      {
        documentId: 'protocol-1',
        baseRevision: 3,
        sequence: 3,
        markdown: '# Protocol\nAgenda\n\nDecision',
      },
    ]);
    expect(session.state).toMatchObject({
      phase: 'clean',
      isDirty: false,
      acknowledgedSequence: 3,
    });
  });

  it('allows only one save in flight and follows with the latest complete value', async () => {
    const first = deferred<{ sequence: number; savedAt: string }>();
    const writes: WorkingDraftSave[] = [];
    const port: WorkingDraftPort = {
      saveWorking(draft) {
        writes.push(draft);
        if (writes.length === 1) return first.promise;
        return Promise.resolve({ sequence: draft.sequence, savedAt: 'second-save' });
      },
    };
    const session = new EditorSession({
      documentId: 'protocol-1',
      baseRevision: 1,
      initialMarkdown: 'initial',
      debounceMs: 50,
      port,
    });

    session.applyEdit('first');
    await vi.advanceTimersByTimeAsync(50);
    session.applyEdit('second');
    session.applyEdit('latest');
    await vi.advanceTimersByTimeAsync(500);
    expect(writes).toHaveLength(1);

    first.resolve({ sequence: 1, savedAt: 'first-save' });
    await vi.runAllTimersAsync();

    expect(writes).toHaveLength(2);
    expect(writes[1]).toMatchObject({ sequence: 3, markdown: 'latest' });
    expect(session.state).toMatchObject({ phase: 'clean', isDirty: false });
  });

  it('retains dirty state after failure and retries the current value', async () => {
    let attempt = 0;
    const port: WorkingDraftPort = {
      async saveWorking(draft) {
        attempt += 1;
        if (attempt === 1) throw new Error('disk full');
        return { sequence: draft.sequence, savedAt: 'after-retry' };
      },
    };
    const session = new EditorSession({
      documentId: 'protocol-1',
      baseRevision: 1,
      initialMarkdown: 'initial',
      debounceMs: 10,
      port,
    });

    session.applyEdit('never lose this text');
    await vi.advanceTimersByTimeAsync(10);
    expect(session.markdown).toBe('never lose this text');
    expect(session.state).toMatchObject({ phase: 'failed', isDirty: true, error: 'disk full' });

    await session.retry();
    expect(attempt).toBe(2);
    expect(session.state).toMatchObject({ phase: 'clean', isDirty: false, error: null });
  });

  it('rejects a stale acknowledgement and never presents it as saved', async () => {
    const port: WorkingDraftPort = {
      async saveWorking(draft) {
        return { sequence: draft.sequence - 1, savedAt: 'stale' };
      },
    };
    const session = new EditorSession({
      documentId: 'protocol-1',
      baseRevision: 1,
      initialMarkdown: 'initial',
      port,
    });
    session.applyEdit('changed');

    await expect(session.flush()).rejects.toBeInstanceOf(AutosaveFailure);
    expect(session.state).toMatchObject({ phase: 'failed', isDirty: true });
  });
});

describe('Markdown export', () => {
  it('round-trips Markdown exactly and removes supported syntax predictably', () => {
    const markdown = `# Meeting protocol

**Decision:** retain canonical Markdown.

- [x] Validate autosave
- Export [the document](https://example.invalid)

> Review remains required.
`;

    expect(exportMarkdown(markdown)).toBe(markdown);
    expect(exportPlainText(markdown)).toBe(`Meeting protocol

Decision: retain canonical Markdown.

Validate autosave
Export the document

Review remains required.`);
  });

  it('keeps long-document editing and export below the interaction budget', async () => {
    vi.useRealTimers();
    const paragraph =
      '## Synthetic agenda item\n\nThe team reviewed a generated fixture and recorded a decision.\n\n';
    const initial = `# Synthetic protocol\n\n${paragraph.repeat(12_000)}`;
    let savedBytes = 0;
    const port: WorkingDraftPort = {
      async saveWorking(draft) {
        savedBytes = new TextEncoder().encode(draft.markdown).byteLength;
        return { sequence: draft.sequence, savedAt: 'measured' };
      },
    };
    const session = new EditorSession({
      documentId: 'long-synthetic-protocol',
      baseRevision: 8,
      initialMarkdown: initial,
      debounceMs: 1_000,
      port,
    });

    const editStarted = performance.now();
    session.applyEdit(`${initial}\nFinal synthetic note.`);
    const editMs = performance.now() - editStarted;
    const exportStarted = performance.now();
    const plainText = exportPlainText(session.markdown);
    const exportMs = performance.now() - exportStarted;
    await session.flush();

    const bytes = new TextEncoder().encode(session.markdown).byteLength;
    console.info(
      `markdown-editor measurement: bytes=${bytes} edit_ms=${editMs.toFixed(3)} export_ms=${exportMs.toFixed(3)}`,
    );
    expect(bytes).toBeGreaterThan(1_000_000);
    expect(savedBytes).toBe(bytes);
    expect(plainText).toContain('Final synthetic note.');
    expect(editMs).toBeLessThan(100);
    expect(exportMs).toBeLessThan(100);
  });
});
