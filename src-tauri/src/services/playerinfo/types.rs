use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PlayerClassEntry {
    pub name: String,
    pub level: u8,
}

impl PlayerClassEntry {
    pub fn new(name: impl Into<String>, level: u8) -> Self {
        Self {
            name: name.into(),
            level,
        }
    }
}

/// Layout per Arbos' playerinfo.bin reverse-engineering:
/// <https://gist.github.com/Arbos/225c724f91309d3f515e0f110524feee>
/// Verified against all 7 save fixtures in `tests/fixtures/saves/`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerInfoData {
    pub first_name: String,
    pub last_name: String,
    pub name: String,

    pub subrace: String,
    pub alignment: String,
    /// Padding `u32` written when last name is absent — otherwise unused.
    pub unknown1: u32,

    /// Good/Evil axis: 4 = Good, 1 = Neutral, 5 = Evil.
    pub alignment_vertical: u32,
    /// Law/Chaos axis: 2 = Lawful, 1 = Neutral, 3 = Chaotic.
    pub alignment_horizontal: u32,
    /// Row id in `backgrounds.2da` — drives the background shown in the load menu.
    pub background_id: u32,

    pub classes: Vec<PlayerClassEntry>,

    pub deity: String,

    pub str_score: u32,
    pub dex_score: u32,
    pub con_score: u32,
    pub int_score: u32,
    pub wis_score: u32,
    pub cha_score: u32,

    pub str_mod: i32,
    pub dex_mod: i32,
    pub con_mod: i32,
    pub int_mod: i32,
    pub wis_mod: i32,
    pub cha_mod: i32,
}

impl PlayerInfoData {
    pub fn new() -> Self {
        Self {
            alignment_vertical: 1,
            alignment_horizontal: 1,
            background_id: 0,
            str_score: 10,
            dex_score: 10,
            con_score: 10,
            int_score: 10,
            wis_score: 10,
            cha_score: 10,
            ..Default::default()
        }
    }

    pub fn total_level(&self) -> u32 {
        self.classes.iter().map(|c| u32::from(c.level)).sum()
    }

    pub fn display_name(&self) -> String {
        if !self.name.is_empty() {
            self.name.clone()
        } else if !self.last_name.is_empty() {
            format!("{} {}", self.first_name, self.last_name)
        } else {
            self.first_name.clone()
        }
    }

    pub fn class_summary(&self) -> String {
        self.classes
            .iter()
            .map(|c| format!("{} {}", c.name, c.level))
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_class_entry() {
        let entry = PlayerClassEntry::new("Fighter", 5);
        assert_eq!(entry.name, "Fighter");
        assert_eq!(entry.level, 5);
    }

    #[test]
    fn test_player_info_data_defaults() {
        let data = PlayerInfoData::new();
        assert_eq!(data.str_score, 10);
        assert_eq!(data.alignment_vertical, 1);
        assert_eq!(data.alignment_horizontal, 1);
        assert_eq!(data.background_id, 0);
        assert_eq!(data.str_mod, 0);
    }

    #[test]
    fn test_total_level() {
        let mut data = PlayerInfoData::new();
        data.classes.push(PlayerClassEntry::new("Fighter", 10));
        data.classes.push(PlayerClassEntry::new("Rogue", 5));
        assert_eq!(data.total_level(), 15);
    }

    #[test]
    fn test_display_name() {
        let mut data = PlayerInfoData::new();
        data.first_name = "John".to_string();
        data.last_name = "Doe".to_string();
        assert_eq!(data.display_name(), "John Doe");

        data.name = "Custom Name".to_string();
        assert_eq!(data.display_name(), "Custom Name");
    }
}
