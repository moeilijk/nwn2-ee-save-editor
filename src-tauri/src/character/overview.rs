use serde::{Deserialize, Serialize};

use crate::loaders::GameData;

use super::classes::{ClassSummaryEntry, XpProgress};
use super::feats::DomainInfo;
use super::identity::Alignment;
use super::types::{HitPoints, SaveBonuses};
use super::Character;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OverviewState {
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub race_id: i32,
    pub race_name: String,
    pub subrace: Option<String>,
    pub gender: String,
    pub age: i32,
    pub deity: String,
    pub alignment: Alignment,
    pub alignment_string: String,
    pub description: String,

    pub total_level: i32,
    pub experience: i32,
    pub xp_progress: XpProgress,
    pub classes: Vec<ClassSummaryEntry>,

    pub hit_points: HitPoints,
    pub armor_class: i32,
    pub base_attack_bonus: i32,
    pub saving_throws: SaveBonuses,

    pub gold: u32,
    pub skill_points_available: i32,

    pub background: Option<String>,
    pub domains: Vec<DomainInfo>,
}

impl Character {
    pub fn get_overview_state(&self, game_data: &GameData) -> OverviewState {
        let alignment = self.alignment();
        let gender_id = self.gender();
        let gender_str = match gender_id {
            0 => "Male",
            1 => "Female",
            _ => "Unknown",
        };

        let domains = self.get_available_domains(game_data)
            .into_iter()
            .filter(|d| d.has_domain)
            .collect();

        let hit_points = HitPoints {
            current: self.current_hp(),
            max: self.max_hp(),
            temp: self.temp_hp(),
        };

        let spellcraft = self.skill_rank(crate::character::types::SkillId(16));
        let spellcraft_save_bonus = spellcraft / 5;

        let base_saves = self.calculate_base_saves(game_data);
        let saving_throws = SaveBonuses {
            fortitude: base_saves.fortitude + spellcraft_save_bonus,
            reflex: base_saves.reflex + spellcraft_save_bonus,
            will: base_saves.will + spellcraft_save_bonus,
        };

        let bab = self.calculate_bab(game_data);
        let dex_mod = self.ability_modifier(super::types::AbilityIndex::DEX);
        let tumble_rank = self.skill_rank(crate::character::types::SkillId(21));
        let base_ac = 10 + dex_mod + self.natural_ac() + (tumble_rank / 10);

        OverviewState {
            first_name: self.first_name(),
            last_name: self.last_name(),
            full_name: self.full_name(),
            race_id: self.race_id().0,
            race_name: self.race_name(game_data),
            subrace: self.subrace(),
            gender: gender_str.to_string(),
            age: self.age(),
            deity: self.deity(),
            alignment,
            alignment_string: alignment.alignment_string(),
            description: self.description(),

            total_level: self.total_level(),
            experience: self.experience(),
            xp_progress: self.get_xp_progress(game_data),
            classes: self.get_class_summary(game_data),

            hit_points,
            armor_class: base_ac,
            base_attack_bonus: bab,
            saving_throws,

            gold: self.gold(),
            skill_points_available: self.get_available_skill_points(),

            background: self.background(game_data),
            domains,
        }
    }
}
