use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
use tracing::debug;

use super::{Character, CharacterError};
use crate::character::feats::FeatSource;
use crate::character::types::{AbilityIndex, AbilityModifiers, ClassId, FeatId, RaceId};
use crate::loaders::GameData;
use crate::parsers::gff::GffValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[repr(i32)]
pub enum SizeCategory {
    Fine = 0,
    Diminutive = 1,
    Tiny = 2,
    Small = 3,
    Medium = 4,
    Large = 5,
    Huge = 6,
    Gargantuan = 7,
    Colossal = 8,
}

impl SizeCategory {
    pub fn from_id(id: i32) -> Self {
        match id {
            0 => Self::Fine,
            1 => Self::Diminutive,
            2 => Self::Tiny,
            3 => Self::Small,
            4 => Self::Medium,
            5 => Self::Large,
            6 => Self::Huge,
            7 => Self::Gargantuan,
            8 => Self::Colossal,
            _ => Self::Medium,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Fine => "Fine",
            Self::Diminutive => "Diminutive",
            Self::Tiny => "Tiny",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Huge => "Huge",
            Self::Gargantuan => "Gargantuan",
            Self::Colossal => "Colossal",
        }
    }

    pub fn ac_modifier_default(&self) -> i32 {
        match self {
            Self::Fine => 8,
            Self::Diminutive => 4,
            Self::Tiny => 2,
            Self::Small => 1,
            Self::Medium => 0,
            Self::Large => -1,
            Self::Huge => -2,
            Self::Gargantuan => -4,
            Self::Colossal => -8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SubraceInfo {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub parent_race: RaceId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SubraceData {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub base_race: i32,
    pub ability_modifiers: AbilityModifiers,
    pub player_race: bool,
    pub favored_class: Option<i32>,
    pub feats_table: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbilityChange {
    pub attribute: String,
    pub old_value: i32,
    pub new_value: i32,
    pub modifier: i32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatChange {
    pub feat_id: i32,
    pub feat_name: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeChange {
    pub old_size: i32,
    pub new_size: i32,
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpeedChange {
    pub old_speed: i32,
    pub new_speed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RaceChangeResult {
    pub old_race_id: i32,
    pub old_race_name: String,
    pub old_subrace: Option<String>,
    pub new_race_id: i32,
    pub new_race_name: String,
    pub new_subrace: Option<String>,
    pub ability_changes: Vec<AbilityChange>,
    pub feats_removed: Vec<FeatChange>,
    pub feats_added: Vec<FeatChange>,
    pub size_change: Option<SizeChange>,
    pub speed_change: Option<SpeedChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RacialProperties {
    pub race_id: i32,
    pub race_name: String,
    pub subrace: Option<String>,
    pub size: i32,
    pub size_name: String,
    pub base_speed: i32,
    pub ability_modifiers: AbilityModifiers,
    pub racial_feats: Vec<i32>,
    pub favored_class: Option<i32>,
    pub available_subraces: Vec<SubraceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }
}

impl Character {
    pub fn race_id(&self) -> RaceId {
        RaceId(self.get_i32("Race").unwrap_or(0))
    }

    pub fn set_race(&mut self, race_id: RaceId) {
        self.set_byte("Race", race_id.0 as u8);
    }

    pub fn subrace_index(&self) -> Option<i32> {
        let val = self.get_i32("Subrace")?;
        if val > 0 { Some(val) } else { None }
    }

    pub fn subrace_string(&self) -> Option<String> {
        match self.get_string("Subrace") {
            Some(s) if !s.is_empty() && s != "0" => Some(s.to_string()),
            _ => None,
        }
    }

    pub fn subrace(&self) -> Option<String> {
        self.subrace_string()
    }

    pub fn subrace_name(&self, game_data: &GameData) -> Option<String> {
        if let Some(name) = self.subrace_string() {
            return Some(name);
        }
        if let Some(idx) = self.subrace_index() {
            return self.get_subrace_name_by_index(idx, game_data);
        }
        None
    }

    pub fn get_subrace_name_by_index(&self, index: i32, game_data: &GameData) -> Option<String> {
        let subrace_table = game_data.get_table("racialsubtypes")?;
        let row = subrace_table.get_by_id(index)?;

        if let Some(Some(strref_str)) = row.get("Name")
            && let Ok(strref) = strref_str.parse::<i32>()
            && let Some(name) = game_data.get_string(strref)
        {
            return Some(name);
        }

        row.get("Label").and_then(|v| v.clone())
    }

    pub fn set_subrace(&mut self, subrace: Option<String>) {
        match subrace {
            Some(name) if !name.is_empty() => {
                self.set_string("Subrace", name);
            }
            _ => {
                self.set_string("Subrace", String::new());
            }
        }
    }

    fn set_subrace_for_game_data(&mut self, subrace: Option<&str>, game_data: &GameData) {
        match subrace {
            Some(subrace_name) if !subrace_name.is_empty() => {
                if let Some(subrace_index) = self.resolve_subrace_row_index(subrace_name, game_data)
                {
                    self.write_subrace_index(subrace_index);
                } else {
                    self.set_string("Subrace", subrace_name.to_string());
                }
            }
            _ => self.clear_subrace_value(),
        }
    }

    fn resolve_subrace_row_index(&self, subrace_name: &str, game_data: &GameData) -> Option<i32> {
        self.get_subrace_data(subrace_name, game_data)
            .map(|subrace| subrace.id)
    }

    fn write_subrace_index(&mut self, index: i32) {
        match self.gff.get("Subrace") {
            Some(GffValue::Byte(_)) if (0..=u8::MAX as i32).contains(&index) => {
                self.set_byte("Subrace", index as u8);
            }
            Some(GffValue::Word(_)) if (0..=u16::MAX as i32).contains(&index) => {
                self.set_u16("Subrace", index as u16);
            }
            Some(GffValue::Dword(_)) if index >= 0 => {
                self.set_u32("Subrace", index as u32);
            }
            Some(GffValue::Short(_)) if (i16::MIN as i32..=i16::MAX as i32).contains(&index) => {
                self.set_i16("Subrace", index as i16);
            }
            Some(GffValue::Int(_)) => {
                self.set_i32("Subrace", index);
            }
            _ if (0..=u8::MAX as i32).contains(&index) => {
                self.set_byte("Subrace", index as u8);
            }
            _ if (0..=u16::MAX as i32).contains(&index) => {
                self.set_u16("Subrace", index as u16);
            }
            _ => {
                self.set_i32("Subrace", index);
            }
        }
    }

    fn clear_subrace_value(&mut self) {
        match self.gff.get("Subrace") {
            Some(GffValue::Byte(_)) => self.set_byte("Subrace", 0),
            Some(GffValue::Word(_)) => self.set_u16("Subrace", 0),
            Some(GffValue::Dword(_)) => self.set_u32("Subrace", 0),
            Some(GffValue::Short(_)) => self.set_i16("Subrace", 0),
            Some(GffValue::Int(_)) => self.set_i32("Subrace", 0),
            _ => self.set_string("Subrace", String::new()),
        }
    }

    pub fn creature_size(&self) -> i32 {
        self.get_i32("CreatureSize").unwrap_or(4)
    }

    pub fn set_creature_size(&mut self, size: i32) {
        self.set_i32("CreatureSize", size);
    }

    pub fn size_category(&self) -> SizeCategory {
        SizeCategory::from_id(self.creature_size())
    }

    pub fn race_name(&self, game_data: &GameData) -> String {
        if let Some(subrace_name) = self.subrace_name(game_data) {
            return subrace_name;
        }
        self.get_race_name_by_id(self.race_id().0, game_data)
    }

    pub fn get_race_name_by_id(&self, race_id: i32, game_data: &GameData) -> String {
        let Some(races) = game_data.get_table("racialtypes") else {
            return format!("Race {race_id}");
        };

        let Some(row) = races.get_by_id(race_id) else {
            return format!("Race {race_id}");
        };

        if let Some(Some(strref_str)) = row.get("Name")
            && let Ok(strref) = strref_str.parse::<i32>()
            && let Some(name) = game_data.get_string(strref)
        {
            return name;
        }

        if let Some(label) = row.get("Label").and_then(std::clone::Clone::clone) {
            return label;
        }

        format!("Race {race_id}")
    }

    pub fn available_subraces(&self, game_data: &GameData) -> Vec<SubraceInfo> {
        self.get_available_subraces_for_race(self.race_id().0, game_data)
    }

    pub fn get_available_subraces_for_race(
        &self,
        race_id: i32,
        game_data: &GameData,
    ) -> Vec<SubraceInfo> {
        let Some(subrace_table) = game_data.get_table("racialsubtypes") else {
            return Vec::new();
        };

        let mut subraces = Vec::new();

        for row_idx in 0..subrace_table.row_count() {
            let Ok(row_data) = subrace_table.parser.get_row_dict(row_idx) else {
                continue;
            };

            let base_race_id = row_data
                .get("BaseRace")
                .or_else(|| row_data.get("base_race"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(-1);

            if base_race_id != race_id {
                continue;
            }

            let player_race = row_data
                .get("PlayerRace")
                .or_else(|| row_data.get("player_race"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1);

            if player_race == 0 {
                continue;
            }

            let label = row_data
                .get("Label")
                .or_else(|| row_data.get("label"))
                .and_then(std::clone::Clone::clone)
                .unwrap_or_default();

            // Name column contains a TLK string reference — resolve it for display
            let name = row_data
                .get("Name")
                .or_else(|| row_data.get("name"))
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok())
                .and_then(|strref| game_data.get_string(strref))
                .unwrap_or_else(|| label.clone());

            if name.is_empty() && label.is_empty() {
                continue;
            }

            subraces.push(SubraceInfo {
                id: row_idx as i32,
                name,
                label,
                parent_race: RaceId(race_id),
            });
        }

        subraces
    }

    pub fn get_subrace_data(
        &self,
        subrace_name: &str,
        game_data: &GameData,
    ) -> Option<SubraceData> {
        if subrace_name.is_empty() {
            return None;
        }

        let subrace_table = game_data.get_table("racialsubtypes")?;
        let subrace_lower = subrace_name.to_lowercase();

        for row_idx in 0..subrace_table.row_count() {
            let Ok(row_data) = subrace_table.parser.get_row_dict(row_idx) else {
                continue;
            };

            let name = row_data
                .get("Name")
                .or_else(|| row_data.get("name"))
                .and_then(std::clone::Clone::clone)
                .unwrap_or_default();
            let resolved_name = name
                .parse::<i32>()
                .ok()
                .and_then(|strref| game_data.get_string(strref));

            let label = row_data
                .get("Label")
                .or_else(|| row_data.get("label"))
                .and_then(std::clone::Clone::clone)
                .unwrap_or_else(|| name.clone());

            let matches_resolved_name = resolved_name
                .as_ref()
                .is_some_and(|resolved| resolved.to_lowercase() == subrace_lower);

            if name.to_lowercase() != subrace_lower
                && label.to_lowercase() != subrace_lower
                && !matches_resolved_name
            {
                continue;
            }

            let base_race = row_data
                .get("BaseRace")
                .or_else(|| row_data.get("base_race"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(-1);

            let player_race = row_data
                .get("PlayerRace")
                .or_else(|| row_data.get("player_race"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1)
                == 1;

            let favored_class = row_data
                .get("FavoredClass")
                .or_else(|| row_data.get("favored_class"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .filter(|&v| v >= 0);

            let feats_table = row_data
                .get("FeatsTable")
                .or_else(|| row_data.get("feats_table"))
                .and_then(std::clone::Clone::clone)
                .filter(|s| !s.is_empty() && s != "****");

            let get_mod = |field: &str| -> i32 {
                row_data
                    .get(field)
                    .and_then(|v| v.as_ref())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0)
            };

            let ability_modifiers = AbilityModifiers::new(
                get_mod("StrAdjust"),
                get_mod("DexAdjust"),
                get_mod("ConAdjust"),
                get_mod("IntAdjust"),
                get_mod("WisAdjust"),
                get_mod("ChaAdjust"),
            );

            return Some(SubraceData {
                id: row_idx as i32,
                name: resolved_name.unwrap_or(name),
                label,
                base_race,
                ability_modifiers,
                player_race,
                favored_class,
                feats_table,
            });
        }

        None
    }

    pub fn get_racial_ability_modifiers_for_race(
        &self,
        race_id: i32,
        game_data: &GameData,
    ) -> AbilityModifiers {
        let Some(racialtypes_table) = game_data.get_table("racialtypes") else {
            return AbilityModifiers::default();
        };

        let row_index = race_id as usize;
        if row_index >= racialtypes_table.row_count() {
            return AbilityModifiers::default();
        }

        let Ok(row_data) = racialtypes_table.get_row(row_index) else {
            return AbilityModifiers::default();
        };

        let get_modifier = |field_name: &str| -> i32 {
            row_data
                .get(field_name)
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0)
        };

        AbilityModifiers::new(
            get_modifier("StrAdjust"),
            get_modifier("DexAdjust"),
            get_modifier("ConAdjust"),
            get_modifier("IntAdjust"),
            get_modifier("WisAdjust"),
            get_modifier("ChaAdjust"),
        )
    }

    pub fn get_race_size(&self, race_id: i32, game_data: &GameData) -> Option<i32> {
        let races = game_data.get_table("racialtypes")?;
        let row = races.get_by_id(race_id)?;

        row.get("Size")
            .or_else(|| row.get("size"))
            .and_then(std::clone::Clone::clone)
            .and_then(|s| s.parse::<i32>().ok())
    }

    pub fn get_base_speed(&self, game_data: &GameData) -> i32 {
        self.get_base_speed_for_race(self.race_id().0, game_data)
    }

    pub fn get_base_speed_for_race(&self, race_id: i32, game_data: &GameData) -> i32 {
        let Some(races) = game_data.get_table("racialtypes") else {
            return 30;
        };

        let Some(row) = races.get_by_id(race_id) else {
            return 30;
        };

        row.get("MovementRate")
            .or_else(|| row.get("movement_rate"))
            .and_then(std::clone::Clone::clone)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(30)
    }

    pub fn get_favored_class(&self, game_data: &GameData) -> Option<ClassId> {
        self.get_favored_class_for_race(self.race_id().0, game_data)
    }

    pub fn get_favored_class_for_race(
        &self,
        race_id: i32,
        game_data: &GameData,
    ) -> Option<ClassId> {
        let races = game_data.get_table("racialtypes")?;
        let row = races.get_by_id(race_id)?;

        row.get("FavoredClass")
            .or_else(|| row.get("favored_class"))
            .and_then(std::clone::Clone::clone)
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&v| v >= 0)
            .map(ClassId)
    }

    pub fn get_racial_feats_for_race(&self, race_id: i32, game_data: &GameData) -> Vec<FeatId> {
        let mut feats = HashSet::new();

        let Some(races) = game_data.get_table("racialtypes") else {
            return Vec::new();
        };

        let Some(row) = races.get_by_id(race_id) else {
            return Vec::new();
        };

        if let Some(feats_table_name) = row
            .get("FeatsTable")
            .or_else(|| row.get("feats_table"))
            .and_then(std::clone::Clone::clone)
            .filter(|s| !s.is_empty() && s != "****")
            && let Some(feats_table) = game_data.get_table(&feats_table_name.to_lowercase())
        {
            for row_idx in 0..feats_table.row_count() {
                if let Ok(feat_row) = feats_table.parser.get_row_dict(row_idx)
                    && let Some(feat_id) = feat_row
                        .get("FeatIndex")
                        .or_else(|| feat_row.get("feat_index"))
                        .and_then(|v| v.as_ref())
                        .and_then(|s| s.parse::<i32>().ok())
                        .filter(|&v| v >= 0)
                {
                    feats.insert(FeatId(feat_id));
                }
            }
        }

        for i in 1..=5 {
            let field_name = format!("Feat{i}");
            if let Some(feat_id) = row
                .get(&field_name)
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok())
                .filter(|&v| v >= 0)
            {
                feats.insert(FeatId(feat_id));
            }
        }

        feats.into_iter().collect()
    }

    fn get_feat_ids_from_table(&self, feats_table_name: &str, game_data: &GameData) -> Vec<FeatId> {
        let mut feats = HashSet::new();

        let Some(feats_table) = game_data.get_table(&feats_table_name.to_lowercase()) else {
            return Vec::new();
        };

        for row_idx in 0..feats_table.row_count() {
            if let Ok(feat_row) = feats_table.parser.get_row_dict(row_idx)
                && let Some(feat_id) = feat_row
                    .get("FeatIndex")
                    .or_else(|| feat_row.get("feat_index"))
                    .and_then(|v| v.as_ref())
                    .and_then(|s| s.parse::<i32>().ok())
                    .filter(|&v| v >= 0)
            {
                feats.insert(FeatId(feat_id));
            }
        }

        feats.into_iter().collect()
    }

    fn get_subrace_feats(&self, subrace_name: &str, game_data: &GameData) -> Vec<FeatId> {
        let Some(subrace_data) = self.get_subrace_data(subrace_name, game_data) else {
            return Vec::new();
        };
        let Some(feats_table_name) = subrace_data.feats_table else {
            return Vec::new();
        };

        self.get_feat_ids_from_table(&feats_table_name, game_data)
    }

    pub fn get_all_racial_feats(&self, game_data: &GameData) -> Vec<FeatId> {
        let mut feats = HashSet::new();

        for feat in self.get_racial_feats_for_race(self.race_id().0, game_data) {
            feats.insert(feat);
        }

        if let Some(subrace_name) = self.subrace_name(game_data) {
            for feat in self.get_subrace_feats(&subrace_name, game_data) {
                feats.insert(feat);
            }
        }

        feats.into_iter().collect()
    }

    fn get_race_appearance_index(&self, race_id: i32, game_data: &GameData) -> Option<i32> {
        let races = game_data.get_table("racialtypes")?;
        let row = races.get_by_id(race_id)?;

        row.get("Appearance")
            .or_else(|| row.get("appearance"))
            .and_then(std::clone::Clone::clone)
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&appearance| appearance >= 0)
            .or_else(|| (0..=6).contains(&race_id).then_some(race_id))
    }

    fn get_subrace_appearance_index(
        &self,
        subrace_name: &str,
        game_data: &GameData,
    ) -> Option<i32> {
        let subrace_id = self.resolve_subrace_row_index(subrace_name, game_data)?;
        let subraces = game_data.get_table("racialsubtypes")?;
        let row = subraces.get_by_id(subrace_id)?;

        row.get("AppearanceIndex")
            .or_else(|| row.get("appearanceindex"))
            .or_else(|| row.get("Appearance"))
            .or_else(|| row.get("appearance"))
            .and_then(std::clone::Clone::clone)
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&appearance| appearance >= 0)
    }

    fn resolve_appearance_index_for_race_change(
        &self,
        race_id: i32,
        subrace_name: Option<&str>,
        game_data: &GameData,
    ) -> Option<i32> {
        subrace_name
            .and_then(|name| self.get_subrace_appearance_index(name, game_data))
            .or_else(|| self.get_race_appearance_index(race_id, game_data))
    }

    pub fn normalize_race_fields_for_save(&mut self, game_data: &GameData) {
        let subrace_name = self.subrace_name(game_data);
        let race_id = self.race_id().0;

        match subrace_name.as_deref() {
            Some(name) => self.set_subrace_for_game_data(Some(name), game_data),
            None if self.subrace_string().is_some() || self.subrace_index().is_some() => {
                self.clear_subrace_value()
            }
            None => {}
        }

        if let Some(size) = self.get_race_size(race_id, game_data)
            && self.creature_size() != size
        {
            self.set_creature_size(size);
        }

        if let Some(appearance_index) = self.resolve_appearance_index_for_race_change(
            race_id,
            subrace_name.as_deref(),
            game_data,
        ) && self.get_i32("Appearance_Type") != Some(appearance_index)
        {
            self.set_u16("Appearance_Type", appearance_index as u16);
        }
    }

    pub fn validate_race_change(
        &self,
        new_race_id: i32,
        new_subrace: Option<&str>,
        game_data: &GameData,
    ) -> ValidationResult {
        let mut errors = Vec::new();

        let Some(races) = game_data.get_table("racialtypes") else {
            errors.push("Could not load racialtypes table".to_string());
            return ValidationResult::failure(errors);
        };

        if races.get_by_id(new_race_id).is_none() {
            errors.push(format!("Unknown race ID: {new_race_id}"));
            return ValidationResult::failure(errors);
        }

        if let Some(subrace_name) = new_subrace {
            let result = self.validate_subrace(new_race_id, subrace_name, game_data);
            if !result.valid {
                errors.extend(result.errors);
            }
        }

        if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(errors)
        }
    }

    pub fn validate_subrace(
        &self,
        race_id: i32,
        subrace_name: &str,
        game_data: &GameData,
    ) -> ValidationResult {
        if subrace_name.is_empty() {
            return ValidationResult::success();
        }

        let Some(subrace_data) = self.get_subrace_data(subrace_name, game_data) else {
            return ValidationResult::failure(vec![format!("Unknown subrace: {}", subrace_name)]);
        };

        if subrace_data.base_race != race_id {
            return ValidationResult::failure(vec![format!(
                "Subrace '{}' does not belong to race ID {}",
                subrace_name, race_id
            )]);
        }

        if !subrace_data.player_race {
            return ValidationResult::failure(vec![format!(
                "Subrace '{}' is not available to players",
                subrace_name
            )]);
        }

        ValidationResult::success()
    }

    pub fn change_race(
        &mut self,
        new_race_id: i32,
        new_subrace: Option<String>,
        preserve_feats: bool,
        game_data: &GameData,
    ) -> Result<RaceChangeResult, CharacterError> {
        debug!(
            "Changing race to {} (subrace: {:?})",
            new_race_id, new_subrace
        );

        let validation = self.validate_race_change(new_race_id, new_subrace.as_deref(), game_data);
        if !validation.valid {
            return Err(CharacterError::ValidationFailed {
                field: "race",
                message: validation.errors.join("; "),
            });
        }

        let old_race_id = self.race_id().0;
        let old_subrace = self.subrace_name(game_data);
        let old_race_name = self.race_name(game_data);
        let old_base_scores = self.base_scores();
        let old_scores = self.get_effective_abilities(game_data);

        let mut result = RaceChangeResult {
            old_race_id,
            old_race_name,
            old_subrace: old_subrace.clone(),
            new_race_id,
            new_race_name: self.get_race_name_by_id(new_race_id, game_data),
            new_subrace: new_subrace.clone(),
            ..Default::default()
        };

        let old_racial_mods = self.get_racial_modifier_deltas(game_data);

        self.set_race(RaceId(new_race_id));
        self.set_subrace_for_game_data(new_subrace.as_deref(), game_data);

        let new_racial_mods = self.get_racial_modifier_deltas(game_data);
        self.record_ability_modifier_changes(
            old_base_scores,
            &old_racial_mods,
            &new_racial_mods,
            "race",
            &mut result,
        );

        let old_size = self.creature_size();
        if let Some(new_size) = self.get_race_size(new_race_id, game_data)
            && old_size != new_size
        {
            self.set_creature_size(new_size);
            result.size_change = Some(SizeChange {
                old_size,
                new_size,
                old_name: self.get_size_name_from_2da(old_size, game_data),
                new_name: self.get_size_name_from_2da(new_size, game_data),
            });
        }

        let old_speed = self.get_base_speed_for_race(old_race_id, game_data);
        let new_speed = self.get_base_speed_for_race(new_race_id, game_data);
        if old_speed != new_speed {
            result.speed_change = Some(SpeedChange {
                old_speed,
                new_speed,
            });
        }

        if let Some(new_appearance_index) = self.resolve_appearance_index_for_race_change(
            new_race_id,
            new_subrace.as_deref(),
            game_data,
        ) && self.get_i32("Appearance_Type") != Some(new_appearance_index)
        {
            self.set_u16("Appearance_Type", new_appearance_index as u16);
        }

        if !preserve_feats {
            let old_racial_feats = self.get_racial_feats_for_race(old_race_id, game_data);
            for feat_id in old_racial_feats {
                if self.has_feat(feat_id) && self.remove_feat(feat_id).is_ok() {
                    result.feats_removed.push(FeatChange {
                        feat_id: feat_id.0,
                        feat_name: self.get_feat_name(feat_id, game_data),
                        source: "race".to_string(),
                    });
                }
            }

            if let Some(ref old_sub_name) = old_subrace {
                for feat_id in self.get_subrace_feats(old_sub_name, game_data) {
                    if self.has_feat(feat_id) && self.remove_feat(feat_id).is_ok() {
                        result.feats_removed.push(FeatChange {
                            feat_id: feat_id.0,
                            feat_name: self.get_feat_name(feat_id, game_data),
                            source: "subrace".to_string(),
                        });
                    }
                }
            }
        }

        let new_racial_feats = self.get_racial_feats_for_race(new_race_id, game_data);
        for feat_id in new_racial_feats {
            if !self.has_feat(feat_id)
                && self.add_feat_with_source(feat_id, FeatSource::Race).is_ok()
            {
                result.feats_added.push(FeatChange {
                    feat_id: feat_id.0,
                    feat_name: self.get_feat_name(feat_id, game_data),
                    source: "race".to_string(),
                });
            }
        }

        if let Some(ref new_sub_name) = new_subrace {
            for feat in self.get_subrace_feats(new_sub_name, game_data) {
                if !self.has_feat(feat) && self.add_feat_with_source(feat, FeatSource::Race).is_ok()
                {
                    result.feats_added.push(FeatChange {
                        feat_id: feat.0,
                        feat_name: self.get_feat_name(feat, game_data),
                        source: "subrace".to_string(),
                    });
                }
            }
        }

        self.apply_ability_batch_side_effects(old_scores, game_data);

        debug!("Race change complete: {:?}", result);
        Ok(result)
    }

    fn record_ability_modifier_changes(
        &self,
        base_scores: super::types::AbilityScores,
        old_mods: &AbilityModifiers,
        new_mods: &AbilityModifiers,
        source: &str,
        result: &mut RaceChangeResult,
    ) {
        for ability in AbilityIndex::all() {
            let old_modifier = old_mods.get(ability);
            let new_modifier = new_mods.get(ability);
            let modifier_delta = new_modifier - old_modifier;

            if modifier_delta == 0 {
                continue;
            }

            let base_value = base_scores.get(ability);
            result.ability_changes.push(AbilityChange {
                attribute: ability.gff_field().to_string(),
                old_value: base_value + old_modifier,
                new_value: base_value + new_modifier,
                modifier: modifier_delta,
                source: source.to_string(),
            });
        }
    }

    pub fn get_racial_properties(&self, game_data: &GameData) -> RacialProperties {
        let race_id = self.race_id().0;
        let subrace = self.subrace_name(game_data);

        let ability_modifiers = self.get_racial_modifier_deltas(game_data);

        let size = self.creature_size();

        RacialProperties {
            race_id,
            race_name: self.race_name(game_data),
            subrace,
            size,
            size_name: SizeCategory::from_id(size).name().to_string(),
            base_speed: self.get_base_speed(game_data),
            ability_modifiers,
            racial_feats: self
                .get_all_racial_feats(game_data)
                .into_iter()
                .map(|f| f.0)
                .collect(),
            favored_class: self.get_favored_class(game_data).map(|c| c.0),
            available_subraces: self.available_subraces(game_data),
        }
    }

    pub fn get_size_modifier(&self, size_id: i32, game_data: &GameData) -> i32 {
        let Some(size_table) = game_data.get_table("creaturesize") else {
            return SizeCategory::from_id(size_id).ac_modifier_default();
        };

        let Some(row) = size_table.get_by_id(size_id) else {
            return SizeCategory::from_id(size_id).ac_modifier_default();
        };

        row.get("ACAttackMod")
            .or_else(|| row.get("ac_attack_mod"))
            .and_then(std::clone::Clone::clone)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or_else(|| SizeCategory::from_id(size_id).ac_modifier_default())
    }

    pub fn get_size_name_from_2da(&self, size_id: i32, game_data: &GameData) -> String {
        let Some(size_table) = game_data.get_table("creaturesize") else {
            return SizeCategory::from_id(size_id).name().to_string();
        };

        let Some(row) = size_table.get_by_id(size_id) else {
            return SizeCategory::from_id(size_id).name().to_string();
        };

        if let Some(label) = row
            .get("Label")
            .or_else(|| row.get("label"))
            .and_then(std::clone::Clone::clone)
            .filter(|s| !s.is_empty() && s != "INVALID")
        {
            let mut chars = label.chars();
            match chars.next() {
                None => label,
                Some(first) => {
                    first.to_uppercase().collect::<String>()
                        + chars.as_str().to_lowercase().as_str()
                }
            }
        } else {
            SizeCategory::from_id(size_id).name().to_string()
        }
    }

    pub fn validate_race(&self, game_data: &GameData) -> ValidationResult {
        let mut errors = Vec::new();

        let race_id = self.race_id().0;

        let Some(races) = game_data.get_table("racialtypes") else {
            errors.push("Could not load racialtypes table".to_string());
            return ValidationResult::failure(errors);
        };

        if races.get_by_id(race_id).is_none() {
            errors.push(format!("Invalid race ID: {race_id}"));
        }

        if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(errors)
        }
    }

    pub fn get_subrace_data_by_index(
        &self,
        index: i32,
        game_data: &GameData,
    ) -> Option<SubraceData> {
        let subrace_table = game_data.get_table("racialsubtypes")?;
        let row_data = subrace_table.parser.get_row_dict(index as usize).ok()?;

        let name = row_data
            .get("Name")
            .or_else(|| row_data.get("name"))
            .and_then(std::clone::Clone::clone)
            .unwrap_or_default();
        let resolved_name = name
            .parse::<i32>()
            .ok()
            .and_then(|strref| game_data.get_string(strref));

        let label = row_data
            .get("Label")
            .or_else(|| row_data.get("label"))
            .and_then(std::clone::Clone::clone)
            .unwrap_or_else(|| name.clone());

        let base_race = row_data
            .get("BaseRace")
            .or_else(|| row_data.get("base_race"))
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(-1);

        let player_race = row_data
            .get("PlayerRace")
            .or_else(|| row_data.get("player_race"))
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1)
            == 1;

        let favored_class = row_data
            .get("FavoredClass")
            .or_else(|| row_data.get("favored_class"))
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&v| v >= 0);

        let feats_table = row_data
            .get("FeatsTable")
            .or_else(|| row_data.get("feats_table"))
            .and_then(std::clone::Clone::clone)
            .filter(|s| !s.is_empty() && s != "****");

        let get_mod = |field: &str| -> i32 {
            row_data
                .get(field)
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        };

        let ability_modifiers = AbilityModifiers::new(
            get_mod("StrAdjust"),
            get_mod("DexAdjust"),
            get_mod("ConAdjust"),
            get_mod("IntAdjust"),
            get_mod("WisAdjust"),
            get_mod("ChaAdjust"),
        );

        Some(SubraceData {
            id: index,
            name: resolved_name.unwrap_or(name),
            label,
            base_race,
            ability_modifiers,
            player_race,
            favored_class,
            feats_table,
        })
    }

    pub fn get_racial_modifier_deltas(&self, game_data: &GameData) -> AbilityModifiers {
        let race_id = self.race_id().0;

        if let Some(ref subrace_name) = self.subrace_string()
            && let Some(subrace_data) = self.get_subrace_data(subrace_name, game_data)
        {
            debug!(
                "get_racial_modifier_deltas: found subrace by name '{}': {:?}",
                subrace_name, subrace_data.ability_modifiers
            );
            return subrace_data.ability_modifiers;
        }

        if let Some(idx) = self.subrace_index()
            && let Some(subrace_data) = self.get_subrace_data_by_index(idx, game_data)
        {
            debug!(
                "get_racial_modifier_deltas: found subrace by index {}: {:?}",
                idx, subrace_data.ability_modifiers
            );
            return subrace_data.ability_modifiers;
        }

        let base_mods = self.get_racial_ability_modifiers_for_race(race_id, game_data);

        debug!(
            "get_racial_modifier_deltas: using base race modifiers: {:?}",
            base_mods
        );
        base_mods
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::{GameData, LoadedTable};
    use crate::parsers::gff::GffValue;
    use crate::parsers::tda::TDAParser;
    use crate::parsers::tlk::TLKParser;
    use ahash::AHashMap;
    use indexmap::IndexMap;
    use std::borrow::Cow;
    use std::sync::Arc;

    fn create_test_character() -> Character {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert(
            "Subrace".to_string(),
            GffValue::String(Cow::Owned("Moon Elf".to_string())),
        );
        fields.insert("CreatureSize".to_string(), GffValue::Int(4));
        Character::from_gff(fields)
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

    #[test]
    fn test_race_id() {
        let character = create_test_character();
        assert_eq!(character.race_id(), RaceId(1));
    }

    #[test]
    fn test_set_race() {
        let mut character = create_test_character();
        character.set_race(RaceId(5));
        assert_eq!(character.race_id(), RaceId(5));
        assert!(character.is_modified());
    }

    #[test]
    fn test_subrace() {
        let character = create_test_character();
        assert_eq!(character.subrace(), Some("Moon Elf".to_string()));
    }

    #[test]
    fn test_subrace_empty() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert(
            "Subrace".to_string(),
            GffValue::String(Cow::Owned(String::new())),
        );
        let character = Character::from_gff(fields);
        assert_eq!(character.subrace(), None);
    }

    #[test]
    fn test_set_subrace() {
        let mut character = create_test_character();
        character.set_subrace(Some("Sun Elf".to_string()));
        assert_eq!(character.subrace(), Some("Sun Elf".to_string()));
        assert!(character.is_modified());
    }

    #[test]
    fn test_set_subrace_none() {
        let mut character = create_test_character();
        character.set_subrace(None);
        assert_eq!(character.subrace(), None);
        assert!(character.is_modified());
    }

    #[test]
    fn test_creature_size() {
        let character = create_test_character();
        assert_eq!(character.creature_size(), 4);
    }

    #[test]
    fn test_set_creature_size() {
        let mut character = create_test_character();
        character.set_creature_size(3);
        assert_eq!(character.creature_size(), 3);
        assert!(character.is_modified());
    }

    #[test]
    fn test_size_category() {
        let character = create_test_character();
        assert_eq!(character.size_category(), SizeCategory::Medium);
    }

    #[test]
    fn test_size_category_from_id() {
        assert_eq!(SizeCategory::from_id(0), SizeCategory::Fine);
        assert_eq!(SizeCategory::from_id(4), SizeCategory::Medium);
        assert_eq!(SizeCategory::from_id(8), SizeCategory::Colossal);
        assert_eq!(SizeCategory::from_id(100), SizeCategory::Medium);
    }

    #[test]
    fn test_size_category_modifiers() {
        assert_eq!(SizeCategory::Fine.ac_modifier_default(), 8);
        assert_eq!(SizeCategory::Small.ac_modifier_default(), 1);
        assert_eq!(SizeCategory::Medium.ac_modifier_default(), 0);
        assert_eq!(SizeCategory::Large.ac_modifier_default(), -1);
        assert_eq!(SizeCategory::Colossal.ac_modifier_default(), -8);
    }

    #[test]
    fn test_size_category_names() {
        assert_eq!(SizeCategory::Tiny.name(), "Tiny");
        assert_eq!(SizeCategory::Medium.name(), "Medium");
        assert_eq!(SizeCategory::Huge.name(), "Huge");
    }

    #[test]
    fn test_change_race_updates_numeric_subrace_and_appearance() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert("Subrace".to_string(), GffValue::Byte(1));
        fields.insert("CreatureSize".to_string(), GffValue::Int(4));
        fields.insert("Appearance_Type".to_string(), GffValue::Word(40));
        fields.insert("Str".to_string(), GffValue::Byte(10));
        fields.insert("Dex".to_string(), GffValue::Byte(10));
        fields.insert("Con".to_string(), GffValue::Byte(10));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));

        let mut character = Character::from_gff(fields);

        let mut game_data = create_mock_game_data();
        game_data.tables.insert(
            "racialtypes".to_string(),
            create_loaded_table(
                "racialtypes",
                &["Label", "Size", "MovementRate", "Appearance"],
                vec![
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Dwarf".to_string())),
                        ("Size".to_string(), Some("3".to_string())),
                        ("MovementRate".to_string(), Some("20".to_string())),
                        ("Appearance".to_string(), None),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("YuanTi".to_string())),
                        ("Size".to_string(), Some("4".to_string())),
                        ("MovementRate".to_string(), Some("30".to_string())),
                        ("Appearance".to_string(), Some("40".to_string())),
                    ]),
                ],
            ),
        );
        game_data.tables.insert(
            "racialsubtypes".to_string(),
            create_loaded_table(
                "racialsubtypes",
                &["Label", "BaseRace", "PlayerRace", "AppearanceIndex"],
                vec![
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Unused0".to_string())),
                        ("BaseRace".to_string(), Some("-1".to_string())),
                        ("PlayerRace".to_string(), Some("0".to_string())),
                        ("AppearanceIndex".to_string(), None),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Unused1".to_string())),
                        ("BaseRace".to_string(), Some("-1".to_string())),
                        ("PlayerRace".to_string(), Some("0".to_string())),
                        ("AppearanceIndex".to_string(), None),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Shield_Dwarf".to_string())),
                        ("BaseRace".to_string(), Some("0".to_string())),
                        ("PlayerRace".to_string(), Some("1".to_string())),
                        ("AppearanceIndex".to_string(), Some("0".to_string())),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Yuan_Ti_Pureblood".to_string())),
                        ("BaseRace".to_string(), Some("1".to_string())),
                        ("PlayerRace".to_string(), Some("1".to_string())),
                        ("AppearanceIndex".to_string(), Some("40".to_string())),
                    ]),
                ],
            ),
        );

