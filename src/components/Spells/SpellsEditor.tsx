
import { useState, useMemo, useEffect, useCallback, useRef } from 'react';
import { Card } from '@/components/ui/Card';
import { AlertCircle, Info, ChevronDown, ChevronUp, Zap } from 'lucide-react';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { useSpellManagement } from '@/hooks/useSpellManagement';
import { useSpellSearch } from '@/hooks/useSpellSearch';
import { mapKnownSpellsToSpellInfo, groupMemorizedSpells, mapCasterClasses, filterSpells, sortSpells } from '@/utils/spellUtils';
import { SpellNavBar, type SpellTab } from './SpellNavBar';
import { SpellTabContent } from './SpellTabContent';
import type { SpellInfo, SpellsState } from './types';
import { useToast } from '@/contexts/ToastContext';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { useTranslations } from '@/hooks/useTranslations';
import { Badge } from '@/components/ui/Badge';
import NWN2Icon from '@/components/ui/NWN2Icon';
import { display } from '@/utils/dataHelpers';
import { stripNwn2Tags } from '@/utils/nwn2Markup';

export default function SpellsEditor() {
  const {
    character,
    isLoading: characterLoading,
    error: characterError,
    totalSpells,
    setTotalSpells
  } = useCharacterContext();
  const spells = useSubsystem('spells');
  const { addSpell, removeSpell } = useSpellManagement();
  const { showToast } = useToast();
  const { handleError } = useErrorHandler();
  const t = useTranslations();

  const [activeTab, setActiveTab] = useState<SpellTab>('my-spells');
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState('name');
  const [selectedClasses, setSelectedClasses] = useState<Set<string>>(new Set());
  const [selectedSchools, setSelectedSchools] = useState<Set<string>>(new Set());
  const [selectedLevels, setSelectedLevels] = useState<Set<number>>(new Set());

  const [availableSpells, setAvailableSpells] = useState<SpellInfo[]>([]);
  const [, setAvailableSpellsLoading] = useState(false);
  const [availableSpellsError, setAvailableSpellsError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [hasNext, setHasNext] = useState(false);
  const [hasPrevious, setHasPrevious] = useState(false);
  const [debugMode, setDebugMode] = useState(false);
  const [abilitySpells, setAbilitySpells] = useState<Array<{
    spell_id: number; name: string; icon: string;
    description?: string; school_name?: string; innate_level: number;
  }>>([]);
  const [abilitySpellsExpanded, setAbilitySpellsExpanded] = useState(false);
  const SPELLS_PER_PAGE = 100;

  const [removingSpellKey, setRemovingSpellKey] = useState<string | null>(null);
  const [addingSpellKey, setAddingSpellKey] = useState<string | null>(null);
  const [addedSpellKey, setAddedSpellKey] = useState<string | null>(null);
  const pendingAddSpellRef = useRef<string | null>(null);

  const spellsData = spells.data as SpellsState | null;
  const isLoading = characterLoading || spells.isLoading;
  const error = characterError || spells.error || availableSpellsError;

  useEffect(() => {
    if (character?.id && !spells.data && !spells.isLoading) {
      spells.load();
    }
  }, [character?.id, spells.data, spells.isLoading, spells]);

  useEffect(() => {
    if (character?.id) {
      CharacterAPI.getAbilitySpells()
        .then(setAbilitySpells)
        .catch(() => setAbilitySpells([]));
    }
  }, [character?.id]);

  useEffect(() => {
    setCurrentPage(1);
  }, [activeTab, searchTerm, selectedClasses, selectedSchools, selectedLevels, debugMode]);

  useEffect(() => {
    const loadAvailableSpells = async () => {
      if (!character?.id) {
        return;
      }

      setAvailableSpellsLoading(true);
      setAvailableSpellsError(null);

      try {
        const schools = selectedSchools.size > 0
          ? Array.from(selectedSchools).join(',')
          : undefined;

        const levels = selectedLevels.size > 0
          ? Array.from(selectedLevels).join(',')
          : undefined;

        const classId = selectedClasses.size === 1
          ? Number(Array.from(selectedClasses)[0])
          : undefined;

        const response = await CharacterAPI.getLegitimateSpells(character.id, {
          page: currentPage,
          limit: SPELLS_PER_PAGE,
          schools,
          levels,
          search: (searchTerm && searchTerm.length >= 3) ? searchTerm : undefined,
          class_id: classId,
          show_all: debugMode || undefined,
        });

        setAvailableSpells(response.spells);
        setTotalSpells(response.pagination.total);
        setHasNext(response.pagination.has_next);
        setHasPrevious(response.pagination.has_previous);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to load available spells';
        setAvailableSpellsError(errorMessage);
      } finally {
        setAvailableSpellsLoading(false);
      }
    };

    loadAvailableSpells();
  }, [character?.id, activeTab, currentPage, SPELLS_PER_PAGE, selectedClasses, selectedSchools, selectedLevels, searchTerm, setTotalSpells, debugMode]);

  const casterClasses = useMemo(() => mapCasterClasses(spellsData?.spellcasting_classes), [spellsData]);

  const hasEditableClasses = useMemo(() => {
    return casterClasses.some(cls => cls.can_edit_spells);
  }, [casterClasses]);

  const readOnlyClassNames = useMemo(() => {
    return casterClasses
      .filter(cls => !cls.can_edit_spells)
      .map(cls => cls.name);
  }, [casterClasses]);

  const availableClassFilters = useMemo(() => {
    if (!spellsData?.spellcasting_classes) return [];
    return spellsData.spellcasting_classes.map((cls) => ({
      name: cls.class_name,
      value: cls.class_id.toString(),
    }));
  }, [spellsData]);

  const allMySpells = useMemo(() => mapKnownSpellsToSpellInfo(spellsData?.known_spells), [spellsData?.known_spells]);

  const preparedSpells = useMemo(() => groupMemorizedSpells(spellsData?.memorized_spells), [spellsData?.memorized_spells]);

  const ownedSpellIds = useMemo(() => new Set(allMySpells.map(s => s.id)), [allMySpells]);

  const filterAndSortSpells = useCallback((spells: SpellInfo[]) => {
    const filtered = filterSpells(spells, {
      classes: selectedClasses,
      schools: selectedSchools,
      levels: selectedLevels,
    });
    return sortSpells(filtered, sortBy);
  }, [selectedClasses, selectedSchools, selectedLevels, sortBy]);

  const { searchResults: searchedMySpells } = useSpellSearch(allMySpells, searchTerm);
  const filteredMySpells = useMemo(() => filterAndSortSpells(searchedMySpells), [searchedMySpells, filterAndSortSpells]);

  const filteredAvailableSpells = useMemo(() => {
    const spellsToShow = debugMode
      ? availableSpells
      : availableSpells.filter(spell => !ownedSpellIds.has(spell.id));
    return sortSpells(spellsToShow, sortBy);
  }, [availableSpells, ownedSpellIds, sortBy, debugMode]);

  const finalAvailableSpells = filteredAvailableSpells;

  const handleAddSpell = useCallback(async (spellId: number, classIndex: number, spellLevel: number) => {
    if (!character?.id || addingSpellKey) return;

    const key = `${spellId}-${spellLevel}`;
    pendingAddSpellRef.current = key;

    setAddingSpellKey(key);
    await new Promise(resolve => setTimeout(resolve, 180));

    try {
      const response = await addSpell(spellId, classIndex, spellLevel);

      setAddingSpellKey(null);
      setAddedSpellKey(pendingAddSpellRef.current);
      pendingAddSpellRef.current = null;
      setTimeout(() => setAddedSpellKey(null), 250);

      showToast(response.message || 'Spell learned successfully', 'success');
    } catch (error) {
      pendingAddSpellRef.current = null;
      setAddingSpellKey(null);
      handleError(error);
    }
  }, [character?.id, addSpell, showToast, handleError, addingSpellKey]);

  const handleRemoveSpell = useCallback(async (spellId: number, classIndex: number, spellLevel: number) => {
    if (!character?.id || removingSpellKey) return;

    const key = `${spellId}-${spellLevel}`;

    if (addedSpellKey === key) setAddedSpellKey(null);
    setRemovingSpellKey(key);

    await new Promise(resolve => setTimeout(resolve, 180));

    try {
      const response = await removeSpell(spellId, classIndex, spellLevel);
      showToast(response.message || 'Spell removed successfully', 'success');
    } catch (error) {
      handleError(error);
    } finally {
      setRemovingSpellKey(null);
    }
  }, [character?.id, removeSpell, showToast, handleError, removingSpellKey, addedSpellKey]);

  const totalPages = useMemo(() => {
    return Math.ceil(totalSpells / SPELLS_PER_PAGE);
  }, [totalSpells, SPELLS_PER_PAGE]);

  // Apply search/filter to prepared spells
  const { searchResults: searchedPreparedSpells } = useSpellSearch(preparedSpells, searchTerm);
  const filteredPreparedSpells = useMemo(() => filterAndSortSpells(searchedPreparedSpells), [searchedPreparedSpells, filterAndSortSpells]);

  const filteredCount = useMemo(() => {
    if (activeTab === 'my-spells') return filteredMySpells.length;
    if (activeTab === 'prepared') return filteredPreparedSpells.length;
    if (activeTab === 'all-spells') return totalSpells;
    return 0;
  }, [activeTab, filteredMySpells.length, filteredPreparedSpells.length, totalSpells]);

  const handlePageChange = useCallback((newPage: number) => {
    setCurrentPage(newPage);
  }, []);

  if (isLoading && !spellsData) {
    return (
      <div className="flex flex-col gap-4">
        <SpellNavBar
          activeTab={activeTab}
          onTabChange={setActiveTab}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
          sortBy={sortBy}
          onSortChange={setSortBy}
          selectedClasses={selectedClasses}
          onClassesChange={setSelectedClasses}
          selectedSchools={selectedSchools}
          onSchoolsChange={setSelectedSchools}
          selectedLevels={selectedLevels}
          onLevelsChange={setSelectedLevels}
          mySpellsCount={0}
          preparedSpellsCount={0}
          availableSpellsCount={totalSpells}
          filteredCount={0}
          currentPage={1}
          totalPages={1}
          hasNext={false}
          hasPrevious={false}
          onPageChange={() => {}}
          availableClasses={availableClassFilters}
          debugMode={debugMode}
          onDebugToggle={() => setDebugMode(prev => !prev)}
        />
        <div className="flex items-center justify-center h-64 bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))] rounded-lg">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[rgb(var(--color-primary))]"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Card variant="error">
        <div className="flex items-center gap-2">
          <AlertCircle className="w-5 h-5 text-error" />
          <p className="text-error">{error}</p>
        </div>
      </Card>
    );
  }

  if (!character || !spellsData) {
    return (
      <Card variant="warning">
        <p className="text-muted">No character loaded. Please import a save file to begin.</p>
      </Card>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="sticky top-0 z-10 mb-4">
        <SpellNavBar
          activeTab={activeTab}
          onTabChange={setActiveTab}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
          sortBy={sortBy}
          onSortChange={setSortBy}
          selectedClasses={selectedClasses}
          onClassesChange={setSelectedClasses}
          selectedSchools={selectedSchools}
          onSchoolsChange={setSelectedSchools}
          selectedLevels={selectedLevels}
          onLevelsChange={setSelectedLevels}
          mySpellsCount={allMySpells.length}
          preparedSpellsCount={preparedSpells.length}
          availableSpellsCount={totalSpells}
          filteredCount={filteredCount}
          currentPage={currentPage}
          totalPages={totalPages}
          hasNext={hasNext}
          hasPrevious={hasPrevious}
          onPageChange={handlePageChange}
          availableClasses={availableClassFilters}
          debugMode={debugMode}
          onDebugToggle={() => setDebugMode(prev => !prev)}
        />
      </div>

      {readOnlyClassNames.length > 0 && (
        <div className="mb-4 flex items-start gap-3 p-3 bg-[rgb(var(--color-surface-2))] border border-[rgb(var(--color-surface-border))] rounded-lg">
          <Info className="w-5 h-5 text-[rgb(var(--color-text-muted))] flex-shrink-0 mt-0.5" />
          <p className="text-sm text-[rgb(var(--color-text-secondary))]">
            {hasEditableClasses ? (
              <>
                <span className="font-medium text-[rgb(var(--color-text-primary))]">
                  {readOnlyClassNames.join(', ')}
                </span>
                {' '}{t('spells.readOnlyClassNotice')}
              </>
            ) : (
              t('spells.noEditableClasses')
            )}
          </p>
        </div>
      )}

      {abilitySpells.length > 0 && (
        <div className="mb-4 bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))] rounded-lg overflow-hidden">
          <button
            onClick={() => setAbilitySpellsExpanded(prev => !prev)}
            className="w-full flex items-center justify-between p-3 hover:bg-[rgb(var(--color-surface-2))] transition-colors"
          >
            <div className="flex items-center gap-2">
              <Zap className="w-4 h-4 text-amber-500" />
              <span className="text-sm font-medium text-[rgb(var(--color-text-primary))]">
                {t('spells.abilitySpells')}
              </span>
              <Badge variant="secondary">{abilitySpells.length}</Badge>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-[rgb(var(--color-text-muted))]">
                {t('spells.abilitySpellsDescription')}
              </span>
              {abilitySpellsExpanded
                ? <ChevronUp className="w-4 h-4 text-[rgb(var(--color-text-muted))]" />
                : <ChevronDown className="w-4 h-4 text-[rgb(var(--color-text-muted))]" />
              }
            </div>
          </button>
          {abilitySpellsExpanded && (
            <div className="border-t border-[rgb(var(--color-surface-border))] p-3">
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
                {abilitySpells.map((spell) => (
                  <div
                    key={spell.spell_id}
                    className="flex items-center gap-3 p-2 rounded-md bg-[rgb(var(--color-surface-2))]"
                  >
                    <NWN2Icon icon={spell.icon} size="md" />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-[rgb(var(--color-text-primary))] truncate">
                        {display(stripNwn2Tags(spell.name))}
                      </p>
                      <div className="flex items-center gap-1.5">
                        {spell.school_name && (
                          <span className="text-xs text-[rgb(var(--color-text-muted))]">
                            {spell.school_name}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      <SpellTabContent
        activeTab={activeTab}
        mySpells={filteredMySpells}
        preparedSpells={filteredPreparedSpells}
        allSpells={finalAvailableSpells}
        ownedSpellIds={ownedSpellIds}
        onAddSpell={handleAddSpell}
        onRemoveSpell={handleRemoveSpell}
        currentPage={currentPage}
        totalPages={totalPages}
        hasNext={hasNext}
        hasPrevious={hasPrevious}
        onPageChange={handlePageChange}
        casterClasses={casterClasses}
        removingSpellKey={removingSpellKey}
        addingSpellKey={addingSpellKey}
        addedSpellKey={addedSpellKey}
      />
    </div>
  );
}