/**
 * A protocol on paper, or as a PDF, which on macOS is the same dialog.
 *
 * No PDF library. The document is already rendered for the editor, and the print
 * step turns exactly that into pages — so what somebody read on screen and what
 * comes out are the same document, printed by the same stylesheet. A separate PDF
 * writer would be a second renderer to keep in agreement with the first, and the
 * first one would lose.
 *
 * The page is built beside the application rather than in place of it, and taken
 * down afterwards, so nothing about the editor's own layout leaks into the sheet.
 *
 * ## Why this paginates itself
 *
 * It used to hand the browser one long document with the header and footer marked
 * `position: fixed`, which is how a browser is asked to repeat something on every
 * printed page. Chromium does. WebKit does not — and macOS prints through
 * WKWebView. Measured on a real three-page export: the header printed once at the
 * top of page one, the footer once at the foot of page three, and page two had
 * neither.
 *
 * So nothing is left to the browser to repeat. The document is cut into page
 * boxes here, each holding its own header, its slice of the document and its own
 * footer, and the printer is asked only to start a new sheet between them. Three
 * things that were impossible follow from doing the pagination rather than asking
 * for it: the bands appear on every page in any engine, the page number is known
 * because we are the thing counting, and the band cannot land on the first line
 * because it is a sibling of the text rather than fixed above it.
 */

import { appearanceStyle, printSideMarginMm } from './appearance';
import { resolveRow, rowIsEmpty } from './furniture';
import type { ProtocolDocument } from './document';
import { escapeHtml, renderBlocks } from './markdown';

/** Where the print sheet is mounted, as a sibling of the application root. */
const ROOT_ID = 'protocol-print-root';

/**
 * Put the protocol on paper.
 *
 * Whether a PDF was actually saved is not knowable from here: the dialog does not
 * report back, and claiming success would be a guess.
 */
export async function printProtocol(
  protocol: ProtocolDocument,
  /** How to reach the platform's own print panel, where there is one. */
  nativePrint?: () => Promise<void>,
  /**
   * Which rendered block begins each page after the first, as the editor measured
   * it. Empty means nobody measured, and the whole document goes on one page box —
   * the printer then breaks it wherever it likes, which is what happened before
   * and is still better than a header that appears once.
   */
  pageStarts: number[] = [],
): Promise<void> {
  const existing = document.getElementById(ROOT_ID);
  if (existing) existing.remove();

  const root = document.createElement('div');
  root.id = ROOT_ID;
  root.setAttribute('aria-hidden', 'true');
  // The page is set the way the screen is set. They read the same values, so a
  // protocol cannot print in a typeface nobody chose.
  root.setAttribute('style', appearanceStyle(protocol.appearance));
  // The paper's margins, so that the printed column is the width the document is
  // set to. Emitted here rather than written into the stylesheet because @page
  // cannot read the document's own custom properties, and because the number
  // depends on the project's measure. It goes with the sheet when the sheet goes.
  const side = printSideMarginMm(protocol.appearance);
  root.innerHTML =
    `<style>@page { size: A4; margin: 25mm ${side}mm 22mm; }</style>` +
    pagesOf(protocol, pageStarts);
  document.body.append(root);

  // The sheet is left in place rather than taken down when this returns. The
  // macOS print panel is detached from this thread and renders the page after
  // the call has already come back, so removing it here would print nothing. It
  // costs nothing to leave: the sheet is hidden on screen, and the next export
  // replaces it.
  if (nativePrint) {
    await nativePrint();
    return;
  }

  if (typeof window === 'undefined' || typeof window.print !== 'function') {
    throw new Error('printDialogUnavailable');
  }
  window.print();
}

/**
 * The document cut into page boxes, each carrying its own furniture.
 *
 * Exported for the tests, which is the only way to look at what will print
 * without a printer.
 */
export function pagesOf(protocol: ProtocolDocument, pageStarts: number[]): string {
  const blocks = renderBlocks(protocol.markdown);
  const slices = sliceAt(blocks, pageStarts);
  const total = slices.length;

  return slices
    .map((slice, index) => {
      const page = index + 1;
      // A title page carries its own heading and usually wants nothing repeated
      // on it. Stored since the furniture was first written and honoured by
      // nothing until the pages became ours to draw.
      const bare = page === 1 && protocol.furniture?.skipFirstPage === true;
      const masthead =
        page === 1
          ? '<header class="print-masthead">' +
            `<h1>${escapeHtml(protocol.title)}</h1>` +
            (protocol.subtitle ? `<p>${escapeHtml(protocol.subtitle)}</p>` : '') +
            '</header>'
          : '';
      return (
        '<section class="print-page">' +
        (bare ? '' : band(protocol, 'header', page, total)) +
        `<div class="print-body">${masthead}${slice}</div>` +
        (bare ? '' : band(protocol, 'footer', page, total)) +
        '</section>'
      );
    })
    .join('');
}

/**
 * The blocks grouped into pages.
 *
 * `pageStarts` holds the index of the block that begins each page after the
 * first, which is what the editor computes to draw the gap between its pages. A
 * start outside the document is ignored rather than producing an empty sheet.
 */
function sliceAt(blocks: string[], pageStarts: number[]): string[] {
  const starts = [...new Set(pageStarts)]
    .filter((at) => at > 0 && at < blocks.length)
    .sort((a, b) => a - b);
  if (starts.length === 0) return [blocks.join('')];

  const pages: string[] = [];
  let from = 0;
  for (const at of starts) {
    pages.push(blocks.slice(from, at).join(''));
    from = at;
  }
  pages.push(blocks.slice(from).join(''));
  return pages;
}

/**
 * The header or footer of one page.
 *
 * The page number is answered here rather than left out. It was omitted for as
 * long as the browser did the paginating — only the thing breaking the pages can
 * know which one this is, and a browser will not say. Now this is that thing.
 */
function band(
  protocol: ProtocolDocument,
  which: 'header' | 'footer',
  page: number,
  total: number,
): string {
  const row = protocol.furniture?.[which];
  const facts = protocol.facts;
  if (!row || !facts || rowIsEmpty(row)) return '';
  const marker = { number: String(page), ofCount: `${page} / ${total}` };
  const slot = (fields: typeof row.left) =>
    `<span>${escapeHtml(resolveRow(fields, facts, marker))}</span>`;
  return (
    `<div class="print-${which}">` +
    `${slot(row.left)}${slot(row.centre)}${slot(row.right)}` +
    `</div>`
  );
}
