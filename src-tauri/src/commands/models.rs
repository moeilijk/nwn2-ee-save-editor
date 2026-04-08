use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info};

use crate::services::model_loader::{self, ModelData};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub filename: String,
    pub resref: String,
    pub zip_source: String,
}

fn strip_extension(name: &str) -> &str {
    if let Some(dot) = name.rfind('.') {
        &name[..dot]
    } else {
        name
    }
}

#[tauri::command]
pub fn load_model(state: State<'_, AppState>, resref: String) -> Result<ModelData, String> {
    info!("Loading model: {}", resref);
    let rm = state.resource_manager.blocking_read();
    match model_loader::load_model(&rm, &resref) {
        Ok(data) => {
            info!(
                "Model loaded: {} meshes, {} hooks, skeleton={}",
                data.meshes.len(),
                data.hooks.len(),
                data.skeleton.is_some()
            );
            for mesh in &data.meshes {
                debug!(
                    "  Mesh '{}' ({}): {} verts, {} indices, diffuse='{}'",
                    mesh.name,
                    mesh.mesh_type,
                    mesh.positions.len() / 3,
                    mesh.indices.len(),
                    mesh.material.diffuse_map
                );
            }
            Ok(data)
        }
        Err(e) => {
            error!("Failed to load model '{}': {}", resref, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_texture_bytes(state: State<'_, AppState>, name: String) -> Result<Vec<u8>, String> {
    debug!("Loading texture: {}", name);
    let rm = state.resource_manager.blocking_read();
    match rm.get_resource_bytes(&name, "dds") {
        Ok(bytes) => {
            debug!("Texture loaded: {} ({} bytes)", name, bytes.len());
            Ok(bytes)
        }
        Err(e) => {
            error!("Texture not found '{}': {}", name, e);
            Err(format!("Texture not found {name}: {e}"))
        }
    }
}

#[tauri::command]
pub fn list_available_models(state: State<'_, AppState>) -> Result<Vec<ModelEntry>, String> {
    info!("Scanning for available models...");
    let rm = state.resource_manager.blocking_read();
    let files = rm.list_resources_by_extension("mdb");
    let count = files.len();
    let result: Vec<ModelEntry> = files
        .into_iter()
        .map(|(filename, zip_source)| {
            let basename = filename.rsplit('/').next().unwrap_or(&filename);
            let resref = strip_extension(basename).to_string();
            ModelEntry {
                filename,
                resref,
                zip_source,
            }
        })
        .collect();
    info!("Found {} MDB models", count);
    Ok(result)
}
