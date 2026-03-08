
import { useState, useMemo, useEffect, useCallback, useRef } from 'react';
import { Card } from '@/components/ui/Card';
import { AlertCircle } from 'lucide-react';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { useFeatManagement } from '@/hooks/useFeatManagement';
import { useFeatSearch } from '@/hooks/useFeatSearch';
import { useDebouncedValue } from '@/hooks/useDebouncedValue';
import { FeatNavBar, type FeatTab } from './FeatNavBar';
import { FeatTabContent } from './FeatTabContent';
import type { FeatInfo, FeatsState } from './types';
import { useToast } from '@/contexts/ToastContext';
import { useErrorHandler } from '@/hooks/useErrorHandler';

export default function FeatsEditor() {
  const {
    character,
    isLoading: characterLoading,
    error: characterError,
    totalFeats,
    setTotalFeats
  } = useCharacterContext();
  const feats = useSubsystem('feats');
  const { addFeat, removeFeat } = useFeatManagement();
  const { showToast } = useToast();
  const { handleError } = useErrorHandler();

  const [activeTab, setActiveTab] = useState<FeatTab>('my-feats');
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearch = useDebouncedValue(searchTerm, 300);
  const [sortBy, setSortBy] = useState('name');
  const [selectedTypes, setSelectedTypes] = useState<Set<number>>(new Set());
  const [showAvailableOnly, setShowAvailableOnly] = useState(false);
  const [availableFeats, setAvailableFeats] = useState<FeatInfo[]>([]);
  const [,setAvailableFeatsLoading] = useState(false);
  const [availableFeatsError, setAvailableFeatsError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [, setAvailableTotal] = useState(0); 
  const [hasNext, setHasNext] = useState(false);
  const [hasPrevious, setHasPrevious] = useState(false);
  const FEATS_PER_PAGE = 100;

  const [removingFeatId, setRemovingFeatId] = useState<number | null>(null);
  const [addingFeatId, setAddingFeatId] = useState<number | null>(null);
  const [addedFeatId, setAddedFeatId] = useState<number | null>(null);
  const pendingAddFeatRef = useRef<number | null>(null);

  const featsData = feats.data as FeatsState | null;
  const isLoading = characterLoading || feats.isLoading;
  const error = characterError || feats.error || availableFeatsError;

  useEffect(() => {
    if (character?.id && !feats.data && !feats.isLoading) {
      feats.load();
    }
  }, [character?.id, feats.data, feats.isLoading, feats]);

  useEffect(() => {
    setCurrentPage(1);
  }, [activeTab, debouncedSearch, selectedTypes, showAvailableOnly]);

  useEffect(() => {
    const loadAvailableFeats = async () => {
      if (!character?.id) {
        return;
      }

      setAvailableFeatsLoading(true);
      setAvailableFeatsError(null);

      try {
        const featTypeBitmask = selectedTypes.size > 0
          ? Array.from(selectedTypes).reduce((acc, type) => acc | type, 0)
          : undefined;

        if (showAvailableOnly) {
           const response = await CharacterAPI.getAvailableFeats(character.id, featTypeBitmask);
           setAvailableFeats(response.available_feats);
           setAvailableTotal(response.total);
           setTotalFeats(response.total);
           setHasNext(false); 
           setHasPrevious(false);
        } else {
           const response = await CharacterAPI.getLegitimateFeats(character.id, {
             page: currentPage,
             limit: FEATS_PER_PAGE,
             featType: featTypeBitmask,
             search: (debouncedSearch && debouncedSearch.length >= 3) ? debouncedSearch : undefined,
           });

           setAvailableFeats(response.feats);
           setTotalFeats(response.pagination.total);
           setAvailableTotal(response.pagination.total);
           setHasNext(response.pagination.has_next);
           setHasPrevious(response.pagination.has_previous);
        }

      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to load available feats';
        setAvailableFeatsError(errorMessage);
      } finally {
        setAvailableFeatsLoading(false);
      }
    };

    loadAvailableFeats();
  }, [character?.id, activeTab, currentPage, FEATS_PER_PAGE, selectedTypes, debouncedSearch, showAvailableOnly, setTotalFeats]);


  const allMyFeats = useMemo(() => {
    if (!featsData?.summary) return [];

    const allFeats = [
      ...(featsData.summary.protected || []),
      ...(featsData.summary.class_feats || []),
      ...(featsData.summary.general_feats || []),
      ...(featsData.summary.custom_feats || []),
      ...(featsData.summary.background_feats || []),
      ...(featsData.summary.domain_feats || []),
    ];

    const uniqueFeats = new Map<number, FeatInfo>();
    allFeats.forEach(feat => {
      uniqueFeats.set(feat.id, feat);
    });

    return Array.from(uniqueFeats.values());
  }, [featsData]);

  const ownedFeatIds = useMemo(() => {
    return new Set(allMyFeats.map(f => f.id));
  }, [allMyFeats]);

  const protectedFeatIds = useMemo(() => {
    if (!featsData?.summary) return new Set<number>();
    return new Set((featsData.summary.protected || []).map(f => f.id));
  }, [featsData]);

  const filterAndSortFeats = useCallback((feats: FeatInfo[]) => {
    let filtered = [...feats];

    if (selectedTypes.size > 0) {
      filtered = filtered.filter(feat => {
        return Array.from(selectedTypes).some(type => (feat.type & type) !== 0);
      });
    }

    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.name.localeCompare(b.name);
        case 'type':
          return a.type - b.type;
        case 'level':
          return 0;
        default:
          return 0;
      }
    });

    return filtered;
  }, [selectedTypes, sortBy]);

  const { searchResults: searchedMyFeats } = useFeatSearch(allMyFeats, searchTerm);
  const filteredMyFeats = useMemo(() => filterAndSortFeats(searchedMyFeats), [searchedMyFeats, filterAndSortFeats]);

  const filteredAvailableFeats = useMemo(() => {
    let listToFilter = availableFeats;
    
    // Filter out owned feats
    listToFilter = listToFilter.filter(feat => !ownedFeatIds.has(feat.id));

    if (showAvailableOnly && searchTerm) {
       const lowerSearch = searchTerm.toLowerCase();
       listToFilter = listToFilter.filter(feat => 
          feat.name.toLowerCase().includes(lowerSearch) || 
          feat.label?.toLowerCase().includes(lowerSearch)
       );
    }

    return listToFilter.sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.name.localeCompare(b.name);
        case 'type':
          return a.type - b.type;
        case 'level':
          return 0;
        default:
          return 0;
      }
    });
  }, [availableFeats, ownedFeatIds, sortBy, showAvailableOnly, searchTerm]);

  const finalAvailableFeats = filteredAvailableFeats;

  const handleAddFeat = useCallback(async (featId: number) => {
    if (!character?.id || addingFeatId) return;

    pendingAddFeatRef.current = featId;
    setAddingFeatId(featId);
    await new Promise(resolve => setTimeout(resolve, 180));

    try {
      const response = await addFeat(featId);

      setAddingFeatId(null);
      setAddedFeatId(pendingAddFeatRef.current);
      pendingAddFeatRef.current = null;
      setTimeout(() => setAddedFeatId(null), 250);

      const parts: string[] = [];
      if (response.auto_added_feats?.length) {
        const names = response.auto_added_feats.map(f => f.label).join(', ');
        parts.push(`Prerequisites added: ${names}`);
      }
      if (response.auto_modified_abilities?.length) {
        const changes = response.auto_modified_abilities
          .map(a => `${a.ability} ${a.old_value} -> ${a.new_value}`)
          .join(', ');
        parts.push(`Abilities modified: ${changes}`);
      }

      if (parts.length > 0) {
        showToast(`Feat added. ${parts.join('. ')}`, 'success', 6000);
      } else {
        showToast('Feat added successfully', 'success');
      }
    } catch (error) {
      pendingAddFeatRef.current = null;
      setAddingFeatId(null);
      handleError(error);
    }
  }, [character?.id, addFeat, showToast, handleError, addingFeatId]);

  const handleRemoveFeat = useCallback(async (featId: number) => {
    if (!character?.id || removingFeatId) return;

    if (addedFeatId === featId) setAddedFeatId(null);
    setRemovingFeatId(featId);

    await new Promise(resolve => setTimeout(resolve, 180));

    try {
      await removeFeat(featId);
      showToast('Feat removed successfully', 'success');
    } catch (error) {
      handleError(error);
    } finally {
      setRemovingFeatId(null);
    }
  }, [character?.id, removeFeat, showToast, handleError, removingFeatId, addedFeatId]);

  const handleLoadFeatDetails = useCallback(async (feat: FeatInfo): Promise<FeatInfo | null> => {
    if (!character?.id) return null;

    try {
      const details = await CharacterAPI.getFeatDetails(character.id, feat.id);
      return details;
    } catch {
      return null;
    }
  }, [character?.id]);

  const totalPages = useMemo(() => {
    return Math.ceil(totalFeats / FEATS_PER_PAGE);
  }, [totalFeats, FEATS_PER_PAGE]);

  const filteredCount = useMemo(() => {
    if (activeTab === 'my-feats') return filteredMyFeats.length;
    if (activeTab === 'all-feats') return totalFeats;
    return 0;
  }, [activeTab, filteredMyFeats.length, totalFeats]);

  const handlePageChange = useCallback((newPage: number) => {
    setCurrentPage(newPage);
  }, []);

  if (isLoading && !featsData) {
    return (
      <div className="flex flex-col gap-4">
        <FeatNavBar
          activeTab={activeTab}
          onTabChange={setActiveTab}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
          sortBy={sortBy}
          onSortChange={setSortBy}
          selectedTypes={selectedTypes}
          onTypesChange={setSelectedTypes}
          myFeatsCount={0}
          availableFeatsCount={totalFeats}
          filteredCount={0}
          currentPage={1}
          totalPages={1}
          hasNext={false}
          hasPrevious={false}
          onPageChange={() => {}}
          showAvailableOnly={showAvailableOnly}
          onAvailableOnlyChange={setShowAvailableOnly}
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

  if (!character || !featsData) {
    return (
      <Card variant="warning">
        <p className="text-muted">No character loaded. Please import a save file to begin.</p>
      </Card>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="sticky top-0 z-10 mb-4">
        <FeatNavBar
          activeTab={activeTab}
          onTabChange={setActiveTab}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
          sortBy={sortBy}
          onSortChange={setSortBy}
          selectedTypes={selectedTypes}
          onTypesChange={setSelectedTypes}
          myFeatsCount={allMyFeats.length}
          availableFeatsCount={totalFeats}
          filteredCount={filteredCount}
          currentPage={currentPage}
          totalPages={totalPages}
          hasNext={hasNext}
          hasPrevious={hasPrevious}
          onPageChange={handlePageChange}
          showAvailableOnly={showAvailableOnly}
          onAvailableOnlyChange={setShowAvailableOnly}
        />
      </div>

      <FeatTabContent
        activeTab={activeTab}
        myFeats={filteredMyFeats}
        allFeats={finalAvailableFeats}
        ownedFeatIds={ownedFeatIds}
        protectedFeatIds={protectedFeatIds}
        onAddFeat={handleAddFeat}
        onRemoveFeat={handleRemoveFeat}
        onLoadFeatDetails={handleLoadFeatDetails}
        currentPage={currentPage}
        totalPages={totalPages}
        hasNext={hasNext}
        hasPrevious={hasPrevious}
        onPageChange={handlePageChange}
        removingFeatId={removingFeatId}
        addingFeatId={addingFeatId}
        addedFeatId={addedFeatId}
      />
    </div>
  );
}
