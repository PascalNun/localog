/**
 * How a protocol is set, turned into the numbers each surface needs.
 *
 * One place decides what "11 pt, standard headings, comfortable" means, and the
 * screen, the PDF and the Word file all read it here. The alternative is three
 * interpretations that agree until somebody changes one of them.
 */

import type { DocumentAppearance } from '../workflow/types';

/** Points to CSS pixels at the usual 96dpi reading of a point. */
const PIXELS_PER_POINT = 96 / 72;

const HEADING_SCALE = { compact: 1.12, standard: 1.22, large: 1.34 } as const;
const LINE_SPACING = { compact: 1.4, comfortable: 1.62, spacious: 1.85 } as const;
/**
 * The text measure, counted in characters of the document's own body size.
 *
 * Turned into an absolute length below rather than left as `em`, because `em`
 * resolves against whatever element reads it: the sheet sets its own font size, the
 * toolbar above it does not, and the same "46em" came out 675px on one and 552px on
 * the other. Two things that are meant to be the same width were not.
 */
const PAGE_WIDTH = { narrow: 32, standard: 40, wide: 52, a4: 46 } as const;

/**
 * The custom properties the document stylesheet reads.
 *
 * Returned as a style string rather than applied here, so that the editor and the
 * print sheet can each put it where they need it.
 */
export function appearanceStyle(appearance: DocumentAppearance): string {
  const step = HEADING_SCALE[appearance.headingScale];
  const bodyPixels = appearance.bodySize * PIXELS_PER_POINT;
  return [
    `--document-font: ${fontStack(appearance.font)}`,
    `--document-size: ${bodyPixels.toFixed(2)}px`,
    `--document-leading: ${LINE_SPACING[appearance.lineSpacing]}`,
    `--document-measure: ${(PAGE_WIDTH[appearance.pageWidth] * bodyPixels).toFixed(1)}px`,
    // Each heading is one step of the scale above the one below it, so choosing
    // "large" moves the whole hierarchy rather than only the top of it.
    `--heading-1: ${(step * step * step).toFixed(3)}em`,
    `--heading-2: ${(step * step).toFixed(3)}em`,
    `--heading-3: ${step.toFixed(3)}em`,
    `--heading-4: 1em`,
  ].join('; ');
}

export function fontStack(font: DocumentAppearance['font']): string {
  switch (font) {
    case 'georgia':
      return "Georgia, 'Times New Roman', serif";
    case 'times-new-roman':
      return "'Times New Roman', Times, serif";
    case 'arial':
      return 'Arial, Helvetica, sans-serif';
    case 'calibri':
      return 'Calibri, Carlito, system-ui, sans-serif';
    default:
      return "'Barlow', system-ui, sans-serif";
  }
}

/** What a Word document must be told, which is one family and no fallback. */
export function wordFontName(font: DocumentAppearance['font']): string {
  switch (font) {
    case 'georgia':
      return 'Georgia';
    case 'times-new-roman':
      return 'Times New Roman';
    case 'arial':
      return 'Arial';
    case 'calibri':
      return 'Calibri';
    default:
      return 'Barlow';
  }
}

/** Word gives sizes in half-points, and heading sizes follow the same scale. */
export function wordSizes(appearance: DocumentAppearance) {
  const step = HEADING_SCALE[appearance.headingScale];
  const half = (size: number) => Math.round(size * 2);
  return {
    body: half(appearance.bodySize),
    heading1: half(appearance.bodySize * step * step * step),
    heading2: half(appearance.bodySize * step * step),
    heading3: half(appearance.bodySize * step),
    heading4: half(appearance.bodySize),
    /** Word's line spacing is in twentieths of a point of line height. */
    line: Math.round(appearance.bodySize * LINE_SPACING[appearance.lineSpacing] * 20),
  };
}

