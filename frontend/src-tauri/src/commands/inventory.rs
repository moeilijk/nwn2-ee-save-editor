use crate::character::{
    BasicItemInfo, FullInventorySummary, AddItemResult, EquipResult,
    RemoveItemResult, UnequipResult, EquipmentSlot, InventoryItem,
    EncumbranceInfo, BaseItemId, ItemProficiencyInfo,
};
use crate::commands::{CommandError, CommandResult};
use crate::state::AppState;
use crate::services::item_property_decoder::{
    EditorContext, ItemBonuses, PropertyMetadata,
    ABILITY_MAP, SAVE_MAP, DAMAGE_TYPE_MAP, IMMUNITY_TYPE_MAP,
    SAVE_ELEMENT_MAP, ALIGNMENT_GROUP_MAP, ALIGNMENT_MAP, LIGHT_MAP,
};
use crate::parsers::gff::GffValue;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use tauri::State;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedTemplate {
    pub resref: String,
    pub name: String,
    pub base_item: i32,
    pub category: i32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquippedItem {
    pub slot: EquipmentSlot,
    pub item: InventoryItem,
    pub bonuses: ItemBonuses,
}

#[tauri::command]
pub async fn get_inventory_items(state: State<'_, AppState>) -> CommandResult<Vec<BasicItemInfo>> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.inventory_items())
}

#[tauri::command]
pub async fn get_equipped_items(
    state: State<'_, AppState>,
) -> CommandResult<Vec<(usize, BasicItemInfo)>> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.equipped_items())
}

#[tauri::command]
pub async fn get_inventory_summary(state: State<'_, AppState>) -> CommandResult<FullInventorySummary> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let decoder = &session.item_property_decoder;
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_full_inventory_summary(&game_data, decoder))
}

#[tauri::command]
pub async fn get_gold(state: State<'_, AppState>) -> CommandResult<u32> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.gold())
}

#[tauri::command]
pub async fn set_gold(state: State<'_, AppState>, amount: u32) -> CommandResult<u32> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    character.set_gold(amount);
    Ok(character.gold())
}

#[tauri::command]
pub async fn add_gold(state: State<'_, AppState>, amount: i32) -> CommandResult<u32> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    let current = character.gold();
    let new_amount = if amount < 0 {
        current.saturating_sub(amount.unsigned_abs())
    } else {
        current.saturating_add(amount as u32)
    };
    character.set_gold(new_amount);
    Ok(new_amount)
}

#[tauri::command]
pub async fn get_equipment_bonuses(state: State<'_, AppState>) -> CommandResult<ItemBonuses> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let decoder = &session.item_property_decoder;
    Ok(character.get_equipment_bonuses(&game_data, decoder))
}

#[tauri::command]
pub async fn get_item_proficiency_info(
    state: State<'_, AppState>,
    base_item_id: i32,
) -> CommandResult<ItemProficiencyInfo> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_item_proficiency_info(base_item_id, &game_data))
}

#[tauri::command]
pub async fn calculate_total_weight(state: State<'_, AppState>) -> CommandResult<f32> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.calculate_total_weight(&game_data))
}

#[tauri::command]
pub async fn equip_item(
    state: State<'_, AppState>,
    inventory_index: usize,
    slot: EquipmentSlot,
) -> CommandResult<EquipResult> {
    let game_data = state.game_data.read();
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.equip_item(inventory_index, slot, &game_data)?)
}

#[tauri::command]
pub async fn unequip_item(
    state: State<'_, AppState>,
    slot: EquipmentSlot,
) -> CommandResult<UnequipResult> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.unequip_item(slot)?)
}

#[tauri::command]
pub async fn add_to_inventory(
    state: State<'_, AppState>,
    base_item_id: i32,
    stack_size: i32,
) -> CommandResult<AddItemResult> {
    let game_data = state.game_data.read();
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.add_item(base_item_id, stack_size, &game_data)?)
}

#[tauri::command]
pub async fn remove_from_inventory(
    state: State<'_, AppState>,
    index: usize,
) -> CommandResult<RemoveItemResult> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.remove_item(index)?)
}

#[tauri::command]
pub async fn calculate_encumbrance(
    state: State<'_, AppState>,
) -> CommandResult<EncumbranceInfo> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_encumbrance_info(&game_data))
}

