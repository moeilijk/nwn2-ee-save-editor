import { useEffect, useCallback, useState } from 'react';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { useAbilityScores } from '@/hooks/useAbilityScores';
import { useTranslations } from '@/hooks/useTranslations';
import CoreAbilityScoresSection from './CoreAbilityScoresSection';
import VitalStatisticsSection from './VitalStatisticsSection';
import AlignmentSection from './AlignmentSection';
import PointBuyModal from './PointBuyModal';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { CharacterStateAPI } from '@/lib/api/character-state';
import type { AbilitiesState, CombatSummary, SaveSummary, AbilityScores } from '@/lib/bindings';

export default function AbilityScoresEditor() {
  const { character } = useCharacterContext();
  const t = useTranslations();
  const attributesData = useSubsystem('abilityScores');
  const combatData = useSubsystem('combat');
  const savesData = useSubsystem('saves');
  const [showPointBuy, setShowPointBuy] = useState(false);

  useEffect(() => {
    if (character?.id) {
      if (!attributesData.data && !attributesData.isLoading) {
        attributesData.load();
      }
      if (!combatData.data && !combatData.isLoading) {
        combatData.load();
      }
      if (!savesData.data && !savesData.isLoading) {
        savesData.load();
      }
    }
  }, [character?.id, attributesData, combatData, savesData]);

  const {
    abilityScores,
    stats,
    alignment,
    updateAbilityScore,
    updateStats,
    updateAlignment,
    pointSummary
  } = useAbilityScores(
    attributesData.data as AbilitiesState | null,
    {
      combat: combatData.data as CombatSummary | null,
      saves: savesData.data as SaveSummary | null
    }
  );
  
  const handleAbilityScoreUpdate = useCallback(async (index: number, newValue: number) => {
    await updateAbilityScore(index, newValue);
  }, [updateAbilityScore]);

  const handleApplyPointBuy = useCallback(async (scores: AbilityScores) => {
    await CharacterStateAPI.applyPointBuy(scores);
    attributesData.invalidate();
    await attributesData.load();
  }, [attributesData]);

  const abilitiesState = attributesData.data as AbilitiesState | null;

  if (attributesData.isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[rgb(var(--color-primary))]"></div>
      </div>
    );
  }

  if (attributesData.error) {
    return (
      <Card variant="error">
        <p className="text-error">{attributesData.error}</p>
      </Card>
    );
  }

  if (!character) {
    return (
      <Card variant="warning">
        <p className="text-muted">No character loaded. Please import a save file to begin.</p>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
       <div className="grid grid-cols-3 gap-3">
         <Card variant="default" padding="sm" className="bg-[rgb(var(--color-surface-1))]">
           <div className="text-center">
             <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase tracking-wider mb-1">{t('abilityScores.pointsSpent')}</div>
             <div className="text-2xl font-bold text-[rgb(var(--color-text-primary))]">
               {pointSummary?.total_spent ?? 0}
             </div>
           </div>
         </Card>
         <Card variant="default" padding="sm" className="bg-[rgb(var(--color-surface-1))]">
           <div className="text-center">
             <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase tracking-wider mb-1">{t('abilityScores.availablePoints')}</div>
             <div className="text-2xl font-bold text-[rgb(var(--color-primary))]">
               {pointSummary?.available ?? 0}
             </div>
           </div>
         </Card>
         <Card variant="default" padding="sm" className="bg-[rgb(var(--color-surface-1))] flex items-center justify-center">
           <Button variant="secondary" size="sm" onClick={() => setShowPointBuy(true)}>
             {t('abilityScores.pointBuy.button')}
           </Button>
         </Card>
       </div>

      <CoreAbilityScoresSection
        abilityScores={abilityScores}
        onAbilityScoreChange={handleAbilityScoreUpdate}
      />

      <VitalStatisticsSection
        stats={stats}
        onStatsChange={updateStats}
      />

      <AlignmentSection
        alignment={alignment}
        onAlignmentChange={updateAlignment}
      />

      <PointBuyModal
        isOpen={showPointBuy}
        onClose={() => setShowPointBuy(false)}
        pointBuyState={abilitiesState?.point_buy ?? null}
        onApply={handleApplyPointBuy}
      />
    </div>
  );
}