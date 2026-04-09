use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::Character;
use crate::loaders::GameData;
use crate::parsers::gff::GffValue;
use crate::utils::parsing::row_str;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct TintChannel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct TintChannels {
    pub channel1: TintChannel,
    pub channel2: TintChannel,
    pub channel3: TintChannel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct AppearanceState {
    pub race_id: i32,
    pub race_name: String,
    pub gender: i32,
    pub gender_name: String,

    pub appearance_head: i32,
    pub appearance_hair: i32,
    pub appearance_fhair: i32,

    pub tint_head: TintChannels,
    pub tint_hair: TintChannels,

    pub color_tattoo1: i32,
    pub color_tattoo2: i32,

    pub model_scale: f32,

    pub soundset: i32,

    pub wings: i32,
    pub wings_name: String,
    pub tail: i32,
    pub tail_name: String,

    pub available_heads: Vec<i32>,
    pub available_hairs: Vec<i32>,
    pub available_fhairs: Vec<i32>,
    pub is_parts_based: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct AppearanceOption {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct CharacterModelParts {
    pub body_parts: Vec<String>,
    pub head_resref: String,
    pub hair_resref: Option<String>,
    pub fhair_resref: Option<String>,
    pub skeleton_resref: String,
    pub wings_resref: Option<String>,
    pub tail_resref: Option<String>,
}

impl Character {
    pub fn appearance_type(&self) -> i32 {
        self.get_i32("Appearance_Type").unwrap_or(0)
    }

    pub fn appearance_head(&self) -> i32 {
        self.get_i32("Appearance_Head").unwrap_or(0)
    }

    pub fn set_appearance_head(&mut self, value: i32) {
        self.set_byte("Appearance_Head", value as u8);
    }

    pub fn appearance_hair(&self) -> i32 {
        self.get_i32("Appearance_Hair").unwrap_or(0)
    }

    pub fn set_appearance_hair(&mut self, value: i32) {
        self.set_byte("Appearance_Hair", value as u8);
    }

    pub fn appearance_fhair(&self) -> i32 {
        self.get_i32("Appearance_FHair").unwrap_or(0)
    }

    pub fn set_appearance_fhair(&mut self, value: i32) {
        self.set_byte("Appearance_FHair", value as u8);
    }

    pub fn color_tattoo1(&self) -> i32 {
        self.get_i32("Color_Tattoo1").unwrap_or(0)
    }

    pub fn set_color_tattoo1(&mut self, value: i32) {
        self.set_byte("Color_Tattoo1", value as u8);
    }

    pub fn color_tattoo2(&self) -> i32 {
        self.get_i32("Color_Tattoo2").unwrap_or(0)
    }

    pub fn set_color_tattoo2(&mut self, value: i32) {
        self.set_byte("Color_Tattoo2", value as u8);
    }

    pub fn soundset(&self) -> i32 {
        self.get_i32("SoundSetFile").unwrap_or(0)
    }

    pub fn set_soundset(&mut self, value: i32) {
        self.set_u16("SoundSetFile", value as u16);
    }

    pub fn wings(&self) -> i32 {
        self.get_i32("Wings_NewID")
            .or_else(|| self.get_i32("Wings"))
            .unwrap_or(0)
    }

    pub fn set_wings(&mut self, value: i32) {
        self.set_i32("Wings_NewID", value);
    }

    pub fn tail(&self) -> i32 {
        self.get_i32("Tail_NewID")
            .or_else(|| self.get_i32("Tail"))
            .unwrap_or(0)
    }

    pub fn set_tail(&mut self, value: i32) {
        self.set_i32("Tail_NewID", value);
    }

    pub fn body_part_value(&self, gff_field: &str) -> i32 {
        self.get_i32(gff_field).unwrap_or(0)
    }

    pub fn set_body_part_value(&mut self, gff_field: &str, value: i32) {
        self.set_byte(gff_field, value as u8);
    }

    // -- Tint reading --

    fn read_tint_channel(fields: &IndexMap<String, GffValue<'static>>) -> TintChannel {
        let get_byte = |key: &str| -> u8 {
            match fields.get(key) {
                Some(GffValue::Byte(v)) => *v,
                _ => 0,
            }
        };
        TintChannel {
            r: get_byte("r"),
            g: get_byte("g"),
            b: get_byte("b"),
            a: get_byte("a"),
        }
    }

    fn read_tint_from_tintable(tintable: &IndexMap<String, GffValue<'static>>) -> TintChannels {
        let tint = match tintable.get("Tint") {
            Some(GffValue::StructOwned(s)) => s.as_ref().clone(),
            Some(GffValue::Struct(lazy)) => lazy.force_load(),
            _ => return TintChannels::default(),
        };
        let ch = |key: &str| -> TintChannel {
            match tint.get(key) {
                Some(GffValue::StructOwned(s)) => Self::read_tint_channel(s),
                Some(GffValue::Struct(lazy)) => Self::read_tint_channel(&lazy.force_load()),
                _ => TintChannel::default(),
            }
        };
        TintChannels {
            channel1: ch("1"),
            channel2: ch("2"),
            channel3: ch("3"),
        }
    }

    fn read_tint_channels_nested(&self, field: &str) -> TintChannels {
        let Some(outer) = self.get_struct_owned(field) else {
            return TintChannels::default();
        };
        let tintable = match outer.get("Tintable") {
            Some(GffValue::StructOwned(s)) => s.as_ref().clone(),
            Some(GffValue::Struct(lazy)) => lazy.force_load(),
            _ => return TintChannels::default(),
        };
        Self::read_tint_from_tintable(&tintable)
    }

    pub fn tint_head(&self) -> TintChannels {
        self.read_tint_channels_nested("Tint_Head")
    }

    pub fn tint_hair(&self) -> TintChannels {
        self.read_tint_channels_nested("Tint_Hair")
    }

    // -- Tint writing --

    fn build_tint_channel_struct(ch: &TintChannel) -> IndexMap<String, GffValue<'static>> {
        let mut map = IndexMap::new();
        map.insert("r".to_string(), GffValue::Byte(ch.r));
        map.insert("g".to_string(), GffValue::Byte(ch.g));
        map.insert("b".to_string(), GffValue::Byte(ch.b));
        map.insert("a".to_string(), GffValue::Byte(ch.a));
        map
    }

    fn build_tint_struct(tints: &TintChannels) -> IndexMap<String, GffValue<'static>> {
        let mut tint_map = IndexMap::new();
        tint_map.insert(
            "1".to_string(),
            GffValue::StructOwned(Box::new(Self::build_tint_channel_struct(&tints.channel1))),
        );
        tint_map.insert(
            "2".to_string(),
            GffValue::StructOwned(Box::new(Self::build_tint_channel_struct(&tints.channel2))),
        );
        tint_map.insert(
            "3".to_string(),
            GffValue::StructOwned(Box::new(Self::build_tint_channel_struct(&tints.channel3))),
        );
        tint_map
    }

    fn build_nested_tint(tints: &TintChannels) -> IndexMap<String, GffValue<'static>> {
        let mut tintable = IndexMap::new();
        tintable.insert(
            "Tint".to_string(),
            GffValue::StructOwned(Box::new(Self::build_tint_struct(tints))),
        );
        let mut outer = IndexMap::new();
        outer.insert(
            "Tintable".to_string(),
            GffValue::StructOwned(Box::new(tintable)),
        );
        outer
    }

    pub fn set_tint_head(&mut self, tints: &TintChannels) {
        let nested = Self::build_nested_tint(tints);
        self.set_struct("Tint_Head", nested);
    }

    pub fn set_tint_hair(&mut self, tints: &TintChannels) {
        let nested = Self::build_nested_tint(tints);
        self.set_struct("Tint_Hair", nested);
    }

    // -- ModelScale --

    pub fn model_scale(&self) -> f32 {
        let Some(scale_struct) = self.get_struct_owned("ModelScale") else {
            return 1.0;
        };
        match scale_struct.get("x") {
            Some(GffValue::Float(v)) => *v,
            _ => 1.0,
        }
    }

    pub fn set_model_scale(&mut self, scale: f32) {
        let mut map = IndexMap::new();
        map.insert("x".to_string(), GffValue::Float(scale));
        map.insert("y".to_string(), GffValue::Float(scale));
        map.insert("z".to_string(), GffValue::Float(scale));
        self.set_struct("ModelScale", map);
    }

    fn resolve_label_from_2da(game_data: &GameData, table_name: &str, row_id: i32) -> String {
        let Some(table) = game_data.get_table(table_name) else {
            return format!("{row_id}");
        };
        let Some(row) = table.get_by_id(row_id) else {
            return format!("{row_id}");
        };
        row_str(&row, "label").unwrap_or_else(|| format!("{row_id}"))
    }

    fn discover_available_variants(
        resource_manager: &crate::services::resource_manager::ResourceManager,
        prefix: &str,
    ) -> Vec<i32> {
        let mdbs = resource_manager.list_resources_by_prefix(&prefix.to_lowercase(), "mdb");
        let prefix_lower = prefix.to_lowercase();
        let mut variants: Vec<i32> = mdbs
            .iter()
            .filter_map(|filename| {
                let name = filename.trim_end_matches(".mdb");
                let num_str = name.strip_prefix(&prefix_lower)?;
                num_str.parse::<i32>().ok()
            })
            .collect();
        variants.sort_unstable();
        variants.dedup();
        variants
    }

    pub fn get_appearance_state(
        &self,
        game_data: &GameData,
        resource_manager: &crate::services::resource_manager::ResourceManager,
    ) -> AppearanceState {
        let gender_id = self.gender();
        let gender_name = match gender_id {
            0 => "Male",
            1 => "Female",
            _ => "Unknown",
        }
        .to_string();

        let wings_id = self.wings();
        let wings_name = Self::resolve_label_from_2da(game_data, "wingmodel", wings_id);

        let tail_id = self.tail();
        let tail_name = Self::resolve_label_from_2da(game_data, "tailmodel", tail_id);

        let (available_heads, available_hairs, available_fhairs, is_parts_based) =
            self.discover_model_variants(game_data, resource_manager);

        AppearanceState {
            race_id: self.race_id().0,
            race_name: self.race_name(game_data),
            gender: gender_id,
            gender_name,

            appearance_head: self.appearance_head(),
            appearance_hair: self.appearance_hair(),
            appearance_fhair: self.appearance_fhair(),

            tint_head: self.tint_head(),
            tint_hair: self.tint_hair(),

            color_tattoo1: self.color_tattoo1(),
            color_tattoo2: self.color_tattoo2(),

            model_scale: self.model_scale(),

            soundset: self.soundset(),

            wings: wings_id,
            wings_name,
            tail: tail_id,
            tail_name,

            available_heads,
            available_hairs,
            available_fhairs,
            is_parts_based,
        }
    }

    fn discover_model_variants(
        &self,
        game_data: &GameData,
        resource_manager: &crate::services::resource_manager::ResourceManager,
    ) -> (Vec<i32>, Vec<i32>, Vec<i32>, bool) {
        let appearance_id = self.appearance_type();
        let Some(appearance_table) = game_data.get_table("appearance") else {
            return (Vec::new(), Vec::new(), Vec::new(), false);
        };
        let Some(row) = appearance_table.get_by_id(appearance_id) else {
            return (Vec::new(), Vec::new(), Vec::new(), false);
        };

        let model_type = row_str(&row, "modeltype").unwrap_or_default();
        if model_type.to_uppercase() != "P" {
            return (Vec::new(), Vec::new(), Vec::new(), false);
        }

        let gender_id = self.gender();
        let gender_letter = if let Some(gender_table) = game_data.get_table("gender")
            && let Some(gender_row) = gender_table.get_by_id(gender_id)
        {
            row_str(&gender_row, "gender").unwrap_or_else(|| "M".to_string())
        } else {
            "M".to_string()
        };

        let head_prefix = row_str(&row, "nwn2_model_head")
            .unwrap_or_default()
            .replace('?', &gender_letter);
        let hair_prefix = row_str(&row, "nwn2_model_hair")
            .unwrap_or_default()
            .replace('?', &gender_letter);
        let fhair_prefix = head_prefix.replace("Head", "FHair");

        let available_heads = Self::discover_available_variants(resource_manager, &head_prefix);
        let available_hairs = Self::discover_available_variants(resource_manager, &hair_prefix);
        let available_fhairs = Self::discover_available_variants(resource_manager, &fhair_prefix);

        (available_heads, available_hairs, available_fhairs, true)
    }

    pub fn get_available_options_from_2da(
        game_data: &GameData,
        table_name: &str,
    ) -> Vec<AppearanceOption> {
        let Some(table) = game_data.get_table(table_name) else {
            return Vec::new();
        };

        let mut options = Vec::new();
        for i in 0..table.row_count() {
            let id = i as i32;
            if let Some(row) = table.get_by_id(id)
                && let Some(label) = row_str(&row, "label")
            {
                options.push(AppearanceOption { id, name: label });
            }
        }
        options
    }

    pub fn resolve_model_parts(&self, game_data: &GameData) -> Option<CharacterModelParts> {
        let appearance_id = self.appearance_type();
        let appearance_table = game_data.get_table("appearance")?;
        let row = appearance_table.get_by_id(appearance_id)?;

        let gender_id = self.gender();
        let gender_table = game_data.get_table("gender")?;
        let gender_row = gender_table.get_by_id(gender_id)?;
        let gender_letter = row_str(&gender_row, "gender").unwrap_or_else(|| "M".to_string());

        let body_template = row_str(&row, "nwn2_model_body")?;
        let head_template = row_str(&row, "nwn2_model_head")?;
        let skel_template = row_str(&row, "nwn2_skeleton_file")?;

        let body_prefix = body_template.replace('?', &gender_letter);
        let head_prefix = head_template.replace('?', &gender_letter);
        let skeleton_resref = skel_template.replace('?', &gender_letter);

        let model_type = row_str(&row, "modeltype").unwrap_or_default();

        let body_parts = if model_type.to_uppercase() == "P" {
            // Parts-based: naked body is {prefix}_NK_Body01
            vec![format!("{body_prefix}_NK_Body01")]
        } else {
            // Simple model: single MDB
            vec![body_prefix]
        };

        let head_id = self.appearance_head();
        let head_resref = format!("{head_prefix}{head_id:02}");

        let hair_resref = row_str(&row, "nwn2_model_hair").map(|hair_template| {
            let hair_prefix = hair_template.replace('?', &gender_letter);
            let hair_id = self.appearance_hair();
            format!("{hair_prefix}{hair_id:02}")
        });

        let fhair_resref = if self.appearance_fhair() > 0 {
            let fhair_prefix = head_prefix.replace("Head", "FHair");
            Some(format!("{}{:02}", fhair_prefix, self.appearance_fhair()))
        } else {
            None
        };

        let wings_resref = if self.wings() > 0 {
            let wing_table = game_data.get_table("wingmodel")?;
            let wing_row = wing_table.get_by_id(self.wings())?;
            row_str(&wing_row, "model")
        } else {
            None
        };

        let tail_resref = if self.tail() > 0 {
            let tail_table = game_data.get_table("tailmodel")?;
            let tail_row = tail_table.get_by_id(self.tail())?;
            row_str(&tail_row, "model")
        } else {
            None
        };

        Some(CharacterModelParts {
            body_parts,
            head_resref,
            hair_resref,
            fhair_resref,
            skeleton_resref,
            wings_resref,
            tail_resref,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::Character;
    use crate::parsers::gff::GffValue;
    use indexmap::IndexMap;

    fn create_test_character() -> Character {
        let mut fields = IndexMap::new();
        fields.insert("Appearance_Head".to_string(), GffValue::Byte(3));
        fields.insert("Appearance_Hair".to_string(), GffValue::Byte(2));
        fields.insert("Appearance_FHair".to_string(), GffValue::Byte(1));
        fields.insert("Color_Tattoo1".to_string(), GffValue::Byte(2));
        fields.insert("Color_Tattoo2".to_string(), GffValue::Byte(7));
        fields.insert("SoundSetFile".to_string(), GffValue::Word(42));
        fields.insert("Wings_NewID".to_string(), GffValue::Int(0));
        fields.insert("Tail_NewID".to_string(), GffValue::Int(0));
        fields.insert("Race".to_string(), GffValue::Byte(1));
        fields.insert("Gender".to_string(), GffValue::Byte(0));
        Character::from_gff(fields)
    }

    #[test]
    fn test_appearance_head() {
        let character = create_test_character();
        assert_eq!(character.appearance_head(), 3);
    }

    #[test]
    fn test_set_appearance_head() {
        let mut character = create_test_character();
        character.set_appearance_head(7);
        assert_eq!(character.appearance_head(), 7);
        assert!(character.is_modified());
    }

    #[test]
    fn test_appearance_hair() {
        let character = create_test_character();
        assert_eq!(character.appearance_hair(), 2);
    }

    #[test]
    fn test_appearance_fhair() {
        let character = create_test_character();
        assert_eq!(character.appearance_fhair(), 1);
    }

    #[test]
    fn test_set_appearance_fhair() {
        let mut character = create_test_character();
        character.set_appearance_fhair(3);
        assert_eq!(character.appearance_fhair(), 3);
        assert!(character.is_modified());
    }

    #[test]
    fn test_tattoo_colors() {
        let character = create_test_character();
        assert_eq!(character.color_tattoo1(), 2);
        assert_eq!(character.color_tattoo2(), 7);
    }

    #[test]
    fn test_model_scale_default() {
        let character = create_test_character();
        assert!((character.model_scale() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_model_scale() {
        let mut character = create_test_character();
        character.set_model_scale(0.85);
        assert!((character.model_scale() - 0.85).abs() < f32::EPSILON);
        assert!(character.is_modified());
    }

    #[test]
    fn test_tint_head_default() {
        let character = create_test_character();
        let tint = character.tint_head();
        assert_eq!(tint.channel1.r, 0);
        assert_eq!(tint.channel1.g, 0);
        assert_eq!(tint.channel1.b, 0);
    }

    #[test]
    fn test_set_tint_head_roundtrip() {
        let mut character = create_test_character();
        let tints = TintChannels {
            channel1: TintChannel {
                r: 255,
                g: 219,
                b: 212,
                a: 0,
            },
            channel2: TintChannel {
                r: 252,
                g: 146,
                b: 32,
                a: 0,
            },
            channel3: TintChannel {
                r: 48,
                g: 43,
                b: 42,
                a: 0,
            },
        };
        character.set_tint_head(&tints);
        let read_back = character.tint_head();
        assert_eq!(read_back.channel1.r, 255);
        assert_eq!(read_back.channel1.g, 219);
        assert_eq!(read_back.channel1.b, 212);
        assert_eq!(read_back.channel2.r, 252);
        assert_eq!(read_back.channel3.r, 48);
        assert!(character.is_modified());
    }

    #[test]
    fn test_set_tint_hair_roundtrip() {
        let mut character = create_test_character();
        let tints = TintChannels {
            channel1: TintChannel {
                r: 127,
                g: 93,
                b: 84,
                a: 0,
            },
            channel2: TintChannel {
                r: 114,
                g: 31,
                b: 0,
                a: 0,
            },
            channel3: TintChannel {
                r: 164,
                g: 53,
                b: 0,
                a: 0,
            },
        };
        character.set_tint_hair(&tints);
        let read_back = character.tint_hair();
        assert_eq!(read_back.channel1.r, 127);
        assert_eq!(read_back.channel2.g, 31);
        assert_eq!(read_back.channel3.r, 164);
    }

    #[test]
    fn test_soundset() {
        let character = create_test_character();
        assert_eq!(character.soundset(), 42);
    }

    #[test]
    fn test_wings_and_tail() {
        let character = create_test_character();
        assert_eq!(character.wings(), 0);
        assert_eq!(character.tail(), 0);
    }

    #[test]
    fn test_set_wings_and_tail() {
        let mut character = create_test_character();
        character.set_wings(2);
        character.set_tail(3);
        assert_eq!(character.wings(), 2);
        assert_eq!(character.tail(), 3);
    }

    #[test]
    fn test_body_part_value() {
        let mut character = create_test_character();
        assert_eq!(character.body_part_value("BodyPart_Torso"), 0);
        character.set_body_part_value("BodyPart_Torso", 5);
        assert_eq!(character.body_part_value("BodyPart_Torso"), 5);
        assert!(character.is_modified());
    }
}
