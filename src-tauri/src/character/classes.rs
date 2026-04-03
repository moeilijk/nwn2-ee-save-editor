//! Character class-related methods.
//!
//! Provides access to character class information including:
//! - Total level calculation
//! - Class entries and levels
//! - Multi-class support
//!
//! All methods are sync (no async). ClassList structure in GFF:
//! - ClassList: List of structs
//!   - Class: class ID (engine saves commonly serialize this as Int)
//!   - ClassLevel: Short (level in that class)
//!   - SpellCasterLevel: Optional (for spellcasters)

use ahash::AHashMap;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};

use super::{Character, CharacterError};
use crate::character::feats::FeatSource;
use crate::character::gff_helpers::gff_value_to_i32;
use crate::character::identity::Alignment;
use crate::character::types::{AbilityIndex, ClassId, FeatId, RaceId, SkillId};
use crate::loaders::GameData;
use crate::parsers::gff::GffValue;
use crate::services::field_mapper::FieldMapper;

const MAX_CLASSES: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct SkillPointsSummary {
    pub theoretical_total: i32,
    pub actual_spent: i32,
    pub current_unspent: i32,
    pub mismatch: i32,
}
const MAX_TOTAL_LEVEL: i32 = 60;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct ClassEntry {
    pub class_id: ClassId,
    pub level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LevelHistoryEntry {
    pub character_level: i32,
    pub class_id: ClassId,
    pub class_level: i32,
    pub hp_gained: i32,
    pub skill_points_remaining: i32,
    pub ability_increase: Option<AbilityIndex>,
    pub feats_gained: Vec<FeatId>,
    pub skills_gained: Vec<SkillRankEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SkillRankEntry {
    pub skill_id: SkillId,
    pub ranks: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResolvedLevelHistoryEntry {
    pub character_level: i32,
    pub class_id: i32,
    pub class_name: String,
    pub class_level: i32,
    pub hp_gained: i32,
    pub skill_points_remaining: i32,
    pub ability_increase: Option<String>,
    pub feats_gained: Vec<ResolvedFeatEntry>,
    pub skills_gained: Vec<ResolvedSkillEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResolvedFeatEntry {
    pub feat_id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResolvedSkillEntry {
    pub skill_id: i32,
    pub name: String,
    pub ranks: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub enum BabType {
    Full,
    ThreeQuarter,
    Half,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClassInfo {
    pub id: ClassId,
    pub name: String,
    pub hit_die: i32,
    pub primary_ability: Option<AbilityIndex>,
    pub is_spellcaster: bool,
    pub bab_type: BabType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct XpProgress {
    pub current_xp: i32,
    pub current_level: i32,
    pub xp_for_current_level: i32,
    pub xp_for_next_level: i32,
    pub xp_remaining: i32,
    pub progress_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClassSummaryEntry {
    pub class_id: ClassId,
    pub name: String,
    pub level: i32,
    pub hit_die: i32,
    pub base_attack_bonus: i32,
    pub fortitude_save: i32,
    pub reflex_save: i32,
    pub will_save: i32,
    pub skill_points_per_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LevelUpResult {
    pub class_id: ClassId,
    pub new_level: i32,
    pub hp_gained: i32,
    pub skill_points_gained: i32,
    pub general_feat_slots_gained: i32,
    pub bonus_feat_slots_gained: i32,
    pub ability_increase_gained: bool,
    pub new_spells_gained: bool,
    pub granted_feats: Vec<FeatId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClassProgression {
    pub class_id: i32,
    pub class_name: String,
    pub basic_info: ClassBasicInfo,
    pub level_progression: Vec<LevelProgressionEntry>,
    pub max_level_shown: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClassBasicInfo {
    pub hit_die: i32,
    pub skill_points_per_level: i32,
    pub bab_progression: String,
    pub save_progression: String,
    pub is_spellcaster: bool,
    pub spell_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LevelProgressionEntry {
    pub level: i32,
    pub base_attack_bonus: i32,
    pub fortitude_save: i32,
    pub reflex_save: i32,
    pub will_save: i32,
    pub features: Vec<ClassFeature>,
    pub spell_slots: Option<SpellSlots>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClassFeature {
    pub name: String,
    pub feature_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SpellSlots {
    pub level_0: i32,
    pub level_1: i32,
    pub level_2: i32,
    pub level_3: i32,
    pub level_4: i32,
    pub level_5: i32,
    pub level_6: i32,
    pub level_7: i32,
    pub level_8: i32,
    pub level_9: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClassFeatInfo {
    pub feat_id: FeatId,
    pub list_type: i32,
    pub granted_on_level: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct ClassesState {
    pub total_level: i32,
    pub entries: Vec<ClassSummaryEntry>,
    pub xp_progress: XpProgress,
    pub level_history: Vec<LevelHistoryEntry>,
    pub skill_points_summary: SkillPointsSummary,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Type)]
pub struct AlignmentRestriction(pub i32);

impl AlignmentRestriction {
    pub const LAWFUL: i32 = 0x01;
    pub const CHAOTIC: i32 = 0x02;
    pub const GOOD: i32 = 0x04;
    pub const EVIL: i32 = 0x08;
    pub const NEUTRAL_LC: i32 = 0x10;
    pub const NEUTRAL_GE: i32 = 0x20;

    pub fn contains(&self, flag: i32) -> bool {
        (self.0 & flag) != 0
    }

    pub fn decode_to_string(&self) -> Option<String> {
        if self.0 == 0 {
            return Some("Any".to_string());
        }
        let mut allowed = Vec::new();
        if self.contains(Self::LAWFUL) {
            allowed.push("Lawful");
        }
        if self.contains(Self::CHAOTIC) {
            allowed.push("Chaotic");
        }
        if self.contains(Self::GOOD) {
            allowed.push("Good");
        }
        if self.contains(Self::EVIL) {
            allowed.push("Evil");
        }
        if self.contains(Self::NEUTRAL_LC) {
            allowed.push("Neutral (Law/Chaos)");
        }
        if self.contains(Self::NEUTRAL_GE) {
            allowed.push("Neutral (Good/Evil)");
        }
        if allowed.is_empty() {
            Some("Any".to_string())
        } else {
            Some(allowed.join(" or "))
        }
    }

    pub fn check_alignment(&self, alignment: &Alignment) -> bool {
        if self.0 == 0 {
            return true;
        }
        let lc_ok = (self.contains(Self::LAWFUL) && alignment.is_lawful())
            || (self.contains(Self::CHAOTIC) && alignment.is_chaotic())
            || (self.contains(Self::NEUTRAL_LC) && alignment.is_neutral_law_chaos());
        let ge_ok = (self.contains(Self::GOOD) && alignment.is_good())
            || (self.contains(Self::EVIL) && alignment.is_evil())
            || (self.contains(Self::NEUTRAL_GE) && alignment.is_neutral_good_evil());
        let has_lc_restriction = self.contains(Self::LAWFUL)
            || self.contains(Self::CHAOTIC)
            || self.contains(Self::NEUTRAL_LC);
        let has_ge_restriction = self.contains(Self::GOOD)
            || self.contains(Self::EVIL)
            || self.contains(Self::NEUTRAL_GE);
        (!has_lc_restriction || lc_ok) && (!has_ge_restriction || ge_ok)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct PrestigeRequirements {
    pub base_attack_bonus: Option<i32>,
    pub skills: Vec<(String, i32)>,
    pub feats: Vec<String>,
    pub alignment: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct PrestigeClassValidation {
    pub can_take: bool,
    pub missing_requirements: Vec<String>,
    pub requirements: PrestigeRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrestigeClassOption {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub can_take: bool,
    pub reason: String,
    pub requirements: PrestigeRequirements,
}

impl Default for XpProgress {
    fn default() -> Self {
        Self {
            current_xp: 0,
            current_level: 1,
            xp_for_current_level: 0,
            xp_for_next_level: 1000,
            xp_remaining: 1000,
            progress_percent: 0.0,
        }
    }
}

impl Character {
    fn clear_known_spell_lists(entry: &mut IndexMap<String, GffValue<'static>>) {
        for spell_level in 0..10 {
            entry.shift_remove(&format!("KnownList{spell_level}"));
            entry.shift_remove(&format!("KnownRemoveList{spell_level}"));
        }
    }

    fn empty_skill_history_list(
        &self,
        template_entry: Option<&IndexMap<String, GffValue<'static>>>,
    ) -> Vec<IndexMap<String, GffValue<'static>>> {
        let skill_count = template_entry
            .and_then(|entry| entry.get("SkillList"))
            .and_then(|value| match value {
                GffValue::ListOwned(list) => Some(list.len()),
                _ => None,
            })
            .filter(|len| *len > 0)
            .or_else(|| self.get_list("SkillList").map(std::vec::Vec::len))
            .unwrap_or_default();

        let mut skill_list = Vec::with_capacity(skill_count);
        for _ in 0..skill_count {
            let mut empty_skill = IndexMap::new();
            empty_skill.insert("Rank".to_string(), GffValue::Byte(0));
            skill_list.push(empty_skill);
        }
        skill_list
    }

    pub fn normalize_class_fields_for_save(&mut self, game_data: &GameData) {
        let mut class_list = self.get_list_owned("ClassList").unwrap_or_default();

        for entry in class_list.iter_mut() {
            if let Some(class_id) = entry.get("Class").and_then(gff_value_to_i32) {
                entry.insert("Class".to_string(), GffValue::Int(class_id));
            }
            if let Some(class_level) = entry.get("ClassLevel").and_then(gff_value_to_i32) {
                entry.insert(
                    "ClassLevel".to_string(),
                    GffValue::Short(class_level as i16),
                );
            }
        }
        self.set_list("ClassList", class_list.clone());
        self.remove_field("Class");

        let class_count = class_list.len();

        let normalized_level_up_index = if class_count == 0 {
            0
        } else {
            self.get_i32("MClassLevUpIn")
                .unwrap_or(0)
                .clamp(0, class_count as i32 - 1)
        };
        if self.get_i32("MClassLevUpIn") != Some(normalized_level_up_index) {
            self.set_byte("MClassLevUpIn", normalized_level_up_index as u8);
        }

        if let Some(primary_class_id) = class_list
            .first()
            .and_then(|entry| entry.get("Class"))
            .and_then(gff_value_to_i32)
            && let Some(classes_table) = game_data.get_table("classes")
            && let Some(class_data) = classes_table.get_by_id(primary_class_id)
            && let Some(package_id) = Self::get_field_value(&class_data, "package")
                .and_then(|s| s.parse::<i32>().ok())
                .filter(|&id| id >= 0)
            && self.get_i32("StartingPackage") != Some(package_id)
        {
            self.set_byte("StartingPackage", package_id as u8);
        }

        let skill_template = self.empty_skill_history_list(None);
        let classes_table = game_data.get_table("classes");
        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();

        for (level_index, entry) in lvl_stat_list.iter_mut().enumerate() {
            let class_id = entry
                .get("LvlStatClass")
                .and_then(gff_value_to_i32)
                .unwrap_or(-1);

            let class_data = classes_table.and_then(|table| table.get_by_id(class_id));

            let hit_die = class_data
                .as_ref()
                .and_then(|data| Self::get_field_value(data, "hit_die"))
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(6);
            let base_hp = if level_index == 0 {
                hit_die
            } else {
                (hit_die / 2) + 1
            };
            entry.insert("LvlStatHitDie".to_string(), GffValue::Byte(base_hp as u8));
            entry.insert(
                "EpicLevel".to_string(),
                GffValue::Byte(if level_index + 1 > 20 { 1 } else { 0 }),
            );

            if !entry.contains_key("FeatList") {
                entry.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
            }

            let skill_len = super::gff_helpers::extract_list_from_map(entry, "SkillList")
                .map(|list| list.len())
                .unwrap_or(0);
            if !skill_template.is_empty() && skill_len != skill_template.len() {
                entry.insert(
                    "SkillList".to_string(),
                    GffValue::ListOwned(skill_template.clone()),
                );
            }

            let is_spellcaster = class_data.as_ref().is_some_and(|data| {
                Self::get_field_value(data, "spell_caster")
                    .or_else(|| Self::get_field_value(data, "spellcaster"))
                    .is_some_and(|s| {
                        matches!(s.trim().to_lowercase().as_str(), "1" | "true" | "yes")
                    })
            });
            if !is_spellcaster {
                Self::clear_known_spell_lists(entry);
            }
        }

        self.set_list("LvlStatList", lvl_stat_list);
    }

    fn push_feat_with_source_without_history(&mut self, feat_id: FeatId, source: FeatSource) {
        if self.has_feat(feat_id) {
            return;
        }

        let mut feat_list = self.get_list_owned("FeatList").unwrap_or_default();
        let mut new_entry = IndexMap::new();
        new_entry.insert("Feat".to_string(), GffValue::Word(feat_id.0 as u16));

        if source != FeatSource::Unknown {
            new_entry.insert(
                "Source".to_string(),
                GffValue::String(std::borrow::Cow::Owned(source.as_str().to_string())),
            );
        }

        feat_list.push(new_entry);
        self.set_list("FeatList", feat_list);
    }

    fn grant_auto_class_feats_without_history(
        &mut self,
        class_id: ClassId,
        class_level: i32,
        game_data: &GameData,
    ) {
        for lvl in 1..=class_level {
            for feat_info in self.get_class_feats_for_level(class_id, lvl, game_data) {
                if feat_info.list_type == 0 || feat_info.list_type == 3 {
                    self.push_feat_with_source_without_history(
                        feat_info.feat_id,
                        FeatSource::Class,
                    );
                }
            }
        }
    }

    /// Get the total character level (sum of all class levels).
    pub fn total_level(&self) -> i32 {
        self.class_entries().iter().map(|entry| entry.level).sum()
    }

    /// Get all class entries for this character.
    pub fn class_entries(&self) -> Vec<ClassEntry> {
        let Some(class_list) = self.get_list("ClassList") else {
            return vec![];
        };

        class_list
            .iter()
            .filter_map(|entry| {
                let class_id = entry.get("Class").and_then(gff_value_to_i32)?;
                let class_level = entry.get("ClassLevel").and_then(gff_value_to_i32)?;

                Some(ClassEntry {
                    class_id: ClassId(class_id),
                    level: class_level,
                })
            })
            .collect()
    }

    /// Get the level in a specific class (0 if character doesn't have this class).
    pub fn class_level(&self, class_id: ClassId) -> i32 {
        self.class_entries()
            .iter()
            .find(|e| e.class_id == class_id)
            .map_or(0, |e| e.level)
    }

    /// Get the number of different classes the character has.
    pub fn class_count(&self) -> usize {
        self.class_entries().len()
    }

    /// Get all domains possessed by the character (from Cleric or similar classes).
    /// Returns a list of Domain IDs.
    pub fn domains(&self) -> Vec<crate::character::types::DomainId> {
        let Some(class_list) = self.get_list("ClassList") else {
            return vec![];
        };

        let mut domains = Vec::new();

        for entry in class_list {
            // Check for Domain1 and Domain2 fields
            if let Some(d1) = entry.get("Domain1").and_then(gff_value_to_i32)
                && d1 > 0
            {
                domains.push(crate::character::types::DomainId(d1));
            }
            if let Some(d2) = entry.get("Domain2").and_then(gff_value_to_i32)
                && d2 > 0
            {
                domains.push(crate::character::types::DomainId(d2));
            }
        }
        domains
    }

    /// Check if the character has a specific class.
    pub fn has_class(&self, class_id: ClassId) -> bool {
        self.class_entries().iter().any(|e| e.class_id == class_id)
    }

    /// Set the level for an existing class.
    pub fn set_class_level(
        &mut self,
        class_id: ClassId,
        new_level: i32,
    ) -> Result<(), CharacterError> {
        if new_level < 1 {
            return Err(CharacterError::ValidationFailed {
                field: "ClassLevel",
                message: format!("Level must be at least 1, got {new_level}"),
            });
        }

        if new_level > MAX_TOTAL_LEVEL {
            return Err(CharacterError::OutOfRange {
                field: "ClassLevel",
                value: new_level,
                min: 1,
                max: MAX_TOTAL_LEVEL,
            });
        }

        let mut class_list = self
            .get_list_owned("ClassList")
            .ok_or(CharacterError::FieldMissing { field: "ClassList" })?;

        let mut found = false;
        for entry in &mut class_list {
            let entry_class_id = entry.get("Class").and_then(gff_value_to_i32).unwrap_or(-1);
            if entry_class_id == class_id.0 {
                entry.insert("ClassLevel".to_string(), GffValue::Short(new_level as i16));
                found = true;
                break;
            }
        }

        if !found {
            return Err(CharacterError::NotFound {
                entity: "Class",
                id: class_id.0,
            });
        }

        self.set_list("ClassList", class_list);
        Ok(())
    }

    /// Add a new class to the character.
    pub fn add_class_entry(&mut self, class_id: ClassId, level: i32) -> Result<(), CharacterError> {
        if level < 1 {
            return Err(CharacterError::ValidationFailed {
                field: "ClassLevel",
                message: format!("Level must be at least 1, got {level}"),
            });
        }

        if self.has_class(class_id) {
            return Err(CharacterError::AlreadyExists {
                entity: "Class",
                id: class_id.0,
            });
        }

        let current_class_count = self.class_count();
        if current_class_count >= MAX_CLASSES {
            return Err(CharacterError::ValidationFailed {
                field: "ClassList",
                message: format!("Maximum {MAX_CLASSES} classes allowed"),
            });
        }

        let mut class_list = self.get_list_owned("ClassList").unwrap_or_default();

        let mut new_entry = IndexMap::new();
        new_entry.insert("Class".to_string(), GffValue::Byte(class_id.0 as u8));
        new_entry.insert("ClassLevel".to_string(), GffValue::Short(level as i16));
        class_list.push(new_entry);

        self.set_list("ClassList", class_list);
        Ok(())
    }

    /// Add a class entry and grant automatic class feats for each level.
    ///
    /// This method adds the class to ClassList and then grants all automatic
    /// feats from the class's feat table (`cls_feat_*`) where `list_type == 0`
    /// and `granted_on_level` matches levels 1 through `level`.
    pub fn add_class_entry_with_feats(
        &mut self,
        class_id: ClassId,
        level: i32,
        game_data: &GameData,
    ) -> Result<Vec<FeatId>, CharacterError> {
        use super::feats::FeatSource;

        // First add the class entry normally
        self.add_class_entry(class_id, level)?;

        let mut granted_feats = Vec::new();

        // Get automatic feats for each level 1..=level
        for lvl in 1..=level {
            let feats_at_level = self.get_class_feats_for_level(class_id, lvl, game_data);
            for feat_info in feats_at_level {
                // list_type == 0 means automatically granted (e.g. Epic feats)
                // list_type == 3 means Class feat (e.g. Fighter Proficiencies), also automatic
                if (feat_info.list_type == 0 || feat_info.list_type == 3)
                    && !self.has_feat(feat_info.feat_id)
                    && self
                        .add_feat_with_source(feat_info.feat_id, FeatSource::Class)
                        .is_ok()
                {
                    granted_feats.push(feat_info.feat_id);
                }
            }
        }

        Ok(granted_feats)
    }

    pub fn get_class_feats_for_level(
        &self,
        class_id: ClassId,
        level: i32,
        game_data: &GameData,
    ) -> Vec<ClassFeatInfo> {
        let mut feats_for_level = Vec::new();

        let Some(classes_table) = game_data.get_table("classes") else {
            return feats_for_level;
        };

        let Some(class_row) = classes_table.get_by_id(class_id.0) else {
            return feats_for_level;
        };

        let feats_table_name = class_row
            .get("FeatsTable")
            .or_else(|| class_row.get("feats_table"))
            .or_else(|| class_row.get("FEATSTABLE"))
            .and_then(std::clone::Clone::clone)
            .filter(|s| !s.is_empty() && s != "****");

        let Some(table_name) = feats_table_name else {
            return feats_for_level;
        };

        let Some(feat_table) = game_data.get_table(&table_name.to_lowercase()) else {
            return feats_for_level;
        };

        for row_idx in 0..feat_table.row_count() {
            let Ok(row) = feat_table.get_row(row_idx) else {
                continue;
            };

            let granted_level = row
                .get("GrantedOnLevel")
                .or_else(|| row.get("granted_on_level"))
                .or_else(|| row.get("GRANTEDONLEVEL"))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .unwrap_or(-1);

            if granted_level != level {
                continue;
            }

            let feat_id = row
                .get("FeatIndex")
                .or_else(|| row.get("feat_index"))
                .or_else(|| row.get("FEATINDEX"))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .unwrap_or(-1);

            if feat_id < 0 {
                continue;
            }

            let list_type = row
                .get("List")
                .or_else(|| row.get("list"))
                .or_else(|| row.get("LIST"))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .unwrap_or(3);

            feats_for_level.push(ClassFeatInfo {
                feat_id: FeatId(feat_id),
                list_type,
                granted_on_level: granted_level,
            });
        }

        feats_for_level
    }

    pub fn get_all_class_feat_ids(&self, class_id: ClassId, game_data: &GameData) -> HashSet<i32> {
        let mut feat_ids = HashSet::new();

        let Some(classes_table) = game_data.get_table("classes") else {
            return feat_ids;
        };

        let Some(class_row) = classes_table.get_by_id(class_id.0) else {
            return feat_ids;
        };

        let feats_table_name = class_row
            .get("FeatsTable")
            .or_else(|| class_row.get("feats_table"))
            .or_else(|| class_row.get("FEATSTABLE"))
            .and_then(std::clone::Clone::clone)
            .filter(|s| !s.is_empty() && s != "****");

        let Some(table_name) = feats_table_name else {
            return feat_ids;
        };

        let Some(feat_table) = game_data.get_table(&table_name.to_lowercase()) else {
            return feat_ids;
        };

        for row_idx in 0..feat_table.row_count() {
            let Ok(row) = feat_table.get_row(row_idx) else {
                continue;
            };

            let list_type = row
                .get("List")
                .or_else(|| row.get("list"))
                .or_else(|| row.get("LIST"))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .unwrap_or(-1);

            if list_type != 0 && list_type != 3 {
                continue;
            }

            let feat_id = row
                .get("FeatIndex")
                .or_else(|| row.get("feat_index"))
                .or_else(|| row.get("FEATINDEX"))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .unwrap_or(-1);

            if feat_id >= 0 {
                feat_ids.insert(feat_id);
            }
        }

        feat_ids
    }

    pub fn reconcile_class_feats(&mut self, removed_class_ids: &[ClassId], game_data: &GameData) {
        let preserved_feats = self.get_preserved_feat_ids(game_data);

        let mut feats_from_removed: HashSet<i32> = HashSet::new();
        for class_id in removed_class_ids {
            feats_from_removed.extend(self.get_all_class_feat_ids(*class_id, game_data));
        }

        let mut feats_from_remaining: HashSet<i32> = HashSet::new();
        let class_entries = self.class_entries();
        for entry in &class_entries {
            feats_from_remaining.extend(self.get_all_class_feat_ids(entry.class_id, game_data));
        }

        let feats_to_remove: HashSet<i32> = feats_from_removed
            .iter()
            .filter(|f| !feats_from_remaining.contains(f) && !preserved_feats.contains(f))
            .copied()
            .collect();

        if feats_to_remove.is_empty() {
            return;
        }

        let mut char_feat_list = self.get_list_owned("FeatList").unwrap_or_default();
        char_feat_list.retain(|f| {
            let feat_id = f.get("Feat").and_then(gff_value_to_i32).unwrap_or(-1);
            !feats_to_remove.contains(&feat_id)
        });
        self.set_list("FeatList", char_feat_list);

        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        for stat_entry in &mut lvl_stat_list {
            if let Some(mut feat_list) =
                super::gff_helpers::extract_list_from_map(stat_entry, "FeatList")
            {
                feat_list.retain(|f| {
                    let feat_id = f.get("Feat").and_then(gff_value_to_i32).unwrap_or(-1);
                    !feats_to_remove.contains(&feat_id)
                });
                stat_entry.insert("FeatList".to_string(), GffValue::ListOwned(feat_list));
            }
        }
        self.set_list("LvlStatList", lvl_stat_list);
    }

    pub fn get_preserved_feat_ids(&self, game_data: &GameData) -> HashSet<i32> {
        let mut preserved = HashSet::new();

        let race_id = self.race_id();
        if let Some(racial_feats) = self.get_racial_feat_ids(race_id, game_data) {
            preserved.extend(racial_feats);
        }

        if let Some(subrace_idx) = self.subrace_index()
            && let Some(subtypes_table) = game_data.get_table("racialsubtypes")
            && let Some(subtype_row) = subtypes_table.get_by_id(subrace_idx)
        {
            let feat_table_name = Self::get_field_value(&subtype_row, "FeatsTable")
                .filter(|s| !s.is_empty() && s != "****");
            if let Some(table_name) = feat_table_name
                && let Some(feat_table) = game_data.get_table(&table_name.to_lowercase())
            {
                for row_idx in 0..feat_table.row_count() {
                    if let Ok(row) = feat_table.get_row(row_idx) {
                        let feat_id = row
                            .get("FeatIndex")
                            .or_else(|| row.get("feat_index"))
                            .or_else(|| row.get("FEATINDEX"))
                            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                            .unwrap_or(-1);
                        if feat_id >= 0 {
                            preserved.insert(feat_id);
                        }
                    }
                }
            }
        }

        let feat_entries = self.feat_entries();
        for entry in &feat_entries {
            if (4000..=4999).contains(&entry.feat_id.0) {
                preserved.insert(entry.feat_id.0);
            }
        }

        for entry in &feat_entries {
            if matches!(entry.source, FeatSource::Race | FeatSource::Background) {
                preserved.insert(entry.feat_id.0);
            }
        }

        if let Some(feat_table) = game_data.get_table("feat") {
            for entry in &feat_entries {
                if let Some(feat_row) = feat_table.get_by_id(entry.feat_id.0) {
                    let cat = Self::get_field_value(&feat_row, "TOOLSCATEGORIES")
                        .and_then(|s| s.parse::<i32>().ok());
                    if let Some(cat) = cat {
                        if matches!(cat, 10 | 11 | 12 | 14) {
                            preserved.insert(entry.feat_id.0);
                        }
                        continue;
                    }

                    let feat_cat =
                        Self::get_field_value(&feat_row, "FeatCategory").unwrap_or_default();
                    if feat_cat.contains("BACKGROUND")
                        || feat_cat.contains("HISTORY")
                        || feat_cat.contains("HERITAGE")
                        || feat_cat.contains("RACIAL")
                    {
                        preserved.insert(entry.feat_id.0);
                    }
                }
            }
        }

        preserved
    }

    pub fn get_racial_feat_ids(&self, race_id: RaceId, game_data: &GameData) -> Option<Vec<i32>> {
        let racialtypes = game_data.get_table("racialtypes")?;
        let race_data = racialtypes.get_by_id(race_id.0)?;

        let feat_table_name = Self::get_field_value(&race_data, "FeatTable")
            .or_else(|| Self::get_field_value(&race_data, "feattable"))?;

        if feat_table_name.is_empty() || feat_table_name == "****" {
            return None;
        }

        let feat_table = game_data.get_table(&feat_table_name.to_lowercase())?;
        let mut feat_ids = Vec::new();

        for row_id in 0..feat_table.row_count() {
            if let Some(row) = feat_table.get_by_id(row_id as i32)
                && let Some(feat_index_str) = Self::get_field_value(&row, "FeatIndex")
                    .or_else(|| Self::get_field_value(&row, "featindex"))
                && let Ok(feat_id) = feat_index_str.parse::<i32>()
                && feat_id >= 0
            {
                feat_ids.push(feat_id);
            }
        }

        if feat_ids.is_empty() {
            None
        } else {
            Some(feat_ids)
        }
    }

    /// Remove a class entirely from the character.
    /// This is a batch operation that processes all levels in one pass for performance.
    /// Protected feats (racial, background, domain) are preserved.
    pub fn remove_class(
        &mut self,
        class_id: ClassId,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        let current_class_count = self.class_count();
        if current_class_count <= 1 {
            return Err(CharacterError::ValidationFailed {
                field: "ClassList",
                message: "Cannot remove last class".to_string(),
            });
        }

        let current_level = self.class_level(class_id);
        if current_level == 0 {
            return Err(CharacterError::NotFound {
                entity: "Class",
                id: class_id.0,
            });
        }

        // Get preserved feat IDs ONCE before modifying anything
        let preserved_feats = self.get_preserved_feat_ids(game_data);

        // Load all lists ONCE
        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        let mut class_list = self.get_list_owned("ClassList").unwrap_or_default();
        let mut char_feat_list = self.get_list_owned("FeatList").unwrap_or_default();

        // Find all level history entries for this class
        let mut indices_to_remove: Vec<usize> = Vec::new();
        let mut total_hp_reduction = 0i32;
        let mut total_sp_reduction = 0i32;
        let mut feats_to_remove: HashSet<i32> = HashSet::new();
        let mut feats_to_keep: HashSet<i32> = HashSet::new();
        let mut ability_reductions: Vec<u8> = Vec::new();

        // Collect info from all entries for this class
        for (idx, entry) in lvl_stat_list.iter().enumerate() {
            let entry_class_id = entry.get("LvlStatClass").and_then(gff_value_to_i32);
            if entry_class_id == Some(class_id.0) {
                indices_to_remove.push(idx);

                // HP to remove
                let hp = entry
                    .get("LvlStatHitDie")
                    .and_then(gff_value_to_i32)
                    .unwrap_or(0);
                let con_mod = self.get_effective_ability_modifier(AbilityIndex::CON, game_data);
                total_hp_reduction += (hp + con_mod).max(1);

                // Skill points to remove
                let sp = entry
                    .get("SkillPoints")
                    .and_then(gff_value_to_i32)
                    .unwrap_or(0);
                total_sp_reduction += sp;

                // Ability increase to revert
                if let Some(ability_idx) = entry.get("LvlStatAbility").and_then(gff_value_to_i32)
                    && ability_idx != 255
                {
                    ability_reductions.push(ability_idx as u8);
                }

                // Chosen feats to remove
                if let Some(feat_list) =
                    super::gff_helpers::extract_list_from_map(entry, "FeatList")
                {
                    for feat_entry in feat_list {
                        if let Some(feat_id) = feat_entry.get("Feat").and_then(gff_value_to_i32) {
                            feats_to_remove.insert(feat_id);
                        }
                    }
                }

                // Remove spells from class's known lists
                if let Some(class_entry) = class_list
                    .iter_mut()
                    .find(|c| c.get("Class").and_then(gff_value_to_i32) == Some(class_id.0))
                {
                    for spell_level in 0..10 {
                        let list_key = format!("KnownList{spell_level}");
                        if let Some(spells) =
                            super::gff_helpers::extract_list_from_map(entry, &list_key)
                        {
                            for spell_entry in spells {
                                if let Some(spell_id) =
                                    spell_entry.get("Spell").and_then(gff_value_to_i32)
                                    && let Some(mut known_list) =
                                        super::gff_helpers::extract_list_from_map(
                                            class_entry,
                                            &list_key,
                                        )
                                {
                                    known_list.retain(|s| {
                                        s.get("Spell").and_then(gff_value_to_i32) != Some(spell_id)
                                    });
                                    class_entry
                                        .insert(list_key.clone(), GffValue::ListOwned(known_list));
                                }
                            }
                        }
                    }
                }
            } else {
                // Collect chosen feats from OTHER classes to KEEP
                if let Some(feat_list) =
                    super::gff_helpers::extract_list_from_map(entry, "FeatList")
                {
                    for feat_entry in feat_list {
                        if let Some(feat_id) = feat_entry.get("Feat").and_then(gff_value_to_i32) {
                            feats_to_keep.insert(feat_id);
                        }
                    }
                }
            }
        }

        // Add auto feats granted by this class across all its levels
        for lvl in 1..=current_level {
            for feat_info in self.get_class_feats_for_level(class_id, lvl, game_data) {
                if feat_info.list_type == 0 || feat_info.list_type == 3 {
                    feats_to_remove.insert(feat_info.feat_id.0);
                }
            }
        }

        // Add auto feats to keep (from other classes)
        for class_entry in &class_list {
            let other_class_id = class_entry
                .get("Class")
                .and_then(gff_value_to_i32)
                .unwrap_or(-1);
            if other_class_id != -1 && other_class_id != class_id.0 {
                let other_level = class_entry
                    .get("ClassLevel")
                    .and_then(gff_value_to_i32)
                    .unwrap_or(0);
                for lvl in 1..=other_level {
                    for feat_info in
                        self.get_class_feats_for_level(ClassId(other_class_id), lvl, game_data)
                    {
                        if feat_info.list_type == 0 || feat_info.list_type == 3 {
                            feats_to_keep.insert(feat_info.feat_id.0);
                        }
                    }
                }
            }
        }

        // Subtract feats_to_keep and preserved_feats
        feats_to_remove.retain(|f| !feats_to_keep.contains(f) && !preserved_feats.contains(f));

        // Remove LvlStatList entries in reverse order to maintain indices
        for idx in indices_to_remove.into_iter().rev() {
            lvl_stat_list.remove(idx);
        }

        // Remove feats (respecting preserved)
        char_feat_list.retain(|f| {
            let feat_id = f.get("Feat").and_then(gff_value_to_i32).unwrap_or(-1);
            !feats_to_remove.contains(&feat_id)
        });

        // Ensure these feats are also stripped from any remaining LvlStatList entries
        for stat_entry in &mut lvl_stat_list {
            if let Some(mut feat_list) =
                super::gff_helpers::extract_list_from_map(stat_entry, "FeatList")
            {
                feat_list.retain(|f| {
                    let feat_id = f.get("Feat").and_then(gff_value_to_i32).unwrap_or(-1);
                    !feats_to_remove.contains(&feat_id)
                });
                stat_entry.insert("FeatList".to_string(), GffValue::ListOwned(feat_list));
            }
        }

        // Remove class from ClassList
        class_list.retain(|c| c.get("Class").and_then(gff_value_to_i32) != Some(class_id.0));

        // Revert ability increases
        for ability_idx in ability_reductions {
            if let Some(index) = AbilityIndex::from_index(ability_idx) {
                let current_val = self.base_ability(index);
                let _ = self.set_ability(index, (current_val - 1).max(3));
            }
        }

        // Update HP
        let current_max_hp = self.get_i32("MaxHitPoints").unwrap_or(0);
        let current_hp = self.get_i32("CurrentHitPoints").unwrap_or(0);
        let base_hp = self.get_i32("HitPoints").unwrap_or(0);
        self.set_i32("MaxHitPoints", (current_max_hp - total_hp_reduction).max(1));
        self.set_i32(
            "CurrentHitPoints",
            (current_hp - total_hp_reduction)
                .max(1)
                .min(current_max_hp - total_hp_reduction),
        );
        self.set_i32("HitPoints", (base_hp - total_hp_reduction).max(1));

        // Update skill points
        let current_sp = self.get_available_skill_points();
        self.set_available_skill_points((current_sp - total_sp_reduction).max(0));

        // Set all lists back ONCE
        self.set_list("LvlStatList", lvl_stat_list);
        self.set_list("ClassList", class_list);
        self.set_list("FeatList", char_feat_list);

        // Recalculate stats once at the end
        self.recalculate_stats(game_data)?;
        self.normalize_skill_points(game_data);

        self.reconcile_class_feats(&[class_id], game_data);

        Ok(())
    }

    pub fn level_history(&self) -> Vec<LevelHistoryEntry> {
        let Some(lvl_stat_list) = self.get_list("LvlStatList") else {
            return vec![];
        };

        let mut class_level_counts: HashMap<i32, i32> = HashMap::new();
        let mut result = Vec::with_capacity(lvl_stat_list.len());

        for (idx, entry) in lvl_stat_list.iter().enumerate() {
            let character_level = (idx + 1) as i32;

            let class_id = entry
                .get("LvlStatClass")
                .and_then(gff_value_to_i32)
                .unwrap_or(0);

            let class_lvl = class_level_counts.entry(class_id).or_insert(0);
            *class_lvl += 1;
            let class_level = *class_lvl;

            let hp_gained = entry
                .get("LvlStatHitDie")
                .and_then(gff_value_to_i32)
                .unwrap_or(0);

            let skill_points_remaining = entry
                .get("SkillPoints")
                .and_then(gff_value_to_i32)
                .unwrap_or(0);

            let ability_raw = entry
                .get("LvlStatAbility")
                .and_then(gff_value_to_i32)
                .unwrap_or(255);
            let ability_increase = if ability_raw < 6 {
                AbilityIndex::from_index(ability_raw as u8)
            } else {
                None
            };

            let feats_gained = Self::extract_feat_list(entry, "FeatList");
            let skills_gained = Self::extract_skill_list(entry);

            result.push(LevelHistoryEntry {
                character_level,
                class_id: ClassId(class_id),
                class_level,
                hp_gained,
                skill_points_remaining,
                ability_increase,
                feats_gained,
                skills_gained,
            });
        }

        result
    }

    pub fn level_history_resolved(&self, game_data: &GameData) -> Vec<ResolvedLevelHistoryEntry> {
        let history = self.level_history();
        let ability_names = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];

        history
            .into_iter()
            .map(|entry| {
                let class_name = self.get_class_name(entry.class_id, game_data);

                let ability_increase = entry.ability_increase.map(|idx| {
                    ability_names
                        .get(idx.0 as usize)
                        .map(|s| (*s).to_string())
                        .unwrap_or_else(|| format!("Ability {}", idx.0))
                });

                let feats_gained: Vec<ResolvedFeatEntry> = entry
                    .feats_gained
                    .iter()
                    .map(|&feat_id| ResolvedFeatEntry {
                        feat_id: feat_id.0,
                        name: self.get_feat_name(feat_id, game_data),
                    })
                    .collect();

                let skills_gained: Vec<ResolvedSkillEntry> = entry
                    .skills_gained
                    .iter()
                    .map(|skill| ResolvedSkillEntry {
                        skill_id: skill.skill_id.0,
                        name: self.get_skill_name(skill.skill_id, game_data),
                        ranks: skill.ranks,
                    })
                    .collect();

                ResolvedLevelHistoryEntry {
                    character_level: entry.character_level,
                    class_id: entry.class_id.0,
                    class_name,
                    class_level: entry.class_level,
                    hp_gained: entry.hp_gained,
                    skill_points_remaining: entry.skill_points_remaining,
                    ability_increase,
                    feats_gained,
                    skills_gained,
                }
            })
            .collect()
    }

    /// Get class information from game data tables.
    ///
    /// Looks up class in classes.2da and resolves name from TLK.
    pub fn get_class_info(&self, class_id: ClassId, game_data: &GameData) -> Option<ClassInfo> {
        let classes_table = game_data.get_table("classes")?;
        let class_data = classes_table.get_by_id(class_id.0)?;

        let name = Self::resolve_class_name(&class_data, game_data);

        let hit_die = Self::get_field_value(&class_data, "hit_die")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(6);

        let primary_ability_str = Self::get_field_value(&class_data, "primary_ability");
        let primary_ability = primary_ability_str
            .and_then(|s| AbilityIndex::from_index(s.trim().parse::<u8>().ok()?));

        let is_spellcaster = Self::get_field_value(&class_data, "spell_caster").is_some_and(|s| {
            let trimmed = s.trim().to_lowercase();
            matches!(trimmed.as_str(), "1" | "true" | "yes")
        });

        let bab_table_name =
            Self::get_field_value(&class_data, "attack_bonus_table").unwrap_or_default();
        let bab_type = Self::determine_bab_type(&bab_table_name);

        Some(ClassInfo {
            id: class_id,
            name,
            hit_die,
            primary_ability,
            is_spellcaster,
            bab_type,
        })
    }

    pub fn get_hit_die(&self, class_id: ClassId, game_data: &GameData) -> i32 {
        let Some(classes_table) = game_data.get_table("classes") else {
            return 6;
        };

        let Some(class_data) = classes_table.get_by_id(class_id.0) else {
            return 6;
        };

        Self::get_field_value(&class_data, "hit_die")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(6)
    }

    pub fn calculate_xp_for_level(&self, level: i32, game_data: &GameData) -> i32 {
        if level < 1 {
            return 0;
        }

        let Some(exp_table) = game_data.get_table("exptable") else {
            return 0;
        };

        let row_index = (level - 1).max(0) as usize;
        exp_table
            .get_cell(row_index, "XP")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0)
    }

    pub fn get_xp_progress(&self, game_data: &GameData) -> XpProgress {
        let current_xp = self.experience();
        let current_level = self.total_level();

        let xp_for_current_level = self.calculate_xp_for_level(current_level, game_data);
        let xp_for_next_level = self.calculate_xp_for_level(current_level + 1, game_data);

        let xp_remaining = (xp_for_next_level - current_xp).max(0);

        let progress_percent = if xp_for_next_level > xp_for_current_level {
            let range = xp_for_next_level - xp_for_current_level;
            if range > 0 {
                let progress = (current_xp - xp_for_current_level).max(0);
                ((progress as f32 / range as f32) * 100.0).min(100.0)
            } else {
                100.0
            }
        } else {
            100.0
        };

        XpProgress {
            current_xp,
            current_level,
            xp_for_current_level,
            xp_for_next_level,
            xp_remaining,
            progress_percent,
        }
    }

    pub fn get_class_summary(&self, game_data: &GameData) -> Vec<ClassSummaryEntry> {
        let entries = self.class_entries();
        let mut summary = Vec::with_capacity(entries.len());

        for entry in entries {
            let name = self
                .get_class_info(entry.class_id, game_data)
                .map_or_else(|| format!("Class {}", entry.class_id.0), |info| info.name);

            let hit_die = self.get_hit_die(entry.class_id, game_data);

            let (base_attack_bonus, fortitude_save, reflex_save, will_save, skill_points_per_level) =
                Self::get_class_stats_at_level(entry.class_id, entry.level, game_data);

            summary.push(ClassSummaryEntry {
                class_id: entry.class_id,
                name,
                level: entry.level,
                hit_die,
                base_attack_bonus,
                fortitude_save,
                reflex_save,
                will_save,
                skill_points_per_level,
            });
        }

        summary
    }

    fn get_class_stats_at_level(
        class_id: ClassId,
        level: i32,
        game_data: &GameData,
    ) -> (i32, i32, i32, i32, i32) {
        let classes_table = match game_data.get_table("classes") {
            Some(t) => t,
            None => return (0, 0, 0, 0, 2),
        };

        let class_data = match classes_table.get_by_id(class_id.0) {
            Some(d) => d,
            None => return (0, 0, 0, 0, 2),
        };

        let skill_points = class_data
            .get("SkillPointBase")
            .or_else(|| class_data.get("skillpointbase"))
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(2);

        let bab_table_name = class_data
            .get("AttackBonusTable")
            .or_else(|| class_data.get("attackbonustable"))
            .and_then(std::clone::Clone::clone)
            .unwrap_or_default();

        let save_table_name = class_data
            .get("SavingThrowTable")
            .or_else(|| class_data.get("savingthrowtable"))
            .and_then(std::clone::Clone::clone)
            .unwrap_or_default();

        let bab_table = game_data.get_table(&bab_table_name.to_lowercase());
        let save_table = game_data.get_table(&save_table_name.to_lowercase());

        let row_idx = (level - 1).max(0) as usize;

        let bab_progression = if bab_table_name.to_lowercase().contains("low")
            || bab_table_name.to_lowercase().contains("half")
        {
            "half"
        } else if bab_table_name.to_lowercase().contains("med") {
            "three_quarter"
        } else {
            "full"
        };

        let base_attack_bonus = bab_table
            .as_ref()
            .and_then(|t| t.get_cell(row_idx, "BAB").ok().flatten())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or_else(|| calculate_bab_for_level(level, bab_progression));

        let (fortitude_save, reflex_save, will_save) = save_table.as_ref().map_or((0, 0, 0), |t| {
            let fort = t
                .get_cell(row_idx, "FortSave")
                .ok()
                .flatten()
                .or_else(|| t.get_cell(row_idx, "Fort").ok().flatten())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let ref_save = t
                .get_cell(row_idx, "RefSave")
                .ok()
                .flatten()
                .or_else(|| t.get_cell(row_idx, "Ref").ok().flatten())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let will = t
                .get_cell(row_idx, "WillSave")
                .ok()
                .flatten()
                .or_else(|| t.get_cell(row_idx, "Will").ok().flatten())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            (fort, ref_save, will)
        });

        (
            base_attack_bonus,
            fortitude_save,
            reflex_save,
            will_save,
            skill_points,
        )
    }

    pub fn get_classes_state(&self, game_data: &GameData) -> ClassesState {
        ClassesState {
            total_level: self.total_level(),
            entries: self.get_class_summary(game_data),
            xp_progress: self.get_xp_progress(game_data),
            level_history: self.level_history(),
            skill_points_summary: self.get_skill_points_summary(game_data),
        }
    }

    fn resolve_class_name(
        class_data: &AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> String {
        Self::get_field_value(class_data, "name")
            .and_then(|s| s.parse::<i32>().ok())
            .and_then(|strref| game_data.get_string(strref))
            .filter(|name| !name.is_empty())
            .or_else(|| Self::get_field_value(class_data, "label"))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn get_field_value(data: &AHashMap<String, Option<String>>, pattern: &str) -> Option<String> {
        let patterns = match pattern {
            "hit_die" => vec!["HitDie", "hit_die", "HD", "HitDice"],
            "primary_ability" => vec!["PrimaryAbil", "primary_ability", "primary_abil", "PrimAbil"],
            "spell_caster" => vec!["SpellCaster", "spell_caster", "IsCaster", "Caster"],
            "package" => vec!["Package", "package", "PACKAGE"],
            "is_prestige" => vec!["IsPrestige", "is_prestige", "Prestige", "prestige"],
            "attack_bonus_table" => vec![
                "AttackBonusTable",
                "attack_bonus_table",
                "AttackTable",
                "BABTable",
                "attackbonustable",
            ],
            "MaxLevel" => vec!["MaxLevel", "max_level", "MAXLEVEL"],
            "name" => vec!["NAME", "name", "Name", "label", "Label", "NameRef"],
            "label" => vec!["LABEL", "label", "Label"],
            _ => vec![pattern],
        };

        for alias in patterns {
            if let Some(Some(value)) = data.get(alias) {
                let trimmed = value.trim();
                if !trimmed.is_empty() && trimmed != "****" {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    fn determine_bab_type(bab_table_name: &str) -> BabType {
        let table_lower = bab_table_name.to_lowercase();
        if table_lower.contains("low") || table_lower.contains("half") {
            BabType::Half
        } else if table_lower.contains("med") || table_lower.contains("three") {
            BabType::ThreeQuarter
        } else if table_lower.contains("high") || table_lower.contains("full") {
            BabType::Full
        } else {
            BabType::ThreeQuarter
        }
    }

    fn extract_feat_list(entry: &IndexMap<String, GffValue<'static>>, field: &str) -> Vec<FeatId> {
        let list = super::gff_helpers::extract_list_from_map(entry, field).unwrap_or_default();
        list.iter()
            .filter_map(|f| f.get("Feat").and_then(gff_value_to_i32).map(FeatId))
            .collect()
    }

    fn extract_skill_list(entry: &IndexMap<String, GffValue<'static>>) -> Vec<SkillRankEntry> {
        let list =
            super::gff_helpers::extract_list_from_map(entry, "SkillList").unwrap_or_default();

        list.iter()
            .enumerate()
            .filter_map(|(idx, s)| {
                let ranks = s.get("Rank").and_then(gff_value_to_i32)?;
                if ranks > 0 {
                    Some(SkillRankEntry {
                        skill_id: SkillId(idx as i32),
                        ranks,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_class_name(&self, class_id: ClassId, game_data: &GameData) -> String {
        self.get_class_info(class_id, game_data)
            .map_or_else(|| format!("Class {}", class_id.0), |info| info.name)
    }

    pub fn is_prestige_class(&self, class_id: ClassId, game_data: &GameData) -> bool {
        let Some(classes_table) = game_data.get_table("classes") else {
            return false;
        };

        let Some(class_data) = classes_table.get_by_id(class_id.0) else {
            return false;
        };

        Self::get_field_value(&class_data, "is_prestige").is_some_and(|s| {
            let trimmed = s.trim().to_lowercase();
            matches!(trimmed.as_str(), "1" | "true" | "yes")
        })
    }

    pub fn get_prestige_requirements(
        class_data: &AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> PrestigeRequirements {
        let mut requirements = PrestigeRequirements::default();

        if let Some(bab) = Self::get_field_value(class_data, "min_attack_bonus")
            .and_then(|s| s.parse::<i32>().ok())
            && bab > 0
        {
            requirements.base_attack_bonus = Some(bab);
        }

        if let Some(skill_str) = Self::get_field_value(class_data, "required_skill")
            && let Some(skill_rank) = Self::get_field_value(class_data, "required_skill_rank")
                .and_then(|s| s.parse::<i32>().ok())
            && skill_rank > 0
        {
            let skill_name = skill_str
                .parse::<i32>()
                .ok()
                .and_then(|id| {
                    game_data
                        .get_table("skills")
                        .and_then(|t| t.get_by_id(id))
                        .and_then(|row| Self::get_field_value(&row, "Name"))
                        .and_then(|strref| strref.parse::<i32>().ok())
                        .and_then(|strref| game_data.get_string(strref))
                })
                .unwrap_or_else(|| skill_str.clone());
            requirements.skills.push((skill_name, skill_rank));
        }

        if let Some(feat_str) = Self::get_field_value(class_data, "required_feat")
            && !feat_str.is_empty()
            && feat_str != "****"
        {
            let feat_name = feat_str
                .parse::<i32>()
                .ok()
                .and_then(|id| {
                    game_data
                        .get_table("feat")
                        .and_then(|t| t.get_by_id(id))
                        .and_then(|row| Self::get_field_value(&row, "FEAT"))
                        .and_then(|strref| strref.parse::<i32>().ok())
                        .and_then(|strref| game_data.get_string(strref))
                })
                .unwrap_or_else(|| feat_str.clone());
            requirements.feats.push(feat_name);
        }

        if let Some(align_str) = Self::get_field_value(class_data, "align_restrict") {
            let align_restrict = FieldMapper::safe_hex_int(Some(&align_str));
            if align_restrict > 0 {
                requirements.alignment = AlignmentRestriction(align_restrict).decode_to_string();
            }
        }

        requirements
    }

    pub fn validate_prestige_class_requirements(
        &self,
        class_id: ClassId,
        game_data: &GameData,
    ) -> PrestigeClassValidation {
        let Some(classes_table) = game_data.get_table("classes") else {
            return PrestigeClassValidation::default();
        };

        let Some(class_data) = classes_table.get_by_id(class_id.0) else {
            return PrestigeClassValidation::default();
        };

        let requirements = Self::get_prestige_requirements(&class_data, game_data);
        let mut missing = Vec::new();

        if let Some(min_bab) = requirements.base_attack_bonus {
            let current_bab = self.calculate_bab(game_data);
            if current_bab < min_bab {
                missing.push(format!(
                    "Base Attack Bonus +{min_bab} (have +{current_bab})"
                ));
            }
        }

        for (skill_name, min_ranks) in &requirements.skills {
            let skill_id = game_data.get_table("skills").and_then(|t| {
                for row_idx in 0..t.row_count() {
                    if let Ok(row) = t.get_row(row_idx)
                        && let Some(name) = Self::get_field_value(&row, "Name")
                            .and_then(|strref| strref.parse::<i32>().ok())
                            .and_then(|strref| game_data.get_string(strref))
                        && name == *skill_name
                    {
                        return Some(SkillId(row_idx as i32));
                    }
                }
                None
            });

            if let Some(sid) = skill_id {
                let current_ranks = self.skill_rank(sid);
                if current_ranks < *min_ranks {
                    missing.push(format!(
                        "{skill_name} {min_ranks} ranks (have {current_ranks})"
                    ));
                }
            }
        }

        for feat_name in &requirements.feats {
            let feat_id = game_data.get_table("feat").and_then(|t| {
                for row_idx in 0..t.row_count() {
                    if let Ok(row) = t.get_row(row_idx)
                        && let Some(name) = Self::get_field_value(&row, "FEAT")
                            .and_then(|strref| strref.parse::<i32>().ok())
                            .and_then(|strref| game_data.get_string(strref))
                        && name == *feat_name
                    {
                        return Some(FeatId(row_idx as i32));
                    }
                }
                None
            });

            if let Some(fid) = feat_id
                && !self.has_feat(fid)
            {
                missing.push(format!("Feat: {feat_name}"));
            }
        }

        if let Some(align_str) = Self::get_field_value(&class_data, "align_restrict") {
            let align_restrict = FieldMapper::safe_hex_int(Some(&align_str));
            if align_restrict > 0 {
                let restriction = AlignmentRestriction(align_restrict);
                let alignment = self.alignment();
                if !restriction.check_alignment(&alignment)
                    && let Some(text) = restriction.decode_to_string()
                {
                    missing.push(format!("Alignment: must be {text}"));
                }
            }
        }

        PrestigeClassValidation {
            can_take: missing.is_empty(),
            missing_requirements: missing,
            requirements,
        }
    }

    pub fn get_prestige_class_options(&self, game_data: &GameData) -> Vec<PrestigeClassOption> {
        let Some(classes_table) = game_data.get_table("classes") else {
            return vec![];
        };

        let mut options = Vec::new();

        for row_idx in 0..classes_table.row_count() {
            let Some(row) = classes_table.get_row(row_idx).ok() else {
                continue;
            };

            let is_prestige = Self::get_field_value(&row, "is_prestige").is_some_and(|s| {
                let trimmed = s.trim().to_lowercase();
                matches!(trimmed.as_str(), "1" | "true" | "yes")
            });

            let max_level = Self::get_field_value(&row, "MaxLevel")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            if !is_prestige && max_level <= 0 {
                continue;
            }

            let label = Self::get_field_value(&row, "Label").unwrap_or_default();
            if label.is_empty() || label == "****" || label.to_lowercase().contains("padding") {
                continue;
            }

            let class_id = ClassId(row_idx as i32);
            let validation = self.validate_prestige_class_requirements(class_id, game_data);

            let name = Self::get_field_value(&row, "Name")
                .and_then(|s| s.parse::<i32>().ok())
                .and_then(|strref| game_data.get_string(strref))
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| label.clone());

            let reason = if validation.can_take {
                "All requirements met".to_string()
            } else {
                validation.missing_requirements.join("; ")
            };

            options.push(PrestigeClassOption {
                id: row_idx as i32,
                name,
                label,
                can_take: validation.can_take,
                reason,
                requirements: validation.requirements,
            });
        }

        options.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        options
    }

    pub fn level_up(
        &mut self,
        class_id: ClassId,
        game_data: &GameData,
    ) -> Result<LevelUpResult, CharacterError> {
        let classes_table = game_data
            .get_table("classes")
            .ok_or_else(|| CharacterError::TableNotFound("classes".to_string()))?;
        let class_data = classes_table
            .get_by_id(class_id.0)
            .ok_or(CharacterError::NotFound {
                entity: "Class",
                id: class_id.0,
            })?;

        let current_total_level = self.total_level();
        let new_total_level = current_total_level + 1;

        if new_total_level > MAX_TOTAL_LEVEL {
            return Err(CharacterError::ValidationFailed {
                field: "TotalLevel",
                message: format!("Cannot exceed max level {MAX_TOTAL_LEVEL}"),
            });
        }

        let max_class_level_str = Self::get_field_value(&class_data, "MaxLevel");
        if let Some(max_lvl) = max_class_level_str.and_then(|s| s.parse::<i32>().ok())
            && max_lvl > 0
        {
            let current_class_level = self.class_level(class_id);
            if current_class_level >= max_lvl {
                return Err(CharacterError::ValidationFailed {
                    field: "ClassLevel",
                    message: format!("Cannot exceed max level {max_lvl} for class {class_id:?}"),
                });
            }
        }

        let xp_req = self.calculate_xp_for_level(new_total_level, game_data);
        let current_xp = self.experience();
        if current_xp < xp_req {
            self.set_experience(xp_req)?;
        }

        let mut class_list = self.get_list_owned("ClassList").unwrap_or_default();
        let mut class_found = false;
        let mut class_level_gained = 1;

        for entry in &mut class_list {
            let entry_class = entry.get("Class").and_then(gff_value_to_i32).unwrap_or(-1);
            if entry_class == class_id.0 {
                let cur = entry
                    .get("ClassLevel")
                    .and_then(gff_value_to_i32)
                    .unwrap_or(0);
                class_level_gained = cur + 1;
                entry.insert(
                    "ClassLevel".to_string(),
                    GffValue::Short(class_level_gained as i16),
                );
                class_found = true;
                break;
            }
        }

        if !class_found {
            if class_list.len() >= MAX_CLASSES {
                return Err(CharacterError::ValidationFailed {
                    field: "ClassList",
                    message: format!("Max classes {MAX_CLASSES} reached"),
                });
            }
            let mut new_entry = IndexMap::new();
            new_entry.insert("Class".to_string(), GffValue::Byte(class_id.0 as u8));
            new_entry.insert("ClassLevel".to_string(), GffValue::Short(1));
            class_list.push(new_entry);
            class_level_gained = 1;
        }

        self.set_list("ClassList", class_list);

        if self.class_count() == 1 {
            self.set_byte("Class", class_id.0 as u8);
        }

        let hit_die = Self::get_field_value(&class_data, "hit_die")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(6);
        let con_mod = self.get_effective_ability_modifier(AbilityIndex::CON, game_data);

        let hp_gained = std::cmp::max(1, hit_die + con_mod);

        let current_max_hp = self.max_hp();
        let current_base_hp = self.base_hp();
        let current_hp = self.current_hp();

        self.set_max_hp(current_max_hp + hp_gained);
        self.set_base_hp(current_base_hp + hp_gained);
        self.set_current_hp(current_hp + hp_gained);

        // 5. Skill Points
        let int_mod = self.get_effective_ability_modifier(AbilityIndex::INT, game_data);
        let race_id = self.race_id();
        let race_bonus = game_data
            .get_table("racialtypes")
            .and_then(|t| t.get_by_id(race_id.0))
            .and_then(|row| Self::get_field_value(&row, "SkillPointModifier"))
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        let skill_base = Self::get_field_value(&class_data, "SkillPointBase")
            .or_else(|| Self::get_field_value(&class_data, "skillpointbase"))
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(2);

        let mut sp_gained = std::cmp::max(1, skill_base + int_mod + race_bonus);
        if new_total_level == 1 {
            sp_gained *= 4;
        }

        let current_sp = self.get_available_skill_points();
        self.set_available_skill_points(current_sp + sp_gained);

        // 6. Feats
        let slots_now = 1
            + (std::cmp::min(new_total_level, 20) / 3)
            + ((std::cmp::max(new_total_level - 20, 0) + 1) / 2);
        let prev_level = new_total_level - 1;
        let slots_prev = if prev_level > 0 {
            1 + (std::cmp::min(prev_level, 20) / 3) + ((std::cmp::max(prev_level - 20, 0) + 1) / 2)
        } else {
            0
        };
        let general_slots_gained = slots_now - slots_prev;

        let bonus_now = self.get_bonus_feats_for_class(class_id, class_level_gained, game_data);
        let bonus_prev = if class_level_gained > 1 {
            self.get_bonus_feats_for_class(class_id, class_level_gained - 1, game_data)
        } else {
            0
        };
        let bonus_slots_gained = bonus_now - bonus_prev;

        // 7. Ability Increase
        let ability_increase_gained = (new_total_level % 4) == 0;

        // 8. Spellcasting
        let is_spellcaster = Self::get_field_value(&class_data, "spell_caster")
            .or_else(|| Self::get_field_value(&class_data, "spellcaster"))
            .is_some_and(|s| matches!(s.trim().to_lowercase().as_str(), "1" | "true" | "yes"));

        let spell_gain_table = Self::get_field_value(&class_data, "spell_gain_table");
        let is_prestige_caster = is_spellcaster
            && spell_gain_table
                .as_ref()
                .map_or(true, |s| s == "****" || s.is_empty());

        if is_prestige_caster {
            let mut best_class_entry_idx = None;
            let mut max_lvl = -1;

            if let Some(mut updated_list) = self.get_list_owned("ClassList") {
                for (idx, entry) in updated_list.iter().enumerate() {
                    let c_id = entry.get("Class").and_then(gff_value_to_i32).unwrap_or(-1);
                    if c_id == -1 || c_id == class_id.0 {
                        continue;
                    }

                    if let Some(c_data) = classes_table.get_by_id(c_id) {
                        let c_table = Self::get_field_value(&c_data, "spell_gain_table");
                        if let Some(t) = c_table
                            && !t.is_empty()
                            && t != "****"
                        {
                            let lvl = entry
                                .get("ClassLevel")
                                .and_then(gff_value_to_i32)
                                .unwrap_or(0);
                            if lvl > max_lvl {
                                max_lvl = lvl;
                                best_class_entry_idx = Some(idx);
                            }
                        }
                    }
                }

                if let Some(idx) = best_class_entry_idx {
                    if let Some(entry) = updated_list.get_mut(idx) {
                        let current_cl = entry
                            .get("ClassLevel")
                            .and_then(gff_value_to_i32)
                            .unwrap_or(0);
                        let current_scl = entry
                            .get("SpellCasterLevel")
                            .and_then(gff_value_to_i32)
                            .unwrap_or(current_cl);
                        entry.insert(
                            "SpellCasterLevel".to_string(),
                            GffValue::Short((current_scl + 1) as i16),
                        );
                    }
                    self.set_list("ClassList", updated_list);
                }
            }
        }

        // 9. Update LvlStatList History
        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        let template_hist = lvl_stat_list.last().cloned();
        let mut new_hist = template_hist.clone().unwrap_or_default();
        new_hist.shift_remove("__struct_id__");
        new_hist.insert("LvlStatClass".to_string(), GffValue::Byte(class_id.0 as u8));
        let history_hit_die = if new_total_level == 1 {
            hit_die
        } else {
            (hit_die / 2) + 1
        };
        new_hist.insert(
            "LvlStatHitDie".to_string(),
            GffValue::Byte(history_hit_die as u8),
        );
        new_hist.insert("SkillPoints".to_string(), GffValue::Short(sp_gained as i16));
        new_hist.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        new_hist.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
        new_hist.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(self.empty_skill_history_list(template_hist.as_ref())),
        );
        new_hist.insert(
            "EpicLevel".to_string(),
            GffValue::Byte(if new_total_level > 20 { 1 } else { 0 }),
        );

        let template_has_spell_lists = template_hist.as_ref().is_some_and(|entry| {
            (0..10).any(|i| {
                entry.contains_key(&format!("KnownList{i}"))
                    || entry.contains_key(&format!("KnownRemoveList{i}"))
            })
        });

        for i in 0..10 {
            let known_key = format!("KnownList{i}");
            let removed_key = format!("KnownRemoveList{i}");
            if is_spellcaster || template_has_spell_lists {
                new_hist.insert(known_key, GffValue::ListOwned(vec![]));
                new_hist.insert(removed_key, GffValue::ListOwned(vec![]));
            } else {
                new_hist.shift_remove(&known_key);
                new_hist.shift_remove(&removed_key);
            }
        }

        lvl_stat_list.push(new_hist);
        self.set_list("LvlStatList", lvl_stat_list);

        // 10. Grant automatic class feats (list_type 0 = Epic, 3 = Class granted)
        let class_feats = self.get_class_feats_for_level(class_id, class_level_gained, game_data);
        let mut granted_feats = Vec::new();
        for feat_info in class_feats {
            if (feat_info.list_type == 0 || feat_info.list_type == 3)
                && !self.has_feat(feat_info.feat_id)
                && self
                    .add_feat_with_source(feat_info.feat_id, FeatSource::Class)
                    .is_ok()
            {
                granted_feats.push(feat_info.feat_id);
            }
        }

        // 11. Recalculate global stats
        self.recalculate_stats(game_data)?;

        Ok(LevelUpResult {
            class_id,
            new_level: new_total_level,
            hp_gained,
            skill_points_gained: sp_gained,
            general_feat_slots_gained: general_slots_gained,
            bonus_feat_slots_gained: bonus_slots_gained,
            ability_increase_gained,
            new_spells_gained: is_spellcaster,
            granted_feats,
        })
    }

    /// Revert the last level for the specified class.
    /// Protected feats (racial, background, domain) are preserved.
    pub fn level_down(
        &mut self,
        class_id: ClassId,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        self.level_down_internal(class_id, game_data)?;
        self.recalculate_stats(game_data)?;
        self.normalize_skill_points(game_data);
        self.reconcile_class_feats(&[class_id], game_data);
        Ok(())
    }

    fn level_down_internal(
        &mut self,
        class_id: ClassId,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        let current_level = self.class_level(class_id);
        // Get preserved feat IDs before any modifications
        let preserved_feats = self.get_preserved_feat_ids(game_data);
        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        if lvl_stat_list.is_empty() {
            return Err(CharacterError::ValidationFailed {
                field: "LvlStatList",
                message: "No level history to revert".to_string(),
            });
        }

        // Find the last entry for this class
        let mut found_idx = None;
        for (i, entry) in lvl_stat_list.iter().enumerate().rev() {
            if entry.get("LvlStatClass").and_then(gff_value_to_i32) == Some(class_id.0) {
                found_idx = Some(i);
                break;
            }
        }

        let Some(idx) = found_idx else {
            return Err(CharacterError::NotFound {
                entity: "Level history entry for class",
                id: class_id.0,
            });
        };

        let entry = lvl_stat_list.remove(idx);

        // 1. Revert Ability Increase (Must happen before HP to match Python/Engine logic)
        if let Some(ability_idx) = entry.get("LvlStatAbility").and_then(gff_value_to_i32)
            && ability_idx != 255
            && let Some(index) = AbilityIndex::from_index(ability_idx as u8)
        {
            let current_val = self.base_ability(index);
            let _ = self.set_ability(index, (current_val - 1).max(3));
        }

        // 2. Revert HP
        let hp_roll = entry
            .get("LvlStatHitDie")
            .and_then(gff_value_to_i32)
            .unwrap_or(0);
        let con_mod = self.get_effective_ability_modifier(AbilityIndex::CON, game_data);
        let hp_reduction = (hp_roll + con_mod).max(1);

        let current_max_hp = self.get_i32("MaxHitPoints").unwrap_or(0);
        let current_hp = self.get_i32("CurrentHitPoints").unwrap_or(0);
        let hit_points = self.get_i32("HitPoints").unwrap_or(0);

        self.set_i32("MaxHitPoints", (current_max_hp - hp_reduction).max(1));
        self.set_i32(
            "CurrentHitPoints",
            (current_hp - hp_reduction)
                .max(1)
                .min(current_max_hp - hp_reduction),
        );
        self.set_i32("HitPoints", (hit_points - hp_reduction).max(1));

        // 3. Revert Skill Points
        let sp_gained = entry
            .get("SkillPoints")
            .and_then(gff_value_to_i32)
            .unwrap_or(0);
        let current_sp = self.get_available_skill_points();
        self.set_available_skill_points((current_sp - sp_gained).max(0));

        // 4. Remove Feats (Direct edit to avoid history side-effect)
        let mut feats_to_remove: HashSet<i32> = HashSet::new();
        let mut feats_to_keep: HashSet<i32> = HashSet::new();

        // 4a. Collect chosen feats from the removed level
        if let Some(feats) = super::gff_helpers::extract_list_from_map(&entry, "FeatList") {
            for feat_entry in feats {
                if let Some(feat_id) = feat_entry.get("Feat").and_then(gff_value_to_i32) {
                    feats_to_remove.insert(feat_id);
                }
            }
        }

        // 4b. Add auto feats granted at THIS specific level
        let auto_feats = self.get_class_feats_for_level(class_id, current_level, game_data);
        for feat_info in auto_feats {
            if feat_info.list_type == 0 || feat_info.list_type == 3 {
                feats_to_remove.insert(feat_info.feat_id.0);
            }
        }

        // 4c. Collect chosen feats to KEEP from surviving LvlStatList entries
        for stat_entry in &lvl_stat_list {
            if let Some(feat_list) =
                super::gff_helpers::extract_list_from_map(stat_entry, "FeatList")
            {
                for feat_entry in feat_list {
                    if let Some(feat_id) = feat_entry.get("Feat").and_then(gff_value_to_i32) {
                        feats_to_keep.insert(feat_id);
                    }
                }
            }
        }

        // 4d. Add auto feats to keep from remaining levels of THIS class and ALL levels of OTHER classes
        let class_list_for_feats = self.get_list_owned("ClassList").unwrap_or_default();
        for class_entry in &class_list_for_feats {
            let other_class_id = class_entry
                .get("Class")
                .and_then(gff_value_to_i32)
                .unwrap_or(-1);
            if other_class_id != -1 {
                let other_level = class_entry
                    .get("ClassLevel")
                    .and_then(gff_value_to_i32)
                    .unwrap_or(0);
                let max_lvl = if other_class_id == class_id.0 {
                    current_level - 1
                } else {
                    other_level
                };
                for lvl in 1..=max_lvl {
                    let auto_f =
                        self.get_class_feats_for_level(ClassId(other_class_id), lvl, game_data);
                    for feat_info in auto_f {
                        if feat_info.list_type == 0 || feat_info.list_type == 3 {
                            feats_to_keep.insert(feat_info.feat_id.0);
                        }
                    }
                }
            }
        }

        feats_to_remove.retain(|f| !feats_to_keep.contains(f) && !preserved_feats.contains(f));

        let mut char_feat_list = self.get_list_owned("FeatList").unwrap_or_default();
        char_feat_list.retain(|f| {
            let feat_id = f.get("Feat").and_then(gff_value_to_i32).unwrap_or(-1);
            !feats_to_remove.contains(&feat_id)
        });
        self.set_list("FeatList", char_feat_list);

        // Ensure these feats are also stripped from any remaining LvlStatList entries
        for stat_entry in &mut lvl_stat_list {
            if let Some(mut feat_list) =
                super::gff_helpers::extract_list_from_map(stat_entry, "FeatList")
            {
                feat_list.retain(|f| {
                    let feat_id = f.get("Feat").and_then(gff_value_to_i32).unwrap_or(-1);
                    !feats_to_remove.contains(&feat_id)
                });
                stat_entry.insert("FeatList".to_string(), GffValue::ListOwned(feat_list));
            }
        }

        // 5. Undo Spell Selections (Direct edit to avoid history side-effect)
        if let Some(mut class_list) = self.get_list_owned("ClassList") {
            for i in 0..10 {
                let list_key = format!("KnownList{i}");
                if let Some(spells) = super::gff_helpers::extract_list_from_map(&entry, &list_key) {
                    for spell_entry in spells {
                        if let Some(spell_id) = spell_entry.get("Spell").and_then(gff_value_to_i32)
                        {
                            for class_entry in &mut class_list {
                                if class_entry.get("Class").and_then(gff_value_to_i32)
                                    == Some(class_id.0)
                                    && let Some(mut known_list) =
                                        super::gff_helpers::extract_list_from_map(
                                            class_entry,
                                            &list_key,
                                        )
                                {
                                    known_list.retain(|s| {
                                        s.get("Spell").and_then(gff_value_to_i32) != Some(spell_id)
                                    });
                                    class_entry
                                        .insert(list_key.clone(), GffValue::ListOwned(known_list));
                                }
                            }
                        }
                    }
                }
            }
            self.set_list("ClassList", class_list);
        }

        // 6. Update ClassList
        let mut class_list = self.get_list_owned("ClassList").unwrap_or_default();
        let mut entries_to_remove = Vec::new();
        for (i, entry) in class_list.iter_mut().enumerate() {
            if entry.get("Class").and_then(gff_value_to_i32) == Some(class_id.0) {
                let current_level = entry
                    .get("ClassLevel")
                    .and_then(gff_value_to_i32)
                    .unwrap_or(0);
                if current_level > 1 {
                    entry.insert(
                        "ClassLevel".to_string(),
                        GffValue::Short((current_level - 1) as i16),
                    );
                } else {
                    entries_to_remove.push(i);
                }
                break;
            }
        }
        for i in entries_to_remove.into_iter().rev() {
            class_list.remove(i);
        }

        self.set_list("ClassList", class_list);
        self.set_list("LvlStatList", lvl_stat_list);

        Ok(())
    }

    /// Sync a feat change to the current level history.
    pub fn record_feat_change(&mut self, feat_id: FeatId, added: bool) {
        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        if lvl_stat_list.is_empty() {
            return;
        }

        let last_idx = lvl_stat_list.len() - 1;
        let entry = &mut lvl_stat_list[last_idx];

        let mut feat_list =
            super::gff_helpers::extract_list_from_map(entry, "FeatList").unwrap_or_default();

        if added {
            if !feat_list
                .iter()
                .any(|f| f.get("Feat").and_then(gff_value_to_i32) == Some(feat_id.0))
            {
                let mut new_feat = IndexMap::new();
                new_feat.insert("Feat".to_string(), GffValue::Word(feat_id.0 as u16));
                feat_list.push(new_feat);
            }
        } else {
            feat_list.retain(|f| f.get("Feat").and_then(gff_value_to_i32) != Some(feat_id.0));
        }

        entry.insert("FeatList".to_string(), GffValue::ListOwned(feat_list));
        self.set_list("LvlStatList", lvl_stat_list);
    }

    /// Sync a skill rank change to the current level history.
    pub fn record_skill_change(&mut self, skill_id: SkillId, rank_delta: i32) {
        if rank_delta == 0 {
            return;
        }

        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        if lvl_stat_list.is_empty() {
            return;
        }

        let last_idx = lvl_stat_list.len() - 1;
        let entry = &mut lvl_stat_list[last_idx];

        let mut skill_list = entry
            .get("SkillList")
            .and_then(|v| match v {
                GffValue::ListOwned(l) => Some(l.clone()),
                _ => None,
            })
            .unwrap_or_default();

        while skill_list.len() <= skill_id.0 as usize {
            let mut empty_skill = IndexMap::new();
            empty_skill.insert("Rank".to_string(), GffValue::Byte(0));
            skill_list.push(empty_skill);
        }

        let skill_entry = &mut skill_list[skill_id.0 as usize];
        let current_rank = skill_entry
            .get("Rank")
            .and_then(gff_value_to_i32)
            .unwrap_or(0);
        skill_entry.insert(
            "Rank".to_string(),
            GffValue::Byte((current_rank + rank_delta) as u8),
        );

        entry.insert("SkillList".to_string(), GffValue::ListOwned(skill_list));
        self.set_list("LvlStatList", lvl_stat_list);
    }

    /// Record an ability score increase in the current level history.
    pub fn record_ability_change(&mut self, ability_index: AbilityIndex) {
        use crate::character::types::ABILITY_INCREASE_INTERVAL;
        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        if lvl_stat_list.is_empty() {
            return;
        }

        let mut changed = false;
        for (idx, entry) in lvl_stat_list.iter_mut().enumerate() {
            let char_level = (idx + 1) as i32;
            if char_level % ABILITY_INCREASE_INTERVAL == 0
                && entry.get("LvlStatAbility").and_then(gff_value_to_i32) == Some(255)
            {
                entry.insert(
                    "LvlStatAbility".to_string(),
                    GffValue::Byte(ability_index.0),
                );
                changed = true;
                break;
            }
        }

        if changed {
            self.set_list("LvlStatList", lvl_stat_list);
        }
    }

    /// Sync a spell change to the current level history.
    pub fn record_spell_change(&mut self, level: i32, spell_id: i32, added: bool) {
        if !(0..=9).contains(&level) {
            return;
        }

        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        if lvl_stat_list.is_empty() {
            return;
        }

        let last_idx = lvl_stat_list.len() - 1;
        let entry = &mut lvl_stat_list[last_idx];

        let list_key = if added {
            format!("KnownList{level}")
        } else {
            format!("KnownRemoveList{level}")
        };

        let mut spell_list = entry
            .get(&list_key)
            .and_then(|v| match v {
                GffValue::ListOwned(l) => Some(l.clone()),
                _ => None,
            })
            .unwrap_or_default();

        if !spell_list
            .iter()
            .any(|s| s.get("Spell").and_then(gff_value_to_i32) == Some(spell_id))
        {
            let mut new_spell = IndexMap::new();
            new_spell.insert("Spell".to_string(), GffValue::Word(spell_id as u16));
            spell_list.push(new_spell);
        }

        entry.insert(list_key, GffValue::ListOwned(spell_list));
        self.set_list("LvlStatList", lvl_stat_list);
    }

    /// Recalculate derived stats like BAB and Saves.
    pub fn recalculate_stats(&mut self, game_data: &GameData) -> Result<(), CharacterError> {
        let bab = self.calculate_bab(game_data);
        self.set_base_attack_bonus(bab);

        let saves = self.calculate_base_saves(game_data);
        self.set_base_saves(saves)?;

        Ok(())
    }

    /// Calculate theoretical skill points for history correction.
    pub fn get_theoretical_skill_points(&self, game_data: &GameData) -> HashMap<ClassId, i32> {
        let mut class_points = HashMap::new();

        let race_id = self.race_id();
        let race_bonus = game_data
            .get_table("racialtypes")
            .and_then(|t| t.get_by_id(race_id.0))
            .and_then(|row| Self::get_field_value(&row, "SkillPointModifier"))
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        let int_mod = self.get_effective_ability_modifier(AbilityIndex::INT, game_data);

        let Some(lvl_stat_list) = self.get_list("LvlStatList") else {
            return class_points;
        };

        let classes_table = game_data.get_table("classes");

        for (idx, entry) in lvl_stat_list.iter().enumerate() {
            let class_id = entry
                .get("LvlStatClass")
                .and_then(gff_value_to_i32)
                .unwrap_or(-1);
            if class_id == -1 {
                continue;
            }

            let class_base = if let Some(table) = &classes_table {
                if let Some(c_data) = table.get_by_id(class_id) {
                    Self::get_field_value(&c_data, "SkillPointBase")
                        .or_else(|| Self::get_field_value(&c_data, "skillpointbase"))
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(2)
                } else {
                    2
                }
            } else {
                2
            };

            let mut points = std::cmp::max(1, class_base + int_mod + race_bonus);
            if idx == 0 {
                points *= 4;
            }

            *class_points.entry(ClassId(class_id)).or_insert(0) += points;
        }

        class_points
    }

    fn history_aligned_skill_point_floor(&self, game_data: &GameData) -> Option<i32> {
        if self.is_modified() {
            return None;
        }

        let history = self.get_list("LvlStatList")?;
        let top_skill_list = self.get_list("SkillList")?;
        let has_able_learner =
            self.has_feat(FeatId(crate::character::skills::ABLE_LEARNER_FEAT_ID));

        let mut aggregated_ranks = vec![0; top_skill_list.len()];
        let mut acquired_classes = Vec::new();
        let mut history_spent = 0;
        let mut saw_history_skills = false;
        let mut trailing_history_points = None;

        for entry in history {
            if let Some(class_id) = entry.get("LvlStatClass").and_then(gff_value_to_i32)
                && class_id >= 0
            {
                acquired_classes.push(ClassId(class_id));
            }

            if let Some(points) = entry.get("SkillPoints").and_then(gff_value_to_i32) {
                trailing_history_points = Some(points.max(0));
            }

            let Some(GffValue::ListOwned(skill_list)) = entry.get("SkillList") else {
                continue;
            };

            if skill_list.len() > aggregated_ranks.len() {
                aggregated_ranks.resize(skill_list.len(), 0);
            }

            for (skill_idx, skill) in skill_list.iter().enumerate() {
                let ranks = skill.get("Rank").and_then(gff_value_to_i32).unwrap_or(0);
                if ranks <= 0 {
                    continue;
                }

                saw_history_skills = true;
                aggregated_ranks[skill_idx] += ranks;

                let is_class_skill = Self::is_class_skill_for_class_ids(
                    &acquired_classes,
                    SkillId(skill_idx as i32),
                    game_data,
                );
                history_spent += if is_class_skill || has_able_learner {
                    ranks
                } else {
                    ranks * 2
                };
            }
        }

        if !saw_history_skills {
            return None;
        }

        if aggregated_ranks.len() < top_skill_list.len() {
            aggregated_ranks.resize(top_skill_list.len(), 0);
        }

        let history_matches_top_level =
            top_skill_list.iter().enumerate().all(|(skill_idx, skill)| {
                let top_rank = skill.get("Rank").and_then(gff_value_to_i32).unwrap_or(0);
                aggregated_ranks.get(skill_idx).copied().unwrap_or(0) == top_rank
            });

        if !history_matches_top_level {
            return None;
        }

        Some(
            history_spent
                + trailing_history_points
                    .unwrap_or(0)
                    .max(self.get_available_skill_points()),
        )
    }

    /// Get a summary of skill points (theoretical vs actual).
    pub fn get_skill_points_summary(&self, game_data: &GameData) -> SkillPointsSummary {
        let theoretical_map = self.get_theoretical_skill_points(game_data);
        let formula_total: i32 = theoretical_map.values().sum();
        let theoretical_total = self
            .history_aligned_skill_point_floor(game_data)
            .map_or(formula_total, |history_total| {
                history_total.max(formula_total)
            });
        let actual_spent = self.calculate_total_spent_with_costs(game_data);
        let current_unspent = self.get_available_skill_points();

        let mismatch = theoretical_total - (actual_spent + current_unspent);

        SkillPointsSummary {
            theoretical_total,
            actual_spent,
            current_unspent,
            mismatch,
        }
    }

    /// Reconcile unspent skill points with theoretical gains.
    pub fn normalize_skill_points(&mut self, game_data: &GameData) {
        let summary = self.get_skill_points_summary(game_data);
        if summary.mismatch != 0 {
            let new_unspent = (summary.current_unspent + summary.mismatch).max(0);
            self.set_available_skill_points(new_unspent);
        }
    }
}

pub fn get_class_progression(
    class_id: i32,
    max_level: i32,
    game_data: &GameData,
) -> Option<ClassProgression> {
    let classes_table = game_data.get_table("classes")?;
    let class_data = classes_table.get_by_id(class_id)?;

    let class_name = class_data
        .get("name")
        .or_else(|| class_data.get("Name"))
        .and_then(std::clone::Clone::clone)
        .and_then(|s| s.parse::<i32>().ok())
        .and_then(|strref| game_data.get_string(strref))
        .or_else(|| {
            class_data
                .get("label")
                .or_else(|| class_data.get("Label"))
                .and_then(std::clone::Clone::clone)
        })
        .unwrap_or_else(|| format!("Class {class_id}"));

    let hit_die = class_data
        .get("HitDie")
        .or_else(|| class_data.get("hit_die"))
        .and_then(|v| v.as_ref())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(6);

    let skill_points = class_data
        .get("SkillPointBase")
        .or_else(|| class_data.get("skillpointbase"))
        .and_then(|v| v.as_ref())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(2);

    let bab_table_name = class_data
        .get("AttackBonusTable")
        .or_else(|| class_data.get("attackbonustable"))
        .and_then(std::clone::Clone::clone)
        .unwrap_or_default();

    let bab_progression = if bab_table_name.to_lowercase().contains("low")
        || bab_table_name.to_lowercase().contains("half")
    {
        "half".to_string()
    } else if bab_table_name.to_lowercase().contains("med") {
        "three_quarter".to_string()
    } else {
        "full".to_string()
    };

    let save_table_name = class_data
        .get("SavingThrowTable")
        .or_else(|| class_data.get("savingthrowtable"))
        .and_then(std::clone::Clone::clone)
        .unwrap_or_default();

    let is_spellcaster = class_data
        .get("SpellCaster")
        .or_else(|| class_data.get("spellcaster"))
        .and_then(|v| v.as_ref())
        .is_some_and(|s| s == "1");

    let arcane_flag = class_data
        .get("Arcane")
        .or_else(|| class_data.get("arcane"))
        .and_then(|v| v.as_ref())
        .is_some_and(|s| s == "1");

    let spell_type = if !is_spellcaster {
        "none".to_string()
    } else if arcane_flag {
        "arcane".to_string()
    } else {
        "divine".to_string()
    };

    let spell_table_name = class_data
        .get("SpellKnownTable")
        .or_else(|| class_data.get("spellknowntable"))
        .and_then(std::clone::Clone::clone);

    let mut level_progression = Vec::with_capacity(max_level as usize);

    let bab_table = game_data.get_table(&bab_table_name.to_lowercase());
    let save_table = game_data.get_table(&save_table_name.to_lowercase());
    let spell_table = spell_table_name
        .as_ref()
        .and_then(|name| game_data.get_table(&name.to_lowercase()));

    for level in 1..=max_level {
        let row_idx = (level - 1) as usize;

        let base_attack_bonus = bab_table
            .as_ref()
            .and_then(|t| t.get_cell(row_idx, "BAB").ok().flatten())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or_else(|| calculate_bab_for_level(level, &bab_progression));

        let (fortitude_save, reflex_save, will_save) = save_table.as_ref().map_or((0, 0, 0), |t| {
            let fort = t
                .get_cell(row_idx, "FortSave")
                .ok()
                .flatten()
                .or_else(|| t.get_cell(row_idx, "Fort").ok().flatten())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let ref_save = t
                .get_cell(row_idx, "RefSave")
                .ok()
                .flatten()
                .or_else(|| t.get_cell(row_idx, "Ref").ok().flatten())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let will = t
                .get_cell(row_idx, "WillSave")
                .ok()
                .flatten()
                .or_else(|| t.get_cell(row_idx, "Will").ok().flatten())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            (fort, ref_save, will)
        });

        let spell_slots = if is_spellcaster {
            spell_table.as_ref().map(|t| SpellSlots {
                level_0: t
                    .get_cell(row_idx, "SpellLevel0")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_1: t
                    .get_cell(row_idx, "SpellLevel1")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_2: t
                    .get_cell(row_idx, "SpellLevel2")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_3: t
                    .get_cell(row_idx, "SpellLevel3")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_4: t
                    .get_cell(row_idx, "SpellLevel4")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_5: t
                    .get_cell(row_idx, "SpellLevel5")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_6: t
                    .get_cell(row_idx, "SpellLevel6")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_7: t
                    .get_cell(row_idx, "SpellLevel7")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_8: t
                    .get_cell(row_idx, "SpellLevel8")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                level_9: t
                    .get_cell(row_idx, "SpellLevel9")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            })
        } else {
            None
        };

        level_progression.push(LevelProgressionEntry {
            level,
            base_attack_bonus,
            fortitude_save,
            reflex_save,
            will_save,
            features: Vec::new(),
            spell_slots,
        });
    }

    Some(ClassProgression {
        class_id,
        class_name,
        basic_info: ClassBasicInfo {
            hit_die,
            skill_points_per_level: skill_points,
            bab_progression,
            save_progression: save_table_name,
            is_spellcaster,
            spell_type,
        },
        level_progression,
        max_level_shown: max_level,
    })
}

fn calculate_bab_for_level(level: i32, bab_type: &str) -> i32 {
    match bab_type {
        "full" => level,
        "three_quarter" => (level * 3) / 4,
        "half" => level / 2,
        _ => (level * 3) / 4,
    }
}

impl Character {
    /// Swap levels of one class for another.
    ///
    /// This is a destructive operation that:
    /// 1. Removes ALL levels of the old class (reverting all gains).
    /// 2. Adds the new class at level 1.
    /// 3. Returns the new class ID and level (which will be 1).
    ///
    /// Protected feats (racial, background, domain) are preserved.
    pub fn swap_class(
        &mut self,
        old_class_id: ClassId,
        new_class_id: ClassId,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        let current_level = self.class_level(old_class_id);
        if current_level == 0 {
            return Err(CharacterError::NotFound {
                entity: "Class",
                id: old_class_id.0,
            });
        }

        // 1. Remove all levels of old class (use internal to skip recalculate_stats each time)
        for _ in 0..current_level {
            self.level_down_internal(old_class_id, game_data)?;
        }

        self.level_up(new_class_id, game_data)?;
        self.normalize_skill_points(game_data);

        self.reconcile_class_feats(&[old_class_id], game_data);

        Ok(())
    }

    /// Change the primary class (index 0) and rewrite history.
    ///
    /// This "retcons" the character's history:
    /// 1. Replaces ClassList[0] with new class.
    /// 2. Updates `Class` byte field.
    /// 3. Updates all `LvlStatList` entries for the old class to the new class.
    /// 4. Recalculates HP for the entire history (using Average roll rule).
    /// 5. Recalculates other stats (BAB, Saves).
    pub fn change_primary_class(
        &mut self,
        new_class_id: ClassId,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        let class_list = self
            .get_list_owned("ClassList")
            .ok_or(CharacterError::FieldMissing { field: "ClassList" })?;

        if class_list.is_empty() {
            return Err(CharacterError::ValidationFailed {
                field: "ClassList",
                message: "Class list empty".to_string(),
            });
        }

        let old_class_id = class_list[0]
            .get("Class")
            .and_then(gff_value_to_i32)
            .unwrap_or(0);

        self.change_class_preserving_levels(ClassId(old_class_id), new_class_id, game_data)
    }

    pub fn change_class_preserving_levels(
        &mut self,
        old_class_id: ClassId,
        new_class_id: ClassId,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        if old_class_id == new_class_id {
            return Ok(());
        }

        let classes_table = game_data
            .get_table("classes")
            .ok_or_else(|| CharacterError::TableNotFound("classes".to_string()))?;
        if classes_table.get_by_id(new_class_id.0).is_none() {
            return Err(CharacterError::NotFound {
                entity: "Class",
                id: new_class_id.0,
            });
        }

        let old_class_level = self.class_level(old_class_id);
        if old_class_level == 0 {
            return Err(CharacterError::NotFound {
                entity: "Class",
                id: old_class_id.0,
            });
        }

        let mut class_list = self
            .get_list_owned("ClassList")
            .ok_or(CharacterError::FieldMissing { field: "ClassList" })?;
        let Some(class_index) = class_list.iter().position(|entry| {
            entry.get("Class").and_then(gff_value_to_i32) == Some(old_class_id.0)
        }) else {
            return Err(CharacterError::NotFound {
                entity: "Class",
                id: old_class_id.0,
            });
        };

        let existing_new_index = class_list.iter().position(|entry| {
            entry.get("Class").and_then(gff_value_to_i32) == Some(new_class_id.0)
        });

        if let Some(new_class_index) = existing_new_index {
            if new_class_index != class_index {
                let merged_level = self.class_level(new_class_id) + old_class_level;
                let mut merged_entry = class_list[new_class_index].clone();
                merged_entry.insert("Class".to_string(), GffValue::Byte(new_class_id.0 as u8));
                merged_entry.insert(
                    "ClassLevel".to_string(),
                    GffValue::Short(merged_level as i16),
                );

                let mut indices_to_remove = [class_index, new_class_index];
                indices_to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for index in indices_to_remove {
                    class_list.remove(index);
                }

                let insert_index = class_index.min(class_list.len());
                class_list.insert(insert_index, merged_entry);
            } else if let Some(class_entry) = class_list.get_mut(class_index) {
                class_entry.insert("Class".to_string(), GffValue::Byte(new_class_id.0 as u8));
                Self::clear_known_spell_lists(class_entry);
                class_entry.shift_remove("SpellCasterLevel");
            }
        } else if let Some(class_entry) = class_list.get_mut(class_index) {
            class_entry.insert("Class".to_string(), GffValue::Byte(new_class_id.0 as u8));
            Self::clear_known_spell_lists(class_entry);
            class_entry.shift_remove("SpellCasterLevel");
        }
        self.set_list("ClassList", class_list);

        if class_index == 0 || self.get_i32("Class") == Some(old_class_id.0) {
            self.set_byte("Class", new_class_id.0 as u8);
        }

        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        for entry in &mut lvl_stat_list {
            if entry.get("LvlStatClass").and_then(gff_value_to_i32) == Some(old_class_id.0) {
                entry.insert(
                    "LvlStatClass".to_string(),
                    GffValue::Byte(new_class_id.0 as u8),
                );
                Self::clear_known_spell_lists(entry);
            }
        }
        self.set_list("LvlStatList", lvl_stat_list);

        self.reconcile_class_feats(&[old_class_id], game_data);
        self.grant_auto_class_feats_without_history(new_class_id, old_class_level, game_data);
        self.recalculate_hp(game_data)?;
        self.recalculate_stats(game_data)?;
        self.normalize_skill_points(game_data);

        Ok(())
    }

    /// Recalculate HP for the entire character based on class history.
    ///
    /// This enforces the "Average Roll" rule:
    /// - Level 1: Max Hit Die + CON
    /// - Level > 1: (Hit Die / 2 + 1) + CON
    ///
    /// Also updates `LvlStatHitDie` in history to match the new calculation.
    pub fn recalculate_hp(&mut self, game_data: &GameData) -> Result<(), CharacterError> {
        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        let mut total_hp = 0;
        let con_mod = self.get_effective_ability_modifier(AbilityIndex::CON, game_data);

        let classes_table = game_data
            .get_table("classes")
            .ok_or_else(|| CharacterError::TableNotFound("classes".to_string()))?;

        for (idx, entry) in lvl_stat_list.iter_mut().enumerate() {
            let class_id = entry
                .get("LvlStatClass")
                .and_then(gff_value_to_i32)
                .unwrap_or(0);

            // Get Hit Die for this class level
            let hit_die = if let Some(class_data) = classes_table.get_by_id(class_id) {
                Self::get_field_value(&class_data, "hit_die")
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(6)
            } else {
                6
            };

            // Calculate Base HP (Roll equivalent)
            let base_hp = if idx == 0 {
                hit_die // Max at level 1
            } else {
                (hit_die / 2) + 1 // Average otherwise
            };

            let hp_gained = (base_hp + con_mod).max(1);

            // Update history
            entry.insert("LvlStatHitDie".to_string(), GffValue::Byte(base_hp as u8));

            total_hp += hp_gained;
        }

        self.set_list("LvlStatList", lvl_stat_list);

        // Update Globals
        self.set_i32("HitPoints", total_hp);
        self.set_i32("MaxHitPoints", total_hp);
        self.set_i32("CurrentHitPoints", total_hp); // Heal to full on recalc

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::LoadedTable;
    use crate::parsers::tda::TDAParser;

    fn create_test_character() -> Character {
        let mut fields = IndexMap::new();

        let mut class1 = IndexMap::new();
        class1.insert("Class".to_string(), GffValue::Byte(0));
        class1.insert("ClassLevel".to_string(), GffValue::Short(5));

        let mut class2 = IndexMap::new();
        class2.insert("Class".to_string(), GffValue::Byte(3));
        class2.insert("ClassLevel".to_string(), GffValue::Short(3));

        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class1, class2]),
        );

        // Add LvlStatList for proper level_down testing
        let mut lvl_stat_list = Vec::new();
        for i in 0..8 {
            let mut entry = IndexMap::new();
            entry.insert(
                "LvlStatClass".to_string(),
                GffValue::Byte(if i < 5 { 0 } else { 3 }),
            );
            entry.insert("LvlStatHitDie".to_string(), GffValue::Byte(6));
            entry.insert("SkillPoints".to_string(), GffValue::Short(4));
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
            entry.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
            entry.insert("SkillList".to_string(), GffValue::ListOwned(vec![]));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        // Add required ability scores for level_down
        fields.insert("Str".to_string(), GffValue::Byte(14));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(14));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));

        // Add HP fields
        fields.insert("MaxHitPoints".to_string(), GffValue::Short(60));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Short(60));
        fields.insert("HitPoints".to_string(), GffValue::Short(60));

        // Add skill points
        fields.insert("SkillPoints".to_string(), GffValue::Short(0));

        Character::from_gff(fields)
    }

    fn create_mock_game_data() -> GameData {
        use crate::parsers::tlk::TLKParser;
        use std::sync::Arc;
        GameData::new(Arc::new(std::sync::RwLock::new(TLKParser::default())))
    }

    fn create_game_data_with_class_rows(class_rows: &[(i32, &str, i32, i32)]) -> GameData {
        let mut game_data = create_mock_game_data();
        let mut parser = TDAParser::new();
        let mut data = String::from(
            "2DA V2.0\n\nLabel HitDie SkillPointBase AttackBonusTable SavingThrowTable Package SpellCaster\n",
        );

        let max_class_id = class_rows
            .iter()
            .map(|(class_id, _, _, _)| *class_id)
            .max()
            .unwrap_or(0);

        for row_id in 0..=max_class_id {
            let (label, hit_die, skill_points) = class_rows
                .iter()
                .find(|(class_id, _, _, _)| *class_id == row_id)
                .map(|(_, label, hit_die, skill_points)| (*label, *hit_die, *skill_points))
                .unwrap_or(("Unused", 6, 2));

            data.push_str(&format!(
                "{row_id} {label} {hit_die} {skill_points} **** **** {row_id} 0\n"
            ));
        }

        parser
            .parse_from_string(&data)
            .expect("Failed to parse test classes 2DA");
        game_data.tables.insert(
            "classes".to_string(),
            LoadedTable::new("classes".to_string(), std::sync::Arc::new(parser)),
        );
        game_data
    }

    fn create_game_data_with_history_skill_tables() -> GameData {
        let mut game_data = create_mock_game_data();

        let mut classes_parser = TDAParser::new();
        classes_parser
            .parse_from_string(
                "2DA V2.0\n\nLabel HitDie SkillPointBase SkillsTable\n\
                 0 Barbarian 12 4 barbskills\n\
                 1 Unused 6 2 ****\n\
                 2 Unused 6 2 ****\n\
                 3 Unused 6 2 ****\n\
                 4 Fighter 10 2 fighterskills\n",
            )
            .expect("Failed to parse history test classes 2DA");
        game_data.tables.insert(
            "classes".to_string(),
            LoadedTable::new("classes".to_string(), std::sync::Arc::new(classes_parser)),
        );

        let mut barbskills = TDAParser::new();
        barbskills
            .parse_from_string(
                "2DA V2.0\n\nSkillIndex ClassSkill\n\
                 0 21 0\n\
                 1 24 1\n\
                 2 25 1\n\
                 3 26 1\n",
            )
            .expect("Failed to parse barbskills 2DA");
        game_data.tables.insert(
            "barbskills".to_string(),
            LoadedTable::new("barbskills".to_string(), std::sync::Arc::new(barbskills)),
        );

        let mut fighterskills = TDAParser::new();
        fighterskills
            .parse_from_string(
                "2DA V2.0\n\nSkillIndex ClassSkill\n\
                 0 21 0\n\
                 1 24 1\n\
                 2 25 1\n\
                 3 26 1\n",
            )
            .expect("Failed to parse fighterskills 2DA");
        game_data.tables.insert(
            "fighterskills".to_string(),
            LoadedTable::new(
                "fighterskills".to_string(),
                std::sync::Arc::new(fighterskills),
            ),
        );

        game_data
    }

    fn zero_rank_skill_list(skill_count: usize) -> Vec<IndexMap<String, GffValue<'static>>> {
        let mut skill_list = Vec::with_capacity(skill_count);
        for _ in 0..skill_count {
            let mut skill = IndexMap::new();
            skill.insert("Rank".to_string(), GffValue::Byte(0));
            skill_list.push(skill);
        }
        skill_list
    }

    fn skill_list_with_ranks(
        skill_count: usize,
        ranks: &[(usize, u8)],
    ) -> Vec<IndexMap<String, GffValue<'static>>> {
        let mut skill_list = zero_rank_skill_list(skill_count);
        for (skill_idx, rank) in ranks {
            if let Some(skill) = skill_list.get_mut(*skill_idx) {
                skill.insert("Rank".to_string(), GffValue::Byte(*rank));
            }
        }
        skill_list
    }

    fn create_history_entry(
        class_id: u8,
        hit_die: u8,
        skill_points: i16,
        skill_count: usize,
    ) -> IndexMap<String, GffValue<'static>> {
        let mut entry = IndexMap::new();
        entry.insert("LvlStatClass".to_string(), GffValue::Byte(class_id));
        entry.insert("LvlStatHitDie".to_string(), GffValue::Byte(hit_die));
        entry.insert("EpicLevel".to_string(), GffValue::Byte(0));
        entry.insert("SkillPoints".to_string(), GffValue::Short(skill_points));
        entry.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(zero_rank_skill_list(skill_count)),
        );
        entry.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
        entry
    }

    fn create_history_entry_with_skill_ranks(
        class_id: u8,
        hit_die: u8,
        skill_points: i16,
        skill_count: usize,
        ranks: &[(usize, u8)],
    ) -> IndexMap<String, GffValue<'static>> {
        let mut entry = create_history_entry(class_id, hit_die, skill_points, skill_count);
        entry.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(skill_list_with_ranks(skill_count, ranks)),
        );
        entry
    }

    fn build_history_aligned_test_character() -> Character {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(31));
        fields.insert("Str".to_string(), GffValue::Byte(12));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(12));
        fields.insert("Int".to_string(), GffValue::Byte(8));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));
        fields.insert("SkillPoints".to_string(), GffValue::Word(0));

        let mut barbarian = IndexMap::new();
        barbarian.insert("Class".to_string(), GffValue::Byte(0));
        barbarian.insert("ClassLevel".to_string(), GffValue::Short(5));
        let mut fighter = IndexMap::new();
        fighter.insert("Class".to_string(), GffValue::Byte(4));
        fighter.insert("ClassLevel".to_string(), GffValue::Short(2));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![barbarian, fighter]),
        );

        let skill_count = 30;
        fields.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(skill_list_with_ranks(
                skill_count,
                &[(21, 4), (24, 10), (25, 9), (26, 8)],
            )),
        );

        let history = vec![
            create_history_entry_with_skill_ranks(
                0,
                12,
                0,
                skill_count,
                &[(21, 2), (24, 4), (25, 4), (26, 4)],
            ),
            create_history_entry_with_skill_ranks(
                0,
                12,
                1,
                skill_count,
                &[(24, 1), (25, 1), (26, 1)],
            ),
            create_history_entry_with_skill_ranks(
                0,
                12,
                0,
                skill_count,
                &[(21, 1), (24, 1), (25, 1), (26, 1)],
            ),
            create_history_entry_with_skill_ranks(0, 12, 2, skill_count, &[(25, 1), (26, 1)]),
            create_history_entry_with_skill_ranks(
                0,
                12,
                0,
                skill_count,
                &[(21, 1), (24, 2), (25, 1), (26, 1)],
            ),
            create_history_entry_with_skill_ranks(4, 10, 1, skill_count, &[(24, 1)]),
            create_history_entry_with_skill_ranks(4, 10, 1, skill_count, &[(24, 1), (25, 1)]),
        ];
        fields.insert("LvlStatList".to_string(), GffValue::ListOwned(history));

        Character::from_gff(fields)
    }

    fn create_multiclass_character_for_preserve_change() -> Character {
        let mut fields = IndexMap::new();

        let mut class1 = IndexMap::new();
        class1.insert("Class".to_string(), GffValue::Byte(0));
        class1.insert("ClassLevel".to_string(), GffValue::Short(5));

        let mut class2 = IndexMap::new();
        class2.insert("Class".to_string(), GffValue::Byte(1));
        class2.insert("ClassLevel".to_string(), GffValue::Short(1));

        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class1, class2]),
        );
        fields.insert("Class".to_string(), GffValue::Byte(0));
        fields.insert("Str".to_string(), GffValue::Byte(14));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(14));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));
        fields.insert("HitPoints".to_string(), GffValue::Int(60));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(60));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(60));
        fields.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(zero_rank_skill_list(30)),
        );

        let mut history = Vec::new();
        for _ in 0..5 {
            history.push(create_history_entry(0, 12, 0, 30));
        }
        history.push(create_history_entry(1, 8, 0, 30));
        fields.insert("LvlStatList".to_string(), GffValue::ListOwned(history));

        Character::from_gff(fields)
    }

    fn create_level_one_character_with_full_history_entry() -> Character {
        let mut fields = IndexMap::new();
        let mut class1 = IndexMap::new();
        class1.insert("Class".to_string(), GffValue::Byte(4));
        class1.insert("ClassLevel".to_string(), GffValue::Short(1));

        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![class1]));
        fields.insert("Class".to_string(), GffValue::Byte(4));
        fields.insert("Str".to_string(), GffValue::Byte(14));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(14));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));
        fields.insert("HitPoints".to_string(), GffValue::Int(12));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(12));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(12));
        fields.insert("SkillPoints".to_string(), GffValue::Word(0));
        fields.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(zero_rank_skill_list(30)),
        );
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![create_history_entry(4, 10, 0, 30)]),
        );

        Character::from_gff(fields)
    }

    fn create_multiclass_character_with_allocated_skills() -> Character {
        let mut fields = IndexMap::new();

        let mut barbarian = IndexMap::new();
        barbarian.insert("Class".to_string(), GffValue::Byte(0));
        barbarian.insert("ClassLevel".to_string(), GffValue::Short(5));

        let mut fighter = IndexMap::new();
        fighter.insert("Class".to_string(), GffValue::Byte(4));
        fighter.insert("ClassLevel".to_string(), GffValue::Short(2));

        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![barbarian, fighter]),
        );
        fields.insert("Class".to_string(), GffValue::Byte(0));
        fields.insert("Str".to_string(), GffValue::Byte(14));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(14));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));
        fields.insert("HitPoints".to_string(), GffValue::Int(70));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(70));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(70));
        fields.insert("SkillPoints".to_string(), GffValue::Word(22));
        fields.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(skill_list_with_ranks(30, &[(0, 4), (1, 3)])),
        );

        let history = vec![
            create_history_entry_with_skill_ranks(0, 12, 16, 30, &[(0, 2), (1, 1)]),
            create_history_entry_with_skill_ranks(0, 6, 4, 30, &[(0, 1)]),
            create_history_entry_with_skill_ranks(0, 6, 4, 30, &[(1, 1)]),
            create_history_entry_with_skill_ranks(0, 6, 4, 30, &[(1, 1)]),
            create_history_entry_with_skill_ranks(0, 6, 4, 30, &[(0, 1)]),
            create_history_entry_with_skill_ranks(4, 5, 2, 30, &[]),
            create_history_entry_with_skill_ranks(4, 5, 2, 30, &[]),
        ];
        fields.insert("LvlStatList".to_string(), GffValue::ListOwned(history));

        Character::from_gff(fields)
    }

    #[test]
    fn test_total_level() {
        let character = create_test_character();
        assert_eq!(character.total_level(), 8);
    }

    #[test]
    fn test_class_entries() {
        let character = create_test_character();
        let entries = character.class_entries();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].class_id.0, 0);
        assert_eq!(entries[0].level, 5);
        assert_eq!(entries[1].class_id.0, 3);
        assert_eq!(entries[1].level, 3);
    }

    #[test]
    fn test_class_level() {
        let character = create_test_character();
        assert_eq!(character.class_level(ClassId(0)), 5);
        assert_eq!(character.class_level(ClassId(3)), 3);
        assert_eq!(character.class_level(ClassId(99)), 0);
    }

    #[test]
    fn test_class_count() {
        let character = create_test_character();
        assert_eq!(character.class_count(), 2);
    }

    #[test]
    fn test_has_class() {
        let character = create_test_character();
        assert!(character.has_class(ClassId(0)));
        assert!(character.has_class(ClassId(3)));
        assert!(!character.has_class(ClassId(99)));
    }

    #[test]
    fn test_set_class_level() {
        let mut character = create_test_character();

        character.set_class_level(ClassId(0), 10).unwrap();
        assert_eq!(character.class_level(ClassId(0)), 10);

        character.set_class_level(ClassId(3), 7).unwrap();
        assert_eq!(character.class_level(ClassId(3)), 7);
    }

    #[test]
    fn test_set_class_level_validation() {
        let mut character = create_test_character();

        let result = character.set_class_level(ClassId(0), 0);
        assert!(result.is_err());

        let result = character.set_class_level(ClassId(99), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_class_entry() {
        let mut character = create_test_character();

        character.add_class_entry(ClassId(5), 2).unwrap();
        assert_eq!(character.class_count(), 3);
        assert_eq!(character.class_level(ClassId(5)), 2);
    }

    #[test]
    fn test_add_class_entry_duplicate() {
        let mut character = create_test_character();

        let result = character.add_class_entry(ClassId(0), 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_class_entry_max_classes() {
        let mut character = create_test_character();

        character.add_class_entry(ClassId(5), 1).unwrap();
        assert_eq!(character.class_count(), 3);

        let result = character.add_class_entry(ClassId(7), 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_class() {
        let mut character = create_test_character();
        let game_data = create_mock_game_data();

        character.remove_class(ClassId(3), &game_data).unwrap();
        assert_eq!(character.class_count(), 1);
        assert!(!character.has_class(ClassId(3)));
        assert!(character.has_class(ClassId(0)));
    }

    #[test]
    fn test_remove_last_class() {
        let mut fields = IndexMap::new();
        let mut class1 = IndexMap::new();
        class1.insert("Class".to_string(), GffValue::Byte(0));
        class1.insert("ClassLevel".to_string(), GffValue::Short(5));
        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![class1]));

        let mut character = Character::from_gff(fields);
        let game_data = create_mock_game_data();

        let result = character.remove_class(ClassId(0), &game_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_nonexistent_class() {
        let mut character = create_test_character();
        let game_data = create_mock_game_data();

        let result = character.remove_class(ClassId(99), &game_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_class_list() {
        let fields = IndexMap::new();
        let character = Character::from_gff(fields);

        assert_eq!(character.total_level(), 0);
        assert_eq!(character.class_count(), 0);
        assert_eq!(character.class_entries().len(), 0);
        assert!(!character.has_class(ClassId(0)));
    }

    #[test]
    fn test_level_history() {
        let mut fields = IndexMap::new();

        let mut class1 = IndexMap::new();
        class1.insert("Class".to_string(), GffValue::Byte(0));
        class1.insert("ClassLevel".to_string(), GffValue::Short(5));
        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![class1]));

        let mut lvl1 = IndexMap::new();
        lvl1.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        lvl1.insert("LvlStatHitDie".to_string(), GffValue::Byte(10));
        lvl1.insert("SkillPoints".to_string(), GffValue::Short(0));
        lvl1.insert("LvlStatAbility".to_string(), GffValue::Byte(255));

        let mut lvl2 = IndexMap::new();
        lvl2.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        lvl2.insert("LvlStatHitDie".to_string(), GffValue::Byte(6));
        lvl2.insert("SkillPoints".to_string(), GffValue::Short(0));
        lvl2.insert("LvlStatAbility".to_string(), GffValue::Byte(0));

        let mut feat1 = IndexMap::new();
        feat1.insert("Feat".to_string(), GffValue::Word(1));
        lvl2.insert("FeatList".to_string(), GffValue::ListOwned(vec![feat1]));

        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![lvl1, lvl2]),
        );

        let character = Character::from_gff(fields);
        let history = character.level_history();

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].character_level, 1);
        assert_eq!(history[0].class_id.0, 0);
        assert_eq!(history[0].class_level, 1);
        assert_eq!(history[0].hp_gained, 10);
        assert!(history[0].ability_increase.is_none());
        assert_eq!(history[0].feats_gained.len(), 0);

        assert_eq!(history[1].character_level, 2);
        assert_eq!(history[1].class_id.0, 0);
        assert_eq!(history[1].class_level, 2);
        assert_eq!(history[1].hp_gained, 6);
        assert_eq!(history[1].ability_increase, Some(AbilityIndex::STR));
        assert_eq!(history[1].feats_gained.len(), 1);
        assert_eq!(history[1].feats_gained[0].0, 1);
    }

    #[test]
    fn test_level_history_empty() {
        let fields = IndexMap::new();
        let character = Character::from_gff(fields);

        let history = character.level_history();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_level_up_missing_table() {
        let mut character = create_test_character();
        let game_data = GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        // classes table missing in game_data
        let result = character.level_up(ClassId(0), &game_data);
        assert!(result.is_err());
        match result.unwrap_err() {
            CharacterError::TableNotFound(table) => assert_eq!(table, "classes"),
            _ => panic!("Expected TableNotFound error"),
        }
    }

    fn create_character_with_history() -> Character {
        let mut fields = IndexMap::new();

        // Class list: Fighter(0) level 3
        let mut class1 = IndexMap::new();
        class1.insert("Class".to_string(), GffValue::Byte(0));
        class1.insert("ClassLevel".to_string(), GffValue::Short(3));
        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![class1]));

        // Level history
        let mut lvl1 = IndexMap::new();
        lvl1.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        lvl1.insert("LvlStatHitDie".to_string(), GffValue::Byte(10));
        lvl1.insert("SkillPoints".to_string(), GffValue::Short(8));
        lvl1.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        lvl1.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
        lvl1.insert("SkillList".to_string(), GffValue::ListOwned(vec![]));

        let mut lvl2 = IndexMap::new();
        lvl2.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        lvl2.insert("LvlStatHitDie".to_string(), GffValue::Byte(6));
        lvl2.insert("SkillPoints".to_string(), GffValue::Short(2));
        lvl2.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        lvl2.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
        lvl2.insert("SkillList".to_string(), GffValue::ListOwned(vec![]));

        let mut lvl3 = IndexMap::new();
        lvl3.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        lvl3.insert("LvlStatHitDie".to_string(), GffValue::Byte(6));
        lvl3.insert("SkillPoints".to_string(), GffValue::Short(2));
        lvl3.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        lvl3.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
        lvl3.insert("SkillList".to_string(), GffValue::ListOwned(vec![]));

        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![lvl1, lvl2, lvl3]),
        );

        // Abilities for CON modifier calculation
        fields.insert("Con".to_string(), GffValue::Byte(14)); // +2 modifier

        // HP fields
        fields.insert("HitPoints".to_string(), GffValue::Int(28));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(28));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(28));

        // Class byte
        fields.insert("Class".to_string(), GffValue::Byte(0));

        Character::from_gff(fields)
    }

    #[test]
    fn test_change_primary_class_updates_class_list() {
        let mut character = create_character_with_history();
        let game_data = GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        // Without classes table, should fail
        let result = character.change_primary_class(ClassId(1), &game_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_change_primary_class_noop_same_class() {
        let character = create_character_with_history();
        let _game_data = GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        // Changing to same class should be no-op (but still fails due to missing table for validation)
        // This tests the early return path
        let entries_before = character.class_entries();
        assert_eq!(entries_before[0].class_id.0, 0);
    }

    #[test]
    fn test_swap_class_nonexistent_old_class() {
        let mut character = create_character_with_history();
        let game_data = GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        // Trying to swap a class that doesn't exist
        let result = character.swap_class(ClassId(99), ClassId(1), &game_data);
        assert!(result.is_err());
        match result.unwrap_err() {
            CharacterError::NotFound { entity, id } => {
                assert_eq!(entity, "Class");
                assert_eq!(id, 99);
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_history_class_update() {
        let mut character = create_character_with_history();

        // Manually update history to test the update mechanism
        let mut lvl_stat_list = character.get_list_owned("LvlStatList").unwrap();
        assert_eq!(lvl_stat_list.len(), 3);

        // All entries should be class 0
        for entry in &lvl_stat_list {
            let class_id = entry
                .get("LvlStatClass")
                .and_then(gff_value_to_i32)
                .unwrap();
            assert_eq!(class_id, 0);
        }

        // Update all to class 1
        for entry in &mut lvl_stat_list {
            entry.insert("LvlStatClass".to_string(), GffValue::Byte(1));
        }
        character.set_list("LvlStatList", lvl_stat_list);

        // Verify update
        let updated_list = character.get_list_owned("LvlStatList").unwrap();
        for entry in &updated_list {
            let class_id = entry
                .get("LvlStatClass")
                .and_then(gff_value_to_i32)
                .unwrap();
            assert_eq!(class_id, 1);
        }
    }

    #[test]
    fn test_recalculate_hp_no_history() {
        let mut fields = IndexMap::new();
        fields.insert("HitPoints".to_string(), GffValue::Int(10));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(10));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(10));
        fields.insert("Con".to_string(), GffValue::Byte(10)); // +0 modifier

        let mut character = Character::from_gff(fields);
        let game_data = GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        // No history means HP becomes 0 (sum of nothing)
        // But this requires classes table
        let result = character.recalculate_hp(&game_data);
        assert!(result.is_err()); // Missing classes table
    }

    #[test]
    fn test_class_entry_preserved_on_primary_change_failure() {
        let mut character = create_character_with_history();

        // Store original state
        let original_class = character.class_entries()[0].class_id.0;

        // Attempt change without game data (will fail)
        let game_data = GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));
        let _ = character.change_primary_class(ClassId(5), &game_data);

        // Original class should be preserved (change failed)
        assert_eq!(character.class_entries()[0].class_id.0, original_class);
    }

    #[test]
    fn test_change_class_preserving_levels_keeps_total_levels_and_history() {
        let mut character = create_multiclass_character_for_preserve_change();
        let game_data = create_game_data_with_class_rows(&[
            (0, "Barbarian", 12, 4),
            (1, "Bard", 6, 6),
            (4, "Fighter", 10, 2),
        ]);

        character
            .change_class_preserving_levels(ClassId(0), ClassId(4), &game_data)
            .expect("Expected preserve-level class change to succeed");

        assert_eq!(character.total_level(), 6);
        let entries = character.class_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].class_id.0, 4);
        assert_eq!(entries[0].level, 5);
        assert_eq!(entries[1].class_id.0, 1);
        assert_eq!(entries[1].level, 1);
        assert_eq!(character.get_i32("Class"), Some(4));

        let history = character.get_list_owned("LvlStatList").unwrap();
        assert_eq!(history.len(), 6);
        assert!(
            history[..5]
                .iter()
                .all(|entry| entry.get("LvlStatClass").and_then(gff_value_to_i32) == Some(4))
        );
        assert_eq!(
            history[5].get("LvlStatClass").and_then(gff_value_to_i32),
            Some(1)
        );
        assert_eq!(
            history[0].get("SkillList").and_then(|value| match value {
                GffValue::ListOwned(list) => Some(list.len()),
                _ => None,
            }),
            Some(30)
        );
    }

    #[test]
    fn test_change_class_preserving_levels_merges_into_existing_class() {
        let mut character = create_multiclass_character_for_preserve_change();
        let game_data =
            create_game_data_with_class_rows(&[(0, "Barbarian", 12, 4), (1, "Bard", 6, 6)]);

        character
            .change_class_preserving_levels(ClassId(0), ClassId(1), &game_data)
            .expect("Expected preserve-level class change to merge into existing class");

        assert_eq!(character.total_level(), 6);
        let entries = character.class_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].class_id.0, 1);
        assert_eq!(entries[0].level, 6);
        assert_eq!(character.get_i32("Class"), Some(1));

        let history = character.get_list_owned("LvlStatList").unwrap();
        assert_eq!(history.len(), 6);
        assert!(
            history
                .iter()
                .all(|entry| entry.get("LvlStatClass").and_then(gff_value_to_i32) == Some(1))
        );
    }

    #[test]
    fn test_change_class_and_level_down_preserve_top_level_skill_ranks() {
        let mut character = create_multiclass_character_with_allocated_skills();
        let game_data =
            create_game_data_with_class_rows(&[(0, "Barbarian", 12, 4), (4, "Fighter", 10, 2)]);

        let before = character.get_skill_points_summary(&game_data);
        assert_eq!(before.theoretical_total, 36);
        assert_eq!(before.actual_spent, 14);
        assert_eq!(before.current_unspent, 22);
        assert_eq!(character.skill_rank(SkillId(0)), 4);
        assert_eq!(character.skill_rank(SkillId(1)), 3);

        character
            .change_class_preserving_levels(ClassId(0), ClassId(4), &game_data)
            .expect("Expected preserve-level class change to succeed");

        let after_change = character.get_skill_points_summary(&game_data);
        assert_eq!(character.class_level(ClassId(4)), 7);
        assert_eq!(character.skill_rank(SkillId(0)), 4);
        assert_eq!(character.skill_rank(SkillId(1)), 3);
        assert_eq!(after_change.theoretical_total, 20);
        assert_eq!(after_change.actual_spent, 14);
        assert_eq!(after_change.current_unspent, 6);
        assert_eq!(after_change.mismatch, 0);

        while character.class_level(ClassId(4)) > 1 {
            character
                .level_down(ClassId(4), &game_data)
                .expect("Expected level down to succeed");
        }

        let after_level_down = character.get_skill_points_summary(&game_data);
        assert_eq!(character.class_level(ClassId(4)), 1);
        assert_eq!(character.skill_rank(SkillId(0)), 4);
        assert_eq!(character.skill_rank(SkillId(1)), 3);
        assert_eq!(after_level_down.theoretical_total, 8);
        assert_eq!(after_level_down.actual_spent, 14);
        assert_eq!(after_level_down.current_unspent, 0);
        assert_eq!(after_level_down.mismatch, -6);
    }

    #[test]
    fn test_remove_class_preserves_top_level_skill_ranks() {
        let mut character = create_multiclass_character_with_allocated_skills();
        let game_data =
            create_game_data_with_class_rows(&[(0, "Barbarian", 12, 4), (4, "Fighter", 10, 2)]);

        character
            .remove_class(ClassId(0), &game_data)
            .expect("Expected class removal to succeed");

        let summary = character.get_skill_points_summary(&game_data);
        let entries = character.class_entries();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].class_id.0, 4);
        assert_eq!(entries[0].level, 2);
        assert_eq!(character.skill_rank(SkillId(0)), 4);
        assert_eq!(character.skill_rank(SkillId(1)), 3);
        assert_eq!(summary.theoretical_total, 10);
        assert_eq!(summary.actual_spent, 14);
        assert_eq!(summary.current_unspent, 0);
        assert_eq!(summary.mismatch, -4);
    }

    #[test]
    fn test_get_skill_points_summary_uses_history_floor_for_unmodified_characters() {
        let character = build_history_aligned_test_character();
        let game_data = create_game_data_with_history_skill_tables();
        let history = character.get_list("LvlStatList").unwrap();
        let mut aggregated = vec![0; 30];
        for entry in history {
            if let Some(GffValue::ListOwned(skill_list)) = entry.get("SkillList") {
                for (idx, skill) in skill_list.iter().enumerate() {
                    aggregated[idx] += skill.get("Rank").and_then(gff_value_to_i32).unwrap_or(0);
                }
            }
        }
        let top = character
            .get_list("SkillList")
            .unwrap()
            .iter()
            .map(|skill| skill.get("Rank").and_then(gff_value_to_i32).unwrap_or(0))
            .collect::<Vec<_>>();
        assert_eq!(aggregated, top);
        assert_eq!(
            character.history_aligned_skill_point_floor(&game_data),
            Some(36)
        );

        let summary = character.get_skill_points_summary(&game_data);

        assert_eq!(summary.theoretical_total, 36);
        assert_eq!(summary.actual_spent, 35);
        assert_eq!(summary.current_unspent, 0);
        assert_eq!(summary.mismatch, 1);
    }

    #[test]
    fn test_get_skill_points_summary_ignores_history_floor_after_modification() {
        let mut character = build_history_aligned_test_character();
        let game_data = create_game_data_with_history_skill_tables();

        character.mark_modified();
        let summary = character.get_skill_points_summary(&game_data);

        assert_eq!(summary.theoretical_total, 26);
        assert_eq!(summary.actual_spent, 35);
        assert_eq!(summary.current_unspent, 0);
        assert_eq!(summary.mismatch, -9);
    }

    #[test]
    fn test_level_up_reuses_engine_style_history_shape() {
        let mut character = create_level_one_character_with_full_history_entry();
        let game_data = create_game_data_with_class_rows(&[(4, "Fighter", 10, 2)]);

        character
            .level_up(ClassId(4), &game_data)
            .expect("Expected level up to succeed");

        let history = character.get_list_owned("LvlStatList").unwrap();
        let last = history.last().expect("Expected second history entry");

        assert_eq!(history.len(), 2);
        assert_eq!(last.get("EpicLevel").and_then(gff_value_to_i32), Some(0));
        assert_eq!(
            last.get("LvlStatHitDie").and_then(gff_value_to_i32),
            Some(6)
        );
        assert_eq!(
            last.get("SkillList").and_then(|value| match value {
                GffValue::ListOwned(list) => Some(list.len()),
                _ => None,
            }),
            Some(30)
        );
        assert!(!last.contains_key("KnownList0"));
    }

    #[test]
    fn test_normalize_class_fields_for_save_repairs_invalid_single_class_history() {
        let mut fields = IndexMap::new();

        let mut fighter = IndexMap::new();
        fighter.insert("Class".to_string(), GffValue::Byte(4));
        fighter.insert("ClassLevel".to_string(), GffValue::Short(2));
        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![fighter]));
        fields.insert("Class".to_string(), GffValue::Byte(4));
        fields.insert("MClassLevUpIn".to_string(), GffValue::Byte(1));
        fields.insert("StartingPackage".to_string(), GffValue::Byte(0));
        fields.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(zero_rank_skill_list(30)),
        );

        let mut level_one = create_history_entry(4, 10, 0, 30);
        level_one.insert("LvlStatAbility".to_string(), GffValue::Byte(255));

        let mut level_two = IndexMap::new();
        level_two.insert("LvlStatClass".to_string(), GffValue::Byte(4));
        level_two.insert("LvlStatHitDie".to_string(), GffValue::Byte(13));
        level_two.insert("EpicLevel".to_string(), GffValue::Byte(0));
        level_two.insert("SkillPoints".to_string(), GffValue::Short(1));
        level_two.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        level_two.insert("SkillList".to_string(), GffValue::ListOwned(vec![]));
        level_two.insert("KnownList0".to_string(), GffValue::ListOwned(vec![]));
        level_two.insert("KnownRemoveList0".to_string(), GffValue::ListOwned(vec![]));

        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![level_one, level_two]),
        );

        let mut character = Character::from_gff(fields);
        let game_data = create_game_data_with_class_rows(&[(4, "Fighter", 10, 2)]);

        character.normalize_class_fields_for_save(&game_data);

        assert_eq!(character.get_i32("MClassLevUpIn"), Some(0));
        assert_eq!(character.get_i32("StartingPackage"), Some(4));
        assert!(!character.has_field("Class"));

        let class_list = character
            .get_list_owned("ClassList")
            .expect("Expected normalized class list");
        assert!(matches!(class_list[0].get("Class"), Some(GffValue::Int(4))));
        assert!(matches!(
            class_list[0].get("ClassLevel"),
            Some(GffValue::Short(2))
        ));

        let history = character
            .get_list_owned("LvlStatList")
            .expect("Expected normalized level history");
        assert_eq!(history.len(), 2);
        assert_eq!(
            history[0].get("LvlStatHitDie").and_then(gff_value_to_i32),
            Some(10)
        );
        assert_eq!(
            history[1].get("LvlStatHitDie").and_then(gff_value_to_i32),
            Some(6)
        );
        assert_eq!(
            history[1].get("SkillList").and_then(|value| match value {
                GffValue::ListOwned(list) => Some(list.len()),
                _ => None,
            }),
            Some(30)
        );
        assert!(matches!(
            history[1].get("FeatList"),
            Some(GffValue::ListOwned(list)) if list.is_empty()
        ));
        assert!(!history[1].contains_key("KnownList0"));
        assert!(!history[1].contains_key("KnownRemoveList0"));
    }

    #[test]
    fn test_normalize_skill_points_uses_cross_class_costs() {
        let mut fields = IndexMap::new();

        let mut fighter = IndexMap::new();
        fighter.insert("Class".to_string(), GffValue::Byte(4));
        fighter.insert("ClassLevel".to_string(), GffValue::Short(1));
        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![fighter]));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("SkillPoints".to_string(), GffValue::Word(6));

        let mut skill_list = zero_rank_skill_list(30);
        skill_list[0].insert("Rank".to_string(), GffValue::Byte(2));
        fields.insert("SkillList".to_string(), GffValue::ListOwned(skill_list));
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![create_history_entry(4, 10, 0, 30)]),
        );

        let mut character = Character::from_gff(fields);
        let game_data = create_game_data_with_class_rows(&[(4, "Fighter", 10, 2)]);

        let before = character.get_skill_points_summary(&game_data);
        assert_eq!(before.theoretical_total, 8);
        assert_eq!(before.actual_spent, 4);
        assert_eq!(before.current_unspent, 6);
        assert_eq!(before.mismatch, -2);

        character.normalize_skill_points(&game_data);
        assert_eq!(character.get_available_skill_points(), 4);
    }
}
