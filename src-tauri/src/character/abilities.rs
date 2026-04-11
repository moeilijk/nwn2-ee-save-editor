use crate::character::gff_helpers::gff_value_to_i32;
use crate::character::types::{
    ABILITY_INCREASE_INTERVAL, ABILITY_MAX, ABILITY_MIN, AbilityIndex, AbilityModifiers,
    AbilityScores, HitPoints, calculate_modifier,
};
use crate::character::{Character, CharacterError};
use crate::loaders::GameData;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::services::item_property_decoder::ItemPropertyDecoder;

// Point buy constants (NWN2 standard)
const POINT_BUY_COSTS: [i32; 11] = [0, 1, 2, 3, 4, 5, 6, 8, 10, 13, 16];
pub const POINT_BUY_BUDGET: i32 = 32;
pub const POINT_BUY_MIN: i32 = 8;
pub const POINT_BUY_MAX: i32 = 18;

fn point_buy_cost_for_score(score: i32) -> i32 {
    if score <= 8 {
        0
    } else if score >= 18 {
        16
    } else {
        POINT_BUY_COSTS[(score - 8) as usize]
    }
}

pub fn calculate_point_buy_cost(scores: &AbilityScores) -> i32 {
    point_buy_cost_for_score(scores.str_)
        + point_buy_cost_for_score(scores.dex)
        + point_buy_cost_for_score(scores.con)
        + point_buy_cost_for_score(scores.int)
        + point_buy_cost_for_score(scores.wis)
        + point_buy_cost_for_score(scores.cha)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct AbilityIncrease {
    pub level: i32,
    pub ability: AbilityIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct AbilityPointsSummary {
    pub base_scores: AbilityScores,
    pub level_increases: Vec<AbilityIncrease>,
    pub expected_increases: i32,
    pub actual_increases: i32,
    pub available: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct EncumbranceInfo {
    pub light_limit: f32,
    pub medium_limit: f32,
    pub heavy_limit: f32,
    pub max_limit: f32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Type)]
pub struct PointBuyState {
    pub starting_scores: AbilityScores,
    pub point_buy_cost: i32,
    pub budget: i32,
    pub remaining: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct AbilitiesState {
    pub base_scores: AbilityScores,
    pub effective_scores: AbilityScores,
    pub modifiers: AbilityModifiers,
    pub racial_modifiers: AbilityModifiers,
    pub equipment_modifiers: AbilityModifiers,
    pub hit_points: HitPoints,
    pub encumbrance: EncumbranceInfo,
    pub point_summary: AbilityPointsSummary,
    pub point_buy: PointBuyState,
}

impl Default for EncumbranceInfo {
    fn default() -> Self {
        Self {
            light_limit: 33.0,
            medium_limit: 66.0,
            heavy_limit: 100.0,
            max_limit: 200.0,
        }
    }
}

impl Character {
    pub fn base_ability(&self, ability: AbilityIndex) -> i32 {
        self.get_byte(ability.gff_field())
            .map_or(10, |v| i32::from(v))
    }

    pub fn set_ability(&mut self, ability: AbilityIndex, value: i32) -> Result<(), CharacterError> {
        if !(ABILITY_MIN..=ABILITY_MAX).contains(&value) {
            return Err(CharacterError::OutOfRange {
                field: ability.gff_field(),
                value,
                min: ABILITY_MIN,
                max: ABILITY_MAX,
            });
        }

        self.set_byte(ability.gff_field(), value as u8);
        Ok(())
    }

    /// Set an ability score and trigger cascading effects (like HP recalculation for Constitution).
    pub fn set_ability_with_cascades(
        &mut self,
        ability: AbilityIndex,
        value: i32,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        let old_value = self.base_ability(ability);

        if !(ABILITY_MIN..=ABILITY_MAX).contains(&value) {
            return Err(CharacterError::OutOfRange {
                field: ability.gff_field(),
                value,
                min: ABILITY_MIN,
                max: ABILITY_MAX,
            });
        }

        self.sync_ability_level_up_history(ability, old_value, value)?;

        // 1. Set the raw value first
        self.set_ability(ability, value)?;

        // 2. Handle Cascades
        // For now, only Constitution changes trigger persistent updates (HP)
        if ability == AbilityIndex::CON {
            self.recalculate_hit_points(old_value, value);
        }
        if ability == AbilityIndex::INT {
            self.normalize_skill_points(game_data);
        }

        Ok(())
    }

    fn sync_ability_level_up_history(
        &mut self,
        ability: AbilityIndex,
        old_value: i32,
        new_value: i32,
    ) -> Result<(), CharacterError> {
        let delta = new_value - old_value;
        if delta > 0 {
            self.reserve_ability_increase_slots(ability, delta as usize)?;
        } else if delta < 0 {
            self.release_ability_increase_slots(ability, (-delta) as usize)?;
        }
        Ok(())
    }

    fn reserve_ability_increase_slots(
        &mut self,
        ability: AbilityIndex,
        count: usize,
    ) -> Result<(), CharacterError> {
        use crate::character::types::ABILITY_INCREASE_INTERVAL;
        use crate::parsers::gff::GffValue;

        if count == 0 {
            return Ok(());
        }

        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        let mut remaining = count;

        for (idx, entry) in lvl_stat_list.iter_mut().enumerate() {
            let char_level = (idx + 1) as i32;
            let current_value = entry.get("LvlStatAbility").and_then(gff_value_to_i32);
            let is_available_slot = char_level % ABILITY_INCREASE_INTERVAL == 0
                && matches!(current_value, None | Some(255));

            if is_available_slot {
                entry.insert("LvlStatAbility".to_string(), GffValue::Byte(ability.0));
                remaining -= 1;
                if remaining == 0 {
                    self.set_list("LvlStatList", lvl_stat_list);
                    return Ok(());
                }
            }
        }

        Err(CharacterError::ValidationFailed {
            field: "LvlStatAbility",
            message: format!(
                "Not enough ability increase slots to raise {} by {}",
                ability.short_name(),
                count
            ),
        })
    }

    fn release_ability_increase_slots(
        &mut self,
        ability: AbilityIndex,
        count: usize,
    ) -> Result<(), CharacterError> {
        use crate::character::types::ABILITY_INCREASE_INTERVAL;
        use crate::parsers::gff::GffValue;

        if count == 0 {
            return Ok(());
        }

        let mut lvl_stat_list = self.get_list_owned("LvlStatList").unwrap_or_default();
        let mut matching_indices: Vec<usize> = lvl_stat_list
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                let char_level = (idx + 1) as i32;
                (char_level % ABILITY_INCREASE_INTERVAL == 0
                    && entry.get("LvlStatAbility").and_then(gff_value_to_i32)
                        == Some(i32::from(ability.0)))
                .then_some(idx)
            })
            .collect();

        if matching_indices.len() < count {
            return Err(CharacterError::ValidationFailed {
                field: "LvlStatAbility",
                message: format!(
                    "Cannot lower {} below allocated level-up increases",
                    ability.short_name()
                ),
            });
        }

        matching_indices.reverse();
        for idx in matching_indices.into_iter().take(count) {
            lvl_stat_list[idx].insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        }

        self.set_list("LvlStatList", lvl_stat_list);
        Ok(())
    }

    pub(crate) fn apply_ability_batch_side_effects(
        &mut self,
        old_scores: AbilityScores,
        game_data: &GameData,
    ) {
        let new_scores = self.get_effective_abilities(game_data);

        if old_scores.con != new_scores.con {
            self.recalculate_hit_points(old_scores.con, new_scores.con);
        }

        self.normalize_skill_points(game_data);
    }

    fn recalculate_hit_points(&mut self, old_con: i32, new_con: i32) {
        let old_mod = calculate_modifier(old_con);
        let new_mod = calculate_modifier(new_con);

        if old_mod == new_mod {
            return;
        }

        let total_level = self.total_level();
        if total_level == 0 {
            return;
        }

        let mod_diff = new_mod - old_mod;
        let hp_change = total_level * mod_diff;

        if hp_change == 0 {
            return;
        }

        let current_hp = self.current_hp();
        let max_hp = self.max_hp();
        let new_max_hp = max_hp + hp_change;
        let new_current_hp = 1.max((current_hp + hp_change).min(new_max_hp));

        self.set_max_hp(new_max_hp);
        self.set_current_hp(new_current_hp);
        self.set_base_hp(new_max_hp);
    }

    pub fn base_scores(&self) -> AbilityScores {
        AbilityScores::new(
            self.base_ability(AbilityIndex::STR),
            self.base_ability(AbilityIndex::DEX),
            self.base_ability(AbilityIndex::CON),
            self.base_ability(AbilityIndex::INT),
            self.base_ability(AbilityIndex::WIS),
            self.base_ability(AbilityIndex::CHA),
        )
    }

    pub fn ability_modifier(&self, ability: AbilityIndex) -> i32 {
        let score = self.base_ability(ability);
        calculate_modifier(score)
    }

    pub fn ability_modifiers(&self) -> AbilityModifiers {
        let scores = self.base_scores();
        AbilityModifiers::from_scores(&scores)
    }

    pub fn current_hp(&self) -> i32 {
        self.get_i32("CurrentHitPoints").unwrap_or(0)
    }

    pub fn max_hp(&self) -> i32 {
        self.get_i32("MaxHitPoints").unwrap_or(0)
    }

    pub fn base_hp(&self) -> i32 {
        self.get_i32("HitPoints").unwrap_or(0)
    }

    pub fn temp_hp(&self) -> i32 {
        self.get_i32("TempHitPoints").unwrap_or(0)
    }

    pub fn set_current_hp(&mut self, hp: i32) {
        self.set_i32("CurrentHitPoints", hp);
    }

    pub fn set_max_hp(&mut self, hp: i32) {
        self.set_i32("MaxHitPoints", hp);
    }

    pub fn set_base_hp(&mut self, hp: i32) {
        self.set_i32("HitPoints", hp);
    }

    pub fn set_temp_hp(&mut self, hp: i32) {
        self.set_i32("TempHitPoints", hp);
    }

    pub fn hit_points(&self) -> HitPoints {
        HitPoints::new(self.current_hp(), self.max_hp(), self.temp_hp())
    }

    pub fn get_effective_abilities(&self, game_data: &GameData) -> AbilityScores {
        let mut scores = self.base_scores();
        let racial_mods = self.get_racial_ability_modifiers(game_data);

        for ability in AbilityIndex::all() {
            let current = scores.get(ability);
            scores.set(ability, current + racial_mods.get(ability));
        }

        scores
    }

    pub fn get_racial_ability_modifiers(&self, game_data: &GameData) -> AbilityModifiers {
        // Delegate to get_racial_modifier_deltas in race.rs which handles both
        // base races (from racialtypes.2da) and subraces (from racialsubtypes.2da)
        self.get_racial_modifier_deltas(game_data)
    }

    pub fn get_total_abilities(
        &self,
        game_data: &GameData,
        decoder: &ItemPropertyDecoder,
    ) -> AbilityScores {
        let mut scores = self.get_effective_abilities(game_data);
        let equip_bonuses = self.get_equipment_bonuses(game_data, decoder);

        scores.str_ += equip_bonuses.str_bonus;
        scores.dex += equip_bonuses.dex_bonus;
        scores.con += equip_bonuses.con_bonus;
        scores.int += equip_bonuses.int_bonus;
        scores.wis += equip_bonuses.wis_bonus;
        scores.cha += equip_bonuses.cha_bonus;

        scores
    }

    pub fn get_total_ability_modifiers(
        &self,
        game_data: &GameData,
        decoder: &ItemPropertyDecoder,
    ) -> AbilityModifiers {
        let scores = self.get_total_abilities(game_data, decoder);
        AbilityModifiers::from_scores(&scores)
    }

    pub fn get_effective_ability_modifier(
        &self,
        ability: AbilityIndex,
        game_data: &GameData,
    ) -> i32 {
        let scores = self.get_effective_abilities(game_data);
        calculate_modifier(scores.get(ability))
    }

    pub fn get_level_up_ability_history(&self) -> Vec<AbilityIncrease> {
        let Some(lvl_stat_list) = self.get_list("LvlStatList") else {
            return vec![];
        };

        let mut history = Vec::new();

        for (idx, entry) in lvl_stat_list.iter().enumerate() {
            let char_level = (idx + 1) as i32;

            if char_level % ABILITY_INCREASE_INTERVAL != 0 {
                continue;
            }

            if let Some(ability_value) = entry.get("LvlStatAbility") {
                let ability_index = gff_value_to_i32(ability_value).unwrap_or(-1);
                if (0..6).contains(&ability_index)
                    && let Some(ability) = AbilityIndex::from_index(ability_index as u8)
                {
                    history.push(AbilityIncrease {
                        level: char_level,
                        ability,
                    });
                }
            }
        }

        history
    }

    pub fn get_ability_points_summary(&self) -> AbilityPointsSummary {
        let base_scores = self.base_scores();
        let level_increases = self.get_level_up_ability_history();
        let total_level = self.total_level();
        let expected_increases = total_level / ABILITY_INCREASE_INTERVAL;
        let actual_increases = level_increases.len() as i32;
        let available = expected_increases - actual_increases;

        AbilityPointsSummary {
            base_scores,
            level_increases,
            expected_increases,
            actual_increases,
            available,
        }
    }

    pub fn get_starting_ability_scores(&self, game_data: &GameData) -> AbilityScores {
        let _ = game_data;
        let base = self.base_scores();
        let history = self.get_level_up_ability_history();

        let mut increases = [0i32; 6];
        for inc in &history {
            increases[inc.ability.0 as usize] += 1;
        }

        AbilityScores {
            str_: base.str_ - increases[0],
            dex: base.dex - increases[1],
            con: base.con - increases[2],
            int: base.int - increases[3],
            wis: base.wis - increases[4],
            cha: base.cha - increases[5],
        }
    }

    pub fn get_point_buy_state(&self, game_data: &GameData) -> PointBuyState {
        let starting_scores = self.get_starting_ability_scores(game_data);
        let point_buy_cost = calculate_point_buy_cost(&starting_scores);

        PointBuyState {
            starting_scores,
            point_buy_cost,
            budget: POINT_BUY_BUDGET,
            remaining: POINT_BUY_BUDGET - point_buy_cost,
        }
    }

    pub fn apply_point_buy_scores(
        &mut self,
        new_scores: AbilityScores,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        let old_scores = self.get_effective_abilities(game_data);

        self.clear_ability_level_up_history()?;

        self.set_ability(AbilityIndex::STR, new_scores.str_)?;
        self.set_ability(AbilityIndex::DEX, new_scores.dex)?;
        self.set_ability(AbilityIndex::CON, new_scores.con)?;
        self.set_ability(AbilityIndex::INT, new_scores.int)?;
        self.set_ability(AbilityIndex::WIS, new_scores.wis)?;
        self.set_ability(AbilityIndex::CHA, new_scores.cha)?;

        self.apply_ability_batch_side_effects(old_scores, game_data);

        Ok(())
    }

    pub fn set_starting_ability_scores(
        &mut self,
        new_scores: AbilityScores,
        game_data: &GameData,
    ) -> Result<(), CharacterError> {
        let cost = calculate_point_buy_cost(&new_scores);
        if cost > POINT_BUY_BUDGET {
            return Err(CharacterError::ValidationFailed {
                field: "point_buy_cost",
                message: format!("Point buy cost {cost} exceeds budget {POINT_BUY_BUDGET}"),
            });
        }

        for (field, score) in [
            ("Str", new_scores.str_),
            ("Dex", new_scores.dex),
            ("Con", new_scores.con),
            ("Int", new_scores.int),
            ("Wis", new_scores.wis),
            ("Cha", new_scores.cha),
        ] {
            if !(POINT_BUY_MIN..=POINT_BUY_MAX).contains(&score) {
                return Err(CharacterError::ValidationFailed {
                    field,
                    message: format!("Scores must be between {POINT_BUY_MIN} and {POINT_BUY_MAX}"),
                });
            }
        }

        let old_scores = self.get_effective_abilities(game_data);
        let history = self.get_level_up_ability_history();
        let mut increases = [0i32; 6];
        for inc in history {
            increases[inc.ability.0 as usize] += 1;
        }

        self.set_ability(AbilityIndex::STR, new_scores.str_ + increases[0])?;
        self.set_ability(AbilityIndex::DEX, new_scores.dex + increases[1])?;
        self.set_ability(AbilityIndex::CON, new_scores.con + increases[2])?;
        self.set_ability(AbilityIndex::INT, new_scores.int + increases[3])?;
        self.set_ability(AbilityIndex::WIS, new_scores.wis + increases[4])?;
        self.set_ability(AbilityIndex::CHA, new_scores.cha + increases[5])?;

        self.apply_ability_batch_side_effects(old_scores, game_data);

        Ok(())
    }

    pub fn calculate_encumbrance(&self, game_data: &GameData) -> EncumbranceInfo {
        let strength = self.get_effective_abilities(game_data).str_;

        let heavy = calculate_heavy_load(strength);
        let light = (heavy as f32 * 0.33).round();
        let medium = (heavy as f32 * 0.66).round();
        let max = (heavy as f32 * 2.0).round();

        EncumbranceInfo {
            light_limit: light,
            medium_limit: medium,
            heavy_limit: heavy as f32,
            max_limit: max,
        }
    }

    pub fn get_abilities_state(
        &self,
        game_data: &GameData,
        decoder: &ItemPropertyDecoder,
    ) -> AbilitiesState {
        let base_scores = self.base_scores();
        let effective_scores = self.get_effective_abilities(game_data);
        let racial_modifiers = self.get_racial_ability_modifiers(game_data);

        let equip_bonuses = self.get_equipment_bonuses(game_data, decoder);
        let equipment_modifiers = AbilityModifiers::new(
            equip_bonuses.str_bonus,
            equip_bonuses.dex_bonus,
            equip_bonuses.con_bonus,
            equip_bonuses.int_bonus,
            equip_bonuses.wis_bonus,
            equip_bonuses.cha_bonus,
        );

        let total_scores = AbilityScores::new(
            base_scores.str_ + equip_bonuses.str_bonus,
            base_scores.dex + equip_bonuses.dex_bonus,
            base_scores.con + equip_bonuses.con_bonus,
            base_scores.int + equip_bonuses.int_bonus,
            base_scores.wis + equip_bonuses.wis_bonus,
            base_scores.cha + equip_bonuses.cha_bonus,
        );
        let modifiers = AbilityModifiers::from_scores(&total_scores);

        AbilitiesState {
            base_scores,
            effective_scores,
            modifiers,
            racial_modifiers,
            equipment_modifiers,
            hit_points: self.hit_points(),
            encumbrance: self.calculate_encumbrance(game_data),
            point_summary: self.get_ability_points_summary(),
            point_buy: self.get_point_buy_state(game_data),
        }
    }

    pub fn clear_ability_level_up_history(&mut self) -> Result<(), CharacterError> {
        use crate::parsers::gff::GffValue;

        if let Some(list) = self.get_list_mut("LvlStatList") {
            for entry in list.iter_mut() {
                entry.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
            }
        }
        Ok(())
    }
}

fn calculate_heavy_load(strength: i32) -> i32 {
    match strength {
        ..=0 => 0,
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        6 => 60,
        7 => 70,
        8 => 80,
        9 => 90,
        10 => 100,
        11 => 115,
        12 => 130,
        13 => 150,
        14 => 175,
        15 => 200,
        16 => 230,
        17 => 260,
        18 => 300,
        19 => 350,
        20 => 400,
        21 => 460,
        22 => 520,
        23 => 600,
        24 => 700,
        25 => 800,
        26 => 920,
        27 => 1040,
        28 => 1200,
        29 => 1400,
        _ => {
            let extra_tens = (strength - 29) / 10;
            let remainder = (strength - 29) % 10;
            let base = match remainder {
                0 => 1400,
                1 => 1600,
                2 => 1840,
                3 => 2080,
                4 => 2400,
                5 => 2800,
                6 => 3200,
                7 => 3680,
                8 => 4160,
                9 => 4800,
                _ => 1400,
            };
            base * 4_i32.pow(extra_tens as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::{GameData, LoadedTable};
    use crate::parsers::gff::GffValue;
    use crate::parsers::tda::TDAParser;
    use crate::parsers::tlk::TLKParser;
    use indexmap::IndexMap;
    use std::sync::Arc;

    fn create_test_character() -> Character {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(16));
        fields.insert("Dex".to_string(), GffValue::Byte(14));
        fields.insert("Con".to_string(), GffValue::Byte(12));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(8));
        fields.insert("Cha".to_string(), GffValue::Byte(14));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(50));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(100));
        fields.insert("HitPoints".to_string(), GffValue::Int(80));
        fields.insert("TempHitPoints".to_string(), GffValue::Int(5));
        Character::from_gff(fields)
    }

    fn create_game_data_with_racial_modifiers(
        racial_rows: &[(&str, i32, i32, i32, i32, i32, i32)],
    ) -> GameData {
        let mut game_data = GameData::new(Arc::new(std::sync::RwLock::new(TLKParser::default())));
        let mut parser = TDAParser::new();
        let mut data = String::from(
            "2DA V2.0\n\nLabel StrAdjust DexAdjust ConAdjust IntAdjust WisAdjust ChaAdjust\n",
        );

        for (row_id, (label, str_mod, dex_mod, con_mod, int_mod, wis_mod, cha_mod)) in
            racial_rows.iter().enumerate()
        {
            data.push_str(&format!(
                "{row_id} {label} {str_mod} {dex_mod} {con_mod} {int_mod} {wis_mod} {cha_mod}\n"
            ));
        }

        parser
            .parse_from_string(&data)
            .expect("Failed to parse test racialtypes 2DA");
        game_data.tables.insert(
            "racialtypes".to_string(),
            LoadedTable::new("racialtypes".to_string(), Arc::new(parser)),
        );
        game_data
    }

    #[test]
    fn test_base_ability() {
        let character = create_test_character();
        assert_eq!(character.base_ability(AbilityIndex::STR), 16);
        assert_eq!(character.base_ability(AbilityIndex::DEX), 14);
        assert_eq!(character.base_ability(AbilityIndex::CON), 12);
        assert_eq!(character.base_ability(AbilityIndex::INT), 10);
        assert_eq!(character.base_ability(AbilityIndex::WIS), 8);
        assert_eq!(character.base_ability(AbilityIndex::CHA), 14);
    }

    #[test]
    fn test_base_scores() {
        let character = create_test_character();
        let scores = character.base_scores();
        assert_eq!(scores.str_, 16);
        assert_eq!(scores.dex, 14);
        assert_eq!(scores.con, 12);
        assert_eq!(scores.int, 10);
        assert_eq!(scores.wis, 8);
        assert_eq!(scores.cha, 14);
    }

    #[test]
    fn test_ability_modifier() {
        let character = create_test_character();
        assert_eq!(character.ability_modifier(AbilityIndex::STR), 3);
        assert_eq!(character.ability_modifier(AbilityIndex::DEX), 2);
        assert_eq!(character.ability_modifier(AbilityIndex::CON), 1);
        assert_eq!(character.ability_modifier(AbilityIndex::INT), 0);
        assert_eq!(character.ability_modifier(AbilityIndex::WIS), -1);
        assert_eq!(character.ability_modifier(AbilityIndex::CHA), 2);
    }

    #[test]
    fn test_ability_modifiers() {
        let character = create_test_character();
        let mods = character.ability_modifiers();
        assert_eq!(mods.str_mod, 3);
        assert_eq!(mods.dex_mod, 2);
        assert_eq!(mods.con_mod, 1);
        assert_eq!(mods.int_mod, 0);
        assert_eq!(mods.wis_mod, -1);
        assert_eq!(mods.cha_mod, 2);
    }

    #[test]
    fn test_set_ability_valid() {
        let mut character = create_test_character();
        let result = character.set_ability(AbilityIndex::STR, 18);
        assert!(result.is_ok());
        assert_eq!(character.base_ability(AbilityIndex::STR), 18);
    }

    #[test]
    fn test_set_ability_too_low() {
        let mut character = create_test_character();
        let result = character.set_ability(AbilityIndex::STR, 2);
        assert!(result.is_err());
        match result {
            Err(CharacterError::OutOfRange {
                field,
                value,
                min,
                max,
            }) => {
                assert_eq!(field, "Str");
                assert_eq!(value, 2);
                assert_eq!(min, 3);
                assert_eq!(max, 50);
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }

    #[test]
    fn test_set_ability_too_high() {
        let mut character = create_test_character();
        let result = character.set_ability(AbilityIndex::STR, 51);
        assert!(result.is_err());
    }

    #[test]
    fn test_hit_points_getters() {
        let character = create_test_character();
        assert_eq!(character.current_hp(), 50);
        assert_eq!(character.max_hp(), 100);
        assert_eq!(character.base_hp(), 80);
        assert_eq!(character.temp_hp(), 5);
    }

    #[test]
    fn test_hit_points_setters() {
        let mut character = create_test_character();

        character.set_current_hp(75);
        assert_eq!(character.current_hp(), 75);

        character.set_max_hp(120);
        assert_eq!(character.max_hp(), 120);

        character.set_base_hp(90);
        assert_eq!(character.base_hp(), 90);

        character.set_temp_hp(10);
        assert_eq!(character.temp_hp(), 10);
    }

    #[test]
    fn test_hit_points_struct() {
        let character = create_test_character();
        let hp = character.hit_points();
        assert_eq!(hp.current, 50);
        assert_eq!(hp.max, 100);
        assert_eq!(hp.temp, 5);
        assert_eq!(hp.effective_current(), 55);
    }

    #[test]
    fn test_base_ability_missing_field() {
        let fields = IndexMap::new();
        let character = Character::from_gff(fields);
        assert_eq!(character.base_ability(AbilityIndex::STR), 10);
    }

    #[test]
    fn test_ability_at_boundaries() {
        let mut character = create_test_character();

        character.set_ability(AbilityIndex::STR, 3).unwrap();
        assert_eq!(character.base_ability(AbilityIndex::STR), 3);
        assert_eq!(character.ability_modifier(AbilityIndex::STR), -4);

        character.set_ability(AbilityIndex::STR, 50).unwrap();
        assert_eq!(character.base_ability(AbilityIndex::STR), 50);
        assert_eq!(character.ability_modifier(AbilityIndex::STR), 20);
    }

    #[test]
    fn test_calculate_heavy_load() {
        assert_eq!(calculate_heavy_load(10), 100);
        assert_eq!(calculate_heavy_load(16), 230);
        assert_eq!(calculate_heavy_load(18), 300);
        assert_eq!(calculate_heavy_load(20), 400);
    }

    #[test]
    fn test_calculate_encumbrance() {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(16));
        let character = Character::from_gff(fields);

        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        let info = character.calculate_encumbrance(&game_data);
        assert_eq!(info.heavy_limit, 230.0);
        assert_eq!(info.light_limit, (230.0_f32 * 0.33).round());
        assert_eq!(info.medium_limit, (230.0_f32 * 0.66).round());
        assert_eq!(info.max_limit, 460.0);
    }

    #[test]
    fn test_get_level_up_ability_history_empty() {
        let fields = IndexMap::new();
        let character = Character::from_gff(fields);
        let history = character.get_level_up_ability_history();
        assert!(history.is_empty());
    }

    #[test]
    fn test_get_level_up_ability_history() {
        let mut fields = IndexMap::new();

        let mut lvl_stat_list = Vec::new();
        for i in 0..8 {
            let mut entry = IndexMap::new();
            if i == 3 {
                entry.insert("LvlStatAbility".to_string(), GffValue::Byte(0));
            } else if i == 7 {
                entry.insert("LvlStatAbility".to_string(), GffValue::Byte(1));
            }
            lvl_stat_list.push(entry);
        }

        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );
        let character = Character::from_gff(fields);

        let history = character.get_level_up_ability_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].level, 4);
        assert_eq!(history[0].ability, AbilityIndex::STR);
        assert_eq!(history[1].level, 8);
        assert_eq!(history[1].ability, AbilityIndex::DEX);
    }

    #[test]
    fn test_get_ability_points_summary() {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(16));
        fields.insert("Dex".to_string(), GffValue::Byte(14));
        fields.insert("Con".to_string(), GffValue::Byte(12));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(8));
        fields.insert("Cha".to_string(), GffValue::Byte(14));

        let mut class_list = Vec::new();
        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(8));
        class_list.push(class_entry);
        fields.insert("ClassList".to_string(), GffValue::ListOwned(class_list));

        let mut lvl_stat_list = Vec::new();
        for i in 0..8 {
            let mut entry = IndexMap::new();
            if i == 3 {
                entry.insert("LvlStatAbility".to_string(), GffValue::Byte(0));
            } else if i == 7 {
                entry.insert("LvlStatAbility".to_string(), GffValue::Byte(1));
            }
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let character = Character::from_gff(fields);
        let summary = character.get_ability_points_summary();

        assert_eq!(summary.expected_increases, 2);
        assert_eq!(summary.actual_increases, 2);
        assert_eq!(summary.level_increases.len(), 2);
        assert_eq!(summary.base_scores.str_, 16);
    }

    #[test]
    fn test_effective_abilities_apply_racial_modifiers() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert("Str".to_string(), GffValue::Byte(10));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(8));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));

        let character = Character::from_gff(fields);
        let game_data = create_game_data_with_racial_modifiers(&[
            ("Human", 0, 0, 0, 0, 0, 0),
            ("Elf", 0, 2, -2, 0, 0, 0),
        ]);

        let effective = character.get_effective_abilities(&game_data);
        assert_eq!(effective.dex, 14);
        assert_eq!(effective.con, 6);
    }

    #[test]
    fn test_get_starting_ability_scores_subtracts_level_ups_only() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert("Str".to_string(), GffValue::Byte(10));
        fields.insert("Dex".to_string(), GffValue::Byte(13));
        fields.insert("Con".to_string(), GffValue::Byte(8));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));

        let mut lvl_stat_list = Vec::new();
        for i in 0..4 {
            let mut entry = IndexMap::new();
            let ability = if i == 3 { 1 } else { 255 };
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(ability));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(4));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let character = Character::from_gff(fields);
        let game_data = create_game_data_with_racial_modifiers(&[
            ("Human", 0, 0, 0, 0, 0, 0),
            ("Elf", 0, 2, -2, 0, 0, 0),
        ]);

        let starting = character.get_starting_ability_scores(&game_data);
        assert_eq!(starting.dex, 12);
        assert_eq!(starting.con, 8);
    }

    #[test]
    fn test_get_starting_ability_scores_reconstructs_valid_higher_level_point_buy() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(31));
        fields.insert(
            "Subrace".to_string(),
            GffValue::String("Yuan-ti Pureblood ".to_string().into()),
        );
        fields.insert("Str".to_string(), GffValue::Byte(19));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(14));
        fields.insert("Int".to_string(), GffValue::Byte(8));
        fields.insert("Wis".to_string(), GffValue::Byte(14));
        fields.insert("Cha".to_string(), GffValue::Byte(8));

        let mut lvl_stat_list = Vec::new();
        for i in 0..7 {
            let mut entry = IndexMap::new();
            let ability = if i == 3 { 0 } else { 255 };
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(ability));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(7));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let character = Character::from_gff(fields);
        let mut game_data = create_game_data_with_racial_modifiers(&[
            ("padding", 0, 0, 0, 0, 0, 0),
            ("padding", 0, 0, 0, 0, 0, 0),
        ]);

        let mut subrace_parser = TDAParser::new();
        for column in [
            "Label",
            "BaseRace",
            "PlayerRace",
            "StrAdjust",
            "DexAdjust",
            "ConAdjust",
            "IntAdjust",
            "WisAdjust",
            "ChaAdjust",
        ] {
            subrace_parser.add_column(column);
        }
        let mut yuan_ti_row = ahash::AHashMap::new();
        yuan_ti_row.insert(
            "Label".to_string(),
            Some("Yuan-ti Pureblood ".to_string()),
        );
        yuan_ti_row.insert("BaseRace".to_string(), Some("31".to_string()));
        yuan_ti_row.insert("PlayerRace".to_string(), Some("1".to_string()));
        yuan_ti_row.insert("StrAdjust".to_string(), Some("0".to_string()));
        yuan_ti_row.insert("DexAdjust".to_string(), Some("2".to_string()));
        yuan_ti_row.insert("ConAdjust".to_string(), Some("0".to_string()));
        yuan_ti_row.insert("IntAdjust".to_string(), Some("2".to_string()));
        yuan_ti_row.insert("WisAdjust".to_string(), Some("0".to_string()));
        yuan_ti_row.insert("ChaAdjust".to_string(), Some("2".to_string()));
        for _ in 0..31 {
            subrace_parser.add_row(ahash::AHashMap::new());
        }
        subrace_parser.add_row(yuan_ti_row);
        game_data.tables.insert(
            "racialsubtypes".to_string(),
            LoadedTable::new("racialsubtypes.2da".to_string(), Arc::new(subrace_parser)),
        );

        let starting = character.get_starting_ability_scores(&game_data);
        assert_eq!(starting, AbilityScores::new(18, 12, 14, 8, 14, 8));
        assert_eq!(calculate_point_buy_cost(&starting), 32);
    }

    #[test]
    fn test_con_change_updates_hp() {
        let mut fields = IndexMap::new();
        fields.insert("Con".to_string(), GffValue::Byte(14));

        let mut class_list = Vec::new();
        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(10));
        class_list.push(class_entry);
        fields.insert("ClassList".to_string(), GffValue::ListOwned(class_list));

        fields.insert("MaxHitPoints".to_string(), GffValue::Int(100));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(80));
        fields.insert("HitPoints".to_string(), GffValue::Int(100));

        let mut lvl_stat_list = Vec::new();
        for i in 0..10 {
            let mut entry = IndexMap::new();
            // Ability increase slots at character levels 4 and 8.
            if i == 3 || i == 7 {
                entry.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
            }
            lvl_stat_list.push(entry);
        }
        fields.insert("LvlStatList".to_string(), GffValue::ListOwned(lvl_stat_list));

        let mut character = Character::from_gff(fields);

        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        let result = character.set_ability_with_cascades(AbilityIndex::CON, 16, &game_data);

        assert!(result.is_ok());
        assert_eq!(character.base_ability(AbilityIndex::CON), 16);
        assert_eq!(character.max_hp(), 110);
        assert_eq!(character.base_hp(), 110);
        assert_eq!(character.current_hp(), 90);

        let result = character.set_ability_with_cascades(AbilityIndex::CON, 14, &game_data);

        assert!(result.is_ok());
        assert_eq!(character.base_ability(AbilityIndex::CON), 14);
        assert_eq!(character.max_hp(), 100);
        assert_eq!(character.current_hp(), 80);
    }

    #[test]
    fn test_set_ability_with_cascades_syncs_level_up_history() {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(10));

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(8));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let mut lvl_stat_list = Vec::new();
        for i in 0..8 {
            let mut entry = IndexMap::new();
            let ability = if i == 3 || i == 7 { 255 } else { 254 };
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(ability));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let mut character = Character::from_gff(fields);
        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        character
            .set_ability_with_cascades(AbilityIndex::STR, 12, &game_data)
            .expect("Two available level-up slots should allow +2 STR");

        assert_eq!(character.base_ability(AbilityIndex::STR), 12);
        let summary = character.get_ability_points_summary();
        assert_eq!(summary.actual_increases, 2);
        assert_eq!(summary.available, 0);

        character
            .set_ability_with_cascades(AbilityIndex::STR, 10, &game_data)
            .expect("Lowering back down should free both level-up slots");

        assert_eq!(character.base_ability(AbilityIndex::STR), 10);
        let summary = character.get_ability_points_summary();
        assert_eq!(summary.actual_increases, 0);
        assert_eq!(summary.available, 2);
    }

    #[test]
    fn test_set_ability_with_cascades_rejects_changes_without_matching_level_up_slots() {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(10));
        fields.insert("Dex".to_string(), GffValue::Byte(12));

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(8));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let mut lvl_stat_list = Vec::new();
        for i in 0..8 {
            let mut entry = IndexMap::new();
            let ability = if i == 3 || i == 7 { 1 } else { 254 };
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(ability));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let mut character = Character::from_gff(fields);
        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        let increase_result =
            character.set_ability_with_cascades(AbilityIndex::STR, 11, &game_data);
        assert!(matches!(
            increase_result,
            Err(CharacterError::ValidationFailed { field, .. }) if field == "LvlStatAbility"
        ));
        assert_eq!(character.base_ability(AbilityIndex::STR), 10);

        let decrease_result = character.set_ability_with_cascades(AbilityIndex::STR, 9, &game_data);
        assert!(matches!(
            decrease_result,
            Err(CharacterError::ValidationFailed { field, .. }) if field == "LvlStatAbility"
        ));
        assert_eq!(character.base_ability(AbilityIndex::STR), 10);
    }

    #[test]
    fn test_set_ability_with_cascades_records_single_level_up_per_point() {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(18));

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(10));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let mut lvl_stat_list = Vec::new();
        for _ in 0..10 {
            let mut entry = IndexMap::new();
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let mut character = Character::from_gff(fields);
        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        character
            .set_ability_with_cascades(AbilityIndex::STR, 19, &game_data)
            .expect("Single level-up increase should succeed");

        let summary = character.get_ability_points_summary();
        assert_eq!(summary.expected_increases, 2);
        assert_eq!(summary.actual_increases, 1);
        assert_eq!(summary.available, 1);

        let starting = character.get_starting_ability_scores(&game_data);
        assert_eq!(starting.str_, 18);
    }

    #[test]
    fn test_apply_point_buy_scores_clears_level_history_without_readding_increases() {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(16));
        fields.insert("Dex".to_string(), GffValue::Byte(14));
        fields.insert("Con".to_string(), GffValue::Byte(12));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(8));
        fields.insert("Cha".to_string(), GffValue::Byte(14));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(40));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(40));
        fields.insert("HitPoints".to_string(), GffValue::Int(40));

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(8));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let mut lvl_stat_list = Vec::new();
        for i in 0..8 {
            let mut entry = IndexMap::new();
            let ability = if i == 3 {
                0
            } else if i == 7 {
                1
            } else {
                255
            };
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(ability));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let mut character = Character::from_gff(fields);
        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));
        character
            .apply_point_buy_scores(AbilityScores::new(14, 14, 14, 12, 10, 8), &game_data)
            .expect("Point buy should apply cleanly");

        assert_eq!(character.base_ability(AbilityIndex::STR), 14);
        assert_eq!(character.base_ability(AbilityIndex::CON), 14);
        assert_eq!(character.max_hp(), 48);

        let history = character.get_list_owned("LvlStatList").unwrap();
        assert!(
            history.iter().all(|entry| {
                entry.get("LvlStatAbility").and_then(gff_value_to_i32) == Some(255)
            })
        );
    }

    #[test]
    fn test_apply_point_buy_scores_preserve_base_scores_and_normalize_skill_points() {
        let mut fields = IndexMap::new();
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert("Str".to_string(), GffValue::Byte(12));
        fields.insert("Dex".to_string(), GffValue::Byte(14));
        fields.insert("Con".to_string(), GffValue::Byte(10));
        fields.insert("Int".to_string(), GffValue::Byte(14));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));
        fields.insert("MaxHitPoints".to_string(), GffValue::Int(12));
        fields.insert("CurrentHitPoints".to_string(), GffValue::Int(12));
        fields.insert("HitPoints".to_string(), GffValue::Int(12));
        fields.insert("SkillPoints".to_string(), GffValue::Word(16));

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(1));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let mut history_entry = IndexMap::new();
        history_entry.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        history_entry.insert("LvlStatClass".to_string(), GffValue::Byte(0));
        history_entry.insert("SkillPoints".to_string(), GffValue::Short(16));
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![history_entry]),
        );

        let mut character = Character::from_gff(fields);
        let mut game_data = create_game_data_with_racial_modifiers(&[
            ("Human", 0, 0, 0, 0, 0, 0),
            ("TestRace", 0, 2, -2, 2, 0, 0),
        ]);
        let mut classes_parser = TDAParser::new();
        classes_parser.add_column("Label");
        classes_parser.add_column("HitDie");
        classes_parser.add_column("SkillPointBase");
        let mut fighter_row = ahash::AHashMap::new();
        fighter_row.insert("Label".to_string(), Some("Fighter".to_string()));
        fighter_row.insert("HitDie".to_string(), Some("10".to_string()));
        fighter_row.insert("SkillPointBase".to_string(), Some("2".to_string()));
        classes_parser.add_row(fighter_row);
        game_data.tables.insert(
            "classes".to_string(),
            LoadedTable::new("classes.2da".to_string(), Arc::new(classes_parser)),
        );

        character
            .apply_point_buy_scores(AbilityScores::new(12, 12, 12, 8, 10, 10), &game_data)
            .expect("Point buy should apply cleanly");

        assert_eq!(character.base_ability(AbilityIndex::STR), 12);
        assert_eq!(character.base_ability(AbilityIndex::DEX), 12);
        assert_eq!(character.base_ability(AbilityIndex::CON), 12);
        assert_eq!(character.base_ability(AbilityIndex::INT), 8);
        assert_eq!(character.base_ability(AbilityIndex::WIS), 10);
        assert_eq!(character.base_ability(AbilityIndex::CHA), 10);
        assert_eq!(character.get_available_skill_points(), 8);

        let starting_scores = character.get_starting_ability_scores(&game_data);
        assert_eq!(starting_scores, AbilityScores::new(12, 12, 12, 8, 10, 10));

        let history = character.get_list_owned("LvlStatList").unwrap();
        assert!(
            history.iter().all(|entry| {
                entry.get("LvlStatAbility").and_then(gff_value_to_i32) == Some(255)
            })
        );
    }

    #[test]
    fn test_set_starting_ability_scores_preserves_level_up_history() {
        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(19));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(14));
        fields.insert("Int".to_string(), GffValue::Byte(8));
        fields.insert("Wis".to_string(), GffValue::Byte(14));
        fields.insert("Cha".to_string(), GffValue::Byte(8));

        let mut class_entry = IndexMap::new();
        class_entry.insert("Class".to_string(), GffValue::Byte(0));
        class_entry.insert("ClassLevel".to_string(), GffValue::Short(7));
        fields.insert(
            "ClassList".to_string(),
            GffValue::ListOwned(vec![class_entry]),
        );

        let mut lvl_stat_list = Vec::new();
        for i in 0..7 {
            let mut entry = IndexMap::new();
            let ability = if i == 3 { 0 } else { 255 };
            entry.insert("LvlStatAbility".to_string(), GffValue::Byte(ability));
            lvl_stat_list.push(entry);
        }
        fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(lvl_stat_list),
        );

        let mut character = Character::from_gff(fields);
        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        character
            .set_starting_ability_scores(AbilityScores::new(16, 14, 14, 8, 14, 8), &game_data)
            .expect("Starting scores should update without clearing history");

        assert_eq!(character.base_ability(AbilityIndex::STR), 17);
        assert_eq!(character.base_ability(AbilityIndex::DEX), 14);

        let summary = character.get_ability_points_summary();
        assert_eq!(summary.actual_increases, 1);
        assert_eq!(summary.available, 0);

        let starting = character.get_starting_ability_scores(&game_data);
        assert_eq!(starting, AbilityScores::new(16, 14, 14, 8, 14, 8));
    }

    #[test]
    fn test_get_total_abilities_with_equipment() {
        use crate::services::item_property_decoder::ItemPropertyDecoder;

        let mut fields = IndexMap::new();
        fields.insert("Str".to_string(), GffValue::Byte(14));
        fields.insert("Dex".to_string(), GffValue::Byte(12));
        fields.insert("Con".to_string(), GffValue::Byte(10));
        fields.insert("Int".to_string(), GffValue::Byte(10));
        fields.insert("Wis".to_string(), GffValue::Byte(10));
        fields.insert("Cha".to_string(), GffValue::Byte(10));

        // Create item with +2 STR
        let mut props = Vec::new();
        let mut prop = IndexMap::new();
        prop.insert("PropertyName".to_string(), GffValue::Word(0)); // Ability Bonus (0)
        prop.insert("Subtype".to_string(), GffValue::Word(0)); // Str (0)
        prop.insert("CostValue".to_string(), GffValue::Byte(2)); // +2
        props.push(prop);

        let mut item_struct = IndexMap::new();
        item_struct.insert("__struct_id__".to_string(), GffValue::Dword(16)); // Right Hand (0x10)
        item_struct.insert("BaseItem".to_string(), GffValue::Int(0)); // Shortsword or generic
        item_struct.insert("PropertiesList".to_string(), GffValue::ListOwned(props));

        fields.insert(
            "Equip_ItemList".to_string(),
            GffValue::ListOwned(vec![item_struct]),
        );

        let character = Character::from_gff(fields);

        let paths =
            std::sync::Arc::new(tokio::sync::RwLock::new(crate::config::NWN2Paths::default()));
        let rm = std::sync::Arc::new(tokio::sync::RwLock::new(
            crate::services::resource_manager::ResourceManager::new(paths),
        ));

        let game_data = crate::loaders::GameData::new(std::sync::Arc::new(std::sync::RwLock::new(
            crate::parsers::tlk::TLKParser::default(),
        )));

        let mut decoder = ItemPropertyDecoder::new(rm);
        use std::collections::HashMap;
        let abilities = HashMap::from([
            (0, "Str".to_string()),
            (1, "Dex".to_string()),
            (2, "Con".to_string()),
            (3, "Int".to_string()),
            (4, "Wis".to_string()),
            (5, "Cha".to_string()),
        ]);
        decoder.set_2da_tables(
            abilities,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        );

        let total = character.get_total_abilities(&game_data, &decoder);
        assert_eq!(total.str_, 16); // 14 + 2
        assert_eq!(total.dex, 12);
    }
}
