use std::path::{Path, PathBuf};
use std::sync::Arc;

use app_lib::character::Character;
use app_lib::config::NWN2Paths;
use app_lib::character::types::SkillId;
use app_lib::loaders::GameData;
use app_lib::loaders::data_model_loader::DataModelLoader;
use app_lib::parsers::gff::{GffParser, GffValue};
use app_lib::services::item_property_decoder::ItemPropertyDecoder;
use app_lib::services::resource_manager::ResourceManager;
use app_lib::services::savegame_handler::SaveGameHandler;
use app_lib::state::session_state::SessionState;
use chrono::NaiveDateTime;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tokio::sync::RwLock;

fn parse_save_datetime(name: &str) -> Option<NaiveDateTime> {
    let (_, rest) = name.split_once(" - ")?;
    NaiveDateTime::parse_from_str(rest, "%d-%m-%Y-%H-%M").ok()
}

fn player_count(save_dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let handler = SaveGameHandler::new(save_dir, false, false)?;
    let data = handler.extract_player_data()?;
    let gff = GffParser::from_bytes(data)?;
    let root = gff.read_struct_fields(0)?;
    let count = match root.get("Mod_PlayerList") {
        Some(GffValue::List(entries)) => entries.len(),
        Some(GffValue::ListOwned(entries)) => entries.len(),
        _ => 0,
    };
    Ok(count)
}

fn is_dwarf_fighter_one(character: &Character, game_data: &GameData) -> bool {
    let race_name = character.race_name(game_data).to_lowercase();
    let is_dwarf = race_name.contains("dwarf");
    let class_entries = character.class_entries();
    let is_single_fighter = class_entries.len() == 1
        && class_entries[0].level == 1
        && character.get_class_name(class_entries[0].class_id, game_data) == "Fighter";
    is_dwarf && is_single_fighter
}

fn gff_to_json_primitive(v: &GffValue<'static>) -> Option<JsonValue> {
    match v {
        GffValue::Byte(n) => Some(JsonValue::from(*n)),
        GffValue::Char(n) => Some(JsonValue::from(n.to_string())),
        GffValue::Word(n) => Some(JsonValue::from(*n)),
        GffValue::Short(n) => Some(JsonValue::from(*n)),
        GffValue::Dword(n) => Some(JsonValue::from(*n)),
        GffValue::Int(n) => Some(JsonValue::from(*n)),
        GffValue::Dword64(n) => Some(JsonValue::from(*n)),
        GffValue::Int64(n) => Some(JsonValue::from(*n)),
        GffValue::Float(n) => Some(JsonValue::from(*n as f64)),
        GffValue::Double(n) => Some(JsonValue::from(*n)),
        GffValue::String(s) => Some(JsonValue::from(s.to_string())),
        GffValue::ResRef(s) => Some(JsonValue::from(s.to_string())),
        _ => None,
    }
}

