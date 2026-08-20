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
