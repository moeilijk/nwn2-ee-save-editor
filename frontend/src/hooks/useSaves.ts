import { useState, useEffect, useCallback } from 'react';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { TauriAPI } from '@/lib/tauri-api';
import type { SaveSummary } from '@/lib/bindings';

export type { SaveSummary, SavingThrows, SaveBreakdown } from '@/lib/bindings';

export function useSaves() {
  const { character } = useCharacterContext();
  const [savesData, setSavesData] = useState<SaveSummary | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSaves = useCallback(async () => {
    if (!character?.id) return;

    setIsLoading(true);
    setError(null);

    try {
      const data = await TauriAPI.getSaveSummary() as SaveSummary;
      setSavesData(data);
    } catch (err) {
      console.error("Failed to load save summary:", err);
      setError(err instanceof Error ? err.message : 'Failed to load saves');
    } finally {
      setIsLoading(false);
    }
  }, [character]);

  useEffect(() => {
    if (character?.id) {
      loadSaves();
    }
  }, [character?.id, loadSaves]);

  const updateSavingThrowBonus = useCallback(async (saveType: 'fortitude' | 'reflex' | 'will', bonus: number) => {
    if (!character?.id) return;

    try {
      await CharacterAPI.updateSavingThrows(character.id, { [saveType]: bonus });
      await loadSaves();
    } catch (err) {
      throw err;
    }
  }, [character?.id, loadSaves]);

  return {
    savesData,
    isLoading,
    error,
    reload: loadSaves,
    updateSavingThrowBonus
  };
}
