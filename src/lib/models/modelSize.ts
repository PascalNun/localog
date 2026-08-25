/**
 * A model file's size, in the convention the model files are published under.
 *
 * Binary, and saying GB above a gigabyte, because that is how whisper.cpp's own
 * releases describe them. Somebody comparing what LocaLog offers against the
 * published list should read the same number in both places.
 *
 * Deliberately not `formatBytes` in `../bytes`, which is decimal because a
 * recording's size should read the way the file manager that shows it does. The
 * two are different facts about different things and only look like one
 * function — which is why this one lives here rather than being folded in.
 */
export function formatModelSize(bytes: number): string {
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
  return `${Math.round(bytes / 1024 ** 2)} MB`;
}
