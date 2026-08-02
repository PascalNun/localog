export type WindowChrome = 'macos-overlay' | 'standard';

export function resolveWindowChrome(userAgent: string, hasTauriRuntime: boolean): WindowChrome {
  // A macOS browser preview still needs standard chrome; only the native runtime owns an overlay.
  const isMacOS = /Macintosh|Mac OS X/.test(userAgent);
  return hasTauriRuntime && isMacOS ? 'macos-overlay' : 'standard';
}
