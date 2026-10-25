import { invoke } from '@tauri-apps/api/core';
import {
  open as pluginOpen,
  save as pluginSave,
  type OpenDialogOptions,
  type SaveDialogOptions,
} from '@tauri-apps/plugin-dialog';

/**
 * The app's own file dialogs on Linux.
 *
 * The dialog plugin's GTK chooser has no transient parent, so a tiling compositor
 * lays it out beside the app and a fullscreen app clips its action bar off the
 * screen. The Rust side opens the same chooser with the window as parent, so it
 * floats and is modal; everywhere else the plugin behaves as it always has.
 */
const isLinux = typeof navigator !== 'undefined' && /Linux/.test(navigator.userAgent);

export async function openDialog(options: OpenDialogOptions): Promise<string | string[] | null> {
  if (!isLinux) return pluginOpen(options);
  const found = await invoke<{ path: string } | null>('native_open_path', {
    title: options.title ?? '',
    directory: options.directory ?? false,
    multiple: options.multiple ?? false,
    filters: (options.filters ?? []).map((filter) => ({
      name: filter.name ?? '',
      extensions: filter.extensions ?? [],
    })),
  });
  return found ? found.path : null;
}

export async function saveDialog(options: SaveDialogOptions): Promise<string | null> {
  if (!isLinux) return pluginSave(options);
  const found = await invoke<{ path: string } | null>('native_save_path', {
    title: options.title ?? '',
    defaultPath: options.defaultPath ?? null,
    filters: (options.filters ?? []).map((filter) => ({
      name: filter.name ?? '',
      extensions: filter.extensions ?? [],
    })),
  });
  return found ? found.path : null;
}
