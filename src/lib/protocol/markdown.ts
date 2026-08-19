/**
 * Markdown to HTML, for the one document this application produces.
 *
 * Written rather than installed, for three reasons. A protocol's Markdown is not
 * arbitrary — it comes from prompts this project wrote, and is headings, prose,
 * lists, tables and emphasis, which is a small grammar. A general library would
 * also carry raw HTML passthrough, which is a hole in a document assembled from
 * model output. And this renderer is read by the editor, the PDF and the DOCX
 * alike, so what it does needs to be knowable rather than configurable.
 *
 * Nothing here trusts its input: every character that is not part of a construct
 * this file recognises is escaped, and no HTML in the source survives as HTML.
 */

/** A block of the document, once the lines have been grouped. */
export type Block =
  | { kind: 'heading'; level: number; text: string }
  | { kind: 'paragraph'; text: string }
  | { kind: 'list'; ordered: boolean; items: string[] }
  | { kind: 'quote'; lines: string[] }
  | { kind: 'table'; head: string[]; rows: string[][] }
  | { kind: 'rule' };

const HEADING = /^(#{1,6})\s+(.*)$/;
const UNORDERED = /^[-*+]\s+(.*)$/;
const ORDERED = /^(\d+)[.)]\s+(.*)$/;
const QUOTE = /^>\s?(.*)$/;
const RULE = /^\s*([-*_])(\s*\1){2,}\s*$/;
const TABLE_ROW = /^\s*\|(.+)\|\s*$/;
/** The row of dashes that makes the line above it a table header. */
const TABLE_DIVIDER = /^\s*\|?\s*:?-{1,}:?\s*(\|\s*:?-{1,}:?\s*)*\|?\s*$/;

/**
 * Group the lines of a document into blocks.
 *
 * Separate from rendering because DOCX needs the same grouping and does not want
 * HTML — the structure is the shared thing, not the markup.
 */
export function readBlocks(markdown: string): Block[] {
  const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
  const blocks: Block[] = [];
  // Reading past the end gives an empty line rather than nothing, so that every
  // look-ahead below is one comparison instead of two.
  const at = (index: number): string => lines[index] ?? '';
  let cursor = 0;

  while (cursor < lines.length) {
    const line = at(cursor);

    if (line.trim() === '') {
      cursor += 1;
      continue;
    }

    if (RULE.test(line)) {
      blocks.push({ kind: 'rule' });
      cursor += 1;
      continue;
    }

    const heading = HEADING.exec(line);
    if (heading) {
      blocks.push({
        kind: 'heading',
        level: (heading[1] ?? '').length,
        text: (heading[2] ?? '').trim(),
      });
      cursor += 1;
      continue;
    }

    // A table is a row of cells with a row of dashes beneath it. Without the
    // dashes it is a paragraph that happens to contain pipes, which is what a
    // sentence about a measurement looks like.
    if (TABLE_ROW.test(line) && TABLE_DIVIDER.test(at(cursor + 1))) {
      const head = splitRow(line);
      const rows: string[][] = [];
      cursor += 2;
      while (cursor < lines.length && TABLE_ROW.test(at(cursor))) {
        rows.push(splitRow(at(cursor)));
        cursor += 1;
      }
      blocks.push({ kind: 'table', head, rows });
      continue;
    }

    if (QUOTE.test(line)) {
      const quoted: string[] = [];
      while (cursor < lines.length && QUOTE.test(at(cursor))) {
        quoted.push(QUOTE.exec(at(cursor))?.[1] ?? '');
        cursor += 1;
      }
      blocks.push({ kind: 'quote', lines: quoted });
      continue;
    }

    if (UNORDERED.test(line) || ORDERED.test(line)) {
      const ordered = ORDERED.test(line);
      const items: string[] = [];
      while (cursor < lines.length) {
        const current = at(cursor);
        const item = ordered ? ORDERED.exec(current) : UNORDERED.exec(current);
        if (!item) break;
        items.push(((ordered ? item[2] : item[1]) ?? '').trim());
        cursor += 1;
        // A wrapped item continues on an indented line that starts nothing else.
        while (
          cursor < lines.length &&
          /^\s{2,}\S/.test(at(cursor)) &&
          startsNothing(at(cursor).trim())
        ) {
          items[items.length - 1] = `${items[items.length - 1] ?? ''} ${at(cursor).trim()}`;
          cursor += 1;
        }
      }
      blocks.push({ kind: 'list', ordered, items });
      continue;
    }

    // Anything else is prose, and runs until a blank line or a line that starts
    // something else.
    const prose: string[] = [];
    while (cursor < lines.length && at(cursor).trim() !== '' && startsNothing(at(cursor))) {
      prose.push(at(cursor).trim());
      cursor += 1;
    }
    if (prose.length === 0) {
      // A line that starts something but was not consumed above; keep it rather
      // than looping for ever on it.
      prose.push(line.trim());
      cursor += 1;
    }
    blocks.push({ kind: 'paragraph', text: prose.join(' ') });
  }

  return blocks;
}

