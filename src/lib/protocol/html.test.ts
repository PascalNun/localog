import { describe, expect, it } from 'vitest';
import { toMarkdown, type DomLike } from './html';
import { renderMarkdown } from './markdown';

/** A tiny builder, so the trees below read like the HTML they stand for. */
function node(tag: string, children: (DomLike | string)[] = [], href?: string): DomLike {
  return {
    tag,
    children: children.map((child) =>
      typeof child === 'string' ? { tag: '#text', text: child, children: [] } : child,
    ),
    ...(href ? { href } : {}),
  };
}

const root = (...children: DomLike[]) => node('DIV', children);

describe('reading an edited document back to Markdown', () => {
  it('writes headings at the level they were shown at', () => {
    expect(toMarkdown(root(node('H1', ['Protokoll']), node('H2', ['1. Fassade'])))).toBe(
      '# Protokoll\n\n## 1. Fassade\n',
    );
  });

  it('keeps the marks a person applied', () => {
    const paragraph = node('P', [
      'Frau ',
      node('STRONG', ['Bauleitung']),
      ' und ',
      node('EM', ['Herr Planung']),
      '.',
    ]);
    expect(toMarkdown(root(paragraph))).toBe('Frau **Bauleitung** und *Herr Planung*.\n');
  });

  /// What a browser actually leaves behind, rather than what we would have written.
  it('accepts b and i, which is what an editable region produces', () => {
    expect(toMarkdown(root(node('P', [node('B', ['fett']), ' ', node('I', ['kursiv'])])))).toBe(
      '**fett** *kursiv*\n',
    );
  });

  it('keeps the words inside a tag it does not know', () => {
    const paragraph = node('P', ['Die ', node('SPAN', [node('FONT', ['Fassade'])]), ' wurde.']);
    expect(toMarkdown(root(paragraph))).toBe('Die Fassade wurde.\n');
  });

  it('turns the non-breaking spaces a caret leaves into ordinary ones', () => {
    expect(toMarkdown(root(node('P', ['Ende\u00A0der\u00A0Zeile'])))).toBe('Ende der Zeile\n');
  });

  it('writes both kinds of list', () => {
    const bulleted = node('UL', [node('LI', ['Erster']), node('LI', ['Zweiter'])]);
    const numbered = node('OL', [node('LI', ['Eins']), node('LI', ['Zwei'])]);
    expect(toMarkdown(root(bulleted, numbered))).toBe('- Erster\n- Zweiter\n\n1. Eins\n2. Zwei\n');
  });

  it('indents a list inside a list rather than losing it', () => {
    const nested = node('UL', [node('LI', ['Oben', node('UL', [node('LI', ['Darunter'])])])]);
    expect(toMarkdown(root(nested))).toBe('- Oben\n  - Darunter\n');
  });

  it('writes a table back as a table', () => {
    const table = node('TABLE', [
      node('THEAD', [node('TR', [node('TH', ['Aufgabe']), node('TH', ['Verantwortlich'])])]),
      node('TBODY', [node('TR', [node('TD', ['Angebot']), node('TD', ['Frau Bauleitung'])])]),
    ]);
    expect(toMarkdown(root(table))).toBe(
      '| Aufgabe | Verantwortlich |\n| --- | --- |\n| Angebot | Frau Bauleitung |\n',
    );
  });

  /// A row added to a table is empty until somebody types in it, and it has to
  /// survive being written out and read back or adding a row loses the table.
  it('keeps a table whose cells are empty', () => {
    const table = node('TABLE', [
      node('THEAD', [node('TR', [node('TH', ['Aufgabe']), node('TH', ['Verantwortlich'])])]),
      node('TBODY', [
        node('TR', [node('TD', ['Angebot']), node('TD', [])]),
        node('TR', [node('TD', []), node('TD', [])]),
      ]),
    ]);
    expect(toMarkdown(root(table))).toBe(
      '| Aufgabe | Verantwortlich |\n| --- | --- |\n| Angebot |  |\n|  |  |\n',
    );
  });

  it('pads a row that is short of columns rather than writing a ragged table', () => {
    const table = node('TABLE', [
      node('TR', [node('TH', ['A']), node('TH', ['B']), node('TH', ['C'])]),
      node('TR', [node('TD', ['1'])]),
    ]);
    expect(toMarkdown(root(table))).toBe('| A | B | C |\n| --- | --- | --- |\n| 1 |  |  |\n');
  });

  it('unwraps the div a browser puts round a pasted block', () => {
    const pasted = node('DIV', [node('H2', ['Titel']), node('P', ['Text.'])]);
    expect(toMarkdown(root(pasted))).toBe('## Titel\n\nText.\n');
  });

  it('writes a rule and a quotation', () => {
    expect(toMarkdown(root(node('HR'), node('BLOCKQUOTE', ['Wortlaut.'])))).toBe(
      '---\n\n> Wortlaut.\n',
    );
  });

  it('keeps a link with its destination', () => {
    expect(toMarkdown(root(node('P', [node('A', ['Seite'], 'https://example.org')])))).toBe(
      '[Seite](https://example.org)\n',
    );
  });

  it('does not leave a run of blank lines behind', () => {
    expect(toMarkdown(root(node('P', []), node('P', ['Text.']), node('P', [])))).toBe('Text.\n');
  });

  it('is empty for an empty document', () => {
    expect(toMarkdown(root())).toBe('\n');
  });
});

