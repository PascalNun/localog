import { describe, expect, it } from 'vitest';
import { readBlocks, renderInline, renderMarkdown } from './markdown';

describe('reading a protocol into blocks', () => {
  it('reads headings at every level a protocol uses', () => {
    const blocks = readBlocks('# Protokoll\n\n## 1. Fassade\n\n### 1.1 Fenster');
    expect(blocks).toEqual([
      { kind: 'heading', level: 1, text: 'Protokoll' },
      { kind: 'heading', level: 2, text: '1. Fassade' },
      { kind: 'heading', level: 3, text: '1.1 Fenster' },
    ]);
  });

  it('joins the lines of a paragraph, because a protocol is wrapped prose', () => {
    const blocks = readBlocks('Die Fassade wurde\nausführlich besprochen.');
    expect(blocks).toEqual([
      { kind: 'paragraph', text: 'Die Fassade wurde ausführlich besprochen.' },
    ]);
  });

  it('keeps a numbered list numbered', () => {
    const blocks = readBlocks('1. Erste Option\n2. Zweite Option');
    expect(blocks).toEqual([
      { kind: 'list', ordered: true, items: ['Erste Option', 'Zweite Option'] },
    ]);
  });

  it('gathers a wrapped list item back into one item', () => {
    const blocks = readBlocks('- Ein Punkt, der über\n  zwei Zeilen läuft\n- Ein zweiter');
    expect(blocks).toEqual([
      {
        kind: 'list',
        ordered: false,
        items: ['Ein Punkt, der über zwei Zeilen läuft', 'Ein zweiter'],
      },
    ]);
  });

  it('reads the action table the formal style ends with', () => {
    const blocks = readBlocks(
      '| Aufgabe | Verantwortlich |\n| --- | --- |\n| Angebot einholen | die Bauleitung |',
    );
    expect(blocks).toEqual([
      {
        kind: 'table',
        head: ['Aufgabe', 'Verantwortlich'],
        rows: [['Angebot einholen', 'die Bauleitung']],
      },
    ]);
  });

  /// The distinction the Rust side already makes, and for the same reason: a
  /// sentence about a measurement contains pipes and is not a table.
  it('does not read a line of pipes as a table without a divider', () => {
    const blocks = readBlocks('Die Achsen | A | B | wurden geprüft.');
    expect(blocks[0]?.kind).toBe('paragraph');
  });

  it('does not lose a paragraph that follows a list without a blank line', () => {
    const blocks = readBlocks('- Ein Punkt\nDanach ging es weiter.');
    expect(blocks.map((block) => block.kind)).toEqual(['list', 'paragraph']);
  });

  it('terminates on a line it cannot classify rather than looping', () => {
    expect(() => readBlocks('|\n')).not.toThrow();
  });
});

describe('rendering inline text', () => {
  it('escapes anything that looks like markup', () => {
    expect(renderInline('<script>alert(1)</script>')).toBe('&lt;script&gt;alert(1)&lt;/script&gt;');
  });

  it('renders bold and italic', () => {
    expect(renderInline('**Beschluss** und *Vorbehalt*')).toBe(
      '<strong>Beschluss</strong> und <em>Vorbehalt</em>',
    );
  });

  /// German compounds carry underscores rarely, but file names do, and a protocol
  /// quotes file names.
  it('leaves an underscore inside a word alone', () => {
    expect(renderInline('die Datei plan_2026_final.pdf')).toBe('die Datei plan_2026_final.pdf');
  });

  it('does not read an asterisk inside code as emphasis', () => {
    expect(renderInline('`amix=inputs=2:normalize=0` und *danach*')).toBe(
      '<code>amix=inputs=2:normalize=0</code> und <em>danach</em>',
    );
  });

  it('restores code spans without touching numbers in the prose', () => {
    expect(renderInline('in 3 Tagen `x` in 7 Wochen')).toBe(
      'in 3 Tagen <code>x</code> in 7 Wochen',
    );
  });

  it('keeps a real link and refuses one that is not a destination', () => {
    expect(renderInline('[Seite](https://example.org)')).toBe(
      '<a href="https://example.org">Seite</a>',
    );
    expect(renderInline('[Seite](javascript:alert(1))')).toBe('[Seite](javascript:alert(1))');
  });

  it('unescapes the backslashes the model puts before brackets', () => {
    expect(renderInline('Kein Platzhalter wie \\[Datum\\]')).toBe('Kein Platzhalter wie [Datum]');
  });
});

describe('rendering a whole protocol', () => {
  it('produces a document, not a string of paragraphs', () => {
    const html = renderMarkdown(
      [
        '# Protokoll der Sitzung',
        '',
        '## 1. Teilnehmende',
        '',
        'die Bauleitung, die Planung.',
        '',
        '## 2. Nächste Schritte',
        '',
        '| Aufgabe | Verantwortlich |',
        '| --- | --- |',
        '| Angebot einholen | die Bauleitung |',
      ].join('\n'),
    );
    expect(html).toContain('<h1>Protokoll der Sitzung</h1>');
    expect(html).toContain('<h2>1. Teilnehmende</h2>');
    expect(html).toContain('<th>Aufgabe</th>');
    expect(html).toContain('<td>die Bauleitung</td>');
    expect(html).not.toContain('|');
  });

  it('renders an empty document as nothing rather than failing', () => {
    expect(renderMarkdown('')).toBe('');
    expect(renderMarkdown('\n\n  \n')).toBe('');
  });
});
