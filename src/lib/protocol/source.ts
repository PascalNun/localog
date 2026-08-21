/**
 * Finding a passage of the protocol in the transcript it was written from.
 *
 * Not provenance. Nothing records which segment produced which sentence, and
 * pretending otherwise would be inventing a link nobody established — a protocol
 * legitimately paraphrases, gathers one subject from four places, and says things
 * the transcript only implies.
 *
 * What this does is honest and useful: it looks for the words. A passage naming a
 * figure, a date or a person can almost always be found near where it was said, and
 * finding it is what somebody checking a draft against the recording actually wants.
 * Where the protocol has paraphrased, this finds nothing, and says so rather than
 * offering the nearest thing and letting it be read as the source.
 */

export interface SourceHit {
  segmentId: string;
  startMs: number;
  speaker: string;
  text: string;
  /** How many of the passage's distinctive words this segment carries. */
  shared: number;
}

/** Words too common to tell one passage from another. */
const EVERYWHERE = new Set([
  // German, which is what these meetings are in.
  'der',
  'die',
  'das',
  'und',
  'ist',
  'ich',
  'wir',
  'sie',
  'den',
  'dem',
  'des',
  'ein',
  'eine',
  'einen',
  'einem',
  'einer',
  'nicht',
  'auch',
  'noch',
  'dann',
  'aber',
  'wenn',
  'für',
  'auf',
  'mit',
  'von',
  'dass',
  'sich',
  'oder',
  'als',
  'bei',
  'wie',
  'nur',
  'man',
  'schon',
  'hier',
  'haben',
  'werden',
  'wurde',
  'wurden',
  'sind',
  'war',
  // English, for meetings that are.
  'the',
  'and',
  'that',
  'this',
  'with',
  'for',
  'was',
  'were',
  'are',
  'has',
  'have',
  'from',
  'they',
  'their',
  'which',
  'will',
  'would',
  'been',
  'about',
  'into',
]);

/**
 * The words worth searching on.
 *
 * Numbers are kept whatever their length, because a figure is the most identifying
 * thing a protocol carries and "38" is worth more than any adjective.
 */
export function distinctiveWords(passage: string): string[] {
  const seen = new Set<string>();
  for (const raw of passage.toLowerCase().split(/[^\p{L}\p{N},.]+/u)) {
    const word = raw.replace(/^[.,]+|[.,]+$/g, '');
    if (word === '') continue;
    const isNumber = /\d/.test(word);
    if (!isNumber && (word.length < 4 || EVERYWHERE.has(word))) continue;
    seen.add(word);
  }
  return [...seen];
}

/**
 * Where in the transcript this passage's words appear, most of them first.
 *
 * A segment must carry at least two of the distinctive words, or one that is a
 * figure: a single shared adjective is a coincidence, a shared measurement is not.
 */
export function findInSource(
  passage: string,
  segments: { id: string; startMs: number; speaker: string; text: string }[],
  most = 6,
): SourceHit[] {
  const words = distinctiveWords(passage);
  if (words.length === 0) return [];

  const hits: SourceHit[] = [];
  for (const segment of segments) {
    const haystack = segment.text.toLowerCase();
    let shared = 0;
    let hasFigure = false;
    for (const word of words) {
      if (!haystack.includes(word)) continue;
      shared += 1;
      if (/\d/.test(word)) hasFigure = true;
    }
    if (shared >= 2 || (shared === 1 && hasFigure)) {
      hits.push({
        segmentId: segment.id,
        startMs: segment.startMs,
        speaker: segment.speaker,
        text: segment.text,
        shared,
      });
    }
  }

  return hits.sort((a, b) => b.shared - a.shared || a.startMs - b.startMs).slice(0, most);
}
