import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createNativeWorkspaceStore } from './workspaceStore';
import { buildDocx } from '../protocol/docx';
import { DEFAULT_APPEARANCE } from './types';

const dialogs = vi.hoisted(() => ({ save: vi.fn(), invoke: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ save: dialogs.save, open: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: dialogs.invoke, convertFileSrc: vi.fn() }));

beforeEach(() => {
  vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });
  vi.clearAllMocks();
  dialogs.invoke.mockResolvedValue(undefined);
});
afterEach(() => vi.unstubAllGlobals());

describe('native document exports', () => {
  it('saves PDF directly after one destination choice', async () => {
    dialogs.save.mockResolvedValue('/tmp/synthetic.pdf');
    expect(await createNativeWorkspaceStore()!.exportPdf('Synthetic meeting')).toBe(true);
    expect(dialogs.save).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({ defaultPath: 'Synthetic-meeting.pdf' }),
    );
    expect(dialogs.invoke).toHaveBeenCalledExactlyOnceWith('export_pdf', {
      destination: '/tmp/synthetic.pdf',
    });
  });

  it('does not print or write when the destination choice is cancelled', async () => {
    dialogs.save.mockResolvedValue(null);
    const store = createNativeWorkspaceStore()!;
    expect(await store.exportPdf('Synthetic')).toBe(false);
    expect(await store.exportProtocol('meeting', 'markdown', 'Synthetic')).toBe(false);
    expect(await store.exportProtocolBytes(new Uint8Array([1]), 'Synthetic', 'docx', 'Word')).toBe(
      false,
    );
    expect(dialogs.invoke).not.toHaveBeenCalled();
  });

  it('passes complete Word bytes to the native writer', async () => {
    dialogs.save.mockResolvedValue('/tmp/synthetic.docx');
    const bytes = buildDocx({
      title: 'Synthetic meeting',
      subtitle: '',
      markdown: '# Decisions\n\nBen will send the quote by Friday.',
      appearance: DEFAULT_APPEARANCE,
    });
    expect(Array.from(bytes.slice(0, 4))).toEqual([80, 75, 3, 4]);
    expect(
      await createNativeWorkspaceStore()!.exportProtocolBytes(bytes, 'Synthetic', 'docx', 'Word'),
    ).toBe(true);
    expect(dialogs.invoke).toHaveBeenCalledExactlyOnceWith('export_protocol_bytes', {
      destination: '/tmp/synthetic.docx',
      contents: Array.from(bytes),
    });
  });

  it('exports Markdown from the selected meeting to the selected path', async () => {
    dialogs.save.mockResolvedValue('/tmp/synthetic.md');
    expect(
      await createNativeWorkspaceStore()!.exportProtocol('meeting-1', 'markdown', 'Synthetic'),
    ).toBe(true);
    expect(dialogs.invoke).toHaveBeenCalledExactlyOnceWith('export_protocol', {
      meetingId: 'meeting-1',
      format: 'markdown',
      destination: '/tmp/synthetic.md',
    });
  });

  it('propagates a failed write so it cannot be reported as saved', async () => {
    dialogs.save.mockResolvedValue('/tmp/synthetic.pdf');
    dialogs.invoke.mockRejectedValue('exportFileExists');
    await expect(createNativeWorkspaceStore()!.exportPdf('Synthetic')).rejects.toBe(
      'exportFileExists',
    );
  });
});
