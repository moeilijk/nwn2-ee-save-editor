import { useState, useMemo, useCallback } from 'react';
import { Button, ProgressBar } from '@blueprintjs/core';
import { T } from '../theme';
import { mod, ParchmentDialog } from '../shared';

type AbilityKey = 'Str' | 'Dex' | 'Con' | 'Int' | 'Wis' | 'Cha';

const ABILITY_NAMES: Record<AbilityKey, string> = {
  Str: 'Strength', Dex: 'Dexterity', Con: 'Constitution',
  Int: 'Intelligence', Wis: 'Wisdom', Cha: 'Charisma',
};

const ABILITY_SHORT: Record<AbilityKey, string> = {
  Str: 'STR', Dex: 'DEX', Con: 'CON', Int: 'INT', Wis: 'WIS', Cha: 'CHA',
};

const KEYS: AbilityKey[] = ['Str', 'Dex', 'Con', 'Int', 'Wis', 'Cha'];
const POINT_COSTS = [0, 1, 2, 3, 4, 5, 6, 8, 10, 13, 16];
const BUDGET = 32;
const MIN = 8;
const MAX = 18;

function cost(score: number): number {
  if (score <= 8) return 0;
  if (score >= 18) return 16;
  return POINT_COSTS[score - 8];
}

interface RespecDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function RespecDialog({ isOpen, onClose }: RespecDialogProps) {
  const [scores, setScores] = useState<Record<AbilityKey, number>>({
    Str: 10, Dex: 10, Con: 10, Int: 10, Wis: 10, Cha: 10,
  });

  const totalCost = useMemo(() => KEYS.reduce((s, k) => s + cost(scores[k]), 0), [scores]);
  const remaining = BUDGET - totalCost;

  const change = useCallback((key: AbilityKey, delta: number) => {
    const next = scores[key] + delta;
    if (next < MIN || next > MAX) return;
    const nextScores = { ...scores, [key]: next };
    if (delta > 0 && KEYS.reduce((s, k) => s + cost(nextScores[k]), 0) > BUDGET) return;
    setScores(nextScores);
  }, [scores]);

  const reset = () => setScores({ Str: 8, Dex: 8, Con: 8, Int: 8, Wis: 8, Cha: 8 });

  const handleOpen = () => {
    setScores({ Str: 10, Dex: 10, Con: 10, Int: 10, Wis: 10, Cha: 10 });
  };

  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={handleOpen}
      title="Respec - Point Buy"
      width={480}
      footerActions={
        <Button
          text="Apply"
          intent="primary"
          disabled={remaining < 0}
          onClick={onClose}
          style={{ background: T.accent }}
        />
      }
      footerLeft={
        <Button text="Reset All" icon="reset" minimal onClick={reset} style={{ color: T.negative }} />
      }
    >
      <div style={{
        padding: '8px 12px', marginBottom: 16, borderRadius: 4,
        background: '#fde8e8', border: `1px solid ${T.negative}30`,
        fontSize: 12, color: T.negative, lineHeight: 1.5,
      }}>
        This will reset all base ability scores to the point buy values. Level-up increases, racial bonuses, and equipment bonuses are recalculated automatically.
      </div>

      <div style={{ marginBottom: 16 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 6 }}>
          <span style={{ fontSize: 12, fontWeight: 600, color: T.textMuted }}>
            Points Used: <strong style={{ color: T.text }}>{totalCost}</strong> / {BUDGET}
          </span>
          <span style={{
            fontSize: 12, fontWeight: 700,
            color: remaining === 0 ? T.positive : remaining < 0 ? T.negative : T.accent,
          }}>
            {remaining} remaining
          </span>
        </div>
        <ProgressBar
          value={totalCost / BUDGET}
          intent={remaining === 0 ? 'success' : remaining < 0 ? 'danger' : 'none'}
          stripes={false} animate={false}
          style={{ height: 6 }}
        />
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
        {KEYS.map(key => {
          const score = scores[key];
          const modifier = Math.floor((score - 10) / 2);
          const pts = cost(score);
          return (
            <div key={key} style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '8px 12px', borderRadius: 4,
              background: T.surfaceAlt, border: `1px solid ${T.borderLight}`,
            }}>
              <span style={{ width: 36, fontSize: 12, fontWeight: 700, color: T.accent }}>{ABILITY_SHORT[key]}</span>
              <span style={{ flex: 1, fontSize: 13, fontWeight: 500, color: T.text }}>{ABILITY_NAMES[key]}</span>

              <Button
                icon="minus" small minimal
                disabled={score <= MIN}
                onClick={() => change(key, -1)}
                style={{ color: T.textMuted }}
              />
              <span style={{
                width: 32, textAlign: 'center',
                fontSize: 16, fontWeight: 700, color: T.text,
              }}>
                {score}
              </span>
              <Button
                icon="plus" small minimal
                disabled={score >= MAX || remaining <= 0}
                onClick={() => change(key, 1)}
                style={{ color: T.textMuted }}
              />

              <span style={{
                width: 32, textAlign: 'center', fontSize: 12, fontWeight: 600,
                color: modifier > 0 ? T.positive : modifier < 0 ? T.negative : T.textMuted,
              }}>
                {mod(modifier)}
              </span>
              <span style={{
                width: 40, textAlign: 'right',
                fontSize: 11, color: T.textMuted,
              }}>
                {pts} pts
              </span>
            </div>
          );
        })}
      </div>
    </ParchmentDialog>
  );
}
