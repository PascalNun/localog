import { describe, expect, it } from 'vitest';
import { resolveWindowChrome, parseWindowControlsChoice } from './windowChrome';

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

  it('keeps Windows on standard window chrome', () => {
    expect(resolveWindowChrome('Mozilla/5.0 (Windows NT 10.0; Win64; x64)', true)).toBe('standard');
  });
  it('integrates controls only in native desktop Linux', () => {
    expect(resolveWindowChrome('Mozilla/5.0 (X11; Linux x86_64)', true)).toBe('linux-integrated');
    expect(resolveWindowChrome('Mozilla/5.0 (X11; Linux x86_64)', false)).toBe('standard');
    expect(resolveWindowChrome('Mozilla/5.0 (Linux; Android 15)', true)).toBe('standard');
  });
  it('follows a tiling session while respecting explicit Linux overrides', () => {
    const linux = 'Mozilla/5.0 (X11; Linux x86_64)';
    expect(resolveWindowChrome(linux, true, 'auto', true)).toBe('linux-borderless');
    expect(resolveWindowChrome(linux, true, 'show', true)).toBe('linux-integrated');
    expect(resolveWindowChrome(linux, true, 'hide', false)).toBe('linux-borderless');
    expect(resolveWindowChrome('Macintosh', true, 'hide', true)).toBe('macos-overlay');
    expect(resolveWindowChrome(linux, false, 'hide', true)).toBe('standard');
  });
  it('defaults missing or invalid saved choices to automatic', () => {
    expect(parseWindowControlsChoice(null)).toBe('auto');
    expect(parseWindowControlsChoice('invalid')).toBe('auto');
    expect(parseWindowControlsChoice('show')).toBe('show');
    expect(parseWindowControlsChoice('hide')).toBe('hide');
  });
});
