/**
 * A file size as somebody reading about their own recording would say it.
 *
 * Decimal, because that is what a recorder and a file manager both report: a
 * 40 MB import should say 40 MB, not 38. One decimal place under 10 MB and none
 * above, so the number stops moving once it is large enough not to matter.
 *
 * SettingsView keeps a separate formatSize for model downloads, and that one is
 * binary and says GB — which is the convention the model files themselves are
 * published under. The two are not the same fact and are deliberately not shared.
 */
export function formatBytes(bytes: number): string {
  if (bytes < 1_000_000) return `${Math.round(bytes / 1_000)} KB`;
  return `${(bytes / 1_000_000).toFixed(bytes >= 10_000_000 ? 0 : 1)} MB`;
}
