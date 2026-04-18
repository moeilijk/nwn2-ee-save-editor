use tauri::State;
use tracing::{debug, info};

use crate::character::{
    AccessorySlot, ItemAppearance, ItemAppearanceOptions, cosmetic_gloves_tint, swap_tint_2_3,
};
use crate::services::model_loader::{self, ModelData};
use crate::state::AppState;

fn resref_is_boots_or_gloves(resref: &str) -> bool {
    let lower = resref.to_lowercase();
    lower.contains("_boots") || lower.contains("_gloves")
}

/// Returns every MDB resref matching the slot of `base_item_id`.
///
/// For body-armor base items, derives the slot from `baseitems.2da.equipableslots`
/// and returns all `p_hhm_*_<slot>NN` MDBs that exist in the resource index.
/// For weapons / non-armor items returns an empty vector.
pub fn list_armor_mesh_candidates_impl(
    base_item_id: i32,
    game_data: &crate::loaders::GameData,
    resource_manager: &crate::services::resource_manager::ResourceManager,
) -> Vec<String> {
    use crate::character::{detect_armor_slot, parse_equip_slots};
    use crate::utils::parsing::row_str;

    let Some(table) = game_data.get_table("baseitems") else {
        return Vec::new();
    };
    let Some(row) = table.get_by_id(base_item_id) else {
        return Vec::new();
    };

    let modeltype = row_str(&row, "modeltype").unwrap_or_default();
    // Bracer items ship with modeltype=0 but render as glove meshes; let
    // them through so the debug mesh-override dropdown offers candidates.
    let label = row_str(&row, "label").unwrap_or_default();
    if modeltype.trim() != "3" && !crate::character::is_bracer_label(&label) {
        return Vec::new();
    }

    let equip_slots = parse_equip_slots(&row_str(&row, "equipableslots").unwrap_or_default());
    let Some(slot) = detect_armor_slot(equip_slots) else {
        return Vec::new();
    };
    let slot_fragment = slot.part_name().to_lowercase();

    let mdbs = resource_manager.list_resources_by_prefix("p_hhm_", "mdb");
    let needle = format!("_{slot_fragment}");
    let digits_only = |s: &str| -> bool { !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) };

    let mut out: Vec<String> = mdbs
        .into_iter()
        .filter_map(|name| {
            let stem = name.trim_end_matches(".mdb").to_string();
            let lower = stem.to_lowercase();
            let idx = lower.rfind(&needle)?;
            let after = &lower[idx + needle.len()..];
            if digits_only(after) { Some(stem) } else { None }
        })
        .collect();

    out.sort_unstable();
    out.dedup();
    out
}

#[tauri::command]
pub fn list_armor_mesh_candidates(
    state: State<'_, AppState>,
    base_item_id: i32,
) -> Result<Vec<String>, String> {
    let game_data = state.game_data.read();
    let rm = state.resource_manager.blocking_read();
    Ok(list_armor_mesh_candidates_impl(
        base_item_id,
        &game_data,
        &rm,
    ))
}

#[tauri::command]
pub fn get_item_appearance_options(
    state: State<'_, AppState>,
    base_item_id: i32,
    armor_visual_type: Option<i32>,
) -> Result<ItemAppearanceOptions, String> {
    let game_data = state.game_data.read();
    let rm = state.resource_manager.blocking_read();
    let body_prefix = current_body_prefix(&state, &game_data);
    debug!(
        "Getting appearance options for base item: {} (vt={:?}, body={:?})",
        base_item_id, armor_visual_type, body_prefix
    );

    Ok(ItemAppearance::get_options(
        base_item_id,
        armor_visual_type,
        body_prefix.as_deref(),
        &game_data,
        &rm,
    ))
}

/// Look up the loaded character's race/gender body prefix (e.g. `P_EEM`)
/// so the item viewer previews armor on the same body it sits on in-game.
/// Returns `None` when there's no character loaded — the resolver then
/// falls back to its `P_HHM` default.
fn current_body_prefix(
    state: &State<'_, AppState>,
    game_data: &crate::loaders::GameData,
) -> Option<String> {
    let session = state.session.read();
    let character = session.character.as_ref()?;
    character.body_prefix(game_data)
}

