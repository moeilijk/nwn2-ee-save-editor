use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnPurpose {
    MinLevel,
    MinClassLevel,
    GrantedLevel,
    MinStr,
    MinDex,
    MinCon,
    MinInt,
    MinWis,
    MinCha,
    PrereqFeat,
    OrPrereqFeat,
    AlignmentRestrict,
    SpellLevel,
    SpellId,
    FeatIndex,
    ClassId,
    SkillIndex,
    StrRef,
    Label,
    Name,
    Description,
    FeatsTable,
    SkillsTable,
    SpellsTable,
    FavoredClass,
    WeaponType,
    BaseItem,
    DomainId,
    SchoolId,
    ReqSkill,
    ReqSkillRanks,
    Icon,
    Unknown,
}

impl ColumnPurpose {
    pub fn is_requirement(&self) -> bool {
        matches!(
            self,
            ColumnPurpose::MinLevel
                | ColumnPurpose::MinClassLevel
                | ColumnPurpose::GrantedLevel
                | ColumnPurpose::MinStr
                | ColumnPurpose::MinDex
                | ColumnPurpose::MinCon
                | ColumnPurpose::MinInt
                | ColumnPurpose::MinWis
                | ColumnPurpose::MinCha
                | ColumnPurpose::PrereqFeat
                | ColumnPurpose::OrPrereqFeat
                | ColumnPurpose::AlignmentRestrict
                | ColumnPurpose::SpellLevel
                | ColumnPurpose::ReqSkill
                | ColumnPurpose::ReqSkillRanks
        )
    }

    pub fn is_reference(&self) -> bool {
        matches!(
            self,
            ColumnPurpose::SpellId
                | ColumnPurpose::FeatIndex
                | ColumnPurpose::ClassId
                | ColumnPurpose::SkillIndex
                | ColumnPurpose::FavoredClass
                | ColumnPurpose::WeaponType
                | ColumnPurpose::BaseItem
                | ColumnPurpose::DomainId
                | ColumnPurpose::SchoolId
        )
    }

    pub fn target_table(&self) -> Option<&'static str> {
        match self {
            ColumnPurpose::SpellId => Some("spells"),
            ColumnPurpose::FeatIndex | ColumnPurpose::PrereqFeat | ColumnPurpose::OrPrereqFeat => {
                Some("feat")
            }
            ColumnPurpose::ClassId | ColumnPurpose::FavoredClass => Some("classes"),
            ColumnPurpose::SkillIndex | ColumnPurpose::ReqSkill => Some("skills"),
            ColumnPurpose::BaseItem | ColumnPurpose::WeaponType => Some("baseitems"),
            ColumnPurpose::DomainId => Some("domains"),
            ColumnPurpose::SchoolId => Some("spellschools"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableType {
    ClassFeatProgression,
    ClassSkillList,
    ClassSaves,
    ClassPrerequisites,
    ClassSpellGain,
    ClassSpellKnown,
    ClassAttack,
    RaceFeat,
    ItemProperty,
    Generic,
}

static MIN_CLASS_LEVEL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)MINLEVELCLASS").unwrap());
static GRANTED_LEVEL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)GRANTEDONLEVEL").unwrap());
static MIN_LEVEL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(MIN)?LEVEL$").unwrap());
static MIN_STR_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^MINSTR$").unwrap());
static MIN_DEX_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^MINDEX$").unwrap());
static MIN_CON_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^MINCON$").unwrap());
static MIN_INT_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^MININT$").unwrap());
static MIN_WIS_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^MINWIS$").unwrap());
static MIN_CHA_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^MINCHA$").unwrap());
static OR_PREREQ_FEAT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)ORFEAT\d+$").unwrap());
static PREREQ_FEAT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(PREREQ|REQ)?FEAT\d*$").unwrap());
static ALIGNMENT_RESTRICT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(ALIGN|ALIGNMENT)").unwrap());
static SPELL_LEVEL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(INNATE|SPELL)?LVL$").unwrap());

