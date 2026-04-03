import React, { useState, useCallback, useMemo, useEffect } from 'react';
import { useTranslations } from '@/hooks/useTranslations';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { CharacterStateAPI } from '@/lib/api/character-state';
import type {
  Alignment,
  AbilitiesState,
  CombatSummary,
  SaveSummary,
  AbilityScores as RustAbilityScores,
  AbilitiesUpdates,
} from '@/lib/bindings';

export interface AbilityScore {
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

export interface CharacterStats {
  hitPoints: number;
  maxHitPoints: number;
  experience: number;
  level: number;

  armorClass: {
    base: number;
    total: number;
    dexMod?: number;
    equipment?: number;
  };
  initiative: {
    base: number;
    total: number;
    dexMod?: number;
    feats?: number;
  };

  fortitude: {
    base: number;
    total: number;
    abilityMod?: number;
    classMod?: number;
    equipment?: number;
    racial?: number;
    feat?: number;
  };
  reflex: {
    base: number;
    total: number;
    abilityMod?: number;
    classMod?: number;
    equipment?: number;
    racial?: number;
    feat?: number;
  };
  will: {
    base: number;
    total: number;
    abilityMod?: number;
    classMod?: number;
    equipment?: number;
    racial?: number;
    feat?: number;
  };
}

export interface PointSummary {
  total_available: number;
  total_spent: number;
  available: number;
}

export interface CombatStatsInput {
  combat?: CombatSummary | null;
  saves?: SaveSummary | null;
}

export type { Alignment } from '@/lib/bindings';

type AbilityKey = keyof RustAbilityScores;

export function useAbilityScores(
  abilityScoreData?: AbilitiesState | null,
  combatStats?: CombatStatsInput
) {
  const t = useTranslations();
  const { characterId, invalidateSubsystems } = useCharacterContext();

  const [localStartingOverrides, setLocalStartingOverrides] = useState<Record<string, number>>({});
  const [localLevelUpOverrides, setLocalLevelUpOverrides] = useState<Record<string, number>>({});
  const [localStatsOverrides, setLocalStatsOverrides] = useState<Partial<CharacterStats>>({});

  // Keep track of the last seen base scores to avoid wiping local overrides on unrelated refreshes (e.g. from skills)
  const prevBaseScoresRef = React.useRef<string | null>(null);

  useEffect(() => {
    if (abilityScoreData?.base_scores) {
      const newBaseScoresRaw = JSON.stringify(abilityScoreData.base_scores);
      if (prevBaseScoresRef.current !== newBaseScoresRaw) {
        setLocalStartingOverrides({});
        setLocalLevelUpOverrides({});
        setLocalStatsOverrides({});
        prevBaseScoresRef.current = newBaseScoresRaw;
      }
    }
  }, [abilityScoreData?.base_scores]);

  const calculateModifier = useCallback((value: number): number => {
    return Math.floor((value - 10) / 2);
  }, []);

  const abilityScores = useMemo((): AbilityScore[] => {
    if (!abilityScoreData) return [];

    const getBackendLevelUpValue = (attrKey: AbilityKey): number => {
      if (!abilityScoreData.point_summary?.level_increases) return 0;
      const abilityIndexMap: Record<AbilityKey, number> = { Str: 0, Dex: 1, Con: 2, Int: 3, Wis: 4, Cha: 5 };
      const targetIndex = abilityIndexMap[attrKey];
      return abilityScoreData.point_summary.level_increases.filter(inc => inc.ability === targetIndex).length;
    };

    const getStartingValue = (attrKey: AbilityKey) => {
      const localValue = localStartingOverrides[attrKey];
      if (localValue !== undefined) {
        return localValue;
      }

      const pointBuyStarting = abilityScoreData.point_buy?.starting_scores?.[attrKey];
      if (pointBuyStarting !== undefined) {
        return pointBuyStarting;
      }

      const currentBase = abilityScoreData.base_scores?.[attrKey] ?? 10;
      return currentBase - getBackendLevelUpValue(attrKey);
    };

    const getLevelUpValue = (attrKey: AbilityKey) => {
      return localLevelUpOverrides[attrKey] ?? getBackendLevelUpValue(attrKey);
    };

    const getDisplayValue = (attrKey: AbilityKey) => {
      const racialBonus = abilityScoreData.racial_modifiers?.[attrKey] ?? 0;
      const equipBonus = abilityScoreData.equipment_modifiers?.[attrKey] ?? 0;
      return getStartingValue(attrKey) + getLevelUpValue(attrKey) + racialBonus + equipBonus;
    };

    return [
      {
        name: t('abilityScores.strength'),
        shortName: 'STR',
        value: getDisplayValue('Str'),
        modifier: calculateModifier(getDisplayValue('Str')),
        startingValue: getStartingValue('Str'),
        levelUpValue: getLevelUpValue('Str'),
        breakdown: {
          starting: getStartingValue('Str'),
          levelUp: getLevelUpValue('Str'),
          racial: abilityScoreData.racial_modifiers?.Str ?? 0,
          equipment: abilityScoreData.equipment_modifiers?.Str ?? 0,
          enhancement: 0,
          temporary: 0
        }
      },
      {
        name: t('abilityScores.dexterity'),
        shortName: 'DEX',
        value: getDisplayValue('Dex'),
        modifier: calculateModifier(getDisplayValue('Dex')),
        startingValue: getStartingValue('Dex'),
        levelUpValue: getLevelUpValue('Dex'),
        breakdown: {
          starting: getStartingValue('Dex'),
          levelUp: getLevelUpValue('Dex'),
          racial: abilityScoreData.racial_modifiers?.Dex ?? 0,
          equipment: abilityScoreData.equipment_modifiers?.Dex ?? 0,
          enhancement: 0,
          temporary: 0
        }
      },
      {
        name: t('abilityScores.constitution'),
        shortName: 'CON',
        value: getDisplayValue('Con'),
        modifier: calculateModifier(getDisplayValue('Con')),
        startingValue: getStartingValue('Con'),
        levelUpValue: getLevelUpValue('Con'),
        breakdown: {
          starting: getStartingValue('Con'),
          levelUp: getLevelUpValue('Con'),
          racial: abilityScoreData.racial_modifiers?.Con ?? 0,
          equipment: abilityScoreData.equipment_modifiers?.Con ?? 0,
          enhancement: 0,
          temporary: 0
        }
      },
      {
        name: t('abilityScores.intelligence'),
        shortName: 'INT',
        value: getDisplayValue('Int'),
        modifier: calculateModifier(getDisplayValue('Int')),
        startingValue: getStartingValue('Int'),
        levelUpValue: getLevelUpValue('Int'),
        breakdown: {
          starting: getStartingValue('Int'),
          levelUp: getLevelUpValue('Int'),
          racial: abilityScoreData.racial_modifiers?.Int ?? 0,
          equipment: abilityScoreData.equipment_modifiers?.Int ?? 0,
          enhancement: 0,
          temporary: 0
        }
      },
      {
        name: t('abilityScores.wisdom'),
        shortName: 'WIS',
        value: getDisplayValue('Wis'),
        modifier: calculateModifier(getDisplayValue('Wis')),
        startingValue: getStartingValue('Wis'),
        levelUpValue: getLevelUpValue('Wis'),
        breakdown: {
          starting: getStartingValue('Wis'),
          levelUp: getLevelUpValue('Wis'),
          racial: abilityScoreData.racial_modifiers?.Wis ?? 0,
          equipment: abilityScoreData.equipment_modifiers?.Wis ?? 0,
          enhancement: 0,
          temporary: 0
        }
      },
      {
        name: t('abilityScores.charisma'),
        shortName: 'CHA',
        value: getDisplayValue('Cha'),
        modifier: calculateModifier(getDisplayValue('Cha')),
        startingValue: getStartingValue('Cha'),
        levelUpValue: getLevelUpValue('Cha'),
        breakdown: {
          starting: getStartingValue('Cha'),
          levelUp: getLevelUpValue('Cha'),
          racial: abilityScoreData.racial_modifiers?.Cha ?? 0,
          equipment: abilityScoreData.equipment_modifiers?.Cha ?? 0,
          enhancement: 0,
          temporary: 0
        }
      },
    ];
  }, [abilityScoreData, localStartingOverrides, localLevelUpOverrides, t, calculateModifier]);

  const stats = useMemo((): CharacterStats => {
    const defaultStats: CharacterStats = {
      hitPoints: 0,
      maxHitPoints: 1,
      experience: 0,
      level: 1,
      armorClass: { base: 0, total: 10 },
      fortitude: { base: 0, total: 0 },
      reflex: { base: 0, total: 0 },
      will: { base: 0, total: 0 },
      initiative: { base: 0, total: 0 },
    };

    if (!abilityScoreData) {
      return defaultStats;
    }

    const combat = combatStats?.combat;
    const saves = combatStats?.saves;

    const buildArmorClass = () => {
      if (!combat?.armor_class) return defaultStats.armorClass;
      const ac = combat.armor_class;
      return {
        base: localStatsOverrides.armorClass?.base ?? ac.breakdown?.natural ?? 0,
        total: ac.total ?? 10,
        dexMod: ac.breakdown?.dex ?? 0,
        equipment: (ac.breakdown?.armor ?? 0) + (ac.breakdown?.shield ?? 0)
      };
    };

    const buildInitiative = () => {
      if (!combat?.initiative) return defaultStats.initiative;
      const init = combat.initiative;
      return {
        base: localStatsOverrides.initiative?.base ?? init.misc ?? 0,
        total: init.total ?? 0,
        dexMod: init.dex ?? 0,
        feats: 0
      };
    };

    const buildSave = (saveData: { total: number; base: number; ability: number; equipment: number; feat: number; racial: number; class_bonus: number; misc: number } | undefined, overrideKey: 'fortitude' | 'reflex' | 'will') => {
      if (!saveData) return defaultStats[overrideKey];
      return {
        base: localStatsOverrides[overrideKey]?.base ?? saveData.misc ?? 0,
        total: saveData.total ?? 0,
        abilityMod: saveData.ability ?? 0,
        classMod: saveData.base ?? 0,
        equipment: saveData.equipment ?? 0,
        racial: saveData.racial ?? 0,
        feat: (saveData.feat ?? 0) + (saveData.class_bonus ?? 0),
      };
    };

    return {
      hitPoints: localStatsOverrides.hitPoints ?? abilityScoreData.hit_points?.current ?? 0,
      maxHitPoints: localStatsOverrides.maxHitPoints ?? abilityScoreData.hit_points?.max ?? 1,
      experience: localStatsOverrides.experience ?? 0,
      level: localStatsOverrides.level ?? 1,
      armorClass: buildArmorClass(),
      initiative: buildInitiative(),
      fortitude: buildSave(saves?.saves?.fortitude, 'fortitude'),
      reflex: buildSave(saves?.saves?.reflex, 'reflex'),
      will: buildSave(saves?.saves?.will, 'will'),
    };
  }, [abilityScoreData, combatStats, localStatsOverrides]);

  const [alignment, setAlignment] = useState<Alignment>({
    law_chaos: 50,
    good_evil: 50,
  });

  useEffect(() => {
    const fetchAlignment = async () => {
      if (!characterId) return;

      try {
        const alignmentData = await CharacterAPI.getAlignment(characterId);
        setAlignment({
          law_chaos: alignmentData.law_chaos,
          good_evil: alignmentData.good_evil,
        });
      } catch (error) {
        console.error('Failed to fetch alignment:', error);
      }
    };

    fetchAlignment();
  }, [characterId]);

  const calculatePointBuyCost = useCallback((scores: RustAbilityScores): number => {
    const getCost = (score: number) => {
      if (score <= 8) return 0;
      if (score >= 18) return 16;
      return [0, 1, 2, 3, 4, 5, 6, 8, 10, 13, 16][score - 8];
    };

    return getCost(scores.Str) + getCost(scores.Dex) + getCost(scores.Con)
      + getCost(scores.Int) + getCost(scores.Wis) + getCost(scores.Cha);
  }, []);

  const updateStartingAbilityScore = useCallback(async (index: number, newValue: number) => {
    if (!characterId || !abilityScores[index]) return;

    const clampedValue = Math.max(8, Math.min(18, newValue));
    const attr = abilityScores[index];
    const attributeMapping = {
      'STR': 'Str',
      'DEX': 'Dex', 
      'CON': 'Con',
      'INT': 'Int',
      'WIS': 'Wis',
      'CHA': 'Cha'
    };
    
    const backendAttrName = attributeMapping[attr.shortName as keyof typeof attributeMapping];
    if (!backendAttrName) return;

    const currentStartingScores: RustAbilityScores = {
      Str: localStartingOverrides.Str ?? abilityScoreData?.point_buy?.starting_scores?.Str ?? 10,
      Dex: localStartingOverrides.Dex ?? abilityScoreData?.point_buy?.starting_scores?.Dex ?? 10,
      Con: localStartingOverrides.Con ?? abilityScoreData?.point_buy?.starting_scores?.Con ?? 10,
      Int: localStartingOverrides.Int ?? abilityScoreData?.point_buy?.starting_scores?.Int ?? 10,
      Wis: localStartingOverrides.Wis ?? abilityScoreData?.point_buy?.starting_scores?.Wis ?? 10,
      Cha: localStartingOverrides.Cha ?? abilityScoreData?.point_buy?.starting_scores?.Cha ?? 10,
    };
    const nextScores = { ...currentStartingScores, [backendAttrName]: clampedValue };
    const budget = abilityScoreData?.point_buy?.budget ?? 32;
    if (calculatePointBuyCost(nextScores) > budget) {
      return;
    }

    setLocalStartingOverrides(prev => ({
      ...prev,
      [backendAttrName]: clampedValue
    }));
    
    try {
      await CharacterStateAPI.updateStartingAbilities(nextScores);
      await invalidateSubsystems(['abilityScores', 'combat', 'saves', 'skills', 'feats', 'spells']);

    } catch (err) {
      setLocalStartingOverrides(prev => {
        const updated = { ...prev };
        delete updated[backendAttrName];
        return updated;
      });
      throw err;
    }
  }, [characterId, abilityScores, abilityScoreData, localStartingOverrides, invalidateSubsystems, calculatePointBuyCost]);

  const updateLevelUpScore = useCallback(async (index: number, newValue: number) => {
    if (!characterId || !abilityScores[index]) return;

    const attr = abilityScores[index];
    const attributeMapping = {
      'STR': 'Str',
      'DEX': 'Dex',
      'CON': 'Con',
      'INT': 'Int',
      'WIS': 'Wis',
      'CHA': 'Cha'
    };

    const backendAttrName = attributeMapping[attr.shortName as keyof typeof attributeMapping];
    if (!backendAttrName) return;

    const currentStartingValue = localStartingOverrides[backendAttrName]
      ?? abilityScoreData?.point_buy?.starting_scores?.[backendAttrName as AbilityKey]
      ?? 10;
    const currentLevelUpValue = localLevelUpOverrides[backendAttrName]
      ?? attr.breakdown?.levelUp
      ?? 0;
    const totalAvailable = abilityScoreData?.point_summary?.expected_increases ?? 0;
    const totalSpent = abilityScores.reduce((sum, score) => {
      const key = attributeMapping[score.shortName as keyof typeof attributeMapping];
      return sum + (localLevelUpOverrides[key] ?? score.breakdown?.levelUp ?? 0);
    }, 0);
    const maxForThisAbility = currentLevelUpValue + Math.max(0, totalAvailable - totalSpent);
    const clampedValue = Math.max(0, Math.min(maxForThisAbility, newValue));
    const targetBaseScore = currentStartingValue + clampedValue;

    setLocalLevelUpOverrides(prev => ({
      ...prev,
      [backendAttrName]: clampedValue
    }));

    try {
      await CharacterStateAPI.updateAbilities({
        [backendAttrName]: targetBaseScore
      } as AbilitiesUpdates);
      await invalidateSubsystems(['abilityScores', 'combat', 'saves', 'skills', 'feats', 'spells']);
    } catch (err) {
      setLocalLevelUpOverrides(prev => {
        const updated = { ...prev };
        delete updated[backendAttrName];
        return updated;
      });
      throw err;
    }
  }, [characterId, abilityScores, abilityScoreData, localStartingOverrides, localLevelUpOverrides, invalidateSubsystems]);

  const updateAbilityScoreByShortName = useCallback(async (shortName: string, newValue: number) => {
    const index = abilityScores.findIndex(attr => attr.shortName === shortName);
    if (index !== -1) {
      await updateLevelUpScore(index, newValue);
    }
  }, [abilityScores, updateLevelUpScore]);

  const updateStats = useCallback(async (updates: Partial<CharacterStats>) => {
    if (!characterId) return;

    setLocalStatsOverrides(prev => ({ ...prev, ...updates }));
    
    try {
      if (updates.armorClass?.base !== undefined) {
        await CharacterAPI.updateArmorClass(characterId, updates.armorClass.base);
      }
      
      if (updates.initiative?.base !== undefined) {
        await CharacterAPI.updateInitiativeBonus(characterId, updates.initiative.base);
      }

      const saveUpdates: Record<string, number> = {};
      if (updates.fortitude?.base !== undefined) saveUpdates.fortitude = updates.fortitude.base;
      if (updates.reflex?.base !== undefined) saveUpdates.reflex = updates.reflex.base;
      if (updates.will?.base !== undefined) saveUpdates.will = updates.will.base;
      
      if (Object.keys(saveUpdates).length > 0) {
        await CharacterAPI.updateSavingThrows(characterId, saveUpdates);
      }

      await invalidateSubsystems(['abilityScores', 'combat', 'saves']);

    } catch (err) {
      setLocalStatsOverrides(prev => {
        const reverted = { ...prev };
        Object.keys(updates).forEach(key => delete reverted[key as keyof CharacterStats]);
        return reverted;
      });
      throw err;
    }
  }, [characterId, invalidateSubsystems]);

  const updateAlignment = useCallback(async (updates: Partial<Alignment>) => {
    if (!characterId) return;

    const newAlignment = { ...alignment, ...updates };

    setAlignment(newAlignment);

    try {
      const result = await CharacterAPI.updateAlignment(characterId, newAlignment);

      setAlignment({
        law_chaos: result.law_chaos,
        good_evil: result.good_evil,
      });
    } catch (err) {
      setAlignment(alignment);
      throw err;
    }
  }, [characterId, alignment]);

  const getAbilityScore = useCallback((shortName: string): AbilityScore | undefined => {
    return abilityScores.find(attr => attr.shortName === shortName);
  }, [abilityScores]);

  const getAbilityScoreModifier = useCallback((shortName: string): number => {
    const attr = getAbilityScore(shortName);
    return attr ? attr.modifier : 0;
  }, [getAbilityScore]);

  const pointSummary = useMemo((): PointSummary | undefined => {
    if (!abilityScoreData?.point_summary) return undefined;
    const ps = abilityScoreData.point_summary;
    const attributeMapping: Record<string, string> = {
      'STR': 'Str', 'DEX': 'Dex', 'CON': 'Con',
      'INT': 'Int', 'WIS': 'Wis', 'CHA': 'Cha'
    };
    const totalSpent = abilityScores.reduce((sum, score) => {
      const key = attributeMapping[score.shortName];
      return sum + (localLevelUpOverrides[key] ?? score.breakdown?.levelUp ?? 0);
    }, 0);

    return {
      total_available: ps.expected_increases ?? 0,
      total_spent: totalSpent,
      available: Math.max(0, (ps.expected_increases ?? 0) - totalSpent)
    };
  }, [abilityScoreData, abilityScores, localLevelUpOverrides]);

  const startingSummary = useMemo((): PointSummary | undefined => {
    if (!abilityScoreData?.point_buy?.starting_scores) return undefined;

    const scores: RustAbilityScores = {
      Str: localStartingOverrides.Str ?? abilityScoreData.point_buy.starting_scores.Str,
      Dex: localStartingOverrides.Dex ?? abilityScoreData.point_buy.starting_scores.Dex,
      Con: localStartingOverrides.Con ?? abilityScoreData.point_buy.starting_scores.Con,
      Int: localStartingOverrides.Int ?? abilityScoreData.point_buy.starting_scores.Int,
      Wis: localStartingOverrides.Wis ?? abilityScoreData.point_buy.starting_scores.Wis,
      Cha: localStartingOverrides.Cha ?? abilityScoreData.point_buy.starting_scores.Cha,
    };

    const totalSpent = calculatePointBuyCost(scores);
    const totalAvailable = abilityScoreData.point_buy.budget ?? 32;

    return {
      total_available: totalAvailable,
      total_spent: totalSpent,
      available: totalAvailable - totalSpent,
    };
  }, [abilityScoreData, localStartingOverrides, calculatePointBuyCost]);

  return {
    abilityScores,
    stats,
    alignment,

    updateAbilityScore: updateLevelUpScore,
    updateStartingAbilityScore,
    updateLevelUpScore,
    updateAbilityScoreByShortName,
    getAbilityScore,
    getAbilityScoreModifier,
    calculateModifier,
    updateAlignment,

    pointSummary,
    startingSummary,
    resetAbilityOverrides: () => {
      setLocalStartingOverrides({});
      setLocalLevelUpOverrides({});
    },
  };
}