        character
            .change_race(0, Some("Shield_Dwarf".to_string()), false, &game_data)
            .expect("Race change should succeed");

        assert_eq!(character.race_id(), RaceId(0));
        assert_eq!(character.subrace_index(), Some(2));
        assert_eq!(
            character.subrace_name(&game_data),
            Some("Shield_Dwarf".to_string())
        );
        assert_eq!(character.creature_size(), 3);
        assert_eq!(character.get_i32("Appearance_Type"), Some(0));
        assert!(matches!(
            character.gff().get("Subrace"),
            Some(GffValue::Byte(2))
        ));
    }

    #[test]
    fn test_change_race_normalizes_skill_points_and_updates_hp_side_effects() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(0));
        fields.insert("CreatureSize".to_string(), GffValue::Int(4));
        fields.insert("Str".to_string(), GffValue::Byte(10));
        fields.insert("Dex".to_string(), GffValue::Byte(10));
        fields.insert("Con".to_string(), GffValue::Byte(10));
        fields.insert("Int".to_string(), GffValue::Byte(8));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));
        fields.insert("SkillPoints".to_string(), GffValue::Short(35));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(10));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(10));
        fields.insert("HitPoints".to_string(), GffValue::Int(10));

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(1));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let mut history_entry = IndexMap::new();
        history_entry.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        history_entry.insert("SkillPoints".to_string(), GffValue::Short(35));
        history_entry.insert("LvlStatHitDie".to_string(), GffValue::Byte(10));
        history_entry.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        history_entry.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
        history_entry.insert("SkillList".to_string(), GffValue::ListOwned(vec![]));
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![history_entry]),
        );

        let mut character = Character::from_gff(fields);
        let mut game_data = create_mock_game_data();
        game_data.tables.insert(
            "racialtypes".to_string(),
            create_loaded_table(
                "racialtypes",
                &[
                    "Label",
                    "Size",
                    "MovementRate",
                    "Appearance",
                    "ConAdjust",
                    "IntAdjust",
                    "SkillPointModifier",
                ],
                vec![
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Human".to_string())),
                        ("Size".to_string(), Some("4".to_string())),
                        ("MovementRate".to_string(), Some("30".to_string())),
                        ("Appearance".to_string(), Some("0".to_string())),
                        ("ConAdjust".to_string(), Some("0".to_string())),
                        ("IntAdjust".to_string(), Some("0".to_string())),
                        ("SkillPointModifier".to_string(), Some("0".to_string())),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("YuanTi".to_string())),
                        ("Size".to_string(), Some("4".to_string())),
                        ("MovementRate".to_string(), Some("30".to_string())),
                        ("Appearance".to_string(), Some("40".to_string())),
                        ("ConAdjust".to_string(), Some("2".to_string())),
                        ("IntAdjust".to_string(), Some("2".to_string())),
                        ("SkillPointModifier".to_string(), Some("0".to_string())),
                    ]),
                ],
            ),
        );
        game_data.tables.insert(
            "classes".to_string(),
            create_loaded_table(
                "classes",
                &["Label", "HitDie", "SkillPointBase"],
                vec![AHashMap::from_iter([
                    ("Label".to_string(), Some("Fighter".to_string())),
                    ("HitDie".to_string(), Some("10".to_string())),
                    ("SkillPointBase".to_string(), Some("2".to_string())),
                ])],
            ),
        );

        character
            .change_race(1, None, false, &game_data)
            .expect("Race change should succeed");

        assert_eq!(character.base_ability(AbilityIndex::CON), 10);
        assert_eq!(character.base_ability(AbilityIndex::INT), 8);
        let effective = character.get_effective_abilities(&game_data);
        assert_eq!(effective.con, 12);
        assert_eq!(effective.int, 10);
        assert_eq!(character.get_available_skill_points(), 8);
        assert_eq!(character.max_hp(), 11);
        assert_eq!(character.current_hp(), 11);
        assert_eq!(character.base_hp(), 11);
    }

    #[test]
    fn test_get_racial_modifier_deltas_prefers_subrace_modifiers_over_base_race() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert(
            "Subrace".to_string(),
            GffValue::String(Cow::Owned("Moon Elf".to_string())),
        );

        let character = Character::from_gff(fields);

        let mut game_data = create_mock_game_data();
        game_data.tables.insert(
            "racialtypes".to_string(),
            create_loaded_table(
                "racialtypes",
                &[
                    "Label",
                    "StrAdjust",
                    "DexAdjust",
                    "ConAdjust",
                    "IntAdjust",
                    "WisAdjust",
                    "ChaAdjust",
                ],
                vec![
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Human".to_string())),
                        ("StrAdjust".to_string(), Some("0".to_string())),
                        ("DexAdjust".to_string(), Some("0".to_string())),
                        ("ConAdjust".to_string(), Some("0".to_string())),
                        ("IntAdjust".to_string(), Some("0".to_string())),
                        ("WisAdjust".to_string(), Some("0".to_string())),
                        ("ChaAdjust".to_string(), Some("0".to_string())),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Elf".to_string())),
                        ("StrAdjust".to_string(), Some("0".to_string())),
                        ("DexAdjust".to_string(), Some("2".to_string())),
                        ("ConAdjust".to_string(), Some("-2".to_string())),
                        ("IntAdjust".to_string(), Some("2".to_string())),
                        ("WisAdjust".to_string(), Some("0".to_string())),
                        ("ChaAdjust".to_string(), Some("2".to_string())),
                    ]),
                ],
            ),
        );
        game_data.tables.insert(
            "racialsubtypes".to_string(),
            create_loaded_table(
                "racialsubtypes",
                &[
                    "Label",
                    "BaseRace",
                    "PlayerRace",
                    "StrAdjust",
                    "DexAdjust",
                    "ConAdjust",
                    "IntAdjust",
                    "WisAdjust",
                    "ChaAdjust",
                ],
                vec![AHashMap::from_iter([
                    ("Label".to_string(), Some("Moon Elf".to_string())),
                    ("BaseRace".to_string(), Some("1".to_string())),
                    ("PlayerRace".to_string(), Some("1".to_string())),
                    ("StrAdjust".to_string(), Some("0".to_string())),
                    ("DexAdjust".to_string(), Some("2".to_string())),
                    ("ConAdjust".to_string(), Some("-2".to_string())),
                    ("IntAdjust".to_string(), Some("2".to_string())),
                    ("WisAdjust".to_string(), Some("0".to_string())),
                    ("ChaAdjust".to_string(), Some("2".to_string())),
                ])],
            ),
        );

        let mods = character.get_racial_modifier_deltas(&game_data);

        assert_eq!(mods.str_mod, 0);
        assert_eq!(mods.dex_mod, 2);
        assert_eq!(mods.con_mod, -2);
        assert_eq!(mods.int_mod, 2);
        assert_eq!(mods.wis_mod, 0);
        assert_eq!(mods.cha_mod, 2);
    }

    #[test]
    fn test_change_race_applies_subrace_modifiers_once_when_subrace_is_authoritative() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(0));
        fields.insert("CreatureSize".to_string(), GffValue::Int(4));
        fields.insert("Str".to_string(), GffValue::Byte(10));
        fields.insert("Dex".to_string(), GffValue::Byte(10));
        fields.insert("Con".to_string(), GffValue::Byte(10));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));

        let mut character = Character::from_gff(fields);
        let mut game_data = create_mock_game_data();
        game_data.tables.insert(
            "racialtypes".to_string(),
            create_loaded_table(
                "racialtypes",
                &[
                    "Label",
                    "Size",
                    "MovementRate",
                    "Appearance",
                    "DexAdjust",
                    "IntAdjust",
                    "ChaAdjust",
                ],
                vec![
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Human".to_string())),
                        ("Size".to_string(), Some("4".to_string())),
                        ("MovementRate".to_string(), Some("30".to_string())),
                        ("Appearance".to_string(), Some("0".to_string())),
                        ("DexAdjust".to_string(), Some("0".to_string())),
                        ("IntAdjust".to_string(), Some("0".to_string())),
                        ("ChaAdjust".to_string(), Some("0".to_string())),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("YuanTi".to_string())),
                        ("Size".to_string(), Some("4".to_string())),
                        ("MovementRate".to_string(), Some("30".to_string())),
                        ("Appearance".to_string(), Some("40".to_string())),
                        ("DexAdjust".to_string(), Some("2".to_string())),
                        ("IntAdjust".to_string(), Some("2".to_string())),
                        ("ChaAdjust".to_string(), Some("2".to_string())),
                    ]),
                ],
            ),
        );
        game_data.tables.insert(
            "racialsubtypes".to_string(),
            create_loaded_table(
                "racialsubtypes",
                &[
                    "Label",
                    "BaseRace",
                    "PlayerRace",
                    "DexAdjust",
                    "IntAdjust",
                    "ChaAdjust",
                ],
                vec![AHashMap::from_iter([
                    ("Label".to_string(), Some("Yuan-ti Pureblood ".to_string())),
                    ("BaseRace".to_string(), Some("1".to_string())),
                    ("PlayerRace".to_string(), Some("1".to_string())),
                    ("DexAdjust".to_string(), Some("2".to_string())),
                    ("IntAdjust".to_string(), Some("2".to_string())),
                    ("ChaAdjust".to_string(), Some("2".to_string())),
                ])],
            ),
        );

        character
            .change_race(1, Some("Yuan-ti Pureblood ".to_string()), false, &game_data)
            .expect("Race change should succeed");

        assert_eq!(character.base_ability(AbilityIndex::STR), 10);
        assert_eq!(character.base_ability(AbilityIndex::DEX), 10);
        assert_eq!(character.base_ability(AbilityIndex::CON), 10);
        assert_eq!(character.base_ability(AbilityIndex::INT), 10);
        assert_eq!(character.base_ability(AbilityIndex::WIS), 10);
        assert_eq!(character.base_ability(AbilityIndex::CHA), 10);
        let effective = character.get_effective_abilities(&game_data);
        assert_eq!(effective.str_, 10);
        assert_eq!(effective.dex, 12);
        assert_eq!(effective.con, 10);
        assert_eq!(effective.int, 12);
        assert_eq!(effective.wis, 10);
        assert_eq!(effective.cha, 12);
    }

    #[test]
    fn test_normalize_race_fields_for_save_repairs_string_subrace_and_appearance() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(0));
        fields.insert(
            "Subrace".to_string(),
            GffValue::String(Cow::Owned("Shield_Dwarf".to_string())),
        );
        fields.insert("CreatureSize".to_string(), GffValue::Int(4));
        fields.insert("Appearance_Type".to_string(), GffValue::Word(40));

        let mut character = Character::from_gff(fields);

        let mut game_data = create_mock_game_data();
        game_data.tables.insert(
            "racialtypes".to_string(),
            create_loaded_table(
                "racialtypes",
                &["Label", "Size", "MovementRate", "Appearance"],
                vec![AHashMap::from_iter([
                    ("Label".to_string(), Some("Dwarf".to_string())),
                    ("Size".to_string(), Some("3".to_string())),
                    ("MovementRate".to_string(), Some("20".to_string())),
                    ("Appearance".to_string(), None),
                ])],
            ),
        );
        game_data.tables.insert(
            "racialsubtypes".to_string(),
            create_loaded_table(
                "racialsubtypes",
                &["Label", "BaseRace", "PlayerRace", "AppearanceIndex"],
                vec![
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Unused0".to_string())),
                        ("BaseRace".to_string(), Some("-1".to_string())),
                        ("PlayerRace".to_string(), Some("0".to_string())),
                        ("AppearanceIndex".to_string(), None),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Unused1".to_string())),
                        ("BaseRace".to_string(), Some("-1".to_string())),
                        ("PlayerRace".to_string(), Some("0".to_string())),
                        ("AppearanceIndex".to_string(), None),
                    ]),
                    AHashMap::from_iter([
                        ("Label".to_string(), Some("Shield_Dwarf".to_string())),
                        ("BaseRace".to_string(), Some("0".to_string())),
                        ("PlayerRace".to_string(), Some("1".to_string())),
                        ("AppearanceIndex".to_string(), Some("0".to_string())),
                    ]),
                ],
            ),
        );

        character.normalize_race_fields_for_save(&game_data);

        assert_eq!(character.subrace_index(), Some(2));
        assert_eq!(character.creature_size(), 3);
        assert_eq!(character.get_i32("Appearance_Type"), Some(0));
        assert!(matches!(
            character.gff().get("Subrace"),
            Some(GffValue::Byte(2))
        ));
    }
}
