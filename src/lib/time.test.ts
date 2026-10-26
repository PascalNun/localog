import { describe, expect, it } from 'vitest';
import { clock, clockFromMillis, localDateInput } from './time';

describe('clock', () => {
  it('drops the hour when there is none', () => {
    expect(clockFromMillis(0)).toBe('0:00');
    expect(clockFromMillis(62_000)).toBe('1:02');
    expect(clockFromMillis(3_725_000)).toBe('1:02:05');
  });

  it('shows the second a moment is inside rather than the nearest one', () => {
    expect(clockFromMillis(1_999)).toBe('0:01');
  });

  it('does not go backwards past the start', () => {
    expect(clock(-5)).toBe('0:00');
  });
});

describe('localDateInput', () => {
  it('does not fall back to the UTC day when the two differ', () => {
    const original = process.env.TZ;
    try {
      // UTC+14: the same instant is already tomorrow locally.
      process.env.TZ = 'Pacific/Kiritimati';
      const instant = new Date('2026-10-24T22:30:00Z');
      expect(instant.toISOString().slice(0, 10)).toBe('2026-10-24');
      expect(localDateInput(instant)).toBe('2026-10-25');
    } finally {
      if (original === undefined) delete process.env.TZ;
      else process.env.TZ = original;
    }
  });

  it('pads months and days', () => {
    expect(localDateInput(new Date(2026, 0, 5, 12, 0))).toBe('2026-01-05');
  });
});
