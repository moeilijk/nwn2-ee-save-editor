#![allow(dead_code)]
pub mod context_maps;
pub mod error;
pub mod property_types;

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::{debug, warn};

use crate::services::resource_manager::ResourceManager;

pub use context_maps::*;
pub use error::{ItemPropertyError, ItemPropertyResult};
pub use property_types::{DecodedProperty, PropertyDefinition, PropertyMetadata};

/// Context data for the item editor, containing pre-resolved lookup tables.
/// This mirrors Python's inventory_manager.get_item_editor_metadata() context.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct EditorContext {
    pub abilities: HashMap<u32, String>,
    pub skills: HashMap<u32, String>,
    pub spells: HashMap<u32, String>,
    pub damage_types: HashMap<u32, String>,
    pub immunity_types: HashMap<u32, String>,
    pub saving_throws: HashMap<u32, String>,
    pub save_elements: HashMap<u32, String>,
    pub classes: HashMap<u32, String>,
    pub racial_groups: HashMap<u32, String>,
    pub alignment_groups: HashMap<u32, String>,
    pub alignments: HashMap<u32, String>,
    pub light: HashMap<u32, String>,
    pub feats: HashMap<u32, String>,
}

/// Maps SubTypeResRef values to context keys.
/// Based on Python item_property_decoder.py lines 694-753 subtype_map.
fn get_subtype_context_key(subtype_ref: &str) -> Option<&'static str> {
    match subtype_ref.to_lowercase().as_str() {
        "ability" | "decreaseabilityscore" | "abilitybonus" => Some("abilities"),
        "skill" | "decreasedskill" => Some("skills"),
        "castspell" | "spellimmunity_specific" | "onhitcastspell" => Some("spells"),
        "damagetype" | "armordamagetype" | "damageresist" | "damageimmunity"
        | "damageimmunity_fixed" | "damagepenalty" | "damage_vulnerability"
        | "damage_vulnerability_fixed" | "damagemelee" | "damageranged" | "damage"
        | "damagereduced" | "damagenone" | "damage_reduction" | "damagereduction"
        | "massive_criticals" => Some("damage_types"),
        "immunity" => Some("immunity_types"),
        "savingthrow" | "improvedsavingthrowsspecific" | "reducedspecificsavingthrow"
        | "reducedspecificsaving_throw" => Some("saving_throws"),
        "saveselement" | "improvedsavingthrows" | "reducedsavingthrows" => Some("save_elements"),
        "uselimitationclass" | "classes" | "singlebonusspellofle" => Some("classes"),
        "uselimitationracial" | "racialtype" | "racialtypes" | "armorracinggroup"
        | "armorracialgroup" | "enhancementracialgroup" | "damageracialgroup"
        | "attackbonusracialgroup" | "damageracialtype" => Some("racial_groups"),
        "armoralignmentgroup" | "damagealignmentgroup" | "enhancementalignmentgroup"
        | "attackbonusalignmentgroup" | "uselimitationalignmentgroup" => Some("alignment_groups"),
        "armorspecificalignment" | "damagespecificalignment" | "enhancementspecificalignment"
        | "attackbonusspecificalignment" | "uselimitationspecificalignment"
        | "specificalignment" => Some("alignments"),
        "bonusfeats" => Some("feats"),
        "light" => Some("light"),
        _ => None,
    }
}

