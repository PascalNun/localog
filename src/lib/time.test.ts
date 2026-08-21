import { describe, expect, it } from 'vitest';
import { clock, clockFromMillis } from './time';

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
