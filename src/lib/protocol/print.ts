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
 */

import { appearanceStyle } from './appearance';
import { renderMarkdown } from './markdown';
import type { DocumentAppearance } from '../workflow/types';

/** Where the print sheet is mounted, as a sibling of the application root. */
const ROOT_ID = 'protocol-print-root';

export interface PrintableProtocol {
  title: string;
  /** Shown under the title: the project, and the date the meeting happened. */
  subtitle: string;
  markdown: string;
  /** How the project sets its protocols; the same values the editor shows. */
  appearance: DocumentAppearance;
}

/**
 * Put the protocol on paper.
 *
 * Whether a PDF was actually saved is not knowable from here: the dialog does not
 * report back, and claiming success would be a guess.
 */
export async function printProtocol(
  protocol: PrintableProtocol,
  /** How to reach the platform's own print panel, where there is one. */
  nativePrint?: () => Promise<void>,
): Promise<void> {
  const existing = document.getElementById(ROOT_ID);
  if (existing) existing.remove();

  const root = document.createElement('div');
  root.id = ROOT_ID;
  root.setAttribute('aria-hidden', 'true');
  // The page is set the way the screen is set. They read the same values, so a
  // protocol cannot print in a typeface nobody chose.
  root.setAttribute('style', appearanceStyle(protocol.appearance));
  root.innerHTML = [
    '<header class="print-masthead">',
    `<h1>${escapeText(protocol.title)}</h1>`,
    protocol.subtitle ? `<p>${escapeText(protocol.subtitle)}</p>` : '',
    '</header>',
    `<div class="print-body">${renderMarkdown(protocol.markdown)}</div>`,
  ].join('');
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
    throw new Error('This window cannot print.');
  }
  window.print();
}

const ESCAPES: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;',
};

function escapeText(text: string): string {
  return text.replace(/[&<>"']/g, (character) => ESCAPES[character] ?? character);
}
