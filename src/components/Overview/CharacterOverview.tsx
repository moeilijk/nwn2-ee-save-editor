
import { useState, useEffect } from 'react';
import { useTranslations } from '@/hooks/useTranslations';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { display, formatModifier, formatNumber } from '@/utils/dataHelpers';
import { CharacterAPI, type BackgroundOption, type RaceDataResponse } from '@/services/characterApi';
import { inventoryAPI } from '@/services/inventoryApi';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { gameData } from '@/lib/api/gamedata';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import AlignmentGrid from '@/components/AbilityScores/AlignmentGrid';
import CampaignOverview from './CampaignOverview';
import DeitySelectionModal from './DeitySelectionModal';
import RaceSelectionModal from './RaceSelectionModal';

interface CharacterOverviewProps {
  onNavigate?: (tab: string) => void;
}

interface CollapsibleSectionProps {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
  badge?: string | number;
}

interface GenderOption {
  id: number;
  name: string;
}

interface AlignmentValues {
  law_chaos: number;
  good_evil: number;
}

interface ParsedBackgroundDescription {
  flavor: string;
  prerequisites: string[];
  effects: string[];
}

const DEFAULT_GENDER_OPTIONS: GenderOption[] = [
  { id: 0, name: 'Male' },
  { id: 1, name: 'Female' },
];

function getGenderLabel(id: number, fallbackName?: string): string {
  if (fallbackName && !/^gender\s+\d+$/i.test(fallbackName.trim())) {
    return fallbackName;
  }

  return id === 1 ? 'Female' : 'Male';
}

function getCharacterGenderId(gender: string | undefined): number | null {
  if (!gender) {
    return null;
  }

  const normalized = gender.trim().toLowerCase();
  if (normalized === 'male') {
    return 0;
  }
  if (normalized === 'female') {
    return 1;
  }

  return null;
}

function stripHtmlTags(text: string): string {
  return text.replace(/<\/?[^>]+(>|$)/g, '');
}

function splitDescriptionItems(chunks: string[]): string[] {
  return chunks
    .flatMap(chunk => chunk.split(/\r?\n|;/))
    .flatMap(chunk => chunk.split(/\s*,\s*/))
    .map(item => item.trim())
    .filter(Boolean);
}

function parseBackgroundDescription(rawDescription?: string): ParsedBackgroundDescription {
  if (!rawDescription) {
    return {
      flavor: '',
      prerequisites: [],
      effects: [],
    };
  }

  const lines = stripHtmlTags(rawDescription)
    .split('\n')
    .map(line => line.trim())
    .filter(Boolean);

  const flavorLines: string[] = [];
  const prerequisiteChunks: string[] = [];
  const effectChunks: string[] = [];
  let activeSection: 'flavor' | 'prerequisites' | 'effects' = 'flavor';

  for (const line of lines) {
    const lowerLine = line.toLowerCase();

    if (lowerLine.startsWith('prerequisite:') || lowerLine.startsWith('prerequisites:')) {
      activeSection = 'prerequisites';
      const separatorIndex = line.indexOf(':');
      const content = separatorIndex >= 0 ? line.slice(separatorIndex + 1).trim() : '';
      if (content) {
        prerequisiteChunks.push(content);
      }
      continue;
    }

    if (lowerLine.startsWith('effects:')) {
      activeSection = 'effects';
      const separatorIndex = line.indexOf(':');
      const content = separatorIndex >= 0 ? line.slice(separatorIndex + 1).trim() : '';
      if (content) {
        effectChunks.push(content);
      }
      continue;
    }

    if (activeSection === 'prerequisites') {
      prerequisiteChunks.push(line);
      continue;
    }

    if (activeSection === 'effects') {
      effectChunks.push(line);
      continue;
    }

    flavorLines.push(line);
  }

  return {
    flavor: flavorLines.join(' '),
    prerequisites: splitDescriptionItems(prerequisiteChunks),
    effects: splitDescriptionItems(effectChunks),
  };
}

