import { useState, useMemo, useCallback, useEffect } from 'react';
import { Button, InputGroup, Tabs, Tab, Popover, Menu, MenuItem, NonIdealState, Spinner } from '@blueprintjs/core';
import { useDebouncedValue } from '@/hooks/useDebouncedValue';
import { T, FEAT_TYPE_COLORS } from '../theme';
import { SplitPane, GroupedList } from '../shared';
import type { ListSection } from '../shared';
import { FeatDetail } from './FeatDetail';
import { useSubsystem, useCharacterContext } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { aggregateFeats, filterFeatsByType, sortFeats, FEAT_TYPE_LABELS, getFeatTypeLabel } from '@/utils/featUtils';
import type { FeatInfo, FeatsState } from '@/components/Feats/types';
import { FEAT_TYPES } from '@/components/Feats/types';
import { display } from '@/utils/dataHelpers';
import { useFeatManagement } from '@/hooks/useFeatManagement';
import { useToast } from '@/contexts/ToastContext';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { useTranslations } from '@/hooks/useTranslations';

type TabId = 'my' | 'all';

const FEAT_TYPE_OPTIONS: { label: string; value: number }[] = [
  { label: 'General', value: FEAT_TYPES.GENERAL },
  { label: 'Proficiency', value: FEAT_TYPES.PROFICIENCY },
  { label: 'Skill/Save', value: FEAT_TYPES.SKILL_SAVE },
  { label: 'Metamagic', value: FEAT_TYPES.METAMAGIC },
  { label: 'Divine', value: FEAT_TYPES.DIVINE },
  { label: 'Epic', value: FEAT_TYPES.EPIC },
  { label: 'Class', value: FEAT_TYPES.CLASS },
  { label: 'Background', value: FEAT_TYPES.BACKGROUND },
  { label: 'Spellcasting', value: FEAT_TYPES.SPELLCASTING },
  { label: 'History', value: FEAT_TYPES.HISTORY },
  { label: 'Heritage', value: FEAT_TYPES.HERITAGE },
  { label: 'Item Creation', value: FEAT_TYPES.ITEM_CREATION },
  { label: 'Racial', value: FEAT_TYPES.RACIAL },
  { label: 'Domain', value: FEAT_TYPES.DOMAIN },
];


const MY_FEATS_SECTIONS: { key: keyof FeatsState['summary']; title: string }[] = [
  { key: 'general_feats', title: 'General Feats' },
  { key: 'class_feats', title: 'Class Feats' },
  { key: 'background_feats', title: 'Background Feats' },
  { key: 'domain_feats', title: 'Domain Feats' },
  { key: 'custom_feats', title: 'Custom Feats' },
  { key: 'protected', title: 'Protected Feats' },
];

const FEATS_PER_PAGE = 10000;

