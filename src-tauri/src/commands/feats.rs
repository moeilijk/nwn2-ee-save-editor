use crate::character::{FeatEntry, FeatSlots, PrerequisiteResult, FeatInfo, FeatSummary, DomainInfo, FeatId, DomainId, FeatAvailability, FeatType};
use crate::commands::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FeatActionResult {
    pub success: bool,
    pub message: String,
    pub feat_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FilteredFeatsResponse {
    pub feats: Vec<FeatInfo>,
    pub total: i32,
    pub page: i32,
    pub limit: i32,
    pub pages: i32,
    pub has_next: bool,
    pub has_previous: bool,
}

#[tauri::command]
pub async fn get_feat_list(state: State<'_, AppState>) -> CommandResult<Vec<FeatEntry>> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.feat_entries())
}

#[tauri::command]
pub async fn has_feat(state: State<'_, AppState>, feat_id: i32) -> CommandResult<bool> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.has_feat(FeatId(feat_id)))
}

#[tauri::command]
pub async fn get_feat_info(state: State<'_, AppState>, feat_id: i32) -> CommandResult<FeatInfo> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    character
        .get_feat_info(FeatId(feat_id), &game_data)
        .ok_or_else(|| CommandError::NotFound {
            item: format!("Feat {feat_id}")
        })
}

#[tauri::command]
pub async fn get_feat_summary(state: State<'_, AppState>) -> CommandResult<FeatSummary> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_feat_summary(&game_data))
}

#[tauri::command]
pub async fn get_feat_slots(state: State<'_, AppState>) -> CommandResult<FeatSlots> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_feat_slots(&game_data))
}

#[tauri::command]
pub async fn validate_feat_prerequisites(
    state: State<'_, AppState>,
    feat_id: i32,
) -> CommandResult<PrerequisiteResult> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.validate_feat_prerequisites(FeatId(feat_id), &game_data))
}

#[tauri::command]
pub async fn add_feat(
    state: State<'_, AppState>,
    feat_id: i32,
) -> CommandResult<FeatActionResult> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    match character.add_feat(FeatId(feat_id)) {
        Ok(()) => Ok(FeatActionResult {
            success: true,
            message: "Feat added successfully".to_string(),
            feat_id,
        }),
        Err(e) => Ok(FeatActionResult {
            success: false,
            message: e.to_string(),
            feat_id,
        }),
    }
}

#[tauri::command]
pub async fn remove_feat(
    state: State<'_, AppState>,
    feat_id: i32,
) -> CommandResult<FeatActionResult> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    match character.remove_feat(FeatId(feat_id)) {
        Ok(()) => Ok(FeatActionResult {
            success: true,
            message: "Feat removed successfully".to_string(),
            feat_id,
        }),
        Err(e) => Ok(FeatActionResult {
            success: false,
            message: e.to_string(),
            feat_id,
        }),
    }
}

