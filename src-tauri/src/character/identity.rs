use super::{Character, CharacterError, ClassId, FeatId, FeatSource};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;

use crate::loaders::GameData;

pub const ALIGNMENT_MIN: i32 = 0;
pub const ALIGNMENT_MAX: i32 = 100;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Type)]
pub struct Alignment {
    pub law_chaos: i32,
    pub good_evil: i32,
}

impl Alignment {
    pub fn new(law_chaos: i32, good_evil: i32) -> Self {
        Self {
            law_chaos,
            good_evil,
        }
    }

    pub fn alignment_string(&self) -> String {
        let law_axis = match self.law_chaos {
            0..=30 => "Chaotic",
            31..=69 => "Neutral",
            _ => "Lawful",
        };

        let good_axis = match self.good_evil {
            0..=30 => "Evil",
            31..=69 => "Neutral",
            _ => "Good",
        };

        if law_axis == "Neutral" && good_axis == "Neutral" {
            "True Neutral".to_string()
        } else {
            format!("{law_axis} {good_axis}")
        }
    }

    pub fn is_lawful(&self) -> bool {
        self.law_chaos >= 70
    }

    pub fn is_chaotic(&self) -> bool {
        self.law_chaos <= 30
    }

    pub fn is_good(&self) -> bool {
        self.good_evil >= 70
    }

    pub fn is_evil(&self) -> bool {
        self.good_evil <= 30
    }

    pub fn is_neutral_law_chaos(&self) -> bool {
        self.law_chaos > 30 && self.law_chaos < 70
    }

