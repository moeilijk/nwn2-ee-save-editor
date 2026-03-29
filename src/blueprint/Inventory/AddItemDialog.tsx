import { useState, useMemo, useCallback, useRef, useEffect } from 'react';
import { Button, InputGroup, Menu, MenuItem, Popover, Tab, Tabs, Tag } from '@blueprintjs/core';
import { FixedSizeList as List } from 'react-window';
import { ParchmentDialog } from '../shared';
import { T } from '../theme';

interface BaseItem {
  id: number;
  name: string;
  category: string;
}

const DUMMY_BASE_ITEMS: BaseItem[] = [
  { id: 1, name: 'Longsword', category: 'Weapons' },
  { id: 2, name: 'Shortsword', category: 'Weapons' },
  { id: 3, name: 'Greataxe', category: 'Weapons' },
  { id: 4, name: 'Dagger', category: 'Weapons' },
  { id: 5, name: 'Kama', category: 'Weapons' },
  { id: 6, name: 'Longbow', category: 'Weapons' },
  { id: 7, name: 'Light Crossbow', category: 'Weapons' },
  { id: 8, name: 'Throwing Axe', category: 'Weapons' },
  { id: 10, name: 'Full Plate', category: 'Armor & Clothing' },
  { id: 11, name: 'Chain Shirt', category: 'Armor & Clothing' },
  { id: 12, name: 'Leather Armor', category: 'Armor & Clothing' },
  { id: 13, name: 'Monk Robe', category: 'Armor & Clothing' },
  { id: 14, name: 'Tower Shield', category: 'Armor & Clothing' },
  { id: 15, name: 'Helmet', category: 'Armor & Clothing' },
  { id: 16, name: 'Boots', category: 'Armor & Clothing' },
  { id: 20, name: 'Ring', category: 'Magic Items' },
  { id: 21, name: 'Amulet', category: 'Magic Items' },
  { id: 22, name: 'Belt', category: 'Magic Items' },
  { id: 23, name: 'Cloak', category: 'Magic Items' },
  { id: 24, name: 'Gloves', category: 'Magic Items' },
  { id: 30, name: 'Potion', category: 'Miscellaneous' },
  { id: 31, name: 'Scroll', category: 'Miscellaneous' },
  { id: 32, name: 'Gem', category: 'Miscellaneous' },
  { id: 33, name: 'Trap Kit', category: 'Miscellaneous' },
];

interface Template {
  resref: string;
  name: string;
  category: string;
  source: string;
}

const DUMMY_TEMPLATES: Template[] = [
  { resref: 'nw_wswls012', name: 'Longsword +3', category: 'Weapons', source: 'Campaign' },
  { resref: 'nw_wswgs010', name: 'Greatsword +2', category: 'Weapons', source: 'Campaign' },
  { resref: 'nw_waxgr011', name: 'Greataxe +1', category: 'Weapons', source: 'Campaign' },
  { resref: 'nw_arhe005', name: 'Helmet of Intellect', category: 'Armor & Clothing', source: 'Campaign' },
  { resref: 'nw_arhe010', name: 'Helm of Brilliance', category: 'Armor & Clothing', source: 'Campaign' },
  { resref: 'nw_it_mring021', name: 'Ring of Protection +3', category: 'Magic Items', source: 'Base' },
  { resref: 'nw_it_mring022', name: 'Ring of Regeneration +2', category: 'Magic Items', source: 'Base' },
  { resref: 'nw_it_mpotion001', name: 'Potion of Cure Light Wounds', category: 'Miscellaneous', source: 'Base' },
  { resref: 'nw_it_mpotion021', name: 'Potion of Heal', category: 'Miscellaneous', source: 'Base' },
  { resref: 'nw_it_gem001', name: 'Emerald', category: 'Miscellaneous', source: 'Base' },
];

const CATEGORIES = ['Weapons', 'Armor & Clothing', 'Magic Items', 'Miscellaneous'];

const LIST_HEIGHT = 420;
const ROW_HEIGHT_BASE = 46;
const ROW_HEIGHT_TEMPLATE = 46;

type TabId = 'base' | 'template';

interface AddItemDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AddItemDialog({ isOpen, onClose }: AddItemDialogProps) {
  const [tab, setTab] = useState<TabId>('base');
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState<string>('all');
  const [selectedBaseId, setSelectedBaseId] = useState<number | null>(null);
  const [selectedResref, setSelectedResref] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [listWidth, setListWidth] = useState(680);

  useEffect(() => {
    if (containerRef.current) {
      setListWidth(containerRef.current.offsetWidth);
    }
  }, [isOpen]);

  const clearFilters = useCallback(() => {
    setSearch('');
    setCategory('all');
  }, []);

  const hasFilters = search.length > 0 || category !== 'all';
  const categoryLabel = category === 'all' ? 'Category: All' : category;

  const filteredBase = useMemo(() => {
    let items = DUMMY_BASE_ITEMS;
    if (category !== 'all') items = items.filter(i => i.category === category);
    if (search.trim()) {
      const q = search.toLowerCase();
      items = items.filter(i => i.name.toLowerCase().includes(q));
    }
    return items;
  }, [search, category]);

