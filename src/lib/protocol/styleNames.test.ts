import { describe, expect, it } from 'vitest';
import { chooseLanguage } from '../i18n';
import { everyRustSource } from '../i18n/rustSource';
import { styleDescription, styleName } from './styleNames';

const FORMAL = {
  id: 'style-formal',
  name: 'Formal minutes',
  description: 'Structured record of discussion, decisions, and actions.',
};

describe('what a protocol style is called', () => {
  it('names a shipped style in the reader’s language', () => {
    chooseLanguage('fr');
    expect(styleName(FORMAL)).toBe('Compte rendu officiel');
    expect(styleDescription(FORMAL)).toContain('structuré');
    chooseLanguage('ja');
    expect(styleName(FORMAL)).toBe('正式な議事録');
    chooseLanguage('en');
    expect(styleName(FORMAL)).toBe('Formal minutes');
  });

  /**
   * The condition the whole thing rests on. A style somebody renamed is theirs,
   * and showing them a translation of what it used to be called would be losing
   * their work rather than translating it.
   */
  it('leaves a renamed style exactly as it was renamed', () => {
    chooseLanguage('fr');
    expect(styleName({ ...FORMAL, name: 'Procès-verbal du conseil' })).toBe(
      'Procès-verbal du conseil',
    );
    expect(styleDescription({ ...FORMAL, description: 'Ce que nous faisons ici.' })).toBe(
      'Ce que nous faisons ici.',
    );
    chooseLanguage('en');
  });

  it('leaves a style somebody made from scratch alone', () => {
    chooseLanguage('de');
    expect(styleName({ id: 'style-abc123', name: 'Bauleitung' })).toBe('Bauleitung');
    chooseLanguage('en');
  });
});

/**
 * `styleNames.ts` repeats the text the migration seeds, because it compares
 * against it rather than showing it. A repetition nothing checks is a repetition
 * that drifts, and the drift here would be silent: the comparison would stop
 * matching and every reader would quietly get the English name back.
 */
describe('the text a shipped style arrives with', () => {
  it('is the text the migration actually writes', () => {
    const storage = everyRustSource().find(({ path }) => path.endsWith('storage.rs'));
    expect(storage).toBeDefined();
    const migration = storage!.text;
    for (const style of [
      FORMAL,
      {
        id: 'style-working-note',
        name: 'Internal working note',
        description: 'Concise working record for an internal project team.',
      },
      {
        id: 'style-decision-log',
        name: 'Technical decision log',
        description: 'Emphasises alternatives, constraints, and explicit decisions.',
      },
    ]) {
      // What the interface would show if the comparison held: proof that it does.
      chooseLanguage('fr');
      expect(styleName(style)).not.toBe(style.name);
      chooseLanguage('en');
      expect(migration).toContain(`'${style.name}'`);
      expect(migration).toContain(`'${style.description}'`);
    }
  });
});
