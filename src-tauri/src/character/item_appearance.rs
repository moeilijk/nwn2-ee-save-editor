use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::{debug, info};

use crate::loaders::GameData;
use crate::parsers::gff::GffValue;
use crate::services::resource_manager::ResourceManager;
use crate::utils::parsing::row_str;

use super::appearance_helpers::{TintChannels, read_tint_from_tintable, resolve_armor_prefix};
use super::gff_helpers::gff_value_to_i32;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemAppearance {
    /// Weapon parts (ModelPart1, 2, 3) or Armor Variation
    pub variation: i32,
    /// For weapons: ModelPart1, 2, 3 values.
    /// For armor: These might be used for accessories later.
    pub model_parts: [i32; 3],
    /// Tints for the item
    pub tints: TintChannels,
    /// Armor Visual Type (if applicable)
    pub armor_visual_type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemAppearanceOptions {
    pub available_variations: Vec<i32>,
    pub available_part1: Vec<i32>,
    pub available_part2: Vec<i32>,
    pub available_part3: Vec<i32>,
}

impl ItemAppearance {
    pub fn from_gff(fields: &IndexMap<String, GffValue<'_>>) -> Self {
        let variation = fields
            .get("Variation")
            .and_then(gff_value_to_i32)
            .unwrap_or(1);
        let model_parts = [
            fields
                .get("ModelPart1")
                .and_then(gff_value_to_i32)
                .unwrap_or(1),
            fields
                .get("ModelPart2")
                .and_then(gff_value_to_i32)
                .unwrap_or(1),
            fields
                .get("ModelPart3")
                .and_then(gff_value_to_i32)
                .unwrap_or(1),
        ];

        let tints = fields
            .get("Tintable")
            .and_then(|v| match v {
                GffValue::StructOwned(s) => Some(s.as_ref().clone()),
                GffValue::Struct(lazy) => Some(lazy.force_load()),
                _ => None,
            })
            .map(|t| read_tint_from_tintable(&t))
            .unwrap_or_default();

        let armor_visual_type = fields.get("ArmorVisualType").and_then(gff_value_to_i32);

        Self {
            variation,
            model_parts,
            tints,
            armor_visual_type,
        }
    }

    pub fn get_options(
        base_item_id: i32,
        game_data: &GameData,
        resource_manager: &ResourceManager,
    ) -> ItemAppearanceOptions {
        let Some(table) = game_data.get_table("baseitems") else {
            return ItemAppearanceOptions::new_empty();
        };

        let Some(row) = table.get_by_id(base_item_id) else {
            return ItemAppearanceOptions::new_empty();
        };

        let (prefix, source) = Self::resolve_item_prefix(&row);
        let model_type_str = row_str(&row, "modeltype")
            .unwrap_or_default()
            .to_uppercase();

        info!(
            "Resolving appearance options for item {base_item_id}: prefix='{prefix}' from {source}"
        );

        let full_prefix = Self::normalize_weapon_prefix(&prefix);

        // Weapon parts discovery: {FullPrefix}{NN}_{letter}.mdb
        // Actual NWN2 naming, e.g. W_Lsword01_a / W_Lsword01_b / W_Lsword01_c
        let part1 = Self::discover_variants(resource_manager, &full_prefix, "_a");
        let part2 = Self::discover_variants(resource_manager, &full_prefix, "_b");
        let part3 = Self::discover_variants(resource_manager, &full_prefix, "_c");

        let variations = if model_type_str == "A" || model_type_str == "I" {
            (1..=50).collect()
        } else {
            // Simple items (e.g. W_Arrow01, W_Bolt01): {FullPrefix}{NN}.mdb
            Self::discover_variants(resource_manager, &full_prefix, "")
        };

        ItemAppearanceOptions {
            available_variations: variations,
            available_part1: part1,
            available_part2: part2,
            available_part3: part3,
        }
    }

    fn discover_variants(
        resource_manager: &ResourceManager,
        prefix: &str,
        suffix: &str,
    ) -> Vec<i32> {
        let prefix_lower = prefix.to_lowercase();
        let suffix_lower = suffix.to_lowercase();
        let mdbs = resource_manager.list_resources_by_prefix(&prefix_lower, "mdb");
        debug!(
            "discover_variants: searching prefix '{}' suffix '{}', found {} MDBs",
            prefix_lower,
            suffix_lower,
            mdbs.len()
        );

        let mut variants: Vec<i32> = mdbs
            .iter()
            .filter_map(|name| {
                let stem = name.trim_end_matches(".mdb");
                let after_prefix = stem.strip_prefix(&prefix_lower)?;
                let num_str = if suffix_lower.is_empty() {
                    after_prefix
                } else {
                    after_prefix.strip_suffix(&suffix_lower)?
                };
                let val = num_str.parse::<i32>().ok();
                if let Some(v) = val {
                    debug!("discover_variants: found variant {} from name {}", v, stem);
                }
                val
            })
            .collect();
        variants.sort_unstable();
        variants.dedup();
        variants
    }

    fn normalize_weapon_prefix(prefix: &str) -> String {
        if prefix.to_uppercase().starts_with("W_") {
            prefix.to_string()
        } else {
            format!("W_{prefix}")
        }
    }

    fn resolve_item_prefix(
        row: &ahash::AHashMap<String, Option<String>>,
    ) -> (String, &'static str) {
        if let Some(ic) = row_str(row, "itemclass") {
            return (ic, "itemclass");
        }
        if let Some(lb) = row_str(row, "label") {
            return (lb, "label");
        }
        if let Some(mt) = row_str(row, "modeltype") {
            return (mt, "modeltype");
        }
        (String::new(), "none")
    }

    /// Resolve the resref for a single weapon part (0=a, 1=b, 2=c) at a given variant number.
    /// Returns None if the base item has no usable prefix.
    pub fn resolve_weapon_part_resref(
        base_item_id: i32,
        part_index: usize,
        variant: i32,
        game_data: &GameData,
    ) -> Option<String> {
        let letter = match part_index {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            _ => return None,
        };
        let table = game_data.get_table("baseitems")?;
        let row = table.get_by_id(base_item_id)?;
        let (prefix, _) = Self::resolve_item_prefix(&row);
        if prefix.is_empty() {
            return None;
        }
        let full_prefix = Self::normalize_weapon_prefix(&prefix);
        Some(format!("{full_prefix}{variant:02}_{letter}"))
    }

    pub fn resolve_model_resrefs(&self, base_item_id: i32, game_data: &GameData) -> Vec<String> {
        let Some(table) = game_data.get_table("baseitems") else {
            return Vec::new();
        };
        let Some(row) = table.get_by_id(base_item_id) else {
            return Vec::new();
        };

        let (prefix, source) = Self::resolve_item_prefix(&row);
        let model_type_str = row_str(&row, "modeltype")
            .unwrap_or_default()
            .to_uppercase();

        info!("Resolving model for item {base_item_id}: prefix='{prefix}' from {source}");

        if prefix.is_empty() {
            return Vec::new();
        }
        let full_prefix = Self::normalize_weapon_prefix(&prefix);

        let mut resrefs = Vec::new();

        // 3-part weapons (ModelType 2 in NWN2): W_Lsword01_a / _b / _c
        if model_type_str != "A" && model_type_str != "I" {
            resrefs.push(format!("{full_prefix}{:02}_a", self.model_parts[0]));
            resrefs.push(format!("{full_prefix}{:02}_b", self.model_parts[1]));
            resrefs.push(format!("{full_prefix}{:02}_c", self.model_parts[2]));
        } else if let Some(visual_type) = self.armor_visual_type {
            let prefixes = resolve_armor_prefix(game_data, visual_type, false);
            if let Some(pfx) = prefixes.first() {
                resrefs.push(format!("{pfx}_Body{:02}", self.variation));
            }
        } else {
            // Simple items: W_Arrow01.mdb style (no underscore before variation)
            resrefs.push(format!("{full_prefix}{:02}", self.variation));
        }

        resrefs
    }
}

impl ItemAppearanceOptions {
    fn new_empty() -> Self {
        Self {
            available_variations: Vec::new(),
            available_part1: Vec::new(),
            available_part2: Vec::new(),
            available_part3: Vec::new(),
        }
    }
}
