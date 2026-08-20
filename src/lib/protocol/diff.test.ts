import { describe, expect, it } from 'vitest';
import { diffWords, isUnchanged, type Change } from './diff';

/** The pieces of one kind, joined, so a test can read like the sentence it checks. */
const only = (changes: Change[], kind: Change['kind']) =>
  changes
    .filter((change) => change.kind === kind)
    .map((change) => change.text)
    .join('')
    .trim();

/** Putting the same and removed pieces back together must give the original. */
const rebuild = (changes: Change[], side: 'before' | 'after') =>
  changes
    .filter(
      (change) =>
        change.kind === 'same' || change.kind === (side === 'before' ? 'removed' : 'added'),
    )
    .map((change) => change.text)
    .join('');

describe('showing what a rewrite changed', () => {
  it('says nothing changed when nothing did', () => {
    const changes = diffWords('Die Fläche beträgt 148,5 m².', 'Die Fläche beträgt 148,5 m².');
    expect(isUnchanged(changes)).toBe(true);
  });

  it('names the words that went and the words that came', () => {
    const changes = diffWords(
      'Die Anpassungen wurden aufgrund von Änderungen vorgenommen.',
      'Die Anpassungen wurden wegen Änderungen vorgenommen.',
    );
    expect(only(changes, 'removed')).toBe('aufgrund von');
    expect(only(changes, 'added')).toBe('wegen');
  });

  /// The change this whole panel exists for: a fact altered while the sentence
  /// still reads perfectly well.
  it('shows a figure being rewritten, which is the case that matters', () => {
    const changes = diffWords(
      'Die Anpassungen im 2. Obergeschoss wurden vorgenommen.',
      'Die Anpassungen im Obergeschoss (Etage II) wurden vorgenommen.',
    );
    expect(only(changes, 'removed')).toBe('2.');
    expect(only(changes, 'added')).toContain('(Etage II)');
    expect(isUnchanged(changes)).toBe(false);
  });

  it('can be put back together into either side exactly', () => {
    const before = 'Herr Planung nennt die Kostenspanne bis zum 12. September 2026.';
    const after = 'Die Kostenspanne wird von Herrn Planung bis zum 12. September 2026 genannt.';
    const changes = diffWords(before, after);
    expect(rebuild(changes, 'before')).toBe(before);
    expect(rebuild(changes, 'after')).toBe(after);
  });

  it('handles one side being empty', () => {
    expect(rebuild(diffWords('', 'Neu.'), 'after')).toBe('Neu.');
    expect(rebuild(diffWords('Alt.', ''), 'before')).toBe('Alt.');
  });

  it('falls back to one replacement rather than choking on a huge passage', () => {
    const long = 'Wort '.repeat(2_000);
    const changes = diffWords(long, `${long}extra`);
    expect(changes).toHaveLength(2);
    expect(changes[0]?.kind).toBe('removed');
    expect(changes[1]?.kind).toBe('added');
  });
});