#[tauri::command]
pub async fn swap_feat(
    state: State<'_, AppState>,
    old_feat_id: i32,
    new_feat_id: i32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    character.swap_feat(FeatId(old_feat_id), FeatId(new_feat_id))
        .map(|_| ())
        .map_err(|e| CommandError::OperationFailed {
            operation: format!("swap_feat({old_feat_id} -> {new_feat_id})"),
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn check_feat_progression(
    state: State<'_, AppState>,
    new_feat_id: i32,
) -> CommandResult<Option<i32>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let old_feat = character.check_feat_progression(FeatId(new_feat_id), &game_data);
    Ok(old_feat.map(|f| f.0))
}

#[tauri::command]
pub async fn get_character_domains(state: State<'_, AppState>) -> CommandResult<Vec<i32>> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let domains = character.get_character_domains();
    Ok(domains.into_iter().map(|d| d.0).collect())
}

#[tauri::command]
pub async fn get_available_domains(state: State<'_, AppState>) -> CommandResult<Vec<DomainInfo>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_available_domains(&game_data))
}

#[tauri::command]
pub async fn add_domain(state: State<'_, AppState>, domain_id: i32) -> CommandResult<Vec<i32>> {
    let game_data = state.game_data.read();
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    let feats = character.add_domain(DomainId(domain_id), &game_data)
        .map_err(|e| CommandError::OperationFailed {
            operation: format!("add_domain({domain_id})"),
            reason: e.to_string(),
        })?;
    Ok(feats.into_iter().map(|f| f.0).collect())
}

#[tauri::command]
pub async fn remove_domain(state: State<'_, AppState>, domain_id: i32) -> CommandResult<Vec<i32>> {
    let game_data = state.game_data.read();
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    let feats = character.remove_domain(DomainId(domain_id), &game_data)
        .map_err(|e| CommandError::OperationFailed {
            operation: format!("remove_domain({domain_id})"),
            reason: e.to_string(),
        })?;
    Ok(feats.into_iter().map(|f| f.0).collect())
}

#[tauri::command]
pub async fn get_all_feats(state: State<'_, AppState>) -> CommandResult<Vec<FeatInfo>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;

    let feats_table = game_data.get_table("feat")
        .ok_or(CommandError::NotFound { item: "Feat table".to_string() })?;

    let mut all_feats = Vec::new();
    for i in 0..feats_table.row_count() {
        let feat_id = FeatId(i as i32);

        if let Some(feat_data) = feats_table.get_by_id(feat_id.0) {
            let removed = feat_data
                .get("removed")
                .or_else(|| feat_data.get("REMOVED"))
                .and_then(|s| s.as_ref())
                .is_some_and(|s| s == "1");

            if removed {
                continue;
            }
        }

        if let Some(feat_info) = character.get_feat_info(feat_id, &game_data) {
            if feat_info.label.is_empty()
                || feat_info.label.starts_with("****")
                || feat_info.label.starts_with("DEL_")
                || feat_info.label == "DELETED"
            {
                continue;
            }
            all_feats.push(feat_info);
        }
    }
    Ok(all_feats)
}

#[tauri::command]
pub async fn get_filtered_feats(
    state: State<'_, AppState>,
    page: i32,
    limit: i32,
    feat_type: Option<i32>,
    search: Option<String>,
) -> CommandResult<FilteredFeatsResponse> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;

    let feats_table = game_data.get_table("feat")
        .ok_or(CommandError::NotFound { item: "Feat table".to_string() })?;

    let search_lower = search.as_ref().map(|s| s.to_lowercase());

    let mut filtered_feats = Vec::new();
    for i in 0..feats_table.row_count() {
        let feat_id = FeatId(i as i32);

        if let Some(feat_data) = feats_table.get_by_id(feat_id.0) {
            let removed = feat_data
                .get("removed")
                .or_else(|| feat_data.get("REMOVED"))
                .and_then(|s| s.as_ref())
                .is_some_and(|s| s == "1");

            if removed {
                continue;
            }
        }

        if let Some(feat_info) = character.get_feat_info(feat_id, &game_data) {
            if feat_info.label.is_empty()
                || feat_info.label.starts_with("****")
                || feat_info.label.starts_with("DEL_")
                || feat_info.label == "DELETED"
            {
                continue;
            }

            if let Some(ft) = feat_type
                && (feat_info.feat_type.0 & ft) == 0
            {
                continue;
            }

            if let Some(ref search_str) = search_lower {
                let name_lower = feat_info.name.to_lowercase();
                let label_lower = feat_info.label.to_lowercase();
                let desc_lower = feat_info.description.to_lowercase();

                if !name_lower.contains(search_str)
                    && !label_lower.contains(search_str)
                    && !desc_lower.contains(search_str)
                {
                    continue;
                }
            }

            filtered_feats.push(feat_info);
        }
    }

    let total = filtered_feats.len() as i32;
    let pages = if total == 0 { 1 } else { (total + limit - 1) / limit };
    let start = ((page - 1) * limit) as usize;
    let end = (start + limit as usize).min(filtered_feats.len());

    let paged_feats = if start < filtered_feats.len() {
        filtered_feats[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(FilteredFeatsResponse {
        feats: paged_feats,
        total,
        page,
        limit,
        pages,
        has_next: page < pages,
        has_previous: page > 1,
    })
}

#[tauri::command]
pub async fn check_feat_availability(
    state: State<'_, AppState>,
    feat_id: i32,
) -> CommandResult<FeatAvailability> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;

    let feats_table = game_data.get_table("feat")
        .ok_or(CommandError::NotFound { item: "Feat table".to_string() })?;

    let feat_data = feats_table.get_by_id(feat_id)
        .ok_or(CommandError::NotFound { item: format!("Feat {feat_id}") })?;

    let label = feat_data
        .get("label")
        .and_then(|s| s.as_ref())
        .map_or_else(|| format!("feat_{feat_id}"), |s| s.clone());

    let description = feat_data
        .get("DESCRIPTION")
        .or_else(|| feat_data.get("description"))
        .and_then(|s| s.as_ref()?.parse::<i32>().ok())
        .and_then(|strref| game_data.get_string(strref))
        .unwrap_or_default();

    let feat_type = parse_feat_type_from_data(&feat_data, &description);

    Ok(character.get_feat_availability(FeatId(feat_id), feat_type, &label, &game_data))
}

fn parse_feat_type_from_data(feat_data: &ahash::AHashMap<String, Option<String>>, description: &str) -> FeatType {
    if let Some(type_str) = feat_data
        .get("TOOLCATEGORIES")
        .or_else(|| feat_data.get("ToolsCategories"))
        .or_else(|| feat_data.get("toolscategories"))
        .and_then(|s| s.as_ref())
    {
        return FeatType::from_string(type_str);
    }

    if let Some(type_str) = feat_data
        .get("FeatCategory")
        .or_else(|| feat_data.get("FEATCATEGORY"))
        .or_else(|| feat_data.get("featcategory"))
        .and_then(|s| s.as_ref())
    {
        return FeatType::from_string(type_str);
    }

    if description.contains("Type of Feat:") {
        if description.contains("Background") {
            return FeatType::BACKGROUND;
        }
        if description.contains("History") {
            return FeatType::HISTORY;
        }
    }

    FeatType::GENERAL
}
