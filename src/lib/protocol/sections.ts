/**
 * The parts a protocol is made of, read from its own headings.
 *
 * Nothing is stored to make this work. A protocol already says where its sections
 * are — it is a document with headings — so a second list of them kept alongside
 * would be a second truth to keep in agreement with the first, and the first would
 * win. Everything here reads the Markdown and writes the Markdown.
 */

export interface Section {
  /** The heading's own text, without its hashes. */
  title: string;
  level: number;
  /** Line index of the heading, and of the first line after the section. */
  from: number;
  to: number;
}

const HEADING = /^(#{1,6})\s+(.*)$/;

/**
 * Which heading level counts as a section.
 *
 * The shallowest level that occurs more than once: a protocol usually opens with a
 * single title as `#` and divides into `##`, and taking the shallowest level
 * outright would make the whole document one section. Where nothing repeats, the
 * shallowest is all there is.
 */
function sectionLevel(levels: number[]): number | null {
  if (levels.length === 0) return null;
  const counts = new Map<number, number>();
  for (const level of levels) counts.set(level, (counts.get(level) ?? 0) + 1);
  const repeated = [...counts.entries()].filter(([, count]) => count > 1).map(([level]) => level);
  if (repeated.length > 0) return Math.min(...repeated);
  return Math.min(...levels);
}

export function readSections(markdown: string): Section[] {
  const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
  const headings: { level: number; title: string; at: number }[] = [];
  let insideFence = false;

  lines.forEach((line, at) => {
    if (line.trimStart().startsWith('```')) insideFence = !insideFence;
    if (insideFence) return;
    const found = HEADING.exec(line);
    if (found) {
      headings.push({ level: (found[1] ?? '').length, title: (found[2] ?? '').trim(), at });
    }
  });

  const level = sectionLevel(headings.map((heading) => heading.level));
  if (level === null) return [];

  const starts = headings.filter((heading) => heading.level === level);
  return starts.map((heading, index) => {
    const next = starts[index + 1];
    return {
      title: heading.title,
      level: heading.level,
      from: heading.at,
      to: next ? next.at : lines.length,
    };
  });
}

/** Move a section to another position in the list, and give back the document. */
export function moveSection(markdown: string, from: number, to: number): string {
  const sections = readSections(markdown);
  const moving = sections[from];
  if (!moving || from === to || to < 0 || to >= sections.length) return markdown;

  const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
  const block = lines.slice(moving.from, moving.to);
  const rest = [...lines.slice(0, moving.from), ...lines.slice(moving.to)];

  // Where the target sits once the moving section is out of the way.
  const target = sections[to];
  if (!target) return markdown;
  const before = target.from < moving.from;
  const shift = before ? 0 : moving.from - moving.to;
  const at = (before ? target.from : target.to) + shift;

  return [...rest.slice(0, at), ...block, ...rest.slice(at)].join('\n');
}

/** Take a section out, and give back both the document and what was taken. */
export function removeSection(
  markdown: string,
  index: number,
): { markdown: string; removed: string } {
  const sections = readSections(markdown);
  const going = sections[index];
  if (!going) return { markdown, removed: '' };
  const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
  const removed = lines.slice(going.from, going.to).join('\n').replace(/\n+$/, '');
  const rest = [...lines.slice(0, going.from), ...lines.slice(going.to)];
  return { markdown: tidyBlankLines(rest.join('\n')), removed };
}

/** Put a section back, or add a new one, at the end of the document. */
export function appendSection(markdown: string, block: string): string {
  const trimmed = markdown.replace(/\n+$/, '');
  return `${trimmed}\n\n${block.trim()}\n`;
}

/** A new, empty section at the level this document uses for its sections. */
export function newSection(markdown: string, title: string): string {
  const sections = readSections(markdown);
  const level = sections[0]?.level ?? 2;
  return `${'#'.repeat(level)} ${title.trim() || 'New section'}\n`;
}

/** No more than one blank line in a row, and none at the start. */
function tidyBlankLines(text: string): string {
  return text.replace(/\n{3,}/g, '\n\n').replace(/^\n+/, '');
}