fn locstring_text(value: &GffValue<'_>, rm: &ResourceManager) -> Option<String> {
    match value {
        GffValue::LocString(ls) => {
            if let Some(first) = ls.substrings.first() {
                Some(first.string.to_string())
            } else if ls.string_ref >= 0 {
                Some(rm.get_string(ls.string_ref))
            } else {
                None
            }
        }
        GffValue::String(s) => Some(s.to_string()),
        GffValue::ResRef(s) => Some(s.to_string()),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = NWN2Paths::new();
    let saves_root = paths
        .saves()
        .ok_or("Could not determine NWN2 saves path")?
        .join("multiplayer");

    let paths_arc = Arc::new(RwLock::new(paths));
    let resource_manager = Arc::new(RwLock::new(ResourceManager::new(Arc::clone(&paths_arc))));
    {
        let mut rm = resource_manager.write().await;
        rm.initialize().await?;
    }
    let tlk = {
        let rm = resource_manager.read().await;
        rm.get_tlk_parser().ok_or("Missing TLK parser")?
    };
    let mut loader = DataModelLoader::with_options(Arc::clone(&resource_manager), true, false);
    let game_data = loader.load_game_data(Arc::clone(&tlk)).await?;
    let decoder = ItemPropertyDecoder::new(Arc::clone(&resource_manager));

    let mut saves: Vec<PathBuf> = std::fs::read_dir(&saves_root)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| path.join("resgff.zip").exists())
        .collect();

    saves.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        let a_dt = parse_save_datetime(a_name);
        let b_dt = parse_save_datetime(b_name);
        b_dt.cmp(&a_dt).then_with(|| b_name.cmp(a_name))
    });

    for save_dir in saves {
        let count = player_count(&save_dir)?;
        for player_index in 0..count {
            let mut session = SessionState::new(Arc::clone(&resource_manager));
            session.load_character(
                save_dir.to_str().ok_or("Invalid UTF-8 path")?,
                Some(player_index),
            )?;

            let character = session.character.as_ref().ok_or("No character loaded")?;
            if !is_dwarf_fighter_one(character, &game_data) {
                continue;
            }

            println!("save={}", save_dir.display());
            println!("player_index={player_index}");
            println!("name={}", character.full_name());
            println!("race={}", character.race_name(&game_data));
            println!("natural_ac={}", character.natural_ac());
            println!("creature_size={}", character.creature_size());
            println!(
                "race_table_size={:?}",
                character.get_race_size(character.race_id().0, &game_data)
            );
            println!("dex={}", character.get_effective_abilities(&game_data).dex);
            println!("tumble_rank={}", character.skill_rank(SkillId(21)));
            println!(
                "armor_max_dex={}",
                character.get_equipped_armor_max_dex(&game_data)
            );
            let item_bonuses = character.get_equipment_bonuses(&game_data, &decoder);
            println!(
                "item_bonuses armor={} shield={} natural={} deflection={} dodge={} misc={}",
                item_bonuses.ac_armor_bonus,
                item_bonuses.ac_shield_bonus,
                item_bonuses.ac_natural_bonus,
                item_bonuses.ac_deflection_bonus,
                item_bonuses.ac_dodge_bonus,
                item_bonuses.ac_bonus
            );
            let ac = character.get_armor_class(&game_data, &decoder);
            println!("ac_total={}", ac.total);
            println!(
                "ac_breakdown base={} armor={} shield={} dex={} natural={} dodge={} deflection={} size={} misc={}",
                ac.breakdown.base,
                ac.breakdown.armor,
                ac.breakdown.shield,
                ac.breakdown.dex,
                ac.breakdown.natural,
                ac.breakdown.dodge,
                ac.breakdown.deflection,
                ac.breakdown.size,
                ac.breakdown.misc
            );
            println!("feat_ac={}", character.get_feat_ac_bonuses(&game_data));
            println!("ac_relevant_feats:");
            if let Some(feats_table) = game_data.get_table("feat") {
                for entry in character.feat_entries() {
                    if let Some(feat_data) = feats_table.get_by_id(entry.feat_id.0) {
                        let label = feat_data
                            .get("label")
                            .and_then(|s| s.as_ref())
                            .map_or("", |v| v)
                            .to_string();
                        let name = feat_data
                            .get("name")
                            .and_then(|s| s.as_ref())
                            .and_then(|s| s.parse::<i32>().ok())
                            .and_then(|r| game_data.get_string(r))
                            .unwrap_or_else(|| label.clone());
                        let hay = format!(
                            "{} {}",
                            label.to_ascii_lowercase(),
                            name.to_ascii_lowercase()
                        );
                        if hay.contains("armor")
                            || hay.contains("shield")
                            || hay.contains("dodge")
                            || hay.contains("mobility")
                            || hay.contains("expertise")
                            || hay.contains("deflect")
                            || hay.contains("ac ")
                            || hay.contains("armor class")
                        {
                            println!("  feat_id={} label={} name={}", entry.feat_id.0, label, name);
                        }
                    }
                }
            }
            println!("ac_related_fields:");
            for key in character.field_names() {
                let lower = key.to_ascii_lowercase();
                if lower.contains("ac") || lower.contains("armor") {
                    if let Some(value) = character.gff().get(key) {
                        println!("  {key}={value:?}");
                    }
                }
            }
            if let Some(GffValue::ListOwned(items)) = character.gff().get("Equip_ItemList") {
                let rm = resource_manager.read().await;
                for item in items {
                    let struct_id = item
                        .get("__struct_id__")
                        .and_then(|v| match v {
                            GffValue::Dword(v) => Some(*v),
                            GffValue::Int(v) => Some(*v as u32),
                            _ => None,
                        })
                        .unwrap_or(0);
                    if struct_id == 0 {
                        continue;
                    }
                    let base_item = item
                        .get("BaseItem")
                        .and_then(|v| match v {
                            GffValue::Int(v) => Some(*v),
                            GffValue::Word(v) => Some(*v as i32),
                            _ => None,
                        })
                        .unwrap_or(-1);
                    let name = item
                        .get("LocalizedName")
                        .and_then(|v| locstring_text(v, &rm))
                        .unwrap_or_else(|| "<unnamed>".to_string());
                    println!("item slot=0x{struct_id:04x} base_item={base_item} name={name}");
                    if let Some(GffValue::ListOwned(props)) = item.get("PropertiesList") {
                        let mut prop_maps: Vec<HashMap<String, JsonValue>> = Vec::new();
                        for prop in props {
                            let mut map = HashMap::new();
                            for (k, v) in prop {
                                if let Some(json_val) = gff_to_json_primitive(v) {
                                    map.insert(k.clone(), json_val);
                                }
                            }
                            prop_maps.push(map);
                        }
                        let decoded = decoder.decode_all_properties(&prop_maps);
                        for prop in decoded {
                            println!(
                                "  prop id={} label={} desc={} type={} ac_type={:?} bonus={:?}",
                                prop.property_id,
                                prop.label,
                                prop.description,
                                prop.bonus_type,
                                prop.ac_type,
                                prop.bonus_value
                            );
                        }
                    }
                }
            }
            if let Some(GffValue::ListOwned(effects)) = character.gff().get("EffectList") {
                println!("effect_count={}", effects.len());
                for (idx, effect) in effects.iter().enumerate() {
                    println!("effect[{idx}]={effect:?}");
                }
            } else {
                println!("effect_count=0");
            }
            return Ok(());
        }
    }

    Ok(())
}
