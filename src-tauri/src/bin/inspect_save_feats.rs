use std::collections::HashSet;
use std::sync::Arc;

use app_lib::character::FeatSource;
use app_lib::config::NWN2Paths;
use app_lib::loaders::data_model_loader::DataModelLoader;
use app_lib::services::resource_manager::ResourceManager;
use app_lib::state::session_state::SessionState;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = std::env::args()
        .nth(1)
        .ok_or("usage: inspect_save_feats <save-path> <player-index>")?;
    let player_index = std::env::args()
        .nth(2)
        .ok_or("usage: inspect_save_feats <save-path> <player-index>")?
        .parse::<usize>()?;

    let paths = NWN2Paths::new();
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

    let mut session = SessionState::new(Arc::clone(&resource_manager));
    session.load_character(&save_path, Some(player_index))?;
    let character = session.character.as_ref().ok_or("No character loaded")?;

    let slot_chosen = character.get_slot_chosen_feat_ids(&game_data);
    let slot_chosen_set: HashSet<_> = slot_chosen.iter().copied().collect();
    let slots = character.get_feat_slots(&game_data);

    println!("save={save_path}");
    println!("player_index={player_index}");
    println!("name={}", character.full_name());
    println!("classes={:?}", character.class_entries());
    println!(
        "feat_slots total_general={} total_bonus={} total={} filled={} open={} open_general={} open_bonus={}",
        slots.total_general_slots,
        slots.total_bonus_slots,
        slots.total_slots,
        slots.filled_slots,
        slots.open_slots,
        slots.open_general_slots,
        slots.open_bonus_slots
    );
    println!("slot_chosen_count={}", slot_chosen.len());
    println!("slot_chosen_feats:");
    for feat_id in &slot_chosen {
        if let Some(info) = character.get_feat_info(*feat_id, &game_data) {
            println!(
                "  {} | {} | category={:?} type={} protected={} custom={}",
                feat_id.0, info.label, info.category, info.feat_type.0, info.is_protected, info.is_custom
            );
        } else {
            println!("  {} | <missing>", feat_id.0);
        }
    }

    println!("top_level_feats:");
    for entry in character.feat_entries() {
        let source = match entry.source {
            FeatSource::Unknown => "unknown",
            FeatSource::Manual => "manual",
            FeatSource::Class => "class",
            FeatSource::Race => "race",
            FeatSource::Domain => "domain",
            FeatSource::Level => "level",
            FeatSource::Background => "background",
        };
        let in_slot = slot_chosen_set.contains(&entry.feat_id);
        if let Some(info) = character.get_feat_info(entry.feat_id, &game_data) {
            println!(
                "  {} | {} | category={:?} type={} source={} slot_chosen={} protected={} custom={}",
                entry.feat_id.0,
                info.label,
                info.category,
                info.feat_type.0,
                source,
                in_slot,
                info.is_protected,
                info.is_custom
            );
        } else {
            println!(
                "  {} | <missing> | source={} slot_chosen={}",
                entry.feat_id.0, source, in_slot
            );
        }
    }

    println!("top_level_only_feats:");
    for entry in character.feat_entries() {
        if slot_chosen_set.contains(&entry.feat_id) {
            continue;
        }
        let feat_id = entry.feat_id;
        if let Some(info) = character.get_feat_info(feat_id, &game_data) {
            println!(
                "  {} | {} | category={:?} type={} protected={} custom={}",
                feat_id.0, info.label, info.category, info.feat_type.0, info.is_protected, info.is_custom
            );
        } else {
            println!("  {} | <missing>", feat_id.0);
        }
    }

    println!("level_history:");
    for entry in character.level_history() {
        print!(
            "  char_level={} class_id={} class_level={} ability={:?} feats=[",
            entry.character_level,
            entry.class_id.0,
            entry.class_level,
            entry.ability_increase.map(|idx| idx.0)
        );
        for (idx, feat_id) in entry.feats_gained.iter().enumerate() {
            if idx > 0 {
                print!(", ");
            }
            print!("{}", feat_id.0);
        }
        print!("] skills=[");
        for (idx, skill) in entry.skills_gained.iter().enumerate() {
            if idx > 0 {
                print!(", ");
            }
            print!("{}={}", skill.skill_id.0, skill.ranks);
        }
        println!("]");
    }

    if let Some(classes_table) = game_data.get_table("classes") {
        for class_entry in character.class_entries() {
            if let Some(class_row) = classes_table.get_by_id(class_entry.class_id.0) {
                let feats_table_name = class_row
                    .get("FeatsTable")
                    .or_else(|| class_row.get("feats_table"))
                    .and_then(|s| s.as_ref())
                    .cloned()
                    .unwrap_or_default();
                let bonus_table_name = class_row
                    .get("BonusFeatsTable")
                    .or_else(|| class_row.get("bonus_feats_table"))
                    .and_then(|s| s.as_ref())
                    .cloned()
                    .unwrap_or_default();
                println!(
                    "class_row class_id={} feats_table={} bonus_table={}",
                    class_entry.class_id.0, feats_table_name, bonus_table_name
                );
                if !feats_table_name.is_empty()
                    && !feats_table_name.starts_with("****")
                    && let Some(table) = game_data.get_table(&feats_table_name.to_lowercase())
                {
                    println!("class_feat_rows_for_current_feats:");
                    for feat in character.feat_entries() {
                        for row_id in 0..table.row_count() {
                            let Some(row) = table.get_by_id(row_id as i32) else {
                                continue;
                            };
                            let feat_index = row
                                .get("FeatIndex")
                                .or_else(|| row.get("featindex"))
                                .and_then(|s| s.as_ref())
                                .and_then(|s| s.parse::<i32>().ok());
                            if feat_index != Some(feat.feat_id.0) {
                                continue;
                            }
                            let list = row
                                .get("List")
                                .or_else(|| row.get("list"))
                                .and_then(|s| s.as_ref())
                                .cloned()
                                .unwrap_or_default();
                            let granted = row
                                .get("GrantedOnLevel")
                                .or_else(|| row.get("grantedonlevel"))
                                .and_then(|s| s.as_ref())
                                .cloned()
                                .unwrap_or_default();
                            let on_menu = row
                                .get("OnMenu")
                                .or_else(|| row.get("onmenu"))
                                .and_then(|s| s.as_ref())
                                .cloned()
                                .unwrap_or_default();
                            println!(
                                "  feat={} row={} list={} granted_on_level={} on_menu={}",
                                feat.feat_id.0, row_id, list, granted, on_menu
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
