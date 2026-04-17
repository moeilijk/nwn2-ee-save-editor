import { useCallback } from 'react';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { inventoryAPI, EquipItemRequest, UnequipItemRequest, UpdateItemRequest } from '@/services/inventoryApi';

const EQUIP_DEPENDENT_SUBSYSTEMS = ['abilityScores', 'combat', 'saves', 'skills'] as const;

export function useInventoryManagement() {
  const { character, invalidateSubsystems } = useCharacterContext();
  const inventoryData = useSubsystem('inventory');

  const equipItem = useCallback(async (request: EquipItemRequest) => {
    if (!character?.id) throw new Error('No character loaded');

    const response = await inventoryAPI.equipItem(character.id, request);
    if (response.success) {
      await inventoryData.load({ silent: true });
      await invalidateSubsystems([...EQUIP_DEPENDENT_SUBSYSTEMS]);
    }
    return response;
  }, [character?.id, inventoryData, invalidateSubsystems]);

  const unequipItem = useCallback(async (request: UnequipItemRequest) => {
    if (!character?.id) throw new Error('No character loaded');

    const response = await inventoryAPI.unequipItem(character.id, request);
    if (response.success) {
      await inventoryData.load({ silent: true });
      await invalidateSubsystems([...EQUIP_DEPENDENT_SUBSYSTEMS]);
    }
    return response;
  }, [character?.id, inventoryData, invalidateSubsystems]);

  const deleteItem = useCallback(async (itemIndex: number) => {
    if (!character?.id) throw new Error('No character loaded');

    const response = await inventoryAPI.deleteItem(character.id, itemIndex);
    if (response.success) {
      await inventoryData.load({ silent: true });
    }
    return response;
  }, [character?.id, inventoryData]);

  const addItemByBaseType = useCallback(async (baseItemId: number, iconTemplateResref?: string) => {
    if (!character?.id) throw new Error('No character loaded');

    const response = await inventoryAPI.addItemByBaseType(character.id, {
      base_item_id: baseItemId,
      icon_template_resref: iconTemplateResref,
    });
    if (response.success) {
      await inventoryData.load({ silent: true });
    }
    return response;
  }, [character?.id, inventoryData]);

  const updateItem = useCallback(async (request: UpdateItemRequest) => {
    if (!character?.id) throw new Error('No character loaded');

    const response = await inventoryAPI.updateItem(character.id, request);
    if (response.success) {
      await inventoryData.load({ silent: true });
      await invalidateSubsystems([...EQUIP_DEPENDENT_SUBSYSTEMS]);
    }
    return response;
  }, [character?.id, inventoryData, invalidateSubsystems]);

  const addItemFromTemplate = useCallback(async (resref: string) => {
    if (!character?.id) throw new Error('No character loaded');

    const response = await inventoryAPI.addItemFromTemplate(character.id, resref);
    if (response.success) {
      await inventoryData.load({ silent: true });
      await invalidateSubsystems([...EQUIP_DEPENDENT_SUBSYSTEMS]);
    }
    return response;
  }, [character?.id, inventoryData, invalidateSubsystems]);

  return {
    equipItem,
    unequipItem,
    deleteItem,
    addItemByBaseType,
    updateItem,
    addItemFromTemplate,
  };
}
