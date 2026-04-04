use std::sync::Arc;

use app_lib::config::NWN2Paths;
use app_lib::services::resource_manager::ResourceManager;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = NWN2Paths::new();
    let paths_arc = Arc::new(RwLock::new(paths));
    let resource_manager = Arc::new(RwLock::new(ResourceManager::new(Arc::clone(&paths_arc))));
    {
        let mut rm = resource_manager.write().await;
        rm.initialize().await?;
    }

    let rm = resource_manager.read().await;

    if let Ok(table) = rm.get_2da("racialtypes") {
        println!("RACIALTYPES:");
        for idx in 0..table.row_count().min(16) {
            if let Ok(row) = table.get_row_dict(idx) {
                println!("  [{idx}]={row:?}");
            }
        }
    }

    if let Ok(table) = rm.get_2da("racialsubtypes") {
        println!("RACIALSUBTYPES:");
        for idx in 0..table.row_count().min(24) {
            if let Ok(row) = table.get_row_dict(idx) {
                println!("  [{idx}]={row:?}");
            }
        }
    }

    Ok(())
}