static SPELL_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^SPELL(ID)?$").unwrap());
static FEAT_INDEX_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^FEAT(INDEX)?$").unwrap());
static CLASS_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^CLASS(ID)?$").unwrap());
static SKILL_INDEX_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^SKILL(INDEX)?$").unwrap());
static STR_REF_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(STRREF|STRINGREF)").unwrap());
static LABEL_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^LABEL$").unwrap());
static NAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^NAME$").unwrap());
static DESCRIPTION_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(DESC|DESCRIPTION)$").unwrap());
static FEATS_TABLE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)FEAT(S)?TABLE").unwrap());
static SKILLS_TABLE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)SKILL(S)?TABLE").unwrap());
static SPELLS_TABLE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)SPELL(S)?TABLE").unwrap());
static FAVORED_CLASS_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)FAVORED(CLASS)?").unwrap());
static WEAPON_TYPE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)WEAPON(TYPE)?").unwrap());
static BASE_ITEM_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)BASEITEM").unwrap());
static DOMAIN_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^DOMAIN(ID)?$").unwrap());
static SCHOOL_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^SCHOOL(ID)?$").unwrap());
static REQ_SKILL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^REQSKILL\d*$").unwrap());
static REQ_SKILL_RANKS_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)REQSKILLMINRANKS").unwrap());
static ICON_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^ICON$").unwrap());

static TABLE_TYPE_CLASS_FEAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^cls_feat_").unwrap());
static TABLE_TYPE_CLASS_SKILL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^cls_skill_").unwrap());
static TABLE_TYPE_CLASS_SAVE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^cls_savthr_").unwrap());
static TABLE_TYPE_CLASS_PRES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^cls_pres_").unwrap());
static TABLE_TYPE_CLASS_SPGN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^cls_spgn_").unwrap());
static TABLE_TYPE_CLASS_SPKN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^cls_spkn_").unwrap());
static TABLE_TYPE_CLASS_ATK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^cls_atk_").unwrap());
static TABLE_TYPE_RACE_FEAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^race_feat_").unwrap());
static TABLE_TYPE_IPRP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^iprp_").unwrap());

pub const NULL_VALUE: &str = "****";

#[derive(Debug, Clone)]
pub struct RuleDetector {
    cache: HashMap<String, HashMap<String, ColumnPurpose>>,
}

impl RuleDetector {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn detect_table_type(&self, table_name: &str) -> TableType {
        if TABLE_TYPE_CLASS_FEAT.is_match(table_name) {
            TableType::ClassFeatProgression
        } else if TABLE_TYPE_CLASS_SKILL.is_match(table_name) {
            TableType::ClassSkillList
        } else if TABLE_TYPE_CLASS_SAVE.is_match(table_name) {
            TableType::ClassSaves
        } else if TABLE_TYPE_CLASS_PRES.is_match(table_name) {
            TableType::ClassPrerequisites
        } else if TABLE_TYPE_CLASS_SPGN.is_match(table_name) {
            TableType::ClassSpellGain
        } else if TABLE_TYPE_CLASS_SPKN.is_match(table_name) {
            TableType::ClassSpellKnown
        } else if TABLE_TYPE_CLASS_ATK.is_match(table_name) {
            TableType::ClassAttack
        } else if TABLE_TYPE_RACE_FEAT.is_match(table_name) {
            TableType::RaceFeat
        } else if TABLE_TYPE_IPRP.is_match(table_name) {
            TableType::ItemProperty
        } else {
            TableType::Generic
        }
    }

