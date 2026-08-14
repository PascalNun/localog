import { describe, expect, it } from 'vitest';
import { COMMON_MEETING_LANGUAGES, DETECT_LANGUAGE_LABEL, meetingLanguageLabel } from './languages';

describe('meeting language choices', () => {
  it('keeps the primary German and English workflow choices visible', () => {
    expect(COMMON_MEETING_LANGUAGES).toContain('English');
    expect(COMMON_MEETING_LANGUAGES).toContain('German');
  });

  it('does not treat the convenience list as an exclusive language catalogue', () => {
    expect(COMMON_MEETING_LANGUAGES.length).toBeGreaterThan(10);
  });
});

describe('meetingLanguageLabel', () => {
  it('names a chosen language and says plainly when none was chosen', () => {
    expect(meetingLanguageLabel('German')).toBe('German');
    expect(meetingLanguageLabel('')).toBe(DETECT_LANGUAGE_LABEL);
    expect(meetingLanguageLabel('   ')).toBe(DETECT_LANGUAGE_LABEL);
    expect(meetingLanguageLabel(null)).toBe(DETECT_LANGUAGE_LABEL);
  });
});
