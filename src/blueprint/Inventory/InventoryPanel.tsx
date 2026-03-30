import { useEffect, useState } from 'react';
import { Button, Card, Elevation, NonIdealState, ProgressBar, Spinner } from '@blueprintjs/core';
import { useTranslations } from '@/hooks/useTranslations';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { T } from '../theme';
import { fmtNum } from '../shared';
import { ItemDetails, makeEquippedItem, makeInventoryItem, type AnyItem } from './ItemDetails';
import { AddItemDialog } from './AddItemDialog';
import { EditItemDialog } from './EditItemDialog';
import { BackpackTable } from './BackpackTable';
import { useSubsystem } from '@/contexts/CharacterContext';
import { useInventoryManagement } from '@/hooks/useInventoryManagement';
import { inventoryAPI } from '@/services/inventoryApi';
import { useCharacterContext } from '@/contexts/CharacterContext';
import type { FullEquippedItem } from '@/lib/bindings';

const DISPLAY_SLOT_TO_API: Record<string, string> = {
  'Main Hand': 'right_hand',
  'Off Hand': 'left_hand',
  'Head': 'head',
  'Chest': 'chest',
  'Hands': 'gloves',
  'Feet': 'boots',
  'Cloak': 'cloak',
  'Ring 1': 'left_ring',
  'Ring 2': 'right_ring',
  'Belt': 'belt',
  'Amulet': 'neck',
  'Arrows': 'arrows',
  'Bullets': 'bullets',
  'Bolts': 'bolts',
};

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

function getEquippedItemForDisplaySlot(
  equippedItems: FullEquippedItem[],
  displaySlot: string,
): FullEquippedItem | undefined {
  const apiSlot = DISPLAY_SLOT_TO_API[displaySlot];
  if (!apiSlot) return undefined;
  return equippedItems.find(e => e.slot.toLowerCase() === apiSlot.toLowerCase());
}

interface EquipSlotProps {
  slot: string;
  equippedItem: FullEquippedItem | undefined;
  selected: boolean;
  onSelect: () => void;
}

function EquipSlot({ slot, equippedItem, selected, onSelect }: EquipSlotProps) {
  const label = SLOT_LABELS[slot] || slot.charAt(0);
  const hasItem = !!equippedItem;

  return (
    <div
      onClick={onSelect}
      title={equippedItem ? `${equippedItem.name} (${slot})` : slot}
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
          fontWeight: 700, color: T.accent,
        }}>
          {equippedItem.name.charAt(0)}
        </div>
      ) : (
        <span style={{ fontWeight: 600, color: T.border }}>{label}</span>
      )}
    </div>
  );
}

