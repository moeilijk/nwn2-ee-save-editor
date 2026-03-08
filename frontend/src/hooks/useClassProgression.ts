import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ClassFeature {
  name: string;
  feature_type: string;
  description: string;
}

export interface SpellSlots {
  level_0: number;
  level_1: number;
  level_2: number;
  level_3: number;
  level_4: number;
  level_5: number;
  level_6: number;
  level_7: number;
  level_8: number;
  level_9: number;
}

export interface LevelProgression {
  level: number;
  base_attack_bonus: number;
  fortitude_save: number;
  reflex_save: number;
  will_save: number;
  features: ClassFeature[];
  spell_slots?: SpellSlots;
}

export interface ClassBasicInfo {
  hit_die: number;
  skill_points_per_level: number;
  bab_progression: string;
  save_progression: string;
  is_spellcaster: boolean;
  spell_type: string;
}

export interface ClassProgression {
  class_id: number;
  class_name: string;
  basic_info: ClassBasicInfo;
  level_progression: LevelProgression[];
  max_level_shown: number;
}

export interface UseClassProgressionOptions {
  maxLevel?: number;
  includeSpells?: boolean;
  includeProficiencies?: boolean;
  autoFetch?: boolean;
}

export interface UseClassProgressionReturn {
  progression: ClassProgression | null;
  isLoading: boolean;
  error: string | null;
  fetchProgression: (classId: number, options?: UseClassProgressionOptions) => Promise<void>;
  clearProgression: () => void;
  currentLevelFeatures: (level: number) => ClassFeature[];
  getLevelRange: (startLevel: number, endLevel: number) => LevelProgression[];
  getProgressionSummary: () => {
    totalFeatures: number;
    spellLevels: number;
    combatProgression: string;
    skillProgression: string;
  } | null;
}

export function useClassProgression(
  characterId?: string,
  options: UseClassProgressionOptions = {}
): UseClassProgressionReturn {
  const [progression, setProgression] = useState<ClassProgression | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProgression = useCallback(async (
    classId: number,
    fetchOptions?: UseClassProgressionOptions
  ) => {
    setIsLoading(true);
    setError(null);
    try {
      const maxLevel = fetchOptions?.maxLevel ?? options.maxLevel ?? 20;
      const result = await invoke<ClassProgression>('get_class_progression_details', {
        classId,
        maxLevel
      });
      setProgression(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setProgression(null);
    } finally {
      setIsLoading(false);
    }
  }, [options.maxLevel]);

  const clearProgression = useCallback(() => {
    setProgression(null);
    setError(null);
  }, []);

  const currentLevelFeatures = useCallback((level: number): ClassFeature[] => {
    if (!progression) return [];
    const levelEntry = progression.level_progression.find(lp => lp.level === level);
    return levelEntry?.features ?? [];
  }, [progression]);

  const getLevelRange = useCallback((startLevel: number, endLevel: number): LevelProgression[] => {
    if (!progression) return [];
    return progression.level_progression.filter(
      lp => lp.level >= startLevel && lp.level <= endLevel
    );
  }, [progression]);

  const getProgressionSummary = useCallback(() => {
    if (!progression) return null;

    const totalFeatures = progression.level_progression.reduce(
      (sum, lp) => sum + lp.features.length, 0
    );

    const spellLevels = progression.basic_info.is_spellcaster ? 9 : 0;

    return {
      totalFeatures,
      spellLevels,
      combatProgression: progression.basic_info.bab_progression,
      skillProgression: `${progression.basic_info.skill_points_per_level} per level`
    };
  }, [progression]);

  return {
    progression,
    isLoading,
    error,
    fetchProgression,
    clearProgression,
    currentLevelFeatures,
    getLevelRange,
    getProgressionSummary
  };
}

export function useMultiClassProgression(
  characterId?: string,
  classIds: number[] = [],
  options: UseClassProgressionOptions = {}
) {
  const [progressions, setProgressions] = useState<Record<number, ClassProgression>>({});
  const [isLoading, setIsLoading] = useState(false);
  const [errors, setErrors] = useState<Record<number, string>>({});

  const fetchAllProgressions = useCallback(async () => {
    if (classIds.length === 0) return;

    setIsLoading(true);
    const newProgressions: Record<number, ClassProgression> = {};
    const newErrors: Record<number, string> = {};

    await Promise.all(classIds.map(async (classId) => {
      try {
        const maxLevel = options.maxLevel ?? 20;
        const result = await invoke<ClassProgression>('get_class_progression_details', {
          classId,
          maxLevel
        });
        newProgressions[classId] = result;
      } catch (err) {
        newErrors[classId] = err instanceof Error ? err.message : String(err);
      }
    }));

    setProgressions(newProgressions);
    setErrors(newErrors);
    setIsLoading(false);
  }, [classIds, options.maxLevel]);

  const getProgression = useCallback((classId: number) => {
    return progressions[classId] ?? null;
  }, [progressions]);

  const getError = useCallback((classId: number) => {
    return errors[classId] ?? null;
  }, [errors]);

  return {
    progressions,
    isLoading,
    errors,
    fetchAllProgressions,
    getProgression,
    getError,
    hasAnyData: Object.keys(progressions).length > 0
  };
}
