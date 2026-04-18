use crate::character::Character;
use crate::character::types::{AbilityIndex, FeatId, SaveBonuses, calculate_modifier};
use crate::loaders::GameData;
use crate::utils::parsing::row_int;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub enum SaveType {
    Fortitude,
    Reflex,
    Will,
}

impl SaveType {
    pub fn gff_field(&self) -> &'static str {
        match self {
            Self::Fortitude => "fortbonus",
            Self::Reflex => "refbonus",
            Self::Will => "willbonus",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Fortitude => "Fortitude",
            Self::Reflex => "Reflex",
            Self::Will => "Will",
        }
    }

    pub fn ability(&self) -> &'static str {
        match self {
            Self::Fortitude => "Constitution",
            Self::Reflex => "Dexterity",
            Self::Will => "Wisdom",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct SavingThrows {
    pub fortitude: SaveBreakdown,
    pub reflex: SaveBreakdown,
    pub will: SaveBreakdown,
}

impl SavingThrows {
    pub fn get(&self, save_type: SaveType) -> &SaveBreakdown {
        match save_type {
            SaveType::Fortitude => &self.fortitude,
            SaveType::Reflex => &self.reflex,
            SaveType::Will => &self.will,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct SaveBreakdown {
    pub total: i32,
    pub base: i32,
    pub ability: i32,
    pub equipment: i32,
    pub feat: i32,
    pub racial: i32,
    pub class_bonus: i32,
    pub misc: i32,
}

impl SaveBreakdown {
    pub fn calculate(
        base: i32,
        ability: i32,
        equipment: i32,
        feat: i32,
        racial: i32,
        class_bonus: i32,
        misc: i32,
    ) -> Self {
        Self {
            total: base + ability + equipment + feat + racial + class_bonus + misc,
            base,
            ability,
            equipment,
            feat,
            racial,
            class_bonus,
            misc,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct SaveSummary {
    pub fortitude: i32,
    pub reflex: i32,
    pub will: i32,
    pub saves: SavingThrows,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveCheck {
    pub save_type: SaveType,
    pub dc: i32,
    pub total_bonus: i32,
    pub roll_needed: i32,
    pub success_chance: f32,
    pub auto_success: bool,
    pub auto_fail: bool,
}

impl SaveCheck {
    pub fn evaluate(save_type: SaveType, total_bonus: i32, dc: i32, take_20: bool) -> Self {
        let roll_needed = (dc - total_bonus).max(1);

        let success_chance = if take_20 {
            if 20 + total_bonus >= dc { 100.0 } else { 0.0 }
        } else {
            let successes = (21 - roll_needed).clamp(0, 20);
            (successes as f32 * 5.0).clamp(0.0, 95.0)
        };

        let auto_success = roll_needed <= 1;
        let auto_fail = roll_needed > 20;

        Self {
            save_type,
            dc,
            total_bonus,
            roll_needed,
            success_chance,
            auto_success,
            auto_fail,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveChange {
    pub save_type: SaveType,
    pub old_misc: i32,
    pub new_misc: i32,
}

impl Character {
    pub fn get_save_summary(
        &self,
        game_data: &GameData,
        decoder: &crate::services::item_property_decoder::ItemPropertyDecoder,
    ) -> SaveSummary {
        let saves = self.get_saving_throws(game_data, decoder);

        SaveSummary {
            fortitude: saves.fortitude.total,
            reflex: saves.reflex.total,
            will: saves.will.total,
            saves,
        }
    }

    pub fn get_saving_throws(
        &self,
        game_data: &GameData,
        decoder: &crate::services::item_property_decoder::ItemPropertyDecoder,
    ) -> SavingThrows {
        // Base Saves: Derived exclusively from Class Levels (Heroic + Epic)
        let base_saves = self.calculate_base_saves(game_data);

        // Misc/Magic Bonuses: Stored in the GFF fields (fortbonus, refbonus, willbonus)
        let misc_bonuses = self.save_bonuses();

        let feat_bonuses = self.get_feat_save_bonuses(game_data);
        let item_bonuses = self.get_equipment_bonuses(game_data, decoder);
        let racial_bonuses = self.get_racial_save_bonuses(game_data);
        let class_bonuses = self.get_class_save_bonuses(game_data, &item_bonuses);

        let effective_abilities = self.get_effective_abilities(game_data);
        let con_mod = calculate_modifier(effective_abilities.con + item_bonuses.con_bonus);
        let dex_mod = calculate_modifier(effective_abilities.dex + item_bonuses.dex_bonus);
        let wis_mod = calculate_modifier(effective_abilities.wis + item_bonuses.wis_bonus);

        let fortitude = SaveBreakdown::calculate(
            base_saves.fortitude,
            con_mod,
            item_bonuses.fortitude_bonus,
            feat_bonuses.fortitude,
            racial_bonuses.fortitude,
            class_bonuses.fortitude,
            misc_bonuses.fortitude,
        );

        let reflex = SaveBreakdown::calculate(
            base_saves.reflex,
            dex_mod,
            item_bonuses.reflex_bonus,
            feat_bonuses.reflex,
            racial_bonuses.reflex,
            class_bonuses.reflex,
            misc_bonuses.reflex,
        );

        let will = SaveBreakdown::calculate(
            base_saves.will,
            wis_mod,
            item_bonuses.will_bonus,
            feat_bonuses.will,
            racial_bonuses.will,
            class_bonuses.will,
            misc_bonuses.will,
        );

        SavingThrows {
            fortitude,
            reflex,
            will,
        }
    }

    fn get_racial_save_bonuses(&self, game_data: &GameData) -> SaveBonuses {
        let race_id = self.race_id();

        let Some(races_table) = game_data.get_table("racialtypes") else {
            return SaveBonuses::default();
        };

        let Some(race_data) = races_table.get_by_id(race_id.0) else {
            return SaveBonuses::default();
        };

        let fort = row_int(&race_data, "fortsavebonus", 0);
        let reflex = row_int(&race_data, "refsavebonus", 0);
        let will = row_int(&race_data, "willsavebonus", 0);

        SaveBonuses::new(fort, reflex, will)
    }

    fn get_class_save_bonuses(
        &self,
        game_data: &GameData,
        item_bonuses: &crate::services::item_property_decoder::ItemBonuses,
    ) -> SaveBonuses {
        let _ = game_data;
        let mut bonuses = SaveBonuses::default();

        const DIVINE_GRACE_FEAT_ID: FeatId = FeatId(214);
        const DARK_ONES_LUCK_FEAT_ID: FeatId = FeatId(400);

        let cha_mod =
            calculate_modifier(self.base_ability(AbilityIndex::CHA) + item_bonuses.cha_bonus)
                .max(0);

        if self.has_feat(DIVINE_GRACE_FEAT_ID) {
            bonuses.fortitude += cha_mod;
            bonuses.reflex += cha_mod;
            bonuses.will += cha_mod;
        }

        if self.has_feat(DARK_ONES_LUCK_FEAT_ID) {
            bonuses.fortitude += cha_mod;
            bonuses.reflex += cha_mod;
            bonuses.will += cha_mod;
        }

        bonuses
    }

    pub fn get_save_breakdown(
        &self,
        game_data: &GameData,
        decoder: &crate::services::item_property_decoder::ItemPropertyDecoder,
        save_type: SaveType,
    ) -> SaveBreakdown {
        let saves = self.get_saving_throws(game_data, decoder);
        match save_type {
            SaveType::Fortitude => saves.fortitude,
            SaveType::Reflex => saves.reflex,
            SaveType::Will => saves.will,
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::parsers::gff::GffValue;
    use crate::parsers::tlk::TLKParser;
    use indexmap::IndexMap;
    use std::sync::{Arc, RwLock};

    fn create_test_character() -> Character {
        let mut fields = IndexMap::new();
        fields.insert("fortbonus".to_string(), GffValue::Short(5));
        fields.insert("refbonus".to_string(), GffValue::Short(3));
        fields.insert("willbonus".to_string(), GffValue::Short(7));
        fields.insert("Con".to_string(), GffValue::Byte(14));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Wis".to_string(), GffValue::Byte(16));
        Character::from_gff(fields)
    }

    fn create_test_game_data() -> GameData {
        GameData::new(Arc::new(RwLock::new(TLKParser::default())))
    }

    fn create_test_decoder() -> crate::services::item_property_decoder::ItemPropertyDecoder {
        use crate::config::nwn2_paths::NWN2Paths;
        use crate::services::resource_manager::ResourceManager;
        use tauri::async_runtime::RwLock;
        let paths = Arc::new(RwLock::new(NWN2Paths::default()));
        let rm = Arc::new(RwLock::new(ResourceManager::new(paths)));
        crate::services::item_property_decoder::ItemPropertyDecoder::new(rm)
    }

    #[test]
    fn test_save_breakdown_calculate() {
        let breakdown = SaveBreakdown::calculate(5, 2, 1, 2, 1, 2, 0);
        assert_eq!(breakdown.total, 13);
        assert_eq!(breakdown.base, 5);
        assert_eq!(breakdown.ability, 2);
    }

    #[test]
    fn test_save_check_evaluate() {
        let check = SaveCheck::evaluate(SaveType::Fortitude, 10, 15, false);
        assert_eq!(check.roll_needed, 5);
        assert_eq!(check.success_chance, 80.0);
        assert!(!check.auto_success);
        assert!(!check.auto_fail);
    }

    #[test]
    fn test_save_check_auto_success() {
        let check = SaveCheck::evaluate(SaveType::Reflex, 20, 10, false);
        assert!(check.auto_success);
        assert_eq!(check.success_chance, 95.0);
    }

    #[test]
    fn test_save_check_auto_fail() {
        let check = SaveCheck::evaluate(SaveType::Will, -5, 30, false);
        assert!(check.auto_fail);
        assert_eq!(check.success_chance, 0.0);
    }

    #[test]
    fn test_save_type_fields() {
        assert_eq!(SaveType::Fortitude.gff_field(), "fortbonus");
        assert_eq!(SaveType::Reflex.gff_field(), "refbonus");
        assert_eq!(SaveType::Will.gff_field(), "willbonus");
    }

    #[test]
    fn test_save_type_abilities() {
        assert_eq!(SaveType::Fortitude.ability(), "Constitution");
        assert_eq!(SaveType::Reflex.ability(), "Dexterity");
        assert_eq!(SaveType::Will.ability(), "Wisdom");
    }

    #[test]
    fn test_get_saving_throws() {
        let character = create_test_character();
        let game_data = create_test_game_data();
        let decoder = create_test_decoder();

        let throws = character.get_saving_throws(&game_data, &decoder);

        assert_eq!(throws.fortitude.base, 0);
        assert_eq!(throws.fortitude.misc, 5); // from fortbonus GFF field
        assert_eq!(throws.fortitude.ability, 2);

        assert_eq!(throws.reflex.base, 0);
        assert_eq!(throws.reflex.misc, 3); // from refbonus GFF field
        assert_eq!(throws.reflex.ability, 1);

        assert_eq!(throws.will.base, 0);
        assert_eq!(throws.will.misc, 7); // from willbonus GFF field
        assert_eq!(throws.will.ability, 3);
    }

    #[test]
    fn test_get_save_summary() {
        let character = create_test_character();
        let game_data = create_test_game_data();
        let decoder = create_test_decoder();

        let summary = character.get_save_summary(&game_data, &decoder);

        assert!(summary.fortitude >= 2); // 0 misc + 2 ability
        assert!(summary.reflex >= 1); // 0 misc + 1 ability
        assert!(summary.will >= 3); // 0 misc + 3 ability
    }

    #[test]
    fn test_get_save_breakdown() {
        let character = create_test_character();
        let game_data = create_test_game_data();
        let decoder = create_test_decoder();

        let fortitude = character.get_save_breakdown(&game_data, &decoder, SaveType::Fortitude);
        assert_eq!(fortitude.base, 0);
        assert_eq!(fortitude.misc, 5);
        assert_eq!(fortitude.ability, 2);

        let reflex = character.get_save_breakdown(&game_data, &decoder, SaveType::Reflex);
        assert_eq!(reflex.base, 0);
        assert_eq!(reflex.misc, 3);

        let will = character.get_save_breakdown(&game_data, &decoder, SaveType::Will);
        assert_eq!(will.base, 0);
        assert_eq!(will.misc, 7);
    }
}