pub fn load_item_model_impl(
    base_item_id: i32,
    appearance: &ItemAppearance,
    body_prefix: Option<&str>,
    override_resref: Option<&str>,
    game_data: &crate::loaders::GameData,
    rm: &crate::services::resource_manager::ResourceManager,
) -> Result<ModelData, String> {
    let base_label = game_data
        .get_table("baseitems")
        .and_then(|t| t.get_by_id(base_item_id))
        .and_then(|row| {
            crate::utils::parsing::row_str(&row, "label")
                .or_else(|| crate::utils::parsing::row_str(&row, "modeltype"))
        })
        .unwrap_or_else(|| format!("base_item_{base_item_id}"));

    // Gloves and bracer items share the glove MDB pipeline. In-game ch3
    // is ignored for both; our resolver bakes `override_tints` with ch3
    // forced white so a black ch3 in the save doesn't darken the mesh.
    let is_gloves_like = crate::character::is_bracer_label(&base_label)
        || base_label.eq_ignore_ascii_case("gloves");

    let mut groups = appearance.resolve_model_resrefs(base_item_id, game_data, body_prefix);

    // When an override is provided, replace the primary group with a single-
    // entry group containing exactly that resref. Accessory / nested boots /
    // gloves groups (indexes 1..) are preserved so the rest of the outfit
    // still renders.
    if let Some(override_res) = override_resref {
        if groups.is_empty() {
            groups.push(vec![override_res.to_string()]);
        } else {
            groups[0] = vec![override_res.to_string()];
        }
        info!(
            target: "item_model",
            "ITEM base={base_label}(#{base_item_id}) OVERRIDE={override_res}"
        );
    }

    if groups.is_empty() {
        info!(
            target: "item_model",
            "ITEM base={base_label}(#{base_item_id}) BODY={body_prefix:?} VARIATION={} MP={:?} VT={:?} -> NO_MODEL",
            appearance.variation, appearance.model_parts, appearance.armor_visual_type
        );
        return Ok(ModelData::default());
    }

    let mut combined_data = ModelData::default();
    let mut loaded_resrefs: Vec<String> = Vec::new();

    for group in &groups {
        let mut group_hit: Option<String> = None;
        for resref in group {
            let part_tag = match resref.to_lowercase().as_bytes() {
                [.., b'_', b'a'] => "item_a",
                [.., b'_', b'b'] => "item_b",
                [.., b'_', b'c'] => "item_c",
                _ => "item",
            };
            match model_loader::load_model(rm, resref, part_tag, "item") {
                Ok(mut data) => {
                    if let Some((bone, slot)) =
                        crate::character::resref_attach_bone_and_slot(resref)
                    {
                        let raw_tints = appearance.accessories.get_tints(slot).cloned();
                        let tints = raw_tints.map(|t| {
                            if matches!(slot, AccessorySlot::LtBracer | AccessorySlot::RtBracer) {
                                swap_tint_2_3(&t)
                            } else {
                                t
                            }
                        });
                        for m in &mut data.meshes {
                            m.attach_bone = Some(bone.to_string());
                            if let Some(ref t) = tints {
                                m.override_tints = Some(t.clone());
                            }
                        }
                    } else if is_gloves_like && resref.to_lowercase().contains("_gloves") {
                        // Freeze the tint group so the frontend's live
                        // `updateTintUniforms("item", …)` pass doesn't
                        // overwrite the baked cosmetic override.
                        let cosmetic = cosmetic_gloves_tint(&appearance.tints);
                        for m in &mut data.meshes {
                            m.override_tints = Some(cosmetic.clone());
                            m.tint_group = "item_frozen".to_string();
                        }
                    } else if resref_is_boots_or_gloves(resref) {
                        let swapped = swap_tint_2_3(&appearance.tints);
                        for m in &mut data.meshes {
                            m.override_tints = Some(swapped.clone());
                            m.tint_group = "item_frozen".to_string();
                        }
                    }
                    combined_data.meshes.extend(data.meshes);
                    combined_data.hooks.extend(data.hooks);
                    combined_data.hair.extend(data.hair);
                    combined_data.helm.extend(data.helm);
                    if combined_data.skeleton.is_none() {
                        combined_data.skeleton = data.skeleton;
                        combined_data.animations = data.animations;
                    }
                    group_hit = Some(resref.clone());
                    break;
                }
                Err(e) => {
                    debug!("Candidate '{resref}' failed: {e}. Trying next in group.");
                }
            }
        }
        loaded_resrefs.push(group_hit.unwrap_or_else(|| format!("MISS({group:?})")));
    }

    info!(
        target: "item_model",
        "ITEM base={base_label}(#{base_item_id}) BODY={body_prefix:?} VARIATION={} MP={:?} VT={:?} BOOTS={:?} GLOVES={:?} GROUPS={} LOADED={:?}",
        appearance.variation,
        appearance.model_parts,
        appearance.armor_visual_type,
        appearance.boots,
        appearance.gloves,
        groups.len(),
        loaded_resrefs
    );

    if combined_data.meshes.is_empty() {
        info!("No meshes loaded for base item {base_item_id}; returning empty ModelData");
        return Ok(ModelData::default());
    }

    info!(
        "Loaded item model with {} meshes",
        combined_data.meshes.len()
    );
    Ok(combined_data)
}

#[tauri::command]
pub fn load_item_model(
    state: State<'_, AppState>,
    appearance: ItemAppearance,
    base_item_id: i32,
    override_resref: Option<String>,
) -> Result<ModelData, String> {
    let game_data = state.game_data.read();
    let rm = state.resource_manager.blocking_read();
    let body_prefix = current_body_prefix(&state, &game_data);
    load_item_model_impl(
        base_item_id,
        &appearance,
        body_prefix.as_deref(),
        override_resref.as_deref(),
        &game_data,
        &rm,
    )
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
