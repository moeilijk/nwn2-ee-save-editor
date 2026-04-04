use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use tracing::debug;

use super::gff_helpers::gff_value_to_i32;
use super::types::{ClassId, DomainId, FeatId, SaveBonuses};
use super::{Character, CharacterError};
use crate::loaders::GameData;
use crate::parsers::gff::GffValue;

use crate::services::field_mapper::FieldMapper;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Type)]
#[repr(transparent)]
pub struct FeatType(pub i32);

impl Serialize for FeatType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.0)
    }
}

impl<'de> Deserialize<'de> for FeatType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Ok(FeatType(value))
    }
}

impl FeatType {
    pub const GENERAL: Self = Self(1);
    pub const PROFICIENCY: Self = Self(2);
    pub const SKILL_SAVE: Self = Self(4);
    pub const METAMAGIC: Self = Self(8);
    pub const DIVINE: Self = Self(16);
    pub const EPIC: Self = Self(32);
    pub const CLASS: Self = Self(64);
    pub const BACKGROUND: Self = Self(128);
    pub const SPELLCASTING: Self = Self(256);
    pub const HISTORY: Self = Self(512);
    pub const HERITAGE: Self = Self(1024);
    pub const ITEM_CREATION: Self = Self(2048);
    pub const RACIAL: Self = Self(4096);
    pub const DOMAIN: Self = Self(8192);

    pub fn contains(&self, flag: FeatType) -> bool {
        (self.0 & flag.0) != 0
    }

    pub fn from_string(s: &str) -> Self {
        let upper = s.to_uppercase();
        let value = match upper.as_str() {
            "GENERAL" | "GENERAL_FT_CAT" => 1,
            "PROFICIENCY" | "COMBAT" | "PROFICIENCY_FT_CAT" | "COMBAT_FT_CAT" => 2,
            "SKILLNSAVE" | "SKILL" | "SAVE" | "SKILLNSAVE_FT_CAT" | "SKILL_FT_CAT" => 4,
            "METAMAGIC" | "METAMAGIC_FT_CAT" => 8,
            "DIVINE" | "SPECIAL" | "DIVINE_FT_CAT" | "SPECIAL_FT_CAT" => 16,
            "EPIC" | "EPIC_FT_CAT" => 32,
            "CLASSABILITY" | "CLASS" | "CLASSABILITY_FT_CAT" | "CLASS_FT_CAT" => 64,
            "BACKGROUND" | "BACKGROUND_FT_CAT" => 128,
            "SPELLCASTING" | "SPELLCASTING_FT_CAT" => 256,
            "HISTORY" | "HISTORY_FT_CAT" => 512,
            "HERITAGE" | "HERITAGE_FT_CAT" => 1024,
            "ITEMCREATION" | "ITEM" | "ITEMCREATION_FT_CAT" => 2048,
            "RACIALABILITY" | "RACIAL" | "RACIALABILITY_FT_CAT" | "RACIAL_FT_CAT" => 4096,
            "DOMAIN" | "DOMAIN_FT_CAT" => 8192,
            _ => {
                if upper.contains("PROFICIENCY") || upper.contains("COMBAT") {
                    2
                } else if upper.contains("EPIC") {
                    32
                } else if upper.contains("METAMAGIC") {
                    8
                } else if upper.contains("CLASS") {
                    64
                } else if upper.contains("SKILL") || upper.contains("SAVE") {
                    4
                } else if upper.contains("DIVINE") || upper.contains("SPECIAL") {
                    16
                } else if upper.contains("SPELL") {
                    256
                } else if upper.contains("RACIAL") {
                    4096
                } else if upper.contains("ITEM") {
                    2048
                } else if upper.contains("BACKGROUND") {
                    128
                } else if upper.contains("HISTORY") {
                    512
                } else if upper.contains("HERITAGE") {
                    1024
                } else if upper.contains("DOMAIN") {
                    8192
                } else if upper.contains("GENERAL") {
                    1
                } else {
                    match s.parse::<i32>() {
                        Ok(n) if (1..=13).contains(&n) => Self::from_category_int(n).0,
                        Ok(n) => n,
                        Err(_) => 0,
                    }
                }
            }
        };
        Self(value)
    }

