
import { useState } from 'react';
import { useTranslations } from '@/hooks/useTranslations';
import { Card, CardContent } from '@/components/ui/Card';
import AbilityScoreCard from './AbilityScoreCard';

interface AbilityScore {
  name: string;
  shortName: string;
  value: number;
  modifier: number;
  startingValue?: number;
  levelUpValue?: number;
  breakdown?: {
    starting: number;
    levelUp: number;
    racial: number;
    equipment: number;
    enhancement: number;
    temporary: number;
  };
}

interface CoreAbilityScoresSectionProps {
  abilityScores?: AbilityScore[];
  onStartingScoreChange?: (index: number, value: number) => void;
  onLevelUpScoreChange?: (index: number, value: number) => void;
  availableStartingPoints?: number;
  availableLevelUpPoints?: number;
}

export default function CoreAbilityScoresSection({ 
  abilityScores: externalAbilityScores,
  onStartingScoreChange,
  onLevelUpScoreChange,
  availableStartingPoints,
  availableLevelUpPoints
}: CoreAbilityScoresSectionProps) {
  const t = useTranslations();
  
  const [internalAbilityScores, setInternalAbilityScores] = useState<AbilityScore[]>([
    { name: t('abilityScores.strength'), shortName: 'STR', value: 10, modifier: 0 },
    { name: t('abilityScores.dexterity'), shortName: 'DEX', value: 10, modifier: 0 },
    { name: t('abilityScores.constitution'), shortName: 'CON', value: 10, modifier: 0 },
    { name: t('abilityScores.intelligence'), shortName: 'INT', value: 10, modifier: 0 },
    { name: t('abilityScores.wisdom'), shortName: 'WIS', value: 10, modifier: 0 },
    { name: t('abilityScores.charisma'), shortName: 'CHA', value: 10, modifier: 0 },
  ]);

  const abilityScores = externalAbilityScores || internalAbilityScores;

  const calculateModifier = (value: number): number => {
    return Math.floor((value - 10) / 2);
  };

  const updateStartingScore = (index: number, newValue: number) => {
    const clampedValue = Math.max(8, Math.min(18, newValue));

    if (onStartingScoreChange) {
      onStartingScoreChange(index, clampedValue);
    } else {
      const newAbilityScores = [...internalAbilityScores];
      newAbilityScores[index].value = clampedValue;
      newAbilityScores[index].modifier = calculateModifier(clampedValue);
      setInternalAbilityScores(newAbilityScores);
    }
  };

  const updateLevelUpScore = (index: number, newValue: number) => {
    const clampedValue = Math.max(0, newValue);

    if (onLevelUpScoreChange) {
      onLevelUpScoreChange(index, clampedValue);
    }
  };

  const increaseStartingScore = (index: number) => {
    const currentValue = abilityScores[index].startingValue ?? abilityScores[index].breakdown?.starting ?? 8;
    const newValue = currentValue + 1;
    if (newValue <= 18) {
      updateStartingScore(index, newValue);
    }
  };

  const decreaseStartingScore = (index: number) => {
    const currentValue = abilityScores[index].startingValue ?? abilityScores[index].breakdown?.starting ?? 8;
    const newValue = currentValue - 1;
    if (newValue >= 8) {
      updateStartingScore(index, newValue);
    }
  };

  const increaseLevelUpScore = (index: number) => {
    const currentValue = abilityScores[index].levelUpValue ?? abilityScores[index].breakdown?.levelUp ?? 0;
    updateLevelUpScore(index, currentValue + 1);
  };

  const decreaseLevelUpScore = (index: number) => {
    const currentValue = abilityScores[index].levelUpValue ?? abilityScores[index].breakdown?.levelUp ?? 0;
    if (currentValue > 0) {
      updateLevelUpScore(index, currentValue - 1);
    }
  };

  return (
    <Card variant="container">
      <CardContent className="attribute-section-responsive">
        <h3 className="section-title">{t('abilityScores.title')}</h3>
        <div className="attribute-grid-adaptive">
          {abilityScores.map((attr, index) => (
            <AbilityScoreCard
              key={attr.shortName}
              name={attr.name}
              shortName={attr.shortName}
              value={attr.value}
              modifier={attr.modifier}
              startingValue={attr.startingValue}
              levelUpValue={attr.levelUpValue}
              breakdown={attr.breakdown}
              onStartingIncrease={() => increaseStartingScore(index)}
              onStartingDecrease={() => decreaseStartingScore(index)}
              onStartingChange={(value) => updateStartingScore(index, value)}
              onLevelUpIncrease={() => increaseLevelUpScore(index)}
              onLevelUpDecrease={() => decreaseLevelUpScore(index)}
              onLevelUpChange={(value) => updateLevelUpScore(index, value)}
              availableStartingPoints={availableStartingPoints}
              availableLevelUpPoints={availableLevelUpPoints}
            />
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
