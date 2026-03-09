import { useState } from 'react';
import { Button, InputGroup, ProgressBar, Tree, type TreeNodeInfo } from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER, INVENTORY, BACKPACK } from '../dummy-data';
import { fmtNum } from '../shared';
import { ItemDetails } from './ItemDetails';
import { BackpackTable } from './BackpackTable';

export function InventoryPanel() {
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [filter, setFilter] = useState('');
  const [listTab, setListTab] = useState<'equipped' | 'backpack'>('equipped');

  const selectedItem = INVENTORY[selectedIdx] ?? null;
  const totalWeight = INVENTORY.reduce((s, i) => s + i.weight, 0) + BACKPACK.reduce((s, i) => s + i.weight * i.qty, 0);
  const maxWeight = 200;

  const equipmentNodes: TreeNodeInfo[] = INVENTORY
    .filter(item => !filter || item.name.toLowerCase().includes(filter.toLowerCase()))
    .map((item, i) => ({
      id: i,
      label: <span style={{ fontSize: 13, color: T.text }}>{item.name}</span>,
      secondaryLabel: <span style={{ fontSize: 10, color: T.textMuted }}>{item.slot}</span>,
      icon: item.type === 'Weapon' ? 'ninja' as const : item.type === 'Armor' ? 'shield' as const : 'ring' as const,
      isSelected: i === selectedIdx,
    }));

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <div style={{ width: 360, borderRight: `1px solid ${T.border}`, display: 'flex', flexDirection: 'column', background: T.surfaceAlt }}>
        <div style={{ padding: '10px 12px' }}>
          <InputGroup
            leftIcon="search"
            placeholder="Filter items..."
            small
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            rightElement={filter ? <Button icon="cross" small minimal onClick={() => setFilter('')} /> : undefined}
          />
        </div>

        <div style={{ display: 'flex', alignItems: 'center', padding: '0 12px', borderBottom: `1px solid ${T.border}` }}>
          {(['equipped', 'backpack'] as const).map(tab => (
            <button
              key={tab}
              onClick={() => setListTab(tab)}
              style={{
                background: 'none', border: 'none', cursor: 'pointer',
                padding: '8px 12px 8px 0', fontSize: 13,
                fontWeight: listTab === tab ? 700 : 400,
                color: listTab === tab ? T.accent : T.textMuted,
                borderBottom: listTab === tab ? `2px solid ${T.accent}` : '2px solid transparent',
              }}
            >
              {tab === 'equipped' ? `Equipped (${INVENTORY.length})` : `Backpack (${BACKPACK.length})`}
            </button>
          ))}
          <div style={{ flex: 1 }} />
          <Button icon="add" small minimal style={{ color: T.textMuted }} />
        </div>

        <div style={{ flex: 1, overflow: 'auto' }}>
          {listTab === 'equipped' ? (
            <Tree contents={equipmentNodes} onNodeClick={(node) => setSelectedIdx(node.id as number)} />
          ) : (
            <BackpackTable />
          )}
        </div>

        <div style={{ padding: '8px 12px', borderTop: `1px solid ${T.border}`, background: T.sectionBg }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
            <span style={{ fontSize: 12, color: T.textMuted }}>{totalWeight.toFixed(1)} / {maxWeight} lbs</span>
            <span style={{ fontSize: 12, color: T.gold, fontWeight: 700 }}>{fmtNum(CHARACTER.gold)} gp</span>
          </div>
          <ProgressBar value={totalWeight / maxWeight} intent={totalWeight / maxWeight > 0.75 ? 'warning' : 'none'} stripes={false} animate={false} style={{ height: 3 }} />
        </div>
      </div>

      <div style={{ flex: 1, overflow: 'auto' }}>
        <ItemDetails item={selectedItem} />
      </div>
    </div>
  );
}
