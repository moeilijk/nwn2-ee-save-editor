use app_lib::services::model_loader;
use app_lib::services::resource_manager::ResourceManager;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
#[ignore = "diagnostic — run manually with cargo test dump_armor_mesh_names -- --ignored --nocapture"]
async fn dump_armor_mesh_names() {
    let nwn2_paths = Arc::new(RwLock::new(app_lib::config::NWN2Paths::new()));
    let rm = Arc::new(RwLock::new(ResourceManager::new(nwn2_paths)));
    {
        let mut rm_guard = rm.write().await;
        let _ = rm_guard.initialize().await;
    }
    let rm = rm.read().await;

    for resref in [
        "P_HHM_LE_Body01",
        "P_HHM_LE_Body02",
        "P_HHM_PF_Body01",
        "P_HHM_LE_Helm01",
        "P_HHM_LE_Boots01",
        "P_HHM_LE_Gloves01",
        "P_HHM_CL_Cloak01",
        "W_LSword01_a",
    ] {
        println!("\n=== {resref} ===");
        match model_loader::load_model(&rm, resref, "item", "item") {
            Ok(data) => {
                println!("  meshes: {}", data.meshes.len());
                for (i, m) in data.meshes.iter().enumerate() {
                    println!(
                        "    [{i}] name='{}' mesh_type='{}' part='{}' verts={} tris={}",
                        m.name,
                        m.mesh_type,
                        m.part,
                        m.positions.len() / 3,
                        m.indices.len() / 3
                    );
                }
                if let Some(sk) = &data.skeleton {
                    println!("  skeleton: bones={}", sk.bones.len());
                }
            }
            Err(e) => println!("  FAILED: {e}"),
        }
    }
}
