use std::path::{Path, PathBuf};
use std::sync::Arc;

use app_lib::character::Character;
use app_lib::config::NWN2Paths;
use app_lib::loaders::data_model_loader::DataModelLoader;
use app_lib::parsers::gff::{GffParser, GffValue};
use app_lib::services::item_property_decoder::ItemPropertyDecoder;
use app_lib::services::resource_manager::ResourceManager;
use app_lib::services::savegame_handler::SaveGameHandler;
use app_lib::state::session_state::SessionState;
use chrono::NaiveDateTime;
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

fn has_ac_bonus_items(character: &Character) -> bool {
    let Some(GffValue::ListOwned(items)) = character.gff().get("Equip_ItemList") else {
        return false;
    };
    for item in items {
        let Some(GffValue::ListOwned(props)) = item.get("PropertiesList") else {
            continue;
        };
        for prop in props {
            let property_name = match prop.get("PropertyName") {
                Some(GffValue::Word(v)) => *v as i32,
                Some(GffValue::Int(v)) => *v,
                _ => continue,
            };
            if property_name == 1 {
                return true;
            }
        }
    }
    false
}

fn item_names_with_ac_bonus(character: &Character, rm: &ResourceManager) -> Vec<String> {
    let Some(GffValue::ListOwned(items)) = character.gff().get("Equip_ItemList") else {
        return vec![];
    };
    let mut out = Vec::new();
    for item in items {
        let Some(GffValue::ListOwned(props)) = item.get("PropertiesList") else {
            continue;
        };
        let has_ac = props.iter().any(|prop| match prop.get("PropertyName") {
            Some(GffValue::Word(v)) => *v as i32 == 1,
            Some(GffValue::Int(v)) => *v == 1,
            _ => false,
        });
        if !has_ac {
            continue;
        }
        let name = match item.get("LocalizedName") {
            Some(GffValue::LocString(ls)) => {
                if let Some(first) = ls.substrings.first() {
                    first.string.to_string()
                } else if ls.string_ref >= 0 {
                    rm.get_string(ls.string_ref)
                } else {
                    "<unnamed>".to_string()
                }
            }
            _ => "<unnamed>".to_string(),
        };
        out.push(name);
    }
    out
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = NWN2Paths::new();
    let saves_root = paths
        .saves()
        .ok_or("Could not determine NWN2 saves path")?;

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

    let mut save_dirs: Vec<PathBuf> = Vec::new();
    for root in [saves_root.to_path_buf(), saves_root.join("multiplayer")] {
        if !root.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&root)? {
            let path = entry?.path();
            if path.is_dir() && path.join("resgff.zip").exists() {
                save_dirs.push(path);
            }
        }
    }
    save_dirs.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        let a_dt = parse_save_datetime(a_name);
        let b_dt = parse_save_datetime(b_name);
        b_dt.cmp(&a_dt).then_with(|| b_name.cmp(a_name))
    });

    let rm = resource_manager.read().await;
    for save_dir in save_dirs {
        let count = match player_count(&save_dir) {
            Ok(count) => count,
            Err(_) => continue,
        };
        for player_index in 0..count {
            let mut session = SessionState::new(Arc::clone(&resource_manager));
            if session
                .load_character(save_dir.to_str().ok_or("Invalid UTF-8 path")?, Some(player_index))
                .is_err()
            {
                continue;
            }
            let Some(character) = session.character.as_ref() else {
                continue;
            };
            if !has_ac_bonus_items(character) {
                continue;
            }
            let stored_bab = character.get_i32("BaseAttackBonus").unwrap_or(-1);
            let calc_bab = character.calculate_bab(&game_data);
            if stored_bab != calc_bab {
                continue;
            }

            let stored_ac = character.get_i32("ArmorClass").unwrap_or(-1);
            let calc_ac = character.get_armor_class(&game_data, &decoder).total;
            println!(
                "save={} player_index={} name={} stored_bab={} calc_bab={} stored_ac={} calc_ac={} items={:?}",
                save_dir.display(),
                player_index,
                character.full_name(),
                stored_bab,
                calc_bab,
                stored_ac,
                calc_ac,
                item_names_with_ac_bonus(character, &rm)
            );
        }
    }

    Ok(())
}