function startsNothing(line: string): boolean {
  return !(
    HEADING.test(line) ||
    UNORDERED.test(line) ||
    ORDERED.test(line) ||
    QUOTE.test(line) ||
    RULE.test(line) ||
    TABLE_ROW.test(line)
  );
}

function splitRow(line: string): string[] {
  const inner = TABLE_ROW.exec(line)?.[1] ?? '';
  return inner.split('|').map((cell) => cell.trim());
}

/** Render a whole document. */
export function renderMarkdown(markdown: string): string {
  return readBlocks(markdown).map(renderBlock).join('\n');
}

function renderBlock(block: Block): string {
  switch (block.kind) {
    case 'rule':
      return '<hr />';
    case 'heading':
      return `<h${block.level}>${renderInline(block.text)}</h${block.level}>`;
    case 'paragraph':
      return `<p>${renderInline(block.text)}</p>`;
    case 'quote':
      return `<blockquote>${renderInline(block.lines.join(' '))}</blockquote>`;
    case 'list': {
      const tag = block.ordered ? 'ol' : 'ul';
      const items = block.items.map((item) => `<li>${renderInline(item)}</li>`).join('');
      return `<${tag}>${items}</${tag}>`;
    }
    case 'table': {
      const head = block.head.map((cell) => `<th>${renderInline(cell)}</th>`).join('');
      const body = block.rows
        .map((row) => `<tr>${row.map((cell) => `<td>${renderInline(cell)}</td>`).join('')}</tr>`)
        .join('');
      return `<table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>`;
    }
  }
}

const ESCAPES: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;',
};

export function escapeHtml(text: string): string {
  return text.replace(/[&<>"']/g, (character) => ESCAPES[character] ?? character);
}

/** A stretch of text with one set of marks on it. */
export interface Run {
  text: string;
  bold?: boolean;
  italic?: boolean;
  code?: boolean;
  /** Present only when the destination is one a document can honestly carry. */
  link?: string;
}

/**
 * Emphasis, code and links inside one run of text.
 *
 * A scanner rather than a series of replacements, because HTML is not the only
 * thing this has to become: a DOCX needs the same marks as runs, and two parsers
 * that must agree eventually do not. Everything HTML-shaped is escaped by whoever
 * renders these runs, never here.
 */
export function readInline(text: string): Run[] {
  const runs: Run[] = [];
  let plain = '';
  let at = 0;

  const flush = () => {
    if (plain) runs.push({ text: plain });
    plain = '';
  };
  const mark = (content: string, marks: Omit<Run, 'text'>) => {
    flush();
    if (content) runs.push({ text: content, ...marks });
  };
  const isWord = (character: string | undefined) => !!character && /[\w*_]/.test(character);

  while (at < text.length) {
    const character = text[at] ?? '';

    // A backslash before punctuation means the character itself. The model writes
    // these around brackets, so that "[Datum]" is not read as a link.
    if (
      character === '\\' &&
      at + 1 < text.length &&
      /[\\`*_{}[\]()#+\-.!|]/.test(text[at + 1] ?? '')
    ) {
      plain += text[at + 1];
      at += 2;
      continue;
    }

    if (character === '`') {
      const close = text.indexOf('`', at + 1);
      if (close > at) {
        mark(text.slice(at + 1, close), { code: true });
        at = close + 1;
        continue;
      }
    }

    if (character === '*' && text[at + 1] === '*') {
      const close = text.indexOf('**', at + 2);
      if (close > at + 1) {
        mark(text.slice(at + 2, close), { bold: true });
        at = close + 2;
        continue;
      }
    }

    if ((character === '*' || character === '_') && !isWord(text[at - 1])) {
      const close = text.indexOf(character, at + 1);
      if (close > at + 1 && !isWord(text[close + 1])) {
        mark(text.slice(at + 1, close), { italic: true });
        at = close + 1;
        continue;
      }
    }

    if (character === '[') {
      const label = text.indexOf(']', at + 1);
      if (label > at && text[label + 1] === '(') {
        const close = text.indexOf(')', label + 2);
        const target = close > label + 1 ? text.slice(label + 2, close) : '';
        if (close > label + 1 && !/\s/.test(target) && safeLink(target)) {
          mark(text.slice(at + 1, label), { link: target });
          at = close + 1;
          continue;
        }
      }
    }

    plain += character;
    at += 1;
  }

  flush();
  return runs;
}

/** Those runs as HTML. */
export function renderInline(text: string): string {
  return readInline(text).map(runToHtml).join('');
}

function runToHtml(run: Run): string {
  let html = escapeHtml(run.text);
  if (run.code) return `<code>${html}</code>`;
  if (run.bold) html = `<strong>${html}</strong>`;
  if (run.italic) html = `<em>${html}</em>`;
  if (run.link) html = `<a href="${escapeHtml(run.link)}">${html}</a>`;
  return html;
}

/** Only destinations a document can honestly carry. */
function safeLink(target: string): boolean {
  return /^(https?:\/\/|mailto:)/i.test(target);
}
