export const DEFAULT_SIDEBAR_WIDTH = 248;
export const MIN_SIDEBAR_WIDTH = 216;
export const MAX_SIDEBAR_WIDTH = 360;
export const SIDEBAR_WIDTH_STORAGE_KEY = 'localog-sidebar-width';

export function clampSidebarWidth(width: number): number {
  if (!Number.isFinite(width)) return DEFAULT_SIDEBAR_WIDTH;
  return Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, Math.round(width)));
}

export function parseStoredSidebarWidth(value: string | null): number {
  if (value === null || value.trim() === '') return DEFAULT_SIDEBAR_WIDTH;
  return clampSidebarWidth(Number(value));
}

export function sidebarWidthForKey(
  width: number,
  key: string,
  useLargeStep = false,
): number | null {
  const step = useLargeStep ? 24 : 8;

  if (key === 'ArrowLeft') return clampSidebarWidth(width - step);
  if (key === 'ArrowRight') return clampSidebarWidth(width + step);
  if (key === 'Home') return MIN_SIDEBAR_WIDTH;
  if (key === 'End') return MAX_SIDEBAR_WIDTH;
  if (key === 'Enter') return DEFAULT_SIDEBAR_WIDTH;
  return null;
}