describe('going out and coming back', () => {
  /**
   * The property that matters: a document nobody edited must not change when the
   * editor merely opens and closes it. Anything else means opening the document
   * view rewrites protocols by itself.
   */
  it('leaves an untouched protocol as it was', () => {
    const markdown = [
      '# Protokoll der Sitzung',
      '',
      '## 1. Teilnehmende',
      '',
      'Frau **Bauleitung** und Herr *Planung*.',
      '',
      '## 2. Optionen',
      '',
      '- Die leichtere Ausführung',
      '- Die schwerere Ausführung',
      '',
      '1. Erste Frage',
      '2. Zweite Frage',
      '',
      '> Wortlaut aus der Sitzung.',
      '',
      '---',
      '',
      '| Aufgabe | Verantwortlich |',
      '| --- | --- |',
      '| Angebot einholen | Frau Bauleitung |',
    ].join('\n');

    // renderMarkdown produces the HTML the editor shows; parsing it back is what
    // the editor does on every keystroke.
    const html = renderMarkdown(markdown);
    const parsed = parseSimpleHtml(html);
    expect(toMarkdown(parsed)).toBe(`${markdown}\n`);
  });
});

/**
 * A very small HTML reader, for this test only.
 *
 * The application uses the browser's own parser through `fromElement`; here there
 * is no browser, and the HTML being read is the HTML this project just wrote, so
 * it needs to handle exactly that and nothing else.
 */
function parseSimpleHtml(html: string): DomLike {
  const rootNode: DomLike = { tag: 'DIV', children: [] };
  const stack: DomLike[] = [rootNode];
  const pattern = /<(\/?)([a-z0-9]+)((?:\s+[a-z-]+="[^"]*")*)\s*(\/?)>|([^<]+)/gi;

  for (const match of html.matchAll(pattern)) {
    const [, closing, tag, attributes, selfClosing, text] = match;
    const top = stack[stack.length - 1];
    if (!top) continue;

    if (text !== undefined) {
      top.children.push({ tag: '#text', text: decode(text), children: [] });
      continue;
    }
    const name = (tag ?? '').toUpperCase();
    if (closing) {
      if (stack.length > 1) stack.pop();
      continue;
    }
    const href = /href="([^"]*)"/.exec(attributes ?? '')?.[1];
    const element: DomLike = { tag: name, children: [], ...(href ? { href } : {}) };
    top.children.push(element);
    if (!selfClosing && name !== 'HR' && name !== 'BR') stack.push(element);
  }
  return rootNode;
}

function decode(text: string): string {
  return text
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&');
}
