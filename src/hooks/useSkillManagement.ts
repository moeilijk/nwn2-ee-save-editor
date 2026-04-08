import { useCallback } from 'react';
import { CharacterAPI } from '@/services/characterApi';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';

const SKILL_DEPENDENT_SUBSYSTEMS = ['combat', 'classes'] as const;

export function useSkillManagement() {
  const { character, invalidateSubsystems } = useCharacterContext();
  const skills = useSubsystem('skills');

  const updateSkills = useCallback(async (updates: Record<number, number>) => {
    if (!character?.id) throw new Error('No character loaded');

    await CharacterAPI.updateSkills(character.id, updates);
    await skills.load({ silent: true });
    await invalidateSubsystems([...SKILL_DEPENDENT_SUBSYSTEMS]);
  }, [character?.id, skills, invalidateSubsystems]);

  const resetSkills = useCallback(async () => {
    if (!character?.id) throw new Error('No character loaded');

    await CharacterAPI.resetSkills(character.id);
    await skills.load({ silent: true });
    await invalidateSubsystems([...SKILL_DEPENDENT_SUBSYSTEMS]);
  }, [character?.id, skills, invalidateSubsystems]);

  return { updateSkills, resetSkills };
}
