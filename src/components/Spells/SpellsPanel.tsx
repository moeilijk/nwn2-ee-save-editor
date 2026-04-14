import { useState, useMemo, useCallback, useEffect } from 'react';
import { Button, InputGroup, Tabs, Tab, Popover, Menu, MenuItem, Spinner, NonIdealState } from '@blueprintjs/core';
import { GiVisoredHelm, GiFunnel, GiMagnifyingGlass } from 'react-icons/gi';
import { GameIcon } from '../shared/GameIcon';
import { useDebouncedValue } from '@/hooks/useDebouncedValue';
import { T, SPELL_SCHOOL_COLORS } from '../theme';
import { SplitPane, GroupedList } from '../shared';
import type { ListSection } from '../shared';
import { SpellDetail } from './SpellDetail';
import { useSubsystem } from '@/contexts/CharacterContext';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { mapKnownSpellsToSpellInfo, groupMemorizedSpells, mapCasterClasses } from '@/utils/spellUtils';
import type { SpellInfo, SpellsState, AbilitySpellEntry } from '@/components/Spells/types';
import { display } from '@/utils/dataHelpers';
import { useSpellManagement } from '@/hooks/useSpellManagement';
import { useToast } from '@/contexts/ToastContext';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { useTranslations } from '@/hooks/useTranslations';

type TabId = 'known' | 'prepared' | 'all';

const SPELLS_PER_PAGE = 10000;

const SPELL_SCHOOL_OPTIONS = [
  'Abjuration', 'Conjuration', 'Divination', 'Enchantment',
  'Evocation', 'Illusion', 'Necromancy', 'Transmutation',
];

