import { describe, expect, it } from 'vitest';
import {
  INTERFACE_LANGUAGES,
  chooseLanguage,
  failureText,
  isFailureCode,
  preferredLanguage,
} from './index';
import { errorMessage } from '../errors';
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
    // would be a worse answer than an English one. This used to say the same of
    // French and Italian, which stopped being true when they were added — the
    // examples have to be languages the application genuinely does not speak.
    expect(preferredLanguage(null, ['nl-NL'])).toBe('en');
    expect(preferredLanguage(null, ['pt-BR', 'pl-PL'])).toBe('en');
    expect(preferredLanguage(null, [])).toBe('en');
  });

  it('serves every language it offers, not only the first two', () => {
    for (const entry of INTERFACE_LANGUAGES) {
      expect(preferredLanguage(null, [`${entry.id}-XX`])).toBe(entry.id);
    }
  });

  it('ignores a stored value it does not recognise', () => {
    // A language removed from a later build, or a hand-edited value.
    expect(preferredLanguage('kl', ['de-DE'])).toBe('de');
    expect(preferredLanguage('', ['en'])).toBe('en');
  });

  it('takes the first system language it speaks, not the first it is given', () => {
    // Polish rather than Japanese, which this used to say: an example of a language
    // the application does not speak has to keep being one.
    expect(preferredLanguage(null, ['pl-PL', 'de-DE', 'en-US'])).toBe('de');
  });
});

describe('a failure that came from Rust', () => {
  /**
   * The whole point of the conversion. Rust says what happened; the interface
   * says it in words, and says it in whichever language it is currently in.
   */
  it('is rendered in the language the interface is in', () => {
    chooseLanguage('en');
    expect(failureText('missingProject')).toBe('The selected project no longer exists.');
    chooseLanguage('de');
    expect(failureText('missingProject')).toBe('Das gewählte Projekt existiert nicht mehr.');
    chooseLanguage('en');
  });

  it('puts the detail into the sentence', () => {
    chooseLanguage('en');
    expect(failureText('backupNameTaken:LocaLog backup 2026-08-27')).toContain(
      'LocaLog backup 2026-08-27',
    );
    chooseLanguage('de');
    expect(failureText('backupNameTaken:Sicherung')).toContain('Sicherung');
    chooseLanguage('en');
  });

  it('splits on the first colon only, because a detail can be a path', () => {
    // A Windows path, or anything else with a colon in it, must arrive whole.
    const rendered = failureText('backupDamaged:C:/work/a.wav is missing');
    expect(rendered).toContain('C:/work/a.wav is missing');
  });

  it('shows an unknown key as itself rather than as nothing', () => {
    expect(failureText('somethingNobodyWroteDown')).toBe('somethingNobodyWroteDown');
    expect(isFailureCode('somethingNobodyWroteDown')).toBe(false);
  });

  it('leaves a sentence the front end raised alone', () => {
    const own = 'Backing up needs the desktop application.';
    expect(errorMessage(own)).toBe(own);
  });

  it('translates through the funnel every catch site already uses', () => {
    chooseLanguage('de');
    expect(errorMessage('importBusy')).toBe(
      'Es wird bereits eine Aufnahme importiert. Schließen Sie diesen Vorgang ab oder brechen Sie ihn ab.',
    );
    expect(errorMessage(new Error('missingMeeting'))).toBe(
      'Die gewählte Besprechung existiert nicht mehr.',
    );
    chooseLanguage('en');
  });
});