  const filteredTemplates = useMemo(() => {
    let items = DUMMY_TEMPLATES;
    if (category !== 'all') items = items.filter(i => i.category === category);
    if (search.trim()) {
      const q = search.toLowerCase();
      items = items.filter(i => i.name.toLowerCase().includes(q) || i.resref.toLowerCase().includes(q));
    }
    return items;
  }, [search, category]);

  const canAdd = tab === 'base' ? selectedBaseId !== null : selectedResref !== null;

  const categoryMenu = (
    <Menu>
      <MenuItem text="All" active={category === 'all'} onClick={() => setCategory('all')} />
      {CATEGORIES.map(cat => (
        <MenuItem key={cat} text={cat} active={category === cat} onClick={() => setCategory(cat)} />
      ))}
    </Menu>
  );

  const BaseRow = useCallback(({ index, style }: { index: number; style: React.CSSProperties }) => {
    const item = filteredBase[index];
    const selected = selectedBaseId === item.id;
    return (
      <div
        style={{
          ...style,
          display: 'flex', alignItems: 'center', gap: 8,
          padding: '0 12px', cursor: 'pointer',
          background: selected ? `${T.accent}12` : 'transparent',
          borderLeft: selected ? `2px solid ${T.accent}` : '2px solid transparent',
          borderBottom: `1px solid ${T.borderLight}`,
        }}
        onClick={() => setSelectedBaseId(item.id)}
      >
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ color: T.text, fontWeight: selected ? 600 : 400, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{item.name}</div>
          <div style={{ fontSize: 11, color: T.textMuted }}>{item.category}</div>
        </div>
      </div>
    );
  }, [filteredBase, selectedBaseId]);

  const TemplateRow = useCallback(({ index, style }: { index: number; style: React.CSSProperties }) => {
    const item = filteredTemplates[index];
    const selected = selectedResref === item.resref;
    return (
      <div
        style={{
          ...style,
          display: 'flex', alignItems: 'center', gap: 8,
          padding: '0 12px', cursor: 'pointer',
          background: selected ? `${T.accent}12` : 'transparent',
          borderLeft: selected ? `2px solid ${T.accent}` : '2px solid transparent',
          borderBottom: `1px solid ${T.borderLight}`,
        }}
        onClick={() => setSelectedResref(item.resref)}
      >
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ color: T.text, fontWeight: selected ? 600 : 400, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{item.name}</div>
          <div style={{ fontSize: 11, color: T.textMuted }}>{item.category} - {item.source}</div>
        </div>
      </div>
    );
  }, [filteredTemplates, selectedResref]);

  const itemCount = tab === 'base' ? filteredBase.length : filteredTemplates.length;

  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={() => { setSearch(''); setSelectedBaseId(null); setSelectedResref(null); setCategory('all'); setTab('base'); }}
      title="Add Item"
      width={720}
      minHeight={540}
      footerActions={
        <Button intent="primary" text="Add Item" disabled={!canAdd} onClick={onClose} />
      }
      footerLeft={
        <span style={{ fontSize: 11, color: T.textMuted }}>
          {tab === 'base'
            ? `Selected: ${selectedBaseId !== null ? DUMMY_BASE_ITEMS.find(i => i.id === selectedBaseId)?.name : 'None'}`
            : `Selected: ${selectedResref || 'None'}`
          }
        </span>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ padding: '6px 0', borderBottom: `1px solid ${T.borderLight}`, display: 'flex', alignItems: 'center', gap: 8, flexShrink: 0 }}>
          <Tabs
            id="add-item-tabs" selectedTabId={tab}
            onChange={(id) => { setTab(id as TabId); clearFilters(); setSelectedBaseId(null); setSelectedResref(null); }}
            renderActiveTabPanelOnly
          >
            <Tab id="base" title={`Base Type (${DUMMY_BASE_ITEMS.length})`} />
            <Tab id="template" title={`Template (${DUMMY_TEMPLATES.length})`} />
          </Tabs>
          <Popover content={categoryMenu} placement="bottom-start" minimal>
            <Button minimal rightIcon="caret-down" text={categoryLabel} />
          </Popover>
          <InputGroup
            leftIcon="search" placeholder="Filter items..." value={search}
            onChange={e => setSearch(e.target.value)}
            rightElement={search ? <Button icon="cross" minimal onClick={() => setSearch('')} /> : undefined}
            style={{ maxWidth: 200 }}
          />
          <Button minimal icon="filter-remove" text="Clear" onClick={clearFilters} disabled={!hasFilters} />
        </div>

        <div ref={containerRef} style={{ flex: 1, minHeight: LIST_HEIGHT }}>
          {itemCount > 0 ? (
            <List
              height={LIST_HEIGHT}
              itemCount={itemCount}
              itemSize={tab === 'base' ? ROW_HEIGHT_BASE : ROW_HEIGHT_TEMPLATE}
              width={listWidth}
            >
              {tab === 'base' ? BaseRow : TemplateRow}
            </List>
          ) : (
            <div style={{ padding: 32, textAlign: 'center', color: T.textMuted }}>No items match your filters.</div>
          )}
        </div>
      </div>
    </ParchmentDialog>
  );
}
