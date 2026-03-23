import { useState, useMemo, useCallback } from 'react';
import { Button, InputGroup, Tabs, Tab, Popover, Menu, MenuItem } from '@blueprintjs/core';
import { T } from '../theme';
import { FEATS, ALL_FEATS, FEAT_TYPE_OPTIONS } from '../dummy-data';
import type { DummyFeat } from '../dummy-data';
import { SplitPane, GroupedList } from '../shared';
import type { ListSection } from '../shared';
import { FeatDetail } from './FeatDetail';

const TYPE_COLORS: Record<string, string> = {
  Combat: '#d84315', General: '#43a047', Class: '#1e88e5', Proficiency: '#6d4c41',
  Metamagic: '#8e24aa', Divine: '#f9a825', Background: '#00897b', Racial: '#00acc1',
  Epic: '#e53935',
};

type TabId = 'my' | 'all';

export function FeatsPanel() {
  const [tab, setTab] = useState<TabId>('my');
  const [search, setSearch] = useState('');
  const [activeType, setActiveType] = useState<string>('all');
  const [selectedFeat, setSelectedFeat] = useState<DummyFeat | null>(null);

  const clearFilters = useCallback(() => {
    setSearch('');
    setActiveType('all');
  }, []);

  const filterFeat = useCallback((f: DummyFeat) => {
    if (search.length >= 3 && !f.name.toLowerCase().includes(search.toLowerCase())) return false;
    if (activeType !== 'all' && f.type !== activeType) return false;
    return true;
  }, [search, activeType]);

  const mySections: ListSection<DummyFeat>[] = useMemo(() =>
    Object.entries(FEATS).map(([key, cat]) => ({
      key,
      title: cat.title,
      items: cat.feats.filter(filterFeat),
    })).filter(s => s.items.length > 0),
    [filterFeat]
  );

  const allSections: ListSection<DummyFeat>[] = useMemo(() => {
    const filtered = ALL_FEATS.filter(filterFeat);
    const grouped = new Map<string, DummyFeat[]>();
    for (const f of filtered) {
      if (!grouped.has(f.type)) grouped.set(f.type, []);
      grouped.get(f.type)!.push(f);
    }
    return [...grouped.entries()].map(([type, items]) => ({
      key: type,
      title: `${type} Feats`,
      items,
    }));
  }, [filterFeat]);

  const totalOwned = Object.values(FEATS).reduce((sum, cat) => sum + cat.feats.length, 0);
  const hasFilters = search.length > 0 || activeType !== 'all';
  const sections = tab === 'my' ? mySections : allSections;

  const renderFeatItem = useCallback((feat: DummyFeat, selected: boolean) => {
    const typeColor = TYPE_COLORS[feat.type] || T.textMuted;
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{
          color: T.text,
          fontWeight: selected ? 600 : 400,
          flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {feat.name}
        </span>
        <span style={{ color: typeColor, fontWeight: 500, flexShrink: 0 }}>
          {feat.type}
        </span>
      </div>
    );
  }, []);

  const typeLabel = activeType === 'all' ? 'Type: All' : activeType;

  const typeMenu = (
    <Menu>
      <MenuItem text="All" active={activeType === 'all'} onClick={() => setActiveType('all')} />
      {FEAT_TYPE_OPTIONS.map(t => (
        <MenuItem key={t} text={t} active={activeType === t} onClick={() => setActiveType(t)} />
      ))}
    </Menu>
  );

  const toolbar = (
    <>
      <Tabs
        id="feat-tabs" selectedTabId={tab}
        onChange={(id) => { setTab(id as TabId); setSelectedFeat(null); }}
        renderActiveTabPanelOnly
      >
        <Tab id="my" title={`My Feats (${totalOwned})`} />
        <Tab id="all" title={`All Feats (${ALL_FEATS.length})`} />
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

  const list = sections.length > 0 ? (
    <GroupedList
      sections={sections}
      selectedId={selectedFeat?.id ?? null}
      onSelect={setSelectedFeat}
      renderItem={renderFeatItem}
    />
  ) : (
    <div style={{ padding: 24, textAlign: 'center', color: T.textMuted }}>No feats match your filters.</div>
  );

  return (
    <SplitPane
      toolbar={toolbar}
      list={list}
      detail={<FeatDetail feat={selectedFeat} />}
    />
  );
}
