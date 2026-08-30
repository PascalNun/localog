import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { en } from './en';
import { de } from './de';

/**
 * Three guards over the words themselves, written on 30 August 2026 after a
 * fourth sweep found three constructs the previous three could not see.
 *
 * The pattern by now is clear enough to name: **each sweep finds only what its
 * pattern matches, so a sweep is never the check.** August's searches read text
 * between a `>` and a `<`, then attributes, then multi-line Rust literals; what
 * was left after all of them was prose stored as data, an attribute whose value
 * was a template literal, a constant in `types.ts`, and eleven plain text nodes
 * holding an interpolation.
 *
 * None of the three below finds everything either — the first cannot see a
 * language nobody has added, and the other two read markup with regular
 * expressions, which is a known way to be wrong. They differ from a sweep in that
 * they fail rather than reassure, and each was checked against the fault it
 * describes before it was kept. The second one silently found nothing on its
 * first attempt.
 */

const SOURCE = join(process.cwd(), 'src');

/**
 * Words that are genuinely the same in English and German.
 *
 * Kept as values rather than as paths, so a *new* key holding `Status` passes
 * without anybody editing this list — the list is about the words, not about
 * where they happen to be used.
 */
const SAME_IN_BOTH = new Set([
  'PDF',
  'Word',
  'Markdown',
  'Apache 2.0',
  'Optional',
  'optional',
  'Status',
  'Name',
  'Person',
  'Organisation',
  'Export',
  // Placeholder examples, which are names rather than words.
  'Halde, Prüfstelle, Fachplanung',
  'HOAI, Klinker-Nord',
  'Halle 4, Halle 4',
  'Tragwerk, Clusterwohnung',
]);

function untranslated(english: unknown, german: unknown, path: string, found: string[]) {
  if (typeof english === 'string') {
    if (english === german && english.trim() !== '' && !SAME_IN_BOTH.has(english)) {
      found.push(`${path}: ${english}`);
    }
    return;
  }
  if (english && typeof english === 'object') {
    for (const key of Object.keys(english as object)) {
      untranslated(
        (english as Record<string, unknown>)[key],
        (german as Record<string, unknown> | undefined)?.[key],
        `${path}.${key}`,
        found,
      );
    }
  }
}

describe('the dictionary', () => {
  /**
   * German is typed against English, so a key that is *missing* is already a
   * compile error. What typing cannot see is a key added to both files and
   * translated in neither, which is what happens when somebody adds a string in a
   * hurry — and it reads correctly to the one person who speaks English.
   */
  it('says something different in German', () => {
    const found: string[] = [];
    untranslated(en, de, 'en', found);
    expect(found).toEqual([]);
  });
});

/** Attributes that carry something a person reads or hears. */
const SPOKEN = ['aria-label', 'aria-valuetext', 'aria-placeholder', 'title', 'placeholder', 'alt'];

function svelteFiles(directory: string): string[] {
  const found: string[] = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    if (statSync(path).isDirectory()) found.push(...svelteFiles(path));
    else if (entry.endsWith('.svelte')) found.push(path);
  }
  return found;
}

/**
 * The expression an attribute is given, from the opening brace to its match.
 *
 * Counting braces rather than looking for the next `}`, because every template
 * literal worth catching contains a `${}` hole and the first closing brace is
 * inside it. Written the naive way first, and it silently found nothing — which
 * is why the guard was checked against a fault before it was kept.
 */
function expressionAt(source: string, open: number): string {
  let depth = 0;
  for (let at = open; at < source.length; at += 1) {
    if (source[at] === '{') depth += 1;
    else if (source[at] === '}') {
      depth -= 1;
      if (depth === 0) return source.slice(open, at + 1);
    }
  }
  return source.slice(open);
}

/** Whether a stretch of text is words somebody reads rather than an identifier. */
function readsAsWords(text: string): boolean {
  return /[A-Za-z]{2}/.test(text) && /[A-Za-z]\s+[A-Za-z]/.test(text);
}

/**
 * Attribute values that hold words rather than a call into the dictionary.
 *
 * Three shapes, and only the first was ever caught by looking: a quoted value
 * (`aria-label="Remove the line at {time}"`), an expression holding a template
 * literal whose text lives outside its `${}` holes
 * (``aria-label={`Rename ${speaker}`}``), and a quoted sentence inside a ternary.
 * A bare `{expression}` is assumed to come from somewhere this cannot see,
 * because it usually does.
 */
