use tauri::State;
use tracing::{debug, info};

use crate::character::{ItemAppearance, ItemAppearanceOptions};
use crate::services::model_loader::{self, ModelData};
use crate::state::AppState;

#[tauri::command]
pub fn get_item_appearance_options(
    state: State<'_, AppState>,
    base_item_id: i32,
) -> Result<ItemAppearanceOptions, String> {
    debug!("Getting appearance options for base item: {}", base_item_id);
    let game_data = state.game_data.read();
    let rm = state.resource_manager.blocking_read();

    Ok(ItemAppearance::get_options(base_item_id, &game_data, &rm))
}

#[tauri::command]
pub fn load_item_model(
    state: State<'_, AppState>,
    appearance: ItemAppearance,
    base_item_id: i32,
) -> Result<ModelData, String> {
    info!(
        "Loading item model for base item {}, appearance: {:?}",
        base_item_id, appearance
    );
    let game_data = state.game_data.read();
    let rm = state.resource_manager.blocking_read();

    let resrefs = appearance.resolve_model_resrefs(base_item_id, &game_data);

    let mut combined_data = ModelData {
        meshes: Vec::new(),
        hooks: Vec::new(),
        hair: Vec::new(),
        helm: Vec::new(),
        skeleton: None,
        animations: Vec::new(),
    };

    for resref in resrefs {
        // Tag meshes with their part letter (a/b/c) so the frontend can swap them individually.
        let part_tag = match resref.to_lowercase().as_bytes() {
            [.., b'_', b'a'] => "item_a",
            [.., b'_', b'b'] => "item_b",
            [.., b'_', b'c'] => "item_c",
            _ => "item",
        };
        debug!("Adding part to item model: {} (part={})", resref, part_tag);
        match model_loader::load_model(&rm, &resref, part_tag, "item") {
            Ok(data) => {
                combined_data.meshes.extend(data.meshes);
                combined_data.hooks.extend(data.hooks);
                combined_data.hair.extend(data.hair);
                combined_data.helm.extend(data.helm);
                // Items usually don't have skeletons, but if they do (rare), we'll take the first one
                if combined_data.skeleton.is_none() {
                    combined_data.skeleton = data.skeleton;
                    combined_data.animations = data.animations;
                }
            }
            Err(e) => {
                debug!("Failed to load part '{}': {}. Skipping.", resref, e);
            }
        }
    }

    if combined_data.meshes.is_empty() {
        return Err(format!(
            "Could not load any meshes for base item {base_item_id}"
        ));
    }

    info!(
        "Loaded item model with {} meshes",
        combined_data.meshes.len()
    );
    Ok(combined_data)
}

#[tauri::command]
pub fn load_item_part(
    state: State<'_, AppState>,
    base_item_id: i32,
    part_index: u8,
    variant: i32,
) -> Result<ModelData, String> {
    debug!(
        "Loading item part base_item_id={} part_index={} variant={}",
        base_item_id, part_index, variant
    );
    let game_data = state.game_data.read();
    let rm = state.resource_manager.blocking_read();

    let resref = ItemAppearance::resolve_weapon_part_resref(
        base_item_id,
        part_index as usize,
        variant,
        &game_data,
    )
    .ok_or_else(|| format!("No resref for part {part_index} variant {variant}"))?;

    let part_tag = match part_index {
        0 => "item_a",
        1 => "item_b",
        2 => "item_c",
        _ => "item",
    };
    model_loader::load_model(&rm, &resref, part_tag, "item")
        .map_err(|e| format!("Failed to load part '{resref}': {e}"))
}
