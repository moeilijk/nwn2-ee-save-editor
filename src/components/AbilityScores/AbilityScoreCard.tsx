
import { formatModifier } from '@/utils/dataHelpers';
import { useState, useCallback, useRef } from 'react';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

interface AbilityScoreCardProps {
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
  onStartingIncrease: () => void;
  onStartingDecrease: () => void;
  onStartingChange: (value: number) => void;
  onLevelUpIncrease: () => void;
  onLevelUpDecrease: () => void;
  onLevelUpChange: (value: number) => void;
  availableStartingPoints?: number;
  availableLevelUpPoints?: number;
}

export default function AbilityScoreCard({
  name,
  shortName,
  value,
  modifier,
  startingValue,
  levelUpValue,
  breakdown,
  onStartingIncrease,
  onStartingDecrease,
  onStartingChange,
  onLevelUpIncrease,
  onLevelUpDecrease,
  onLevelUpChange,
  availableStartingPoints,
  availableLevelUpPoints
}: AbilityScoreCardProps) {
  const [clickedButton, setClickedButton] = useState<'startingIncrease' | 'startingDecrease' | 'levelIncrease' | 'levelDecrease' | null>(null);
  const startingInputRef = useRef<HTMLInputElement>(null);
  const levelUpInputRef = useRef<HTMLInputElement>(null);
  const currentStartingValue = startingValue !== undefined ? startingValue : (breakdown?.starting ?? 8);
  const currentLevelUpValue = levelUpValue !== undefined ? levelUpValue : (breakdown?.levelUp ?? 0);
  const startingMin = 8;
  const startingMax = 18;
  const canIncreaseStarting = availableStartingPoints === undefined || availableStartingPoints > 0;
  const currentStartingMax = canIncreaseStarting ? startingMax : currentStartingValue;
  const levelUpMin = 0;
  const canIncreaseLevelUps = availableLevelUpPoints === undefined || availableLevelUpPoints > 0;
  const levelUpMax = canIncreaseLevelUps
    ? currentLevelUpValue + 1
    : currentLevelUpValue;

  const getModifierClass = useCallback(() => {
    if (modifier > 0) return 'positive';
    if (modifier < 0) return 'negative';
    return 'zero';
  }, [modifier]);

  const getValueClass = useCallback((value: number) => {
    if (value > 0) return 'positive';
    if (value < 0) return 'negative';
    return 'zero';
  }, []);

  const handleStartingInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = parseInt(e.target.value) || startingMin;
    const clampedValue = Math.max(startingMin, Math.min(currentStartingMax, newValue));

    onStartingChange(clampedValue);
  }, [startingMin, currentStartingMax, onStartingChange]);

  const handleLevelUpInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = parseInt(e.target.value) || levelUpMin;
    const clampedValue = Math.max(levelUpMin, Math.min(levelUpMax, newValue));

    onLevelUpChange(clampedValue);
  }, [levelUpMin, levelUpMax, onLevelUpChange]);

  const handleStartingIncrease = useCallback(() => {
    setClickedButton('startingIncrease');
    onStartingIncrease();
    setTimeout(() => {
      setClickedButton(null);
    }, 200);
  }, [onStartingIncrease]);

  const handleStartingDecrease = useCallback(() => {
    setClickedButton('startingDecrease');
    onStartingDecrease();
    setTimeout(() => {
      setClickedButton(null);
    }, 200);
  }, [onStartingDecrease]);

  const handleLevelUpIncrease = useCallback(() => {
    setClickedButton('levelIncrease');
    onLevelUpIncrease();
    setTimeout(() => {
      setClickedButton(null);
    }, 200);
  }, [onLevelUpIncrease]);

  const handleLevelUpDecrease = useCallback(() => {
    setClickedButton('levelDecrease');
    onLevelUpDecrease();
    setTimeout(() => {
      setClickedButton(null);
    }, 200);
  }, [onLevelUpDecrease]);

  const handleStartingKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        if (currentStartingValue < currentStartingMax) handleStartingIncrease();
        break;
      case 'ArrowDown':
        e.preventDefault();
        if (currentStartingValue > startingMin) handleStartingDecrease();
        break;
      case '+':
      case '=':
        e.preventDefault();
        if (currentStartingValue < currentStartingMax) handleStartingIncrease();
        break;
      case '-':
        e.preventDefault();
        if (currentStartingValue > startingMin) handleStartingDecrease();
        break;
    }
  }, [currentStartingValue, currentStartingMax, startingMin, handleStartingIncrease, handleStartingDecrease]);

  const handleLevelUpKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        if (currentLevelUpValue < levelUpMax) handleLevelUpIncrease();
        break;
      case 'ArrowDown':
        e.preventDefault();
        if (currentLevelUpValue > levelUpMin) handleLevelUpDecrease();
        break;
      case '+':
      case '=':
        e.preventDefault();
        if (currentLevelUpValue < levelUpMax) handleLevelUpIncrease();
        break;
      case '-':
        e.preventDefault();
        if (currentLevelUpValue > levelUpMin) handleLevelUpDecrease();
        break;
    }
  }, [currentLevelUpValue, levelUpMax, levelUpMin, handleLevelUpIncrease, handleLevelUpDecrease]);

  return (
    <Card 
      variant="interactive"
      className="flex flex-col h-full"
      role="group"
      aria-labelledby={`${shortName}-label`}
    >
      <div className="attribute-header-responsive">
        <span 
          id={`${shortName}-label`}
          className="attribute-name-responsive"
          title={`${name} (${shortName})`}
        >
          {name}
        </span>
      </div>

      <div 
        className="attribute-breakdown"
        role="region"
        aria-labelledby={`${shortName}-breakdown-label`}
      >
        <div 
          id={`${shortName}-breakdown-label`}
            className="sr-only"
        >
          {name} breakdown details
        </div>
        <div className="breakdown-row breakdown-base">
          <span className="breakdown-label">Starting:</span>
          <div className="breakdown-controls">
            <Button
              onClick={handleStartingDecrease}
              variant="outline"
              size="xs"
              disabled={currentStartingValue <= startingMin}
              clicked={clickedButton === 'startingDecrease'}
              aria-label={`Decrease ${name} starting score`}
              title={`Decrease ${name} starting score (min: ${startingMin})`}
              className="breakdown-button"
            >
              −
            </Button>
            
            <input
              ref={startingInputRef}
              type="number"
              value={currentStartingValue}
              onChange={handleStartingInputChange}
              onKeyDown={handleStartingKeyDown}
              className="breakdown-input"
              min={startingMin}
              max={currentStartingMax}
              aria-label={`${name} starting value`}
              title={`${name} starting: ${currentStartingValue}, total: ${value} (${formatModifier(modifier)})`}
              aria-describedby={`${shortName}-help`}
            />
            
            <Button
              onClick={handleStartingIncrease}
              variant="outline"
              size="xs"
              disabled={!canIncreaseStarting || currentStartingValue >= currentStartingMax}
              clicked={clickedButton === 'startingIncrease'}
              aria-label={`Increase ${name} starting score`}
              title={`Increase ${name} starting score (max: ${currentStartingMax})`}
              className="breakdown-button"
            >
              +
            </Button>
          </div>
        </div>
        
        {breakdown && (
          <>
            <div className="breakdown-row">
              <span className="breakdown-label">Level-ups:</span>
              <div className="breakdown-controls">
                <Button
                  onClick={handleLevelUpDecrease}
                  variant="outline"
                  size="xs"
                  disabled={currentLevelUpValue <= levelUpMin}
                  clicked={clickedButton === 'levelDecrease'}
                  aria-label={`Decrease ${name} level-up allocations`}
                  title={`Decrease ${name} level-up allocations (min: ${levelUpMin})`}
                  className="breakdown-button"
                >
                  −
                </Button>

                <input
                  ref={levelUpInputRef}
                  type="number"
                  value={currentLevelUpValue}
                  onChange={handleLevelUpInputChange}
                  onKeyDown={handleLevelUpKeyDown}
                  className="breakdown-input"
                  min={levelUpMin}
                  max={levelUpMax}
                  aria-label={`${name} level-up value`}
                  title={`${name} level-up allocations: ${currentLevelUpValue}`}
                  aria-describedby={`${shortName}-help`}
                />

                <Button
                  onClick={handleLevelUpIncrease}
                  variant="outline"
                  size="xs"
                  disabled={!canIncreaseLevelUps || currentLevelUpValue >= levelUpMax}
                  clicked={clickedButton === 'levelIncrease'}
                  aria-label={`Increase ${name} level-up allocations`}
                  title={`Increase ${name} level-up allocations (max: ${levelUpMax})`}
                  className="breakdown-button"
                >
                  +
                </Button>
              </div>
            </div>
            <div className="breakdown-row">
              <span className="breakdown-label">Racial:</span>
              <div className="breakdown-value-container">
                <span className={`breakdown-value ${getValueClass(breakdown.racial)}`}>
                  {formatModifier(breakdown.racial)}
                </span>
              </div>
            </div>
            <div className="breakdown-row">
              <span className="breakdown-label">Equipment:</span>
              <div className="breakdown-value-container">
                <span className={`breakdown-value ${getValueClass(breakdown.equipment)}`}>
                  {formatModifier(breakdown.equipment)}
                </span>
              </div>
            </div>
            {breakdown.enhancement !== 0 && (
              <div className="breakdown-row">
                <span className="breakdown-label">Enhancement:</span>
                <div className="breakdown-value-container">
                  <span className={`breakdown-value ${getValueClass(breakdown.enhancement)}`}>
                    {formatModifier(breakdown.enhancement)}
                  </span>
                </div>
              </div>
            )}
            {breakdown.temporary !== 0 && (
              <div className="breakdown-row">
                <span className="breakdown-label">Temporary:</span>
                <div className="breakdown-value-container">
                  <span className={`breakdown-value ${getValueClass(breakdown.temporary)}`}>
                    {formatModifier(breakdown.temporary)}
                  </span>
                </div>
              </div>
            )}
            <hr className="breakdown-divider" />
            <div className="breakdown-row breakdown-effective-row">
              <span className="breakdown-label">Total:</span>
              <div className="breakdown-value-container">
                <span className="breakdown-value breakdown-effective">{value}</span>
              </div>
            </div>
            <div className="breakdown-row">
              <span className="breakdown-label">Modifier:</span>
              <div className="breakdown-value-container">
                <span className={`breakdown-value ${getModifierClass()}`}>
                  {formatModifier(modifier)}
                </span>
              </div>
            </div>
          </>
        )}
      </div>

      <div 
        id={`${shortName}-help`} 
        className="sr-only"
        aria-hidden="true"
      >
        Use arrow keys or +/- to adjust starting and level-up values.
      </div>
    </Card>
  );
}