function hardCoded(source: string): string[] {
  const found: string[] = [];
  for (const name of SPOKEN) {
    for (const match of source.matchAll(new RegExp(`\\b${name}=(["'{])`, 'g'))) {
      const quote = match[1] ?? '{';
      const at = match.index + match[0].length;
      if (quote !== '{') {
        const end = source.indexOf(quote, at);
        const value = source.slice(at, end === -1 ? at : end);
        // A `{…}` hole is a value from elsewhere; what surrounds it is the words.
        if (/[A-Za-z]{2}/.test(value.replace(/\{[^}]*\}/g, ''))) found.push(`${name}="${value}"`);
        continue;
      }
      const expression = expressionAt(source, at - 1);
      for (const template of expression.matchAll(/`([^`]*)`/g)) {
        const text = (template[1] ?? '').replace(/\$\{[^}]*\}/g, '');
        if (/[A-Za-z]{2}/.test(text)) found.push(`${name}={\`${template[1]}\`}`);
      }
      for (const quoted of expression.matchAll(/'([^']*)'|"([^"]*)"/g)) {
        const text = quoted[1] ?? quoted[2] ?? '';
        if (readsAsWords(text)) found.push(`${name}={… '${text}' …}`);
      }
    }
  }
  return found;
}

describe('what the interface says through an attribute', () => {
  it('comes from the dictionary, not from the markup', () => {
    const offenders: string[] = [];
    for (const path of svelteFiles(SOURCE)) {
      for (const found of hardCoded(readFileSync(path, 'utf8'))) {
        offenders.push(`${path.slice(SOURCE.length + 1)} — ${found}`);
      }
    }
    expect(offenders).toEqual([]);
  });
});

/**
 * The markup of a component, with everything that is not markup removed.
 *
 * Svelte puts the script first, so the template is what follows the *last*
 * `</script>` — taken that way rather than by a non-greedy match, because
 * `App.svelte` has two script blocks and matching the first left half its
 * instance script looking like page text.
 */
function template(source: string): string {
  const at = source.lastIndexOf('</script>');
  const markup = at === -1 ? source : source.slice(at + '</script>'.length);
  const withoutQuotedAttributes = markup
    .replace(/<style[^>]*>[\s\S]*?<\/style>/g, '')
    .replace(/<!--[\s\S]*?-->/g, '')
    // Attributes belong to the guard above, and a class list is not a sentence.
    .replace(/\s[a-zA-Z:_-]+=("[^"]*"|'[^']*')/g, '');
  return withoutBraces(withoutQuotedAttributes);
}

/**
 * Every `{…}` group removed, counting braces rather than matching a pattern.
 *
 * A regex cannot do this: an `onclick` handler and an `{#each}` header both hold
 * braces of their own and both run over several lines, and a non-greedy
 * `\{[^}]*\}` cuts them in half — leaving the second half looking exactly like
 * page text. Eight false findings, all of them JavaScript, before this was
 * written properly.
 */
function withoutBraces(markup: string): string {
  let kept = '';
  let depth = 0;
  for (const character of markup) {
    if (character === '{') depth += 1;
    else if (character === '}') depth = Math.max(0, depth - 1);
    else if (depth === 0) kept += character;
  }
  return kept;
}

/**
 * Text a component says in its own markup rather than through the dictionary.
 *
 * A `{…}` hole is a value from elsewhere, so it is removed before looking; what
 * is left is what the file itself says. Two words, or one capitalised word on its
 * own, is the threshold — below that the false positives outnumber the findings,
 * and a single lowercase word is almost always punctuation or a unit.
 */
function saysItself(source: string): string[] {
  const found: string[] = [];
  for (const line of template(source).split('\n')) {
    const between = line.matchAll(/>([^<>]*)(?=<|$)/g);
    for (const match of between) {
      const text = (match[1] ?? '').trim();
      if (/[A-Za-z]{2,}\s+[A-Za-z]{2,}/.test(text) || /^[A-Z][a-z]{3,}$/.test(text)) {
        found.push(text.slice(0, 60));
      }
    }
    const bare = line.trim();
    if (
      bare &&
      !/^[<>/*#:}]/.test(bare) &&
      /^[A-Za-z]{2,}\s+[A-Za-z]{2,}/.test(bare) &&
      !bare.includes('<')
    ) {
      found.push(bare.slice(0, 60));
    }
  }
  return found;
}

describe('what the interface says in its own markup', () => {
  it('comes from the dictionary too', () => {
    const offenders: string[] = [];
    for (const path of svelteFiles(SOURCE)) {
      for (const said of saysItself(readFileSync(path, 'utf8'))) {
        offenders.push(`${path.slice(SOURCE.length + 1)} — ${said}`);
      }
    }
    expect(offenders).toEqual([]);
  });
});
