import { describe, expect, it } from 'vitest';
import { atMoment, distinctiveWords, findInSource } from './source';

const segments = [
  { id: 's1', startMs: 0, speaker: 'Frau Bauleitung', text: 'Guten Morgen, wir fangen an.' },
  {
    id: 's2',
    startMs: 62_000,
    speaker: 'Herr Planung',
    text: 'Die betroffene Fläche beträgt 148,5 m² im zweiten Obergeschoss.',
  },
  {
    id: 's3',
    startMs: 130_000,
    speaker: 'Frau Bauleitung',
    text: 'Die Heizlastberechnung liegt bis KW 38 vor, die Kosten etwa 4.200 Euro.',
  },
  { id: 's4', startMs: 190_000, speaker: 'Herr Planung', text: 'Ja, das sehe ich auch so.' },
];

describe('the words worth searching on', () => {
  it('drops the words every sentence has', () => {
    const words = distinctiveWords('Die Fläche ist und das war nicht');
    expect(words).toContain('fläche');
    expect(words).not.toContain('die');
    expect(words).not.toContain('nicht');
  });

  /// A figure is the most identifying thing a protocol carries.
  it('keeps a number however short', () => {
    expect(distinctiveWords('bis KW 38')).toContain('38');
    expect(distinctiveWords('148,5 m²')).toContain('148,5');
  });
});

describe('finding a passage in the transcript', () => {
  it('finds where a figure was said', () => {
    const hits = findInSource('Die Fläche beträgt 148,5 m².', segments);
    expect(hits[0]?.segmentId).toBe('s2');
  });

  it('ranks the segment sharing most of the passage first', () => {
    const hits = findInSource('Heizlastberechnung bis KW 38, Kosten 4.200 Euro', segments);
    expect(hits[0]?.segmentId).toBe('s3');
    expect(hits[0]?.shared).toBeGreaterThan(1);
  });

  /// The case this must not get wrong: a paraphrase has no source to point at, and
  /// offering the nearest segment would let a guess be read as provenance.
  it('finds nothing rather than the nearest thing when the words are not there', () => {
    expect(findInSource('Der Vorstand hat zugestimmt.', segments)).toEqual([]);
  });

  it('ignores a single shared ordinary word', () => {
    expect(findInSource('Morgen besprechen wir etwas anderes.', segments)).toEqual([]);
  });

  it('is empty for a passage with nothing distinctive in it', () => {
    expect(findInSource('und das ist so', segments)).toEqual([]);
  });
});

describe('reading a timestamp', () => {
  it('says it the way a transcript does', () => {
    expect(atMoment(0)).toBe('0:00');
    expect(atMoment(62_000)).toBe('1:02');
    expect(atMoment(3_725_000)).toBe('1:02:05');
  });
});