/** What the choices are called where somebody is choosing between them. */
export const APPEARANCE_CHOICES = {
  font: [
    { value: 'barlow', label: 'Barlow' },
    { value: 'calibri', label: 'Calibri' },
    { value: 'arial', label: 'Arial' },
    { value: 'georgia', label: 'Georgia' },
    { value: 'times-new-roman', label: 'Times New Roman' },
  ],
  bodySize: [
    { value: 10, label: '10 pt' },
    { value: 11, label: '11 pt' },
    { value: 12, label: '12 pt' },
    { value: 13, label: '13 pt' },
  ],
  headingScale: [
    { value: 'compact', label: 'Compact' },
    { value: 'standard', label: 'Standard' },
    { value: 'large', label: 'Large' },
  ],
  lineSpacing: [
    { value: 'compact', label: 'Compact' },
    { value: 'comfortable', label: 'Comfortable' },
    { value: 'spacious', label: 'Spacious' },
  ],
  pageWidth: [
    { value: 'narrow', label: 'Narrow' },
    { value: 'standard', label: 'Standard' },
    { value: 'wide', label: 'Wide' },
    { value: 'a4', label: 'A4 text column' },
  ],
} as const;

/**
 * The height of one printed page's text column, in CSS pixels.
 *
 * A4 is 297mm tall and the print stylesheet takes 25mm off the top and 22mm off the
 * bottom, so 250mm is left for text. A CSS pixel is 1/96 inch by definition, which
 * is what makes this arithmetic and not a guess — the same definition the print
 * stylesheet's own millimetres are resolved against.
 */
export const PAGE_CONTENT_PIXELS = (250 * 96) / 25.4;

/**
 * Where the pages would break, given the blocks of the document and their heights.
 *
 * An estimate, and it is important to be clear which kind. It follows the two rules
 * the print stylesheet actually states — a heading and a table are not split — and
 * it lets prose split, because print does. What it cannot know is what the printer
 * finally does with a widow or an orphan, so this says where a page ends to within
 * a line or two rather than exactly.
 *
 * Offsets in, offsets out: this does no measuring itself, so it can be tested
 * without a browser.
 */
export interface MeasuredBlock {
  top: number;
  height: number;
  /** Blocks the print stylesheet refuses to split. */
  unbreakable: boolean;
}

export function pageBreaks(blocks: MeasuredBlock[], pageHeight: number): number[] {
  if (pageHeight <= 0) return [];
  const breaks: number[] = [];
  let bottomOfPage = pageHeight;

  for (const block of blocks) {
    const bottom = block.top + block.height;
    if (bottom <= bottomOfPage) continue;

    // A block that will not split and has already started moves to the next page
    // whole, which is what `break-inside: avoid` does.
    const movesWhole = block.unbreakable && block.top < bottomOfPage && block.top > 0;
    const at = movesWhole ? block.top : bottomOfPage;
    breaks.push(at);
    bottomOfPage = at + pageHeight;

    // Something taller than a page still has to be crossed, however it is set.
    while (bottom > bottomOfPage) {
      breaks.push(bottomOfPage);
      bottomOfPage += pageHeight;
    }
  }
  return breaks;
}

/**
 * Which block begins each new page.
 *
 * `pageBreaks` answers where a page *ends*, part-way through a paragraph where
 * print would divide one. That is the right answer for print and the wrong one for
 * the editor, which cannot divide a paragraph without dividing the element: it drew
 * a rule across the words instead and painted the page label on top of them, so a
 * line of the meeting was hidden behind "PAGE 3".
 *
 * This never divides a block. A paragraph that would straddle the boundary is shown
 * whole at the top of the next page, and the note under the document says the
 * printer may split it. An estimate that hides nothing beats an exact one that does.
 */
export function pageStarts(blocks: MeasuredBlock[], pageHeight: number): number[] {
  if (pageHeight <= 0) return [];
  const starts: number[] = [];
  let bottomOfPage = pageHeight;

  blocks.forEach((block, index) => {
    // The first block starts the first page, which needs no gap before it.
    if (index === 0) return;
    if (block.top + block.height <= bottomOfPage) return;
    starts.push(index);
    // The next page begins at this block, whatever was left of the last one.
    bottomOfPage = block.top + pageHeight;
  });

  return starts;
}
