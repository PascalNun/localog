import { describe, expect, it } from 'vitest';
import { transcriptToMarkdown, transcriptToText } from './transcriptExport';
import type { TranscriptDocument, TranscriptSegment } from '../workflow/types';

function segment(overrides: Partial<TranscriptSegment>): TranscriptSegment {
  return {
    id: 's1',
    startMs: 0,
    endMs: 1000,
    speaker: 'Speaker 1',
    text: 'Something was said.',
    needsReview: false,
    ...overrides,
  };
}

function transcript(segments: TranscriptSegment[]): TranscriptDocument {
  return {
    schemaVersion: 1,
    meetingId: 'meeting-1',
    revisionId: 'revision-1',
    language: 'German',
    speakerResolution: 'resolved',
    segments,
    baseRevisionId: 'revision-1',
    isDirty: false,
    saveState: 'saved',
    savedAtMs: 0,
  };
}

const context = {
  meetingTitle: 'Jour fixe',
  projectName: 'Beispielquartier',
  occurredAt: '2026-08-27',
};

describe('exporting a transcript', () => {
  it('writes who spoke, when, and what, in reading order', () => {
    const text = transcriptToText(
      transcript([
        segment({ id: 'a', startMs: 12_000, speaker: 'Fachplanung', text: 'We start.' }),
        segment({ id: 'b', startMs: 91_000, speaker: 'Prüfstelle', text: 'Agreed.' }),
      ]),
      context,
    );
    expect(text).toContain('Jour fixe');
    expect(text).toContain('Beispielquartier · 2026-08-27');
    expect(text).toContain('[0:12] Fachplanung: We start.');
    // Past a minute the clock keeps counting rather than restarting.
    expect(text).toContain('[1:31] Prüfstelle: Agreed.');
    expect(text.indexOf('We start.')).toBeLessThan(text.indexOf('Agreed.'));
  });

  it('leaves out the speaker rather than inventing one', () => {
    const text = transcriptToText(
      transcript([segment({ speaker: '   ', text: 'Nobody is named here.' })]),
      context,
    );
    expect(text).toContain('[0:00] Nobody is named here.');
    // No stray colon where a name would have been.
    expect(text).not.toContain(': Nobody');
  });

  it('writes spoken characters through rather than escaping them', () => {
    // Somebody said "asterisk" out loud, or the model wrote one. Escaping it
    // would be editing the record to suit a renderer.
    const said = 'The rate is 5 * 3 and _that_ is final.';
    const markdown = transcriptToMarkdown(transcript([segment({ text: said })]), context);
    expect(markdown).toContain(said);
  });

  it('makes one heading for the meeting and none for the segments', () => {
    const markdown = transcriptToMarkdown(
      transcript([segment({ id: 'a' }), segment({ id: 'b', startMs: 5000 })]),
      context,
    );
    const headings = markdown.split('\n').filter((line) => line.startsWith('#'));
    expect(headings).toEqual(['# Jour fixe']);
    expect(markdown).toContain('**0:00 · Speaker 1**');
  });

  it('handles a transcript with nothing in it without producing rubbish', () => {
    const text = transcriptToText(transcript([]), context);
    expect(text.trim()).toBe('Jour fixe\nBeispielquartier · 2026-08-27');
    const markdown = transcriptToMarkdown(transcript([]), context);
    expect(markdown).toContain('# Jour fixe');
  });

  it('counts past an hour rather than wrapping round', () => {
    const text = transcriptToText(
      transcript([segment({ startMs: 3_661_000, text: 'Still going.' })]),
      context,
    );
    expect(text).toContain('[1:01:01]');
  });
});
