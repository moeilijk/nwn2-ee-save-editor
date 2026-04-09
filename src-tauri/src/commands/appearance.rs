use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use tracing::{debug, warn};

use crate::character::{AppearanceOption, AppearanceState, Character, TintChannels};
use crate::commands::{CommandError, CommandResult};
use crate::services::model_loader::{self, MeshData, ModelData};
use crate::state::AppState;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct AppearanceUpdates {
    pub appearance_head: Option<i32>,
    pub appearance_hair: Option<i32>,
    pub appearance_fhair: Option<i32>,
    pub tint_head: Option<TintChannels>,
    pub tint_hair: Option<TintChannels>,
    pub color_tattoo1: Option<i32>,
    pub color_tattoo2: Option<i32>,
    pub model_scale: Option<f32>,
    pub soundset: Option<i32>,
    pub wings: Option<i32>,
    pub tail: Option<i32>,
}

#[tauri::command]
pub async fn get_appearance_state(state: State<'_, AppState>) -> CommandResult<AppearanceState> {
    let rm = state.resource_manager.read().await;
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    Ok(character.get_appearance_state(&game_data, &rm))
}

#[tauri::command]
pub async fn update_appearance(
    state: State<'_, AppState>,
    updates: AppearanceUpdates,
) -> CommandResult<AppearanceState> {
    let rm = state.resource_manager.read().await;

    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;

    if let Some(v) = updates.appearance_head {
        character.set_appearance_head(v);
    }
    if let Some(v) = updates.appearance_hair {
        character.set_appearance_hair(v);
    }
    if let Some(v) = updates.appearance_fhair {
        character.set_appearance_fhair(v);
    }
    if let Some(ref tints) = updates.tint_head {
        character.set_tint_head(tints);
    }
    if let Some(ref tints) = updates.tint_hair {
        character.set_tint_hair(tints);
    }
    if let Some(v) = updates.color_tattoo1 {
        character.set_color_tattoo1(v);
    }
    if let Some(v) = updates.color_tattoo2 {
        character.set_color_tattoo2(v);
    }
    if let Some(v) = updates.model_scale {
        character.set_model_scale(v);
    }
    if let Some(v) = updates.soundset {
        character.set_soundset(v);
    }
    if let Some(v) = updates.wings {
        character.set_wings(v);
    }
    if let Some(v) = updates.tail {
        character.set_tail(v);
    }

    let game_data = state.game_data.read();
    Ok(character.get_appearance_state(&game_data, &rm))
}

#[tauri::command]
pub async fn get_available_wings(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AppearanceOption>> {
    let game_data = state.game_data.read();
    Ok(Character::get_available_options_from_2da(
        &game_data,
        "wingmodel",
    ))
}

#[tauri::command]
pub async fn get_available_tails(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AppearanceOption>> {
    let game_data = state.game_data.read();
    Ok(Character::get_available_options_from_2da(
        &game_data,
        "tailmodel",
    ))
}

#[tauri::command]
pub fn load_character_model(state: State<'_, AppState>) -> CommandResult<ModelData> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    let parts = character
        .resolve_model_parts(&game_data)
        .ok_or_else(|| CommandError::Internal("Failed to resolve character model parts".into()))?;

    debug!(
        "Resolved model parts: {} body parts, head={}, hair={:?}, skel={}",
        parts.body_parts.len(),
        parts.head_resref,
        parts.hair_resref,
        parts.skeleton_resref
    );

    let rm = state.resource_manager.blocking_read();

    let mut all_meshes: Vec<MeshData> = Vec::new();
    let mut skeleton = None;

    for part_resref in &parts.body_parts {
        match model_loader::load_model_with_skeleton(&rm, part_resref, &parts.skeleton_resref) {
            Ok(part_data) => {
                if skeleton.is_none() {
                    skeleton = part_data.skeleton;
                }
                all_meshes.extend(part_data.meshes);
            }
            Err(e) => debug!("Body part '{}' not found: {}", part_resref, e),
        }
    }

    match model_loader::load_model_with_skeleton(&rm, &parts.head_resref, &parts.skeleton_resref) {
        Ok(head_data) => {
            if skeleton.is_none() {
                skeleton = head_data.skeleton;
            }
            all_meshes.extend(head_data.meshes);
        }
        Err(e) => warn!("Failed to load head model '{}': {}", parts.head_resref, e),
    }

    if let Some(ref hair_resref) = parts.hair_resref {
        match model_loader::load_model_with_skeleton(&rm, hair_resref, &parts.skeleton_resref) {
            Ok(hair_data) => all_meshes.extend(hair_data.meshes),
            Err(e) => debug!("Hair model '{}' not found: {}", hair_resref, e),
        }
    }

    if let Some(ref fhair_resref) = parts.fhair_resref {
        match model_loader::load_model_with_skeleton(&rm, fhair_resref, &parts.skeleton_resref) {
            Ok(fhair_data) => all_meshes.extend(fhair_data.meshes),
            Err(e) => debug!("Facial hair model '{}' not found: {}", fhair_resref, e),
        }
    }

    if let Some(ref wing_resref) = parts.wings_resref {
        match model_loader::load_model(&rm, wing_resref) {
            Ok(wing_data) => all_meshes.extend(wing_data.meshes),
            Err(e) => warn!("Failed to load wing model '{}': {}", wing_resref, e),
        }
    }

    if let Some(ref tail_resref) = parts.tail_resref {
        match model_loader::load_model(&rm, tail_resref) {
            Ok(tail_data) => all_meshes.extend(tail_data.meshes),
            Err(e) => warn!("Failed to load tail model '{}': {}", tail_resref, e),
        }
    }

    if all_meshes.is_empty() {
        return Err(CommandError::Internal(
            "No model meshes could be loaded".into(),
        ));
    }

    Ok(ModelData {
        meshes: all_meshes,
        hooks: Vec::new(),
        hair: Vec::new(),
        helm: Vec::new(),
        skeleton,
    })
}
