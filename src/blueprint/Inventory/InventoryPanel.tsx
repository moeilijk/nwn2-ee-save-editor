import { useState } from 'react';
import { Button, Card, Elevation, ProgressBar } from '@blueprintjs/core';
import { T, RARITY_COLORS } from '../theme';
import { CHARACTER, INVENTORY, BACKPACK } from '../dummy-data';
import { fmtNum } from '../shared';
import { ItemDetails, type AnyItem } from './ItemDetails';
import { AddItemDialog } from './AddItemDialog';
import { EditItemDialog } from './EditItemDialog';

type InventoryItem = typeof INVENTORY[number];

const SLOT_LABELS: Record<string, string> = {
  'Main Hand': 'MH',
  'Off Hand': 'OH',
  'Head': 'H',
  'Chest': 'Ch',
  'Hands': 'Gl',
  'Feet': 'Ft',
  'Cloak': 'Ck',
  'Ring 1': 'R1',
  'Ring 2': 'R2',
  'Belt': 'Bt',
  'Amulet': 'Nk',
};

const EQUIPMENT_GRID: (string | null)[][] = [
  [null, 'Head', 'Amulet', null],
  ['Main Hand', 'Chest', 'Cloak', 'Off Hand'],
  ['Ring 1', 'Belt', 'Hands', 'Ring 2'],
  [null, 'Feet', null, null],
];

const AMMO_SLOTS = ['Arrows', 'Bullets', 'Bolts'];

function getItemForSlot(slot: string): InventoryItem | undefined {
  return INVENTORY.find(item => item.slot === slot);
}

function EquipSlot({ slot, selected, onSelect }: { slot: string; selected: boolean; onSelect: () => void }) {
  const item = getItemForSlot(slot);
  const label = SLOT_LABELS[slot] || slot.charAt(0);
  const hasItem = !!item;

  return (
    <div
      onClick={onSelect}
      title={item ? `${item.name} (${slot})` : slot}
      style={{
        width: 44, height: 44,
        borderRadius: 4,
        border: `2px solid ${selected ? T.accent : hasItem ? T.border : T.borderLight}`,
        background: selected ? `${T.accent}15` : hasItem ? T.surface : T.surfaceAlt,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        cursor: hasItem ? 'pointer' : 'default',
        transition: 'all 0.15s',
        position: 'relative',
      }}
    >
      {hasItem ? (
        <div style={{
          width: 30, height: 30, borderRadius: 3,
          background: T.sectionBg,
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          fontWeight: 700, color: RARITY_COLORS[item.rarity] || T.text,
        }}>
          {item.name.charAt(0)}
        </div>
      ) : (
        <span style={{ fontWeight: 600, color: T.border }}>{label}</span>
      )}
    </div>
  );
}

