import { describe, expect, it } from 'vitest';
import {
  DEFAULT_SIDEBAR_WIDTH,
  MAX_SIDEBAR_WIDTH,
  MIN_SIDEBAR_WIDTH,
  clampSidebarWidth,
  parseStoredSidebarWidth,
  sidebarWidthForKey,
} from './sidebarSizing';

describe('sidebar sizing', () => {
  it('keeps widths within the approved desktop range', () => {
    expect(clampSidebarWidth(180)).toBe(MIN_SIDEBAR_WIDTH);
    expect(clampSidebarWidth(284.4)).toBe(284);
    expect(clampSidebarWidth(480)).toBe(MAX_SIDEBAR_WIDTH);
  });

  it('uses the default for missing or invalid persisted values', () => {
    expect(parseStoredSidebarWidth(null)).toBe(DEFAULT_SIDEBAR_WIDTH);
    expect(parseStoredSidebarWidth('')).toBe(DEFAULT_SIDEBAR_WIDTH);
    expect(parseStoredSidebarWidth('not-a-number')).toBe(DEFAULT_SIDEBAR_WIDTH);
    expect(parseStoredSidebarWidth('320')).toBe(320);
  });

  it('supports bounded keyboard adjustment and reset', () => {
    expect(sidebarWidthForKey(248, 'ArrowLeft')).toBe(240);
    expect(sidebarWidthForKey(248, 'ArrowRight', true)).toBe(272);
    expect(sidebarWidthForKey(248, 'Home')).toBe(MIN_SIDEBAR_WIDTH);
    expect(sidebarWidthForKey(248, 'End')).toBe(MAX_SIDEBAR_WIDTH);
    expect(sidebarWidthForKey(320, 'Enter')).toBe(DEFAULT_SIDEBAR_WIDTH);
    expect(sidebarWidthForKey(248, 'Escape')).toBeNull();
  });
});
