export type WindowChrome = 'macos-overlay' | 'linux-integrated' | 'linux-borderless' | 'standard';
export type WindowControlsChoice = 'auto' | 'show' | 'hide';
export const WINDOW_CONTROLS_KEY = 'localog-window-controls';
export function parseWindowControlsChoice(value: string | null): WindowControlsChoice {
  return value === 'show' || value === 'hide' ? value : 'auto';
}
export function resolveWindowChrome(
  userAgent: string,
  hasTauriRuntime: boolean,
  choice: WindowControlsChoice = 'auto',
  tilingSession = false,
): WindowChrome {
  const isMacOS = /Macintosh|Mac OS X/.test(userAgent);
  if (!hasTauriRuntime) return 'standard';
  if (isMacOS) return 'macos-overlay';
  if (/Linux/.test(userAgent) && !/Android/.test(userAgent)) {
    return choice === 'hide' || (choice === 'auto' && tilingSession)
      ? 'linux-borderless'
      : 'linux-integrated';
  }
  return 'standard';
}
