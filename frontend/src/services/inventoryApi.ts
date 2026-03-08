import { invoke } from '@tauri-apps/api/core';

export interface EquipItemRequest {
  item_data: Record<string, unknown>;
  slot: string;
  inventory_index?: number;
}

export interface EquipItemResponse {
  success: boolean;
  warnings: string[];
  message: string;
  has_unsaved_changes: boolean;
}

export interface UnequipItemRequest {
  slot: string;
}

export interface UnequipItemResponse {
  success: boolean;
  item_data: Record<string, unknown> | null;
  message: string;
  has_unsaved_changes: boolean;
}

export interface UpdateGoldRequest {
  gold: number;
}

export interface UpdateGoldResponse {
  success: boolean;
  gold: number;
  message: string;
  has_unsaved_changes: boolean;
}

export interface DeleteItemResponse {
  success: boolean;
  item_data: Record<string, unknown> | null;
  message: string;
  has_unsaved_changes: boolean;
}

export interface UpdateItemRequest {
  item_index?: number;
  slot?: string;
  item_data: Record<string, unknown>;
}

export interface UpdateItemResponse {
  success: boolean;
  message: string;
  has_unsaved_changes: boolean;
}

export interface AddItemByBaseTypeRequest {
  base_item_id: number;
}

export interface AddToInventoryResponse {
  success: boolean;
  message: string;
  has_unsaved_changes: boolean;
  item_index?: number;
}

export interface PropertyMetadata {
  id: number;
  label: string;
  description: string;
  has_subtype: boolean;
  has_cost_table: boolean;
  has_param1: boolean;
  cost_table_idx?: number;
  param1_idx?: number;
  subtype_options?: Record<number, string>;
  cost_table_options?: Record<number, string>;
  param1_options?: Record<number, string>;
}

export interface ItemEditorMetadataResponse {
  property_types: PropertyMetadata[];
  abilities: Record<number, string>;
  skills: Record<number, string>;
  damage_types: Record<number, string>;
  alignment_groups: Record<number, string>;
  racial_groups: Record<number, string>;
  saving_throws: Record<number, string>;
  immunity_types: Record<number, string>;
  classes: Record<number, string>;
  spells: Record<number, string>;
}

export interface BaseItem {
  id: number;
  name: string;
  type: number;
  category: string;
}

export interface ItemTemplate {
  resref: string;
  name: string;
  base_item: number;
  category: number;
  source: string;
}

export const ITEM_CATEGORIES = {
  0: 'Armor & Clothing',
  1: 'Weapons',
  2: 'Magic Items',
  3: 'Accessories',
  4: 'Miscellaneous'
} as const;

export interface AddItemFromTemplateResponse {
    success: boolean;
    message: string;
    new_item: Record<string, unknown>;
}

interface AvailableBaseItem {
    id: number;
    name: string;
    item_class: string | null;
    weight: number | null;
    base_cost: number | null;
}

interface TemplateInfo {
    name?: string;
    base_item?: number;
    source?: string;
}

interface AddItemResult {
    success: boolean;
    message?: string;
    inventory_index?: number;
}

function mapItemClassToCategory(itemClass: string | null): string {
    if (!itemClass) return 'Miscellaneous';
    const cls = itemClass.toUpperCase();
    if (cls.includes('ARMOR') || cls.includes('CLOTH') || cls.includes('HELM') || cls.includes('BOOT') || cls.includes('GLOVE') || cls.includes('BELT') || cls.includes('CLOAK') || cls.includes('ROBE') || cls.includes('BRACER')) return 'Armor & Clothing';
    if (cls.includes('WEAPON') || cls.includes('SWORD') || cls.includes('AXE') || cls.includes('BOW') || cls.includes('DAGGER') || cls.includes('STAFF') || cls.includes('MACE') || cls.includes('HAMMER') || cls.includes('SPEAR') || cls.includes('HALBERD') || cls.includes('FLAIL') || cls.includes('CROSSBOW') || cls.includes('SLING') || cls.includes('ARROW') || cls.includes('BOLT') || cls.includes('BULLET') || cls.includes('THROWN')) return 'Weapons';
    if (cls.includes('RING') || cls.includes('AMULET') || cls.includes('NECK')) return 'Accessories';
    if (cls.includes('POTION') || cls.includes('SCROLL') || cls.includes('WAND') || cls.includes('ROD') || cls.includes('MAGIC')) return 'Magic Items';
    return 'Miscellaneous';
}