#[allow(dead_code)]
const PROPERTY_ID_ABILITY_BONUS: u32 = 0;
const PROPERTY_ID_AC_BONUS: u32 = 1;
const PROPERTY_ID_ENHANCEMENT: u32 = 6;
const PROPERTY_ID_ATTACK_PENALTY: u32 = 10;
const PROPERTY_ID_BONUS_FEAT: u32 = 12;
const PROPERTY_ID_BONUS_SPELL_SLOT: u32 = 13;
const PROPERTY_ID_CAST_SPELL: u32 = 15;
const PROPERTY_ID_DAMAGE_BONUS: u32 = 16;
const PROPERTY_ID_DAMAGE_RESISTANCE: u32 = 23;
const PROPERTY_ID_DAMAGE_VULNERABILITY: u32 = 24;
const PROPERTY_ID_ABILITY_PENALTY: u32 = 27;
const PROPERTY_ID_HASTE: u32 = 35;
const PROPERTY_ID_IMMUNITY: u32 = 37;
const PROPERTY_ID_SPELL_RESISTANCE: u32 = 39;
const PROPERTY_ID_SAVING_THROW_BONUS: u32 = 40;
const PROPERTY_ID_SAVING_THROW_VS: u32 = 41;
const PROPERTY_ID_LIGHT: u32 = 44;
const PROPERTY_ID_REGENERATION: u32 = 51;
const PROPERTY_ID_SKILL_BONUS: u32 = 52;
const PROPERTY_ID_ATTACK_BONUS: u32 = 56;
const PROPERTY_ID_USE_LIMIT_ALIGNMENT: u32 = 62;
const PROPERTY_ID_USE_LIMIT_CLASS: u32 = 63;
const PROPERTY_ID_TRAP: u32 = 70;
const PROPERTY_ID_TRUE_SEEING: u32 = 71;
const PROPERTY_ID_FREEDOM_OF_MOVEMENT: u32 = 75;
const PROPERTY_ID_WEIGHT_MODIFIER: u32 = 81;
const PROPERTY_ID_DAMAGE_REDUCTION: u32 = 90;

pub struct ItemPropertyDecoder {
    rm: Arc<tokio::sync::RwLock<ResourceManager>>,
    property_cache: HashMap<u32, PropertyDefinition>,
    initialized: bool,
}

impl ItemPropertyDecoder {
    pub fn new(rm: Arc<tokio::sync::RwLock<ResourceManager>>) -> Self {
        Self {
            rm,
            property_cache: HashMap::new(),
            initialized: false,
        }
    }

    pub async fn initialize(&mut self) -> ItemPropertyResult<()> {
        if self.initialized {
            return Ok(());
        }

        self.load_property_definitions().await?;
        self.initialized = true;
        debug!(
            "ItemPropertyDecoder initialized with {} property definitions",
            self.property_cache.len()
        );

        Ok(())
    }

    async fn load_property_definitions(&mut self) -> ItemPropertyResult<()> {
        let rm = self.rm.read().await;

        let itempropdef = match rm.get_2da("itempropdef") {
            Ok(parser) => parser,
            Err(e) => {
                warn!("Failed to load itempropdef.2da: {}", e);
                return Ok(());
            }
        };

        for row_idx in 0..itempropdef.row_count() {
            if let Ok(row) = itempropdef.get_row_dict(row_idx) {
                let id = row_idx as u32;

                let label = row
                    .get("Label")
                    .or_else(|| row.get("label"))
                    .and_then(std::clone::Clone::clone)
                    .unwrap_or_default();

                if label.is_empty() || is_invalid_label(&label) {
                    continue;
                }

                let subtype_ref = row
                    .get("SubTypeResRef")
                    .or_else(|| row.get("subtyperesref"))
                    .and_then(std::clone::Clone::clone)
                    .filter(|s| !s.is_empty() && s != "****");

                let cost_table_ref = row
                    .get("CostTableResRef")
                    .or_else(|| row.get("costtableresref"))
                    .and_then(std::clone::Clone::clone)
                    .filter(|s| !s.is_empty() && s != "****");

                let param1_ref = row
                    .get("Param1ResRef")
                    .or_else(|| row.get("param1resref"))
                    .and_then(std::clone::Clone::clone)
                    .filter(|s| !s.is_empty() && s != "****");

                let game_str_ref = row
                    .get("GameStrRef")
                    .or_else(|| row.get("gamestrref"))
                    .and_then(|v| v.as_ref())
                    .and_then(|s| s.parse::<i32>().ok());

                let description = if let Some(str_ref) = game_str_ref {
                    rm.get_string(str_ref)
                } else {
                    String::new()
                };

                let display_label = get_property_label_override(id)
                    .map(std::string::ToString::to_string)
                    .unwrap_or_else(|| clean_label(&label));

                self.property_cache.insert(
                    id,
                    PropertyDefinition {
                        id,
                        label: display_label,
                        subtype_ref,
                        cost_table_ref,
                        param1_ref,
                        description,
                        game_str_ref,
                        raw_label: label,
                        raw_name: None,
                    },
                );
            }
        }

        Ok(())
    }