export function FeatsPanel() {
  const { character, allFeatsCache } = useCharacterContext();
  const featsSubsystem = useSubsystem('feats');
  const featsData = featsSubsystem.data as FeatsState | null;
  const isLoading = featsSubsystem.isLoading;
  const loadError = featsSubsystem.error;
  const { addFeat, removeFeat } = useFeatManagement({ autoLoadFeats: false, enableValidation: false });
  const { showToast } = useToast();
  const { handleError } = useErrorHandler();
  const t = useTranslations();

  const [tab, setTab] = useState<TabId>('my');
  const [search, setSearch] = useState('');
  const [activeTypeBit, setActiveTypeBit] = useState<number | null>(null);
  const [selectedFeat, setSelectedFeat] = useState<FeatInfo | null>(null);

  const [allFeats, setAllFeats] = useState<FeatInfo[]>([]);
  const [allFeatsLoading, setAllFeatsLoading] = useState(false);
  const [allFeatsError, setAllFeatsError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [_hasNext, setHasNext] = useState(false);
  const [_hasPrevious, setHasPrevious] = useState(false);
  const [allFeatsTotal, setAllFeatsTotal] = useState(0);
  const [usedCache, setUsedCache] = useState(false);

  const debouncedSearch = useDebouncedValue(search, 300);

  useEffect(() => {
    if (character && !featsData && !featsSubsystem.isLoading) {
      featsSubsystem.load();
    }
  }, [character, featsData, featsSubsystem]);

  // Use preloaded cache for initial unfiltered load
  useEffect(() => {
    if (allFeatsCache && allFeats.length === 0 && !usedCache) {
      setAllFeats(allFeatsCache.feats);
      setAllFeatsTotal(allFeatsCache.pagination.total);
      setHasNext(allFeatsCache.pagination.has_next);
      setHasPrevious(allFeatsCache.pagination.has_previous);
      setUsedCache(true);
    }
  }, [allFeatsCache]); // eslint-disable-line react-hooks/exhaustive-deps

  // Fetch when filters change on the 'all' tab
  useEffect(() => {
    if (tab !== 'all' || !character?.id) return;

    // Restore cache when returning to unfiltered state
    const hasFilters = activeTypeBit != null || debouncedSearch.length >= 3;
    if (!hasFilters && currentPage === 1 && usedCache && allFeatsCache) {
      setAllFeats(allFeatsCache.feats);
      setAllFeatsTotal(allFeatsCache.pagination.total);
      setHasNext(allFeatsCache.pagination.has_next);
      setHasPrevious(allFeatsCache.pagination.has_previous);
      return;
    }

    let cancelled = false;
    setAllFeatsLoading(true);
    setAllFeatsError(null);

    const featTypeBitmask = activeTypeBit ?? undefined;
    const searchParam = debouncedSearch.length >= 3 ? debouncedSearch : undefined;

    CharacterAPI.getLegitimateFeats(character.id, {
      page: currentPage,
      limit: FEATS_PER_PAGE,
      featType: featTypeBitmask,
      search: searchParam,
    }).then(response => {
      if (!cancelled) {
        setAllFeats(response.feats);
        setAllFeatsTotal(response.pagination.total);
        setHasNext(response.pagination.has_next);
        setHasPrevious(response.pagination.has_previous);
      }
    }).catch(err => {
      if (!cancelled) {
        setAllFeatsError(err instanceof Error ? err.message : 'Failed to load feats');
      }
    }).finally(() => {
      if (!cancelled) setAllFeatsLoading(false);
    });

    return () => { cancelled = true; };
  }, [tab, character?.id, currentPage, activeTypeBit, debouncedSearch, usedCache]);

  useEffect(() => {
    setCurrentPage(1);
  }, [tab, activeTypeBit, debouncedSearch]);

  const clearFilters = useCallback(() => {
    setSearch('');
    setActiveTypeBit(null);
  }, []);

  const handleAddFeat = useCallback(async (featId: number) => {
    try {
      const response = await addFeat(featId);
      showToast(response.message || t('placeholders.featAdded'), 'success');
      setAllFeats(prev => prev.filter(f => f.id !== featId));
      setAllFeatsTotal(prev => prev - 1);
      setSelectedFeat(null);
    } catch (error) { handleError(error); }
  }, [addFeat, showToast, handleError, t]);

  const handleRemoveFeat = useCallback(async (featId: number) => {
    try {
      await removeFeat(featId);
      showToast(t('placeholders.featRemoved'), 'success');
      setSelectedFeat(null);
    } catch (error) { handleError(error); }
  }, [removeFeat, showToast, handleError, t]);

  const allMyFeats = useMemo(() => aggregateFeats(featsData?.summary), [featsData]);

  const mySections: ListSection<FeatInfo>[] = useMemo(() => {
    const typeFilter = activeTypeBit !== null ? new Set([activeTypeBit]) : new Set<number>();
    const searchLower = search.toLowerCase();

    return MY_FEATS_SECTIONS.flatMap(({ key, title }) => {
      const raw = (featsData?.summary?.[key] as FeatInfo[] | undefined) ?? [];
      let items = typeFilter.size > 0 ? filterFeatsByType(raw, typeFilter) : raw;
      if (search.length >= 3) {
        items = items.filter(f => f.name.toLowerCase().includes(searchLower));
      }
      items = sortFeats(items, 'name');
      if (items.length === 0) return [];
      return [{ key, title, items }];
    });
  }, [featsData, activeTypeBit, search]);

  const allSections: ListSection<FeatInfo>[] = useMemo(() => {
    const grouped = new Map<string, FeatInfo[]>();
    for (const f of allFeats) {
      const label = getFeatTypeLabel(f.type);
      if (!grouped.has(label)) grouped.set(label, []);
      grouped.get(label)!.push(f);
    }
    return [...grouped.entries()].map(([label, items]) => ({
      key: label,
      title: `${label} Feats`,
      items,
    }));
  }, [allFeats]);

  const totalOwned = allMyFeats.length;
  const hasFilters = search.length > 0 || activeTypeBit !== null;
  const sections = tab === 'my' ? mySections : allSections;

  const renderFeatItem = useCallback((feat: FeatInfo, selected: boolean) => {
    const typeLabel = getFeatTypeLabel(feat.type);
    const typeColor = FEAT_TYPE_COLORS[typeLabel] || T.textMuted;
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{
          color: T.text,
          fontWeight: selected ? 600 : 400,
          flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {display(feat.name)}
        </span>
        <span style={{ color: typeColor, fontWeight: 500, flexShrink: 0 }}>
          {typeLabel}
        </span>
      </div>
    );
  }, []);

  const typeLabel = activeTypeBit === null
    ? 'Type: All'
    : (FEAT_TYPE_LABELS[activeTypeBit] ?? 'Type: All');

  const typeMenu = (
    <Menu>
      <MenuItem text="All" active={activeTypeBit === null} onClick={() => setActiveTypeBit(null)} />
      {FEAT_TYPE_OPTIONS.map(opt => (
        <MenuItem
          key={opt.value}
          text={opt.label}
          active={activeTypeBit === opt.value}
          onClick={() => setActiveTypeBit(opt.value)}
        />
      ))}
    </Menu>
  );

  const allFeatsCount = allFeatsTotal || allFeatsCache?.pagination.total || 0;
  const allTabTitle = allFeatsCount > 0
    ? `All Feats (${allFeatsCount})`
    : 'All Feats';

  const toolbar = (
    <>
      <Tabs
        id="feat-tabs" selectedTabId={tab}
        onChange={(id) => { setTab(id as TabId); setSelectedFeat(null); }}
        renderActiveTabPanelOnly
      >
        <Tab id="my" title={`My Feats (${totalOwned})`} />
        <Tab id="all" title={allTabTitle} />
      </Tabs>
      <Popover content={typeMenu} placement="bottom-start" minimal>
        <Button minimal rightIcon="caret-down" text={typeLabel} />
      </Popover>
      <InputGroup
        leftIcon="search" placeholder="Filter feats..." value={search}
        onChange={e => setSearch(e.target.value)}
        rightElement={search ? <Button icon="cross" minimal onClick={() => setSearch('')} /> : undefined}
        style={{ maxWidth: 220 }}
      />
      <Button minimal icon="filter-remove" text="Clear" onClick={clearFilters} disabled={!hasFilters} />
      <div style={{ flex: 1 }} />
    </>
  );

  const renderList = () => {
    if (tab === 'my') {
      if (isLoading && !featsData) {
        return (
          <NonIdealState icon={<Spinner size={30} />} title="Loading feats..." />
        );
      }
      if (loadError) {
        return (
          <NonIdealState icon="error" title="Failed to load feats" description={loadError} />
        );
      }
      if (!character || !featsData) {
        return (
          <NonIdealState icon="person" title="No character loaded" description="Load a save file to view feats." />
        );
      }
      if (sections.length === 0) {
        return (
          <NonIdealState icon="search" title="No feats match your filters" action={<Button minimal text="Clear filters" onClick={clearFilters} />} />
        );
      }
    }

    if (tab === 'all') {
      if (!character) {
        return (
          <NonIdealState icon="person" title="No character loaded" description="Load a save file to browse feats." />
        );
      }
      if (allFeatsLoading) {
        return (
          <NonIdealState icon={<Spinner size={30} />} title="Loading feats..." />
        );
      }
      if (allFeatsError) {
        return (
          <NonIdealState icon="error" title="Failed to load feats" description={allFeatsError} />
        );
      }
      if (sections.length === 0) {
        return (
          <NonIdealState icon="search" title="No feats match your filters" action={<Button minimal text="Clear filters" onClick={clearFilters} />} />
        );
      }
    }

    return (
      <GroupedList
        sections={sections}
        selectedId={selectedFeat?.id ?? null}
        onSelect={setSelectedFeat}
        renderItem={renderFeatItem}
      />
    );
  };

  return (
    <SplitPane
      toolbar={toolbar}
      list={renderList()}
      detail={
        <FeatDetail
          feat={selectedFeat}
          isOwned={tab === 'my'}
          onAdd={handleAddFeat}
          onRemove={handleRemoveFeat}
        />
      }
    />
  );
}
