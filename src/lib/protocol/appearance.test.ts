import { describe, expect, it } from 'vitest';
import { PAGE_CONTENT_PIXELS, pageBreaks, type MeasuredBlock } from './appearance';

const block = (top: number, height: number, unbreakable = false): MeasuredBlock => ({
  top,
  height,
  unbreakable,
});

describe('where the pages break', () => {
  it('measures an A4 text column the way the print stylesheet does', () => {
    // 297mm less 25mm and 22mm of margin, at 96 CSS pixels to the inch.
    expect(Math.round(PAGE_CONTENT_PIXELS)).toBe(945);
  });

  it('finds no break in a document that fits on one page', () => {
    expect(pageBreaks([block(0, 200), block(200, 300)], 945)).toEqual([]);
  });

  it('breaks where the page ends when prose runs over it', () => {
    expect(pageBreaks([block(0, 1200)], 945)).toEqual([945]);
  });

  /// The rule the print stylesheet states, and the fault it exists to prevent: a
  /// heading at the foot of a page with its section overleaf.
  it('moves a heading to the next page rather than splitting it', () => {
    const breaks = pageBreaks([block(0, 900), block(900, 90, true), block(990, 400)], 945);
    expect(breaks).toEqual([900]);
  });

  it('splits prose at the page edge even though it would move a heading', () => {
    const breaks = pageBreaks([block(0, 900), block(900, 90, false)], 945);
    expect(breaks).toEqual([945]);
  });

  it('crosses a block taller than a whole page', () => {
    expect(pageBreaks([block(0, 2500)], 945)).toEqual([945, 1890]);
  });

  it('keeps counting pages through a long document', () => {
    const blocks = Array.from({ length: 40 }, (_, at) => block(at * 100, 100));
    expect(pageBreaks(blocks, 945)).toEqual([945, 1890, 2835, 3780]);
  });

  it('is empty rather than infinite for a nonsense page height', () => {
    expect(pageBreaks([block(0, 500)], 0)).toEqual([]);
  });
});
