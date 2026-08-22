/**
 * A protocol as something to be rendered, rather than edited.
 *
 * Printing and Word are two renderings of one document, and each described that
 * document with its own interface — PrintableProtocol and WordProtocol, identical
 * field for field. A field added for one exporter and not the other would have
 * been a difference nothing complained about.
 */

import type { DocumentAppearance, PageFurniture, ProtocolDraft } from '../workflow/types';
import type { DocumentFacts } from './furniture';

export interface ProtocolDocument {
  title: string;
  /** Shown under the title: the project, and the date the meeting happened. */
  subtitle: string;
  markdown: string;
  /** How the project sets its protocols; the same values the editor shows. */
  appearance: DocumentAppearance;
  /** What repeats on every page; absent means nothing does. */
  furniture?: PageFurniture;
  facts?: DocumentFacts;
}

/**
 * How a protocol's review state is said to a person.
 *
 * The editor's header and the printed document's own facts each spelled these
 * three strings out, and in a different branch order — which is the tell that
 * they were written separately rather than shared.
 */
export function reviewStateLabel(state: ProtocolDraft['reviewState']): string {
  if (state === 'reviewed') return 'Reviewed';
  if (state === 'changed_since_review') return 'Changed since review';
  return 'Draft';
}
