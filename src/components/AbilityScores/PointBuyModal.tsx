import { useState, useEffect, useMemo } from 'react';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useTranslations } from '@/hooks/useTranslations';
import type { AbilityScores, PointBuyState } from '@/lib/bindings';

const X = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
  </svg>
);

interface PointBuyModalProps {
  isOpen: boolean;
  onClose: () => void;
  pointBuyState: PointBuyState | null;
  onApply: (scores: AbilityScores) => Promise<void>;
}

const POINT_BUY_COSTS = [0, 1, 2, 3, 4, 5, 6, 8, 10, 13, 16];
const BUDGET = 32;
const MIN_SCORE = 8;
const MAX_SCORE = 18;

type AbilityKey = 'Str' | 'Dex' | 'Con' | 'Int' | 'Wis' | 'Cha';

const ABILITIES: AbilityKey[] = ['Str', 'Dex', 'Con', 'Int', 'Wis', 'Cha'];
const ABILITY_LABELS: Record<AbilityKey, string> = {
  Str: 'Strength',
  Dex: 'Dexterity',
  Con: 'Constitution',
  Int: 'Intelligence',
  Wis: 'Wisdom',
  Cha: 'Charisma',
};

function getCost(score: number): number {
  if (score <= 8) return 0;
  if (score >= 18) return 16;
  return POINT_BUY_COSTS[score - 8];
}

export default function PointBuyModal({ isOpen, onClose, pointBuyState, onApply }: PointBuyModalProps) {
  const t = useTranslations();
  const [scores, setScores] = useState<Record<AbilityKey, number>>({
    Str: 10, Dex: 10, Con: 10, Int: 10, Wis: 10, Cha: 10
  });
  const [isApplying, setIsApplying] = useState(false);

  useEffect(() => {
    if (isOpen && pointBuyState?.starting_scores) {
      setScores({
        Str: pointBuyState.starting_scores.Str,
        Dex: pointBuyState.starting_scores.Dex,
        Con: pointBuyState.starting_scores.Con,
        Int: pointBuyState.starting_scores.Int,
        Wis: pointBuyState.starting_scores.Wis,
        Cha: pointBuyState.starting_scores.Cha,
      });
    }
  }, [isOpen, pointBuyState]);

  const totalCost = useMemo(() => {
    return ABILITIES.reduce((sum, key) => sum + getCost(scores[key]), 0);
  }, [scores]);

  const remaining = BUDGET - totalCost;

  const handleChange = (key: AbilityKey, delta: number) => {
    const newValue = scores[key] + delta;
    if (newValue < MIN_SCORE || newValue > MAX_SCORE) return;

    const newScores = { ...scores, [key]: newValue };
    const newCost = ABILITIES.reduce((sum, k) => sum + getCost(newScores[k]), 0);

    if (delta > 0 && newCost > BUDGET) return;

    setScores(newScores);
  };

  const handleReset = () => {
    setScores({
      Str: 8, Dex: 8, Con: 8, Int: 8, Wis: 8, Cha: 8
    });
  };

  const handleApply = async () => {
    setIsApplying(true);
    try {
      await onApply(scores as AbilityScores);
      onClose();
    } catch (error) {
      console.error('Failed to apply point buy:', error);
    } finally {
      setIsApplying(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
      <Card className="w-full max-w-md bg-[rgb(var(--color-surface-1))] border-[rgb(var(--color-surface-border))] shadow-2xl relative overflow-hidden">
        <div className="absolute top-4 right-4 z-20">
          <Button onClick={onClose} variant="ghost" size="sm" className="h-8 w-8 p-0 bg-black/20 hover:bg-black/40 text-white rounded-full transition-all">
            <X className="w-5 h-5" />
          </Button>
        </div>

        <div className="p-6">
          <h2 className="text-xl font-bold text-[rgb(var(--color-text-primary))] mb-2">
            {t('abilityScores.pointBuy.title')}
          </h2>

          <p className="text-sm text-[rgb(var(--color-text-muted))] mb-4">
            {t('abilityScores.pointBuy.warning')}
          </p>

          <div className={`text-center text-lg font-bold mb-4 p-2 rounded ${remaining < 0 ? 'bg-red-500/20 text-red-400' : remaining === 0 ? 'bg-green-500/20 text-green-400' : 'bg-[rgb(var(--color-surface-2))]'}`}>
            {t('abilityScores.pointBuy.pointsUsed')}: {totalCost} / {BUDGET}
            <span className="ml-2">
              ({remaining} {t('abilityScores.pointBuy.remaining')})
            </span>
          </div>

          <div className="space-y-3">
            {ABILITIES.map((key) => (
              <div key={key} className="flex items-center justify-between bg-[rgb(var(--color-surface-2))] p-2 rounded">
                <span className="w-28 font-medium text-[rgb(var(--color-text-primary))]">
                  {ABILITY_LABELS[key]}
                </span>
                <div className="flex items-center gap-2">
                  <Button
                    size="xs"
                    variant="secondary"
                    onClick={() => handleChange(key, -1)}
                    disabled={scores[key] <= MIN_SCORE}
                    className="w-8 h-8"
                  >
                    -
                  </Button>
                  <span className="w-8 text-center font-bold text-lg text-[rgb(var(--color-text-primary))]">
                    {scores[key]}
                  </span>
                  <Button
                    size="xs"
                    variant="secondary"
                    onClick={() => handleChange(key, 1)}
                    disabled={scores[key] >= MAX_SCORE || remaining <= 0}
                    className="w-8 h-8"
                  >
                    +
                  </Button>
                  <span className="w-16 text-right text-sm text-[rgb(var(--color-text-muted))]">
                    ({getCost(scores[key])} pts)
                  </span>
                </div>
              </div>
            ))}
          </div>

          <div className="flex justify-between mt-6">
            <Button variant="outline" onClick={handleReset} className="text-red-500 hover:text-red-400 hover:bg-red-500/10 border-red-500/50">
              {t('abilityScores.pointBuy.reset')}
            </Button>
            <div className="flex gap-2">
              <Button variant="ghost" onClick={onClose}>
                {t('actions.cancel')}
              </Button>
              <Button
                variant="primary"
                onClick={handleApply}
                disabled={isApplying || remaining < 0}
              >
                {isApplying ? t('actions.saving') : t('actions.apply')}
              </Button>
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}
