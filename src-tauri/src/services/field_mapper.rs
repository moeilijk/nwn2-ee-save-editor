use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub static FIELD_PATTERNS: std::sync::LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            "str_adjust",
            vec![
                "str_adjust",
                "StrAdjust",
                "strength_adjust",
                "STRAdjust",
                "StrMod",
            ],
        );
        m.insert(
            "dex_adjust",
            vec![
                "dex_adjust",
                "DexAdjust",
                "dexterity_adjust",
                "DEXAdjust",
                "DexMod",
            ],
        );
        m.insert(
            "con_adjust",
            vec![
                "con_adjust",
                "ConAdjust",
                "constitution_adjust",
                "CONAdjust",
                "ConMod",
            ],
        );
        m.insert(
            "int_adjust",
            vec![
                "int_adjust",
                "IntAdjust",
                "intelligence_adjust",
                "INTAdjust",
                "IntMod",
            ],
        );
        m.insert(
            "wis_adjust",
            vec![
                "wis_adjust",
                "WisAdjust",
                "wisdom_adjust",
                "WISAdjust",
                "WisMod",
            ],
        );
        m.insert(
            "cha_adjust",
            vec![
                "cha_adjust",
                "ChaAdjust",
                "charisma_adjust",
                "CHAAdjust",
                "ChaMod",
            ],
        );
        m.insert(
            "fort_save",
            vec![
                "fort_save",
                "fortitude_save",
                "FortSave",
                "Fort",
                "FortitudeBonus",
                "FortBonus",
            ],
        );
        m.insert(
            "ref_save",
            vec![
                "ref_save",
                "reflex_save",
                "RefSave",
                "Ref",
                "ReflexBonus",
                "RefBonus",
            ],
        );
        m.insert(
            "will_save",
            vec!["will_save", "WillSave", "Will", "WillBonus"],
        );
        m.insert(
            "creature_size",
            vec!["creature_size", "size", "CreatureSize", "Size", "RaceSize"],
        );
        m.insert(
            "movement_rate",
            vec![
                "movement_rate",
                "base_speed",
                "speed",
                "MovementRate",
                "BaseSpeed",
                "Speed",
                "Endurance",
            ],
        );
        m.insert(
            "ac_attack_mod",
            vec![
                "ACATTACKMOD",
                "ac_attack_mod",
                "AcAttackMod",
                "ACAttackMod",
                "ACMod",
            ],
        );
        m.insert("label", vec!["LABEL", "label", "Label"]);
        m.insert(
            "name",
            vec!["NAME", "name", "Name", "label", "Label", "NameRef"],
        );
        m.insert("feat_name_strref", vec!["FEAT", "Feat"]);
        m.insert(
            "key_ability",
            vec!["KeyAbility", "key_ability", "keyability", "KEYABILITY"],
        );
        m.insert(
            "skill_index",
            vec![
                "SkillIndex",
                "skill_index",
                "skillindex",
                "SKILLINDEX",
                "Skill",
            ],
        );
        m.insert(
            "class_skill",
            vec![
                "ClassSkill",
                "class_skill",
                "classskill",
                "CLASSSKILL",
                "IsClassSkill",
            ],
        );
        m.insert(
            "prereq_str",
            vec!["prereq_str", "PreReqStr", "MinStr", "min_str", "ReqStr"],
        );
        m.insert(
            "prereq_dex",
            vec!["prereq_dex", "PreReqDex", "MinDex", "min_dex", "ReqDex"],
        );
        m.insert(
            "prereq_con",
            vec!["prereq_con", "PreReqCon", "MinCon", "min_con", "ReqCon"],
        );
        m.insert(
            "prereq_int",
            vec!["prereq_int", "PreReqInt", "MinInt", "min_int", "ReqInt"],
        );
        m.insert(
            "prereq_wis",
            vec!["prereq_wis", "PreReqWis", "MinWis", "min_wis", "ReqWis"],
        );
        m.insert(
            "prereq_cha",
            vec!["prereq_cha", "PreReqCha", "MinCha", "min_cha", "ReqCha"],
        );
        m.insert(
            "prereq_feat1",
            vec![
                "prereq_feat1",
                "PreReqFeat1",
                "ReqFeat1",
                "prereqfeat1",
                "PREREQFEAT1",
            ],
        );
        m.insert(
            "prereq_feat2",
            vec![
                "prereq_feat2",
                "PreReqFeat2",
                "ReqFeat2",
                "prereqfeat2",
                "PREREQFEAT2",
            ],
        );
        m.insert(
            "prereq_bab",
            vec![
                "prereq_bab",
                "PreReqBAB",
                "MinAttackBonus",
                "MinBAB",
                "ReqBAB",
            ],
        );
        m.insert(
            "required_class",
            vec![
                "required_class",
                "reqclass",
                "ReqClass",
                "ClassReq",
                "MinLevelClass",
            ],
        );
        m.insert(
            "min_level",
            vec!["min_level", "minlevel", "MinLevel", "LevelReq", "ReqLevel"],
        );
        m.insert(
            "prereq_spell_level",
            vec![
                "prereq_spell_level",
                "MinSpell",
                "SpellLevel",
                "ReqSpellLevel",
            ],
        );
        m.insert(
            "favored_class",
            vec![
                "favored_class",
                "FavoredClass",
                "favoured_class",
                "FavouredClass",
            ],
        );
        m.insert(
            "feat_index",
            vec!["FeatIndex", "feat_index", "featindex", "feat_id"],
        );
        m.insert(
            "granted_on_level",
            vec![
                "GrantedOnLevel",
                "granted_on_level",
                "grantedlevel",
                "level",
            ],
        );
        m.insert(
            "racial_feats",
            vec![
                "racial_feats",
                "feats",
                "special_abilities",
                "RacialFeats",
                "Feats",
            ],
        );
        m.insert("feat0", vec!["Feat0", "feat0", "Feat", "feat"]);
        m.insert("feat1", vec!["Feat1", "feat1"]);
        m.insert("feat2", vec!["Feat2", "feat2"]);
        m.insert("feat3", vec!["Feat3", "feat3"]);
        m.insert("feat4", vec!["Feat4", "feat4"]);
        m.insert("feat5", vec!["Feat5", "feat5"]);
        m.insert(
            "attack_bonus_table",
            vec![
                "AttackBonusTable",
                "attack_bonus_table",
                "AttackTable",
                "BABTable",
            ],
        );
        m.insert(
            "saving_throw_table",
            vec![
                "SavingThrowTable",
                "saving_throw_table",
                "SaveTable",
                "SavTable",
            ],
        );
        m.insert(
            "skills_table",
            vec!["SkillsTable", "skills_table", "SkillTable"],
        );
        m.insert(
            "feats_table",
            vec![
                "FeatsTable",
                "feats_table",
                "FeatTable",
                "featstable",
                "FEATSTABLE",
            ],
        );
        m.insert(
            "bonus_feats_table",
            vec![
                "BonusFeatsTable",
                "bonus_feats_table",
                "BonusFeatTable",
                "bonus_feat_table",
            ],
        );
        m.insert("hit_die", vec!["HitDie", "hit_die", "HD", "HitDice"]);
        m.insert(
            "skill_point_base",
            vec!["SkillPointBase", "skill_point_base", "SkillPoints", "SP"],
        );
        m.insert(
            "max_level",
            vec!["MaxLevel", "max_level", "max_lvl", "MaxLvl"],
        );
        m.insert(
            "has_arcane",
            vec!["HasArcane", "has_arcane", "arcane", "Arcane"],
        );
        m.insert(
            "has_divine",
            vec!["HasDivine", "has_divine", "divine", "Divine"],
        );
        m.insert(
            "primary_ability",
            vec!["PrimaryAbil", "primary_ability", "primary_abil", "PrimAbil"],
        );
        m.insert("bab", vec!["BAB", "bab", "AttackBonus", "BaseAttack"]);
        m.insert(
            "fort_save_table",
            vec!["FortSave", "fort_save", "fort", "Fort", "FortitudeBonus"],
        );
        m.insert(
            "ref_save_table",
            vec!["RefSave", "ref_save", "ref", "Ref", "ReflexBonus"],
        );
        m.insert(
            "will_save_table",
            vec!["WillSave", "will_save", "will", "Will", "WillBonus"],
        );
        m.insert(
            "spell_caster",
            vec!["SpellCaster", "spell_caster", "IsCaster", "Caster"],
        );
        m.insert(
            "prereq_table",
            vec![
                "PreReqTable",
                "prereq_table",
                "prereqtable",
                "PrerequisiteTable",
            ],
        );
        m.insert(
            "spell_gain_table",
            vec!["spell_gain_table", "SpellGainTable", "SpellTable"],
        );
        m.insert(
            "spell_known_table",
            vec!["spell_known_table", "SpellKnownTable", "KnownTable"],
        );
        m.insert(
            "align_restrict",
            vec!["align_restrict", "AlignRestrict", "AlignmentRestrict"],
        );
        m.insert(
            "align_restrict_type",
            vec!["align_restrict_type", "AlignRstrctType", "AlignmentType"],
        );
        m.insert(
            "player_race",
            vec!["player_race", "PlayerRace", "PCRace", "Playable"],
        );
        m.insert(
            "player_class",
            vec!["player_class", "PlayerClass", "PCClass", "Playable"],
        );
        m.insert(
            "icon",
            vec!["ICON", "icon", "Icon", "IconResRef", "IconRef"],
        );
        m.insert(
            "bordered_icon",
            vec!["bordered_icon", "BorderedIcon", "IconBordered"],
        );
        m.insert("damage_type", vec!["damage_type", "DamageType", "DmgType"]);
        m.insert("damage_die", vec!["damage_die", "DamageDie", "DmgDie"]);
        m.insert(
            "crit_threat",
            vec!["crit_threat", "CritThreat", "ThreatRange"],
        );
        m.insert("crit_mult", vec!["crit_mult", "CritMult", "CritMultiplier"]);
        m.insert(
            "base_item",
            vec!["base_item", "BaseItem", "ItemType", "Type"],
        );
        m.insert("item_class", vec!["item_class", "ItemClass", "Class"]);
        m.insert("weapon_type", vec!["weapon_type", "WeaponType", "WpnType"]);
        m.insert("cost", vec!["cost", "Cost", "Price", "Value"]);
        m.insert("weight", vec!["weight", "Weight", "Wt"]);
        m.insert(
            "base_race",
            vec!["base_race", "BaseRace", "baserace", "BASERACE"],
        );
        m.insert("subrace_name", vec!["Name", "name", "Label", "label"]);
        m.insert("subrace_label", vec!["Label", "label", "Name", "name"]);
        m.insert(
            "effective_character_level",
            vec![
                "ecl",
                "ECL",
                "effective_character_level",
                "EffectiveCharacterLevel",
            ],
        );
        m.insert(
            "has_favored_class",
            vec![
                "has_favored_class",
                "HasFavoredClass",
                "hasfavoredclass",
                "HASFAVOREDCLASS",
            ],
        );
        m.insert(
            "description",
            vec![
                "DESCRIPTION",
                "description",
                "Description",
                "Desc",
                "DescRef",
            ],
        );
        m.insert("category", vec!["category", "Category", "Cat", "Type"]);
        m.insert(
            "type",
            vec!["CATEGORY", "FeatCategory", "Category", "Type", "category"],
        );
        m.insert(
            "constant",
            vec!["constant", "Constant", "Const", "ConstantValue"],
        );
        m.insert("bonus", vec!["Bonus", "bonus", "BONUS"]);
        m
    });

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbilityModifiers {
    pub str_mod: i32,
    pub dex_mod: i32,
    pub con_mod: i32,
    pub int_mod: i32,
    pub wis_mod: i32,
    pub cha_mod: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveBonuses {
    pub fortitude: i32,
    pub reflex: i32,
    pub will: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatPrerequisites {
    pub abilities: HashMap<String, i32>,
    pub feats: Vec<i32>,
    pub required_class: i32,
    pub min_level: i32,
    pub bab: i32,
    pub spell_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassProperties {
    pub label: String,
    pub name: String,
    pub hit_die: i32,
    pub skill_points: i32,
    pub max_level: i32,
    pub attack_bonus_table: String,
    pub saving_throw_table: String,
    pub skills_table: String,
    pub feats_table: String,
    pub spell_caster: bool,
    pub has_arcane: bool,
    pub has_divine: bool,
    pub spell_gain_table: String,
    pub spell_known_table: String,
    pub primary_ability: String,
    pub align_restrict: i32,
    pub prereq_table: String,
    pub player_class: bool,
    pub description: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrereqTableParsed {
    pub table_type: Option<String>,
    pub identifier: Option<String>,
    pub raw_table: String,
}

use ahash::AHashMap;

pub struct FieldMapper {
    reverse_map: AHashMap<String, String>,
}

impl Default for FieldMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldMapper {
    pub fn new() -> Self {
        let mut reverse_map = AHashMap::new();
        for (pattern, aliases) in FIELD_PATTERNS.iter() {
            for alias in aliases {
                reverse_map
                    .entry(alias.to_lowercase())
                    .or_insert_with(|| (*pattern).to_string());
            }
        }
        Self { reverse_map }
    }

    pub fn get_field_value(
        &self,
        data: &AHashMap<String, Option<String>>,
        pattern: &str,
    ) -> Option<String> {
        let fallback: &[&str] = &[pattern];
        let aliases = FIELD_PATTERNS.get(pattern).map_or(fallback, Vec::as_slice);

        for alias in aliases {
            if let Some(Some(value)) = data.get(*alias) {
                let trimmed = value.trim();
                if !trimmed.is_empty() && trimmed != "****" {
                    return Some(trimmed.to_string());
                }
            }
            let lower = alias.to_lowercase();
            if let Some(Some(value)) = data.get(&lower) {
                let trimmed = value.trim();
                if !trimmed.is_empty() && trimmed != "****" {
                    return Some(trimmed.to_string());
                }
            }
            let upper = alias.to_uppercase();
            if let Some(Some(value)) = data.get(&upper) {
                let trimmed = value.trim();
                if !trimmed.is_empty() && trimmed != "****" {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    pub fn safe_int(value: Option<&str>) -> i32 {
        Self::safe_int_with_default(value, 0)
    }

    pub fn safe_int_with_default(value: Option<&str>, default: i32) -> i32 {
        match value {
            None => default,
            Some(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() || trimmed == "****" {
                    return default;
                }
                if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                    i32::from_str_radix(&trimmed[2..], 16).unwrap_or(default)
                } else {
                    trimmed.parse().unwrap_or(default)
                }
            }
        }
    }

    pub fn safe_bool(value: Option<&str>) -> bool {
        Self::safe_bool_with_default(value, false)
    }

    pub fn safe_bool_with_default(value: Option<&str>, default: bool) -> bool {
        match value {
            None => default,
            Some(s) => {
                let trimmed = s.trim().to_lowercase();
                if trimmed.is_empty() || trimmed == "****" {
                    return default;
                }
                match trimmed.as_str() {
                    "true" | "yes" | "1" => true,
                    "false" | "no" | "0" => false,
                    _ => trimmed.parse::<i32>().map(|v| v != 0).unwrap_or(default),
                }
            }
        }
    }

    pub fn safe_hex_int(value: Option<&str>) -> i32 {
        Self::safe_hex_int_with_default(value, 0)
    }

    pub fn safe_hex_int_with_default(value: Option<&str>, default: i32) -> i32 {
        match value {
            None => default,
            Some(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() || trimmed == "****" {
                    return default;
                }
                if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                    i32::from_str_radix(&trimmed[2..], 16).unwrap_or(default)
                } else {
                    Self::safe_int_with_default(Some(trimmed), default)
                }
            }
        }
    }

    pub fn get_ability_modifiers(
        &self,
        data: &AHashMap<String, Option<String>>,
    ) -> AbilityModifiers {
        AbilityModifiers {
            str_mod: Self::safe_int(self.get_field_value(data, "str_adjust").as_deref()),
            dex_mod: Self::safe_int(self.get_field_value(data, "dex_adjust").as_deref()),
            con_mod: Self::safe_int(self.get_field_value(data, "con_adjust").as_deref()),
            int_mod: Self::safe_int(self.get_field_value(data, "int_adjust").as_deref()),
            wis_mod: Self::safe_int(self.get_field_value(data, "wis_adjust").as_deref()),
            cha_mod: Self::safe_int(self.get_field_value(data, "cha_adjust").as_deref()),
        }
    }

    pub fn get_racial_saves(&self, data: &AHashMap<String, Option<String>>) -> SaveBonuses {
        SaveBonuses {
            fortitude: Self::safe_int(self.get_field_value(data, "fort_save").as_deref()),
            reflex: Self::safe_int(self.get_field_value(data, "ref_save").as_deref()),
            will: Self::safe_int(self.get_field_value(data, "will_save").as_deref()),
        }
    }

    pub fn get_feat_prerequisites(
        &self,
        data: &AHashMap<String, Option<String>>,
    ) -> FeatPrerequisites {
        let mut prereqs = FeatPrerequisites::default();

        for ability in ["str", "dex", "con", "int", "wis", "cha"] {
            let pattern = format!("prereq_{ability}");
            let value = Self::safe_int(self.get_field_value(data, &pattern).as_deref());
            prereqs.abilities.insert(ability.to_uppercase(), value);
        }

        for i in 1..=2 {
            let pattern = format!("prereq_feat{i}");
            let feat_req = Self::safe_int(self.get_field_value(data, &pattern).as_deref());
            if feat_req > 0 {
                prereqs.feats.push(feat_req);
            }
        }

        prereqs.required_class = Self::safe_int_with_default(
            self.get_field_value(data, "required_class").as_deref(),
            -1,
        );
        prereqs.min_level = Self::safe_int(self.get_field_value(data, "min_level").as_deref());
        prereqs.bab = Self::safe_int(self.get_field_value(data, "prereq_bab").as_deref());
        prereqs.spell_level =
            Self::safe_int(self.get_field_value(data, "prereq_spell_level").as_deref());

        prereqs
    }

    pub fn get_racial_feats(&self, data: &AHashMap<String, Option<String>>) -> Vec<i32> {
        let mut feats = Vec::new();

        if let Some(racial_feats) = self.get_field_value(data, "racial_feats") {
            for feat_str in racial_feats.split(',') {
                let feat_id = Self::safe_int(Some(feat_str.trim()));
                if feat_id > 0 && !feats.contains(&feat_id) {
                    feats.push(feat_id);
                }
            }
        }

        for i in 0..10 {
            let pattern = format!("feat{i}");
            let feat_id = Self::safe_int(self.get_field_value(data, &pattern).as_deref());
            if feat_id > 0 && !feats.contains(&feat_id) {
                feats.push(feat_id);
            }
        }

        feats
    }

    pub fn get_class_properties(&self, data: &AHashMap<String, Option<String>>) -> ClassProperties {
        let prereq_table = self
            .get_field_value(data, "prereq_table")
            .unwrap_or_default();

        ClassProperties {
            label: self.get_field_value(data, "label").unwrap_or_default(),
            name: self.get_field_value(data, "name").unwrap_or_default(),
            hit_die: Self::safe_int_with_default(
                self.get_field_value(data, "hit_die").as_deref(),
                8,
            ),
            skill_points: Self::safe_int_with_default(
                self.get_field_value(data, "skill_point_base").as_deref(),
                2,
            ),
            max_level: Self::safe_int_with_default(
                self.get_field_value(data, "max_level").as_deref(),
                20,
            ),
            attack_bonus_table: self
                .get_field_value(data, "attack_bonus_table")
                .unwrap_or_default(),
            saving_throw_table: self
                .get_field_value(data, "saving_throw_table")
                .unwrap_or_default(),
            skills_table: self
                .get_field_value(data, "skills_table")
                .unwrap_or_default(),
            feats_table: self
                .get_field_value(data, "feats_table")
                .unwrap_or_default(),
            spell_caster: Self::safe_bool(self.get_field_value(data, "spell_caster").as_deref()),
            has_arcane: Self::safe_bool(self.get_field_value(data, "has_arcane").as_deref()),
            has_divine: Self::safe_bool(self.get_field_value(data, "has_divine").as_deref()),
            spell_gain_table: self
                .get_field_value(data, "spell_gain_table")
                .unwrap_or_default(),
            spell_known_table: self
                .get_field_value(data, "spell_known_table")
                .unwrap_or_default(),
            primary_ability: self
                .get_field_value(data, "primary_ability")
                .unwrap_or_default(),
            align_restrict: Self::safe_hex_int(
                self.get_field_value(data, "align_restrict").as_deref(),
            ),
            prereq_table: prereq_table.clone(),
            player_class: Self::safe_bool_with_default(
                self.get_field_value(data, "player_class").as_deref(),
                true,
            ),
            description: Self::safe_int(self.get_field_value(data, "description").as_deref()),
        }
    }

    pub fn parse_prerequisite_table(prereq_table_name: &str) -> PrereqTableParsed {
        if prereq_table_name.is_empty() || prereq_table_name == "****" {
            return PrereqTableParsed::default();
        }

        let parts: Vec<&str> = prereq_table_name.split('_').collect();
        if parts.len() < 3 || parts[0] != "CLS" || parts[1] != "PRES" {
            return PrereqTableParsed {
                raw_table: prereq_table_name.to_string(),
                ..Default::default()
            };
        }

        let identifier = parts[2..].join("_");

        PrereqTableParsed {
            table_type: Some("class_prerequisites".to_string()),
            identifier: Some(identifier),
            raw_table: prereq_table_name.to_string(),
        }
    }

    pub fn get_canonical_field(&self, alias: &str) -> Option<&str> {
        self.reverse_map
            .get(&alias.to_lowercase())
            .map(std::string::String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_int() {
        assert_eq!(FieldMapper::safe_int(Some("42")), 42);
        assert_eq!(FieldMapper::safe_int(Some("****")), 0);
        assert_eq!(FieldMapper::safe_int(Some("")), 0);
        assert_eq!(FieldMapper::safe_int(None), 0);
        assert_eq!(FieldMapper::safe_int(Some("0x10")), 16);
    }

    #[test]
    fn test_safe_bool() {
        assert!(FieldMapper::safe_bool(Some("1")));
        assert!(FieldMapper::safe_bool(Some("true")));
        assert!(FieldMapper::safe_bool(Some("yes")));
        assert!(!FieldMapper::safe_bool(Some("0")));
        assert!(!FieldMapper::safe_bool(Some("false")));
        assert!(!FieldMapper::safe_bool(Some("****")));
    }

    #[test]
    fn test_parse_prerequisite_table() {
        let result = FieldMapper::parse_prerequisite_table("CLS_PRES_FIGHTER");
        assert_eq!(result.table_type, Some("class_prerequisites".to_string()));
        assert_eq!(result.identifier, Some("FIGHTER".to_string()));

        let result = FieldMapper::parse_prerequisite_table("****");
        assert!(result.table_type.is_none());
    }

    #[test]
    fn test_field_patterns() {
        assert!(FIELD_PATTERNS.contains_key("str_adjust"));
        assert!(
            FIELD_PATTERNS
                .get("str_adjust")
                .unwrap()
                .contains(&"StrAdjust")
        );
    }
}
