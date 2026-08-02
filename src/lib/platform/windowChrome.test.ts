import { describe, expect, it } from 'vitest';
import { resolveWindowChrome } from './windowChrome';

describe('resolveWindowChrome', () => {
  it('uses integrated window chrome only for the native macOS shell', () => {
    expect(resolveWindowChrome('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)', true)).toBe(
      'macos-overlay',
    );
  });

  it('keeps browser previews on standard window chrome', () => {
    expect(resolveWindowChrome('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)', false)).toBe(
      'standard',
    );
  });

  it('keeps other desktop platforms on standard window chrome', () => {
    expect(resolveWindowChrome('Mozilla/5.0 (Windows NT 10.0; Win64; x64)', true)).toBe('standard');
    expect(resolveWindowChrome('Mozilla/5.0 (X11; Linux x86_64)', true)).toBe('standard');
  });
});
