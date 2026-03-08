use crate::character::classes::{
    AlignmentRestriction, ClassProgression, LevelUpResult, PrestigeClassOption,
    PrestigeClassValidation, ResolvedLevelHistoryEntry, get_class_progression,
};
use crate::character::{ClassEntry, ClassId, ClassSummaryEntry, XpProgress};
use crate::commands::{CommandError, CommandResult};
use crate::services::class_categorizer::{CategorizedClasses, get_categorized_classes};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_total_level(state: State<'_, AppState>) -> CommandResult<i32> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.total_level())
}

#[tauri::command]
pub async fn get_class_entries(state: State<'_, AppState>) -> CommandResult<Vec<ClassEntry>> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.class_entries())
}

#[tauri::command]
pub async fn get_class_level(state: State<'_, AppState>, class_id: i32) -> CommandResult<i32> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.class_level(ClassId(class_id)))
}

#[tauri::command]
pub async fn get_class_summary(
    state: State<'_, AppState>,
) -> CommandResult<Vec<ClassSummaryEntry>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_class_summary(&game_data))
}

#[tauri::command]
pub async fn get_class_name(state: State<'_, AppState>, class_id: i32) -> CommandResult<String> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_class_name(ClassId(class_id), &game_data))
}

#[tauri::command]
pub async fn get_xp_progress(state: State<'_, AppState>) -> CommandResult<XpProgress> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_xp_progress(&game_data))
}

#[tauri::command]
pub async fn get_level_history(
    state: State<'_, AppState>,
) -> CommandResult<Vec<ResolvedLevelHistoryEntry>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.level_history_resolved(&game_data))
}

#[tauri::command]
pub async fn set_experience(state: State<'_, AppState>, xp: i32) -> CommandResult<()> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .set_experience(xp)
        .map_err(|e| CommandError::ValidationError {
            field: "experience".to_string(),
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn add_class_entry(
    state: State<'_, AppState>,
    class_id: i32,
    level: i32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .add_class_entry(ClassId(class_id), level)
        .map_err(|e| CommandError::ValidationError {
            field: "class_entry".to_string(),
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn set_class_level(
    state: State<'_, AppState>,
    class_id: i32,
    new_level: i32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .set_class_level(ClassId(class_id), new_level)
        .map_err(|e| CommandError::ValidationError {
            field: "class_level".to_string(),
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn remove_class_entry(state: State<'_, AppState>, class_id: i32) -> CommandResult<()> {
    let mut session = state.session.write();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .remove_class(ClassId(class_id), &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "class".to_string(),
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn is_prestige_class(state: State<'_, AppState>, class_id: i32) -> CommandResult<bool> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.is_prestige_class(ClassId(class_id), &game_data))
}

#[tauri::command]
pub async fn check_prestige_class_requirements(
    state: State<'_, AppState>,
    class_id: i32,
) -> CommandResult<PrestigeClassValidation> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.validate_prestige_class_requirements(ClassId(class_id), &game_data))
}

#[tauri::command]
pub async fn get_available_prestige_classes(
    state: State<'_, AppState>,
) -> CommandResult<Vec<PrestigeClassOption>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_prestige_class_options(&game_data))
}

#[tauri::command]
pub async fn decode_alignment_restriction(bits: i32) -> CommandResult<Option<String>> {
    Ok(AlignmentRestriction(bits).decode_to_string())
}

#[tauri::command]
pub async fn add_class_level(
    state: State<'_, AppState>,
    class_id: i32,
) -> CommandResult<LevelUpResult> {
    let mut session = state.session.write();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;

    character
        .level_up(ClassId(class_id), &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "level_up".to_string(),
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn change_class(
    state: State<'_, AppState>,
    old_class_id: i32,
    new_class_id: i32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;

    character
        .swap_class(ClassId(old_class_id), ClassId(new_class_id), &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "change_class".to_string(),
            reason: e.to_string(),
        })
}

#[tauri::command]
pub async fn remove_class_levels(
    state: State<'_, AppState>,
    class_id: i32,
    count: i32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;

    for _ in 0..count {
        character
            .level_down(ClassId(class_id), &game_data)
            .map_err(|e| CommandError::ValidationError {
                field: "level_down".to_string(),
                reason: e.to_string(),
            })?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_class_progression_details(
    state: State<'_, AppState>,
    class_id: i32,
    max_level: Option<i32>,
) -> CommandResult<ClassProgression> {
    let _session = state.session.read();
    let game_data = state.game_data.read();
    let max = max_level.unwrap_or(20);
    get_class_progression(class_id, max, &game_data).ok_or_else(|| CommandError::NotFound {
        item: format!("Class {class_id}"),
    })
}

#[tauri::command]
pub async fn get_all_categorized_classes(
    state: State<'_, AppState>,
) -> CommandResult<CategorizedClasses> {
    let game_data = state.game_data.read();
    Ok(get_categorized_classes(&game_data))
}
