import { safeToNumber } from '@/utils/dataHelpers';
import type { FullInventorySummary, FullEquippedItem } from '@/lib/bindings';

export const INVENTORY_COLS = 8;
export const INVENTORY_ROWS = 8;

export const SLOT_MAPPING: Record<string, string> = {
  'helmet': 'head', 'head': 'head',
  'chest': 'chest',
  'belt': 'belt',
  'boots': 'boots',
  'neck': 'neck',
  'cloak': 'cloak',
  'gloves': 'gloves',
  'l ring': 'left_ring', 'left_ring': 'left_ring',
  'r ring': 'right_ring', 'right_ring': 'right_ring',
  'l hand': 'left_hand', 'left_hand': 'left_hand',
  'r hand': 'right_hand', 'right_hand': 'right_hand',
  'arrows': 'arrows', 'bullets': 'bullets', 'bolts': 'bolts',
};

export function mapSlotName(slot: string): string | null {
  return SLOT_MAPPING[slot.toLowerCase()] ?? null;
}

export interface GridItem {
  id: string;
  name: string;
  icon?: string;
  stackSize?: number;
  maxStack?: number;
  type: 'weapon' | 'armor' | 'accessory' | 'consumable' | 'misc';
  equipped?: boolean;
  slot?: string;
  defaultSlot?: string;
  rarity?: 'common' | 'uncommon' | 'rare' | 'epic' | 'legendary';
  enhancement_bonus?: number;
  charges?: number;
  is_custom?: boolean;
  is_identified?: boolean;
  is_plot?: boolean;
  is_cursed?: boolean;
  is_stolen?: boolean;
}

export function parseInventoryGrid(data: FullInventorySummary | null): (GridItem | null)[] {
  const grid: (GridItem | null)[] = Array(INVENTORY_COLS * INVENTORY_ROWS).fill(null);
  if (!data) return grid;

  const items = data.inventory || [];
  for (const item of items) {
    const idx = safeToNumber(item.index, -1);
    if (idx < 0 || idx >= grid.length) continue;

    grid[idx] = {
      id: `inventory_${idx}`,
      name: item.name || `Item ${item.base_item || 0}`,
      type: (item.category || 'misc') as GridItem['type'],
      rarity: item.is_custom ? 'legendary' : 'common',
      equipped: false,
      defaultSlot: item.default_slot || undefined,
      stackSize: item.stack_size > 1 ? item.stack_size : undefined,
      enhancement_bonus: item.enhancement || 0,
      charges: item.charges ?? undefined,
      is_custom: item.is_custom || false,
      is_identified: item.identified,
      is_plot: item.plot,
      is_cursed: item.cursed,
      is_stolen: item.stolen,
    };
  }

  return grid;
}

export function resolveEquipSlot(
  defaultSlot: string,
  equippableSlots: string[],
  equipped: FullEquippedItem[]
): string {
  if (equippableSlots.length <= 1) return defaultSlot;

  const defaultOccupied = equipped.some(e => e.slot.toLowerCase() === defaultSlot.toLowerCase());
  if (!defaultOccupied) return defaultSlot;

  const emptySlot = equippableSlots.find(slot =>
    !equipped.some(e => e.slot.toLowerCase() === slot.toLowerCase())
  );

  return emptySlot || defaultSlot;
}

export function getEquippedItemForSlot(
  data: FullInventorySummary | null,
  slotName: string
): { name: string; base_item: number; is_custom: boolean } | null {
  if (!data) return null;

  const mapped = mapSlotName(slotName) || slotName;
  const equipData = data.equipped?.find(e => e.slot.toLowerCase() === mapped.toLowerCase());
  if (!equipData) return null;

  return {
    name: equipData.name || `Item ${equipData.base_item}`,
    base_item: equipData.base_item,
    is_custom: equipData.custom || false,
  };
}
