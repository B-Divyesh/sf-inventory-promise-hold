import { describe, expect, it } from 'vitest';
import { relativeExpiry } from './time';

describe('relativeExpiry', () => {
  it('uses plain-language minute and hour states', () => {
    const now = 1_000_000;
    expect(relativeExpiry(Math.floor(now / 1000) + 45, now)).toBe('Less than a minute left');
    expect(relativeExpiry(Math.floor(now / 1000) + 300, now)).toBe('5 min left');
    expect(relativeExpiry(Math.floor(now / 1000) + 3_900, now)).toBe('1 hr 5 min left');
  });
});
