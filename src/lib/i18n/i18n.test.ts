import { describe, expect, it } from 'vitest';
import { INTERFACE_LANGUAGES, preferredLanguage } from './index';
import { en } from './en';
import { de } from './de';

/** Every key in a nested dictionary, as dotted paths. */
function keysOf(value: unknown, prefix = ''): string[] {
  if (typeof value !== 'object' || value === null) return [prefix];
  return Object.entries(value).flatMap(([key, inner]) =>
    keysOf(inner, prefix ? `${prefix}.${key}` : key),
  );
}

describe('the interface dictionaries', () => {
  /**
   * TypeScript already refuses a German file that is missing a key, so this is
   * not that. It catches the reverse and the subtler thing: a key that exists in
   * both but is a function in one and a plain string in the other, which type
   * checks through the `as const` and then throws when something calls it.
   */
  it('have exactly the same keys, of the same kinds', () => {
    expect(keysOf(de).sort()).toEqual(keysOf(en).sort());
  });

  it('agree about which values take arguments', () => {
    const kinds = (dictionary: unknown, prefix = ''): Record<string, string> => {
      if (typeof dictionary !== 'object' || dictionary === null) return {};
      return Object.entries(dictionary).reduce<Record<string, string>>((all, [key, value]) => {
        const path = prefix ? `${prefix}.${key}` : key;
        if (typeof value === 'object' && value !== null) return { ...all, ...kinds(value, path) };
        return { ...all, [path]: typeof value };
      }, {});
    };
    expect(kinds(de)).toEqual(kinds(en));
  });

  it('never leaves a value empty', () => {
    const empties = (value: unknown, prefix = ''): string[] => {
      if (typeof value === 'string') return value.trim() === '' ? [prefix] : [];
      if (typeof value !== 'object' || value === null) return [];
      return Object.entries(value).flatMap(([key, inner]) =>
        empties(inner, prefix ? `${prefix}.${key}` : key),
      );
    };
    expect(empties(en)).toEqual([]);
    expect(empties(de)).toEqual([]);
  });

  it('offers every language it lists', () => {
    for (const entry of INTERFACE_LANGUAGES) {
      expect(entry.label.trim()).not.toBe('');
      expect(keysOf(entry.strings).length).toBe(keysOf(en).length);
    }
  });
});

describe('choosing the language to start in', () => {
  it('honours what somebody chose before anything else', () => {
    expect(preferredLanguage('de', ['en-GB'])).toBe('de');
    expect(preferredLanguage('en', ['de-DE'])).toBe('en');
  });

  it('follows the system when nobody has chosen', () => {
    expect(preferredLanguage(null, ['de-DE', 'en-GB'])).toBe('de');
    expect(preferredLanguage(null, ['en-US'])).toBe('en');
  });

  it('serves Austria and Switzerland in German', () => {
    expect(preferredLanguage(null, ['de-AT'])).toBe('de');
    expect(preferredLanguage(null, ['de-CH'])).toBe('de');
  });

  it('falls back to English rather than to something merely nearby', () => {
    // Dutch is not German, and a Dutch speaker being given a German interface
    // would be a worse answer than an English one.
    expect(preferredLanguage(null, ['nl-NL'])).toBe('en');
    expect(preferredLanguage(null, ['fr-FR', 'it-IT'])).toBe('en');
    expect(preferredLanguage(null, [])).toBe('en');
  });

  it('ignores a stored value it does not recognise', () => {
    // A language removed from a later build, or a hand-edited value.
    expect(preferredLanguage('kl', ['de-DE'])).toBe('de');
    expect(preferredLanguage('', ['en'])).toBe('en');
  });

  it('takes the first system language it speaks, not the first it is given', () => {
    expect(preferredLanguage(null, ['ja-JP', 'de-DE', 'en-US'])).toBe('de');
  });
});