    /// Maps NWN2 sequential TOOLCATEGORIES integer IDs to bitmask values.
    fn from_category_int(id: i32) -> Self {
        match id {
            1 => Self::GENERAL,
            2 => Self::PROFICIENCY,
            3 => Self::METAMAGIC,
            4 => Self::DIVINE,
            5 => Self::CLASS,
            6 => Self::EPIC,
            7 => Self::ITEM_CREATION,
            8 => Self::SPELLCASTING,
            9 => Self::BACKGROUND,
            10 => Self::HISTORY,
            11 => Self::HERITAGE,
            12 => Self::RACIAL,
            13 => Self::SKILL_SAVE,
            _ => Self(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Type)]
pub enum FeatCategory {
    #[default]
    Unknown,
    General,
    Proficiency,
    SkillSave,
    Metamagic,
    Divine,
    Epic,
    Class,
    Background,
    Spellcasting,
    History,
    Heritage,
    ItemCreation,
    Racial,
    Domain,
}

impl FeatCategory {
    pub fn from_feat_type(feat_type: FeatType, is_domain: bool) -> Self {
        if is_domain {
            return Self::Domain;
        }

        if feat_type.contains(FeatType::EPIC) {
            Self::Epic
        } else if feat_type.contains(FeatType::BACKGROUND) {
            Self::Background
        } else if feat_type.contains(FeatType::HISTORY) {
            Self::History
        } else if feat_type.contains(FeatType::HERITAGE) {
            Self::Heritage
        } else if feat_type.contains(FeatType::RACIAL) {
            Self::Racial
        } else if feat_type.contains(FeatType::CLASS) {
            Self::Class
        } else if feat_type.contains(FeatType::METAMAGIC) {
            Self::Metamagic
        } else if feat_type.contains(FeatType::DIVINE) {
            Self::Divine
        } else if feat_type.contains(FeatType::ITEM_CREATION) {
            Self::ItemCreation
        } else if feat_type.contains(FeatType::SPELLCASTING) {
            Self::Spellcasting
        } else if feat_type.contains(FeatType::SKILL_SAVE) {
            Self::SkillSave
        } else if feat_type.contains(FeatType::PROFICIENCY) {
            Self::Proficiency
        } else if feat_type.contains(FeatType::GENERAL) {
            Self::General
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FeatInfo {
    pub id: FeatId,
    pub label: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(rename = "type")]
    pub feat_type: FeatType,
    pub category: FeatCategory,
    #[serde(rename = "protected")]
    pub is_protected: bool,
    #[serde(rename = "custom")]
    pub is_custom: bool,
    pub has_feat: bool,
    pub can_take: bool,
    pub missing_requirements: Vec<String>,
    pub prerequisites: FeatPrerequisites,
    pub availability: FeatAvailability,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct FeatPrerequisites {
    pub feats: Vec<PrereqFeat>,
    pub abilities: Vec<PrereqAbility>,
    pub bab: Option<PrereqValue>,
    pub level: Option<PrereqValue>,
    pub caster_level: Option<PrereqValue>,
    pub spell_level: Option<PrereqValue>,
    pub skills: Vec<PrereqSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrereqValue {
    pub required: i32,
    pub current: i32,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrereqSkill {
    pub skill: String,
    pub required: i32,
    pub current: i32,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrereqFeat {
    pub id: FeatId,
    pub name: String,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrereqAbility {
    pub ability: String,
    pub required: i32,
    pub current: i32,
    pub met: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct FeatSummary {
    pub total: i32,
    #[serde(rename = "protected")]
    pub protected_feats: Vec<FeatInfo>,
    pub class_feats: Vec<FeatInfo>,
    pub general_feats: Vec<FeatInfo>,
    pub custom_feats: Vec<FeatInfo>,
    pub background_feats: Vec<FeatInfo>,
    pub domain_feats: Vec<FeatInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DomainInfo {
    pub id: DomainId,
    pub name: String,
    pub description: String,
    pub granted_feat: Option<FeatId>,
    pub castable_feat: Option<FeatId>,
    pub epithet_feat: Option<FeatId>,
    pub has_domain: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Type)]
pub enum FeatSource {
    #[default]
    Unknown,
    Manual,
    Class,
    Race,
    Domain,
    Level,
    Background,
}

impl FeatSource {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "manual" => Self::Manual,
            "class" => Self::Class,
            "race" | "racial" => Self::Race,
            "domain" => Self::Domain,
            "level" => Self::Level,
            "background" => Self::Background,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Manual => "manual",
            Self::Class => "class",
            Self::Race => "race",
            Self::Domain => "domain",
            Self::Level => "level",
            Self::Background => "background",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FeatEntry {
    pub feat_id: FeatId,
    pub source: FeatSource,
    pub uses: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct FeatSlots {
    pub total_general_slots: i32,
    pub total_bonus_slots: i32,
    pub total_slots: i32,
    pub filled_slots: i32,
    pub open_slots: i32,
    pub open_general_slots: i32,
    pub open_bonus_slots: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct FeatAddResult {
    pub success: bool,
    pub feat_id: FeatId,
    pub auto_added_feats: Vec<AutoAddedFeat>,
    pub auto_modified_abilities: Vec<AbilityChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AutoAddedFeat {
    pub feat_id: FeatId,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AbilityChange {
    pub ability: String,
    pub old_value: i32,
    pub new_value: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct PrerequisiteResult {
    pub can_take: bool,
    pub missing_requirements: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct FeatsState {
    pub summary: FeatSummary,
    pub feat_slots: FeatSlots,
    pub domains: Vec<DomainInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct FeatAvailability {
    pub available: bool,
    pub reasons: Vec<String>,
}

impl PrerequisiteResult {
    pub fn success() -> Self {
        Self {
            can_take: true,
            missing_requirements: vec![],
        }
    }

    pub fn failure(requirements: Vec<String>) -> Self {
        Self {
            can_take: false,
            missing_requirements: requirements,
        }
    }
}

enum SaveType {
    Universal,
    Fortitude,
    Reflex,
    Will,
    FortitudeAndWill,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct FeatDescriptionSections<'a> {
    flavor: &'a str,
    prerequisites: Option<&'a str>,
    effects: Option<&'a str>,
}

static SAVE_PATTERNS: LazyLock<Vec<(Regex, SaveType)>> = LazyLock::new(|| {
    vec![
        (
            Regex::new(r"([+-]\d+)\s+(?:\w+\s+)?bonus\s+(?:to|on)\s+all\s+(?:saving\s+throws|saves)")
                .unwrap(),
            SaveType::Universal,
        ),
        (
            Regex::new(r"([+-]\d+)\s+(?:to\s+)?all\s+(?:saving\s+throws|saves)").unwrap(),
            SaveType::Universal,
        ),
        (
            Regex::new(
                r"([+-]\d+)\s+(?:bonus\s+)?(?:to|on)\s+Fortitude\s+and\s+Will\s+(?:saving\s+throws|saves?)",
            )
            .unwrap(),
            SaveType::FortitudeAndWill,
        ),
        (
            Regex::new(
                r"([+-]\d+)\s+(?:\w+\s+)?bonus\s+(?:to|on)\s+(?:all\s+)?Fortitude\s+(?:saving\s+throws|saves?)",
            )
            .unwrap(),
            SaveType::Fortitude,
        ),
        (
            Regex::new(r"([+-]\d+)\s+Fortitude\s+Save").unwrap(),
            SaveType::Fortitude,
        ),
        (
            Regex::new(
                r"([+-]\d+)\s+(?:\w+\s+)?bonus\s+(?:to|on)\s+(?:all\s+)?Reflex\s+(?:saving\s+throws|saves?)",
            )
            .unwrap(),
            SaveType::Reflex,
        ),
        (
            Regex::new(r"([+-]\d+)\s+Reflex\s+Save").unwrap(),
            SaveType::Reflex,
        ),
        (
            Regex::new(
                r"([+-]\d+)\s+(?:\w+\s+)?bonus\s+(?:to|on)\s+(?:all\s+)?Will\s+(?:saving\s+throws|saves?)",
            )
            .unwrap(),
            SaveType::Will,
        ),
        (
            Regex::new(r"([+-]\d+)\s+(?:to|on)\s+all\s+Will\s+(?:saving\s+throws|saves?)").unwrap(),
            SaveType::Will,
        ),
        (
            Regex::new(r"([+-]\d+)\s+Will\s+Save").unwrap(),
            SaveType::Will,
        ),
        (
            Regex::new(r"([+-]\d+)\s+racial\s+(?:\w+\s+)?bonus\s+on\s+all\s+saving\s+throws").unwrap(),
            SaveType::Universal,
        ),
    ]
});

static AC_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"\+(\d+)\s+(?:\w+\s+)?bonus\s+to\s+(?:Armor\s+Class|AC)").unwrap(),
        Regex::new(r"\+(\d+)\s+(?:to\s+)?AC(?:\s|\.|\,)").unwrap(),
        Regex::new(r"\+(\d+)\s+AC\s+bonus").unwrap(),
    ]
});

static AC_DODGE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\+(\d+)").unwrap());

static INITIATIVE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"\+(\d+)\s+(?:\w+\s+)?bonus\s+to\s+initiative").unwrap(),
        Regex::new(r"\+(\d+)\s+(?:to\s+)?initiative").unwrap(),
    ]
});

static FEAT_PROGRESSION_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)[\s_]?(\d+)$").unwrap());

const SAVE_CONDITIONAL_KEYWORDS: &[&str] = &[
    "against",
    "vs ",
    "versus",
    "to avoid",
    "made to",
    "while raging",
    "while in a rage",
    "while in a frenzy",
    "during rage",
    "during frenzy",
    "when raging",
];

/// Keywords in feat descriptions indicating bonuses require a temporary active state.
const ACTIVATION_CONDITIONAL_KEYWORDS: &[&str] = &[
    "while raging",
    "while in a rage",
    "while in a frenzy",
    "during rage",
    "during frenzy",
    "when raging",
    "while in defensive stance",
    "during defensive stance",
    "while frenzied",
    "when frenzied",
];
const AC_CONDITIONAL_KEYWORDS: &[&str] = &[
    "against",
    "vs ",
    "versus",
    "when ",
    "while ",
    "if ",
    "when wielding",
    "when wearing",
    "when using",
    "when fighting",
];

/// Determines if a feat's bonuses are conditional (only apply in specific situations).
/// Uses description keywords to detect conditional bonuses. Works with modded feats.
/// Note: IsActive+SPELLID is NOT used as a standalone filter because permanent feats
/// like Blessed of Waukeen also use spells to apply their effects.
fn is_conditional_feat(
    _feat_data: &ahash::AHashMap<String, Option<String>>,
    description_lower: &str,
    context_keywords: &[&str],
) -> bool {
    if ACTIVATION_CONDITIONAL_KEYWORDS
        .iter()
        .any(|kw| description_lower.contains(kw))
    {
        return true;
    }

    context_keywords
        .iter()
        .any(|kw| description_lower.contains(kw))
}

/// Pre-compute domain feat ID sets from the domains 2DA table.
/// Returns (all_domain_feats, epithet_feats) for O(1) lookups.
pub fn build_domain_feat_sets(game_data: &GameData) -> (HashSet<i32>, HashSet<i32>) {
    let mut all_domain_feats = HashSet::new();
    let mut epithet_feats = HashSet::new();

    let Some(domains_table) = game_data.get_table("domains") else {
        return (all_domain_feats, epithet_feats);
    };

    for row_id in 0..domains_table.row_count() {
        let Some(domain_data) = domains_table.get_by_id(row_id as i32) else {
            continue;
        };

        for field in [
            "granted_feat",
            "castable_feat",
            "GrantedFeat",
            "CastableFeat",
        ] {
            if let Some(feat_id) = domain_data
                .get(field)
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&id| id >= 0)
            {
                all_domain_feats.insert(feat_id);
            }
        }

        for field in ["epithet_feat", "EpithetFeat"] {
            if let Some(feat_id) = domain_data
                .get(field)
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&id| id >= 0)
            {
                all_domain_feats.insert(feat_id);
                epithet_feats.insert(feat_id);
            }
        }
    }

    (all_domain_feats, epithet_feats)
}

impl Character {
    // ========== BASIC FEAT ACCESS ==========

    pub fn feat_ids(&self) -> Vec<FeatId> {
        self.feat_entries()
            .into_iter()
            .map(|entry| entry.feat_id)
            .collect()
    }

    pub fn feat_entries(&self) -> Vec<FeatEntry> {
        let Some(feat_list) = self.get_list("FeatList") else {
            return Vec::new();
        };

        feat_list
            .iter()
            .filter_map(|entry| {
                let feat_id = entry.get("Feat").and_then(gff_value_to_i32)?;
                let source = entry
                    .get("Source")
                    .and_then(|v| match v {
                        GffValue::String(s) => Some(FeatSource::parse(s)),
                        GffValue::ResRef(s) => Some(FeatSource::parse(s)),
                        _ => None,
                    })
                    .unwrap_or(FeatSource::Unknown);
                let uses = entry.get("Uses").and_then(gff_value_to_i32);

                Some(FeatEntry {
                    feat_id: FeatId(feat_id),
                    source,
                    uses,
                })
            })
            .collect()
    }

    pub fn has_feat(&self, feat_id: FeatId) -> bool {
        let Some(feat_list) = self.get_list("FeatList") else {
            return false;
        };

        feat_list.iter().any(|entry| {
            entry
                .get("Feat")
                .and_then(gff_value_to_i32)
                .is_some_and(|id| id == feat_id.0)
        })
    }

    pub fn feat_count(&self) -> usize {
        self.get_list("FeatList").map_or(0, std::vec::Vec::len)
    }

    pub fn add_feat(&mut self, feat_id: FeatId) -> Result<(), CharacterError> {
        self.add_feat_with_source(feat_id, FeatSource::Manual)
    }

    pub fn add_feat_with_source(
        &mut self,
        feat_id: FeatId,
        source: FeatSource,
    ) -> Result<(), CharacterError> {
        if self.has_feat(feat_id) {
            return Err(CharacterError::FeatAlreadyExists(feat_id.0));
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

        self.record_feat_change(feat_id, true);

        Ok(())
    }

    /// Add feat with auto-prerequisites - recursively adds missing prerequisite feats
    /// and auto-increases ability scores to meet requirements.
    pub fn add_feat_with_prerequisites(
        &mut self,
        feat_id: FeatId,
        source: FeatSource,
        game_data: &GameData,
    ) -> Result<FeatAddResult, CharacterError> {
        let mut result = FeatAddResult {
            success: false,
            feat_id,
            auto_added_feats: vec![],
            auto_modified_abilities: vec![],
        };

        if self.has_feat(feat_id) {
            result.success = true;
            return Ok(result);
        }

        let feats_table = game_data
            .get_table("feat")
            .ok_or_else(|| CharacterError::TableNotFound("feat".to_string()))?;

        let feat_data = feats_table
            .get_by_id(feat_id.0)
            .ok_or(CharacterError::NotFound {
                entity: "Feat",
                id: feat_id.0,
            })?;

        let ability_fields = [
            ("Str", "MINSTR"),
            ("Dex", "MINDEX"),
            ("Con", "MINCON"),
            ("Int", "MININT"),
            ("Wis", "MINWIS"),
            ("Cha", "MINCHA"),
        ];

        for (ability_field, min_field) in ability_fields {
            let min_val = feat_data
                .get(min_field)
                .or_else(|| feat_data.get(&min_field.to_lowercase()))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&v| v > 0);

            if let Some(min_val) = min_val {
                let current = self.get_i32(ability_field).unwrap_or(10);
                if current < min_val {
                    self.set_i32(ability_field, min_val);
                    result.auto_modified_abilities.push(AbilityChange {
                        ability: ability_field.to_string(),
                        old_value: current,
                        new_value: min_val,
                    });
                }
            }
        }

        for prereq_field in ["PREREQFEAT1", "PREREQFEAT2"] {
            let prereq_id = feat_data
                .get(prereq_field)
                .or_else(|| feat_data.get(&prereq_field.to_lowercase()))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&id| id >= 0);

            if let Some(prereq_id) = prereq_id {
                let prereq_feat_id = FeatId(prereq_id);
                if !self.has_feat(prereq_feat_id) {
                    let nested = self.add_feat_with_prerequisites(
                        prereq_feat_id,
                        FeatSource::Manual,
                        game_data,
                    )?;

                    result.auto_added_feats.extend(nested.auto_added_feats);
                    result
                        .auto_modified_abilities
                        .extend(nested.auto_modified_abilities);

                    let prereq_label = self.get_feat_name(prereq_feat_id, game_data);
                    result.auto_added_feats.push(AutoAddedFeat {
                        feat_id: prereq_feat_id,
                        label: prereq_label,
                    });
                }
            }
        }

        self.add_feat_with_source(feat_id, source)?;
        result.success = true;

        Ok(result)
    }

    pub fn remove_feat(&mut self, feat_id: FeatId) -> Result<(), CharacterError> {
        if !self.has_feat(feat_id) {
            return Err(CharacterError::FeatNotFound(feat_id.0));
        }

        let mut feat_list = self.get_list_owned("FeatList").unwrap_or_default();

        feat_list.retain(|entry| {
            entry
                .get("Feat")
                .and_then(gff_value_to_i32)
                .map_or(true, |id| id != feat_id.0)
        });

        self.set_list("FeatList", feat_list);

        if let Some(mut lvl_stat_list) = self.get_list_owned("LvlStatList") {
            for stat_entry in &mut lvl_stat_list {
                if let Some(mut history_feat_list) =
                    super::gff_helpers::extract_list_from_map(stat_entry, "FeatList")
                {
                    history_feat_list.retain(|entry| {
                        entry.get("Feat").and_then(gff_value_to_i32) != Some(feat_id.0)
                    });
                    stat_entry.insert(
                        "FeatList".to_string(),
                        GffValue::ListOwned(history_feat_list),
                    );
                }
            }
            self.set_list("LvlStatList", lvl_stat_list);
        }

        Ok(())
    }

    /// Check if a feat is part of a progression chain (e.g., Toughness_1 -> Toughness_2).
    /// Returns the old feat ID that should be removed, or None if no progression detected.
    pub fn check_feat_progression(
        &self,
        new_feat_id: FeatId,
        game_data: &GameData,
    ) -> Option<FeatId> {
        let feats_table = game_data.get_table("feat")?;
        let new_feat_data = feats_table.get_by_id(new_feat_id.0)?;

        let new_label = new_feat_data
            .get("label")
            .and_then(|s| s.as_ref())
            .cloned()?;

        let captures = FEAT_PROGRESSION_PATTERN.captures(&new_label)?;
        let base_name = captures.get(1)?.as_str().trim_end_matches('_');
        let new_number: i32 = captures.get(2)?.as_str().parse().ok()?;

        if new_number < 2 {
            return None;
        }

        let current_feat_ids = self.feat_ids();

        for feat_id in current_feat_ids {
            let Some(feat_data) = feats_table.get_by_id(feat_id.0) else {
                continue;
            };

            let Some(label) = feat_data.get("label").and_then(|s| s.as_ref()) else {
                continue;
            };

            if !label.starts_with(base_name) {
                continue;
            }

            if let Some(old_captures) = FEAT_PROGRESSION_PATTERN.captures(label) {
                let old_base = old_captures
                    .get(1)
                    .map(|m| m.as_str().trim_end_matches('_'))
                    .unwrap_or("");
                let old_number: i32 = old_captures
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);

                if old_base == base_name && old_number < new_number {
                    return Some(feat_id);
                }
            } else if label == base_name || label.trim_end_matches('_') == base_name {
                return Some(feat_id);
            }
        }

        None
    }

    // ========== FEAT USES ==========

    pub fn get_feat_uses(&self, feat_id: FeatId) -> Option<i32> {
        let feat_list = self.get_list("FeatList")?;

        for entry in feat_list {
            let id = entry.get("Feat").and_then(gff_value_to_i32)?;
            if id == feat_id.0 {
                let uses = entry.get("Uses").and_then(gff_value_to_i32)?;
                return if uses >= 0 { Some(uses) } else { None };
            }
        }

        None
    }

    pub fn set_feat_uses(&mut self, feat_id: FeatId, uses: i32) -> bool {
        let mut feat_list = match self.get_list_owned("FeatList") {
            Some(list) => list,
            None => return false,
        };

        for entry in &mut feat_list {
            let id = entry.get("Feat").and_then(gff_value_to_i32);
            if id == Some(feat_id.0) {
                entry.insert("Uses".to_string(), GffValue::Int(uses));
                self.set_list("FeatList", feat_list);
                return true;
            }
        }

        false
    }

    // ========== FEAT SOURCE TRACKING ==========

    pub fn feat_source(&self, feat_id: FeatId) -> Option<FeatSource> {
        let entries = self.feat_entries();
        entries
            .iter()
            .find(|e| e.feat_id == feat_id)
            .map(|e| e.source)
    }

    pub fn is_feat_protected(&self, feat_id: FeatId, game_data: &GameData) -> bool {
        if let Some(source) = self.feat_source(feat_id)
            && matches!(source, FeatSource::Race | FeatSource::Background)
        {
            return true;
        }

        self.is_domain_epithet_feat(feat_id, game_data)
    }

    // ========== DOMAIN METHODS ==========

    pub fn get_character_domains(&self) -> Vec<DomainId> {
        let mut domains = Vec::new();

        let Some(class_list) = self.get_list("ClassList") else {
            debug!("get_character_domains: No ClassList found");
            return domains;
        };

        debug!(
            "get_character_domains: Found {} class entries",
            class_list.len()
        );

        for (idx, class_entry) in class_list.iter().enumerate() {
            let domain1_raw = class_entry.get("Domain1");
            let domain2_raw = class_entry.get("Domain2");
            debug!(
                "get_character_domains: Class {} - Domain1 raw: {:?}, Domain2 raw: {:?}",
                idx, domain1_raw, domain2_raw
            );

            if let Some(domain1) = class_entry.get("Domain1").and_then(gff_value_to_i32)
                && domain1 >= 0
            {
                debug!("get_character_domains: Adding Domain1 = {}", domain1);
                domains.push(DomainId(domain1));
            }
            if let Some(domain2) = class_entry.get("Domain2").and_then(gff_value_to_i32)
                && domain2 >= 0
            {
                debug!("get_character_domains: Adding Domain2 = {}", domain2);
                domains.push(DomainId(domain2));
            }
        }

        debug!("get_character_domains: Total domains found: {:?}", domains);
        domains
    }

    pub fn is_domain_feat(&self, feat_id: FeatId, game_data: &GameData) -> bool {
        let Some(domains_table) = game_data.get_table("domains") else {
            return false;
        };

        for row_id in 0..domains_table.row_count() {
            let Some(domain_data) = domains_table.get_by_id(row_id as i32) else {
                continue;
            };

            let feats = [
                domain_data
                    .get("granted_feat")
                    .and_then(|s| s.as_ref()?.parse::<i32>().ok()),
                domain_data
                    .get("castable_feat")
                    .and_then(|s| s.as_ref()?.parse::<i32>().ok()),
                domain_data
                    .get("epithet_feat")
                    .and_then(|s| s.as_ref()?.parse::<i32>().ok()),
            ];

            for domain_feat in feats.into_iter().flatten() {
                if domain_feat >= 0 && domain_feat == feat_id.0 {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_domain_epithet_feat(&self, feat_id: FeatId, game_data: &GameData) -> bool {
        let Some(domains_table) = game_data.get_table("domains") else {
            return false;
        };

        for row_id in 0..domains_table.row_count() {
            let Some(domain_data) = domains_table.get_by_id(row_id as i32) else {
                continue;
            };

            let epithet = domain_data
                .get("epithet_feat")
                .and_then(|s| s.as_ref()?.parse::<i32>().ok());

            if epithet == Some(feat_id.0) {
                return true;
            }
        }

        false
    }

    pub fn add_domain(
        &mut self,
        domain_id: DomainId,
        game_data: &GameData,
    ) -> Result<Vec<FeatId>, CharacterError> {
        let Some(domains_table) = game_data.get_table("domains") else {
            return Err(CharacterError::TableNotFound("domains".to_string()));
        };

        let Some(domain_data) = domains_table.get_by_id(domain_id.0) else {
            return Err(CharacterError::NotFound {
                entity: "Domain",
                id: domain_id.0,
            });
        };

        let mut added_feats = Vec::new();

        let granted_feat = domain_data
            .get("granted_feat")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&id| id >= 0)
            .map(FeatId);

        let castable_feat = domain_data
            .get("castable_feat")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&id| id >= 0)
            .map(FeatId);

        let epithet_feat = domain_data
            .get("epithet_feat")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&id| id >= 0)
            .map(FeatId);

        for feat_id in [granted_feat, castable_feat, epithet_feat]
            .into_iter()
            .flatten()
        {
            if !self.has_feat(feat_id) {
                self.add_feat_with_source(feat_id, FeatSource::Domain)?;
                added_feats.push(feat_id);
            }
        }

        Ok(added_feats)
    }

    pub fn remove_domain(
        &mut self,
        domain_id: DomainId,
        game_data: &GameData,
    ) -> Result<Vec<FeatId>, CharacterError> {
        // Remove domain spells first (before feats) - cascade from Python implementation
        let _removed_spells = self.remove_domain_spells(domain_id, game_data)?;

        let Some(domains_table) = game_data.get_table("domains") else {
            return Err(CharacterError::TableNotFound("domains".to_string()));
        };

        let Some(domain_data) = domains_table.get_by_id(domain_id.0) else {
            return Err(CharacterError::NotFound {
                entity: "Domain",
                id: domain_id.0,
            });
        };

        let mut removed_feats = Vec::new();

        let granted_feat = domain_data
            .get("granted_feat")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&id| id >= 0)
            .map(FeatId);

        let castable_feat = domain_data
            .get("castable_feat")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&id| id >= 0)
            .map(FeatId);

        let epithet_feat = domain_data
            .get("epithet_feat")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&id| id >= 0)
            .map(FeatId);

        for feat_id in [granted_feat, castable_feat, epithet_feat]
            .into_iter()
            .flatten()
        {
            if self.has_feat(feat_id) {
                self.remove_feat(feat_id)?;
                removed_feats.push(feat_id);
            }
        }

        Ok(removed_feats)
    }

    pub fn remove_all_domain_feats(&mut self, game_data: &GameData) -> Vec<FeatId> {
        let Some(domains_table) = game_data.get_table("domains") else {
            return Vec::new();
        };

        let mut removed = Vec::new();
        let current_feats = self.feat_ids();

        for row_id in 0..domains_table.row_count() {
            let Some(domain_data) = domains_table.get_by_id(row_id as i32) else {
                continue;
            };

            let feats = [
                domain_data
                    .get("granted_feat")
                    .and_then(|s| s.as_ref()?.parse::<i32>().ok()),
                domain_data
                    .get("castable_feat")
                    .and_then(|s| s.as_ref()?.parse::<i32>().ok()),
                domain_data
                    .get("epithet_feat")
                    .and_then(|s| s.as_ref()?.parse::<i32>().ok()),
            ];

            for domain_feat in feats.into_iter().flatten() {
                let feat_id = FeatId(domain_feat);
                if domain_feat >= 0
                    && current_feats.contains(&feat_id)
                    && self.remove_feat(feat_id).is_ok()
                {
                    removed.push(feat_id);
                }
            }
        }

        removed
    }

    pub fn change_cleric_domains(
        &mut self,
        old_domain_ids: &[DomainId],
        new_domain_ids: &[DomainId],
        game_data: &GameData,
    ) -> Result<(Vec<FeatId>, Vec<FeatId>), CharacterError> {
        let old_set: std::collections::HashSet<_> = old_domain_ids.iter().collect();
        let new_set: std::collections::HashSet<_> = new_domain_ids.iter().collect();

        let mut removed = Vec::new();
        let mut added = Vec::new();

        for domain_id in old_domain_ids {
            if !new_set.contains(domain_id) {
                let feats = self.remove_domain(*domain_id, game_data)?;
                removed.extend(feats);
            }
        }

        for domain_id in new_domain_ids {
            if !old_set.contains(domain_id) {
                let feats = self.add_domain(*domain_id, game_data)?;
                added.extend(feats);
            }
        }

        Ok((removed, added))
    }

    // ========== FEAT SLOTS CALCULATION (Blueprint Method) ==========

    const QUICK_TO_MASTER_FEAT_ID: i32 = 258;
    const AUTO_GRANTED_FEAT_TYPES: [i32; 3] = [8192, 128, 512]; // Domain, Background, History

    /// Calculate feat slots using the Blueprint Method - analyzes level history for accuracy.
    pub fn get_feat_slots(&self, game_data: &GameData) -> FeatSlots {
        let slot_sequence = self.build_feat_slot_sequence(game_data);

        if slot_sequence.is_empty() {
            return FeatSlots::default();
        }
        let filled_general_slots = slot_sequence
            .iter()
            .filter(|(is_general, feat)| *is_general && feat.is_some())
            .count() as i32;
        let filled_bonus_slots = slot_sequence
            .iter()
            .filter(|(is_general, feat)| !*is_general && feat.is_some())
            .count() as i32;
        let open_general_slots = slot_sequence
            .iter()
            .filter(|(is_general, feat)| *is_general && feat.is_none())
            .count() as i32;
        let open_bonus_slots = slot_sequence
            .iter()
            .filter(|(is_general, feat)| !*is_general && feat.is_none())
            .count() as i32;

        let racial_bonus = self.get_racial_bonus_feats();

        let total_general_slots = filled_general_slots + open_general_slots + racial_bonus;
        let total_bonus_slots = filled_bonus_slots + open_bonus_slots;
        let total_slots = total_general_slots + total_bonus_slots;

        let open_slots = open_general_slots + open_bonus_slots;
        let filled_slots = filled_general_slots + filled_bonus_slots;

        FeatSlots {
            total_general_slots,
            total_bonus_slots,
            total_slots,
            filled_slots,
            open_slots,
            open_general_slots,
            open_bonus_slots,
        }
    }

    /// Returns the feat IDs that were explicitly chosen via general/bonus feat slots.
    /// Uses the same logic as get_feat_slots to identify selectable (non-auto-granted) feats.
    pub fn get_slot_chosen_feat_ids(&self, game_data: &GameData) -> Vec<FeatId> {
        self.build_feat_slot_sequence(game_data)
            .into_iter()
            .filter_map(|(_, feat_id)| feat_id)
            .collect()
    }

    pub fn normalize_level_one_feat_history_for_save(&mut self) {
        let Some(mut lvl_stat_list) = self.get_list_owned("LvlStatList") else {
            return;
        };
        if lvl_stat_list.is_empty() {
            return;
        }

        let top_level_feat_list = self.get_list_owned("FeatList").unwrap_or_default();
        if top_level_feat_list.is_empty() {
            return;
        }

        let top_level_feat_ids: Vec<i32> = top_level_feat_list
            .iter()
            .filter_map(|entry| entry.get("Feat").and_then(gff_value_to_i32))
            .collect();
        if top_level_feat_ids.is_empty() {
            return;
        }

        let mut residual_counts: HashMap<i32, usize> = HashMap::new();
        for feat_id in &top_level_feat_ids {
            *residual_counts.entry(*feat_id).or_insert(0) += 1;
        }

        for entry in lvl_stat_list.iter().skip(1) {
            let Some(history_feat_list) =
                super::gff_helpers::extract_list_from_map(entry, "FeatList")
            else {
                continue;
            };

            for feat_entry in history_feat_list {
                let Some(feat_id) = feat_entry.get("Feat").and_then(gff_value_to_i32) else {
                    continue;
                };

                let Some(count) = residual_counts.get_mut(&feat_id) else {
                    return;
                };
                if *count == 0 {
                    return;
                }
                *count -= 1;
            }
        }

        let rebuilt_level_one_feats: Vec<IndexMap<String, GffValue<'static>>> = top_level_feat_ids
            .into_iter()
            .filter_map(|feat_id| {
                let count = residual_counts.get_mut(&feat_id)?;
                if *count == 0 {
                    return None;
                }
                *count -= 1;

                let mut feat_entry = IndexMap::new();
                feat_entry.insert("Feat".to_string(), GffValue::Word(feat_id as u16));
                Some(feat_entry)
            })
            .collect();

        let current_level_one_feats: Vec<i32> = lvl_stat_list[0]
            .get("FeatList")
            .and_then(|value| match value {
                GffValue::ListOwned(list) => Some(
                    list.iter()
                        .filter_map(|entry| entry.get("Feat").and_then(gff_value_to_i32))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        let rebuilt_feat_ids: Vec<i32> = rebuilt_level_one_feats
            .iter()
            .filter_map(|entry| entry.get("Feat").and_then(gff_value_to_i32))
            .collect();

        if current_level_one_feats == rebuilt_feat_ids {
            return;
        }

        lvl_stat_list[0].insert(
            "FeatList".to_string(),
            GffValue::ListOwned(rebuilt_level_one_feats),
        );
        self.set_list("LvlStatList", lvl_stat_list);
    }

    fn build_feat_slot_sequence(&self, game_data: &GameData) -> Vec<(bool, Option<FeatId>)> {
        let level_history = self.level_history();
        let mut slots: Vec<(bool, Option<FeatId>)> = Vec::new();

        let mut class_level_tracker: std::collections::HashMap<i32, i32> =
            std::collections::HashMap::new();
        let mut class_feat_table_cache: std::collections::HashMap<
            i32,
            std::collections::HashMap<i32, i32>,
        > = std::collections::HashMap::new();

        for (total_level_idx, level_entry) in level_history.iter().enumerate() {
            let total_level = (total_level_idx + 1) as i32;
            let class_id = level_entry.class_id.0;

            *class_level_tracker.entry(class_id).or_insert(0) += 1;
            let class_level = class_level_tracker[&class_id];

            let level_feat_ids: Vec<i32> = level_entry.feats_gained.iter().map(|f| f.0).collect();

            class_feat_table_cache
                .entry(class_id)
                .or_insert_with(|| self.load_class_feat_table(ClassId(class_id), game_data));
            let class_feat_table = &class_feat_table_cache[&class_id];

            let mut selectable_feats: Vec<i32> = Vec::new();
            for feat_id in &level_feat_ids {
                if class_feat_table.get(feat_id).copied() == Some(3) {
                    continue;
                }
                if !self.is_slot_eligible_feat(FeatId(*feat_id), game_data) {
                    continue;
                }
                selectable_feats.push(*feat_id);
            }

            let has_general_slot = if total_level <= 20 {
                total_level == 1 || total_level % 3 == 0
            } else {
                total_level % 2 != 0
            };

            if has_general_slot {
                let feat_id = selectable_feats.first().copied().map(FeatId);
                if feat_id.is_some() {
                    selectable_feats.remove(0);
                }
                slots.push((true, feat_id));
            }

            let has_bonus_slot =
                self.check_bonus_feat_slot(ClassId(class_id), class_level, game_data);
            if has_bonus_slot {
                let feat_id = selectable_feats.first().copied().map(FeatId);
                if feat_id.is_some() {
                    selectable_feats.remove(0);
                }
                slots.push((false, feat_id));
            }
        }

        if slots.is_empty() {
            return slots;
        }

        let mut occupied_feats: HashSet<FeatId> =
            slots.iter().filter_map(|(_, feat_id)| *feat_id).collect();

        let extra_manual_feats: Vec<FeatId> = self
            .feat_entries()
            .into_iter()
            .filter(|entry| entry.source == FeatSource::Manual)
            .map(|entry| entry.feat_id)
            .filter(|feat_id| !occupied_feats.contains(feat_id))
            .filter(|feat_id| self.is_slot_eligible_feat(*feat_id, game_data))
            .collect();

        let mut extra_iter = extra_manual_feats.into_iter();
        for (_, feat_id) in &mut slots {
            if feat_id.is_none()
                && let Some(extra_feat_id) = extra_iter.next()
            {
                *feat_id = Some(extra_feat_id);
                occupied_feats.insert(extra_feat_id);
            }
        }

        slots
    }

    fn is_slot_eligible_feat(&self, feat_id: FeatId, game_data: &GameData) -> bool {
        let Some(feat_info) = self.get_feat_info(feat_id, game_data) else {
            return false;
        };
        let is_domain = self.is_domain_feat(feat_id, game_data);
        let raw_feat_type = self
            .get_feat_type(feat_id, game_data)
            .unwrap_or(feat_info.feat_type);
        let raw_category = FeatCategory::from_feat_type(raw_feat_type, is_domain);

        if feat_info.is_protected {
            return false;
        }

        if matches!(
            self.feat_source(feat_id),
            Some(
                FeatSource::Race | FeatSource::Background | FeatSource::Domain | FeatSource::Class
            )
        ) {
            return false;
        }

        if matches!(
            raw_category,
            FeatCategory::Background
                | FeatCategory::History
                | FeatCategory::Heritage
                | FeatCategory::Racial
                | FeatCategory::Domain
                | FeatCategory::Class
        ) {
            return false;
        }

        if matches!(
            feat_info.category,
            FeatCategory::Background
                | FeatCategory::History
                | FeatCategory::Heritage
                | FeatCategory::Racial
                | FeatCategory::Domain
                | FeatCategory::Class
        ) {
            return false;
        }

        let label_upper = feat_info.label.to_ascii_uppercase();
        let name_upper = feat_info.name.to_ascii_uppercase();
        if label_upper.starts_with("FEAT_EPITHET_")
            || name_upper.starts_with("FEAT_EPITHET_")
            || label_upper.starts_with("WATCH_RANK_")
            || name_upper.starts_with("WATCH_RANK_")
        {
            return false;
        }

        !Self::AUTO_GRANTED_FEAT_TYPES.contains(&raw_feat_type.0)
    }

    fn load_class_feat_table(
        &self,
        class_id: ClassId,
        game_data: &GameData,
    ) -> std::collections::HashMap<i32, i32> {
        let mut feat_table = std::collections::HashMap::new();

        let Some(classes_table) = game_data.get_table("classes") else {
            return feat_table;
        };

        let Some(class_data) = classes_table.get_by_id(class_id.0) else {
            return feat_table;
        };

        let feats_table_name_opt = class_data
            .get("FeatsTable")
            .or_else(|| class_data.get("feats_table"))
            .and_then(|s| s.as_ref());

        let Some(feats_table_name) = feats_table_name_opt else {
            return feat_table;
        };

        if feats_table_name.starts_with("****") {
            return feat_table;
        }

        let Some(table) = game_data.get_table(&feats_table_name.to_lowercase()) else {
            return feat_table;
        };

        for row_id in 0..table.row_count() {
            let Some(row) = table.get_by_id(row_id as i32) else {
                continue;
            };

            let feat_id = row
                .get("featindex")
                .or_else(|| row.get("FeatIndex"))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .unwrap_or(-1);

            let list_type = row
                .get("list")
                .or_else(|| row.get("List"))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .unwrap_or(0);

            if feat_id >= 0 {
                feat_table.insert(feat_id, list_type);
            }
        }

        feat_table
    }

    fn check_bonus_feat_slot(
        &self,
        class_id: ClassId,
        class_level: i32,
        game_data: &GameData,
    ) -> bool {
        let Some(classes_table) = game_data.get_table("classes") else {
            return false;
        };

        let Some(class_data) = classes_table.get_by_id(class_id.0) else {
            return false;
        };

        let bonus_table_name_opt = class_data
            .get("BonusFeatsTable")
            .or_else(|| class_data.get("bonus_feats_table"))
            .and_then(|s| s.as_ref());

        let Some(bonus_table_name) = bonus_table_name_opt else {
            return false;
        };

        if bonus_table_name.starts_with("****") {
            return false;
        }

        let Some(bonus_table) = game_data.get_table(&bonus_table_name.to_lowercase()) else {
            return false;
        };

        let level_idx = class_level - 1;
        if level_idx < 0 {
            return false;
        }

        let Some(level_data) = bonus_table.get_by_id(level_idx) else {
            return false;
        };

        let bonus = level_data
            .get("bonus")
            .or_else(|| level_data.get("Bonus"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .unwrap_or(0);

        bonus > 0
    }

    fn get_racial_bonus_feats(&self) -> i32 {
        i32::from(self.has_feat(FeatId(Self::QUICK_TO_MASTER_FEAT_ID)))
    }

    fn get_feat_type(&self, feat_id: FeatId, game_data: &GameData) -> Option<FeatType> {
        let feats_table = game_data.get_table("feat")?;
        let feat_data = feats_table.get_by_id(feat_id.0)?;

        let feat_type_str = feat_data
            .get("feat")
            .or_else(|| feat_data.get("FEAT"))
            .and_then(|s| s.as_ref())
            .map_or("0", std::string::String::as_str);

        Some(FeatType::from_string(feat_type_str))
    }

    /// Calculate general feat slots (kept for convenience/backwards compat).
    pub fn calculate_general_feat_slots(&self) -> i32 {
        let total_level = self.total_level();
        if total_level <= 0 {
            return 0;
        }

        let heroic_level = total_level.min(20);
        let epic_levels = (total_level - 20).max(0);

        let heroic_slots = 1 + heroic_level / 3;
        let epic_slots = (epic_levels + 1) / 2;

        heroic_slots + epic_slots + self.get_racial_bonus_feats()
    }

    /// Calculate bonus feat slots from class tables (kept for convenience/backwards compat).
    pub fn calculate_bonus_feat_slots(&self, game_data: &GameData) -> i32 {
        let class_entries = self.class_entries();
        let mut total_bonus = 0;

        for entry in &class_entries {
            total_bonus += self.get_bonus_feats_for_class(entry.class_id, entry.level, game_data);
        }

        total_bonus
    }

    pub fn get_bonus_feats_for_class(
        &self,
        class_id: ClassId,
        level: i32,
        game_data: &GameData,
    ) -> i32 {
        let mut count = 0;
        for lvl in 1..=level {
            if self.check_bonus_feat_slot(class_id, lvl, game_data) {
                count += 1;
            }
        }
        count
    }

    // ========== PREREQUISITE VALIDATION ==========

    pub fn validate_feat_prerequisites(
        &self,
        feat_id: FeatId,
        game_data: &GameData,
    ) -> PrerequisiteResult {
        let Some(feats_table) = game_data.get_table("feat") else {
            return PrerequisiteResult::failure(vec!["Feat table not loaded".to_string()]);
        };

        let Some(feat_data) = feats_table.get_by_id(feat_id.0) else {
            return PrerequisiteResult::failure(vec![format!("Feat {} not found", feat_id.0)]);
        };

        let mut missing = Vec::new();

        let prereq_feat1 = feat_data
            .get("PREREQFEAT1")
            .or_else(|| feat_data.get("prereq_feat1"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok());
        let prereq_feat2 = feat_data
            .get("PREREQFEAT2")
            .or_else(|| feat_data.get("prereq_feat2"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok());

        if let Some(prereq_id) = prereq_feat1
            && prereq_id >= 0
            && !self.has_feat(FeatId(prereq_id))
        {
            let prereq_name = self.get_feat_name(FeatId(prereq_id), game_data);
            missing.push(format!("Requires: {prereq_name}"));
        }

        if let Some(prereq_id) = prereq_feat2
            && prereq_id >= 0
            && !self.has_feat(FeatId(prereq_id))
        {
            let prereq_name = self.get_feat_name(FeatId(prereq_id), game_data);
            missing.push(format!("Requires: {prereq_name}"));
        }

        let str_score = self.get_i32("Str").unwrap_or(10);
        let dex_score = self.get_i32("Dex").unwrap_or(10);
        let con_score = self.get_i32("Con").unwrap_or(10);
        let int_score = self.get_i32("Int").unwrap_or(10);
        let wis_score = self.get_i32("Wis").unwrap_or(10);
        let cha_score = self.get_i32("Cha").unwrap_or(10);

        if let Some(min_str) = feat_data
            .get("MINSTR")
            .or_else(|| feat_data.get("prereq_str"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_str > 0
            && str_score < min_str
        {
            missing.push(format!("Requires Strength {min_str}"));
        }

        if let Some(min_dex) = feat_data
            .get("MINDEX")
            .or_else(|| feat_data.get("prereq_dex"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_dex > 0
            && dex_score < min_dex
        {
            missing.push(format!("Requires Dexterity {min_dex}"));
        }

        if let Some(min_con) = feat_data
            .get("MINCON")
            .or_else(|| feat_data.get("prereq_con"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_con > 0
            && con_score < min_con
        {
            missing.push(format!("Requires Constitution {min_con}"));
        }

        if let Some(min_int) = feat_data
            .get("MININT")
            .or_else(|| feat_data.get("prereq_int"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_int > 0
            && int_score < min_int
        {
            missing.push(format!("Requires Intelligence {min_int}"));
        }

        if let Some(min_wis) = feat_data
            .get("MINWIS")
            .or_else(|| feat_data.get("prereq_wis"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_wis > 0
            && wis_score < min_wis
        {
            missing.push(format!("Requires Wisdom {min_wis}"));
        }

        if let Some(min_cha) = feat_data
            .get("MINCHA")
            .or_else(|| feat_data.get("prereq_cha"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_cha > 0
            && cha_score < min_cha
        {
            missing.push(format!("Requires Charisma {min_cha}"));
        }

        if let Some(min_bab) = feat_data
            .get("MINATTACKBONUS")
            .or_else(|| feat_data.get("prereq_bab"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_bab > 0
        {
            let bab = self.calculate_bab(game_data);
            if bab < min_bab {
                missing.push(format!("Requires Base Attack Bonus +{min_bab}"));
            }
        }

        if let Some(min_level) = feat_data
            .get("MinLevel")
            .or_else(|| feat_data.get("MINLEVEL"))
            .or_else(|| feat_data.get("min_level"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            && min_level > 0
        {
            let level = self.total_level();
            if level < min_level {
                missing.push(format!("Requires Level {min_level}"));
            }
        }

        if missing.is_empty() {
            PrerequisiteResult::success()
        } else {
            PrerequisiteResult::failure(missing)
        }
    }

    pub fn get_feat_name(&self, feat_id: FeatId, game_data: &GameData) -> String {
        let Some(feats_table) = game_data.get_table("feat") else {
            return format!("Feat {}", feat_id.0);
        };

        let Some(feat_data) = feats_table.get_by_id(feat_id.0) else {
            return format!("Feat {}", feat_id.0);
        };

        let field_mapper = FieldMapper::new();

        // Use FieldMapper to find "name" field (handles case and aliases like NameRef)
        // Priority: Check "feat_name_strref" (FEAT/Feat) first as per Python implementation
        let name_strref = field_mapper
            .get_field_value(&feat_data, "feat_name_strref")
            .or_else(|| field_mapper.get_field_value(&feat_data, "name"))
            .and_then(|s| s.parse::<i32>().ok());

        if let Some(strref) = name_strref
            && let Some(name) = game_data.get_string(strref)
            && !name.is_empty()
        {
            return name;
        }

        // Use FieldMapper to find "label" field (handles case "Label", "LABEL", etc.)
        field_mapper
            .get_field_value(&feat_data, "label")
            .unwrap_or_else(|| format!("Feat {}", feat_id.0))
    }

    // ========== BONUS CALCULATION FROM FEAT DESCRIPTIONS ==========

    pub fn get_feat_save_bonuses(&self, game_data: &GameData) -> SaveBonuses {
        let mut bonuses = SaveBonuses::default();
        let feat_entries = self.feat_entries();

        for entry in &feat_entries {
            let Some(feats_table) = game_data.get_table("feat") else {
                continue;
            };

            let Some(feat_data) = feats_table.get_by_id(entry.feat_id.0) else {
                continue;
            };

            let description = Self::resolve_feat_description(&feat_data, game_data);
            let sections = Self::parse_feat_description_sections(&description);
            let relevant_text = sections.effects.unwrap_or(&description);
            let relevant_text_lower = relevant_text.to_ascii_lowercase();

            if is_conditional_feat(&feat_data, &relevant_text_lower, SAVE_CONDITIONAL_KEYWORDS) {
                continue;
            }

            let mut found_fort = false;
            let mut found_ref = false;
            let mut found_will = false;

            for (pattern, save_type) in SAVE_PATTERNS.iter() {
                if let Some(captures) = pattern.captures(relevant_text)
                    && let Some(bonus_str) = captures.get(1)
                    && let Ok(bonus_value) = bonus_str.as_str().parse::<i32>()
                {
                    match save_type {
                        SaveType::Universal => {
                            if !found_fort {
                                bonuses.fortitude += bonus_value;
                                found_fort = true;
                            }
                            if !found_ref {
                                bonuses.reflex += bonus_value;
                                found_ref = true;
                            }
                            if !found_will {
                                bonuses.will += bonus_value;
                                found_will = true;
                            }
                        }
                        SaveType::Fortitude => {
                            if !found_fort {
                                bonuses.fortitude += bonus_value;
                                found_fort = true;
                            }
                        }
                        SaveType::Reflex => {
                            if !found_ref {
                                bonuses.reflex += bonus_value;
                                found_ref = true;
                            }
                        }
                        SaveType::Will => {
                            if !found_will {
                                bonuses.will += bonus_value;
                                found_will = true;
                            }
                        }
                        SaveType::FortitudeAndWill => {
                            if !found_fort {
                                bonuses.fortitude += bonus_value;
                                found_fort = true;
                            }
                            if !found_will {
                                bonuses.will += bonus_value;
                                found_will = true;
                            }
                        }
                    }
                }
            }
        }

        bonuses
    }

    pub fn get_feat_ac_bonuses(&self, game_data: &GameData) -> i32 {
        let mut total_ac = 0;
        let feat_entries = self.feat_entries();

        for entry in &feat_entries {
            let Some(feats_table) = game_data.get_table("feat") else {
                continue;
            };

            let Some(feat_data) = feats_table.get_by_id(entry.feat_id.0) else {
                continue;
            };

            let label = feat_data
                .get("label")
                .and_then(|s| s.as_ref().map(std::string::ToString::to_string))
                .unwrap_or_default()
                .to_lowercase();

            let description = Self::resolve_feat_description(&feat_data, game_data);
            let sections = Self::parse_feat_description_sections(&description);
            let relevant_text = sections.effects.unwrap_or(&description);
            let relevant_text_lower = relevant_text.to_ascii_lowercase();

            if is_conditional_feat(&feat_data, &relevant_text_lower, AC_CONDITIONAL_KEYWORDS) {
                continue;
            }

            if label.contains("dodge") || label.contains("mobility") {
                if let Some(captures) = AC_DODGE_PATTERN.captures(relevant_text) {
                    if let Some(bonus_str) = captures.get(1)
                        && let Ok(bonus_value) = bonus_str.as_str().parse::<i32>()
                    {
                        total_ac += bonus_value;
                        continue;
                    }
                } else if label.contains("dodge") {
                    total_ac += 1;
                    continue;
                }
            }

            for pattern in AC_PATTERNS.iter() {
                if let Some(captures) = pattern.captures(relevant_text)
                    && let Some(bonus_str) = captures.get(1)
                    && let Ok(bonus_value) = bonus_str.as_str().parse::<i32>()
                {
                    total_ac += bonus_value;
                    break;
                }
            }
        }

        total_ac
    }

    pub fn get_feat_initiative_bonus(&self, game_data: &GameData) -> i32 {
        let mut bonus = 0;
        let feat_entries = self.feat_entries();

        for entry in &feat_entries {
            let Some(feats_table) = game_data.get_table("feat") else {
                continue;
            };

            let Some(feat_data) = feats_table.get_by_id(entry.feat_id.0) else {
                continue;
            };

            let label = feat_data
                .get("label")
                .and_then(|s| s.as_ref().map(std::string::ToString::to_string))
                .unwrap_or_default()
                .replace(['_', ' '], "")
                .to_lowercase();

            if label.contains("improvedinitiative") {
                bonus += 4;
                continue;
            }

            let description = Self::resolve_feat_description(&feat_data, game_data);
            let sections = Self::parse_feat_description_sections(&description);
            let relevant_text = sections.effects.unwrap_or(&description);

            for pattern in INITIATIVE_PATTERNS.iter() {
                if let Some(captures) = pattern.captures(relevant_text)
                    && let Some(bonus_str) = captures.get(1)
                    && let Ok(bonus_value) = bonus_str.as_str().parse::<i32>()
                {
                    bonus += bonus_value;
                    break;
                }
            }
        }

        bonus
    }

    pub fn get_feat_skill_bonuses(
        &self,
        game_data: &GameData,
    ) -> std::collections::HashMap<String, i32> {
        use std::collections::HashMap;

        static SKILL_BONUS_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
            vec![
                Regex::new(r"(?i)\+(\d+)\s+(?:\w+\s+)?bonus\s+(?:to|on)\s+([A-Za-z][A-Za-z\s]+?)\s+(?:checks|skill)")
                    .expect("Invalid skill bonus regex 1"),
                Regex::new(r"(?i)\+(\d+)\s+(?:to|on)\s+([A-Za-z][A-Za-z\s]+?)\s+(?:checks|skill)")
                    .expect("Invalid skill bonus regex 2"),
                Regex::new(r"(?i)\+(\d+)\s+(?:\w+\s+)?bonus\s+(?:to|on)\s+all\s+([A-Za-z][A-Za-z\s]+?)\s+checks")
                    .expect("Invalid skill bonus regex 3"),
            ]
        });

        static DUAL_SKILL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?i)\+(\d+)\s+(?:\w+\s+)?bonus\s+(?:to|on)\s+([A-Za-z][A-Za-z\s]+?)\s+and\s+([A-Za-z][A-Za-z\s]+?)\s+(?:checks|skills?)")
                .expect("Invalid dual skill regex")
        });

        static EFFECTS_ENTRY_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"([+-]\d+)\s+([A-Za-z][A-Za-z\s]*[A-Za-z])")
                .expect("Invalid effects entry regex")
        });

        static SKILL_CONDITIONAL_KEYWORDS: &[&str] = &[
            "against", "vs ", "versus", "when ", "while ", "if ", "to avoid",
        ];

        let known_skills = Self::build_known_skill_names(game_data);

        let mut skill_bonuses: HashMap<String, i32> = HashMap::new();
        let feat_entries = self.feat_entries();

        for entry in &feat_entries {
            let Some(feats_table) = game_data.get_table("feat") else {
                continue;
            };

            let Some(feat_data) = feats_table.get_by_id(entry.feat_id.0) else {
                continue;
            };

            let description = Self::resolve_feat_description(&feat_data, game_data);
            let sections = Self::parse_feat_description_sections(&description);
            let effects_text = sections.effects;
            let conditional_text = effects_text.unwrap_or(&description);
            let conditional_text_lower = conditional_text.to_ascii_lowercase();

            let feat_label = feat_data
                .get("label")
                .and_then(|s| s.as_ref().map(String::as_str))
                .unwrap_or("unknown");

            if is_conditional_feat(
                &feat_data,
                &conditional_text_lower,
                SKILL_CONDITIONAL_KEYWORDS,
            ) {
                debug!("[feat_skill] Skipping conditional feat '{}'", feat_label);
                continue;
            }

            if let Some(effects_text) = effects_text {
                debug!(
                    "[feat_skill] Feat '{}' has effects line: {:?}",
                    feat_label, effects_text
                );

                for cap in EFFECTS_ENTRY_PATTERN.captures_iter(effects_text) {
                    let Some(value_str) = cap.get(1) else {
                        continue;
                    };
                    let Some(name_match) = cap.get(2) else {
                        continue;
                    };
                    let Ok(bonus_value) = value_str.as_str().parse::<i32>() else {
                        continue;
                    };

                    let normalized = Self::normalize_skill_name(name_match.as_str());
                    if known_skills.contains(&normalized) {
                        *skill_bonuses.entry(normalized).or_insert(0) += bonus_value;
                    }
                }
                continue;
            }

            if let Some(captures) = DUAL_SKILL_PATTERN.captures(&description)
                && let (Some(bonus_str), Some(skill1), Some(skill2)) =
                    (captures.get(1), captures.get(2), captures.get(3))
                && let Ok(bonus_value) = bonus_str.as_str().parse::<i32>()
            {
                let skill1_name = Self::normalize_skill_name(skill1.as_str());
                let skill2_name = Self::normalize_skill_name(skill2.as_str());
                *skill_bonuses.entry(skill1_name).or_insert(0) += bonus_value;
                *skill_bonuses.entry(skill2_name).or_insert(0) += bonus_value;
                continue;
            }

            for pattern in SKILL_BONUS_PATTERNS.iter() {
                if let Some(captures) = pattern.captures(&description)
                    && let (Some(bonus_str), Some(skill_match)) = (captures.get(1), captures.get(2))
                    && let Ok(bonus_value) = bonus_str.as_str().parse::<i32>()
                {
                    let skill_name = Self::normalize_skill_name(skill_match.as_str());
                    *skill_bonuses.entry(skill_name).or_insert(0) += bonus_value;
                    break;
                }
            }
        }

        debug!("[feat_skill] Final skill_bonuses: {:?}", skill_bonuses);
        skill_bonuses
    }

    fn build_known_skill_names(game_data: &GameData) -> HashSet<String> {
        let mut names = HashSet::new();
        let Some(skills_table) = game_data.get_table("skills") else {
            return names;
        };

        for i in 0..skills_table.row_count() {
            let Some(skill_data) = skills_table.get_by_id(i as i32) else {
                continue;
            };

            let label = skill_data
                .get("label")
                .or_else(|| skill_data.get("Label"))
                .and_then(|opt| opt.as_deref())
                .unwrap_or("");

            if label.starts_with("****") || label.starts_with("DEL_") || label.contains("DELETED") {
                continue;
            }

            names.insert(Self::normalize_skill_name(label));
        }

        names
    }

    fn normalize_skill_name(name: &str) -> String {
        let mut normalized = name.trim().to_uppercase().replace([' ', '-', '_'], "");
        if normalized.ends_with("TRAPS") {
            normalized = normalized.trim_end_matches('S').to_string();
        }
        normalized
    }

    fn resolve_feat_description(
        feat_data: &ahash::AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> String {
        let field_mapper = FieldMapper::new();
        let desc_strref = field_mapper
            .get_field_value(feat_data, "description")
            .and_then(|s| s.parse::<i32>().ok());

        if let Some(strref) = desc_strref
            && let Some(desc) = game_data.get_string(strref)
            && !desc.is_empty()
        {
            return Self::strip_html_tags(&desc);
        }

        String::new()
    }

    fn parse_feat_description_sections(description: &str) -> FeatDescriptionSections<'_> {
        let description_lower = description.to_ascii_lowercase();
        let prerequisite_index = description_lower.find("prerequisite:");
        let effects_index = description_lower.find("effects:");

        let flavor_end = match (prerequisite_index, effects_index) {
            (Some(prereq), Some(effects)) => prereq.min(effects),
            (Some(prereq), None) => prereq,
            (None, Some(effects)) => effects,
            (None, None) => description.len(),
        };

        let prerequisites = prerequisite_index.map(|start| {
            let end = effects_index.unwrap_or(description.len());
            description[start..end].trim()
        });

        let effects = effects_index.map(|start| description[start..].trim());

        FeatDescriptionSections {
            flavor: description[..flavor_end].trim(),
            prerequisites,
            effects,
        }
    }

    fn strip_html_tags(text: &str) -> String {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"<[^>]+>").expect("Invalid HTML regex"));
        RE.replace_all(text, "").to_string()
    }

    // ========== FEAT AVAILABILITY CHECKS ==========

    pub fn get_feat_availability(
        &self,
        feat_id: FeatId,
        feat_type: FeatType,
        label: &str,
        game_data: &GameData,
    ) -> FeatAvailability {
        let mut result = FeatAvailability {
            available: true,
            reasons: vec![],
        };

        let class_check = self.check_class_availability(feat_id, game_data);
        if !class_check.available {
            result.available = false;
            result.reasons.extend(class_check.reasons);
        }

        let label_check = self.check_label_class_restriction(label, game_data);
        if !label_check.available {
            result.reasons.extend(label_check.reasons);
        }

        let ability_check = self.check_ability_requirement(label, game_data);
        if !ability_check.available {
            result.reasons.extend(ability_check.reasons);
        }

        let level_check = self.check_first_level_only(feat_type);
        if !level_check.available {
            result.reasons.extend(level_check.reasons);
        }

        result
    }

    fn check_class_availability(&self, feat_id: FeatId, game_data: &GameData) -> FeatAvailability {
        let mut result = FeatAvailability {
            available: true,
            reasons: vec![],
        };

        let Some(feats_table) = game_data.get_table("feat") else {
            return result;
        };
        let Some(feat_data) = feats_table.get_by_id(feat_id.0) else {
            return result;
        };

        let all_classes = feat_data
            .get("ALLCLASSESCANUSE")
            .or_else(|| feat_data.get("AllClassesCanUse"))
            .or_else(|| feat_data.get("allclassescanuse"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .unwrap_or(1);

        if all_classes == 0 {
            let class_entries = self.class_entries();
            let mut found_in_class = false;

            for entry in &class_entries {
                let class_feat_table = self.load_class_feat_table(entry.class_id, game_data);
                if class_feat_table.contains_key(&feat_id.0) {
                    found_in_class = true;
                    break;
                }
            }

            if !found_in_class {
                result.available = false;
                result
                    .reasons
                    .push("Not available for your classes".to_string());
            }
        }

        result
    }

    fn check_label_class_restriction(&self, label: &str, game_data: &GameData) -> FeatAvailability {
        static CLASS_KEYWORDS: &[(&str, &[&str])] = &[
            ("Bard", &["bard song", "bardic", "extra bard"]),
            (
                "Paladin",
                &["smite", "divine grace", "lay on hands", "aura of"],
            ),
            ("Barbarian", &["rage", "tireless rage", "greater rage"]),
            (
                "Rogue",
                &["sneak attack", "crippling strike", "opportunist"],
            ),
            (
                "Monk",
                &[
                    "flurry",
                    "stunning fist",
                    "wholeness of body",
                    "quivering palm",
                ],
            ),
            ("Cleric", &["turn undead", "divine might", "divine shield"]),
            ("Druid", &["wild shape", "wildshape"]),
            ("Ranger", &["favored enemy", "woodland stride"]),
            ("Sorcerer", &["sorcerer"]),
            ("Wizard", &["wizard"]),
        ];

        let mut result = FeatAvailability {
            available: true,
            reasons: vec![],
        };

        let label_lower = label.to_lowercase();
        let class_entries = self.class_entries();

        for (class_name, keywords) in CLASS_KEYWORDS {
            for keyword in *keywords {
                if label_lower.contains(keyword) {
                    let has_class = class_entries.iter().any(|entry| {
                        self.get_class_name(entry.class_id, game_data)
                            .to_lowercase()
                            .contains(&class_name.to_lowercase())
                    });

                    if !has_class {
                        result.reasons.push(format!("Requires {class_name} class"));
                        return result;
                    }
                }
            }
        }

        result
    }

    fn check_ability_requirement(&self, label: &str, game_data: &GameData) -> FeatAvailability {
        static ABILITY_FEATS: &[(&str, &[i32])] = &[
            ("smiting", &[6]),
            ("rage", &[2]),
            ("sneak_attack", &[9]),
            ("turn_undead", &[3, 6]),
            ("wildshape", &[4]),
            ("wild_shape", &[4]),
            ("ki_strike", &[5]),
            ("lay_on_hands", &[6]),
        ];

        let mut result = FeatAvailability {
            available: true,
            reasons: vec![],
        };

        let label_normalized = label.to_lowercase().replace(' ', "_");
        let class_entries = self.class_entries();
        let class_ids: Vec<i32> = class_entries.iter().map(|e| e.class_id.0).collect();

        for (keyword, required_class_ids) in ABILITY_FEATS {
            if label_normalized.contains(keyword) {
                let has_required_class = required_class_ids
                    .iter()
                    .any(|req_id| class_ids.contains(req_id));

                if !has_required_class {
                    let class_names: Vec<String> = required_class_ids
                        .iter()
                        .filter_map(|id| {
                            let name = self.get_class_name(ClassId(*id), game_data);
                            if name.starts_with("Class") {
                                None
                            } else {
                                Some(name)
                            }
                        })
                        .collect();

                    if !class_names.is_empty() {
                        result.reasons.push(format!(
                            "Requires class ability from: {}",
                            class_names.join(" or ")
                        ));
                    }
                    return result;
                }
            }
        }

        result
    }

    fn check_first_level_only(&self, feat_type: FeatType) -> FeatAvailability {
        let mut result = FeatAvailability {
            available: true,
            reasons: vec![],
        };

        if feat_type.contains(FeatType::BACKGROUND) || feat_type.contains(FeatType::HISTORY) {
            let total_level = self.total_level();
            if total_level > 1 {
                result
                    .reasons
                    .push("Only selectable at character creation".to_string());
            }
        }

        result
    }

    // ========== FEAT INFO AND METADATA ==========

    pub fn get_feat_info(&self, feat_id: FeatId, game_data: &GameData) -> Option<FeatInfo> {
        let feats_table = game_data.get_table("feat")?;
        let feat_data = feats_table.get_by_id(feat_id.0)?;

        let label = feat_data
            .get("label")
            .and_then(|s| s.as_ref())
            .map_or_else(|| format!("feat_{}", feat_id.0), |s| s.clone());
        let name = Self::resolve_feat_name_from_data(&feat_data, game_data);
        let description = Self::resolve_feat_description(&feat_data, game_data);
        let icon = feat_data
            .get("icon")
            .and_then(|s| s.as_ref())
            .map_or("", std::string::String::as_str)
            .to_string();

        let mut feat_type = Self::parse_feat_type(&feat_data, &description);

        let is_domain = self.is_domain_feat(feat_id, game_data);
        if is_domain {
            feat_type = FeatType(feat_type.0 | FeatType::DOMAIN.0);
        }
        let category = FeatCategory::from_feat_type(feat_type, is_domain);

        let is_protected = self.is_feat_protected(feat_id, game_data);
        let is_custom = feat_data
            .get("custom")
            .and_then(|s| s.as_ref())
            .is_some_and(|s| s == "1" || s.to_lowercase() == "true");
        let has_feat = self.has_feat(feat_id);

        let prerequisites = self.build_feat_prerequisites(&feat_data, game_data);
        let availability = self.get_feat_availability(feat_id, feat_type, &label, game_data);
        let prereq_result = self.validate_feat_prerequisites(feat_id, game_data);

        Some(FeatInfo {
            id: feat_id,
            label,
            name,
            description,
            icon,
            feat_type,
            category,
            is_protected,
            is_custom,
            has_feat,
            can_take: prereq_result.can_take,
            missing_requirements: prereq_result.missing_requirements,
            prerequisites,
            availability,
        })
    }

    /// Fast path for listing: skips prerequisites, availability checks, and
    /// uses pre-computed HashSets for domain/epithet lookups instead of
    /// iterating the domains table per feat.
    pub fn get_feat_info_display(
        &self,
        feat_id: FeatId,
        game_data: &GameData,
        domain_feats: &HashSet<i32>,
        epithet_feats: &HashSet<i32>,
        feat_sources: &HashMap<FeatId, FeatSource>,
        owned_feats: &HashSet<FeatId>,
    ) -> Option<FeatInfo> {
        let feats_table = game_data.get_table("feat")?;
        let feat_data = feats_table.get_by_id(feat_id.0)?;

        let label = feat_data
            .get("label")
            .and_then(|s| s.as_ref())
            .map_or_else(|| format!("feat_{}", feat_id.0), |s| s.clone());
        let name = Self::resolve_feat_name_from_data(&feat_data, game_data);
        let description = Self::resolve_feat_description(&feat_data, game_data);
        let icon = feat_data
            .get("icon")
            .and_then(|s| s.as_ref())
            .map_or("", std::string::String::as_str)
            .to_string();

        let mut feat_type = Self::parse_feat_type(&feat_data, &description);

        let is_domain = domain_feats.contains(&feat_id.0);
        if is_domain {
            feat_type = FeatType(feat_type.0 | FeatType::DOMAIN.0);
        }
        let category = FeatCategory::from_feat_type(feat_type, is_domain);

        let is_protected = feat_sources
            .get(&feat_id)
            .is_some_and(|s| matches!(s, FeatSource::Race | FeatSource::Background))
            || epithet_feats.contains(&feat_id.0);

        let is_custom = feat_data
            .get("custom")
            .and_then(|s| s.as_ref())
            .is_some_and(|s| s == "1" || s.to_lowercase() == "true");
        let has_feat = owned_feats.contains(&feat_id);

        Some(FeatInfo {
            id: feat_id,
            label,
            name,
            description,
            icon,
            feat_type,
            category,
            is_protected,
            is_custom,
            has_feat,
            can_take: true,
            missing_requirements: vec![],
            prerequisites: FeatPrerequisites::default(),
            availability: FeatAvailability {
                available: true,
                reasons: vec![],
            },
        })
    }

    pub(crate) fn parse_feat_type(
        feat_data: &ahash::AHashMap<String, Option<String>>,
        description: &str,
    ) -> FeatType {
        if let Some(type_str) = feat_data
            .get("TOOLCATEGORIES")
            .or_else(|| feat_data.get("ToolsCategories"))
            .or_else(|| feat_data.get("toolscategories"))
            .and_then(|s| s.as_ref())
        {
            return FeatType::from_string(type_str);
        }

        if let Some(type_str) = feat_data
            .get("FeatCategory")
            .or_else(|| feat_data.get("FEATCATEGORY"))
            .or_else(|| feat_data.get("featcategory"))
            .and_then(|s| s.as_ref())
        {
            return FeatType::from_string(type_str);
        }

        static TYPE_OF_FEAT_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"Type of Feat:\s*(\w+)").expect("Invalid regex"));

        if let Some(caps) = TYPE_OF_FEAT_RE.captures(description)
            && let Some(type_match) = caps.get(1)
        {
            return FeatType::from_string(type_match.as_str());
        }

        FeatType::GENERAL
    }

    fn build_feat_prerequisites(
        &self,
        feat_data: &ahash::AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> FeatPrerequisites {
        let mut prereqs = FeatPrerequisites::default();

        let ability_fields = [
            ("MINSTR", "minstr", "Strength"),
            ("MINDEX", "mindex", "Dexterity"),
            ("MINCON", "mincon", "Constitution"),
            ("MININT", "minint", "Intelligence"),
            ("MINWIS", "minwis", "Wisdom"),
            ("MINCHA", "mincha", "Charisma"),
        ];

        let ability_scores = [
            self.get_i32("Str").unwrap_or(10),
            self.get_i32("Dex").unwrap_or(10),
            self.get_i32("Con").unwrap_or(10),
            self.get_i32("Int").unwrap_or(10),
            self.get_i32("Wis").unwrap_or(10),
            self.get_i32("Cha").unwrap_or(10),
        ];

        for (i, (upper_field, lower_field, ability_name)) in ability_fields.iter().enumerate() {
            let min_val = feat_data
                .get(*upper_field)
                .or_else(|| feat_data.get(*lower_field))
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&v| v > 0);

            if let Some(required) = min_val {
                let current = ability_scores[i];
                prereqs.abilities.push(PrereqAbility {
                    ability: (*ability_name).to_string(),
                    required,
                    current,
                    met: current >= required,
                });
            }
        }

        for prereq_field in ["PREREQFEAT1", "PREREQFEAT2", "prereq_feat1", "prereq_feat2"] {
            let prereq_id = feat_data
                .get(prereq_field)
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&id| id >= 0);

            if let Some(prereq_id) = prereq_id {
                let feat_id = FeatId(prereq_id);
                let name = self.get_feat_name(feat_id, game_data);
                let met = self.has_feat(feat_id);
                prereqs.feats.push(PrereqFeat {
                    id: feat_id,
                    name,
                    met,
                });
            }
        }

        let bab_val = feat_data
            .get("MINATTACKBONUS")
            .or_else(|| feat_data.get("prereq_bab"))
            .or_else(|| feat_data.get("minattackbonus"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&v| v > 0);

        if let Some(required) = bab_val {
            let current = self.calculate_bab(game_data);
            prereqs.bab = Some(PrereqValue {
                required,
                current,
                met: current >= required,
            });
        }

        let min_level = feat_data
            .get("MinLevel")
            .or_else(|| feat_data.get("MINLEVEL"))
            .or_else(|| feat_data.get("min_level"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&v| v > 0);

        if let Some(required) = min_level {
            let current = self.total_level();
            prereqs.level = Some(PrereqValue {
                required,
                current,
                met: current >= required,
            });
        }

        let min_caster_level = feat_data
            .get("MinCasterLevel")
            .or_else(|| feat_data.get("MINCASTERLEVEL"))
            .or_else(|| feat_data.get("min_caster_level"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&v| v > 0);

        if let Some(required) = min_caster_level {
            let current = self.get_highest_caster_level(game_data);
            prereqs.caster_level = Some(PrereqValue {
                required,
                current,
                met: current >= required,
            });
        }

        let min_spell_level = feat_data
            .get("MinSpellLevel")
            .or_else(|| feat_data.get("MINSPELLLVL"))
            .or_else(|| feat_data.get("min_spell_level"))
            .and_then(|s| s.as_ref()?.parse::<i32>().ok())
            .filter(|&v| v > 0);

        if let Some(required) = min_spell_level {
            let current = self.get_max_spell_level(game_data);
            prereqs.spell_level = Some(PrereqValue {
                required,
                current,
                met: current >= required,
            });
        }

        prereqs
    }

    fn get_highest_caster_level(&self, game_data: &GameData) -> i32 {
        self.class_entries()
            .iter()
            .map(|entry| self.get_caster_level(entry.class_id, game_data))
            .max()
            .unwrap_or(0)
    }

    fn get_max_spell_level(&self, game_data: &GameData) -> i32 {
        let caster_level = self.get_highest_caster_level(game_data);
        if caster_level == 0 {
            return 0;
        }
        ((caster_level + 1) / 2).min(9)
    }

    pub fn get_feat_summary(&self, game_data: &GameData) -> FeatSummary {
        let feat_entries = self.feat_entries();
        let mut summary = FeatSummary {
            total: feat_entries.len() as i32,
            ..Default::default()
        };

        for entry in &feat_entries {
            let Some(feat_info) = self.get_feat_info(entry.feat_id, game_data) else {
                continue;
            };

            if feat_info.is_protected {
                summary.protected_feats.push(feat_info);
            } else if feat_info.is_custom {
                summary.custom_feats.push(feat_info);
            } else {
                match feat_info.category {
                    FeatCategory::Domain => summary.domain_feats.push(feat_info),
                    FeatCategory::Background | FeatCategory::History | FeatCategory::Heritage => {
                        summary.background_feats.push(feat_info);
                    }
                    FeatCategory::Class => summary.class_feats.push(feat_info),
                    _ => summary.general_feats.push(feat_info),
                }
            }
        }

        summary
    }

    pub fn get_feats_state(&self, game_data: &GameData) -> FeatsState {
        let domains = self
            .get_available_domains(game_data)
            .into_iter()
            .filter(|d| d.has_domain)
            .collect();

        FeatsState {
            summary: self.get_feat_summary(game_data),
            feat_slots: self.get_feat_slots(game_data),
            domains,
        }
    }

    pub fn get_available_domains(&self, game_data: &GameData) -> Vec<DomainInfo> {
        let Some(domains_table) = game_data.get_table("domains") else {
            debug!("get_available_domains: No domains table found");
            return Vec::new();
        };

        let character_domains = self.get_character_domains();
        debug!(
            "get_available_domains: character_domains = {:?}",
            character_domains
        );
        let mut available = Vec::new();
        let row_count = domains_table.row_count();
        debug!(
            "get_available_domains: domains_table row_count = {}",
            row_count
        );

        for domain_id in 0..row_count {
            let domain_id = DomainId(domain_id as i32);

            let Some(domain_data) = domains_table.get_by_id(domain_id.0) else {
                if domain_id.0 == 25 || domain_id.0 == 18 {
                    debug!(
                        "get_available_domains: domain_id {} - no domain_data found",
                        domain_id.0
                    );
                }
                continue;
            };

            let label = domain_data
                .get("Label")
                .and_then(|s| s.as_ref())
                .map_or("", std::string::String::as_str);

            // Skip deleted/invalid domains but allow empty labels
            if label.starts_with("****") || label.starts_with("DEL_") {
                continue;
            }

            let name = Self::resolve_domain_name(&domain_data, game_data);
            let description = Self::resolve_domain_description(&domain_data, game_data);

            let granted_feat = domain_data
                .get("GrantedFeat")
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&id| id >= 0)
                .map(FeatId);

            let castable_feat = domain_data
                .get("CastableFeat")
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&id| id >= 0)
                .map(FeatId);

            let epithet_feat = domain_data
                .get("EpithetFeat")
                .and_then(|s| s.as_ref()?.parse::<i32>().ok())
                .filter(|&id| id >= 0)
                .map(FeatId);

            let has_domain = character_domains.contains(&domain_id);

            if has_domain {
                debug!(
                    "get_available_domains: Domain {} ({}) has_domain=true",
                    domain_id.0, name
                );
            }

            available.push(DomainInfo {
                id: domain_id,
                name,
                description,
                granted_feat,
                castable_feat,
                epithet_feat,
                has_domain,
            });
        }

        let with_domain: Vec<_> = available.iter().filter(|d| d.has_domain).collect();
        debug!(
            "get_available_domains: Total available={}, with has_domain=true: {}",
            available.len(),
            with_domain.len()
        );

        available
    }

    pub fn swap_feat(
        &mut self,
        old_feat_id: FeatId,
        new_feat_id: FeatId,
    ) -> Result<(FeatId, FeatId), CharacterError> {
        self.remove_feat(old_feat_id)?;
        self.add_feat(new_feat_id)?;
        Ok((old_feat_id, new_feat_id))
    }

    fn resolve_feat_name_from_data(
        feat_data: &ahash::AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> String {
        let field_mapper = FieldMapper::new();

        // Try feat_name_strref first (handles FEAT column), then name
        let name_strref = field_mapper
            .get_field_value(feat_data, "feat_name_strref")
            .or_else(|| field_mapper.get_field_value(feat_data, "name"))
            .and_then(|s| s.parse::<i32>().ok());

        if let Some(strref) = name_strref
            && let Some(name) = game_data.get_string(strref)
            && !name.is_empty()
        {
            return Self::strip_html_tags(&name);
        }

        // Fallback to label
        field_mapper
            .get_field_value(feat_data, "label")
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn resolve_domain_name(
        domain_data: &ahash::AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> String {
        let name_strref = domain_data
            .get("Name")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok());

        if let Some(strref) = name_strref
            && let Some(name) = game_data.get_string(strref)
            && !name.is_empty()
        {
            return name;
        }

        domain_data
            .get("Label")
            .and_then(|s| s.as_ref())
            .map_or("Unknown", std::string::String::as_str)
            .to_string()
    }

    fn resolve_domain_description(
        domain_data: &ahash::AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> String {
        let desc_strref = domain_data
            .get("Description")
            .and_then(|s| s.as_ref()?.parse::<i32>().ok());

        if let Some(strref) = desc_strref
            && let Some(desc) = game_data.get_string(strref)
            && !desc.is_empty()
        {
            return desc;
        }

        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashMap;
    use indexmap::IndexMap;
    use std::sync::Arc;

    use crate::loaders::{GameData, LoadedTable};
    use crate::parsers::tda::TDAParser;
    use crate::parsers::tlk::TLKParser;

    fn create_test_character_with_feats() -> Character {
        let mut fields = IndexMap::new();

        let mut feat1 = IndexMap::new();
        feat1.insert("Feat".to_string(), GffValue::Word(10));

        let mut feat2 = IndexMap::new();
        feat2.insert("Feat".to_string(), GffValue::Word(20));

        let mut feat3 = IndexMap::new();
        feat3.insert("Feat".to_string(), GffValue::Word(30));

        fields.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![feat1, feat2, feat3]),
        );

        Character::from_gff(fields)
    }

    fn create_empty_character() -> Character {
        Character::from_gff(IndexMap::new())
    }

    fn create_mock_game_data() -> GameData {
        GameData::new(Arc::new(std::sync::RwLock::new(TLKParser::default())))
    }

    fn create_loaded_table(
        name: &str,
        columns: &[&str],
        rows: Vec<AHashMap<String, Option<String>>>,
    ) -> LoadedTable {
        let mut parser = TDAParser::new();
        for column in columns {
            parser.add_column(column);
        }
        for row in rows {
            parser.add_row(row);
        }
        LoadedTable::new(name.to_string(), Arc::new(parser))
    }

    fn create_slot_test_character() -> Character {
        let mut fields = IndexMap::new();

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(2));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let make_feat_entry = |feat_id: u16| {
            let mut feat = IndexMap::new();
            feat.insert("Feat".to_string(), GffValue::Word(feat_id));
            feat
        };

        let mut level_one = IndexMap::new();
        level_one.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        level_one.insert("LvlStatHitDie".to_string(), GffValue::Byte(10));
        level_one.insert("SkillPoints".to_string(), GffValue::Short(0));
        level_one.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        level_one.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![
                make_feat_entry(2),
                make_feat_entry(3),
                make_feat_entry(1),
            ]),
        );

        let mut level_two = IndexMap::new();
        level_two.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        level_two.insert("LvlStatHitDie".to_string(), GffValue::Byte(8));
        level_two.insert("SkillPoints".to_string(), GffValue::Short(0));
        level_two.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        level_two.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![make_feat_entry(4)]),
        );

        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![level_one, level_two]),
        );
        fields.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![
                make_feat_entry(1),
                make_feat_entry(2),
                make_feat_entry(3),
                make_feat_entry(4),
            ]),
        );

        Character::from_gff(fields)
    }

    fn create_slot_test_character_with_story_history() -> Character {
        let mut fields = IndexMap::new();

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(2));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let make_feat_entry = |feat_id: u16| {
            let mut feat = IndexMap::new();
            feat.insert("Feat".to_string(), GffValue::Word(feat_id));
            feat
        };

        let mut level_one = IndexMap::new();
        level_one.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        level_one.insert("LvlStatHitDie".to_string(), GffValue::Byte(10));
        level_one.insert("SkillPoints".to_string(), GffValue::Short(0));
        level_one.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        level_one.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![
                make_feat_entry(2),
                make_feat_entry(5),
                make_feat_entry(6),
                make_feat_entry(1),
            ]),
        );

        let mut level_two = IndexMap::new();
        level_two.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        level_two.insert("LvlStatHitDie".to_string(), GffValue::Byte(8));
        level_two.insert("SkillPoints".to_string(), GffValue::Short(0));
        level_two.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        level_two.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![make_feat_entry(4)]),
        );

        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![level_one, level_two]),
        );
        fields.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![
                make_feat_entry(1),
                make_feat_entry(2),
                make_feat_entry(4),
                make_feat_entry(5),
                make_feat_entry(6),
            ]),
        );

        Character::from_gff(fields)
    }

