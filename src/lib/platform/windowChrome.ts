export type WindowChrome = 'macos-overlay' | 'linux-integrated' | 'standard';

export function resolveWindowChrome(userAgent: string, hasTauriRuntime: boolean): WindowChrome {
  // A macOS browser preview still needs standard chrome; only the native runtime owns an overlay.
  const isMacOS = /Macintosh|Mac OS X/.test(userAgent);
  if (!hasTauriRuntime) return 'standard';
  if (isMacOS) return 'macos-overlay';
  return /Linux/.test(userAgent) && !/Android/.test(userAgent) ? 'linux-integrated' : 'standard';
}
