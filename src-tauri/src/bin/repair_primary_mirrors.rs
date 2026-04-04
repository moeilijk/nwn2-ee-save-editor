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
        .ok_or("usage: repair_primary_mirrors <save-path> [player-index]")?;
    let player_index = std::env::args()
        .nth(2)
        .map(|s| s.parse::<usize>())
        .transpose()?;

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
    session.load_character(&save_path, player_index)?;
    session.normalize_loaded_skill_points(&game_data);
    let changed = session.sync_primary_mirrors(&game_data)?;
    println!("save={save_path}");
    println!("player_index={:?}", player_index);
    println!("mirrors_synced={changed}");
    Ok(())
}