    pub fn is_neutral_good_evil(&self) -> bool {
        self.good_evil > 30 && self.good_evil < 70
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct Biography {
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub gender: i32,
    pub deity: String,
    pub description: String,
    pub background: Option<String>,
    pub experience: i32,
    pub alignment: Alignment,
}

impl Character {
    pub fn biography(&self, game_data: &GameData) -> Biography {
        Biography {
            name: self.full_name(),
            first_name: self.first_name(),
            last_name: self.last_name(),
            age: self.age(),
            gender: self.gender(),
            deity: self.deity(),
            description: self.description(),
            background: self.background(game_data),
            experience: self.experience(),
            alignment: self.alignment(),
        }
    }

    pub fn first_name(&self) -> String {
        self.get_localized_string_value("FirstName")
            .unwrap_or_default()
    }

    pub fn last_name(&self) -> String {
        self.get_localized_string_value("LastName")
            .unwrap_or_default()
    }

    pub fn full_name(&self) -> String {
        let first = self.first_name();
        let last = self.last_name();
        format!("{first} {last}").trim().to_string()
    }

    pub fn set_first_name(&mut self, name: String) {
        self.set_localized_string("FirstName", name);
    }

    pub fn set_last_name(&mut self, name: String) {
        self.set_localized_string("LastName", name);
    }

    pub fn age(&self) -> i32 {
        self.get_i32("Age").unwrap_or(0)
    }

    pub fn set_age(&mut self, age: i32) -> Result<(), CharacterError> {
        if age < 0 {
            return Err(CharacterError::OutOfRange {
                field: "Age",
                value: age,
                min: 0,
                max: i32::MAX,
            });
        }
        self.set_i32("Age", age);
        Ok(())
    }

    pub fn experience(&self) -> i32 {
        self.get_i32("Experience").unwrap_or(0)
    }

    pub fn set_experience(&mut self, xp: i32) -> Result<(), CharacterError> {
        if xp < 0 {
            return Err(CharacterError::OutOfRange {
                field: "Experience",
                value: xp,
                min: 0,
                max: i32::MAX,
            });
        }
        self.set_i32("Experience", xp);
        Ok(())
    }

    pub fn alignment(&self) -> Alignment {
        let law_chaos = self.get_i32("LawfulChaotic").unwrap_or(50);
        let good_evil = self.get_i32("GoodEvil").unwrap_or(50);
        Alignment::new(law_chaos, good_evil)
    }

    pub fn set_alignment(
        &mut self,
        law_chaos: Option<i32>,
        good_evil: Option<i32>,
    ) -> Result<(), CharacterError> {
        let current = self.alignment();

        let new_law_chaos = law_chaos.unwrap_or(current.law_chaos);
        let new_good_evil = good_evil.unwrap_or(current.good_evil);

        let clamped_law_chaos = new_law_chaos.clamp(ALIGNMENT_MIN, ALIGNMENT_MAX);
        let clamped_good_evil = new_good_evil.clamp(ALIGNMENT_MIN, ALIGNMENT_MAX);

        self.set_i32("LawfulChaotic", clamped_law_chaos);
        self.set_i32("GoodEvil", clamped_good_evil);

        Ok(())
    }

    /// Get the character's deity.
    pub fn deity(&self) -> String {
        // Deity can be stored as a simple String (CExoString) or LocString
        if let Some(s) = self.get_string("Deity") {
            return s.to_string();
        }
        self.get_localized_string_value("Deity").unwrap_or_default()
    }

    pub fn set_deity(&mut self, deity: String) {
        self.set_localized_string("Deity", deity);
    }

    pub fn description(&self) -> String {
        self.get_localized_string_value("Description")
            .unwrap_or_default()
    }

    pub fn set_description(&mut self, description: String) {
        self.set_localized_string("Description", description);
    }

    pub fn gender(&self) -> i32 {
        self.get_i32("Gender").unwrap_or(0)
    }

    pub fn set_gender(&mut self, gender: i32) -> Result<(), CharacterError> {
        if !(0..=1).contains(&gender) {
            return Err(CharacterError::OutOfRange {
                field: "Gender",
                value: gender,
                min: 0,
                max: 1,
            });
        }
        self.set_i32("Gender", gender);
        Ok(())
    }

    fn background_row_i32(row: &AHashMap<String, Option<String>>, keys: &[&str]) -> Option<i32> {
        keys.iter()
            .find_map(|key| row.get(*key).and_then(std::clone::Clone::clone))
            .and_then(|value| value.parse::<i32>().ok())
    }

    fn background_row_removed(row: &AHashMap<String, Option<String>>) -> bool {
        Self::background_row_i32(row, &["REMOVED", "Removed"]).unwrap_or(0) != 0
    }

    fn background_row_name(row: &AHashMap<String, Option<String>>, game_data: &GameData) -> String {
        row.get("label")
            .or_else(|| row.get("Label"))
            .and_then(std::clone::Clone::clone)
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                row.get("name")
                    .or_else(|| row.get("Name"))
                    .and_then(std::clone::Clone::clone)
                    .and_then(|value| value.parse::<i32>().ok())
                    .and_then(|str_ref| game_data.get_string(str_ref))
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(|| row.get("Label").and_then(std::clone::Clone::clone))
            .unwrap_or_else(|| "Background".to_string())
    }

    fn background_row_feat_ids(row: &AHashMap<String, Option<String>>) -> Vec<FeatId> {
        let mut feat_ids = Vec::new();

        for keys in [
            ["DisplayFeat", "display_feat"].as_slice(),
            ["FeatGained", "feat_gained"].as_slice(),
            ["MasterFeatGained", "master_feat_gained"].as_slice(),
        ] {
            if let Some(feat_id) =
                Self::background_row_i32(row, keys).filter(|feat_id| *feat_id >= 0)
            {
                let feat_id = FeatId(feat_id);
                if !feat_ids.contains(&feat_id) {
                    feat_ids.push(feat_id);
                }
            }
        }

        feat_ids
    }

    fn background_row_requirements(
        &self,
        row: &AHashMap<String, Option<String>>,
        game_data: &GameData,
    ) -> Vec<String> {
        let mut missing = Vec::new();

        let ability_requirements = [
            (
                "MINSTR",
                "Strength",
                self.get_i32("Str").unwrap_or(10),
                true,
            ),
            (
                "MINDEX",
                "Dexterity",
                self.get_i32("Dex").unwrap_or(10),
                true,
            ),
            (
                "MINCON",
                "Constitution",
                self.get_i32("Con").unwrap_or(10),
                true,
            ),
            (
                "MININT",
                "Intelligence",
                self.get_i32("Int").unwrap_or(10),
                true,
            ),
            ("MINWIS", "Wisdom", self.get_i32("Wis").unwrap_or(10), true),
            (
                "MINCHA",
                "Charisma",
                self.get_i32("Cha").unwrap_or(10),
                true,
            ),
            (
                "MAXSTR",
                "Strength",
                self.get_i32("Str").unwrap_or(10),
                false,
            ),
            (
                "MAXDEX",
                "Dexterity",
                self.get_i32("Dex").unwrap_or(10),
                false,
            ),
            (
                "MAXCON",
                "Constitution",
                self.get_i32("Con").unwrap_or(10),
                false,
            ),
            (
                "MAXINT",
                "Intelligence",
                self.get_i32("Int").unwrap_or(10),
                false,
            ),
            ("MAXWIS", "Wisdom", self.get_i32("Wis").unwrap_or(10), false),
            (
                "MAXCHA",
                "Charisma",
                self.get_i32("Cha").unwrap_or(10),
                false,
            ),
        ];

        for (field, label, score, is_minimum) in ability_requirements {
            if let Some(value) = Self::background_row_i32(row, &[field]).filter(|value| *value > 0)
            {
                let invalid = if is_minimum {
                    score < value
                } else {
                    score > value
                };
                if invalid {
                    let requirement = if is_minimum {
                        format!("Requires {label} {value}")
                    } else {
                        format!("Requires {label} {value} or lower")
                    };
                    missing.push(requirement);
                }
            }
        }

        if let Some(min_bab) =
            Self::background_row_i32(row, &["MINATTACKBONUS"]).filter(|value| *value > 0)
        {
            let bab = self.calculate_bab(game_data);
            if bab < min_bab {
                missing.push(format!("Requires Base Attack Bonus +{min_bab}"));
            }
        }

        if let Some(required_gender) =
            Self::background_row_i32(row, &["Gender"]).filter(|value| *value >= 0)
        {
            let current_gender = self.gender();
            if required_gender <= 1 && current_gender != required_gender {
                let gender_label = if required_gender == 1 {
                    "Female"
                } else {
                    "Male"
                };
                missing.push(format!("Requires {gender_label}"));
            }
        }

        let required_classes: Vec<ClassId> = ["OrReqClass0", "OrReqClass1", "OrReqClass2"]
            .iter()
            .filter_map(|field| Self::background_row_i32(row, &[*field]))
            .filter(|class_id| *class_id >= 0)
            .map(ClassId)
            .collect();

        if !required_classes.is_empty()
            && !required_classes
                .iter()
                .any(|class_id| self.class_level(*class_id) > 0)
        {
            let class_names = required_classes
                .iter()
                .map(|class_id| self.get_class_name(*class_id, game_data))
                .collect::<Vec<_>>()
                .join(", ");
            missing.push(format!("Requires one of: {class_names}"));
        }

        missing
    }

    fn current_background_feat_ids(&self, game_data: &GameData) -> Vec<FeatId> {
        let mut background_feat_ids = Vec::new();

        if let Some(backgrounds_table) = game_data.get_table("backgrounds") {
            for row_id in 0..backgrounds_table.row_count() as i32 {
                let Some(row) = backgrounds_table.get_by_id(row_id) else {
                    continue;
                };
                if Self::background_row_removed(&row) {
                    continue;
                }

                for feat_id in Self::background_row_feat_ids(&row) {
                    if self.has_feat(feat_id) && !background_feat_ids.contains(&feat_id) {
                        background_feat_ids.push(feat_id);
                    }
                }
            }
        }

        if !background_feat_ids.is_empty() {
            return background_feat_ids;
        }

        self.feat_ids()
            .into_iter()
            .filter(|feat_id| self.feat_source(*feat_id) == Some(FeatSource::Background))
            .collect()
    }

    fn resolve_background_selection(
        &self,
        background_id: i32,
        game_data: &GameData,
    ) -> Result<(Vec<FeatId>, String, AHashMap<String, Option<String>>), CharacterError> {
        let backgrounds_table = game_data
            .get_table("backgrounds")
            .ok_or_else(|| CharacterError::TableNotFound("backgrounds".to_string()))?;

        let row = backgrounds_table
            .get_by_id(background_id)
            .ok_or(CharacterError::NotFound {
                entity: "Background",
                id: background_id,
            })?;

        if Self::background_row_removed(&row) {
            return Err(CharacterError::NotFound {
                entity: "Background",
                id: background_id,
            });
        }

        let feat_ids = Self::background_row_feat_ids(&row);
        if feat_ids.is_empty() {
            Err(CharacterError::ValidationFailed {
                field: "background",
                message: format!("Background {background_id} does not define any feats"),
            })
        } else {
            let name = Self::background_row_name(&row, game_data);

            Ok((feat_ids, name, row))
        }
    }

    fn background_name_for_feat(&self, feat_id: FeatId, game_data: &GameData) -> Option<String> {
        let backgrounds_table = game_data.get_table("backgrounds")?;

        for row_id in 0..backgrounds_table.row_count() as i32 {
            let Some(row) = backgrounds_table.get_by_id(row_id) else {
                continue;
            };
            if Self::background_row_removed(&row) {
                continue;
            }
            if !Self::background_row_feat_ids(&row).contains(&feat_id) {
                continue;
            }

            return Some(Self::background_row_name(&row, game_data));
        }

        None
    }

    pub fn set_background(
        &mut self,
        background_id: Option<i32>,
        game_data: &GameData,
    ) -> Result<Option<String>, CharacterError> {
        let current_background_feat_ids = self.current_background_feat_ids(game_data);

        let Some(background_id) = background_id else {
            for feat_id in current_background_feat_ids {
                self.remove_feat(feat_id)?;
            }
            return Ok(None);
        };

        let (new_feat_ids, background_name, background_row) =
            self.resolve_background_selection(background_id, game_data)?;

        let mut missing_requirements = self.background_row_requirements(&background_row, game_data);

        if let Some(primary_feat_id) = new_feat_ids
            .first()
            .copied()
            .filter(|feat_id| !self.has_feat(*feat_id))
        {
            let prereq_result = self.validate_feat_prerequisites(primary_feat_id, game_data);
            missing_requirements.extend(prereq_result.missing_requirements);
        }

        if !missing_requirements.is_empty() {
            missing_requirements.sort();
            missing_requirements.dedup();
            return Err(CharacterError::ValidationFailed {
                field: "background",
                message: missing_requirements.join("; "),
            });
        }

        for feat_id in current_background_feat_ids
            .into_iter()
            .filter(|feat_id| !new_feat_ids.contains(feat_id))
        {
            self.remove_feat(feat_id)?;
        }

        for feat_id in new_feat_ids {
            if !self.has_feat(feat_id) {
                self.add_feat_with_source(feat_id, FeatSource::Background)?;
            }
        }

        Ok(Some(background_name))
    }

    /// Get the character's background trait if present.
    /// In NWN2, backgrounds are handled via history feats or traits.
    fn resolved_background_row(&self, game_data: &GameData) -> Option<(i32, String)> {
        let current_background_feat_ids = self.current_background_feat_ids(game_data);
        if current_background_feat_ids.is_empty() {
            return None;
        }

        let current_background_feat_set: HashSet<i32> = current_background_feat_ids
            .iter()
            .map(|feat_id| feat_id.0)
            .collect();

        if let Some(backgrounds_table) = game_data.get_table("backgrounds") {
            let mut best_match: Option<(usize, i32, String)> = None;

            for row_id in 0..backgrounds_table.row_count() as i32 {
                let Some(row) = backgrounds_table.get_by_id(row_id) else {
                    continue;
                };
                if Self::background_row_removed(&row) {
                    continue;
                }

                let row_feat_ids = Self::background_row_feat_ids(&row);
                if row_feat_ids.is_empty() {
                    continue;
                }

                let matches_row = row_feat_ids
                    .iter()
                    .all(|feat_id| current_background_feat_set.contains(&feat_id.0));
                if !matches_row {
                    continue;
                }

                let row_name = Self::background_row_name(&row, game_data);
                let row_specificity = row_feat_ids.len();
                if best_match
                    .as_ref()
                    .is_none_or(|(best_specificity, _, _)| row_specificity > *best_specificity)
                {
                    best_match = Some((row_specificity, row_id, row_name));
                }
            }

            if let Some((_, row_id, background_name)) = best_match {
                return Some((row_id, background_name));
            }
        }

        current_background_feat_ids
            .into_iter()
            .next()
            .map(|feat_id| {
                (
                    -1,
                    self.background_name_for_feat(feat_id, game_data)
                        .unwrap_or_else(|| self.get_feat_name(feat_id, game_data)),
                )
            })
    }

    pub fn background_id(&self, game_data: &GameData) -> Option<i32> {
        self.resolved_background_row(game_data)
            .map(|(row_id, _)| row_id)
            .filter(|row_id| *row_id >= 0)
    }

    pub fn background(&self, game_data: &GameData) -> Option<String> {
        self.resolved_background_row(game_data)
            .map(|(_, background_name)| background_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::LoadedTable;
    use crate::parsers::gff::{GffValue, LocalizedString, LocalizedSubstring};
    use crate::parsers::tda::TDAParser;
    use crate::parsers::tlk::TLKParser;
    use ahash::AHashMap;
    use indexmap::IndexMap;
    use std::borrow::Cow;
    use std::sync::{Arc, RwLock};

    fn create_test_game_data() -> GameData {
        GameData::new(Arc::new(RwLock::new(TLKParser::default())))
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

    fn create_test_character() -> Character {
        let mut fields = IndexMap::new();

        fields.insert(
            "FirstName".to_string(),
            GffValue::LocString(LocalizedString {
                string_ref: -1,
                substrings: vec![LocalizedSubstring {
                    string: Cow::Owned("TestFirst".to_string()),
                    language: 0,
                    gender: 0,
                }],
            }),
        );

        fields.insert(
            "LastName".to_string(),
            GffValue::LocString(LocalizedString {
                string_ref: -1,
                substrings: vec![LocalizedSubstring {
                    string: Cow::Owned("TestLast".to_string()),
                    language: 0,
                    gender: 0,
                }],
            }),
        );

        fields.insert("Age".to_string(), GffValue::Int(30));
        fields.insert("Experience".to_string(), GffValue::Int(5000));
        fields.insert("LawfulChaotic".to_string(), GffValue::Byte(75));
        fields.insert("GoodEvil".to_string(), GffValue::Byte(80));
        fields.insert("Gender".to_string(), GffValue::Byte(0));

        fields.insert(
            "Description".to_string(),
            GffValue::LocString(LocalizedString {
                string_ref: -1,
                substrings: vec![LocalizedSubstring {
                    string: Cow::Owned("A brave hero".to_string()),
                    language: 0,
                    gender: 0,
                }],
            }),
        );

        fields.insert(
            "Deity".to_string(),
            GffValue::LocString(LocalizedString {
                string_ref: -1,
                substrings: vec![LocalizedSubstring {
                    string: Cow::Owned("".to_string()),
                    language: 0,
                    gender: 0,
                }],
            }),
        );

        Character::from_gff(fields)
    }

    fn create_character_with_background_feats(feat_ids: &[i32]) -> Character {
        let mut character = create_test_character();
        let feat_list = feat_ids
            .iter()
            .map(|feat_id| {
                let mut feat_entry = IndexMap::new();
                feat_entry.insert("Feat".to_string(), GffValue::Word(*feat_id as u16));
                feat_entry.insert(
                    "Source".to_string(),
                    GffValue::String(Cow::Owned("Background".to_string())),
                );
                feat_entry
            })
            .collect();
        character.set_list("FeatList", feat_list);
        character
    }

    fn create_game_data_with_backgrounds() -> GameData {
        let mut game_data = create_test_game_data();

        let feat_rows = vec![
            AHashMap::from([
                (
                    "label".to_string(),
                    Some("OLD_BACKGROUND_DISPLAY".to_string()),
                ),
                ("name".to_string(), Some("-1".to_string())),
            ]),
            AHashMap::from([
                (
                    "label".to_string(),
                    Some("NEW_BACKGROUND_DISPLAY".to_string()),
                ),
                ("name".to_string(), Some("-1".to_string())),
            ]),
            AHashMap::from([
                (
                    "label".to_string(),
                    Some("OLD_BACKGROUND_BONUS".to_string()),
                ),
                ("name".to_string(), Some("-1".to_string())),
            ]),
            AHashMap::from([
                (
                    "label".to_string(),
                    Some("NEW_BACKGROUND_BONUS".to_string()),
                ),
                ("name".to_string(), Some("-1".to_string())),
            ]),
            AHashMap::from([
                (
                    "label".to_string(),
                    Some("NEW_BACKGROUND_MASTER".to_string()),
                ),
                ("name".to_string(), Some("-1".to_string())),
            ]),
        ];

        let background_rows = vec![
            AHashMap::from([
                ("name".to_string(), Some("-1".to_string())),
                ("label".to_string(), Some("Old Background".to_string())),
                ("DisplayFeat".to_string(), Some("0".to_string())),
                ("FeatGained".to_string(), Some("2".to_string())),
                ("REMOVED".to_string(), Some("0".to_string())),
            ]),
            AHashMap::from([
                ("name".to_string(), Some("-1".to_string())),
                ("label".to_string(), Some("New Background".to_string())),
                ("DisplayFeat".to_string(), Some("1".to_string())),
                ("FeatGained".to_string(), Some("3".to_string())),
                ("MasterFeatGained".to_string(), Some("4".to_string())),
                ("REMOVED".to_string(), Some("0".to_string())),
            ]),
        ];

        game_data.tables.insert(
            "feat".to_string(),
            create_loaded_table("feat", &["label", "name"], feat_rows),
        );
        game_data.tables.insert(
            "backgrounds".to_string(),
            create_loaded_table(
                "backgrounds",
                &[
                    "name",
                    "label",
                    "DisplayFeat",
                    "FeatGained",
                    "MasterFeatGained",
                    "REMOVED",
                ],
                background_rows,
            ),
        );

        game_data
    }

    #[test]
    fn test_first_name() {
        let character = create_test_character();
        assert_eq!(character.first_name(), "TestFirst");
    }

    #[test]
    fn test_last_name() {
        let character = create_test_character();
        assert_eq!(character.last_name(), "TestLast");
    }

    #[test]
    fn test_full_name() {
        let character = create_test_character();
        assert_eq!(character.full_name(), "TestFirst TestLast");
    }

    #[test]
    fn test_set_first_name() {
        let mut character = create_test_character();
        character.set_first_name("NewFirst".to_string());
        assert_eq!(character.first_name(), "NewFirst");
        assert!(character.is_modified());
    }

    #[test]
    fn test_set_last_name() {
        let mut character = create_test_character();
        character.set_last_name("NewLast".to_string());
        assert_eq!(character.last_name(), "NewLast");
        assert!(character.is_modified());
    }

    #[test]
    fn test_age() {
        let character = create_test_character();
        assert_eq!(character.age(), 30);
    }

    #[test]
    fn test_set_age() {
        let mut character = create_test_character();
        let result = character.set_age(50);
        assert!(result.is_ok());
        assert_eq!(character.age(), 50);
        assert!(character.is_modified());
    }

    #[test]
    fn test_set_age_negative_error() {
        let mut character = create_test_character();
        let result = character.set_age(-5);
        assert!(result.is_err());
    }

    #[test]
    fn test_experience() {
        let character = create_test_character();
        assert_eq!(character.experience(), 5000);
    }

    #[test]
    fn test_set_experience() {
        let mut character = create_test_character();
        let result = character.set_experience(10000);
        assert!(result.is_ok());
        assert_eq!(character.experience(), 10000);
        assert!(character.is_modified());
    }

    #[test]
    fn test_set_experience_negative_error() {
        let mut character = create_test_character();
        let result = character.set_experience(-100);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_background_replaces_existing_background_feat() {
        let game_data = create_game_data_with_backgrounds();
        let mut character = create_character_with_background_feats(&[0, 2]);

        let result = character.set_background(Some(1), &game_data).unwrap();

        assert_eq!(result, Some("New Background".to_string()));
        assert!(!character.has_feat(FeatId(0)));
        assert!(!character.has_feat(FeatId(2)));
        assert!(character.has_feat(FeatId(1)));
        assert!(character.has_feat(FeatId(3)));
        assert!(character.has_feat(FeatId(4)));
        assert_eq!(
            character.background(&game_data).as_deref(),
            Some("New Background")
        );
    }

    #[test]
    fn test_set_background_none_removes_all_background_feats() {
        let game_data = create_game_data_with_backgrounds();
        let mut character = create_character_with_background_feats(&[1, 3, 4]);

        let result = character.set_background(None, &game_data).unwrap();

        assert_eq!(result, None);
        assert!(!character.has_feat(FeatId(1)));
        assert!(!character.has_feat(FeatId(3)));
        assert!(!character.has_feat(FeatId(4)));
        assert_eq!(character.background(&game_data), None);
    }

    #[test]
    fn test_background_prefers_most_specific_matching_row() {
        let mut game_data = create_game_data_with_backgrounds();
        let background_rows = vec![
            AHashMap::from([
                ("name".to_string(), Some("-1".to_string())),
                ("label".to_string(), Some("Complex".to_string())),
                ("DisplayFeat".to_string(), Some("1".to_string())),
                ("REMOVED".to_string(), Some("0".to_string())),
            ]),
            AHashMap::from([
                ("name".to_string(), Some("-1".to_string())),
                ("label".to_string(), Some("Real Background".to_string())),
                ("DisplayFeat".to_string(), Some("1".to_string())),
                ("FeatGained".to_string(), Some("3".to_string())),
                ("MasterFeatGained".to_string(), Some("4".to_string())),
                ("REMOVED".to_string(), Some("0".to_string())),
            ]),
        ];
        game_data.tables.insert(
            "backgrounds".to_string(),
            create_loaded_table(
                "backgrounds",
                &[
                    "name",
                    "label",
                    "DisplayFeat",
                    "FeatGained",
                    "MasterFeatGained",
                    "REMOVED",
                ],
                background_rows,
            ),
        );

        let character = create_character_with_background_feats(&[1, 3, 4]);

        assert_eq!(
            character.background(&game_data).as_deref(),
            Some("Real Background")
        );
    }

    #[test]
    fn test_background_ignores_removed_rows_with_shared_feats() {
        let mut game_data = create_test_game_data();

        let feat_rows = vec![
            AHashMap::from([
                ("label".to_string(), Some("ARMOR_PROF_MED".to_string())),
                ("name".to_string(), Some("-1".to_string())),
            ]),
            AHashMap::from([
                (
                    "label".to_string(),
                    Some("FEAT_BACKGROUND_NATURAL_LEADER".to_string()),
                ),
                ("name".to_string(), Some("-1".to_string())),
            ]),
        ];

        let background_rows = vec![
            AHashMap::from([
                ("name".to_string(), Some("-1".to_string())),
                ("label".to_string(), Some("Complex".to_string())),
                ("DisplayFeat".to_string(), Some("4".to_string())),
                ("REMOVED".to_string(), Some("1".to_string())),
            ]),
            AHashMap::from([
                ("name".to_string(), Some("-1".to_string())),
                ("label".to_string(), Some("NaturalLeader".to_string())),
                ("DisplayFeat".to_string(), Some("1724".to_string())),
                ("REMOVED".to_string(), Some("0".to_string())),
            ]),
        ];

        game_data.tables.insert(
            "feat".to_string(),
            create_loaded_table("feat", &["label", "name"], feat_rows),
        );
        game_data.tables.insert(
            "backgrounds".to_string(),
            create_loaded_table(
                "backgrounds",
                &["name", "label", "DisplayFeat", "REMOVED"],
                background_rows,
            ),
        );

        let mut character = create_test_character();
        let feat_list = vec![{
            let mut feat_entry = IndexMap::new();
            feat_entry.insert("Feat".to_string(), GffValue::Word(4));
            feat_entry
        }];
        character.set_list("FeatList", feat_list);

        assert_eq!(character.background(&game_data), None);
    }

    #[test]
    fn test_set_background_rejects_missing_prerequisites() {
        let mut game_data = create_test_game_data();
        let mut character = create_test_character();

        let feat_rows = vec![
            AHashMap::from([
                ("label".to_string(), Some("LOCKED_BACKGROUND".to_string())),
                ("name".to_string(), Some("-1".to_string())),
                ("PREREQFEAT1".to_string(), Some("1".to_string())),
            ]),
            AHashMap::from([
                ("label".to_string(), Some("REQUIRED_FEAT".to_string())),
                ("name".to_string(), Some("-1".to_string())),
            ]),
        ];

        let background_rows = vec![AHashMap::from([
            ("name".to_string(), Some("-1".to_string())),
            ("label".to_string(), Some("Locked Background".to_string())),
            ("DisplayFeat".to_string(), Some("0".to_string())),
            ("REMOVED".to_string(), Some("0".to_string())),
        ])];

        game_data.tables.insert(
            "feat".to_string(),
            create_loaded_table("feat", &["label", "name", "PREREQFEAT1"], feat_rows),
        );
        game_data.tables.insert(
            "backgrounds".to_string(),
            create_loaded_table(
                "backgrounds",
                &["name", "label", "DisplayFeat", "REMOVED"],
                background_rows,
            ),
        );

        let result = character.set_background(Some(0), &game_data);

        assert!(result.is_err());
        assert!(!character.has_feat(FeatId(0)));
    }

    #[test]
    fn test_alignment() {
        let character = create_test_character();
        let alignment = character.alignment();
        assert_eq!(alignment.law_chaos, 75);
        assert_eq!(alignment.good_evil, 80);
        assert_eq!(alignment.alignment_string(), "Lawful Good");
    }

    #[test]
    fn test_set_alignment() {
        let mut character = create_test_character();
        let result = character.set_alignment(Some(0), Some(0));
        assert!(result.is_ok());

        let alignment = character.alignment();
        assert_eq!(alignment.law_chaos, 0);
        assert_eq!(alignment.good_evil, 0);
        assert_eq!(alignment.alignment_string(), "Chaotic Evil");
        assert!(character.is_modified());
    }

    #[test]
    fn test_set_alignment_clamping() {
        let mut character = create_test_character();
        let result = character.set_alignment(Some(150), Some(-50));
        assert!(result.is_ok());

        let alignment = character.alignment();
        assert_eq!(alignment.law_chaos, 100);
        assert_eq!(alignment.good_evil, 0);
    }

    #[test]
    fn test_set_alignment_partial() {
        let mut character = create_test_character();
        let result = character.set_alignment(Some(25), None);
        assert!(result.is_ok());

        let alignment = character.alignment();
        assert_eq!(alignment.law_chaos, 25);
        assert_eq!(alignment.good_evil, 80);
    }

    #[test]
    fn test_deity() {
        let character = create_test_character();
        assert_eq!(character.deity(), "");
    }

    #[test]
    fn test_set_deity() {
        let mut character = create_test_character();
        character.set_deity("Tyr".to_string());
        assert_eq!(character.deity(), "Tyr");
        assert!(character.is_modified());
    }

    #[test]
    fn test_description() {
        let character = create_test_character();
        assert_eq!(character.description(), "A brave hero");
    }

    #[test]
    fn test_set_description() {
        let mut character = create_test_character();
        let new_desc = "A legendary warrior from the North".to_string();
        character.set_description(new_desc.clone());
        assert_eq!(character.description(), new_desc);
        assert!(character.is_modified());
    }

    #[test]
    fn test_gender() {
        let character = create_test_character();
        assert_eq!(character.gender(), 0);
    }

    #[test]
    fn test_set_gender() {
        let mut character = create_test_character();
        let result = character.set_gender(1);
        assert!(result.is_ok());
        assert_eq!(character.gender(), 1);
        assert!(character.is_modified());
    }

    #[test]
    fn test_set_gender_invalid() {
        let mut character = create_test_character();
        let result = character.set_gender(2);
        assert!(result.is_err());
    }

    #[test]
    fn test_alignment_is_lawful() {
        let alignment = Alignment::new(80, 50);
        assert!(alignment.is_lawful());
        assert!(!alignment.is_chaotic());
    }

    #[test]
    fn test_alignment_is_chaotic() {
        let alignment = Alignment::new(20, 50);
        assert!(alignment.is_chaotic());
        assert!(!alignment.is_lawful());
    }

    #[test]
    fn test_alignment_is_good() {
        let alignment = Alignment::new(50, 80);
        assert!(alignment.is_good());
        assert!(!alignment.is_evil());
    }

    #[test]
    fn test_alignment_is_evil() {
        let alignment = Alignment::new(50, 20);
        assert!(alignment.is_evil());
        assert!(!alignment.is_good());
    }

    #[test]
    fn test_alignment_true_neutral() {
        let alignment = Alignment::new(50, 50);
        assert_eq!(alignment.alignment_string(), "True Neutral");
        assert!(alignment.is_neutral_law_chaos());
        assert!(alignment.is_neutral_good_evil());
    }

    #[test]
    fn test_alignment_boundaries() {
        assert_eq!(Alignment::new(30, 50).alignment_string(), "Chaotic Neutral");
        assert_eq!(Alignment::new(31, 50).alignment_string(), "True Neutral");
        assert_eq!(Alignment::new(69, 50).alignment_string(), "True Neutral");
        assert_eq!(Alignment::new(70, 50).alignment_string(), "Lawful Neutral");

        assert_eq!(Alignment::new(50, 30).alignment_string(), "Neutral Evil");
        assert_eq!(Alignment::new(50, 31).alignment_string(), "True Neutral");
        assert_eq!(Alignment::new(50, 69).alignment_string(), "True Neutral");
        assert_eq!(Alignment::new(50, 70).alignment_string(), "Neutral Good");
    }

    #[test]
    fn test_biography() {
        let character = create_test_character();
        let game_data = create_test_game_data();
        let bio = character.biography(&game_data);

        assert_eq!(bio.name, "TestFirst TestLast");
        assert_eq!(bio.first_name, "TestFirst");
        assert_eq!(bio.last_name, "TestLast");
        assert_eq!(bio.age, 30);
        assert_eq!(bio.experience, 5000);
        assert_eq!(bio.gender, 0);
        assert_eq!(bio.description, "A brave hero");
        assert_eq!(bio.alignment.law_chaos, 75);
        assert_eq!(bio.alignment.good_evil, 80);
        assert_eq!(bio.alignment.alignment_string(), "Lawful Good");
    }
}
