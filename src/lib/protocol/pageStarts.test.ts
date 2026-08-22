import { describe, expect, it } from 'vitest';
import { pageStarts } from './appearance';

const block = (top: number, height: number, unbreakable = false) => ({ top, height, unbreakable });

describe('pageStarts', () => {
  it('starts a page at the block that would not fit on the last one', () => {
    // Three blocks of 40 on a page of 100: the third crosses the boundary.
    expect(pageStarts([block(0, 40), block(40, 40), block(80, 40)], 100)).toEqual([2]);
  });

  it('never divides a block, however it falls', () => {
    // A paragraph straddling the boundary moves whole rather than being cut.
    const starts = pageStarts([block(0, 90), block(90, 60), block(150, 30)], 100);
    expect(starts).toEqual([1]);
  });

  it('measures the next page from where the block actually starts', () => {
    // Page two begins at 90, so it runs to 190 and the block at 150 stays on it.
    expect(pageStarts([block(0, 90), block(90, 60), block(150, 30)], 100)).toEqual([1]);
  });

  it('gives no starts to a document that fits on one page', () => {
    expect(pageStarts([block(0, 40), block(40, 30)], 100)).toEqual([]);
  });

  it('leaves the first block alone even when it is taller than a page', () => {
    expect(pageStarts([block(0, 260)], 100)).toEqual([]);
  });
});
