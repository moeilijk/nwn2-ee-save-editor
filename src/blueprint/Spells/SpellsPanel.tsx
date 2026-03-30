import { useState, useMemo, useCallback, useEffect } from 'react';
import { Button, InputGroup, Tabs, Tab, Popover, Menu, MenuItem, Spinner, NonIdealState } from '@blueprintjs/core';
import { useDebouncedValue } from '@/hooks/useDebouncedValue';
import { T, SPELL_SCHOOL_COLORS } from '../theme';
import { SplitPane, GroupedList } from '../shared';
import type { ListSection } from '../shared';
import { SpellDetail } from './SpellDetail';
import { useSubsystem } from '@/contexts/CharacterContext';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { mapKnownSpellsToSpellInfo, groupMemorizedSpells, mapCasterClasses } from '@/utils/spellUtils';
import type { SpellInfo, SpellsState } from '@/components/Spells/types';
import { display } from '@/utils/dataHelpers';

type TabId = 'known' | 'prepared' | 'all';

const SPELLS_PER_PAGE = 100;

const SPELL_SCHOOL_OPTIONS = [
  'Abjuration', 'Conjuration', 'Divination', 'Enchantment',
  'Evocation', 'Illusion', 'Necromancy', 'Transmutation',
];

export function SpellsPanel() {
  const { character } = useCharacterContext();
  const spells = useSubsystem('spells');
  const spellsData = spells.data as SpellsState | null;

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
  const [allSpellsHasNext, setAllSpellsHasNext] = useState(false);
  const [allSpellsHasPrev, setAllSpellsHasPrev] = useState(false);

  const debouncedSearch = useDebouncedValue(search, 300);

  const [abilitySpells, setAbilitySpells] = useState<Array<{
    spell_id: number;
    name: string;
    icon: string;
    description?: string;
    school_name?: string;
    innate_level: number;
  }>>([]);

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
    const characterId = character?.id;
    if (!characterId) return;

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

    if (tab === 'all') {
      loadAllSpells();
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [character?.id, tab, allSpellsPage, activeSchool, activeLevel, activeClass, debouncedSearch]);

  const casterClasses = useMemo(() => mapCasterClasses(spellsData?.spellcasting_classes), [spellsData]);

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

  const levelLabel = (l: number) => l === 0 ? 'Cantrips' : `Level ${l} Spells`;

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

  const schoolLabel = activeSchool === 'all' ? 'School: All' : activeSchool;
  const levelFilterLabel = activeLevel === 'all' ? 'Level: All' : activeLevel === '0' ? 'Cantrips' : `Level ${activeLevel}`;
  const classLabel = activeClass === 'all' ? 'Class: All' : activeClass;

  const schoolMenu = (
    <Menu>
      <MenuItem text="All" active={activeSchool === 'all'} onClick={() => setActiveSchool('all')} />
      {SPELL_SCHOOL_OPTIONS.map(s => (
        <MenuItem key={s} text={s} active={activeSchool === s} onClick={() => setActiveSchool(s)} />
      ))}
    </Menu>
  );

  const levelMenu = (
    <Menu>
      <MenuItem text="All" active={activeLevel === 'all'} onClick={() => setActiveLevel('all')} />
      <MenuItem text="Cantrips" active={activeLevel === '0'} onClick={() => setActiveLevel('0')} />
      {[1, 2, 3, 4, 5, 6, 7, 8, 9].map(l => (
        <MenuItem key={l} text={`Level ${l}`} active={activeLevel === String(l)} onClick={() => setActiveLevel(String(l))} />
      ))}
    </Menu>
  );

  const classMenu = (
    <Menu>
      <MenuItem text="All" active={activeClass === 'all'} onClick={() => setActiveClass('all')} />
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
        <span style={{
          color: T.text,
          fontWeight: selected ? 600 : 400,
          flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {display(spell.name)}
        </span>
        {spell.is_domain_spell && (
          <span style={{ color: '#c62828', fontWeight: 500, flexShrink: 0 }}>Domain</span>
        )}
        {spell.memorized_count !== undefined && spell.memorized_count > 0 && (
          <span style={{ color: T.accent, fontWeight: 500, flexShrink: 0 }}>{spell.memorized_count}x</span>
        )}
        {schoolName && (
          <span style={{ color: schoolColor, fontWeight: 500, flexShrink: 0 }}>
            {schoolName}
          </span>
        )}
      </div>
    );
  }, []);

  const totalKnown = knownSpells.length;
  const totalPrepared = preparedSpells.length;

  const allTabTitle = allSpellsTotal > 0
    ? `All Spells (${allSpellsTotal})`
    : 'All Spells';

  const toolbar = (
    <>
      <Tabs
        id="spell-tabs" selectedTabId={tab}
        onChange={(id) => { setTab(id as TabId); setSelectedSpell(null); setAllSpellsPage(1); }}
        renderActiveTabPanelOnly
      >
        <Tab id="known" title={`Known (${totalKnown})`} />
        <Tab id="prepared" title={`Prepared (${totalPrepared})`} />
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
        leftIcon="search" placeholder="Filter spells..." value={search}
        onChange={e => setSearch(e.target.value)}
        rightElement={search ? <Button icon="cross" minimal onClick={() => setSearch('')} /> : undefined}
        style={{ maxWidth: 220 }}
      />
      <Button minimal icon="filter-remove" text="Clear" onClick={clearFilters} disabled={!hasFilters} />
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
          icon="person"
          title="No character loaded"
          description="Load a save file to view spells."
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
          No spells match your filters.
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
    <div>
      {listContent()}
      {tab === 'all' && allSpellsTotal > SPELLS_PER_PAGE && (
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8, padding: '8px 12px', borderTop: `1px solid ${T.sectionBorder}` }}>
          <Button minimal small icon="chevron-left" disabled={!allSpellsHasPrev} onClick={() => setAllSpellsPage(p => Math.max(1, p - 1))} />
          <span style={{ color: T.textMuted, fontSize: 12 }}>
            Page {allSpellsPage} of {Math.ceil(allSpellsTotal / SPELLS_PER_PAGE)}
          </span>
          <Button minimal small icon="chevron-right" disabled={!allSpellsHasNext} onClick={() => setAllSpellsPage(p => p + 1)} />
        </div>
      )}
      {abilitySpells.length > 0 && (
        <>
          <div style={{
            display: 'flex', alignItems: 'center', gap: 6,
            padding: '6px 12px',
            background: T.sectionBg,
            borderBottom: `1px solid ${T.sectionBorder}`,
            borderTop: `1px solid ${T.sectionBorder}`,
          }}>
            <span style={{ fontWeight: 700, color: T.accent, flex: 1 }}>Special Abilities</span>
            <span style={{ color: T.textMuted }}>{abilitySpells.length}</span>
          </div>
          {abilitySpells.map(a => (
            <div key={a.spell_id} style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '5px 12px 5px 28px',
              borderBottom: `1px solid ${T.borderLight}`,
            }}>
              <span style={{ color: T.text, flex: 1 }}>{display(a.name)}</span>
              {a.school_name && (
                <span style={{ color: T.textMuted, flexShrink: 0 }}>{a.school_name}</span>
              )}
              <span style={{ color: T.accent, fontWeight: 500, flexShrink: 0 }}>Lv {a.innate_level}</span>
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
      detail={<SpellDetail spell={selectedSpell} memorizedCount={detailMemoizedCount} />}
    />
  );
}
