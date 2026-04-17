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
use super::inventory::EquipmentSlot;

/// Classification derived from `baseitems.2da`'s `modeltype` column.
///
/// NWN2's real values are `"0"` (single-part), `"2"` (3-part weapon),
/// `"3"` (body armour), or empty (no in-world model).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemModelKind {
    ThreePartWeapon,
    SinglePart,
    BodyArmor,
    None,
}

fn classify_model_type(raw: &str) -> ItemModelKind {
    match raw.trim() {
        "2" => ItemModelKind::ThreePartWeapon,
        "0" => ItemModelKind::SinglePart,
        "3" => ItemModelKind::BodyArmor,
        _ => ItemModelKind::None,
    }
}

/// Which armor part the item occupies, derived from `baseitems.2da`'s
/// `equipableslots` bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArmorSlot {
    Head,
    Body,
    Boots,
    Gloves,
    Cloak,
}

impl ArmorSlot {
    /// The filename fragment NWN2 uses for this slot
    /// (e.g. `P_HHM_LE_Body01.mdb`, `P_HHM_LE_Helm01.mdb`).
    fn part_name(self) -> &'static str {
        match self {
            Self::Head => "Helm",
            Self::Body => "Body",
            Self::Boots => "Boots",
            Self::Gloves => "Gloves",
            Self::Cloak => "Cloak",
        }
    }
}

fn parse_equip_slots(raw: &str) -> u32 {
    let s = raw.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        s.parse::<u32>().unwrap_or(0)
    }
}

fn detect_armor_slot(equip_slots: u32) -> Option<ArmorSlot> {
    // Order matters: head before chest because head has its own dedicated helmet
    // base items, while a few chest-style items also occupy other bits.
    let has = |slot: EquipmentSlot| equip_slots & slot.to_bitmask() != 0;
    if has(EquipmentSlot::Head) {
        Some(ArmorSlot::Head)
    } else if has(EquipmentSlot::Chest) {
        Some(ArmorSlot::Body)
    } else if has(EquipmentSlot::Boots) {
        Some(ArmorSlot::Boots)
    } else if has(EquipmentSlot::Gloves) {
        Some(ArmorSlot::Gloves)
    } else if has(EquipmentSlot::Cloak) {
        Some(ArmorSlot::Cloak)
    } else {
        None
    }
}

/// Default body prefix for the isolated item viewer. NWN2 armor meshes are
/// stamped per race/gender (e.g. `P_HHM_` for Human Male); without a wearer
/// context we pick a standard one. Files not matching this prefix simply
/// won't load — the caller handles that by showing "No preview available".
const DEFAULT_BODY_PREFIX: &str = "P_HHM";

/// Common armor-material prefixes from `armor.2da`, tried as fallbacks when
/// the item's own `ArmorVisualType` doesn't resolve (e.g. for helmets, whose
/// material normally comes from the wearer's chest armor).
const FALLBACK_ARMOR_PREFIXES: &[&str] = &["LE", "CL", "CH", "BA", "PF"];

