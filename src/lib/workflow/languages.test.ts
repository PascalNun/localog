import { describe, expect, it } from 'vitest';
import {
  COMMON_MEETING_LANGUAGES,
  detectLanguageLabel,
  meetingLanguageField,
  meetingLanguageLabel,
  meetingLanguageValue,
} from './languages';
import { chooseLanguage } from '../i18n';

describe('meeting language choices', () => {
  it('keeps the primary German and English workflow choices visible', () => {
    expect(COMMON_MEETING_LANGUAGES).toContain('English');
    expect(COMMON_MEETING_LANGUAGES).toContain('German');
  });

  it('does not treat the convenience list as an exclusive language catalogue', () => {
    expect(COMMON_MEETING_LANGUAGES.length).toBeGreaterThan(10);
  });
});

describe('meetingLanguageLabel', () => {
  it('names a chosen language and says plainly when none was chosen', () => {
    chooseLanguage('en');
    expect(meetingLanguageLabel('German')).toBe('German');
    expect(meetingLanguageLabel('')).toBe(detectLanguageLabel());
    expect(meetingLanguageLabel('   ')).toBe(detectLanguageLabel());
    expect(meetingLanguageLabel(null)).toBe(detectLanguageLabel());
  });

  /**
   * The stored value stays English because the transcription runtime is handed it
   * and the database keeps it. Only the label moves.
   */
  it('names the language in the reader’s language, not in the stored one', () => {
    chooseLanguage('fr');
    expect(meetingLanguageLabel('German')).toBe('Allemand');
    expect(meetingLanguageLabel('')).toBe('Détecter d’après l’enregistrement');
    chooseLanguage('en');
  });

  it('shows a language nobody listed exactly as it was typed', () => {
    chooseLanguage('de');
    expect(meetingLanguageLabel('Schwyzerdütsch')).toBe('Schwyzerdütsch');
    chooseLanguage('en');
  });

  /** Every choice the picker offers is one the dictionary can name. */
  it('has a name for every language it offers', () => {
    chooseLanguage('ja');
    for (const language of COMMON_MEETING_LANGUAGES) {
      expect(meetingLanguageLabel(language)).not.toBe(language);
    }
    chooseLanguage('en');
  });
});

describe('what the language field holds and what is stored', () => {
  /**
   * The value is handed to the transcription runtime and written into the
   * database, so a French user choosing *Allemand* must produce `German`. This is
   * the identifier-and-label split again, and here getting it wrong would not look
   * like a translation fault: it would transcribe the meeting in the wrong
   * language.
   */
  it('stores the identifier for the name somebody chose', () => {
    chooseLanguage('fr');
    expect(meetingLanguageValue('Allemand')).toBe('German');
    expect(meetingLanguageValue('allemand')).toBe('German');
    chooseLanguage('ja');
    expect(meetingLanguageValue('日本語')).toBe('Japanese');
    chooseLanguage('en');
    expect(meetingLanguageValue('German')).toBe('German');
  });

  it('stores what somebody typed when it names no language it knows', () => {
    chooseLanguage('de');
    expect(meetingLanguageValue('Schwyzerdütsch')).toBe('Schwyzerdütsch');
    expect(meetingLanguageValue('   ')).toBe('');
    chooseLanguage('en');
  });

  /** Empty stays empty, so the placeholder can say what an empty field means. */
  it('leaves an unset language as an empty field', () => {
    chooseLanguage('fr');
    expect(meetingLanguageField('')).toBe('');
    expect(meetingLanguageField(null)).toBe('');
    expect(meetingLanguageField('German')).toBe('Allemand');
    chooseLanguage('en');
  });

  it('round-trips every language the picker offers, in every language', () => {
    for (const interfaceLanguage of ['en', 'de', 'fr', 'es', 'it', 'ja', 'ko', 'zh'] as const) {
      chooseLanguage(interfaceLanguage);
      for (const stored of COMMON_MEETING_LANGUAGES) {
        expect(meetingLanguageValue(meetingLanguageField(stored))).toBe(stored);
      }
    }
    chooseLanguage('en');
  });
});