export function InventoryPanel() {
  const t = useTranslations();
  const { handleError } = useErrorHandler();
  const { character, characterId } = useCharacterContext();
  const inventorySubsystem = useSubsystem('inventory');
  const { unequipItem, deleteItem } = useInventoryManagement();

  const [selectedItem, setSelectedItem] = useState<AnyItem | null>(null);
  const [goldInput, setGoldInput] = useState('0');
  const [gold, setGold] = useState(0);
  const [addItemOpen, setAddItemOpen] = useState(false);
  const [editItemOpen, setEditItemOpen] = useState(false);

  const inventoryData = inventorySubsystem.data;

  useEffect(() => {
    if (!inventorySubsystem.data && !inventorySubsystem.isLoading && characterId) {
      inventorySubsystem.load();
    }
  }, [characterId, inventorySubsystem]);

  useEffect(() => {
    if (inventoryData) {
      const g = inventoryData.gold ?? character?.gold ?? 0;
      setGold(g);
      setGoldInput(g.toString());
    }
  }, [inventoryData, character?.gold]);

  const goldDirty = goldInput !== gold.toString();

  const handleGoldSubmit = async () => {
    const val = parseInt(goldInput, 10);
    if (isNaN(val) || val < 0) return;
    const clamped = Math.min(val, 99_999_999);
    try {
      await inventoryAPI.updateGold(characterId!, clamped);
      setGold(clamped);
      setGoldInput(clamped.toString());
      await inventorySubsystem.load({ silent: true });
    } catch (err) {
      handleError(err);
      setGoldInput(gold.toString());
    }
  };

  const handleGoldReset = () => setGoldInput(gold.toString());

  const handleUnequip = async (slot: string) => {
    try {
      await unequipItem({ slot });
      setSelectedItem(null);
    } catch (err) {
      handleError(err);
    }
  };

  const handleDelete = async (index: number) => {
    try {
      await deleteItem(index);
      setSelectedItem(null);
    } catch (err) {
      handleError(err);
    }
  };

  const equippedItems = inventoryData?.equipped ?? [];
  const backpackItems = inventoryData?.inventory ?? [];
  const encumbrance = inventoryData?.encumbrance;

  const totalWeight = encumbrance?.total_weight ?? 0;
  const maxWeight = encumbrance?.heavy_load ?? 200;
  const weightRatio = maxWeight > 0 ? Math.min(totalWeight / maxWeight, 1) : 0;
  const totalItems = equippedItems.length + backpackItems.length;

  if (inventorySubsystem.isLoading) {
    return (
      <div style={{ padding: 16, height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <Spinner />
      </div>
    );
  }

  if (inventorySubsystem.error) {
    return (
      <div style={{ padding: 16, height: '100%' }}>
        <NonIdealState
          icon="error"
          title="Failed to load inventory"
          description={inventorySubsystem.error}
          action={<Button onClick={() => inventorySubsystem.load()}>Retry</Button>}
        />
      </div>
    );
  }

  if (!inventoryData) {
    return (
      <div style={{ padding: 16, height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <NonIdealState icon="person" title="No character loaded" description="Load a character to view inventory." />
      </div>
    );
  }

  return (
    <div style={{ padding: 16, height: '100%' }}>
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden', display: 'flex', height: '100%' }}>

        <div style={{ width: 400, borderRight: `1px solid ${T.borderLight}`, display: 'flex', flexDirection: 'column' }}>

          <div style={{ padding: '12px 16px' }}>
            <div style={{ fontWeight: 700, color: T.accent, marginBottom: 10 }}>{t('inventory.equipment')}</div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 4, alignItems: 'center' }}>
              {EQUIPMENT_GRID.map((row, ri) => (
                <div key={ri} style={{ display: 'flex', gap: 4 }}>
                  {row.map((slot, ci) => {
                    if (!slot) return <div key={ci} style={{ width: 44, height: 44 }} />;
                    const equippedItem = getEquippedItemForDisplaySlot(equippedItems, slot);
                    return (
                      <EquipSlot
                        key={slot}
                        slot={slot}
                        equippedItem={equippedItem}
                        selected={
                          selectedItem !== null &&
                          selectedItem._kind === 'equipped' &&
                          selectedItem.slot === DISPLAY_SLOT_TO_API[slot]
                        }
                        onSelect={() => {
                          if (equippedItem) setSelectedItem(makeEquippedItem(equippedItem));
                        }}
                      />
                    );
                  })}
                </div>
              ))}

              <div style={{ display: 'flex', gap: 4, marginTop: 4, paddingTop: 8, borderTop: `1px solid ${T.borderLight}` }}>
                {AMMO_SLOTS.map(slot => {
                  const equippedItem = getEquippedItemForDisplaySlot(equippedItems, slot);
                  return (
                    <EquipSlot
                      key={slot}
                      slot={slot}
                      equippedItem={equippedItem}
                      selected={
                        selectedItem !== null &&
                        selectedItem._kind === 'equipped' &&
                        selectedItem.slot === DISPLAY_SLOT_TO_API[slot]
                      }
                      onSelect={() => {
                        if (equippedItem) setSelectedItem(makeEquippedItem(equippedItem));
                      }}
                    />
                  );
                })}
              </div>
            </div>
          </div>

          <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '12px 16px', flex: 1, overflow: 'auto', minHeight: 0 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 10 }}>
              <div style={{ fontWeight: 700, color: T.accent }}>
                Backpack <span style={{ fontWeight: 400, color: T.textMuted }}>({backpackItems.length})</span>
              </div>
              <Button
                icon="add"
                small
                minimal
                text={t('inventory.addItem')}
                style={{ color: T.textMuted }}
                onClick={() => setAddItemOpen(true)}
              />
            </div>
            <BackpackTable
              items={backpackItems}
              selectedIndex={selectedItem?._kind === 'inventory' ? (selectedItem as unknown as { index: number }).index : null}
              onSelectItem={(index) => {
                const item = backpackItems.find(i => i.index === index);
                if (item) setSelectedItem(makeInventoryItem(item));
              }}
            />
          </div>

          <div style={{ padding: '10px 16px', borderTop: `1px solid ${T.borderLight}`, background: T.surfaceAlt }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <span style={{ color: weightRatio > 0.75 ? T.negative : T.textMuted }}>
                {totalWeight.toFixed(1)} / {maxWeight} lbs
                {encumbrance && (
                  <span style={{ marginLeft: 6, color: T.textMuted }}>({encumbrance.encumbrance_level})</span>
                )}
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
              <span style={{ color: T.gold, fontWeight: 600 }}>{t('inventory.gold')}</span>
              <input
                type="text"
                value={goldInput}
                onChange={(e) => { if (e.target.value === '' || /^\d+$/.test(e.target.value)) setGoldInput(e.target.value); }}
                onKeyDown={(e) => { if (e.key === 'Enter') handleGoldSubmit(); if (e.key === 'Escape') handleGoldReset(); }}
                className="bp6-input"
                style={{ width: 100, textAlign: 'center', padding: '2px 8px', height: 24 }}
              />
              <span style={{ color: T.textMuted, fontSize: 12 }}>{fmtNum(gold)} gp</span>
              <Button minimal icon="tick" intent="success" onClick={handleGoldSubmit} disabled={!goldDirty} style={{ opacity: goldDirty ? 1 : 0.3 }} />
              <Button minimal icon="cross" onClick={handleGoldReset} disabled={!goldDirty} style={{ opacity: goldDirty ? 1 : 0.3 }} />
            </div>
          </div>
        </div>

        <div style={{ flex: 1, overflow: 'auto' }}>
          <ItemDetails
            item={selectedItem}
            onEdit={() => setEditItemOpen(true)}
            onUnequip={handleUnequip}
            onDelete={handleDelete}
          />
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
