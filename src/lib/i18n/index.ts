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
import { fr } from './fr';
import { es } from './es';
import { it } from './it';
import { ja } from './ja';
import { ko } from './ko';
import { zh } from './zh';
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
  { id: 'fr', label: 'Français', strings: fr },
  { id: 'es', label: 'Español', strings: es },
  { id: 'it', label: 'Italiano', strings: it },
  { id: 'ja', label: '日本語', strings: ja },
  { id: 'ko', label: '한국어', strings: ko },
  { id: 'zh', label: '简体中文', strings: zh },
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
 * Rust returns a key — `missingProject`, or `backupDamaged:projects/a.wav` —
 * and never a sentence, so the application can be translated without the
 * backend knowing or caring which language the interface is in.
 *
 * The detail is whatever follows the *first* colon, because a detail is often a
 * path and a path contains colons. A key with nothing to render it comes back as
 * itself: ugly on screen, and far better than an empty dialog, because it names
 * exactly what to add to the dictionary.
 */
export function failureText(raw: string): string {
  const at = raw.indexOf(':');
  const code = at === -1 ? raw : raw.slice(0, at);
  const detail = at === -1 ? '' : raw.slice(at + 1);
  const found = (get(t).failures as Record<string, unknown>)[code];
  if (typeof found === 'function') return (found as (value: string) => string)(detail);
  if (typeof found === 'string') return found;
  return raw;
}

/** Whether a string is a key this can render, rather than text to show as it is. */
export function isFailureCode(raw: string): boolean {
  const code = raw.includes(':') ? raw.slice(0, raw.indexOf(':')) : raw;
  return Object.prototype.hasOwnProperty.call(get(t).failures, code);
}

/**
 * The words for a job stage, in the language the interface is in.
 *
 * The same shape as `failureText` and for the same reason: the backend says *which*
 * stage, and the interface says it in words. A stage may carry a live detail after a
 * colon — `finding_subjects:3 of 13` — because a step lasting minutes must not show
 * the same words throughout, and the detail is split on the *first* colon so that a
 * detail containing one arrives whole.
 *
 * A code with no words falls back to "Working", which is what it always did. That is
 * a real fallback rather than a theoretical one: a stage was once renamed on one side
 * and not the other, and the longest phase of writing a protocol read "Working" for
 * weeks. `jobStages.test.ts` is what stops that happening again.
 */
export function stageText(raw: string): string {
  const at = raw.indexOf(':');
  const code = at === -1 ? raw : raw.slice(0, at);
  const detail = at === -1 ? '' : raw.slice(at + 1);
  const stages = get(t).jobStages as Record<string, unknown>;
  const found = stages[code];
  if (typeof found === 'function') return (found as (value: string) => string)(detail);
  if (typeof found === 'string') return found;
  return stages.working as string;
}

/**
 * What a failed job is called, and what it says about what is safe.
 *
 * The code names the class of failure; a code nobody wrote words for falls back to
 * the general one rather than to nothing, because a person whose work has just
 * stopped needs a sentence more than anybody.
 *
 * The `detail` a step stored beats the default for the class, because a step often
 * knows something the class does not — which model went missing, how much would not
 * fit. It arrives as a code too, so it goes through the same funnel; a plain sentence
 * from an older build passes through unchanged.
 */
export function jobErrorTitle(code: string): string {
  const errors = get(t).jobErrors as Record<string, { title: string; detail: string } | undefined>;
  return (errors[code] ?? get(t).jobErrors.unknown).title;
}

export function jobErrorDetail(code: string, stored: string): string {
  const said = stored.trim();
  if (said !== '') return isFailureCode(said) ? failureText(said) : said;
  const errors = get(t).jobErrors as Record<string, { title: string; detail: string } | undefined>;
  return (errors[code] ?? get(t).jobErrors.unknown).detail;
}
