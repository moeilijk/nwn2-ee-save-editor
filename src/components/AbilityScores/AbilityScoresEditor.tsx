import { useEffect, useCallback } from 'react';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { useAbilityScores } from '@/hooks/useAbilityScores';
import CoreAbilityScoresSection from './CoreAbilityScoresSection';
import VitalStatisticsSection from './VitalStatisticsSection';
import AlignmentSection from './AlignmentSection';
import { Card } from '@/components/ui/Card';
import type { AbilitiesState, CombatSummary, SaveSummary } from '@/lib/bindings';

export default function AbilityScoresEditor() {
  const { character } = useCharacterContext();
  const attributesData = useSubsystem('abilityScores');
  const combatData = useSubsystem('combat');
  const savesData = useSubsystem('saves');

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
    updateStartingAbilityScore,
    updateLevelUpScore,
    updateStats,
    updateAlignment,
    pointSummary,
    startingSummary,
  } = useAbilityScores(
    attributesData.data as AbilitiesState | null,
    {
      combat: combatData.data as CombatSummary | null,
      saves: savesData.data as SaveSummary | null
    }
  );
  
  const handleStartingScoreUpdate = useCallback(async (index: number, newValue: number) => {
    await updateStartingAbilityScore(index, newValue);
  }, [updateStartingAbilityScore]);

  const handleLevelUpScoreUpdate = useCallback(async (index: number, newValue: number) => {
    await updateLevelUpScore(index, newValue);
  }, [updateLevelUpScore]);

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
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
         <Card variant="default" padding="sm" className="bg-[rgb(var(--color-surface-1))]">
           <div className="text-center">
             <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase tracking-wider mb-1">Point Buy Spent</div>
             <div className="text-2xl font-bold text-[rgb(var(--color-text-primary))]">
               {startingSummary?.total_spent ?? 0}
             </div>
           </div>
         </Card>
         <Card variant="default" padding="sm" className="bg-[rgb(var(--color-surface-1))]">
           <div className="text-center">
             <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase tracking-wider mb-1">Point Buy Remaining</div>
             <div className="text-2xl font-bold text-[rgb(var(--color-primary))]">
               {startingSummary?.available ?? 0}
             </div>
           </div>
         </Card>
         <Card variant="default" padding="sm" className="bg-[rgb(var(--color-surface-1))]">
           <div className="text-center">
             <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase tracking-wider mb-1">Level-Ups Used</div>
             <div className="text-2xl font-bold text-[rgb(var(--color-text-primary))]">
               {pointSummary?.total_spent ?? 0}
             </div>
           </div>
         </Card>
         <Card variant="default" padding="sm" className="bg-[rgb(var(--color-surface-1))]">
           <div className="text-center">
             <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase tracking-wider mb-1">Level-Ups Available</div>
             <div className="text-2xl font-bold text-[rgb(var(--color-primary))]">
               {pointSummary?.available ?? 0}
             </div>
           </div>
         </Card>
       </div>

      <CoreAbilityScoresSection
        abilityScores={abilityScores}
        onStartingScoreChange={handleStartingScoreUpdate}
        onLevelUpScoreChange={handleLevelUpScoreUpdate}
        availableStartingPoints={startingSummary?.available}
        availableLevelUpPoints={pointSummary?.available}
      />

      <VitalStatisticsSection
        stats={stats}
        onStatsChange={updateStats}
      />

      <AlignmentSection
        alignment={alignment}
        onAlignmentChange={updateAlignment}
      />
    </div>
  );
}