export function SpellsPanel() {
  const { character, allSpellsCache } = useCharacterContext();

  const spells = useSubsystem('spells');
  const spellsData = spells.data as SpellsState | null;
  const abilitySpells = spellsData?.ability_spells ?? [];
  const { addSpell, removeSpell } = useSpellManagement();
  const { showToast } = useToast();
  const { handleError } = useErrorHandler();
  const t = useTranslations();

  const [tab, setTab] = useState<TabId>('known');
  const [search, setSearch] = useState('');
  const [activeSchool, setActiveSchool] = useState<string>('all');
  const [activeLevel, setActiveLevel] = useState<string>('all');
  const [activeClass, setActiveClass] = useState<string>('all');
  const [selectedSpell, setSelectedSpell] = useState<SpellInfo | null>(null);

  const [allSpells, setAllSpells] = useState<SpellInfo[]>([]);
  const [allSpellsLoading, setAllSpellsLoading] = useState(false);
  const [allSpellsPage, setAllSpellsPage] = useState(1);
  const [allSpellsTotal, setAllSpellsTotal] = useState(0);
  const [_allSpellsHasNext, setAllSpellsHasNext] = useState(false);
  const [_allSpellsHasPrev, setAllSpellsHasPrev] = useState(false);
  const [usedCache, setUsedCache] = useState(false);
  const [abilitiesOpen, setAbilitiesOpen] = useState(true);

  const debouncedSearch = useDebouncedValue(search, 300);

  useEffect(() => {
    if (character?.id && !spells.data && !spells.isLoading) {
      spells.load();
    }
  }, [character?.id, spells.data, spells.isLoading, spells]);

  // Use preloaded cache for initial unfiltered load
  useEffect(() => {
    if (allSpellsCache && allSpells.length === 0 && !usedCache) {
      setAllSpells(allSpellsCache.spells);
      setAllSpellsTotal(allSpellsCache.pagination.total);
      setAllSpellsHasNext(allSpellsCache.pagination.has_next);
      setAllSpellsHasPrev(allSpellsCache.pagination.has_previous);
      setUsedCache(true);
    }
  }, [allSpellsCache]); // eslint-disable-line react-hooks/exhaustive-deps

  // Fetch when filters change on the 'all' tab
  useEffect(() => {
    const characterId = character?.id;
    if (!characterId || tab !== 'all') return;

    // Skip fetch if we can use the cache (no filters active, page 1)
    const hasFilters = activeSchool !== 'all' || activeLevel !== 'all' || activeClass !== 'all' || debouncedSearch.length >= 3;
    if (!hasFilters && allSpellsPage === 1 && usedCache) return;

    const loadAllSpells = async () => {
      setAllSpellsLoading(true);
      try {
        const schoolParam = activeSchool !== 'all' ? activeSchool : undefined;
        const levelParam = activeLevel !== 'all' ? activeLevel : undefined;
        const searchParam = debouncedSearch.length >= 3 ? debouncedSearch : undefined;
        const foundClass = activeClass !== 'all'
          ? casterClasses.find(c => c.name === activeClass)
          : undefined;
        const classParam = foundClass?.class_id;

        const response = await CharacterAPI.getLegitimateSpells(characterId, {
          page: allSpellsPage,
          limit: SPELLS_PER_PAGE,
          schools: schoolParam,
          levels: levelParam,
          search: searchParam,
          ...(classParam !== undefined ? { class_id: classParam } : {}),
        });

        setAllSpells(response.spells);
        setAllSpellsTotal(response.pagination.total);
        setAllSpellsHasNext(response.pagination.has_next);
        setAllSpellsHasPrev(response.pagination.has_previous);
      } catch {
        setAllSpells([]);
      } finally {
        setAllSpellsLoading(false);
      }
    };

    loadAllSpells();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [character?.id, tab, allSpellsPage, activeSchool, activeLevel, activeClass, debouncedSearch, usedCache]);

  const casterClasses = useMemo(() => mapCasterClasses(spellsData?.spellcasting_classes), [spellsData]);

  const editableClasses = useMemo(
    () => casterClasses.filter(c => c.can_edit_spells),
    [casterClasses]
  );

  const handleAddSpell = useCallback(async (spellId: number, classIndex: number, spellLevel: number) => {
    try {
      const response = await addSpell(spellId, classIndex, spellLevel);
      showToast(response.message || t('placeholders.spellAdded'), 'success');
      setAllSpells(prev => prev.filter(s => s.id !== spellId));
      setAllSpellsTotal(prev => prev - 1);
      setSelectedSpell(null);
    } catch (error) { handleError(error); }
  }, [addSpell, showToast, handleError, t]);

  const handleRemoveSpell = useCallback(async (spellId: number, classIndex: number, spellLevel: number) => {
    try {
      const response = await removeSpell(spellId, classIndex, spellLevel);
      showToast(response.message || t('placeholders.spellRemoved'), 'success');
      setSelectedSpell(null);
    } catch (error) { handleError(error); }
  }, [removeSpell, showToast, handleError, t]);

  const clearFilters = useCallback(() => {
    setSearch('');
    setActiveSchool('all');
    setActiveLevel('all');
    setActiveClass('all');
    setAllSpellsPage(1);
  }, []);

  const clientFilter = useCallback((spell: SpellInfo) => {
    if (search.length >= 3 && !spell.name.toLowerCase().includes(search.toLowerCase())) return false;
    if (activeSchool !== 'all' && (spell.school_name || spell.school) !== activeSchool) return false;
    if (activeLevel !== 'all' && spell.level !== Number(activeLevel)) return false;
    if (activeClass !== 'all') {
      const cls = casterClasses.find(c => c.name === activeClass);
      if (cls && spell.class_id !== undefined && spell.class_id !== cls.class_id) return false;
    }
    return true;
  }, [search, activeSchool, activeLevel, activeClass, casterClasses]);

  const knownSpells = useMemo(() => mapKnownSpellsToSpellInfo(spellsData?.known_spells), [spellsData?.known_spells]);
  const preparedSpells = useMemo(() => groupMemorizedSpells(spellsData?.memorized_spells), [spellsData?.memorized_spells]);

  const levelLabel = (l: number) => l === 0 ? t('spells.cantrips') : t('spells.levelSpells', { level: l });

  const knownSections: ListSection<SpellInfo>[] = useMemo(() => {
    const filtered = knownSpells.filter(clientFilter);
    const grouped = new Map<number, SpellInfo[]>();
    for (const s of filtered) {
      if (!grouped.has(s.level)) grouped.set(s.level, []);
      grouped.get(s.level)!.push(s);
    }
    return [...grouped.entries()]
      .sort(([a], [b]) => a - b)
      .map(([level, items]) => ({
        key: `lvl-${level}`,
        title: levelLabel(level),
        items,
      }));
  }, [knownSpells, clientFilter]);

  const preparedSections: ListSection<SpellInfo>[] = useMemo(() => {
    const filtered = preparedSpells.filter(clientFilter);
    const grouped = new Map<number, SpellInfo[]>();
    for (const s of filtered) {
      if (!grouped.has(s.level)) grouped.set(s.level, []);
      grouped.get(s.level)!.push(s);
    }
    return [...grouped.entries()]
      .sort(([a], [b]) => a - b)
      .map(([level, items]) => ({
        key: `prep-${level}`,
        title: levelLabel(level),
        items,
      }));
  }, [preparedSpells, clientFilter]);

  const allSections: ListSection<SpellInfo>[] = useMemo(() => {
    const grouped = new Map<number, SpellInfo[]>();
    for (const s of allSpells) {
      if (!grouped.has(s.level)) grouped.set(s.level, []);
      grouped.get(s.level)!.push(s);
    }
    return [...grouped.entries()]
      .sort(([a], [b]) => a - b)
      .map(([level, items]) => ({
        key: `all-${level}`,
        title: levelLabel(level),
        items,
      }));
  }, [allSpells]);

  const hasFilters = search.length > 0 || activeSchool !== 'all' || activeLevel !== 'all' || activeClass !== 'all';

  const sections = tab === 'known' ? knownSections : tab === 'prepared' ? preparedSections : allSections;

  const schoolLabel = activeSchool === 'all' ? t('spells.schoolAll') : activeSchool;
  const levelFilterLabel = activeLevel === 'all' ? t('spells.levelAll') : activeLevel === '0' ? t('spells.cantrips') : t('spells.levelSpells', { level: activeLevel });
  const classLabel = activeClass === 'all' ? t('spells.classAll') : activeClass;

  const schoolMenu = (
    <Menu>
      <MenuItem text={t('common.all')} active={activeSchool === 'all'} onClick={() => setActiveSchool('all')} />
      {SPELL_SCHOOL_OPTIONS.map(s => (
        <MenuItem key={s} text={s} active={activeSchool === s} onClick={() => setActiveSchool(s)} />
      ))}
    </Menu>
  );

  const levelMenu = (
    <Menu>
      <MenuItem text={t('common.all')} active={activeLevel === 'all'} onClick={() => setActiveLevel('all')} />
      <MenuItem text={t('spells.cantrips')} active={activeLevel === '0'} onClick={() => setActiveLevel('0')} />
      {[1, 2, 3, 4, 5, 6, 7, 8, 9].map(l => (
        <MenuItem key={l} text={t('spells.levelSpells', { level: l })} active={activeLevel === String(l)} onClick={() => setActiveLevel(String(l))} />
      ))}
    </Menu>
  );

  const classMenu = (
    <Menu>
      <MenuItem text={t('common.all')} active={activeClass === 'all'} onClick={() => setActiveClass('all')} />
      {casterClasses.map(c => (
        <MenuItem key={c.class_id} text={c.name} active={activeClass === c.name} onClick={() => setActiveClass(c.name)} />
      ))}
    </Menu>
  );

  const renderSpellItem = useCallback((spell: SpellInfo, selected: boolean) => {
    const schoolName = spell.school_name || spell.school;
    const schoolColor = SPELL_SCHOOL_COLORS[schoolName || ''] || T.textMuted;
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span className={selected ? 't-semibold' : undefined} style={{
          color: T.text,
          flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {display(spell.name)}
        </span>
        {spell.is_domain_spell && (
          <span className="t-medium" style={{ color: '#c62828', flexShrink: 0 }}>Domain</span>
        )}
        {spell.memorized_count !== undefined && spell.memorized_count > 0 && (
          <span className="t-medium" style={{ color: T.accent, flexShrink: 0 }}>{spell.memorized_count}x</span>
        )}
        {schoolName && (
          <span className="t-medium" style={{ color: schoolColor, flexShrink: 0 }}>
            {schoolName}
          </span>
        )}
      </div>
    );
  }, []);

  const totalKnown = knownSpells.length;
  const totalPrepared = preparedSpells.length;

  const allSpellsCount = allSpellsTotal || allSpellsCache?.pagination.total || 0;
  const allTabTitle = allSpellsCount > 0
    ? `${t('spells.allSpells')} (${allSpellsCount})`
    : t('spells.allSpells');

  const toolbar = (
    <>
      <Tabs
        id="spell-tabs" selectedTabId={tab}
        onChange={(id) => { setTab(id as TabId); setSelectedSpell(null); setAllSpellsPage(1); }}
        renderActiveTabPanelOnly
      >
        <Tab id="known" title={`${t('spells.known')} (${totalKnown})`} />
        <Tab id="prepared" title={`${t('spells.prepared')} (${totalPrepared})`} />
        <Tab id="all" title={allTabTitle} />
      </Tabs>
      {casterClasses.length > 1 && (
        <Popover content={classMenu} placement="bottom-start" minimal>
          <Button minimal rightIcon="caret-down" text={classLabel} />
        </Popover>
      )}
      <Popover content={schoolMenu} placement="bottom-start" minimal>
        <Button minimal rightIcon="caret-down" text={schoolLabel} />
      </Popover>
      <Popover content={levelMenu} placement="bottom-start" minimal>
        <Button minimal rightIcon="caret-down" text={levelFilterLabel} />
      </Popover>
      <InputGroup
        leftIcon="search" placeholder={t('spells.filterSpells')} value={search}
        onChange={e => setSearch(e.target.value)}
        rightElement={search ? <Button icon="cross" minimal onClick={() => setSearch('')} /> : undefined}
        style={{ maxWidth: 220 }}
      />
      <Button minimal icon={<GameIcon icon={GiFunnel} size={14} />} text={t('common.clear')} onClick={clearFilters} disabled={!hasFilters} />
      <div style={{ flex: 1 }} />
    </>
  );

  const isLoadingInitial = spells.isLoading && !spellsData;

  const listContent = () => {
    if (isLoadingInitial) {
      return (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 48 }}>
          <Spinner size={32} />
        </div>
      );
    }

    if (!character) {
      return (
        <NonIdealState
          icon={<GameIcon icon={GiVisoredHelm} size={40} />}
          title={t('common.noCharacterLoaded')}
          description={t('common.loadSaveToView', { section: t('navigation.spells').toLowerCase() })}
        />
      );
    }

    if (tab === 'all' && allSpellsLoading) {
      return (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 48 }}>
          <Spinner size={32} />
        </div>
      );
    }

    if (sections.length === 0) {
      return (
        <div style={{ padding: 24, textAlign: 'center', color: T.textMuted }}>
          {t('common.noMatchFilters', { items: t('navigation.spells').toLowerCase() })}
        </div>
      );
    }

    return (
      <GroupedList
        sections={sections}
        selectedId={selectedSpell?.id ?? null}
        onSelect={setSelectedSpell}
        renderItem={renderSpellItem}
      />
    );
  };

  const list = (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ flex: 1, minHeight: 0 }}>
        {listContent()}
      </div>
      {abilitySpells.length > 0 && (
        <>
          <div
            style={{
              display: 'flex', alignItems: 'center', gap: 6,
              padding: '6px 12px',
              background: T.sectionBg,
              borderBottom: `1px solid ${T.sectionBorder}`,
              borderTop: `1px solid ${T.sectionBorder}`,
              cursor: 'pointer',
              userSelect: 'none',
            }}
            onClick={() => setAbilitiesOpen(o => !o)}
          >
            <span style={{ color: T.accent, width: 10 }}>
              {abilitiesOpen ? '\u25BC' : '\u25B6'}
            </span>
            <span className="t-bold" style={{ color: T.accent, flex: 1 }}>{t('spells.specialAbilities')}</span>
            <span style={{ color: T.textMuted }}>{abilitySpells.length}</span>
          </div>
          {abilitiesOpen && abilitySpells.map(a => (
            <div key={a.spell_id} style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '5px 12px 5px 28px',
              borderBottom: `1px solid ${T.borderLight}`,
            }}>
              <span style={{ color: T.text, flex: 1 }}>{display(a.name)}</span>
              {a.school_name && (
                <span style={{ color: T.textMuted, flexShrink: 0 }}>{a.school_name}</span>
              )}
              <span className="t-medium" style={{ color: T.accent, flexShrink: 0 }}>Lv {a.innate_level}</span>
            </div>
          ))}
        </>
      )}
    </div>
  );

  const detailMemoizedCount = selectedSpell?.memorized_count;

  return (
    <SplitPane
      toolbar={toolbar}
      list={list}
      detail={
        <SpellDetail
          spell={selectedSpell}
          memorizedCount={detailMemoizedCount}
          isOwned={tab !== 'all'}
          editableClasses={editableClasses}
          onAdd={handleAddSpell}
          onRemove={handleRemoveSpell}
        />
      }
    />
  );
}
