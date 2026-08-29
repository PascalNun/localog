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
import type { Strings } from '../i18n';

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
 *
 * The two drifted apart again during the translation, when the inspector's copy was
 * translated and this one was not, because it prints *into* the document and whether
 * a document follows the interface's language was still open. It is decided now — the
 * interface's — so they are one function again.
 */
export function reviewStateLabel(state: ProtocolDraft['reviewState'], words: Strings): string {
  if (state === 'reviewed') return words.protocol.statusReviewed;
  if (state === 'changed_since_review') return words.protocol.statusChanged;
  return words.protocol.statusDraft;
}

/**
 * A meeting's date, written the way the interface's language writes dates.
 *
 * Midday and not midnight. `new Date('2026-08-27')` is parsed as UTC, so a browser
 * anywhere west of Greenwich renders that date as the 26th — which on a document
 * somebody files is not a formatting preference.
 *
 * An unparseable value is returned as it stands. A date nobody can read beats a
 * document that says "Invalid Date" where its date should be.
 */
export function formatMeetingDate(
  iso: string,
  words: Strings,
  length: 'long' | 'short' = 'long',
): string {
  if (!iso) return '';
  const at = new Date(`${iso}T12:00:00`);
  if (Number.isNaN(at.getTime())) return iso;
  return new Intl.DateTimeFormat(words.locale, {
    day: 'numeric',
    month: length,
    year: 'numeric',
  }).format(at);
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
  words: Strings,
): DocumentFacts {
  return {
    projectName: project?.name ?? '',
    meetingTitle: meeting.title,
    meetingDate: formatMeetingDate(meeting.occurredAt, words),
    documentType: words.protocol.documentType,
    protocolStatus: reviewStateLabel(protocol.reviewState, words),
  };
}
