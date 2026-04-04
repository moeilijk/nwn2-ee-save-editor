use std::sync::Arc;

use app_lib::config::NWN2Paths;
use app_lib::loaders::data_model_loader::DataModelLoader;
use app_lib::services::resource_manager::ResourceManager;
use app_lib::state::session_state::SessionState;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = std::env::args()
        .nth(1)
        .ok_or("usage: inspect_appearance_size <save-path> <player-index>")?;
    let player_index = std::env::args()
        .nth(2)
        .ok_or("usage: inspect_appearance_size <save-path> <player-index>")?
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

    let appearance_type = character.get_i32("Appearance_Type").unwrap_or(-1);
    println!("name={}", character.full_name());
    println!("race={}", character.race_name(&game_data));
    println!("appearance_type={appearance_type}");
    println!("creature_size={}", character.creature_size());

    if let Some(table) = game_data.get_table("appearance")
        && let Some(row) = table.get_by_id(appearance_type)
    {
        println!("appearance_row={row:?}");
    } else {
        println!("appearance_row=<missing>");
    }

    Ok(())
}
