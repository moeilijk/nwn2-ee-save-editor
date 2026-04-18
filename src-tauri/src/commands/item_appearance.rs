use tauri::State;
use tracing::{debug, info};

use crate::character::{ItemAppearance, ItemAppearanceOptions};
use crate::services::model_loader::{self, ModelData};
use crate::state::AppState;

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

#[tauri::command]
pub fn load_item_model(
    state: State<'_, AppState>,
    appearance: ItemAppearance,
    base_item_id: i32,
) -> Result<ModelData, String> {
    let game_data = state.game_data.read();
    let rm = state.resource_manager.blocking_read();

    // Snapshot the base item label up front so the per-item summary stays on a
    // single line regardless of how many fallbacks we try.
    let base_label = game_data
        .get_table("baseitems")
        .and_then(|t| t.get_by_id(base_item_id))
        .and_then(|row| {
            crate::utils::parsing::row_str(&row, "label")
                .or_else(|| crate::utils::parsing::row_str(&row, "modeltype"))
        })
        .unwrap_or_else(|| format!("base_item_{base_item_id}"));

    let body_prefix = current_body_prefix(&state, &game_data);
    let groups = appearance.resolve_model_resrefs(base_item_id, &game_data, body_prefix.as_deref());

    if groups.is_empty() {
        info!(
            target: "item_model",
            "ITEM base={base_label}(#{base_item_id}) BODY={:?} VARIATION={} MP={:?} VT={:?} -> NO_MODEL",
            body_prefix, appearance.variation, appearance.model_parts, appearance.armor_visual_type
        );
        return Ok(ModelData::default());
    }

    let mut combined_data = ModelData::default();
    let mut loaded_resrefs: Vec<String> = Vec::new();

    // Each outer group represents an independent model part (e.g. the _a/_b/_c
    // pieces of a 3-part weapon, or the body/nested-boots/nested-gloves of an
    // armor set). Within a group the candidates are ordered fallbacks — the
    // first one that loads wins, the rest are skipped.
    for group in &groups {
        let mut group_hit: Option<String> = None;
        for resref in group {
            let part_tag = match resref.to_lowercase().as_bytes() {
                [.., b'_', b'a'] => "item_a",
                [.., b'_', b'b'] => "item_b",
                [.., b'_', b'c'] => "item_c",
                _ => "item",
            };
            match model_loader::load_model(&rm, resref, part_tag, "item") {
                Ok(mut data) => {
                    // Accessory resrefs: pull the slot's bone + per-slot
                    // tints off `appearance.accessories` so each accessory
                    // parents correctly and keeps its own colour. Returns
                    // None for non-accessory resrefs (body/boots/etc.).
                    if let Some((bone, slot)) =
                        crate::character::resref_attach_bone_and_slot(resref)
                    {
                        let tints = appearance.accessories.get_tints(slot).cloned();
                        for m in &mut data.meshes {
                            m.attach_bone = Some(bone.to_string());
                            if let Some(ref t) = tints {
                                m.override_tints = Some(t.clone());
                            }
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

    // Single-line summary so filtering with RUST_LOG=item_model=info is enough
    // to see exactly what each item viewer load decided on.
    info!(
        target: "item_model",
        "ITEM base={base_label}(#{base_item_id}) BODY={:?} VARIATION={} MP={:?} VT={:?} BOOTS={:?} GLOVES={:?} GROUPS={} LOADED={:?}",
        body_prefix,
        appearance.variation,
        appearance.model_parts,
        appearance.armor_visual_type,
        appearance.boots,
        appearance.gloves,
        groups.len(),
        loaded_resrefs
    );

    // If every candidate resref failed to load, treat this as "no preview" —
    // frontend renders a placeholder instead of a red error overlay. Items that
    // legitimately have no model (wands, accessories with unknown ItemClass)
    // land here naturally.
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