#[tauri::command]
pub async fn get_equipped_item(
    state: State<'_, AppState>,
    slot: EquipmentSlot,
) -> CommandResult<Option<EquippedItem>> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;

    if let Some(basic_info) = character.get_equipped_item_by_slot(slot) {
        Ok(Some(EquippedItem {
            slot,
            item: InventoryItem {
                index: 0,
                base_item_id: BaseItemId(basic_info.base_item),
                name: basic_info.tag.clone(),
                tag: basic_info.tag,
                stack_size: basic_info.stack_size,
                identified: basic_info.identified,
            },
            bonuses: ItemBonuses::default(),
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn get_available_templates(
    state: State<'_, AppState>,
) -> CommandResult<Vec<IndexedTemplate>> {
    let resource_manager = state.resource_manager.read().await;
    let game_data = state.game_data.read();
    let templates = resource_manager.get_all_item_templates();

    let baseitems = game_data.get_table("baseitems");
    let tag_regex = Regex::new(r"<[^>]+>").unwrap();

    let mut indexed: Vec<IndexedTemplate> = templates
        .iter()
        .filter_map(|(resref, info)| {
            let fields = resource_manager.get_item_template_fields(info).ok()?;

            let base_item = fields.get("BaseItem")
                .and_then(|v| match v {
                    GffValue::Int(i) => Some(*i),
                    GffValue::Short(s) => Some(*s as i32),
                    GffValue::Byte(b) => Some(*b as i32),
                    _ => None,
                })
                .unwrap_or(0);

            let name = if let Some(GffValue::LocString(ls)) = fields.get("LocalizedName") {
                if !ls.substrings.is_empty() {
                    Some(ls.substrings[0].string.to_string())
                } else if ls.string_ref >= 0 {
                    game_data.get_string(ls.string_ref)
                } else {
                    None
                }
            } else {
                None
            };

            let name = name.unwrap_or_else(|| resref.replace(".uti", ""));
            let name = tag_regex.replace_all(&name, "").to_string();

            let category = baseitems
                .and_then(|t| t.get_by_id(base_item))
                .and_then(|row| {
                    row.get("StorePanel")
                        .or_else(|| row.get("storepanel"))
                        .and_then(|v| v.as_ref())
                        .and_then(|s| s.parse::<i32>().ok())
                })
                .unwrap_or(4);

            let source = format!("{:?}", info.source);

            Some(IndexedTemplate {
                resref: resref.clone(),
                name,
                base_item,
                category,
                source,
            })
        })
        .collect();

    indexed.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(indexed)
}

#[tauri::command]
pub async fn add_item_from_template(
    state: State<'_, AppState>,
    resref: String,
) -> CommandResult<AddItemResult> {
    // Load template first (async operation)
    let item_fields = {
        let rm = state.resource_manager.read().await;
        let templates = rm.get_all_item_templates();
        let template_info = templates.get(&resref).cloned().ok_or_else(|| {
            CommandError::NotFound {
                item: format!("Item template '{resref}'"),
            }
        })?;
        rm.get_item_template_fields(&template_info).map_err(|e| {
            CommandError::Internal(format!("Failed to load item template: {e}"))
        })?
    };

    // Now acquire sync locks and add item
    let game_data = state.game_data.read();
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;

    Ok(character.add_item_from_struct(item_fields, &game_data)?)
}

/// Response for the item editor metadata endpoint.
/// Mirrors Python's ItemEditorMetadataResponse.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemEditorMetadataResponse {
    pub property_types: Vec<PropertyMetadata>,
    pub abilities: HashMap<u32, String>,
    pub skills: HashMap<u32, String>,
    pub damage_types: HashMap<u32, String>,
    pub alignment_groups: HashMap<u32, String>,
    pub alignments: HashMap<u32, String>,
    pub racial_groups: HashMap<u32, String>,
    pub saving_throws: HashMap<u32, String>,
    pub save_elements: HashMap<u32, String>,
    pub immunity_types: HashMap<u32, String>,
    pub classes: HashMap<u32, String>,
    pub spells: HashMap<u32, String>,
    pub light: HashMap<u32, String>,
}

/// Helper to convert static &'static str maps to HashMap<u32, String>.
fn static_map_to_hashmap(map: &HashMap<u32, &'static str>) -> HashMap<u32, String> {
    map.iter().map(|(k, v)| (*k, (*v).to_string())).collect()
}

/// Load skills from game data skills.2da table.
fn load_skills_from_game_data(
    game_data: &crate::loaders::GameData,
) -> HashMap<u32, String> {
    let mut skills = HashMap::new();
    let Some(skill_table) = game_data.get_table("skills") else {
        return skills;
    };

    for row_idx in 0..skill_table.row_count() {
        let Ok(row) = skill_table.get_row(row_idx) else {
            continue;
        };

        // Try Name column with TLK lookup
        let name: Option<String> = row
            .get("Name")
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .and_then(|str_ref| game_data.get_string(str_ref));

        // Fallback to Label
        let label = name.or_else(|| {
            row.get("Label")
                .and_then(|v| v.clone())
                .filter(|s| !s.is_empty() && !s.starts_with("DEL_"))
        });

        if let Some(skill_name) = label {
            skills.insert(row_idx as u32, skill_name);
        }
    }
    skills
}

/// Load classes from game data classes.2da table.
fn load_classes_from_game_data(
    game_data: &crate::loaders::GameData,
) -> HashMap<u32, String> {
    let mut classes = HashMap::new();
    let Some(class_table) = game_data.get_table("classes") else {
        return classes;
    };

    for row_idx in 0..class_table.row_count() {
        let Ok(row) = class_table.get_row(row_idx) else {
            continue;
        };

        let name: Option<String> = row
            .get("Name")
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .and_then(|str_ref| game_data.get_string(str_ref));

        let label = name.or_else(|| {
            row.get("Label")
                .and_then(|v| v.clone())
                .filter(|s| !s.is_empty() && !s.starts_with("DEL_"))
        });

        if let Some(class_name) = label {
            classes.insert(row_idx as u32, class_name);
        }
    }
    classes
}

/// Load racial types from game data racialtypes.2da table.
fn load_racial_groups_from_game_data(
    game_data: &crate::loaders::GameData,
) -> HashMap<u32, String> {
    let mut races = HashMap::new();
    let Some(race_table) = game_data.get_table("racialtypes") else {
        return races;
    };

    for row_idx in 0..race_table.row_count() {
        let Ok(row) = race_table.get_row(row_idx) else {
            continue;
        };

        let name: Option<String> = row
            .get("Name")
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .and_then(|str_ref| game_data.get_string(str_ref));

        let label = name.or_else(|| {
            row.get("Label")
                .and_then(|v| v.clone())
                .filter(|s| !s.is_empty() && !s.starts_with("DEL_"))
        });

        if let Some(race_name) = label {
            races.insert(row_idx as u32, race_name);
        }
    }
    races
}

/// Load 2DA options from resource manager.
fn load_2da_options_from_rm(
    rm: &crate::services::resource_manager::ResourceManager,
    table_name: &str,
) -> HashMap<u32, String> {
    use crate::services::item_property_decoder::{clean_label, is_invalid_label};

    let mut options = HashMap::new();
    let Ok(table) = rm.get_2da(table_name) else {
        return options;
    };

    for row_idx in 0..table.row_count() {
        let Ok(row) = table.get_row_dict(row_idx) else {
            continue;
        };

        // Try Name column with TLK lookup
        let name = row
            .get("Name")
            .or_else(|| row.get("name"))
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&n| n > 100)
            .map(|str_ref| rm.get_string(str_ref))
            .filter(|s| !s.is_empty());

        // Try GameString column
        let game_str = name.clone().or_else(|| {
            row.get("GameString")
                .or_else(|| row.get("gamestring"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .map(|str_ref| rm.get_string(str_ref))
                .filter(|s| !s.is_empty())
        });

        // Fallback to Label column
        let label = game_str.or_else(|| {
            row.get("Label")
                .or_else(|| row.get("label"))
                .and_then(|v| v.clone())
                .filter(|s| !s.is_empty() && s != "****")
        });

        if let Some(display_name) = label {
            if !is_invalid_label(&display_name) {
                options.insert(row_idx as u32, clean_label(&display_name));
            }
        }
    }
    options
}

/// Get editor metadata for the item property editor.
/// Mirrors Python inventory_manager.get_item_editor_metadata().
#[tauri::command]
pub fn get_editor_metadata(state: State<'_, AppState>) -> CommandResult<ItemEditorMetadataResponse> {
    // Build static context from hardcoded maps
    let abilities = static_map_to_hashmap(&ABILITY_MAP);
    let saving_throws = static_map_to_hashmap(&SAVE_MAP);
    let damage_types = static_map_to_hashmap(&DAMAGE_TYPE_MAP);
    let immunity_types = static_map_to_hashmap(&IMMUNITY_TYPE_MAP);
    let save_elements = static_map_to_hashmap(&SAVE_ELEMENT_MAP);
    let alignment_groups = static_map_to_hashmap(&ALIGNMENT_GROUP_MAP);
    let alignments = static_map_to_hashmap(&ALIGNMENT_MAP);
    let light = static_map_to_hashmap(&LIGHT_MAP);

    // Load dynamic context from game data
    let (skills, classes, racial_groups) = {
        let game_data = state.game_data.read();
        (
            load_skills_from_game_data(&game_data),
            load_classes_from_game_data(&game_data),
            load_racial_groups_from_game_data(&game_data),
        )
    };

    // Load spells from iprp_spells.2da via resource manager
    let spells = {
        let rm = state.resource_manager.blocking_read();
        load_2da_options_from_rm(&rm, "iprp_spells")
    };

    // Build context for property metadata generation
    let context = EditorContext {
        abilities: abilities.clone(),
        skills: skills.clone(),
        spells: spells.clone(),
        damage_types: damage_types.clone(),
        immunity_types: immunity_types.clone(),
        saving_throws: saving_throws.clone(),
        save_elements: save_elements.clone(),
        classes: classes.clone(),
        racial_groups: racial_groups.clone(),
        alignment_groups: alignment_groups.clone(),
        alignments: alignments.clone(),
        light: light.clone(),
        feats: HashMap::new(),
    };

    // Get property metadata from decoder's property_cache
    let property_types = {
        let session = state.session.read();
        session.item_property_decoder.get_editor_property_metadata_sync(&context)
    };

    Ok(ItemEditorMetadataResponse {
        property_types,
        abilities,
        skills,
        damage_types,
        alignment_groups,
        alignments,
        racial_groups,
        saving_throws,
        save_elements,
        immunity_types,
        classes,
        spells,
        light,
    })
}