    fn create_slot_test_game_data() -> GameData {
        let mut game_data = create_mock_game_data();

        let mut class_row = AHashMap::new();
        class_row.insert("FeatsTable".to_string(), Some("fighterfeat".to_string()));
        class_row.insert(
            "BonusFeatsTable".to_string(),
            Some("fighterbonus".to_string()),
        );
        game_data.tables.insert(
            "classes".to_string(),
            create_loaded_table(
                "classes",
                &["FeatsTable", "BonusFeatsTable"],
                vec![class_row],
            ),
        );

        let mut class_feat_auto = AHashMap::new();
        class_feat_auto.insert("FeatIndex".to_string(), Some("2".to_string()));
        class_feat_auto.insert("List".to_string(), Some("3".to_string()));
        let mut class_feat_selectable = AHashMap::new();
        class_feat_selectable.insert("FeatIndex".to_string(), Some("4".to_string()));
        class_feat_selectable.insert("List".to_string(), Some("1".to_string()));
        game_data.tables.insert(
            "fighterfeat".to_string(),
            create_loaded_table(
                "fighterfeat",
                &["FeatIndex", "List"],
                vec![class_feat_auto, class_feat_selectable],
            ),
        );

        let mut level_one_bonus = AHashMap::new();
        level_one_bonus.insert("Bonus".to_string(), Some("0".to_string()));
        let mut level_two_bonus = AHashMap::new();
        level_two_bonus.insert("Bonus".to_string(), Some("1".to_string()));
        game_data.tables.insert(
            "fighterbonus".to_string(),
            create_loaded_table(
                "fighterbonus",
                &["Bonus"],
                vec![level_one_bonus, level_two_bonus],
            ),
        );

        let make_feat_row = |feat_type: &str, label: &str| {
            let mut row = AHashMap::new();
            row.insert("FEAT".to_string(), Some(feat_type.to_string()));
            row.insert("label".to_string(), Some(label.to_string()));
            row
        };
        game_data.tables.insert(
            "feat".to_string(),
            create_loaded_table(
                "feat",
                &["FEAT", "label"],
                vec![
                    make_feat_row("GENERAL", "AUTO_0"),
                    make_feat_row("GENERAL", "POWER_ATTACK"),
                    make_feat_row("GENERAL", "AUTO_CLASS_FEAT"),
                    make_feat_row("BACKGROUND", "BACKGROUND_PICK"),
                    make_feat_row("GENERAL", "WEAPON_FOCUS_LONGSWORD"),
                    make_feat_row("GENERAL", "FEAT_EPITHET_MERCHANTS_FRIEND"),
                    make_feat_row("GENERAL", "FEAT_EPITHET_BLESSED_OF_WAUKEEN"),
                ],
            ),
        );

        game_data
    }

