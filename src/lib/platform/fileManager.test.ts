import { describe, expect, it } from 'vitest';
import { resolveFileManager, showInFileManagerKey } from './fileManager';

const MAC = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)';
const WINDOWS = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)';
const LINUX = 'Mozilla/5.0 (X11; Linux x86_64)';

describe('resolveFileManager', () => {
  it('names the program each system actually has', () => {
    expect(resolveFileManager(MAC)).toBe('finder');
    expect(resolveFileManager(WINDOWS)).toBe('explorer');
    expect(resolveFileManager(LINUX)).toBe('fileManager');
  });

  // The fault this replaced: one label, reading "Show in Finder" on Windows,
  // above a button that did nothing at all.
  it('picks a different word for each system', () => {
    const keys = [MAC, WINDOWS, LINUX].map(showInFileManagerKey);
    expect(new Set(keys).size).toBe(3);
  });
});
