/**
 * A transcript as a document somebody can keep, send, or read outside LocaLog.
 *
 * The protocol has been exportable four ways since it existed, and the transcript
 * — which is the thing somebody spent an hour correcting, and the record closest
 * to what was actually said — could not leave the application at all. This is
 * that, and it is built here for the same reason the Word export is: the writing
 * of bytes happens in Rust, and what goes in them is decided next to the screen
 * showing the same content, so the two cannot drift.
 *
 * Two formats, matching the two the protocol offers for plain writing. There is
 * no PDF or Word here on purpose: a transcript is working material, and somebody
 * who wants it set as a document wants a protocol.
 */

import { clockFromMillis } from '../time';
import type { TranscriptDocument, TranscriptSegment } from '../workflow/types';

/** Everything a rendered transcript needs that the transcript itself does not carry. */
export interface TranscriptExportContext {
  meetingTitle: string;
  projectName: string;
  occurredAt: string;
}

/**
 * The speaker's name, or nothing where a name would be a guess.
 *
 * Diarisation labels speakers `Speaker 1` and so on until somebody renames them,
 * and a transcript where nobody has done that should not pretend otherwise — but
 * the label is still what distinguishes one voice from the next, so it stays.
 */
function speakerOf(segment: TranscriptSegment): string {
  return segment.speaker.trim();
}

/**
 * Plain text, in the shape a person reads a transcript in.
 *
 * One line per segment, time first, then who, then what. No wrapping: whoever
 * opens this decides how wide their window is, and a hard-wrapped transcript is
 * worse to search and worse to quote from.
 */
export function transcriptToText(
  transcript: TranscriptDocument,
  context: TranscriptExportContext,
): string {
  const lines: string[] = [
    context.meetingTitle,
    `${context.projectName} · ${context.occurredAt}`,
    '',
  ];
  for (const segment of transcript.segments) {
    const who = speakerOf(segment);
    const time = clockFromMillis(segment.startMs);
    lines.push(who ? `[${time}] ${who}: ${segment.text}` : `[${time}] ${segment.text}`);
  }
  return `${lines.join('\n')}\n`;
}

/**
 * The same, as Markdown.
 *
 * The timestamp becomes a bold prefix rather than a heading: a heading per
 * segment would produce a document with four hundred headings and a table of
 * contents nobody can use.
 *
 * Text is written through unchanged. A transcript is what somebody said, and
 * escaping an asterisk they spoke would be editing the record to protect a
 * renderer.
 */
export function transcriptToMarkdown(
  transcript: TranscriptDocument,
  context: TranscriptExportContext,
): string {
  const lines: string[] = [
    `# ${context.meetingTitle}`,
    '',
    `${context.projectName} · ${context.occurredAt}`,
    '',
  ];
  for (const segment of transcript.segments) {
    const who = speakerOf(segment);
    const time = clockFromMillis(segment.startMs);
    lines.push(who ? `**${time} · ${who}** ${segment.text}` : `**${time}** ${segment.text}`);
    lines.push('');
  }
  return `${lines.join('\n')}`;
}
