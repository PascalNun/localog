import { describe, expect, it } from 'vitest';
import { pagesOf } from './print';
import { DEFAULT_APPEARANCE, EMPTY_FURNITURE } from '../workflow/types';
import type { ProtocolDocument } from './document';
import type { FurnitureField } from '../workflow/types';

const markdown = ['# One', 'First.', '# Two', 'Second.', '# Three', 'Third.'].join('\n\n');

const doc = (over: Partial<ProtocolDocument> = {}): ProtocolDocument => ({
  title: 'Jour fixe',
  subtitle: 'Neubau Halle 4 · 29.07.2026',
  markdown,
  appearance: DEFAULT_APPEARANCE,
  furniture: EMPTY_FURNITURE,
  facts: {
    projectName: 'Neubau Halle 4',
    meetingTitle: 'Jour fixe',
    meetingDate: '29.07.2026',
    documentType: 'Protokoll',
    protocolStatus: 'Entwurf',
  },
  ...over,
});

const withFurniture = (header: FurnitureField[], footer: FurnitureField[]) => ({
  header: { left: header, centre: [], right: [] },
  footer: { left: footer, centre: [], right: [] },
  skipFirstPage: false,
});

const pageCount = (html: string) => html.split('<section class="print-page">').length - 1;

describe('cutting the document into pages', () => {
  it('makes one page when nobody measured', () => {
    expect(pageCount(pagesOf(doc(), []))).toBe(1);
  });

  it('makes a page for each break the editor found', () => {
    expect(pageCount(pagesOf(doc(), [2, 4]))).toBe(3);
  });

  it('puts the masthead on the first page only', () => {
    const html = pagesOf(doc(), [2, 4]);
    expect(html.split('print-masthead').length - 1).toBe(1);
    expect(html.indexOf('print-masthead')).toBeLessThan(html.indexOf('<section class="print-page">', 1));
  });

  it('keeps every block, in order, across the pages', () => {
    const whole = pagesOf(doc(), []);
    const cut = pagesOf(doc(), [2, 4]);
    const text = (html: string) => html.replace(/<[^>]*>/g, '');
    expect(text(cut)).toBe(text(whole));
  });

  it('ignores a break outside the document rather than printing a blank sheet', () => {
    expect(pageCount(pagesOf(doc(), [99]))).toBe(1);
    expect(pageCount(pagesOf(doc(), [0]))).toBe(1);
  });
});

describe('the furniture on each page', () => {
  const furniture = withFurniture([{ kind: 'projectName' }], [{ kind: 'pageOfCount' }]);

  it('repeats the header on every page', () => {
    const html = pagesOf(doc({ furniture }), [2, 4]);
    // The failure this replaces: one header for the whole document.
    expect(html.split('class="print-header"').length - 1).toBe(3);
    expect(html.split('class="print-footer"').length - 1).toBe(3);
  });

  it('counts the pages, which the browser could not', () => {
    const html = pagesOf(doc({ furniture }), [2, 4]);
    expect(html).toContain('1 / 3');
    expect(html).toContain('2 / 3');
    expect(html).toContain('3 / 3');
  });

  it('leaves the first page bare when the title page asks for it', () => {
    const html = pagesOf(doc({ furniture: { ...furniture, skipFirstPage: true } }), [2, 4]);
    expect(html.split('class="print-header"').length - 1).toBe(2);
    expect(html).not.toContain('1 / 3');
    expect(html).toContain('2 / 3');
  });

  it('draws no band at all when there is no furniture', () => {
    const html = pagesOf(doc(), [2, 4]);
    expect(html).not.toContain('print-header');
    expect(html).not.toContain('print-footer');
  });
});
