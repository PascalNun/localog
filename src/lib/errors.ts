import { failureText, isFailureCode } from './i18n';

/**
 * What to show a person when something failed.
 *
 * `instanceof Error` alone is not enough here. Every Tauri command returns its
 * failure as a string, so what arrives at a `catch` from the real application is
 * usually a plain string and never an Error — a check for Error alone would show
 * "[object Object]" for exactly the failures that come from the desktop side.
 *
 * Written out thirty-one times across nine files before it was written here once.
 */
export function errorMessage(cause: unknown, fallback?: string): string {
  // A string from a Tauri command is a key — `missingProject`, or
  // `backupDamaged:projects/a.wav` — because the backend stopped writing
  // sentences on 27 August 2026. Anything that is not a key it recognises is
  // passed through unchanged, which covers a message the front end raised itself
  // and a key that has not been written down yet.
  if (typeof cause === 'string') return isFailureCode(cause) ? failureText(cause) : cause;
  if (cause instanceof Error) {
    return isFailureCode(cause.message) ? failureText(cause.message) : cause.message;
  }
  // Anything else has no message worth reading. A caller who knows what was being
  // attempted can say so; without that, showing the value is better than silence.
  return fallback ?? String(cause);
}
