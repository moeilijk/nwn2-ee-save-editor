use rand::seq::SliceRandom;
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
    pub height: Option<f32>,
    pub girth: Option<f32>,
    pub soundset: Option<i32>,
    pub wings: Option<i32>,
    pub tail: Option<i32>,
    pub never_draw_helmet: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VoiceSetInfo {
    pub id: u32,
    pub name: String,
    pub gender: u8,
    pub resref: String,
    pub voice_type: u8,
}

#[tauri::command]
pub fn get_appearance_state(state: State<'_, AppState>) -> CommandResult<AppearanceState> {
    let rm = state.resource_manager.blocking_read();
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    Ok(character.get_appearance_state(&game_data, &rm))
}

#[tauri::command]
pub fn update_appearance(
    state: State<'_, AppState>,
    updates: AppearanceUpdates,
) -> CommandResult<AppearanceState> {
    let rm = state.resource_manager.blocking_read();

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
    if let Some(v) = updates.height {
        character.set_height(v);
    }
    if let Some(v) = updates.girth {
        character.set_girth(v);
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
    if let Some(v) = updates.never_draw_helmet {
        character.set_never_draw_helmet(v);
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
        "Resolved model parts: {} body parts, head={}, hair={:?}, helm={:?}, skel={}",
        parts.body_parts.len(),
        parts.head_resref,
        parts.hair_resref,
        parts.helm_candidates.first().unwrap_or(&String::new()),
        parts.skeleton_resref
    );

    let rm = state.resource_manager.blocking_read();

    let mut all_meshes: Vec<MeshData> = Vec::new();
    let mut skeleton = None;

    // Try body candidates in order, stop at first success
    let mut body_loaded = false;
    for part_resref in &parts.body_parts {
        if let Ok(part_data) = model_loader::load_model_with_skeleton(
            &rm,
            part_resref,
            &parts.skeleton_resref,
            "body",
            "body",
        ) {
            if skeleton.is_none() {
                skeleton = part_data.skeleton;
            }
            all_meshes.extend(part_data.meshes);
            body_loaded = true;
            break;
        }
    }
    if !body_loaded {
        if let Ok(part_data) = model_loader::load_model_with_skeleton(
            &rm,
            &parts.naked_body_resref,
            &parts.skeleton_resref,
            "body",
            "body",
        ) {
            if skeleton.is_none() {
                skeleton = part_data.skeleton;
            }
            all_meshes.extend(part_data.meshes);
        }
    }

    match model_loader::load_model_with_skeleton(
        &rm,
        &parts.head_resref,
        &parts.skeleton_resref,
        "head",
        "head",
    ) {
        Ok(head_data) => {
            if skeleton.is_none() {
                skeleton = head_data.skeleton;
            }
            for mut mesh in head_data.meshes {
                if mesh.name.to_lowercase().contains("_fhair") {
                    // Facial hair meshes are embedded in head MDB
                    // Show only if Appearance_FHair > 0
                    if parts.show_fhair {
                        mesh.part = "fhair".to_string();
                        mesh.tint_group = "hair".to_string();
                        all_meshes.push(mesh);
                    }
                } else {
                    all_meshes.push(mesh);
                }
            }
        }
        Err(e) => warn!("Failed to load head model '{}': {}", parts.head_resref, e),
    }

    if let Some(ref hair_resref) = parts.hair_resref {
        match model_loader::load_model_with_skeleton(
            &rm,
            hair_resref,
            &parts.skeleton_resref,
            "hair",
            "hair",
        ) {
            Ok(hair_data) => all_meshes.extend(hair_data.meshes),
            Err(e) => debug!("Hair model '{}' not found: {}", hair_resref, e),
        }
    }

    if let Some(ref wing_resref) = parts.wings_resref {
        match model_loader::load_model(&rm, wing_resref, "wings", "none") {
            Ok(wing_data) => all_meshes.extend(wing_data.meshes),
            Err(e) => warn!("Failed to load wing model '{}': {}", wing_resref, e),
        }
    }

    if let Some(ref tail_resref) = parts.tail_resref {
        match model_loader::load_model(&rm, tail_resref, "tail", "none") {
            Ok(tail_data) => all_meshes.extend(tail_data.meshes),
            Err(e) => warn!("Failed to load tail model '{}': {}", tail_resref, e),
        }
    }

    if parts.show_helmet {
        for helm_resref in &parts.helm_candidates {
            if let Ok(helm_data) = model_loader::load_model_with_skeleton(
                &rm,
                helm_resref,
                &parts.skeleton_resref,
                "helm",
                "body",
            ) {
                all_meshes.extend(helm_data.meshes);
                break;
            }
        }
    }

    if !parts.boots_candidates.is_empty() {
        let mut loaded = false;
        for boots_resref in &parts.boots_candidates {
            if let Ok(data) = model_loader::load_model_with_skeleton(
                &rm,
                boots_resref,
                &parts.skeleton_resref,
                "boots",
                "body",
            ) {
                all_meshes.extend(data.meshes);
                loaded = true;
                break;
            }
        }
        if !loaded {
            warn!(
                "Failed to load boots from candidates: {:?}",
                parts.boots_candidates
            );
        }
    }

    if !parts.gloves_candidates.is_empty() {
        let mut loaded = false;
        for gloves_resref in &parts.gloves_candidates {
            if let Ok(data) = model_loader::load_model_with_skeleton(
                &rm,
                gloves_resref,
                &parts.skeleton_resref,
                "gloves",
                "body",
            ) {
                all_meshes.extend(data.meshes);
                loaded = true;
                break;
            }
        }
        if !loaded {
            warn!(
                "Failed to load gloves from candidates: {:?}",
                parts.gloves_candidates
            );
        }
    }

    if let Some(ref cloak_resref) = parts.cloak_resref {
        match model_loader::load_model_with_skeleton(
            &rm,
            cloak_resref,
            &parts.skeleton_resref,
            "cloak",
            "body",
        ) {
            Ok(cloak_data) => all_meshes.extend(cloak_data.meshes),
            Err(e) => warn!("Failed to load cloak model '{}': {}", cloak_resref, e),
        }
    }

    if all_meshes.is_empty() {
        return Err(CommandError::Internal(
            "No model meshes could be loaded".to_string(),
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

#[tauri::command]
pub fn load_character_part(state: State<'_, AppState>, part: String) -> CommandResult<ModelData> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    let parts = character.resolve_model_parts(&game_data).ok_or_else(|| {
        CommandError::Internal("Failed to resolve character model parts".to_string())
    })?;

    let rm = state.resource_manager.blocking_read();
    let mut meshes: Vec<MeshData> = Vec::new();

    match part.as_str() {
        "head" => {
            if let Ok(data) = model_loader::load_model_with_skeleton(
                &rm,
                &parts.head_resref,
                &parts.skeleton_resref,
                "head",
                "head",
            ) {
                meshes.extend(
                    data.meshes
                        .into_iter()
                        .filter(|m| !m.name.to_lowercase().contains("_fhair")),
                );
            }
        }
        "hair" => {
            if let Some(ref resref) = parts.hair_resref
                && let Ok(data) = model_loader::load_model_with_skeleton(
                    &rm,
                    resref,
                    &parts.skeleton_resref,
                    "hair",
                    "hair",
                )
            {
                meshes.extend(data.meshes);
            }
        }
        "fhair" => {
            if parts.show_fhair
                && let Ok(data) = model_loader::load_model_with_skeleton(
                    &rm,
                    &parts.head_resref,
                    &parts.skeleton_resref,
                    "fhair",
                    "hair",
                )
            {
                for mesh in data.meshes {
                    if mesh.name.to_lowercase().contains("_fhair") {
                        meshes.push(mesh);
                    }
                }
            }
        }
        "wings" => {
            if let Some(ref resref) = parts.wings_resref
                && let Ok(data) = model_loader::load_model(&rm, resref, "wings", "none")
            {
                meshes.extend(data.meshes);
            }
        }
        "tail" => {
            if let Some(ref resref) = parts.tail_resref
                && let Ok(data) = model_loader::load_model(&rm, resref, "tail", "none")
            {
                meshes.extend(data.meshes);
            }
        }
        "helm" => {
            if parts.show_helmet {
                for resref in &parts.helm_candidates {
                    if let Ok(data) = model_loader::load_model_with_skeleton(
                        &rm,
                        resref,
                        &parts.skeleton_resref,
                        "helm",
                        "body",
                    ) {
                        meshes.extend(data.meshes);
                        break;
                    }
                }
            }
        }
        "body" => {
            let mut loaded = false;
            for part_resref in &parts.body_parts {
                if let Ok(part_data) = model_loader::load_model_with_skeleton(
                    &rm,
                    part_resref,
                    &parts.skeleton_resref,
                    "body",
                    "body",
                ) {
                    meshes.extend(part_data.meshes);
                    loaded = true;
                    break;
                }
            }
            if !loaded {
                if let Ok(part_data) = model_loader::load_model_with_skeleton(
                    &rm,
                    &parts.naked_body_resref,
                    &parts.skeleton_resref,
                    "body",
                    "body",
                ) {
                    meshes.extend(part_data.meshes);
                }
            }
        }
        "cloak" => {
            if let Some(ref resref) = parts.cloak_resref
                && let Ok(data) = model_loader::load_model_with_skeleton(
                    &rm,
                    resref,
                    &parts.skeleton_resref,
                    "cloak",
                    "body",
                )
            {
                meshes.extend(data.meshes);
            }
        }
        _ => {
            return Err(CommandError::Internal(format!("Unknown part type: {part}")));
        }
    }

    Ok(ModelData {
        meshes,
        hooks: Vec::new(),
        hair: Vec::new(),
        helm: Vec::new(),
        skeleton: None,
    })
}

#[tauri::command]
pub async fn get_available_voicesets(
    state: State<'_, AppState>,
) -> CommandResult<Vec<VoiceSetInfo>> {
    // Extract all soundset rows first so we can drop game_data before the await
    let candidates: Vec<(i32, String, u8, i32, String)> = {
        let game_data = state.game_data.read();
        let soundset_table = game_data
            .get_table("soundset")
            .ok_or_else(|| CommandError::Internal("soundset.2da not loaded".to_string()))?;

        let mut rows = Vec::new();
        for i in 0..soundset_table.row_count() {
            let id = i as i32;
            let Some(row) = soundset_table.get_by_id(id) else {
                continue;
            };
            let Some(resref) = crate::utils::parsing::row_str(&row, "resref") else {
                continue;
            };
            if resref.eq_ignore_ascii_case("none") {
                continue;
            }
            let type_val = crate::utils::parsing::row_int(&row, "type", -1) as u8;
            let gender = crate::utils::parsing::row_int(&row, "gender", 0) as u8;
            let strref = crate::utils::parsing::row_int(&row, "strref", -1);
            let label = crate::utils::parsing::row_str(&row, "label")
                .unwrap_or_else(|| format!("Voice {id}"));
            rows.push((id, resref, type_val | ((gender as u8) << 4), strref, label));
        }
        rows
    };

    let rm = state.resource_manager.read().await;
    let game_data = state.game_data.read();
    let mut voicesets = Vec::new();

    for (id, resref, packed, strref, label) in &candidates {
        let type_val = *packed & 0x0F;
        let gender = *packed >> 4;

        let Ok(ssf_data) = rm.get_resource_bytes(resref, "ssf") else {
            continue;
        };

        let has_audio = crate::parsers::ssf::parse_ssf(&ssf_data)
            .map(|wav_resrefs| {
                wav_resrefs
                    .iter()
                    .any(|wr| rm.get_resource_bytes(wr, "wav").is_ok())
            })
            .unwrap_or(false);
        if !has_audio {
            continue;
        }

        let name = if *strref >= 0 {
            game_data
                .get_string(*strref)
                .unwrap_or_else(|| label.clone())
        } else {
            label.clone()
        };

        voicesets.push(VoiceSetInfo {
            id: *id as u32,
            name,
            gender,
            resref: resref.clone(),
            voice_type: type_val,
        });
    }

    voicesets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(voicesets)
}

#[tauri::command]
pub async fn preview_voiceset(
    state: State<'_, AppState>,
    resref: String,
) -> CommandResult<Vec<u8>> {
    let rm = state.resource_manager.read().await;

    let ssf_data = rm
        .get_resource_bytes(&resref, "ssf")
        .map_err(|e| CommandError::Internal(format!("SSF file not found: {e}")))?;

    let wav_resrefs = crate::parsers::ssf::parse_ssf(&ssf_data)
        .map_err(|e| CommandError::Internal(format!("Failed to parse SSF: {e}")))?;

    if wav_resrefs.is_empty() {
        return Err(CommandError::Internal(
            "No voice lines found in SSF".to_string(),
        ));
    }

    let mut rng = rand::rng();
    let mut shuffled = wav_resrefs;
    shuffled.shuffle(&mut rng);

    for wav_resref in &shuffled {
        if let Ok(data) = rm.get_resource_bytes(wav_resref, "wav") {
            return Ok(data);
        }
    }

    Err(CommandError::Internal(format!(
        "No WAV files found for voiceset: {resref}"
    )))
}
