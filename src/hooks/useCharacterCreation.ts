import { useState } from 'react';
import { useTauri } from '@/providers/TauriProvider';

interface CharacterData {
  firstName: string;
  lastName: string;
  age: number;
  gender: number;
  deity: string;
  raceId: number;
  subraceId?: number;
  classes: Array<{
    classId: number;
    level: number;
    domains?: [number?, number?];
  }>;
  strength: number;
  dexterity: number;
  constitution: number;
  intelligence: number;
  wisdom: number;
  charisma: number;
  lawChaos: number;
  goodEvil: number;
  skills: Record<number, number>;
  feats: number[];
  appearanceType: number;
  portraitId: string;
  hairStyle: number;
  hairColor?: { r: number; g: number; b: number; a: number; };
  headModel: number;
}

export const useCharacterCreation = () => {
  const { api: _api } = useTauri();
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createCharacter = async (characterData: CharacterData) => {
    setIsCreating(true);
    setError(null);
    try {
        throw new Error("Character creation not implemented in Rust backend yet");
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
      throw err;
    } finally {
      setIsCreating(false);
    }
  };

  const getTemplates = async () => {
    try {
        return [];
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
      throw err;
    }
  };

  const exportToLocalVault = async (sourcePath: string, backupExisting: boolean = true) => {
    throw new Error("Export not implemented");
  };

  const createAndExportForPlay = async (characterData: CharacterData) => {
    try {
      const createResult = await createCharacter(characterData);
      return createResult;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
      throw err;
    }
  };

  return {
    createCharacter,
    getTemplates,
    exportToLocalVault,
    createAndExportForPlay,
    isCreating,
    error
  };
};