function BackgroundDescriptionSections({
  description,
  missingRequirements,
}: {
  description?: string;
  missingRequirements?: string[];
}) {
  const sections = parseBackgroundDescription(description);
  const hasContent =
    sections.flavor.length > 0 ||
    sections.prerequisites.length > 0 ||
    sections.effects.length > 0 ||
    (missingRequirements?.length ?? 0) > 0;

  if (!hasContent) {
    return null;
  }

  return (
    <div className="mt-2 space-y-2">
      {sections.flavor && (
        <div className="text-sm text-[rgb(var(--color-text-secondary))]">
          {sections.flavor}
        </div>
      )}

      {sections.prerequisites.length > 0 && (
        <div className="space-y-1">
          <div className="text-[10px] uppercase font-semibold tracking-wide text-[rgb(var(--color-text-muted))]">
            Requires
          </div>
          <div className="flex flex-wrap gap-1.5">
            {sections.prerequisites.map(requirement => (
              <span
                key={requirement}
                className="inline-flex items-center rounded-md border border-[rgb(var(--color-surface-border)/0.65)] bg-[rgb(var(--color-surface-2)/0.8)] px-2 py-1 text-xs text-[rgb(var(--color-text-secondary))]"
              >
                {requirement}
              </span>
            ))}
          </div>
        </div>
      )}

      {sections.effects.length > 0 && (
        <div className="space-y-1">
          <div className="text-[10px] uppercase font-semibold tracking-wide text-[rgb(var(--color-text-muted))]">
            Effects
          </div>
          <div className="flex flex-wrap gap-1.5">
            {sections.effects.map(effect => (
              <span
                key={effect}
                className="inline-flex items-center rounded-md border border-[rgb(var(--color-primary)/0.25)] bg-[rgb(var(--color-primary)/0.08)] px-2 py-1 text-xs text-[rgb(var(--color-text-primary))]"
              >
                {effect}
              </span>
            ))}
          </div>
        </div>
      )}

      {missingRequirements && missingRequirements.length > 0 && (
        <div className="space-y-1">
          <div className="text-[10px] uppercase font-semibold tracking-wide text-[rgb(var(--color-danger))]">
            Missing
          </div>
          <div className="flex flex-wrap gap-1.5">
            {missingRequirements.map(requirement => (
              <span
                key={requirement}
                className="inline-flex items-center rounded-md border border-[rgb(var(--color-danger)/0.35)] bg-[rgb(var(--color-danger)/0.08)] px-2 py-1 text-xs text-[rgb(var(--color-danger))]"
              >
                {requirement}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function CollapsibleSection({ title, children, defaultOpen = false, badge }: CollapsibleSectionProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  
  return (
    <div className="group">
      <div className={`bg-gradient-to-r ${isOpen ? 'from-[rgb(var(--color-surface-2))] to-[rgb(var(--color-surface-1))]' : 'from-[rgb(var(--color-surface-1))] to-[rgb(var(--color-surface-1))]'} rounded-lg border border-[rgb(var(--color-surface-border)/0.5)] overflow-hidden transition-all duration-300 hover:border-[rgb(var(--color-primary)/0.3)]`}>
        <Button
          onClick={() => setIsOpen(!isOpen)}
          variant="ghost"
          className="w-full p-4 flex items-center justify-between h-auto"
        >
          <div className="flex items-center space-x-3">
            <h3 className="text-lg font-semibold text-[rgb(var(--color-text-primary))]">{title}</h3>
            {badge && (
              <span className="px-2.5 py-1 bg-gradient-to-r from-[rgb(var(--color-primary)/0.15)] to-[rgb(var(--color-primary)/0.1)] text-[rgb(var(--color-primary))] text-xs font-medium rounded-full">
                {badge}
              </span>
            )}
          </div>
          <svg 
            className={`w-5 h-5 text-[rgb(var(--color-text-muted))] transition-all duration-300 ${isOpen ? 'rotate-180' : ''}`}
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </Button>
        <div className={`transition-all duration-300 ease-out ${isOpen ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0 overflow-hidden'}`}>
          <div className={`px-6 md:px-8 pb-6 md:pb-8 ${isOpen ? 'border-t border-[rgb(var(--color-surface-border)/0.3)]' : 'border-t-0 border-transparent'}`}>
            <div className="pt-6 md:pt-8 grid-flow-row">
              {children}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function CharacterOverview({ onNavigate: _onNavigate }: CharacterOverviewProps) {
  const t = useTranslations();
  const { character, isLoading, error, refreshAll: _refreshAll, updateCharacterPartial, invalidateSubsystems } = useCharacterContext();
  const combat = useSubsystem('combat');
  const skills = useSubsystem('skills');
  const feats = useSubsystem('feats');
  const spells = useSubsystem('spells');
  const saves = useSubsystem('saves');
  const abilities = useSubsystem('abilityScores');
  const classes = useSubsystem('classes');
  const { handleError } = useErrorHandler();

  // Name editing state
  const [isEditingName, setIsEditingName] = useState(false);
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [activeOverviewEditor, setActiveOverviewEditor] = useState<null | 'identity' | 'alignment' | 'progress' | 'background'>(null);
  const [availableGenders, setAvailableGenders] = useState<GenderOption[]>([]);
  const [availableBackgrounds, setAvailableBackgrounds] = useState<BackgroundOption[]>([]);
  const [selectedGenderId, setSelectedGenderId] = useState<number | null>(null);
  const [selectedBackgroundId, setSelectedBackgroundId] = useState<number | null>(null);
  const [raceData, setRaceData] = useState<RaceDataResponse | null>(null);
  const [ageInput, setAgeInput] = useState('');
  const [alignmentDraft, setAlignmentDraft] = useState<AlignmentValues>({ law_chaos: 50, good_evil: 50 });
  const [experienceInput, setExperienceInput] = useState('');
  const [goldInput, setGoldInput] = useState('');
  
  // Deity editing state
  const [isDeityModalOpen, setIsDeityModalOpen] = useState(false);
  const [, setDeity] = useState('');

  // Race editing state
  const [isRaceModalOpen, setIsRaceModalOpen] = useState(false);

  
  // Biography editing state
  const [isEditingBio, setIsEditingBio] = useState(false);
  const [biography, setBiography] = useState('');

  const BACKGROUND_DEPENDENT_SUBSYSTEMS = ['feats', 'skills', 'combat', 'abilityScores', 'saves'] as const;
  const RACE_DEPENDENT_SUBSYSTEMS = ['abilityScores', 'combat', 'saves', 'skills', 'feats', 'spells'] as const;
  

  
  // Initialize name fields when character changes
  useEffect(() => {
    if (character && character.name) {
      const parts = character.name.split(' ');
      setFirstName(parts[0] || '');
      setLastName(parts.slice(1).join(' ') || '');
    }
  }, [character]);
  
  // Initialize deity when character changes
  useEffect(() => {
    if (character) {
      setDeity(character.deity || '');
    }
  }, [character]);
  
  // Initialize biography when character changes
  useEffect(() => {
    if (character) {
      setBiography(character.biography || '');
    }
  }, [character]);

  useEffect(() => {
    gameData.genders()
      .then(response => {
        const normalizedOptions = response.results
          .filter(option => option.id === 0 || option.id === 1)
          .map(option => ({
            id: option.id,
            name: getGenderLabel(option.id, option.name),
          }));

        setAvailableGenders(
          normalizedOptions.length > 0 ? normalizedOptions : DEFAULT_GENDER_OPTIONS
        );
      })
      .catch(error => {
        console.warn('Failed to load genders for overview editor:', error);
        setAvailableGenders(DEFAULT_GENDER_OPTIONS);
      });
  }, []);

  useEffect(() => {
    gameData.backgrounds()
      .then(response => setAvailableBackgrounds(response.results))
      .catch(error => {
        console.warn('Failed to load backgrounds for overview editor:', error);
        setAvailableBackgrounds([]);
      });
  }, []);

  useEffect(() => {
    if (!character) {
      return;
    }

    setAgeInput(String(character.age ?? 0));
    setExperienceInput(String(character.experience ?? 0));
    setGoldInput(String(character.gold ?? 0));
    setAlignmentDraft(character.alignment_values ?? { law_chaos: 50, good_evil: 50 });
  }, [character]);

  useEffect(() => {
    if (!character || availableGenders.length === 0) {
      return;
    }

    setSelectedGenderId(getCharacterGenderId(character.gender));
  }, [availableGenders, character]);

  useEffect(() => {
    if (!character) {
      return;
    }

    const matchedBackground = availableBackgrounds.find(
      option => option.name.toLowerCase() === character.background?.name?.toLowerCase()
    );
    setSelectedBackgroundId(matchedBackground?.id ?? null);
  }, [availableBackgrounds, character]);
  
  // Handle name save
  const handleSaveName = async () => {
    if (!character?.id || isSaving) return;
    
    setIsSaving(true);
    try {
      await CharacterAPI.updateCharacter(character.id, {
        first_name: firstName.trim(),
        last_name: lastName.trim()
      });
      
      // Update local state smoothly
      if (updateCharacterPartial) {
        // Construct the new name properly based on input
        const newFullName = lastName.trim() ? `${firstName.trim()} ${lastName.trim()}` : firstName.trim();
        updateCharacterPartial({ 
            name: newFullName,
            first_name: firstName.trim(),
            last_name: lastName.trim()
        });
      }
      
      setIsEditingName(false);
    } catch (error) {
      handleError(error);
    } finally {
      setIsSaving(false);
    }
  };
  
  // Handle cancel edit
  const handleCancelEdit = () => {
    if (character && character.name) {
      const parts = character.name.split(' ');
      setFirstName(parts[0] || '');
      setLastName(parts.slice(1).join(' ') || '');
    }
    setIsEditingName(false);
  };

  const openIdentityEditor = () => {
    if (!character) {
      return;
    }

    setAgeInput(String(character.age ?? 0));
    setSelectedGenderId(getCharacterGenderId(character.gender));
    setActiveOverviewEditor('identity');
  };

  const openAlignmentEditor = () => {
    setAlignmentDraft(character?.alignment_values ?? { law_chaos: 50, good_evil: 50 });
    setActiveOverviewEditor('alignment');
  };

  const openProgressEditor = () => {
    if (!character) {
      return;
    }

    setExperienceInput(String(character.experience ?? 0));
    setGoldInput(String(character.gold ?? 0));
    setActiveOverviewEditor('progress');
  };

  const openBackgroundEditor = () => {
    if (!character) {
      return;
    }

    const matchedBackground = availableBackgrounds.find(
      option => option.name.toLowerCase() === character.background?.name?.toLowerCase()
    );
    setSelectedBackgroundId(matchedBackground?.id ?? null);
    setActiveOverviewEditor('background');
  };

  const handleCloseOverviewEditor = () => {
    if (character) {
      setAgeInput(String(character.age ?? 0));
      setExperienceInput(String(character.experience ?? 0));
      setGoldInput(String(character.gold ?? 0));
      setAlignmentDraft(character.alignment_values ?? { law_chaos: 50, good_evil: 50 });
      setSelectedGenderId(getCharacterGenderId(character.gender));
      const matchedBackground = availableBackgrounds.find(
        option => option.name.toLowerCase() === character.background?.name?.toLowerCase()
      );
      setSelectedBackgroundId(matchedBackground?.id ?? null);
    }

    setActiveOverviewEditor(null);
  };

  const handleSaveIdentity = async () => {
    if (!character?.id || isSaving) return;

    const parsedAge = Number.parseInt(ageInput, 10);
    if (Number.isNaN(parsedAge) || parsedAge < 0) {
      handleError(new Error('Age must be a non-negative whole number.'));
      return;
    }

    const currentGenderId = getCharacterGenderId(character.gender);
    const genderChanged = selectedGenderId !== null && selectedGenderId !== currentGenderId;
    const ageChanged = parsedAge !== character.age;

    if (!ageChanged && !genderChanged) {
      setActiveOverviewEditor(null);
      return;
    }

    setIsSaving(true);
    try {
      if (ageChanged) {
        await CharacterAPI.updateCharacter(character.id, { age: parsedAge });
      }

      if (genderChanged && selectedGenderId !== null) {
        await CharacterAPI.setGender(character.id, selectedGenderId);
      }

      const refreshed = await CharacterAPI.getCharacterState(character.id);
      updateCharacterPartial(refreshed);
      setActiveOverviewEditor(null);
    } catch (error) {
      handleError(error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveAlignment = async () => {
    if (!character?.id || isSaving) return;

    const currentAlignment = character.alignment_values ?? { law_chaos: 50, good_evil: 50 };
    const alignmentChanged =
      alignmentDraft.law_chaos !== currentAlignment.law_chaos ||
      alignmentDraft.good_evil !== currentAlignment.good_evil;

    if (!alignmentChanged) {
      setActiveOverviewEditor(null);
      return;
    }

    setIsSaving(true);
    try {
      const refreshed = await CharacterAPI.updateCharacter(character.id, {
        alignment: [alignmentDraft.law_chaos, alignmentDraft.good_evil]
      });
      updateCharacterPartial(refreshed);
      setActiveOverviewEditor(null);
    } catch (error) {
      handleError(error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveProgress = async () => {
    if (!character?.id || isSaving) return;

    const parsedExperience = Number.parseInt(experienceInput, 10);
    const parsedGold = Number.parseInt(goldInput, 10);

    if (Number.isNaN(parsedExperience) || parsedExperience < 0) {
      handleError(new Error('Experience must be a non-negative whole number.'));
      return;
    }

    if (Number.isNaN(parsedGold) || parsedGold < 0) {
      handleError(new Error('Gold must be a non-negative whole number.'));
      return;
    }

    const experienceChanged = parsedExperience !== character.experience;
    const goldChanged = parsedGold !== character.gold;

    if (!experienceChanged && !goldChanged) {
      setActiveOverviewEditor(null);
      return;
    }

    setIsSaving(true);
    try {
      if (experienceChanged) {
        await CharacterAPI.updateCharacter(character.id, { experience: parsedExperience });
      }

      if (goldChanged) {
        await inventoryAPI.updateGold(character.id, parsedGold);
      }

      const refreshed = await CharacterAPI.getCharacterState(character.id);
      updateCharacterPartial(refreshed);
      setActiveOverviewEditor(null);
    } catch (error) {
      handleError(error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveBackground = async () => {
    if (!character?.id || isSaving) return;

    setIsSaving(true);
    try {
      const refreshed = await CharacterAPI.setBackground(character.id, selectedBackgroundId);
      updateCharacterPartial(refreshed);
      await invalidateSubsystems([...BACKGROUND_DEPENDENT_SUBSYSTEMS]);
      setActiveOverviewEditor(null);
    } catch (error) {
      handleError(error);
    } finally {
      setIsSaving(false);
    }
  };
  
  const handleSelectRace = async (raceId: number, raceName: string, subrace: string | null) => {
    if (!character?.id) return;
    try {
      await CharacterAPI.changeRace(character.id, raceId, subrace);
      updateCharacterPartial({ race: raceName, subrace: subrace ?? undefined });
      await invalidateSubsystems([...RACE_DEPENDENT_SUBSYSTEMS]);
      setIsRaceModalOpen(false);
    } catch (e) {
      console.error('Failed to change race:', e);
    }
  };

  const handleSelectDeity = async (selectedDeityName: string) => {
    if (!character?.id || isSaving) return;
    
    setIsSaving(true);
    try {
      await CharacterAPI.setDeity(character.id, selectedDeityName.trim());
      
      // Update local state immediately
      setDeity(selectedDeityName.trim());
      
      // Update context smoothly
      if (updateCharacterPartial) {
        updateCharacterPartial({ deity: selectedDeityName.trim() });
      }
      setIsDeityModalOpen(false);
    } catch (error) {
      handleError(error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveBiography = async () => {
    if (!character?.id || isSaving) return;
    
    setIsSaving(true);
    try {
      await CharacterAPI.setBiography(character.id, biography.trim());
      
      // Update context smoothly
      if (updateCharacterPartial) {
        updateCharacterPartial({ biography: biography.trim() });
      }
      setIsEditingBio(false);
    } catch (error) {
      handleError(error);
    } finally {
      setIsSaving(false);
    }
  };
  
  // Refetch data on mount to ensure calculated data is completely fresh
  // after edits in other tabs
  useEffect(() => {
    let mounted = true;
    if (character?.id) {
      // Force refresh data silently
      abilities.load({ silent: true, force: true }).catch(console.warn);
      combat.load({ silent: true, force: true }).catch(console.warn);
      skills.load({ silent: true, force: true }).catch(console.warn);
      feats.load({ silent: true, force: true }).catch(console.warn);
      spells.load({ silent: true, force: true }).catch(console.warn);
      saves.load({ silent: true, force: true }).catch(console.warn);
      classes.load({ silent: true, force: true }).catch(console.warn);

      // Also refresh character data to get updated XP, Level, Gold etc.
      CharacterAPI.getCharacterState(character.id)
        .then(data => {
            if (mounted && updateCharacterPartial) {
                updateCharacterPartial(data);
            }
        })
        .catch(console.warn);
    }
    return () => { mounted = false; };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [character?.id]);

  useEffect(() => {
    let mounted = true;

    if (character?.id) {
      CharacterAPI.getRaceData(character.id)
        .then(data => {
          if (mounted) {
            setRaceData(data);
          }
        })
        .catch(error => {
          console.warn('Failed to load race data for overview:', error);
          if (mounted) {
            setRaceData(null);
          }
        });
    } else {
      setRaceData(null);
    }

    return () => {
      mounted = false;
    };
  }, [character?.id, character?.race, character?.subrace]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[rgb(var(--color-primary))]"></div>
      </div>
    );
  }

  if (error) {
    return (
      <Card variant="error">
        <p className="text-error">{error}</p>
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

  const availableSkillPoints =
    skills.data?.available_points ??
    classes.data?.skill_points_summary?.available_points ??
    character.availableSkillPoints ??
    0;
  const totalSkillPoints =
    classes.data?.skill_points_summary?.total_points ??
    character.totalSkillPoints ??
    skills.data?.spent_points ??
    0;
  const totalFeats =
    feats.data?.summary?.total_count ??
    character.totalFeats ??
    0;
  const hasSpellcasting =
    (spells.data?.spellcasting_classes?.length ?? 0) > 0 ||
    character.knownSpells !== undefined;
  const knownSpellsCount =
    spells.data?.known_spells?.length ??
    character.knownSpells ??
    0;
  const baseAttackBonus =
    combat.data?.base_attack_bonus ??
    character.baseAttackBonus ??
    classes.data?.entries?.reduce((sum, entry) => sum + entry.base_attack_bonus, 0) ??
    0;
  const armorClassTotal = combat.data?.armor_class?.total;
  const movementSpeed =
    combat.data?.movement?.current ??
    raceData?.base_speed ??
    character.movementSpeed;
  const initiativeTotal =
    combat.data?.initiative?.total ??
    character.initiative;
  const sizeLabel =
    raceData?.size_name ??
    character.size;
  const abilityOverviewEntries = abilities.data?.effective_scores
    ? ([
        ['Str', abilities.data.effective_scores.Str, abilities.data.modifiers?.Str ?? 0],
        ['Dex', abilities.data.effective_scores.Dex, abilities.data.modifiers?.Dex ?? 0],
        ['Con', abilities.data.effective_scores.Con, abilities.data.modifiers?.Con ?? 0],
        ['Int', abilities.data.effective_scores.Int, abilities.data.modifiers?.Int ?? 0],
        ['Wis', abilities.data.effective_scores.Wis, abilities.data.modifiers?.Wis ?? 0],
        ['Cha', abilities.data.effective_scores.Cha, abilities.data.modifiers?.Cha ?? 0],
      ] as [string, number, number][])
    : null;
  const formatDerivedModifier = (value: number | null | undefined): string =>
    value == null ? '-' : formatModifier(value);

  return (
    <div className="space-y-6">
      {/* Main Character Card - Hero Section */}
      <div className="relative overflow-hidden rounded-xl bg-gradient-to-br from-[rgb(var(--color-surface-2))] to-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border)/0.5)]">
        {/* Background Pattern */}
        <div className="absolute inset-0 opacity-5">
          <div className="absolute top-0 right-0 w-96 h-96 bg-gradient-to-bl from-[rgb(var(--color-primary))] to-transparent rounded-full blur-3xl" />
          <div className="absolute bottom-0 left-0 w-64 h-64 bg-gradient-to-tr from-[rgb(var(--color-secondary))] to-transparent rounded-full blur-3xl" />
        </div>
        
        <div className="relative p-6 md:p-8">
            {/* Top Section: Identity & Bio */}
            <div className="space-y-6">
                {/* Header Row: Name & Edit */}
                {isEditingName ? (
                  <div className="flex items-center gap-2 mb-4">
                    <input
                      type="text"
                      value={firstName}
                      onChange={(e) => setFirstName(e.target.value)}
                      placeholder="First Name"
                      className="px-3 py-2 text-2xl font-bold bg-[rgb(var(--color-surface-2))] border border-[rgb(var(--color-surface-border))] rounded-lg text-[rgb(var(--color-text-primary))] focus:outline-none focus:border-[rgb(var(--color-primary))]"
                      disabled={isSaving}
                    />
                    <input
                      type="text"
                      value={lastName}
                      onChange={(e) => setLastName(e.target.value)}
                      placeholder="Last Name"
                      className="px-3 py-2 text-2xl font-bold bg-[rgb(var(--color-surface-2))] border border-[rgb(var(--color-surface-border))] rounded-lg text-[rgb(var(--color-text-primary))] focus:outline-none focus:border-[rgb(var(--color-primary))]"
                      disabled={isSaving}
                    />
                    <Button
                      type="button"
                      onClick={handleSaveName}
                      disabled={isSaving}
                      variant="primary"
                      size="sm"
                    >
                      {isSaving ? (
                        <svg className="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                        </svg>
                      ) : (
                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                      )}
                    </Button>
                    <Button
                      type="button"
                      onClick={handleCancelEdit}
                      disabled={isSaving}
                      variant="outline"
                      size="sm"
                    >
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </Button>
                  </div>
                ) : (
                  <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
                    <div className="flex items-center gap-2">
                        <h1 className="text-4xl font-bold text-[rgb(var(--color-text-primary))]">{display(character.name, 'Unknown Character')}</h1>
                        <Button
                        type="button"
                        onClick={() => setIsEditingName(true)}
                        variant="ghost"
                        size="icon"
                        className="text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] transition-colors"
                        title="Edit character name"
                        >
                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                        </svg>
                        </Button>
                    </div>
                    {/* Classes Pills */}
                     <div className="flex flex-wrap items-center gap-2">
                      {(character.classes || []).map((cls, idx) => (
                        <span key={idx} className="px-3 py-1.5 bg-gradient-to-r from-[rgb(var(--color-primary)/0.2)] to-[rgb(var(--color-primary)/0.1)] border border-[rgb(var(--color-primary)/0.3)] shadow-[0_0_10px_rgba(var(--color-primary),0.1)] rounded-lg text-sm font-bold text-[rgb(var(--color-primary-light))]">
                          {cls.name} {cls.level}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
                
                {/* Structured Bio Grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-4 pt-4 border-t border-[rgb(var(--color-surface-border)/0.4)]">
                    
                    {/* Basic Info Row */}
                    <div className="col-span-1 md:col-span-2 grid grid-cols-2 md:grid-cols-4 gap-4 pb-4 border-b border-[rgb(var(--color-surface-border)/0.2)]">
                         <div>
                            <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Race & Origin</div>
                            <div className="flex items-center gap-2">
                              <span className="font-medium text-[rgb(var(--color-text-primary))] truncate" title={character.subrace ? `${character.race} (${character.subrace})` : character.race}>
                                {character.race}{character.subrace ? <span className="text-[rgb(var(--color-text-secondary))] text-sm ml-1">({character.subrace})</span> : null}
                              </span>
                              <Button
                                type="button"
                                onClick={() => setIsRaceModalOpen(true)}
                                variant="ghost"
                                size="sm"
                                className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
                                disabled={isSaving}
                                title="Change Race"
                              >
                                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                                </svg>
                              </Button>
                            </div>
                         </div>
	                         <div>
	                            <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Gender & Age</div>
	                            <div className="flex items-center gap-2">
	                              <div className="font-medium text-[rgb(var(--color-text-primary))]">
	                                  {display(character.gender)} <span className="text-[rgb(var(--color-text-muted))] text-sm">/ {character.age || '?'} yrs</span>
	                              </div>
	                              <Button
	                                type="button"
	                                onClick={openIdentityEditor}
	                                variant="ghost"
	                                size="sm"
	                                className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
	                                disabled={isSaving}
	                                title="Edit Gender & Age"
	                              >
	                                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
	                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
	                                </svg>
	                              </Button>
	                            </div>
	                         </div>
	                         <div>
	                            <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Alignment</div>
	                             <div className="flex items-center gap-2">
	                               <div className="font-medium text-[rgb(var(--color-text-primary))]">{display(character.alignment)}</div>
	                               <Button
	                                  type="button"
	                                  onClick={openAlignmentEditor}
	                                  variant="ghost"
	                                  size="sm"
	                                  className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
	                                  disabled={isSaving}
	                                  title="Edit Alignment"
	                               >
	                                 <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
	                                   <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
	                                 </svg>
	                               </Button>
	                             </div>
	                         </div>
                         <div>
                             <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Deity</div>
                               <div className="flex items-center gap-2">
                                 <span className="font-medium text-[rgb(var(--color-text-primary))] truncate max-w-[120px]" title={character.deity || 'None'}>{character.deity || '-'}</span>
                                 <Button 
                                    type="button"
                                    onClick={() => setIsDeityModalOpen(true)} 
                                    variant="ghost" 
                                    size="sm" 
                                    className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
                                    disabled={isSaving}
                                    title="Change Deity"
                                 >
                                   <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                     <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                                   </svg>
                                 </Button>
                               </div>
                         </div>

                         {/* Row 2: Core Stats (HP, AC, XP, Gold) */}
                         <div>
                            <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Hit Points</div>
                            <div className="font-medium text-[rgb(var(--color-text-primary))]">
                                {abilities.isLoading ? <span className="text-xs">...</span> : (
                                    <>
                                        {display(abilities.data?.hit_points?.current ?? character.hitPoints ?? 0)}
                                        <span className="text-[rgb(var(--color-text-muted))]">/{display(abilities.data?.hit_points?.max ?? character.maxHitPoints ?? 0)}</span>
                                    </>
                                )}
                            </div>
                         </div>
                         <div>
                            <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Armor Class</div>
                            <div className="font-medium text-[rgb(var(--color-text-primary))]">
                                {combat.isLoading ? <span className="text-xs">...</span> : display(combat.data?.armor_class?.total)}
                            </div>
                         </div>
	                         <div>
	                            <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Experience</div>
	                            <div className="flex items-center gap-2">
	                              <div className="font-medium text-[rgb(var(--color-text-primary))]">{formatNumber(character.experience)}</div>
	                              <Button
	                                type="button"
	                                onClick={openProgressEditor}
	                                variant="ghost"
	                                size="sm"
	                                className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
	                                disabled={isSaving}
	                                title="Edit Experience & Gold"
	                              >
	                                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
	                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
	                                </svg>
	                              </Button>
	                            </div>
	                         </div>
	                         <div>
	                            <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Gold</div>
	                            <div className="flex items-center gap-2">
	                              <div className="font-medium text-[rgb(var(--color-warning))]">{formatNumber(character.gold || 0)}</div>
	                              <Button
	                                type="button"
	                                onClick={openProgressEditor}
	                                variant="ghost"
	                                size="sm"
	                                className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
	                                disabled={isSaving}
	                                title="Edit Experience & Gold"
	                              >
	                                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
	                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
	                                </svg>
	                              </Button>
	                            </div>
	                         </div>
	                    </div>

	                    {activeOverviewEditor === 'identity' && (
	                      <div className="col-span-1 md:col-span-2 rounded-lg border border-[rgb(var(--color-surface-border)/0.6)] bg-[rgb(var(--color-surface-2)/0.65)] p-4 space-y-4">
	                        <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase font-semibold">Edit Gender & Age</div>
	                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
	                          <div className="space-y-2">
	                            <span className="text-sm text-[rgb(var(--color-text-secondary))]">Gender</span>
	                            <div className="grid grid-cols-2 gap-2">
	                              {availableGenders.map(option => {
	                                const isSelected = selectedGenderId === option.id;
	                                return (
	                                  <Button
	                                    key={option.id}
	                                    type="button"
	                                    variant={isSelected ? 'primary' : 'outline'}
	                                    size="sm"
	                                    onClick={() => setSelectedGenderId(option.id)}
	                                    disabled={isSaving}
	                                    aria-pressed={isSelected}
	                                    className="justify-center"
	                                  >
	                                    {option.name}
	                                  </Button>
	                                );
	                              })}
	                            </div>
	                          </div>
	                          <label className="space-y-2">
	                            <span className="text-sm text-[rgb(var(--color-text-secondary))]">Age</span>
	                            <input
	                              type="number"
	                              min={0}
	                              step={1}
	                              value={ageInput}
	                              onChange={(e) => setAgeInput(e.target.value)}
	                              className="w-full rounded-lg border border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] px-3 py-2 text-sm text-[rgb(var(--color-text-primary))] focus:outline-none focus:ring-2 focus:ring-[rgb(var(--color-primary))]"
	                              disabled={isSaving}
	                            />
	                          </label>
	                        </div>
	                        <div className="flex gap-2">
	                          <Button type="button" onClick={handleSaveIdentity} disabled={isSaving} variant="primary" size="sm">
	                            {isSaving ? 'Saving...' : 'Save'}
	                          </Button>
	                          <Button type="button" onClick={handleCloseOverviewEditor} disabled={isSaving} variant="outline" size="sm">
	                            Cancel
	                          </Button>
	                        </div>
	                      </div>
	                    )}

	                    {activeOverviewEditor === 'alignment' && (
	                      <div className="col-span-1 md:col-span-2 rounded-lg border border-[rgb(var(--color-surface-border)/0.6)] bg-[rgb(var(--color-surface-2)/0.65)] p-4 space-y-4">
	                        <div>
	                          <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase font-semibold mb-1">Edit Alignment</div>
	                          <div className="text-sm text-[rgb(var(--color-text-secondary))]">{display(character.alignment)}</div>
	                        </div>
	                        <AlignmentGrid
	                          currentAlignment={alignmentDraft}
	                          onAlignmentSelect={(law_chaos, good_evil) => setAlignmentDraft({ law_chaos, good_evil })}
	                        />
	                        <div className="flex gap-2">
	                          <Button type="button" onClick={handleSaveAlignment} disabled={isSaving} variant="primary" size="sm">
	                            {isSaving ? 'Saving...' : 'Save'}
	                          </Button>
	                          <Button type="button" onClick={handleCloseOverviewEditor} disabled={isSaving} variant="outline" size="sm">
	                            Cancel
	                          </Button>
	                        </div>
	                      </div>
	                    )}

	                    {activeOverviewEditor === 'progress' && (
	                      <div className="col-span-1 md:col-span-2 rounded-lg border border-[rgb(var(--color-surface-border)/0.6)] bg-[rgb(var(--color-surface-2)/0.65)] p-4 space-y-4">
	                        <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase font-semibold">Edit Experience & Gold</div>
	                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
	                          <label className="space-y-2">
	                            <span className="text-sm text-[rgb(var(--color-text-secondary))]">Experience</span>
	                            <input
	                              type="number"
	                              min={0}
	                              step={1}
	                              value={experienceInput}
	                              onChange={(e) => setExperienceInput(e.target.value)}
	                              className="w-full rounded-lg border border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] px-3 py-2 text-sm text-[rgb(var(--color-text-primary))] focus:outline-none focus:ring-2 focus:ring-[rgb(var(--color-primary))]"
	                              disabled={isSaving}
	                            />
	                          </label>
	                          <label className="space-y-2">
	                            <span className="text-sm text-[rgb(var(--color-text-secondary))]">Gold</span>
	                            <input
	                              type="number"
	                              min={0}
	                              step={1}
	                              value={goldInput}
	                              onChange={(e) => setGoldInput(e.target.value)}
	                              className="w-full rounded-lg border border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] px-3 py-2 text-sm text-[rgb(var(--color-text-primary))] focus:outline-none focus:ring-2 focus:ring-[rgb(var(--color-primary))]"
	                              disabled={isSaving}
	                            />
	                          </label>
	                        </div>
	                        <div className="flex gap-2">
	                          <Button type="button" onClick={handleSaveProgress} disabled={isSaving} variant="primary" size="sm">
	                            {isSaving ? 'Saving...' : 'Save'}
	                          </Button>
	                          <Button type="button" onClick={handleCloseOverviewEditor} disabled={isSaving} variant="outline" size="sm">
	                            Cancel
	                          </Button>
	                        </div>
	                      </div>
	                    )}

	                    {activeOverviewEditor === 'background' && (
	                      <div className="col-span-1 md:col-span-2 rounded-lg border border-[rgb(var(--color-surface-border)/0.6)] bg-[rgb(var(--color-surface-2)/0.65)] p-4 space-y-4">
	                        <div>
	                          <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase font-semibold mb-1">Edit Background</div>
	                          <div className="text-sm text-[rgb(var(--color-text-secondary))]">
	                            {character.background?.name ?? 'None'}
	                          </div>
	                        </div>
	                        <div className="grid gap-2 max-h-72 overflow-y-auto pr-1">
	                          <button
	                            type="button"
	                            onClick={() => setSelectedBackgroundId(null)}
	                            disabled={isSaving}
	                            className={`w-full rounded-lg border px-3 py-3 text-left transition-colors ${
	                              selectedBackgroundId === null
	                                ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.12)]'
	                                : 'border-[rgb(var(--color-surface-border)/0.7)] bg-[rgb(var(--color-surface-1))] hover:border-[rgb(var(--color-primary)/0.45)]'
	                            }`}
	                          >
	                            <div className="font-medium text-[rgb(var(--color-text-primary))]">None</div>
	                            <div className="mt-2 text-sm text-[rgb(var(--color-text-secondary))]">Remove the current background feat.</div>
	                          </button>
	                          {availableBackgrounds.map(option => {
	                            const isSelected = selectedBackgroundId === option.id;
	                            const isUnavailable = option.can_take === false && !isSelected;
	                            return (
	                              <button
	                                key={option.id}
	                                type="button"
	                                onClick={() => setSelectedBackgroundId(option.id)}
	                                disabled={isSaving || isUnavailable}
	                                className={`w-full rounded-lg border px-3 py-3 text-left transition-colors ${
	                                  isSelected
	                                    ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.12)]'
	                                    : isUnavailable
	                                      ? 'border-[rgb(var(--color-surface-border)/0.45)] bg-[rgb(var(--color-surface-1)/0.6)] opacity-60 cursor-not-allowed'
	                                      : 'border-[rgb(var(--color-surface-border)/0.7)] bg-[rgb(var(--color-surface-1))] hover:border-[rgb(var(--color-primary)/0.45)]'
	                                }`}
	                              >
	                                <div className="font-medium text-[rgb(var(--color-text-primary))]">{option.name}</div>
	                                <BackgroundDescriptionSections
	                                  description={option.description}
	                                  missingRequirements={option.can_take === false ? option.missing_requirements : undefined}
	                                />
	                              </button>
	                            );
	                          })}
	                        </div>
	                        <div className="flex gap-2">
	                          <Button type="button" onClick={handleSaveBackground} disabled={isSaving} variant="primary" size="sm">
	                            {isSaving ? 'Saving...' : 'Save'}
	                          </Button>
	                          <Button type="button" onClick={handleCloseOverviewEditor} disabled={isSaving} variant="outline" size="sm">
	                            Cancel
	                          </Button>
	                        </div>
	                      </div>
	                    )}


	                    {/* Background */}
	                    <div className="flex flex-col gap-1">
	                        <div className="flex items-center gap-2">
	                          <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase font-semibold mb-1">
	                               Background
	                          </div>
	                          <Button
	                            type="button"
	                            onClick={openBackgroundEditor}
	                            variant="ghost"
	                            size="sm"
	                            className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
	                            disabled={isSaving}
	                            title="Edit Background"
	                          >
	                            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
	                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
	                            </svg>
	                          </Button>
	                        </div>
	                        <div className="flex items-center">
	                            {character.background ? (
                                <span className="inline-flex items-center px-3 py-1.5 rounded-md bg-[rgb(var(--color-surface-3))] border border-[rgb(var(--color-surface-border)/0.5)] text-[rgb(var(--color-text-primary))] text-sm font-medium">
                                    {character.background.name}
                                </span>
                            ) : (
                                <span className="text-[rgb(var(--color-text-muted))] text-sm italic">None</span>
                            )}
                        </div>
                    </div>

                    {/* Domains */}
                     <div className="flex flex-col gap-1">
                        <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase font-semibold mb-1">
                            Domains
                        </div>
                        <div className="flex flex-wrap gap-2">
                            {character.domains && character.domains.length > 0 ? (
                                character.domains.map((domain, idx) => (
                                    <span key={idx} className="inline-flex items-center px-3 py-1.5 rounded-md bg-gradient-to-br from-[rgb(var(--color-secondary)/0.15)] to-[rgb(var(--color-secondary)/0.05)] border border-[rgb(var(--color-secondary)/0.2)] text-[rgb(var(--color-text-primary))] text-sm font-medium">
                                        {domain.name}
                                    </span>
                                ))
                            ) : (
                                <span className="text-[rgb(var(--color-text-muted))] text-sm italic">None</span>
                            )}
                        </div>
                    </div>
                    
                    {/* Biography */}
                    <div className="col-span-1 md:col-span-2 flex flex-col gap-1">
                        <div className="flex items-center justify-between">
                          <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase font-semibold">Biography</div>
                          {!isEditingBio && (

                            <Button 
                                type="button"
                                onClick={() => setIsEditingBio(true)} 
                                variant="ghost" 
                                size="sm" 
                                className="h-6 w-6 text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-primary))] p-0 transition-colors"
                                title="Edit Biography"
                            >
                              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                              </svg>
                            </Button>
                          )}
                        </div>
                        {isEditingBio ? (
                          <div className="space-y-2">
                            <textarea
                              value={biography}
                              onChange={(e) => setBiography(e.target.value)}
                              placeholder="Enter character biography..."
                              className="w-full px-3 py-2 text-sm bg-[rgb(var(--color-surface-2))] border border-[rgb(var(--color-surface-border))] rounded-lg text-[rgb(var(--color-text-primary))] focus:outline-none focus:border-[rgb(var(--color-primary))] min-h-[100px] resize-y"
                              disabled={isSaving}
                            />
                            <div className="flex gap-2">
                              <Button type="button" onClick={handleSaveBiography} disabled={isSaving} variant="primary" size="sm">
                                {isSaving ? 'Saving...' : 'Save'}
                              </Button>
                              <Button type="button" onClick={() => { setBiography(character.biography || ''); setIsEditingBio(false); }} disabled={isSaving} variant="outline" size="sm">
                                Cancel
                              </Button>
                            </div>
                          </div>
                        ) : (
                          <div className="text-sm text-[rgb(var(--color-text-secondary))]">
                            {character.biography ? (
                              <p className="whitespace-pre-wrap">{character.biography}</p>
                            ) : (
                              <span className="text-[rgb(var(--color-text-muted))] italic">No biography written</span>
                            )}
                          </div>
                        )}
                    </div>
                </div>
              </div>
      </div>
      </div>

      {/* Progressive Disclosure Sections */}
      <div className="space-y-4">
        
        {/* Core Abilities - Always important for RPG */}
        <CollapsibleSection
          title={t('navigation.abilityScores')}
          defaultOpen={true}
          badge={abilities.data?.modifiers ? formatModifier(abilities.data.modifiers.Str + abilities.data.modifiers.Dex + abilities.data.modifiers.Con + abilities.data.modifiers.Int + abilities.data.modifiers.Wis + abilities.data.modifiers.Cha) : '-'}
        >
          {abilities.isLoading && !abilityOverviewEntries ? (
            <div className="text-sm text-[rgb(var(--color-text-muted))]">Loading ability scores...</div>
          ) : abilityOverviewEntries ? (
            <div className="grid grid-cols-3 md:grid-cols-6 gap-y-6 gap-x-4">
              {abilityOverviewEntries.map(([key, value, modifier]) => {
                const modifierColor = modifier > 0 ? 'var(--color-success)' : modifier < 0 ? 'var(--color-error)' : 'var(--color-text-muted)';
                return (
                  <div key={key} className="">
                    <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase tracking-wider mb-1">{
                      key === 'Str' ? t('abilityScores.strength') :
                      key === 'Dex' ? t('abilityScores.dexterity') :
                      key === 'Con' ? t('abilityScores.constitution') :
                      key === 'Int' ? t('abilityScores.intelligence') :
                      key === 'Wis' ? t('abilityScores.wisdom') :
                      key === 'Cha' ? t('abilityScores.charisma') :
                      key
                    }</div>
                    <div className="text-2xl font-bold text-[rgb(var(--color-text-primary))] leading-none mb-1">{value}</div>
                    <div className={`text-sm font-medium text-[rgb(${modifierColor})]`}>
                      {modifier >= 0 ? '+' : ''}{modifier}
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="text-sm text-[rgb(var(--color-text-muted))]">Ability data unavailable.</div>
          )}
        </CollapsibleSection>


        {/* Combat & Progression */}
        <CollapsibleSection
          title="Combat & Progression"
          defaultOpen={false}
          badge={
            (skills.data?.overdrawn_points ?? 0) > 0
              ? `${skills.data?.overdrawn_points} overdrawn`
              : availableSkillPoints > 0
                ? `${availableSkillPoints} skill points`
                : armorClassTotal != null
                  ? `AC ${armorClassTotal}`
                  : 'Combat'
          }
        >
          <div className="space-y-6">
            {/* Combat Stats */}
            <div>
              <div className="grid grid-cols-3 gap-4 mb-4">
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.baseAttackBonus')}</div>
                  <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                    {combat.isLoading ? (
                      <span className="text-sm">...</span>
                    ) : (
                      formatModifier(baseAttackBonus)
                    )}
                  </div>
                </div>
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.meleeAttack')}</div>
                  <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">{formatDerivedModifier(combat.data?.attack_bonuses?.melee)}</div>
                </div>
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.rangedAttack')}</div>
                  <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">{formatDerivedModifier(combat.data?.attack_bonuses?.ranged)}</div>
                </div>
              </div>
              <div className="grid grid-cols-3 gap-4">
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('abilityScores.fortitude')}</div>
                  <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">
                    {saves.isLoading ? (
                      <span className="text-sm">...</span>
                    ) : (
                      formatDerivedModifier(saves.data?.saves?.fortitude?.total)
                    )}
                  </div>
                </div>
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('abilityScores.reflex')}</div>
                  <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">
                    {saves.isLoading ? (
                      <span className="text-sm">...</span>
                    ) : (
                      formatDerivedModifier(saves.data?.saves?.reflex?.total)
                    )}
                  </div>
                </div>
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('abilityScores.will')}</div>
                  <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">
                    {saves.isLoading ? (
                      <span className="text-sm">...</span>
                    ) : (
                      formatDerivedModifier(saves.data?.saves?.will?.total)
                    )}
                  </div>
                </div>
              </div>
            </div>

            {/* Character Development */}
            <div className="pt-4 border-t border-[rgb(var(--color-surface-border)/0.6)]">
              <div className="grid grid-cols-3 gap-4 mb-4">
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Total Skill Points</div>
                  <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">{display(totalSkillPoints)}</div>
                </div>
                <div className="">
                  <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.totalFeats')}</div>
                  <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">{display(totalFeats)}</div>
                </div>
                {hasSpellcasting && (
                  <div className="">
                    <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.knownSpells')}</div>
                    <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">{display(knownSpellsCount)}</div>
                  </div>
                )}
              </div>

              {/* Movement & Physical Stats */}
              <div className="pt-4 border-t border-[rgb(var(--color-surface-border)/0.6)]">
                <div className="grid grid-cols-3 gap-4">
                  <div className="">
                    <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.speed')}</div>
                    <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">
                      {movementSpeed !== undefined ? `${display(movementSpeed)} ft` : display(undefined)}
                    </div>
                  </div>
                  <div className="">
                    <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.initiative')}</div>
                    <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">
                      {formatModifier(initiativeTotal)}
                    </div>
                  </div>
                  <div className="">
                    <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">{t('character.size')}</div>
                    <div className="text-lg font-bold text-[rgb(var(--color-text-primary))]">{display(sizeLabel)}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CollapsibleSection>


        {/* Advanced/Rare Data - Hidden by default */}
        {(character.damageResistances || character.damageImmunities || character.spellResistance) && (
          <CollapsibleSection 
            title="Special Defenses" 
            defaultOpen={false}
            badge={character.spellResistance ? `SR ${character.spellResistance}` : "Resistances"}
          >
            <div className="space-y-3">
              {character.spellResistance && (
                <div className="flex justify-between">
                  <span className="text-[rgb(var(--color-text-secondary))]">{t('character.spellResistance')}</span>
                  <span className="font-medium text-[rgb(var(--color-text-primary))]">{character.spellResistance}</span>
                </div>
              )}
              {character.damageResistances && character.damageResistances.length > 0 && (
                <div>
                  <div className="space-y-1">
                    {character.damageResistances.map((res, idx) => (
                      <div key={idx} className="flex justify-between text-sm">
                        <span className="text-[rgb(var(--color-text-secondary))]">{res.type}</span>
                        <span className="font-medium text-[rgb(var(--color-text-primary))]">{res.amount}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {character.damageImmunities && character.damageImmunities.length > 0 && (
                <div>
                  <div className="flex flex-wrap gap-1">
                    {character.damageImmunities.map((immunity, idx) => (
                      <span key={idx} className="px-2 py-1 bg-[rgb(var(--color-surface-1)/0.5)] backdrop-blur rounded text-xs text-[rgb(var(--color-text-secondary))]">
                        {immunity}
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </CollapsibleSection>
        )}

        {/* Enhanced Campaign Overview */}
        <CampaignOverview character={character} />
      </div>

      {/* Deity Selection Modal */}
      <DeitySelectionModal
        isOpen={isDeityModalOpen}
        onClose={() => setIsDeityModalOpen(false)}
        onSelectDeity={handleSelectDeity}
        characterId={character?.id || 0}
        currentDeity={character.deity}
      />

      <RaceSelectionModal
        isOpen={isRaceModalOpen}
        onClose={() => setIsRaceModalOpen(false)}
        onSelectRace={handleSelectRace}
        currentRaceName={character.race}
        currentSubrace={character.subrace}
      />
    </div>
  );
}