    pub fn detect_column_purpose(&self, column_name: &str) -> ColumnPurpose {
        if MIN_CLASS_LEVEL_PATTERN.is_match(column_name) {
            ColumnPurpose::MinClassLevel
        } else if GRANTED_LEVEL_PATTERN.is_match(column_name) {
            ColumnPurpose::GrantedLevel
        } else if MIN_LEVEL_PATTERN.is_match(column_name) {
            ColumnPurpose::MinLevel
        } else if MIN_STR_PATTERN.is_match(column_name) {
            ColumnPurpose::MinStr
        } else if MIN_DEX_PATTERN.is_match(column_name) {
            ColumnPurpose::MinDex
        } else if MIN_CON_PATTERN.is_match(column_name) {
            ColumnPurpose::MinCon
        } else if MIN_INT_PATTERN.is_match(column_name) {
            ColumnPurpose::MinInt
        } else if MIN_WIS_PATTERN.is_match(column_name) {
            ColumnPurpose::MinWis
        } else if MIN_CHA_PATTERN.is_match(column_name) {
            ColumnPurpose::MinCha
        } else if OR_PREREQ_FEAT_PATTERN.is_match(column_name) {
            ColumnPurpose::OrPrereqFeat
        } else if PREREQ_FEAT_PATTERN.is_match(column_name) {
            ColumnPurpose::PrereqFeat
        } else if ALIGNMENT_RESTRICT_PATTERN.is_match(column_name) {
            ColumnPurpose::AlignmentRestrict
        } else if SPELL_LEVEL_PATTERN.is_match(column_name) {
            ColumnPurpose::SpellLevel
        } else if SPELL_ID_PATTERN.is_match(column_name) {
            ColumnPurpose::SpellId
        } else if FEAT_INDEX_PATTERN.is_match(column_name) {
            ColumnPurpose::FeatIndex
        } else if CLASS_ID_PATTERN.is_match(column_name) {
            ColumnPurpose::ClassId
        } else if SKILL_INDEX_PATTERN.is_match(column_name) {
            ColumnPurpose::SkillIndex
        } else if STR_REF_PATTERN.is_match(column_name) {
            ColumnPurpose::StrRef
        } else if LABEL_PATTERN.is_match(column_name) {
            ColumnPurpose::Label
        } else if NAME_PATTERN.is_match(column_name) {
            ColumnPurpose::Name
        } else if DESCRIPTION_PATTERN.is_match(column_name) {
            ColumnPurpose::Description
        } else if FEATS_TABLE_PATTERN.is_match(column_name) {
            ColumnPurpose::FeatsTable
        } else if SKILLS_TABLE_PATTERN.is_match(column_name) {
            ColumnPurpose::SkillsTable
        } else if SPELLS_TABLE_PATTERN.is_match(column_name) {
            ColumnPurpose::SpellsTable
        } else if FAVORED_CLASS_PATTERN.is_match(column_name) {
            ColumnPurpose::FavoredClass
        } else if WEAPON_TYPE_PATTERN.is_match(column_name) {
            ColumnPurpose::WeaponType
        } else if BASE_ITEM_PATTERN.is_match(column_name) {
            ColumnPurpose::BaseItem
        } else if DOMAIN_ID_PATTERN.is_match(column_name) {
            ColumnPurpose::DomainId
        } else if SCHOOL_ID_PATTERN.is_match(column_name) {
            ColumnPurpose::SchoolId
        } else if REQ_SKILL_RANKS_PATTERN.is_match(column_name) {
            ColumnPurpose::ReqSkillRanks
        } else if REQ_SKILL_PATTERN.is_match(column_name) {
            ColumnPurpose::ReqSkill
        } else if ICON_PATTERN.is_match(column_name) {
            ColumnPurpose::Icon
        } else {
            ColumnPurpose::Unknown
        }
    }

    pub fn analyze_columns(&mut self, table_name: &str, columns: &[String]) {
        if self.cache.contains_key(table_name) {
            return;
        }

        let mut column_map = HashMap::new();
        for column in columns {
            let purpose = self.detect_column_purpose(column);
            column_map.insert(column.clone(), purpose);
        }
        self.cache.insert(table_name.to_string(), column_map);
    }

    pub fn get_column_purpose(&self, table_name: &str, column_name: &str) -> Option<ColumnPurpose> {
        self.cache
            .get(table_name)
            .and_then(|columns| columns.get(column_name).copied())
    }

