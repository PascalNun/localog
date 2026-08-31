/**
 * What this system calls the program that shows a folder.
 *
 * The button next to the workspace path opens that folder, and the label has to
 * name the right program: somebody told to look in Finder on Windows is being
 * sent to look for something that does not exist there. The backend picks the
 * program on the same split — `open`, `explorer`, `xdg-open`.
 *
 * Read from the user agent rather than from the backend, because the label is
 * needed while the screen renders and a round trip would show the wrong word
 * first. Anything that is neither macOS nor Windows gets the generic name, which
 * is also the honest one: Linux has no single file manager to name.
 */
export type FileManager = 'finder' | 'explorer' | 'fileManager';

export function resolveFileManager(userAgent: string): FileManager {
  if (/Macintosh|Mac OS X/.test(userAgent)) return 'finder';
  if (/Windows/.test(userAgent)) return 'explorer';
  return 'fileManager';
}

/** The interface key holding the words for it. */
export function showInFileManagerKey(
  userAgent: string,
): 'showInFinder' | 'showInExplorer' | 'showInFileManager' {
  const manager = resolveFileManager(userAgent);
  if (manager === 'finder') return 'showInFinder';
  if (manager === 'explorer') return 'showInExplorer';
  return 'showInFileManager';
}
