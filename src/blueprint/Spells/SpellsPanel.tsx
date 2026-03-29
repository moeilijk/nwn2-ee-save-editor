import { useState, useMemo, useCallback } from 'react';
import { Button, InputGroup, Tabs, Tab, Popover, Menu, MenuItem } from '@blueprintjs/core';
import { T, SPELL_SCHOOL_COLORS } from '../theme';
import { SPELLS, MEMORIZED_SPELLS, ALL_SPELLS, ABILITY_SPELLS, SPELL_SCHOOL_OPTIONS } from '../dummy-data';
import type { DummySpell } from '../dummy-data';
import { SplitPane, GroupedList } from '../shared';
import type { ListSection } from '../shared';
import { SpellDetail } from './SpellDetail';

type SpellWithLevel = DummySpell & { level: number; memorizedCount?: number };

type TabId = 'known' | 'prepared' | 'all';

export function SpellsPanel() {
  const [tab, setTab] = useState<TabId>('known');
  const [search, setSearch] = useState('');
  const [activeSchool, setActiveSchool] = useState<string>('all');
  const [activeLevel, setActiveLevel] = useState<string>('all');
  const [activeClass, setActiveClass] = useState<string>('all');
  const [selectedSpell, setSelectedSpell] = useState<SpellWithLevel | null>(null);

  const casterClasses = SPELLS.casterClasses;

  const clearFilters = useCallback(() => {
    setSearch('');
    setActiveSchool('all');
    setActiveLevel('all');
    setActiveClass('all');
  }, []);

  const filterSpell = useCallback((name: string, school: string, level: number) => {
    if (search.length >= 3 && !name.toLowerCase().includes(search.toLowerCase())) return false;
    if (activeSchool !== 'all' && school !== activeSchool) return false;
    if (activeLevel !== 'all' && level !== Number(activeLevel)) return false;
    return true;
  }, [search, activeSchool, activeLevel]);

  const totalKnown = SPELLS.known.reduce((sum, g) => sum + g.spells.length, 0);
  const totalPrepared = MEMORIZED_SPELLS.length;

  const levelLabel = (l: number) => l === 0 ? 'Cantrips' : `Level ${l} Spells`;

  const knownSections: ListSection<SpellWithLevel>[] = useMemo(() =>
    SPELLS.known.map(g => ({
      key: `lvl-${g.level}`,
      title: levelLabel(g.level),
      items: g.spells
        .filter(s => filterSpell(s.name, s.school, g.level))
        .map(s => ({ ...s, level: g.level })),
    })).filter(s => s.items.length > 0),
    [filterSpell]
  );

  const preparedSections: ListSection<SpellWithLevel>[] = useMemo(() => {
    const filtered = MEMORIZED_SPELLS.filter(s => filterSpell(s.name, s.school, s.level));
    const grouped = new Map<number, SpellWithLevel[]>();
    for (const s of filtered) {
      if (!grouped.has(s.level)) grouped.set(s.level, []);
      grouped.get(s.level)!.push({
        id: s.id, name: s.name, school: s.school,
        description: '', isDomain: s.isDomain,
        level: s.level, memorizedCount: s.count,
      });
    }
    return [...grouped.entries()]
      .sort(([a], [b]) => a - b)
      .map(([level, items]) => ({
        key: `prep-${level}`,
        title: levelLabel(level),
        items,
      }));
  }, [filterSpell]);

  const allSections: ListSection<SpellWithLevel>[] = useMemo(() => {
    const filtered = ALL_SPELLS.filter(s => filterSpell(s.name, s.school, s.level));
    const grouped = new Map<number, SpellWithLevel[]>();
    for (const s of filtered) {
      if (!grouped.has(s.level)) grouped.set(s.level, []);
      grouped.get(s.level)!.push({ ...s, level: s.level });
    }
    return [...grouped.entries()]
      .sort(([a], [b]) => a - b)
      .map(([level, items]) => ({
        key: `all-${level}`,
        title: levelLabel(level),
        items,
      }));
  }, [filterSpell]);

  const hasFilters = search.length > 0 || activeSchool !== 'all' || activeLevel !== 'all' || activeClass !== 'all';

  const sections = tab === 'known' ? knownSections : tab === 'prepared' ? preparedSections : allSections;

  const schoolLabel = activeSchool === 'all' ? 'School: All' : activeSchool;
  const levelFilterLabel = activeLevel === 'all' ? 'Level: All' : activeLevel === '0' ? 'Cantrips' : `Level ${activeLevel}`;

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

  const classLabel = activeClass === 'all' ? 'Class: All' : activeClass;

  const classMenu = (
    <Menu>
      <MenuItem text="All" active={activeClass === 'all'} onClick={() => setActiveClass('all')} />
      {casterClasses.map(c => (
        <MenuItem key={c.className} text={c.className} active={activeClass === c.className} onClick={() => setActiveClass(c.className)} />
      ))}
    </Menu>
  );

  const renderSpellItem = useCallback((spell: SpellWithLevel, selected: boolean) => {
    const schoolColor = SPELL_SCHOOL_COLORS[spell.school] || T.textMuted;
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{
          color: T.text,
          fontWeight: selected ? 600 : 400,
          flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {spell.name}
        </span>
        {spell.isDomain && (
          <span style={{ color: '#c62828', fontWeight: 500, flexShrink: 0 }}>Domain</span>
        )}
        {spell.memorizedCount && spell.memorizedCount > 0 && (
          <span style={{ color: T.accent, fontWeight: 500, flexShrink: 0 }}>{spell.memorizedCount}x</span>
        )}
        <span style={{ color: schoolColor, fontWeight: 500, flexShrink: 0 }}>
          {spell.school}
        </span>
      </div>
    );
  }, []);

  const toolbar = (
    <>
      <Tabs
        id="spell-tabs" selectedTabId={tab}
        onChange={(id) => { setTab(id as TabId); setSelectedSpell(null); }}
        renderActiveTabPanelOnly
      >
        <Tab id="known" title={`Known (${totalKnown})`} />
        <Tab id="prepared" title={`Prepared (${totalPrepared})`} />
        <Tab id="all" title={`All Spells (${ALL_SPELLS.length})`} />
      </Tabs>
      <Popover content={classMenu} placement="bottom-start" minimal>
        <Button minimal rightIcon="caret-down" text={classLabel} />
      </Popover>
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

  const list = (
    <div>
      {sections.length > 0 ? (
        <GroupedList
          sections={sections}
          selectedId={selectedSpell?.id ?? null}
          onSelect={setSelectedSpell}
          renderItem={renderSpellItem}
        />
      ) : (
        <div style={{ padding: 24, textAlign: 'center', color: T.textMuted }}>No spells match your filters.</div>
      )}
      {ABILITY_SPELLS.length > 0 && (
        <>
          <div style={{
            display: 'flex', alignItems: 'center', gap: 6,
            padding: '6px 12px',
            background: T.sectionBg,
            borderBottom: `1px solid ${T.sectionBorder}`,
            borderTop: `1px solid ${T.sectionBorder}`,
          }}>
            <span style={{ fontWeight: 700, color: T.accent, flex: 1 }}>Special Abilities</span>
            <span style={{ color: T.textMuted }}>{ABILITY_SPELLS.length}</span>
          </div>
          {ABILITY_SPELLS.map(a => (
            <div key={a.id} style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '5px 12px 5px 28px',
              borderBottom: `1px solid ${T.borderLight}`,
            }}>
              <span style={{ color: T.text, flex: 1 }}>{a.name}</span>
              <span style={{ color: T.textMuted, flexShrink: 0 }}>{a.source}</span>
              <span style={{ color: T.accent, fontWeight: 500, flexShrink: 0 }}>{a.usesPerDay}/day</span>
            </div>
          ))}
        </>
      )}
    </div>
  );

  return (
    <SplitPane
      toolbar={toolbar}
      list={list}
      detail={<SpellDetail spell={selectedSpell} memorizedCount={selectedSpell?.memorizedCount} />}
    />
  );
}
