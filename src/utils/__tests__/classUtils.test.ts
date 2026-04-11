import { describe, it, expect } from 'vitest';
import { capXP, aggregateClassStats, hasLevelMismatch } from '../classUtils';

describe('capXP', () => {
  it('returns value unchanged when under cap', () => {
    expect(capXP(500000)).toBe(500000);
  });
  it('caps at 1,770,000', () => {
    expect(capXP(2000000)).toBe(1770000);
  });
  it('returns 0 for negative values', () => {
    expect(capXP(-100)).toBe(0);
  });
});

describe('aggregateClassStats', () => {
  const classes = [
    { baseAttackBonus: 4, fortitudeSave: 4, reflexSave: 1, willSave: 1 },
    { baseAttackBonus: 9, fortitudeSave: 8, reflexSave: 8, willSave: 8 },
  ];

  it('sums BAB and saves across classes', () => {
    const result = aggregateClassStats(classes);
    expect(result).toEqual({ totalBAB: 13, totalFort: 12, totalRef: 9, totalWill: 9 });
  });

  it('returns zeros for empty array', () => {
    expect(aggregateClassStats([])).toEqual({ totalBAB: 0, totalFort: 0, totalRef: 0, totalWill: 0 });
  });
});

describe('hasLevelMismatch', () => {
  it('returns true when XP level differs from total class level', () => {
    expect(hasLevelMismatch({ current_level: 10 }, 12)).toBe(true);
  });
  it('returns false when levels match', () => {
    expect(hasLevelMismatch({ current_level: 10 }, 10)).toBe(false);
  });
  it('returns false for null xpProgress', () => {
    expect(hasLevelMismatch(null, 10)).toBe(false);
  });
});
