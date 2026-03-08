use serde::{Deserialize, Serialize};
use specta::Type;

use crate::character::types::{ClassId, FeatId, SpellId};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct CharacterUpdates {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub age: Option<i32>,
    pub deity: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub alignment: Option<(i32, i32)>,
    pub experience: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct AbilitiesUpdates {
    #[serde(rename = "Str")]
    pub str_: Option<i32>,
    #[serde(rename = "Dex")]
    pub dex: Option<i32>,
    #[serde(rename = "Con")]
    pub con: Option<i32>,
    #[serde(rename = "Int")]
    pub int: Option<i32>,
    #[serde(rename = "Wis")]
    pub wis: Option<i32>,
    #[serde(rename = "Cha")]
    pub cha: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct CombatUpdates {
    pub natural_armor: Option<i32>,
    pub initiative_misc: Option<i32>,
    pub fortitude_misc: Option<i32>,
    pub reflex_misc: Option<i32>,
    pub will_misc: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum FeatAction {
    Add { feat_id: FeatId },
    Remove { feat_id: FeatId },
    Swap { old_feat_id: FeatId, new_feat_id: FeatId },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SpellAction {
    Learn { class_id: ClassId, spell_id: SpellId },
    Forget { class_id: ClassId, spell_id: SpellId },
    Prepare { class_id: ClassId, spell_id: SpellId, level: i32 },
    Unprepare { class_id: ClassId, spell_id: SpellId, level: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ItemAction {
    Add { template_resref: String },
    Remove { index: usize },
    Unequip { slot: String },
}
