export type WindowChrome = 'macos-overlay' | 'standard';

export function resolveWindowChrome(userAgent: string, hasTauriRuntime: boolean): WindowChrome {
  const isMacOS = /Macintosh|Mac OS X/.test(userAgent);
  return hasTauriRuntime && isMacOS ? 'macos-overlay' : 'standard';
}
