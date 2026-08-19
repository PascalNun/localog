/**
 * The document back to Markdown, after somebody has typed into it.
 *
 * Markdown stays the stored form. The editable surface is HTML because that is
 * what a person can be shown and can type into, so every edit has to come home
 * again — and what a browser leaves behind in an editable region is not the HTML
 * this project wrote. It inserts `div`s where paragraphs were asked for, `span`s
 * carrying inline styles, `b` where `strong` went in, and non-breaking spaces at
 * the ends of lines.
 *
 * So this reads only the vocabulary a protocol has, and anything it does not know
 * it reduces to the text inside it. Losing a stray `span` is right; losing the
 * words inside it never is.
 *
 * The tree is described by a small interface rather than taken as DOM nodes, so
 * that the rules can be tested without a browser or a simulated one. `fromElement`
 * is the only part that touches a real document.
 */

/** A node, as much of one as this needs to know about. */
export interface DomLike {
  /** An uppercase tag name, or `#text`. */
  tag: string;
  text?: string;
  href?: string;
  children: DomLike[];
}

const HEADINGS: Record<string, number> = { H1: 1, H2: 2, H3: 3, H4: 4, H5: 5, H6: 6 };
/** Everything that stands on its own line, as opposed to marking a run of words. */
const BLOCKS = new Set([
  'P',
  'DIV',
  'UL',
  'OL',
  'LI',
  'BLOCKQUOTE',
  'HR',
  'TABLE',
  'THEAD',
  'TBODY',
  'TR',
  ...Object.keys(HEADINGS),
]);

/** Read a real element, once, at the edge. */
export function fromElement(element: Element): DomLike {
  const children: DomLike[] = [];
  for (const child of Array.from(element.childNodes)) {
    if (child.nodeType === 3) {
      children.push({ tag: '#text', text: child.nodeValue ?? '', children: [] });
    } else if (child.nodeType === 1) {
      children.push(fromElement(child as Element));
    }
  }
  const href = element.getAttribute?.('href');
  return {
    tag: element.tagName.toUpperCase(),
    children,
    ...(href ? { href } : {}),
  };
}

/** The whole editable region as Markdown. */
export function toMarkdown(root: DomLike): string {
  const written = root.children
    .map((child) => blockToMarkdown(child))
    .filter((part) => part.trim() !== '')
    .join('\n\n');
  // One trailing newline, and never a run of blank lines: the stored form should
  // not churn just because somebody pressed return at the end.
  return `${written.replace(/\n{3,}/g, '\n\n').trimEnd()}\n`;
}

function blockToMarkdown(node: DomLike): string {
  const level = HEADINGS[node.tag];
  if (level !== undefined) {
    return `${'#'.repeat(level)} ${inline(node).trim()}`;
  }

  switch (node.tag) {
    case '#text': {
      const text = clean(node.text ?? '');
      return text.trim() === '' ? '' : text;
    }
    case 'HR':
      return '---';
    case 'BR':
      return '';
    case 'UL':
    case 'OL':
      return node.children
        .filter((child) => child.tag === 'LI')
        .map((item, index) => {
          const marker = node.tag === 'OL' ? `${index + 1}.` : '-';
          // A list inside a list keeps its indent; deeper than that a protocol
          // does not go, and flattening is better than inventing structure.
          const nested = item.children
            .filter((child) => child.tag === 'UL' || child.tag === 'OL')
            .map((child) => indent(blockToMarkdown(child)))
            .join('\n');
          const own = inline({
            ...item,
            children: item.children.filter((child) => child.tag !== 'UL' && child.tag !== 'OL'),
          }).trim();
          return `${marker} ${own}${nested ? `\n${nested}` : ''}`;
        })
        .filter((line) => line.trim() !== '-' && line.trim() !== '')
        .join('\n');
    case 'BLOCKQUOTE':
      return inline(node)
        .trim()
        .split('\n')
        .map((line) => `> ${line}`)
        .join('\n');
    case 'TABLE':
      return tableToMarkdown(node);
    case 'DIV':
    case 'P':
    default: {
      // A div holding blocks is a wrapper, not a paragraph — which is what an
      // editable region leaves behind when somebody pastes.
      if (node.children.some((child) => BLOCKS.has(child.tag))) {
        return node.children
          .map((child) => blockToMarkdown(child))
          .filter((part) => part.trim() !== '')
          .join('\n\n');
      }
      return inline(node).trim();
    }
  }
}

function indent(block: string): string {
  return block
    .split('\n')
    .map((line) => `  ${line}`)
    .join('\n');
}

function tableToMarkdown(node: DomLike): string {
  const rows: string[][] = [];
  const walk = (current: DomLike) => {
    if (current.tag === 'TR') {
      rows.push(
        current.children
          .filter((cell) => cell.tag === 'TD' || cell.tag === 'TH')
          .map((cell) => inline(cell).trim().replace(/\|/g, '\\|')),
      );
      return;
    }
    current.children.forEach(walk);
  };
  node.children.forEach(walk);
  if (rows.length === 0) return '';

  const width = Math.max(...rows.map((row) => row.length));
  const line = (row: string[]) =>
    `| ${Array.from({ length: width }, (_, index) => row[index] ?? '').join(' | ')} |`;
  const [head, ...body] = rows;
  return [
    line(head ?? []),
    `| ${Array.from({ length: width }, () => '---').join(' | ')} |`,
    ...body.map(line),
  ].join('\n');
}

/** A run of words, with the marks a protocol carries. */
function inline(node: DomLike): string {
  return node.children.map(markToMarkdown).join('');
}

function markToMarkdown(node: DomLike): string {
  if (node.tag === '#text') return escapeMarkdown(clean(node.text ?? ''));
  if (node.tag === 'BR') return '\n';

  const inside = inline(node);
  if (inside.trim() === '') return inside;

  switch (node.tag) {
    case 'STRONG':
    case 'B':
      return `**${inside}**`;
    case 'EM':
    case 'I':
      return `*${inside}*`;
    case 'CODE':
      return `\`${inside}\``;
    case 'A':
      return node.href ? `[${inside}](${node.href})` : inside;
    default:
      // A span, a font, whatever the editor left: keep the words, drop the tag.
      return inside;
  }
}

/**
 * What an editable region puts in the text and a document should not keep.
 *
 * The non-breaking space is the one that matters: browsers insert it to hold a
 * caret at the end of a line, and it survives into the file as a character that
 * looks like a space and is not one.
 */
function clean(text: string): string {
  return text.replace(/\u00A0/g, ' ').replace(/\r/g, '');
}

/**
 * Punctuation that would be read back as markup.
 *
 * Only where it would actually change the reading: a lone asterisk in prose is
 * left alone, because escaping every one of them makes the stored file unreadable
 * for the sake of a case that does not arise.
 */
function escapeMarkdown(text: string): string {
  return text.replace(/([*_`])\1?(?=\S)/g, (match) => `\\${match.split('').join('\\')}`);
}