/// A body-armour item's nested Boots/Gloves sub-part. NWN2 chest armour items
/// store these inline — each has its own `ArmorVisualType` (indexing `armor.2da`)
/// and `Variation` (mesh variant number).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct NestedArmorPart {
    pub armor_visual_type: Option<i32>,
    pub variation: i32,
}

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
    /// Nested Boots sub-part (only populated for body armour items that
    /// ship with a matching pair of boots baked into the item GFF).
    #[serde(default)]
    pub boots: Option<NestedArmorPart>,
    /// Nested Gloves sub-part (see `boots`).
    #[serde(default)]
    pub gloves: Option<NestedArmorPart>,
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
        // NWN2 mesh variant filenames start at 01 (e.g. `W_LSword01_a.mdb`,
        // `P_HHM_LE_Body01.mdb`). A stored Variation / ModelPart of 0 would
        // format to `...00.mdb` and never resolve, so clamp to 1.
        let variation = fields
            .get("Variation")
            .and_then(gff_value_to_i32)
            .filter(|&v| v > 0)
            .unwrap_or(1);
        let clamp_part = |key: &str| {
            fields
                .get(key)
                .and_then(gff_value_to_i32)
                .filter(|&v| v > 0)
                .unwrap_or(1)
        };
        let model_parts = [
            clamp_part("ModelPart1"),
            clamp_part("ModelPart2"),
            clamp_part("ModelPart3"),
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

        let boots = read_nested_armor_part(fields, "Boots");
        let gloves = read_nested_armor_part(fields, "Gloves");

        Self {
            variation,
            model_parts,
            tints,
            armor_visual_type,
            boots,
            gloves,
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
        let kind = classify_model_type(&row_str(&row, "modeltype").unwrap_or_default());

        info!(
            "Resolving appearance options for item {base_item_id}: prefix='{prefix}' from {source}, kind={kind:?}"
        );

        match kind {
            ItemModelKind::ThreePartWeapon => {
                let full_prefix = normalize_weapon_prefix(&prefix);
                ItemAppearanceOptions {
                    available_variations: Vec::new(),
                    available_part1: Self::discover_variants(resource_manager, &full_prefix, "_a"),
                    available_part2: Self::discover_variants(resource_manager, &full_prefix, "_b"),
                    available_part3: Self::discover_variants(resource_manager, &full_prefix, "_c"),
                }
            }
            ItemModelKind::SinglePart => {
                let full_prefix = normalize_weapon_prefix(&prefix);
                ItemAppearanceOptions {
                    available_variations: Self::discover_variants(
                        resource_manager,
                        &full_prefix,
                        "",
                    ),
                    available_part1: Vec::new(),
                    available_part2: Vec::new(),
                    available_part3: Vec::new(),
                }
            }
            ItemModelKind::BodyArmor => {
                // Armor prefix is per-item (driven by ArmorVisualType on the equipped item),
                // so we can't discover variations from the base_item_id alone. Offer a
                // reasonable range; the loader skips missing variants gracefully.
                ItemAppearanceOptions {
                    available_variations: (1..=50).collect(),
                    available_part1: Vec::new(),
                    available_part2: Vec::new(),
                    available_part3: Vec::new(),
                }
            }
            ItemModelKind::None => ItemAppearanceOptions::new_empty(),
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
        let full_prefix = normalize_weapon_prefix(&prefix);
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
        let kind = classify_model_type(&row_str(&row, "modeltype").unwrap_or_default());

        if kind == ItemModelKind::BodyArmor {
            let equip_slots =
                parse_equip_slots(&row_str(&row, "equipableslots").unwrap_or_default());
            let slot = detect_armor_slot(equip_slots);
            let item_armor_prefix = self.armor_visual_type.and_then(|vt| {
                resolve_armor_prefix(game_data, vt, false)
                    .into_iter()
                    .next()
            });

            info!(
                "Resolving armor item {base_item_id}: slot={slot:?}, item_armor_prefix={item_armor_prefix:?}, equip_slots=0x{equip_slots:04x}, boots={:?}, gloves={:?}",
                self.boots, self.gloves
            );

            let mut resrefs = build_armor_resrefs(
                slot,
                item_armor_prefix.as_deref(),
                self.variation,
                self.armor_visual_type,
                self.model_parts[0],
            );

            // For body armour, also emit the nested Boots/Gloves sub-part
            // resrefs so the viewer renders the complete outfit (NWN2 stores
            // those parts inline on the chest item).
            if slot == Some(ArmorSlot::Body) {
                resrefs.extend(nested_part_resrefs(
                    self.boots.as_ref(),
                    ArmorSlot::Boots,
                    game_data,
                    item_armor_prefix.as_deref(),
                ));
                resrefs.extend(nested_part_resrefs(
                    self.gloves.as_ref(),
                    ArmorSlot::Gloves,
                    game_data,
                    item_armor_prefix.as_deref(),
                ));
            }

            return resrefs;
        }

        info!(
            "Resolving weapon item {base_item_id}: prefix='{prefix}' from {source}, kind={kind:?}"
        );

        build_weapon_resrefs(kind, &prefix, self.variation, self.model_parts)
    }
}

/// Pure weapon resref builder. For `ThreePartWeapon` also emits a single-part
/// fallback, since a few base items (e.g. `magicstaff`) are tagged `modeltype=2`
/// in `baseitems.2da` but actually ship as a single merged `.mdb`.
fn build_weapon_resrefs(
    kind: ItemModelKind,
    base_prefix: &str,
    variation: i32,
    model_parts: [i32; 3],
) -> Vec<String> {
    if base_prefix.is_empty() {
        return Vec::new();
    }
    match kind {
        ItemModelKind::ThreePartWeapon => {
            let full = normalize_weapon_prefix(base_prefix);
            vec![
                format!("{full}{:02}_a", model_parts[0]),
                format!("{full}{:02}_b", model_parts[1]),
                format!("{full}{:02}_c", model_parts[2]),
                format!("{full}{variation:02}"),
            ]
        }
        ItemModelKind::SinglePart => {
            let full = normalize_weapon_prefix(base_prefix);
            vec![format!("{full}{variation:02}")]
        }
        ItemModelKind::BodyArmor | ItemModelKind::None => Vec::new(),
    }
}

/// Pure armor resref builder. The viewer has no wearer context, so we stamp a
/// default race/gender body prefix and try the item's own armor prefix plus
/// common fallbacks — whichever file exists will load, the rest are skipped.
///
/// NWN2 armor file pattern: `{body_prefix}_{armor_prefix}_{Part}{NN}.mdb`
/// e.g. `P_HHM_LE_Body01.mdb`, `P_HHM_LE_Helm05.mdb`, `P_HHM_CL_Cloak01.mdb`.
fn build_armor_resrefs(
    slot: Option<ArmorSlot>,
    item_armor_prefix: Option<&str>,
    variation: i32,
    armor_visual_type: Option<i32>,
    model_part1: i32,
) -> Vec<String> {
    let Some(slot) = slot else {
        return Vec::new();
    };

    match slot {
        ArmorSlot::Body => match item_armor_prefix {
            Some(pfx) => armor_resref_candidates(slot, variation, &[pfx], false),
            None => Vec::new(),
        },
        ArmorSlot::Boots | ArmorSlot::Gloves => match item_armor_prefix {
            Some(pfx) => armor_resref_candidates(slot, variation, &[pfx], false),
            None => Vec::new(),
        },
        ArmorSlot::Head => {
            // Helmets ride on the wearer's chest armor prefix in-game. For the
            // viewer we don't know chest context, so try the common materials.
            // The helmet item's `ArmorVisualType` is the variation number
            // (style index), not a material.
            let nn = armor_visual_type.filter(|v| *v > 0).unwrap_or(variation);
            armor_resref_candidates(slot, nn, &[], true)
        }
        ArmorSlot::Cloak => {
            // Cloak uses the hardcoded `CL` armor prefix. Variant lives in
            // ModelPart1 for most cloaks; fall back to Variation.
            let nn = if model_part1 > 0 {
                model_part1
            } else {
                variation
            };
            armor_resref_candidates(slot, nn, &["CL"], false)
        }
    }
}

/// Emits `{DEFAULT_BODY_PREFIX}_{prefix}_{Part}{nn:02}` for each primary
/// prefix, optionally followed by the common material fallbacks. Fallbacks
/// are only correct for slots where the primary can't identify the material
/// (e.g. helmets, which take the wearer's chest material in-game).
fn armor_resref_candidates(
    slot: ArmorSlot,
    nn: i32,
    primary_prefixes: &[&str],
    include_material_fallbacks: bool,
) -> Vec<String> {
    if nn <= 0 {
        return Vec::new();
    }
    let part = slot.part_name();
    let mut out = Vec::new();
    let mut push = |pfx: &str| {
        let s = format!("{DEFAULT_BODY_PREFIX}_{pfx}_{part}{nn:02}");
        if !out.contains(&s) {
            out.push(s);
        }
    };
    for p in primary_prefixes {
        push(p);
    }
    if include_material_fallbacks {
        for p in FALLBACK_ARMOR_PREFIXES {
            push(p);
        }
    }
    out
}

fn normalize_weapon_prefix(prefix: &str) -> String {
    if prefix.to_uppercase().starts_with("W_") {
        prefix.to_string()
    } else {
        format!("W_{prefix}")
    }
}

fn read_nested_armor_part(
    fields: &IndexMap<String, GffValue<'_>>,
    key: &str,
) -> Option<NestedArmorPart> {
    let part_fields = match fields.get(key)? {
        GffValue::StructOwned(s) => s.as_ref().clone(),
        GffValue::Struct(lazy) => lazy.force_load(),
        _ => return None,
    };

    let variation = part_fields
        .get("Variation")
        .and_then(gff_value_to_i32)
        .unwrap_or(0);
    if variation <= 0 {
        return None;
    }
    let armor_visual_type = part_fields
        .get("ArmorVisualType")
        .and_then(gff_value_to_i32);

    Some(NestedArmorPart {
        armor_visual_type,
        variation,
    })
}

/// Build resrefs for a nested Boots/Gloves sub-part of a body-armour item.
/// The nested part's own `ArmorVisualType` can differ from the chest's, and
/// the file may only exist for one of them (or for a neutral fallback), so
/// we emit candidates for both and a fallback list of common materials.
fn nested_part_resrefs(
    part: Option<&NestedArmorPart>,
    slot: ArmorSlot,
    game_data: &GameData,
    chest_armor_prefix: Option<&str>,
) -> Vec<String> {
    let Some(part) = part else { return Vec::new() };
    let part_prefix = part.armor_visual_type.and_then(|vt| {
        resolve_armor_prefix(game_data, vt, false)
            .into_iter()
            .next()
    });

    let primaries: Vec<&str> = [part_prefix.as_deref(), chest_armor_prefix]
        .into_iter()
        .flatten()
        .collect();
    armor_resref_candidates(slot, part.variation, &primaries, true)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_nwn2_modeltype_values() {
        assert_eq!(classify_model_type("2"), ItemModelKind::ThreePartWeapon);
        assert_eq!(classify_model_type("0"), ItemModelKind::SinglePart);
        assert_eq!(classify_model_type("3"), ItemModelKind::BodyArmor);
        assert_eq!(classify_model_type(""), ItemModelKind::None);
        assert_eq!(classify_model_type("   "), ItemModelKind::None);
        assert_eq!(classify_model_type("A"), ItemModelKind::None);
    }

    #[test]
    fn three_part_weapon_emits_abc_plus_single_part_fallback() {
        let r = build_weapon_resrefs(ItemModelKind::ThreePartWeapon, "W_LSword", 1, [1, 2, 3]);
        // 3-part pattern first, single-part fallback at the end (for items like
        // magicstaff that are tagged modeltype=2 but ship as one merged mdb).
        assert_eq!(
            r,
            vec!["W_LSword01_a", "W_LSword02_b", "W_LSword03_c", "W_LSword01"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn three_part_weapon_w_prefixes_bare_label() {
        let r = build_weapon_resrefs(ItemModelKind::ThreePartWeapon, "Axe", 1, [1, 1, 1]);
        assert_eq!(r[0], "W_Axe01_a");
        assert_eq!(r[3], "W_Axe01");
    }

    #[test]
    fn magicstaff_single_part_file_reachable_via_fallback() {
        // magicstaff has modeltype=2 but the real file is `w_mstaff01.mdb`.
        let r = build_weapon_resrefs(ItemModelKind::ThreePartWeapon, "w_mstaff", 1, [1, 1, 1]);
        assert!(r.contains(&"w_mstaff01".to_string()));
    }

    #[test]
    fn single_part_crossbow_emits_single_resref() {
        let r = build_weapon_resrefs(ItemModelKind::SinglePart, "w_crsbL", 1, [0, 0, 0]);
        assert_eq!(r, vec!["w_crsbL01".to_string()]);
    }

    #[test]
    fn single_part_shield_preserves_mixed_case_label() {
        let r = build_weapon_resrefs(ItemModelKind::SinglePart, "w_she_large", 3, [0, 0, 0]);
        assert_eq!(r, vec!["w_she_large03".to_string()]);
    }

    #[test]
    fn single_part_arrow_uses_variation_not_model_parts() {
        let r = build_weapon_resrefs(ItemModelKind::SinglePart, "W_Arrow", 5, [99, 99, 99]);
        assert_eq!(r, vec!["W_Arrow05".to_string()]);
    }

    #[test]
    fn empty_base_prefix_skips_weapon_resolution() {
        let three = build_weapon_resrefs(ItemModelKind::ThreePartWeapon, "", 1, [1, 1, 1]);
        let single = build_weapon_resrefs(ItemModelKind::SinglePart, "", 1, [0, 0, 0]);
        assert!(three.is_empty());
        assert!(single.is_empty());
    }

    #[test]
    fn body_armor_uses_armor_prefix_and_body_suffix() {
        let r = build_armor_resrefs(Some(ArmorSlot::Body), Some("LE"), 7, Some(2), 0);
        assert_eq!(r, vec!["P_HHM_LE_Body07".to_string()]);
    }

    #[test]
    fn body_armor_without_resolved_prefix_emits_nothing() {
        let r = build_armor_resrefs(Some(ArmorSlot::Body), None, 1, None, 0);
        assert!(r.is_empty());
    }

    #[test]
    fn helmet_uses_armor_visual_type_as_variation_with_material_fallbacks() {
        let r = build_armor_resrefs(Some(ArmorSlot::Head), None, 0, Some(5), 0);
        assert!(r.contains(&"P_HHM_LE_Helm05".to_string()));
        assert!(r.contains(&"P_HHM_CL_Helm05".to_string()));
        assert!(r.contains(&"P_HHM_CH_Helm05".to_string()));
    }

    #[test]
    fn standalone_boots_use_only_item_prefix() {
        // Standalone boots know their material from ArmorVisualType. Falling
        // back across materials would load a visibly-wrong mesh, so we don't.
        let r = build_armor_resrefs(Some(ArmorSlot::Boots), Some("LE"), 3, Some(2), 0);
        assert_eq!(r, vec!["P_HHM_LE_Boots03".to_string()]);
    }

    #[test]
    fn standalone_gloves_use_only_item_prefix() {
        let r = build_armor_resrefs(Some(ArmorSlot::Gloves), Some("CH"), 2, Some(4), 0);
        assert_eq!(r, vec!["P_HHM_CH_Gloves02".to_string()]);
    }

    #[test]
    fn cloak_prefers_model_part1_over_variation() {
        let r = build_armor_resrefs(Some(ArmorSlot::Cloak), None, 99, None, 4);
        assert_eq!(r, vec!["P_HHM_CL_Cloak04".to_string()]);
    }

    #[test]
    fn cloak_falls_back_to_variation_when_model_part1_zero() {
        let r = build_armor_resrefs(Some(ArmorSlot::Cloak), None, 7, None, 0);
        assert_eq!(r, vec!["P_HHM_CL_Cloak07".to_string()]);
    }

    #[test]
    fn armor_with_unknown_slot_is_empty() {
        let r = build_armor_resrefs(None, Some("LE"), 1, Some(2), 0);
        assert!(r.is_empty());
    }

    #[test]
    fn parses_hex_and_decimal_equip_slots() {
        assert_eq!(parse_equip_slots("0x20002"), 0x20002);
        assert_eq!(parse_equip_slots("0X0001"), 0x0001);
        assert_eq!(parse_equip_slots("64"), 64);
        assert_eq!(parse_equip_slots(""), 0);
        assert_eq!(parse_equip_slots("garbage"), 0);
    }

    #[test]
    fn detects_armor_slots_from_real_baseitems_bitmasks() {
        // bitmasks copied from real baseitems.2da rows
        assert_eq!(detect_armor_slot(0x00001), Some(ArmorSlot::Head)); // helmet
        assert_eq!(detect_armor_slot(0x20002), Some(ArmorSlot::Body)); // armor (chest + creature-armor bit)
        assert_eq!(detect_armor_slot(0x00004), Some(ArmorSlot::Boots));
        assert_eq!(detect_armor_slot(0x00008), Some(ArmorSlot::Gloves));
        assert_eq!(detect_armor_slot(0x00040), Some(ArmorSlot::Cloak));
        assert_eq!(detect_armor_slot(0x00000), None);
        assert_eq!(detect_armor_slot(0x00200), None); // amulet (neck) — not armor
    }

    #[test]
    fn normalize_weapon_prefix_is_case_insensitive() {
        assert_eq!(normalize_weapon_prefix("W_Axe"), "W_Axe");
        assert_eq!(normalize_weapon_prefix("w_Lbow"), "w_Lbow");
        assert_eq!(normalize_weapon_prefix("Axe"), "W_Axe");
    }
}
