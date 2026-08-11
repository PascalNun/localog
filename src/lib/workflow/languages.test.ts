import { describe, expect, it } from 'vitest';
import { COMMON_MEETING_LANGUAGES } from './languages';

describe('meeting language choices', () => {
  it('keeps the primary German and English workflow choices visible', () => {
    expect(COMMON_MEETING_LANGUAGES).toContain('English');
    expect(COMMON_MEETING_LANGUAGES).toContain('German');
  });

  it('does not treat the convenience list as an exclusive language catalogue', () => {
    expect(COMMON_MEETING_LANGUAGES.length).toBeGreaterThan(10);
  });
});