function BackpackRow({ item, selected, onSelect }: {
  item: typeof BACKPACK[number];
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <div
      onClick={onSelect}
      style={{
        display: 'flex', alignItems: 'center', gap: 8,
        padding: '5px 8px',
        borderRadius: 3,
        background: selected ? `${T.accent}15` : 'transparent',
        cursor: 'pointer',
        transition: 'background 0.1s',
      }}
    >
      <div style={{
        width: 26, height: 26, borderRadius: 3, flexShrink: 0,
        background: T.sectionBg, border: `1px solid ${T.borderLight}`,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        fontWeight: 700, color: RARITY_COLORS[item.rarity] || T.text,
      }}>
        {item.name.charAt(0)}
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{
          fontWeight: 500, color: RARITY_COLORS[item.rarity] || T.text,
          overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {item.name}
        </div>
      </div>
      <span style={{ color: T.textMuted, flexShrink: 0 }}>
        {item.qty > 1 ? `x${item.qty}` : ''}
      </span>
      <span style={{ color: item.value > 0 ? T.gold : T.border, fontWeight: 600, flexShrink: 0, minWidth: 32, textAlign: 'right' }}>
        {item.value > 0 ? fmtNum(item.value) : '-'}
      </span>
    </div>
  );
}

export function InventoryPanel() {
  const [selectedItem, setSelectedItem] = useState<AnyItem | null>(INVENTORY[0] ?? null);
  const [goldInput, setGoldInput] = useState(CHARACTER.gold.toString());
  const [gold, setGold] = useState(CHARACTER.gold);
  const [addItemOpen, setAddItemOpen] = useState(false);
  const [editItemOpen, setEditItemOpen] = useState(false);

  const goldDirty = goldInput !== gold.toString();

  const handleGoldSubmit = () => {
    const val = parseInt(goldInput, 10);
    if (isNaN(val) || val < 0) return;
    setGold(Math.min(val, 99_999_999));
  };

  const handleGoldReset = () => setGoldInput(gold.toString());

  const totalWeight = INVENTORY.reduce((s, i) => s + i.weight, 0) + BACKPACK.reduce((s, i) => s + i.weight * i.qty, 0);
  const maxWeight = 200;
  const weightRatio = totalWeight / maxWeight;
  const totalItems = INVENTORY.length + BACKPACK.length;

  return (
    <div style={{ padding: 16, height: '100%' }}>
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden', display: 'flex', height: '100%' }}>

        <div style={{ width: 400, borderRight: `1px solid ${T.borderLight}`, display: 'flex', flexDirection: 'column' }}>

          <div style={{ padding: '12px 16px' }}>
            <div style={{ fontWeight: 700, color: T.accent, marginBottom: 10 }}>Equipment</div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 4, alignItems: 'center' }}>
              {EQUIPMENT_GRID.map((row, ri) => (
                <div key={ri} style={{ display: 'flex', gap: 4 }}>
                  {row.map((slot, ci) => {
                    if (!slot) return <div key={ci} style={{ width: 44, height: 44 }} />;
                    const item = getItemForSlot(slot);
                    return (
                      <EquipSlot
                        key={slot}
                        slot={slot}
                        selected={selectedItem !== null && 'slot' in selectedItem && selectedItem.slot === slot}
                        onSelect={() => item && setSelectedItem(item)}
                      />
                    );
                  })}
                </div>
              ))}

              <div style={{ display: 'flex', gap: 4, marginTop: 4, paddingTop: 8, borderTop: `1px solid ${T.borderLight}` }}>
                {AMMO_SLOTS.map(slot => {
                  const item = getItemForSlot(slot);
                  return (
                    <EquipSlot
                      key={slot}
                      slot={slot}
                      selected={selectedItem !== null && 'slot' in selectedItem && selectedItem.slot === slot}
                      onSelect={() => item && setSelectedItem(item)}
                    />
                  );
                })}
              </div>
            </div>
          </div>

          <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '12px 16px', flex: 1, overflow: 'auto', minHeight: 0 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 10 }}>
              <div style={{ fontWeight: 700, color: T.accent }}>
                Backpack <span style={{ fontWeight: 400, color: T.textMuted }}>({BACKPACK.length})</span>
              </div>
              <Button icon="add" small minimal text="Add item" style={{ color: T.textMuted }} onClick={() => setAddItemOpen(true)} />
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 8px 4px', marginBottom: 2, borderBottom: `1px solid ${T.borderLight}` }}>
              <div style={{ width: 26 }} />
              <div style={{ flex: 1, fontWeight: 600, color: T.textMuted }}>Item</div>
              <span style={{ fontWeight: 600, color: T.textMuted, flexShrink: 0 }}>Qty</span>
              <span style={{ fontWeight: 600, color: T.textMuted, flexShrink: 0, minWidth: 32, textAlign: 'right' }}>Value</span>
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
              {BACKPACK.map((item, i) => (
                <BackpackRow
                  key={i}
                  item={item}
                  selected={selectedItem === item}
                  onSelect={() => setSelectedItem(item)}
                />
              ))}
            </div>
          </div>

          <div style={{ padding: '10px 16px', borderTop: `1px solid ${T.borderLight}`, background: T.surfaceAlt }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <span style={{ color: weightRatio > 0.75 ? T.negative : T.textMuted }}>
                {totalWeight.toFixed(1)} / {maxWeight} lbs
              </span>
              <div style={{ flex: 1 }}>
                <ProgressBar
                  value={weightRatio}
                  intent={weightRatio > 0.9 ? 'danger' : weightRatio > 0.75 ? 'warning' : 'none'}
                  stripes={false}
                  animate={false}
                  style={{ height: 3 }}
                />
              </div>
              <span style={{ color: T.textMuted }}>{totalItems} items</span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span style={{ color: T.gold, fontWeight: 600 }}>Gold</span>
              <input
                type="text"
                value={goldInput}
                onChange={(e) => { if (e.target.value === '' || /^\d+$/.test(e.target.value)) setGoldInput(e.target.value); }}
                onKeyDown={(e) => { if (e.key === 'Enter') handleGoldSubmit(); if (e.key === 'Escape') handleGoldReset(); }}
                className="bp6-input"
                style={{ width: 100, textAlign: 'center', padding: '2px 8px', height: 24 }}
              />
              <Button minimal icon="tick" intent="success" onClick={handleGoldSubmit} disabled={!goldDirty} style={{ opacity: goldDirty ? 1 : 0.3 }} />
              <Button minimal icon="cross" onClick={handleGoldReset} disabled={!goldDirty} style={{ opacity: goldDirty ? 1 : 0.3 }} />
            </div>
          </div>
        </div>

        <div style={{ flex: 1, overflow: 'auto' }}>
          <ItemDetails item={selectedItem} onEdit={() => setEditItemOpen(true)} />
        </div>
      </Card>

      <AddItemDialog isOpen={addItemOpen} onClose={() => setAddItemOpen(false)} />
      <EditItemDialog
        isOpen={editItemOpen}
        onClose={() => setEditItemOpen(false)}
        itemName={selectedItem?.name}
      />
    </div>
  );
}
