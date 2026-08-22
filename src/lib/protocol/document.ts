/**
 * A protocol as something to be rendered, rather than edited.
 *
 * Printing and Word are two renderings of one document, and each described that
 * document with its own interface — PrintableProtocol and WordProtocol, identical
 * field for field. A field added for one exporter and not the other would have
 * been a difference nothing complained about.
 */

import type {
  DocumentAppearance,
  MeetingSummary,
  PageFurniture,
  ProjectSummary,
  ProtocolDraft,
} from '../workflow/types';
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

/**
 * What a header or footer field can be filled in from.
 *
 * Read at the moment it is wanted rather than stored, because a protocol marked
 * reviewed after the header was set should say so on the page.
 *
 * Shared rather than built where it is needed: the editor shows the resolved band
 * between the pages and the exporters print it, and a preview that assembled its
 * own facts would be a second account of the document that could disagree with
 * the first while looking authoritative.
 */
export function documentFacts(
  project: ProjectSummary | undefined,
  meeting: MeetingSummary,
  protocol: ProtocolDraft,
): DocumentFacts {
  return {
    projectName: project?.name ?? '',
    meetingTitle: meeting.title,
    meetingDate: meeting.occurredAt,
    documentType: 'Protocol',
    protocolStatus: reviewStateLabel(protocol.reviewState),
  };
}
