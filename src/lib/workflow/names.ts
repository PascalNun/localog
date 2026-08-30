/**
 * The names somebody types when a project is created, turned into terms.
 *
 * This is the smallest piece of the largest measured improvement in the project.
 * Giving whisper thirty proper nouns as its initial prompt took fourteen counted
 * terms from three spelled correctly to thirteen, and made the word for structural
 * engineering appear at all — without it, it occurs nowhere in seventy-two thousand
 * characters, and the protocol therefore files a named engineer under a discipline
 * he does not practise. No later model can recover from that.
 *
 * The mechanism for using them has existed the whole time. What was missing was the
 * asking.
 *
 * ## Why the separators are what they are
 *
 * Commas, semicolons and line breaks, because those are what people type into a box
 * of names and none of them is worth correcting somebody about.
 *
 * **Never spaces.** A person is "Halde", a client is "Klinker-Nord", a
 * project is "Halle 4". Splitting on spaces would turn every one of those into
 * fragments, and a fragment in the initial prompt is worse than nothing: it biases
 * the transcriber towards half a name.
 */

/** A kind of name, and the category it is stored under. */
export type NameKind = 'Person' | 'Organisation' | 'Project' | 'Technical term';

export interface TypedName {
  term: string;
  category: NameKind;
}

/**
 * Split what somebody typed into separate terms.
 *
 * Repeats are dropped without regard to case, keeping the spelling that was typed
 * first: somebody who writes a name twice meant it once, and the storage layer would
 * refuse the second anyway — better here, where nothing has been written yet and no
 * error has to be shown for something that is not a mistake.
 */
export function parseNames(typed: string): string[] {
  const seen = new Set<string>();
  const terms: string[] = [];
  for (const part of typed.split(/[,;\n\r]+/)) {
    const term = part.trim();
    if (term === '') continue;
    const key = term.toLocaleLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    terms.push(term);
  }
  return terms;
}

/**
 * Everything typed across the fields, as terms with the category their field means.
 *
 * The category is taken from which box a name was typed into rather than asked for,
 * because asking somebody to classify twelve words is how a minute's work becomes
 * five and then does not happen. It is not decoration: when the list outgrows the
 * transcriber's short prompt, people and organisations are what survive the trim.
 *
 * A term repeated across two fields keeps the first field's category, in the order
 * the fields are asked. The alternative is storing it twice, which spends part of
 * that same short prompt saying one thing twice.
 */
export function namesFromFields(fields: Record<NameKind, string>): TypedName[] {
  const order: NameKind[] = ['Person', 'Organisation', 'Project', 'Technical term'];
  const seen = new Set<string>();
  const names: TypedName[] = [];
  for (const category of order) {
    for (const term of parseNames(fields[category] ?? '')) {
      const key = term.toLocaleLowerCase();
      if (seen.has(key)) continue;
      seen.add(key);
      names.push({ term, category });
    }
  }
  return names;
}
