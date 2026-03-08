use tauri::State;
use crate::commands::{CommandError, CommandResult};
use crate::services::campaign::CampaignManager;
use crate::parsers::xml::{FullSummary, CompanionStatus};
use crate::services::campaign::content::{ModuleInfo, ModuleVariables};
use crate::state::AppState;
use crate::services::campaign::journal::QuestDefinition;
use crate::services::campaign::settings::CampaignSettings;
use std::collections::HashMap;

#[tauri::command]
pub async fn get_campaign_summary(
    state: State<'_, AppState>,
) -> CommandResult<FullSummary> {
    let session = state.session.read();
    let handler = session.savegame_handler.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::get_summary(handler).map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_module_info(
    state: State<'_, AppState>,
) -> CommandResult<(ModuleInfo, ModuleVariables)> {
    let paths = state.paths.read();
    let session = state.session.read();
    let handler = session.savegame_handler.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::get_module_info(handler, &paths).map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_journal(
    state: State<'_, AppState>,
) -> CommandResult<HashMap<String, QuestDefinition>> {
    let session = state.session.read();
    let handler = session.savegame_handler.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::get_journal(handler).map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_global_int(
    state: State<'_, AppState>,
    name: String,
    value: i32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let handler = session.savegame_handler.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::update_global_int(handler, &name, value).map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_global_float(
    state: State<'_, AppState>,
    name: String,
    value: f32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let handler = session.savegame_handler.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::update_global_float(handler, &name, value).map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_global_string(
    state: State<'_, AppState>,
    name: String,
    value: String,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let handler = session.savegame_handler.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::update_global_string(handler, &name, &value).map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_campaign_settings(
    state: State<'_, AppState>,
    campaign_id: String,
) -> CommandResult<CampaignSettings> {
    let paths = state.paths.read();
    CampaignManager::get_campaign_settings(&campaign_id, &paths).map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_campaign_settings(
    state: State<'_, AppState>,
    settings: CampaignSettings
) -> CommandResult<()> {
    let paths = state.paths.read();
    CampaignManager::update_campaign_settings(&settings, &paths).map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_companion_influence(
    state: State<'_, AppState>,
) -> CommandResult<HashMap<String, CompanionStatus>> {
    let session = state.session.read();
    let handler = session.savegame_handler.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::get_companion_influence(handler).map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_companion_influence(
    state: State<'_, AppState>,
    companion_id: String,
    new_influence: i32,
) -> CommandResult<()> {
    let mut session = state.session.write();
    let handler = session.savegame_handler.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    CampaignManager::update_companion_influence(handler, &companion_id, new_influence).map_err(CommandError::from)
}
