use std::path::{Path, PathBuf};
use std::sync::Arc;

use app_lib::character::Character;
use app_lib::config::NWN2Paths;
use app_lib::loaders::GameData;
use app_lib::loaders::data_model_loader::DataModelLoader;
use app_lib::parsers::gff::{GffParser, GffValue};
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

fn is_dwarf_fighter_one(character: &Character, game_data: &GameData) -> bool {
    let race_name = character.race_name(game_data).to_lowercase();
    let is_dwarf = race_name.contains("dwarf");
    let class_entries = character.class_entries();
    let is_single_fighter = class_entries.len() == 1
        && class_entries[0].level == 1
        && character.get_class_name(class_entries[0].class_id, game_data) == "Fighter";
    is_dwarf && is_single_fighter
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

            let items = character
                .get_list_owned("Equip_ItemList")
                .ok_or("missing equip list")?;
            for item in items {
                let base_item = item
                    .get("BaseItem")
                    .and_then(|v| match v {
                        GffValue::Int(v) => Some(*v),
                        GffValue::Word(v) => Some(*v as i32),
                        _ => None,
                    })
                    .unwrap_or(-1);
                if ![16, 57, 78, 52, 19].contains(&base_item) {
                    continue;
                }
                println!("BASE_ITEM={base_item}");
                for (k, v) in &item {
                    println!("  {k}={v:?}");
                }
            }
            return Ok(());
        }
    }

    Ok(())
}
