import { describe, expect, it } from 'vitest';
import { chooseLanguage, failureText, isFailureCode } from './index';
import { everyRustSource, shipped } from './rustSource';
import { en } from './en';
import { de } from './de';

/**
 * Rust names a failure; this side says it in words. Nothing enforces the join, and
 * for three months nothing needed to, because the words *were* the code — the
 * backend wrote English sentences and the interface printed them.
 *
 * It stopped being English-only on 28 August 2026, and the sentences that were
 * missed did not become visibly wrong: they went on reading correctly to the one
 * reader who speaks English, and reached everybody else in a language they may not
 * have. That is the failure mode this guard exists for — not a crash, not a blank,
 * but a screen that is right for the person who wrote it.
 *
 * Twenty-two of them were still there on 30 August, all inside a multi-line call
 * that the earlier searches read as three lines rather than as one string.
 */

/**
 * Every failure code this can see the backend raising.
 *
 * **Deliberately not a complete list**, in the direction that is safe: a code the
 * pattern cannot see means this guard checks less, never that it fails on correct
 * code. `InvalidData` takes a `&'static str`, so every one of them is a literal and
 * this finds them all today; that is a property of the type rather than a promise,
 * and if it ever stops holding this quietly checks fewer.
 */
function raised(source: string): string[] {
  const found: string[] = [];
  for (const call of source.matchAll(/InvalidData\(\s*"([^"]+)"/g)) {
    if (call[1]) found.push(call[1]);
  }
  // A `code()` method mapping its variants to codes, which is how the storage
  // layer and the generation runtime both report themselves. Added on 30 August
  // after a new module raised five codes this could not see — the same lesson
  // this file already records, arriving again from a different direction.
  //
  // The name must be camelCase, which is what every code in the dictionary is.
  // Requiring one capital is what keeps `"none".to_string()` — a state, not a
  // failure — from being read as a code nobody wrote words for.
  for (const arm of source.matchAll(/=>\s*"([a-z]+[A-Z][a-zA-Z]*)"\.(?:into|to_string)\(\)/g)) {
    if (arm[1]) found.push(arm[1]);
  }
  return found;
}

/** The status the Ollama row in Settings reports, which is a code like any other. */
function providerStatus(source: string): string[] {
  const found: string[] = [];
  for (const line of source.matchAll(/status\.message = (?:format!\(\s*)?"([a-zA-Z]+)/g)) {
    if (line[1]) found.push(line[1]);
  }
  return found;
}

describe('every failure the backend can raise', () => {
  it('has words for it, in English', () => {
    const known = new Set(Object.keys(en.failures));
    const missing = new Set<string>();
    for (const { text } of everyRustSource()) {
      const source = shipped(text);
      for (const code of [...raised(source), ...providerStatus(source)]) {
        if (!known.has(code)) missing.add(code);
      }
    }
    expect([...missing].sort()).toEqual([]);
  });

  /**
   * German is typed against English, so a *missing* key is already a compile error.
   * What that cannot catch is a key left holding the English sentence, which is what
   * happens when somebody adds one to both files and translates neither.
   */
  it('says something different in German', () => {
    const shared = Object.entries(en.failures).filter(
      ([key, value]) =>
        typeof value === 'string' &&
        value === (de.failures as Record<string, unknown>)[key] &&
        // A sentence that is genuinely the same in both. `Ollama` is a proper noun
        // and `FFmpeg` is a program; neither has a German form.
        !/^(Ollama|FFmpeg)/.test(value),
    );
    expect(shared.map(([key]) => key)).toEqual([]);
  });
});

describe('the Ollama row in Settings', () => {
  it('renders each of its five states in the reader’s language', () => {
    const states = [
      'ollamaNotRunning:connection refused',
      'ollamaModelsUnreadable:unexpected end of JSON',
      'ollamaReadyNoModel',
      'ollamaModelReady',
      'ollamaSelectedModelMissing',
    ];
    chooseLanguage('de');
    for (const state of states) {
      expect(isFailureCode(state)).toBe(true);
      const said = failureText(state);
      expect(said).not.toContain('ollama');
      expect(said.length).toBeGreaterThan(10);
    }
    chooseLanguage('en');
  });

  /**
   * The guidance is translated; the transport failure is not, and should not be.
   * It names the port and the reason, and is the only part of this row somebody can
   * act on when "start Ollama" was not the answer.
   */
  it('keeps the diagnostic after the colon, whatever the language', () => {
    chooseLanguage('ja');
    expect(failureText('ollamaNotRunning:connection refused (os error 61)')).toContain(
      'connection refused (os error 61)',
    );
    chooseLanguage('en');
    expect(failureText('ollamaNotRunning')).not.toMatch(/\s$/);
  });
});