    pub fn detect_relationships(
        &self,
        table_name: &str,
        columns: &[String],
    ) -> Vec<(String, String, String)> {
        let mut relationships = Vec::new();

        for column in columns {
            let purpose = self.detect_column_purpose(column);
            if let Some(target) = purpose.target_table() {
                relationships.push((
                    table_name.to_string(),
                    column.clone(),
                    target.to_string(),
                ));
            }

            if column.to_lowercase().ends_with("table") {
                relationships.push((
                    table_name.to_string(),
                    column.clone(),
                    "dynamic".to_string(),
                ));
            }
        }

        relationships
    }

    pub fn is_null_value(value: Option<&str>) -> bool {
        match value {
            None => true,
            Some(v) => v == NULL_VALUE || v.is_empty(),
        }
    }

    pub fn parse_int(value: Option<&str>) -> Option<i32> {
        let v = value?;
        if Self::is_null_value(Some(v)) {
            return None;
        }

        if let Some(hex) = v.strip_prefix("0x") {
            i32::from_str_radix(hex, 16).ok()
        } else {
            v.parse().ok()
        }
    }
}

impl Default for RuleDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_column_purpose() {
        let detector = RuleDetector::new();

        assert_eq!(
            detector.detect_column_purpose("MINLEVEL"),
            ColumnPurpose::MinLevel
        );
        assert_eq!(
            detector.detect_column_purpose("MINSTR"),
            ColumnPurpose::MinStr
        );
        assert_eq!(
            detector.detect_column_purpose("PREREQFEAT1"),
            ColumnPurpose::PrereqFeat
        );
        assert_eq!(
            detector.detect_column_purpose("ORFEAT1"),
            ColumnPurpose::OrPrereqFeat
        );
        assert_eq!(
            detector.detect_column_purpose("SPELLID"),
            ColumnPurpose::SpellId
        );
        assert_eq!(
            detector.detect_column_purpose("FEATINDEX"),
            ColumnPurpose::FeatIndex
        );
        assert_eq!(
            detector.detect_column_purpose("LABEL"),
            ColumnPurpose::Label
        );
        assert_eq!(
            detector.detect_column_purpose("STRREF"),
            ColumnPurpose::StrRef
        );
    }

    #[test]
    fn test_detect_table_type() {
        let detector = RuleDetector::new();

        assert_eq!(
            detector.detect_table_type("cls_feat_fighter"),
            TableType::ClassFeatProgression
        );
        assert_eq!(
            detector.detect_table_type("cls_skill_bard"),
            TableType::ClassSkillList
        );
        assert_eq!(
            detector.detect_table_type("cls_pres_duelist"),
            TableType::ClassPrerequisites
        );
        assert_eq!(
            detector.detect_table_type("race_feat_elf"),
            TableType::RaceFeat
        );
        assert_eq!(
            detector.detect_table_type("iprp_abilities"),
            TableType::ItemProperty
        );
        assert_eq!(detector.detect_table_type("classes"), TableType::Generic);
    }

    #[test]
    fn test_is_null_value() {
        assert!(RuleDetector::is_null_value(None));
        assert!(RuleDetector::is_null_value(Some("****")));
        assert!(RuleDetector::is_null_value(Some("")));
        assert!(!RuleDetector::is_null_value(Some("10")));
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(RuleDetector::parse_int(Some("42")), Some(42));
        assert_eq!(RuleDetector::parse_int(Some("0x10")), Some(16));
        assert_eq!(RuleDetector::parse_int(Some("****")), None);
        assert_eq!(RuleDetector::parse_int(None), None);
    }

    #[test]
    fn test_column_purpose_target_table() {
        assert_eq!(ColumnPurpose::SpellId.target_table(), Some("spells"));
        assert_eq!(ColumnPurpose::FeatIndex.target_table(), Some("feat"));
        assert_eq!(ColumnPurpose::ClassId.target_table(), Some("classes"));
        assert_eq!(ColumnPurpose::Label.target_table(), None);
    }
}