    #[test]
    fn test_normalize_level_one_feat_history_for_save_rebuilds_missing_residual_feats() {
        let mut character = create_empty_character();

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(4));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(3));
        character.set_list("ClassList", vec![class_entry]);

        let make_feat = |feat_id: u16| {
            let mut feat = IndexMap::new();
            feat.insert("Feat".to_string(), GffValue::Word(feat_id));
            feat
        };

        character.set_list(
            "FeatList",
            vec![
                make_feat(2),
                make_feat(28),
                make_feat(106),
                make_feat(1701),
                make_feat(6),
                make_feat(40),
            ],
        );

        let mut level_one = IndexMap::new();
        level_one.insert("LvlStatClass".to_string(), GffValue::Byte(4));
        level_one.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![make_feat(28), make_feat(106)]),
        );
        let mut level_two = IndexMap::new();
        level_two.insert("LvlStatClass".to_string(), GffValue::Byte(4));
        level_two.insert("FeatList".to_string(), GffValue::ListOwned(vec![make_feat(6)]));
        let mut level_three = IndexMap::new();
        level_three.insert("LvlStatClass".to_string(), GffValue::Byte(4));
        level_three.insert("FeatList".to_string(), GffValue::ListOwned(vec![make_feat(40)]));

        character.set_list("LvlStatList", vec![level_one, level_two, level_three]);

        character.normalize_level_one_feat_history_for_save();

        let history = character.get_list_owned("LvlStatList").unwrap();
        let level_one_feats: Vec<i32> = history[0]
            .get("FeatList")
            .and_then(|value| match value {
                GffValue::ListOwned(list) => Some(
                    list.iter()
                        .filter_map(|entry| entry.get("Feat").and_then(gff_value_to_i32))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap();

        assert_eq!(level_one_feats, vec![2, 28, 106, 1701]);
    }

    #[test]
    fn test_normalize_level_one_feat_history_for_save_sets_single_level_history_to_top_level() {
        let mut character = create_empty_character();

        let make_feat = |feat_id: u16| {
            let mut feat = IndexMap::new();
            feat.insert("Feat".to_string(), GffValue::Word(feat_id));
            feat
        };

        character.set_list(
            "FeatList",
            vec![make_feat(2), make_feat(28), make_feat(106), make_feat(1717)],
        );

        let mut level_one = IndexMap::new();
        level_one.insert("LvlStatClass".to_string(), GffValue::Byte(4));
        level_one.insert("FeatList".to_string(), GffValue::ListOwned(vec![make_feat(28)]));
        character.set_list("LvlStatList", vec![level_one]);

        character.normalize_level_one_feat_history_for_save();

        let history = character.get_list_owned("LvlStatList").unwrap();
        let level_one_feats: Vec<i32> = history[0]
            .get("FeatList")
            .and_then(|value| match value {
                GffValue::ListOwned(list) => Some(
                    list.iter()
                        .filter_map(|entry| entry.get("Feat").and_then(gff_value_to_i32))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap();

        assert_eq!(level_one_feats, vec![2, 28, 106, 1717]);
    }

    #[test]
    fn test_feat_ids() {
        let character = create_test_character_with_feats();
        let feat_ids = character.feat_ids();

        assert_eq!(feat_ids.len(), 3);
        assert_eq!(feat_ids[0].0, 10);
        assert_eq!(feat_ids[1].0, 20);
        assert_eq!(feat_ids[2].0, 30);
    }

    #[test]
    fn test_feat_ids_empty() {
        let character = create_empty_character();
        let feat_ids = character.feat_ids();

        assert_eq!(feat_ids.len(), 0);
    }

    #[test]
    fn test_slot_chosen_feat_ids_only_return_removable_slot_picks() {
        let character = create_slot_test_character();
        let game_data = create_slot_test_game_data();

        let chosen = character.get_slot_chosen_feat_ids(&game_data);
        let slots = character.get_feat_slots(&game_data);

        assert_eq!(chosen, vec![FeatId(1), FeatId(4)]);
        assert_eq!(slots.filled_slots, 2);
        assert_eq!(chosen.len() as i32, slots.filled_slots);
    }

    #[test]
    fn test_slot_chosen_feat_ids_ignore_story_feats_in_level_history() {
        let character = create_slot_test_character_with_story_history();
        let game_data = create_slot_test_game_data();

        let chosen = character.get_slot_chosen_feat_ids(&game_data);
        let slots = character.get_feat_slots(&game_data);

        assert_eq!(chosen, vec![FeatId(1), FeatId(4)]);
        assert_eq!(slots.filled_slots, 2);
        assert_eq!(slots.open_slots, 0);
    }

    #[test]
    fn test_remove_feat_also_updates_level_history_slots() {
        let mut character = create_slot_test_character();
        let game_data = create_slot_test_game_data();

        character.remove_feat(FeatId(1)).unwrap();

        let chosen = character.get_slot_chosen_feat_ids(&game_data);
        let slots = character.get_feat_slots(&game_data);

        assert_eq!(chosen, vec![FeatId(4)]);
        assert_eq!(slots.filled_slots, 1);
        assert_eq!(slots.open_slots, 1);
        assert!(!character.has_feat(FeatId(1)));
    }

    #[test]
    fn test_has_feat() {
        let character = create_test_character_with_feats();

        assert!(character.has_feat(FeatId(10)));
        assert!(character.has_feat(FeatId(20)));
        assert!(character.has_feat(FeatId(30)));
        assert!(!character.has_feat(FeatId(99)));
    }

    #[test]
    fn test_has_feat_empty() {
        let character = create_empty_character();

        assert!(!character.has_feat(FeatId(10)));
    }

    #[test]
    fn test_feat_count() {
        let character = create_test_character_with_feats();
        assert_eq!(character.feat_count(), 3);

        let empty = create_empty_character();
        assert_eq!(empty.feat_count(), 0);
    }

    #[test]
    fn test_add_feat() {
        let mut character = create_test_character_with_feats();

        let result = character.add_feat(FeatId(40));
        assert!(result.is_ok());

        assert_eq!(character.feat_count(), 4);
        assert!(character.has_feat(FeatId(40)));
        assert!(character.is_modified());
    }

    #[test]
    fn test_add_feat_to_empty() {
        let mut character = create_empty_character();

        let result = character.add_feat(FeatId(1));
        assert!(result.is_ok());

        assert_eq!(character.feat_count(), 1);
        assert!(character.has_feat(FeatId(1)));
    }

    #[test]
    fn test_add_feat_duplicate() {
        let mut character = create_test_character_with_feats();

        let result = character.add_feat(FeatId(10));
        assert!(result.is_err());

        match result.unwrap_err() {
            CharacterError::FeatAlreadyExists(id) => assert_eq!(id, 10),
            _ => panic!("Expected FeatAlreadyExists error"),
        }

        assert_eq!(character.feat_count(), 3);
    }

    #[test]
    fn test_remove_feat() {
        let mut character = create_test_character_with_feats();

        let result = character.remove_feat(FeatId(20));
        assert!(result.is_ok());

        assert_eq!(character.feat_count(), 2);
        assert!(!character.has_feat(FeatId(20)));
        assert!(character.has_feat(FeatId(10)));
        assert!(character.has_feat(FeatId(30)));
        assert!(character.is_modified());
    }

    #[test]
    fn test_remove_feat_not_found() {
        let mut character = create_test_character_with_feats();

        let result = character.remove_feat(FeatId(99));
        assert!(result.is_err());

        match result.unwrap_err() {
            CharacterError::FeatNotFound(id) => assert_eq!(id, 99),
            _ => panic!("Expected FeatNotFound error"),
        }

        assert_eq!(character.feat_count(), 3);
    }

    #[test]
    fn test_remove_feat_from_empty() {
        let mut character = create_empty_character();

        let result = character.remove_feat(FeatId(1));
        assert!(result.is_err());

        match result.unwrap_err() {
            CharacterError::FeatNotFound(id) => assert_eq!(id, 1),
            _ => panic!("Expected FeatNotFound error"),
        }
    }

    #[test]
    fn test_add_and_remove_cycle() {
        let mut character = create_test_character_with_feats();

        let result = character.add_feat(FeatId(50));
        assert!(result.is_ok());
        assert_eq!(character.feat_count(), 4);
        assert!(character.has_feat(FeatId(50)));

        let result = character.remove_feat(FeatId(50));
        assert!(result.is_ok());
        assert_eq!(character.feat_count(), 3);
        assert!(!character.has_feat(FeatId(50)));

        let result = character.add_feat(FeatId(50));
        assert!(result.is_ok());
        assert_eq!(character.feat_count(), 4);
        assert!(character.has_feat(FeatId(50)));
    }

    #[test]
    fn test_multiple_add_feats() {
        let mut character = create_empty_character();

        for id in 1..=10 {
            let result = character.add_feat(FeatId(id));
            assert!(result.is_ok());
        }

        assert_eq!(character.feat_count(), 10);

        for id in 1..=10 {
            assert!(character.has_feat(FeatId(id)));
        }
    }

    #[test]
    fn test_remove_all_feats() {
        let mut character = create_test_character_with_feats();

        let feat_ids: Vec<FeatId> = character.feat_ids();

        for feat_id in feat_ids {
            let result = character.remove_feat(feat_id);
            assert!(result.is_ok());
        }

        assert_eq!(character.feat_count(), 0);
    }

    #[test]
    fn test_feat_progression_regex() {
        let pattern = &*FEAT_PROGRESSION_PATTERN;

        let captures = pattern.captures("Toughness_2");
        assert!(captures.is_some());
        let caps = captures.unwrap();
        assert_eq!(
            caps.get(1).unwrap().as_str().trim_end_matches('_'),
            "Toughness"
        );
        assert_eq!(caps.get(2).unwrap().as_str(), "2");

        let captures = pattern.captures("GreatFortitude3");
        assert!(captures.is_some());
        let caps = captures.unwrap();
        assert_eq!(
            caps.get(1).unwrap().as_str().trim_end_matches('_'),
            "GreatFortitude"
        );
        assert_eq!(caps.get(2).unwrap().as_str(), "3");

        let captures = pattern.captures("ResistEnergy 1");
        assert!(captures.is_some());
        let caps = captures.unwrap();
        assert_eq!(
            caps.get(1).unwrap().as_str().trim_end_matches(' '),
            "ResistEnergy"
        );
        assert_eq!(caps.get(2).unwrap().as_str(), "1");

        let captures = pattern.captures("PowerAttack");
        assert!(captures.is_none());
    }

    #[test]
    fn test_parse_feat_description_sections_splits_flavor_prereq_and_effects() {
        let description =
            "Flavor text here. Prerequisite: Dexterity 10+ Effects: +1 Listen, -1 Intimidate";

        let sections = Character::parse_feat_description_sections(description);

        assert_eq!(sections.flavor, "Flavor text here.");
        assert_eq!(sections.prerequisites, Some("Prerequisite: Dexterity 10+"));
        assert_eq!(sections.effects, Some("Effects: +1 Listen, -1 Intimidate"));
    }
}
