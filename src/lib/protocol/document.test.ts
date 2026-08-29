import { describe, expect, it } from 'vitest';
import { documentFacts, formatMeetingDate, reviewStateLabel } from './document';
import { en } from '../i18n/en';
import { de } from '../i18n/de';
import type { MeetingSummary, ProjectSummary, ProtocolDraft } from '../workflow/types';

const project = { id: 'p1', name: 'Neubau Halle 4' } as ProjectSummary;
const meeting = {
  id: 'm1',
  title: 'Jour fixe Fassade',
  occurredAt: '2026-08-10',
} as MeetingSummary;
const draft = (reviewState: ProtocolDraft['reviewState']) => ({ reviewState }) as ProtocolDraft;

describe('a date printed into a document', () => {
  it('is written the way the interface writes dates', () => {
    expect(formatMeetingDate('2026-08-10', en)).toBe('10 August 2026');
    expect(formatMeetingDate('2026-08-10', de)).toBe('10. August 2026');
  });

  /**
   * `new Date('2026-08-10')` is parsed as UTC, so anywhere west of Greenwich it
   * renders as the 9th. On a document somebody files, a date off by one is not a
   * formatting preference — hence midday rather than midnight.
   */
  it('names the day it was given, whatever timezone the machine is in', () => {
    expect(formatMeetingDate('2026-08-10', en)).toContain('10');
    expect(formatMeetingDate('2026-01-01', en)).toBe('1 January 2026');
    expect(formatMeetingDate('2026-12-31', en)).toBe('31 December 2026');
  });

  it('returns something it cannot read as it stands', () => {
    // Better a date nobody can parse than a document printing "Invalid Date".
    expect(formatMeetingDate('not a date', en)).toBe('not a date');
    expect(formatMeetingDate('', en)).toBe('');
  });
});

describe('the facts printed in a page header', () => {
  /**
   * The decision of 29 August 2026: text that is part of the document follows the
   * interface's language, not the meeting's. So the same protocol prints a German
   * header for somebody working in German and an English one for somebody working
   * in English, and nothing in it is left in a language the rest is not.
   */
  it('follow the interface, so nothing in the header is left in another language', () => {
    const german = documentFacts(project, meeting, draft('draft'), de);
    expect(german.documentType).toBe('Protokoll');
    expect(german.protocolStatus).toBe('Entwurf');
    expect(german.meetingDate).toBe('10. August 2026');

    const english = documentFacts(project, meeting, draft('draft'), en);
    expect(english.documentType).toBe('Protocol');
    expect(english.protocolStatus).toBe('Draft');
    expect(english.meetingDate).toBe('10 August 2026');
  });

  it('take the names from the work rather than from the dictionary', () => {
    // The project and the meeting are what somebody typed. Translating those would
    // be the application editing the record.
    for (const words of [en, de]) {
      const facts = documentFacts(project, meeting, draft('reviewed'), words);
      expect(facts.projectName).toBe('Neubau Halle 4');
      expect(facts.meetingTitle).toBe('Jour fixe Fassade');
    }
  });

  it('says every review state in both languages', () => {
    const states: ProtocolDraft['reviewState'][] = ['draft', 'reviewed', 'changed_since_review'];
    for (const state of states) {
      expect(reviewStateLabel(state, en)).not.toBe(reviewStateLabel(state, de));
    }
    expect(reviewStateLabel('reviewed', de)).toBe('Geprüft');
    expect(reviewStateLabel('changed_since_review', de)).toBe('Seit der Prüfung geändert');
  });
});
