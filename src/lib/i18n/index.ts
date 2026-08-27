/**
 * Which language the application speaks, and how a component asks for a word.
 *
 * A store rather than a prop threaded through every component: the language is
 * one fact for the whole window, and passing it down would put a parameter on
 * every component in the application in order to say something none of them
 * decide.
 *
 * ## Choosing it
 *
 * The stored choice wins. With none — a first run — the system's own language
 * decides, so somebody whose Mac is in German gets German without being asked.
 * A system language nothing here speaks falls back to English rather than to
 * something merely near it: a Swiss German or an Austrian locale is served well
 * by German, and that is what `startsWith` gives, but Dutch is not German and
 * must not silently become it.
 */

import { writable, derived, get } from 'svelte/store';
import { en } from './en';
import { de } from './de';
import type { Strings } from './en';

export type { Strings };

/**
 * The languages the interface itself is written in.
 *
 * Nothing to do with the twelve a meeting can be transcribed in: a German office
 * regularly minutes an English meeting, and the application it does that in
 * should still be in German.
 */
export const INTERFACE_LANGUAGES = [
  { id: 'en', label: 'English', strings: en },
  { id: 'de', label: 'Deutsch', strings: de },
] as const;

export type LanguageId = (typeof INTERFACE_LANGUAGES)[number]['id'];

const STORAGE_KEY = 'localog-interface-language';

function known(value: string | null | undefined): LanguageId | null {
  return INTERFACE_LANGUAGES.some((each) => each.id === value) ? (value as LanguageId) : null;
}

/**
 * What to start in, before anybody has chosen.
 *
 * `de-AT` and `de-CH` are served by German, so the tag is matched on its first
 * part. Anything unrecognised is English, which is the language this application
 * was written in and the one its documentation is in.
 */
export function preferredLanguage(
  stored: string | null,
  systemLanguages: readonly string[],
): LanguageId {
  const chosen = known(stored);
  if (chosen) return chosen;
  for (const tag of systemLanguages) {
    const base = known(tag.split('-')[0]?.toLowerCase());
    if (base) return base;
  }
  return 'en';
}

export const language = writable<LanguageId>('en');

/** The dictionary for the current language. Components read `$t`. */
export const t = derived(language, ($language) => {
  const found = INTERFACE_LANGUAGES.find((each) => each.id === $language);
  return (found ?? INTERFACE_LANGUAGES[0]).strings;
});

/** Read once outside a component, where a subscription would be the wrong shape. */
export function strings(): Strings {
  return get(t);
}

/** Remember the choice, and apply it now. */
export function chooseLanguage(id: LanguageId) {
  language.set(id);
  try {
    localStorage.setItem(STORAGE_KEY, id);
  } catch {
    // A window with no storage still gets the language it asked for; it just
    // will not be remembered. Not worth an error somebody has to dismiss.
  }
  if (typeof document !== 'undefined') {
    // So the page announces its own language, which screen readers and the
    // browser's own hyphenation both read.
    document.documentElement.lang = id;
  }
}

/** Work out the starting language and apply it. Called once, at startup. */
export function startLanguage() {
  // Declared without a value: both branches below set one, and an initialiser
  // nothing reads is a claim that there is a third case.
  let stored: string | null;
  try {
    stored = localStorage.getItem(STORAGE_KEY);
  } catch {
    stored = null;
  }
  const system =
    typeof navigator === 'undefined' ? [] : (navigator.languages ?? [navigator.language]);
  chooseLanguage(preferredLanguage(stored, system));
}

/**
 * The sentence for something that failed in the backend.
 *
 * Rust returns a code — `missingProject` — and never a sentence, so that the
 * application can be translated without the backend knowing or caring which
 * language the interface is in. A code with no entry here is shown as itself:
 * ugly, and far better than an empty dialog, because it names exactly what to
 * look up.
 */
export function failureText(code: string, detail?: number): string {
  const table = get(t).failures as Record<string, unknown>;
  const found = table[code];
  if (typeof found === 'function') return (found as (value: number) => string)(detail ?? 0);
  if (typeof found === 'string') return found;
  return code;
}