export class InventoryAPI {
  async equipItem(characterId: number, request: EquipItemRequest): Promise<EquipItemResponse> {
    // Rust equip_item takes (index, slot)
    // request.inventory_index is optional in TS but required for Rust logic typically
    // If inventory_index is missing, we might need a workaround or fail. 
    // Assuming UI always provides index.
    if (request.inventory_index === undefined) {
         throw new Error("Inventory index required for Rust backend");
    }
    
    // Map string slot to Rust enum equivalent (likely handled by serde if string matches)
    // or we pass enum variant index.
    // Assuming check logic in frontend or rust accepts string. 
    // Rust expects `EquipmentSlot` enum.
    try {
        const result = await invoke<any>('equip_item', { 
            inventoryIndex: request.inventory_index, 
            slot: request.slot 
        });
        return {
            success: result.success,
            warnings: result.warnings || [],
            message: result.message || "Item equipped",
            has_unsaved_changes: true
        };
    } catch (e) {
        throw new Error(String(e));
    }
  }

  async unequipItem(characterId: number, request: UnequipItemRequest): Promise<UnequipItemResponse> {
    try {
        const result = await invoke<any>('unequip_item', { slot: request.slot });
        return {
            success: result.success,
            item_data: null, // Rust doesn't return item data on unequip usually? check return type
            message: result.message || "Item unequipped",
            has_unsaved_changes: true
        };
    } catch (e) {
        throw new Error(String(e));
    }
  }

  async updateGold(characterId: number, gold: number): Promise<UpdateGoldResponse> {
      try {
          // set_gold returns new total
          const newTotal = await invoke<number>('set_gold', { amount: gold });
          return {
              success: true,
              gold: newTotal,
              message: "Gold updated",
              has_unsaved_changes: true
          };
      } catch (e) {
             throw new Error(String(e));
      }
  }

  async deleteItem(characterId: number, itemIndex: number): Promise<DeleteItemResponse> {
      try {
          await invoke('remove_from_inventory', { index: itemIndex });
          return {
              success: true,
              item_data: null,
              message: "Item removed",
              has_unsaved_changes: true
          };
      } catch (e) {
          throw new Error(String(e));
      }
  }

  async getEditorMetadata(_characterId: number): Promise<ItemEditorMetadataResponse> {
      try {
          const result = await invoke<ItemEditorMetadataResponse>('get_editor_metadata');
          return result;
      } catch (e) {
          console.error('Failed to get editor metadata:', e);
          return {
              property_types: [], abilities: {}, skills: {}, damage_types: {},
              alignment_groups: {}, racial_groups: {}, saving_throws: {},
              immunity_types: {}, classes: {}, spells: {}
          };
      }
  }

  async addItemByBaseType(characterId: number, request: AddItemByBaseTypeRequest): Promise<AddToInventoryResponse> {
      try {
          const result = await invoke<any>('add_to_inventory', { 
              baseItemId: request.base_item_id, 
              stackSize: 1 
          });
          return {
              success: result.success,
              message: result.message || "Item added",
              has_unsaved_changes: true,
              item_index: result.index // verify rust returns index
          };
      } catch (e) {
          throw new Error(String(e));
      }
  }

  async updateItem(characterId: number, request: UpdateItemRequest): Promise<UpdateItemResponse> {
    // TODO: Implement item property editing in Rust
    console.warn("updateItem not implemented in Rust backend yet");
    return { success: false, message: "Not implemented", has_unsaved_changes: false };
  }

  async getAllBaseItems(_characterId: number): Promise<{ base_items: BaseItem[] }> {
    try {
        const items = await invoke<AvailableBaseItem[]>('get_available_base_items');
        return {
            base_items: items.map(item => ({
                id: item.id,
                name: item.name,
                type: item.id,
                category: mapItemClassToCategory(item.item_class)
            }))
        };
    } catch (e) {
        console.error('Failed to get base items:', e);
        return { base_items: [] };
    }
  }


  async getAvailableTemplates(_characterId: number): Promise<{ templates: ItemTemplate[] }> {
    try {
        const templates = await invoke<ItemTemplate[]>('get_available_templates');
        return { templates };
    } catch (e) {
        console.error('Failed to get templates:', e);
        return { templates: [] };
    }
  }

  async addItemFromTemplate(_characterId: number, resref: string): Promise<AddItemFromTemplateResponse> {
      try {
          const result = await invoke<AddItemResult>('add_item_from_template', { resref });
          return {
              success: result.success,
              message: result.message || 'Item added',
              new_item: {}
          };
      } catch (e) {
          console.error('Failed to add item from template:', e);
          return { success: false, message: String(e), new_item: {} };
      }
  }
}

export const inventoryAPI = new InventoryAPI();