    pub fn decode_property(
        &self,
        property_data: &HashMap<String, serde_json::Value>,
    ) -> Option<DecodedProperty> {
        let property_id = get_u32(property_data, "PropertyName")?;
        let subtype = get_u32(property_data, "Subtype").unwrap_or(0);
        let cost_value = get_u32(property_data, "CostValue").unwrap_or(0);
        let param1_value = get_u32(property_data, "Param1Value").unwrap_or(0);

        let raw_data = property_data.clone();

        let decoded = match property_id {
            PROPERTY_ID_ABILITY_BONUS => {
                property_types::decode_ability_bonus(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_ABILITY_PENALTY => {
                property_types::decode_ability_penalty(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_AC_BONUS => {
                property_types::decode_ac_bonus(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_ENHANCEMENT => {
                property_types::decode_enhancement_bonus(cost_value, raw_data)
            }
            PROPERTY_ID_ATTACK_BONUS => {
                property_types::decode_attack_bonus(cost_value, raw_data)
            }
            PROPERTY_ID_DAMAGE_BONUS => {
                property_types::decode_damage_bonus(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_DAMAGE_RESISTANCE => {
                property_types::decode_damage_resistance(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_DAMAGE_VULNERABILITY => {
                property_types::decode_damage_vulnerability(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_SAVING_THROW_BONUS => {
                property_types::decode_saving_throw_bonus(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_SAVING_THROW_VS => {
                property_types::decode_saving_throw_vs_element(subtype, cost_value, raw_data)
            }
            PROPERTY_ID_SKILL_BONUS => {
                let skill_name = format!("Skill_{subtype}");
                property_types::decode_skill_bonus(&skill_name, cost_value, raw_data)
            }
            PROPERTY_ID_IMMUNITY => property_types::decode_immunity(subtype, raw_data),
            PROPERTY_ID_CAST_SPELL => {
                let spell_name = format!("Spell_{subtype}");
                property_types::decode_cast_spell(&spell_name, cost_value, param1_value, raw_data)
            }
            PROPERTY_ID_BONUS_FEAT => {
                let feat_name = format!("Feat_{subtype}");
                property_types::decode_bonus_feat(&feat_name, raw_data)
            }
            PROPERTY_ID_SPELL_RESISTANCE => {
                property_types::decode_spell_resistance(cost_value, raw_data)
            }
            PROPERTY_ID_REGENERATION => {
                property_types::decode_regeneration(cost_value, raw_data)
            }
            PROPERTY_ID_FREEDOM_OF_MOVEMENT => {
                property_types::decode_freedom_of_movement(raw_data)
            }
            PROPERTY_ID_HASTE => property_types::decode_haste(raw_data),
            PROPERTY_ID_TRUE_SEEING => property_types::decode_true_seeing(raw_data),
            PROPERTY_ID_USE_LIMIT_CLASS => {
                let class_name = format!("Class_{subtype}");
                property_types::decode_use_limitation_class(&class_name, raw_data)
            }
            PROPERTY_ID_USE_LIMIT_ALIGNMENT => {
                let alignment = ALIGNMENT_GROUP_MAP
                    .get(&subtype)
                    .copied()
                    .unwrap_or("Unknown");
                property_types::decode_use_limitation_alignment(alignment, raw_data)
            }
            PROPERTY_ID_DAMAGE_REDUCTION => {
                let bypass = format!("Bypass_{param1_value}");
                property_types::decode_damage_reduction(cost_value, &bypass, raw_data)
            }
            _ => {
                let label = self
                    .property_cache
                    .get(&property_id)
                    .map(|def| def.label.clone())
                    .unwrap_or_else(|| format!("Property {property_id}"));

                property_types::decode_generic(property_id, &label, raw_data)
            }
        };

        Some(decoded)
    }

    pub fn decode_all_properties(
        &self,
        properties: &[HashMap<String, serde_json::Value>],
    ) -> Vec<DecodedProperty> {
        properties
            .iter()
            .filter_map(|prop| self.decode_property(prop))
            .collect()
    }

    pub fn get_item_bonuses(
        &self,
        properties: &[HashMap<String, serde_json::Value>],
        base_item_id: i32,
    ) -> ItemBonuses {
        let decoded = self.decode_all_properties(properties);
        let mut bonuses = ItemBonuses::default();

        let is_armor = base_item_id == 16;
        let is_shield = [14, 56, 57].contains(&base_item_id);

        for prop in decoded {
            match prop.bonus_type.as_str() {
                "ability" => {
                    if let (Some(ability), Some(value)) = (&prop.ability, prop.bonus_value) {
                        match ability.as_str() {
                            "Str" => bonuses.str_bonus += value,
                            "Dex" => bonuses.dex_bonus += value,
                            "Con" => bonuses.con_bonus += value,
                            "Int" => bonuses.int_bonus += value,
                            "Wis" => bonuses.wis_bonus += value,
                            "Cha" => bonuses.cha_bonus += value,
                            _ => {}
                        }
                    }
                }
                "ability_penalty" => {
                    if let (Some(ability), Some(value)) = (&prop.ability, prop.penalty_value) {
                        match ability.as_str() {
                            "Str" => bonuses.str_bonus -= value,
                            "Dex" => bonuses.dex_bonus -= value,
                            "Con" => bonuses.con_bonus -= value,
                            "Int" => bonuses.int_bonus -= value,
                            "Wis" => bonuses.wis_bonus -= value,
                            "Cha" => bonuses.cha_bonus -= value,
                            _ => {}
                        }
                    }
                }
                "ac" => {
                    if let (Some(ac_type), Some(value)) = (&prop.ac_type, prop.bonus_value) {
                        match ac_type.as_str() {
                            "Armor" => bonuses.ac_armor_bonus += value,
                            "Shield" => bonuses.ac_shield_bonus += value,
                            "Natural" => bonuses.ac_natural_bonus += value,
                            "Deflection" => bonuses.ac_deflection_bonus += value,
                            "Dodge" => bonuses.ac_dodge_bonus += value,
                            _ => bonuses.ac_bonus += value,
                        }
                    } else if let Some(value) = prop.bonus_value {
                        bonuses.ac_bonus += value;
                    }
                }
                "enhancement" => {
                    if let Some(value) = prop.bonus_value {
                        if is_armor {
                            bonuses.ac_armor_bonus += value;
                        } else if is_shield {
                            bonuses.ac_shield_bonus += value;
                        } else {
                            bonuses.attack_bonus += value;
                        }
                    }
                }
                "attack" => {
                    if let Some(value) = prop.bonus_value {
                        bonuses.attack_bonus += value;
                    }
                }
                "saving_throw" => {
                    if let (Some(save_type), Some(value)) = (&prop.save_type, prop.bonus_value) {
                        match save_type.as_str() {
                            "Fortitude" => bonuses.fortitude_bonus += value,
                            "Reflex" => bonuses.reflex_bonus += value,
                            "Will" => bonuses.will_bonus += value,
                            _ => {}
                        }
                    }
                }
                "skill" => {
                    if let (Some(skill), Some(value)) = (&prop.skill_name, prop.bonus_value) {
                        *bonuses.skill_bonuses.entry(skill.clone()).or_insert(0) += value;
                    }
                }
                "spell_resistance" => {
                    if let Some(value) = prop.bonus_value {
                        bonuses.spell_resistance = bonuses.spell_resistance.max(value);
                    }
                }
                "damage_resistance" => {
                    if let (Some(dtype), Some(value)) =
                        (&prop.damage_type, prop.resistance_value)
                    {
                        *bonuses
                            .damage_resistances
                            .entry(dtype.clone())
                            .or_insert(0) = (*bonuses
                            .damage_resistances
                            .get(dtype)
                            .unwrap_or(&0))
                        .max(value);
                    }
                }
                _ => {}
            }
        }

        bonuses
    }

    /// Load options from a 2DA table, mirroring Python `_get_iprp_table_options()`.
    /// Handles TLK string resolution and label cleanup.
    pub async fn load_2da_options(&self, ref_name: &str) -> HashMap<u32, String> {
        let rm = self.rm.read().await;
        let mut options = HashMap::new();

        let Ok(table) = rm.get_2da(ref_name) else {
            return options;
        };

        for row_idx in 0..table.row_count() {
            let Ok(row) = table.get_row_dict(row_idx) else {
                continue;
            };

            // Try Name column with TLK lookup first (Python lines 992-1004)
            let name_str_ref = row
                .get("Name")
                .or_else(|| row.get("name"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .filter(|&n| n > 100); // TLK refs typically > 100

            let name = name_str_ref.map(|str_ref| rm.get_string(str_ref))
                .filter(|s| !s.is_empty());

            // Try GameString column (Python lines 1011-1018)
            let game_str = name.clone().or_else(|| {
                row.get("GameString")
                    .or_else(|| row.get("gamestring"))
                    .and_then(|v| v.as_ref())
                    .and_then(|s| s.parse::<i32>().ok())
                    .map(|str_ref| rm.get_string(str_ref))
                    .filter(|s| !s.is_empty())
            });

            // Fallback to Label column (Python lines 1020-1046)
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

    /// Resolve indirect cost table reference via iprp_costtable.2da.
    async fn resolve_cost_table(&self, cost_table_idx: &str) -> Option<String> {
        let idx: usize = cost_table_idx.parse().ok()?;
        let rm = self.rm.read().await;
        let mapping = rm.get_2da("iprp_costtable").ok()?;
        let row = mapping.get_row_dict(idx).ok()?;
        row.get("Name")
            .or_else(|| row.get("name"))
            .and_then(|v| v.clone())
    }

    /// Resolve indirect param table reference via iprp_paramtable.2da.
    async fn resolve_param_table(&self, param_idx: &str) -> Option<String> {
        let idx: usize = param_idx.parse().ok()?;
        let rm = self.rm.read().await;
        let mapping = rm.get_2da("iprp_paramtable").ok()?;
        let row = mapping.get_row_dict(idx).ok()?;
        row.get("TableResRef")
            .or_else(|| row.get("tableresref"))
            .and_then(|v| v.clone())
    }

    /// Get editor property metadata with context-based option population.
    /// Mirrors Python item_property_decoder.get_editor_property_metadata().
    pub async fn get_editor_property_metadata(
        &self,
        context: &EditorContext,
    ) -> Vec<PropertyMetadata> {
        let mut metadata = Vec::new();

        for (id, def) in &self.property_cache {
            if is_hidden_property(*id) {
                continue;
            }

            let mut meta = PropertyMetadata {
                id: *id,
                label: def.label.clone(),
                original_label: def.raw_label.clone(),
                description: def.description.clone(),
                ..Default::default()
            };

            // Resolve subtype options (Python lines 816-832)
            if let Some(ref subtype_ref) = def.subtype_ref {
                let mapping_key = get_subtype_context_key(subtype_ref);

                meta.subtype_options = if let Some(key) = mapping_key {
                    match key {
                        "abilities" => context.abilities.clone(),
                        "skills" => context.skills.clone(),
                        "spells" => context.spells.clone(),
                        "damage_types" => context.damage_types.clone(),
                        "immunity_types" => context.immunity_types.clone(),
                        "saving_throws" => context.saving_throws.clone(),
                        "save_elements" => context.save_elements.clone(),
                        "classes" => context.classes.clone(),
                        "racial_groups" => context.racial_groups.clone(),
                        "alignment_groups" => context.alignment_groups.clone(),
                        "alignments" => context.alignments.clone(),
                        "light" => context.light.clone(),
                        "feats" => context.feats.clone(),
                        _ => self.load_2da_options(subtype_ref).await,
                    }
                } else {
                    self.load_2da_options(subtype_ref).await
                };

                meta.has_subtype = !meta.subtype_options.is_empty();
            }

            // Resolve cost table options via iprp_costtable indirection
            if let Some(ref cost_ref) = def.cost_table_ref {
                if let Some(table_name) = self.resolve_cost_table(cost_ref).await {
                    meta.cost_table_options = self.load_2da_options(&table_name).await;
                    meta.has_cost_table = !meta.cost_table_options.is_empty();
                }
            }

            // Resolve param1 options via iprp_paramtable indirection
            if let Some(ref param_ref) = def.param1_ref {
                if let Some(table_name) = self.resolve_param_table(param_ref).await {
                    // Some param tables map to context types
                    let param_key = get_subtype_context_key(&table_name);
                    meta.param1_options = if let Some(key) = param_key {
                        match key {
                            "racial_groups" => context.racial_groups.clone(),
                            "classes" => context.classes.clone(),
                            _ => self.load_2da_options(&table_name).await,
                        }
                    } else {
                        self.load_2da_options(&table_name).await
                    };
                    meta.has_param1 = !meta.param1_options.is_empty();
                }
            }

            meta.is_flat = !meta.has_subtype && !meta.has_cost_table && !meta.has_param1;
            metadata.push(meta);
        }

        metadata.sort_by(|a, b| a.label.cmp(&b.label));
        metadata
    }

    pub fn get_property_definition(&self, id: u32) -> Option<&PropertyDefinition> {
        self.property_cache.get(&id)
    }

    /// Synchronous version of get_editor_property_metadata for use in sync contexts.
    /// Only populates subtype_options from context; cost_table and param1 options
    /// would need separate pre-loading for full support.
    pub fn get_editor_property_metadata_sync(&self, context: &EditorContext) -> Vec<PropertyMetadata> {
        let mut metadata = Vec::new();

        for (id, def) in &self.property_cache {
            if is_hidden_property(*id) {
                continue;
            }

            let mut meta = PropertyMetadata {
                id: *id,
                label: def.label.clone(),
                original_label: def.raw_label.clone(),
                description: def.description.clone(),
                ..Default::default()
            };

            // Resolve subtype options from pre-loaded context
            if let Some(ref subtype_ref) = def.subtype_ref {
                let mapping_key = get_subtype_context_key(subtype_ref);

                meta.subtype_options = if let Some(key) = mapping_key {
                    match key {
                        "abilities" => context.abilities.clone(),
                        "skills" => context.skills.clone(),
                        "spells" => context.spells.clone(),
                        "damage_types" => context.damage_types.clone(),
                        "immunity_types" => context.immunity_types.clone(),
                        "saving_throws" => context.saving_throws.clone(),
                        "save_elements" => context.save_elements.clone(),
                        "classes" => context.classes.clone(),
                        "racial_groups" => context.racial_groups.clone(),
                        "alignment_groups" => context.alignment_groups.clone(),
                        "alignments" => context.alignments.clone(),
                        "light" => context.light.clone(),
                        "feats" => context.feats.clone(),
                        _ => HashMap::new(), // Can't load 2DA in sync context
                    }
                } else {
                    HashMap::new() // Can't load 2DA in sync context
                };

                meta.has_subtype = !meta.subtype_options.is_empty();
            }

            // Note: cost_table and param1 options require async 2DA loading
            // For sync version, these remain empty unless pre-loaded into context
            meta.is_flat = !meta.has_subtype && !meta.has_cost_table && !meta.has_param1;
            metadata.push(meta);
        }

        metadata.sort_by(|a, b| a.label.cmp(&b.label));
        metadata
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ItemBonuses {
    pub str_bonus: i32,
    pub dex_bonus: i32,
    pub con_bonus: i32,
    pub int_bonus: i32,
    pub wis_bonus: i32,
    pub cha_bonus: i32,
    pub ac_bonus: i32,
    pub ac_armor_bonus: i32,
    pub ac_shield_bonus: i32,
    pub ac_deflection_bonus: i32,
    pub ac_natural_bonus: i32,
    pub ac_dodge_bonus: i32,
    pub attack_bonus: i32,
    pub damage_bonus: i32,
    pub fortitude_bonus: i32,
    pub reflex_bonus: i32,
    pub will_bonus: i32,
    pub spell_resistance: i32,
    pub skill_bonuses: HashMap<String, i32>,
    pub damage_resistances: HashMap<String, i32>,
}

fn get_u32(data: &HashMap<String, serde_json::Value>, key: &str) -> Option<u32> {
    data.get(key).and_then(|v| {
        v.as_u64()
            .map(|n| n as u32)
            .or_else(|| v.as_i64().map(|n| n as u32))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_u32() {
        let mut data = HashMap::new();
        data.insert(
            "PropertyName".to_string(),
            serde_json::Value::Number(serde_json::Number::from(5u64)),
        );

        assert_eq!(get_u32(&data, "PropertyName"), Some(5));
        assert_eq!(get_u32(&data, "Missing"), None);
    }

    #[test]
    fn test_item_bonuses_default() {
        let bonuses = ItemBonuses::default();
        assert_eq!(bonuses.str_bonus, 0);
        assert_eq!(bonuses.ac_bonus, 0);
    }
}
