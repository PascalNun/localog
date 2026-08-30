import { describe, expect, it } from 'vitest';
import { appendSection, moveSection, newSection, readSections, removeSection } from './sections';

const protocol = [
  '# Protokoll der Sitzung',
  '',
  '## 1. Teilnehmende',
  '',
  'die Bauleitung, die Planung.',
  '',
  '## 2. Fassade',
  '',
  'Die Ausführung wurde besprochen.',
  '',
  '### 2.1 Offene Fragen',
  '',
  'Wer holt das Angebot ein?',
  '',
  '## 3. Nächste Schritte',
  '',
  '| Aufgabe | Wer |',
  '| --- | --- |',
  '| Angebot | Bauleitung |',
].join('\n');

describe('reading a protocol into sections', () => {
  it('takes the level that repeats, not the shallowest', () => {
    // The title is the only `#`, so taking the shallowest outright would make the
    // whole document one section.
    expect(readSections(protocol).map((section) => section.title)).toEqual([
      '1. Teilnehmende',
      '2. Fassade',
      '3. Nächste Schritte',
    ]);
  });

  it('keeps a subsection inside its section rather than beside it', () => {
    const fassade = readSections(protocol)[1];
    const lines = protocol.split('\n').slice(fassade!.from, fassade!.to);
    expect(lines.join('\n')).toContain('2.1 Offene Fragen');
  });

  it('finds nothing in a document with no headings', () => {
    expect(readSections('Nur Fließtext.')).toEqual([]);
  });

  it('does not read a hash inside a code fence as a heading', () => {
    const withCode = ['## Eins', '', '```', '# nicht eine Überschrift', '```', '', '## Zwei'].join(
      '\n',
    );
    expect(readSections(withCode).map((s) => s.title)).toEqual(['Eins', 'Zwei']);
  });
});

describe('rearranging the sections', () => {
  it('moves a section down, carrying everything under it', () => {
    const moved = moveSection(protocol, 0, 1);
    const titles = readSections(moved).map((section) => section.title);
    expect(titles).toEqual(['2. Fassade', '1. Teilnehmende', '3. Nächste Schritte']);
    expect(moved).toContain('2.1 Offene Fragen');
    expect(moved).toContain('die Bauleitung, die Planung.');
  });

  it('moves a section up', () => {
    const titles = readSections(moveSection(protocol, 2, 0)).map((s) => s.title);
    expect(titles).toEqual(['3. Nächste Schritte', '1. Teilnehmende', '2. Fassade']);
  });

  it('leaves the document alone for a move that goes nowhere', () => {
    expect(moveSection(protocol, 1, 1)).toBe(protocol);
    expect(moveSection(protocol, 0, 9)).toBe(protocol);
  });

  /// The property that matters: rearranging must not lose a word.
  it('keeps every line, whatever the order', () => {
    const words = (text: string) => text.split(/\s+/).filter(Boolean).sort().join(' ');
    for (const [from, to] of [
      [0, 2],
      [2, 0],
      [1, 2],
    ]) {
      expect(words(moveSection(protocol, from!, to!))).toBe(words(protocol));
    }
  });
});

describe('taking a section out and putting it back', () => {
  it('gives back what it took, and the document without it', () => {
    const { markdown, removed } = removeSection(protocol, 1);
    expect(removed).toContain('## 2. Fassade');
    expect(removed).toContain('2.1 Offene Fragen');
    expect(markdown).not.toContain('Fassade');
    expect(readSections(markdown).map((s) => s.title)).toEqual([
      '1. Teilnehmende',
      '3. Nächste Schritte',
    ]);
  });

  it('puts it back whole', () => {
    const { markdown, removed } = removeSection(protocol, 1);
    const restored = appendSection(markdown, removed);
    expect(restored).toContain('2.1 Offene Fragen');
    expect(readSections(restored).map((s) => s.title)).toEqual([
      '1. Teilnehmende',
      '3. Nächste Schritte',
      '2. Fassade',
    ]);
  });

  it('adds a new section at the level the document uses', () => {
    const added = appendSection(protocol, newSection(protocol, 'Termine'));
    expect(added).toContain('## Termine');
    expect(readSections(added).map((s) => s.title)).toContain('Termine');
  });
});
