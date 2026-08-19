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

import { renderMarkdown } from './markdown';

/** Where the print sheet is mounted, as a sibling of the application root. */
const ROOT_ID = 'protocol-print-root';

export interface PrintableProtocol {
  title: string;
  /** Shown under the title: the project, and the date the meeting happened. */
  subtitle: string;
  markdown: string;
}

/**
 * Put the protocol on paper.
 *
 * Resolves once the print dialog has been dismissed, which is also when the sheet
 * is removed. Whether a PDF was actually saved is not knowable from here: the
 * dialog does not report back, and claiming success would be a guess.
 */
export async function printProtocol(protocol: PrintableProtocol): Promise<void> {
  if (typeof window === 'undefined' || typeof window.print !== 'function') {
    throw new Error('This window cannot print.');
  }

  const existing = document.getElementById(ROOT_ID);
  if (existing) existing.remove();

  const root = document.createElement('div');
  root.id = ROOT_ID;
  root.setAttribute('aria-hidden', 'true');
  root.innerHTML = [
    '<header class="print-masthead">',
    `<h1>${escapeText(protocol.title)}</h1>`,
    protocol.subtitle ? `<p>${escapeText(protocol.subtitle)}</p>` : '',
    '</header>',
    `<div class="print-body">${renderMarkdown(protocol.markdown)}</div>`,
  ].join('');
  document.body.append(root);

  try {
    // Printing is synchronous in every engine this runs on: the call returns once
    // the dialog closes. The await exists so callers can sequence around it.
    await new Promise<void>((resolve) => {
      window.print();
      resolve();
    });
  } finally {
    root.remove();
  }
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
