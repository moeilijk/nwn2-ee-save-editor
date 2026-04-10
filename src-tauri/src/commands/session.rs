use crate::commands::{CommandError, CommandResult};
use crate::state::AppState;
use tauri::{AppHandle, Manager, State};
use tracing::{error, info, instrument, warn};

#[tauri::command]
#[instrument(name = "load_character_command", skip(state, app), fields(file_path = %file_path))]
pub async fn load_character(
    state: State<'_, AppState>,
    app: AppHandle,
    file_path: String,
) -> CommandResult<bool> {
    info!("Load character command invoked");

    let mut session = state.session.write();
    match session.load_character(&file_path) {
        Ok(()) => {
            info!("Character loaded successfully via command");
            drop(session);
            tokio::spawn(async move {
                let state = app.state::<AppState>();

                // Pre-warm feat cache
                let cache = {
                    let game_data = state.game_data.read();
                    let session = state.session.read();
                    let Some(character) = session.character.as_ref() else {
                        return;
                    };
                    match super::feats::build_feat_list(character, &game_data) {
                        Ok(c) => c,
                        Err(e) => {
                            warn!("Failed to pre-warm feat cache: {e}");
                            return;
                        }
                    }
                };
                let mut session = state.session.write();
                if session.feat_cache.is_none() {
                    session.feat_cache = Some(cache);
                }
                drop(session);
            });
            Ok(true)
        }
        Err(e) => {
            error!("Failed to load character: {}", e);
            Err(CommandError::CharacterNotFound { path: file_path })
        }
    }
}

#[tauri::command]
#[instrument(name = "save_character_command", skip(state))]
pub async fn save_character(
    state: State<'_, AppState>,
    _file_path: Option<String>,
) -> CommandResult<bool> {
    info!("Save character command invoked");

    let game_data = state.game_data.read();
    let mut session = state.session.write();
    match session.save_character(&game_data) {
        Ok(()) => {
            info!("Character saved successfully via command");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to save character: {}", e);
            Err(CommandError::FileError {
                message: e.clone(),
                path: session
                    .current_file_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
            })
        }
    }
}

#[tauri::command]
#[instrument(name = "close_character_command", skip(state))]
pub async fn close_character(state: State<'_, AppState>) -> CommandResult<bool> {
    info!("Close character command invoked");
    let mut session = state.session.write();
    session.close_character();
    info!("Character closed successfully");
    Ok(true)
}

#[derive(serde::Serialize, specta::Type)]
pub struct SessionInfo {
    pub character_loaded: bool,
    pub file_path: Option<String>,
    pub dirty: bool,
}

#[tauri::command]
pub async fn get_session_info(state: State<'_, AppState>) -> CommandResult<SessionInfo> {
    let session = state.session.read();
    Ok(SessionInfo {
        character_loaded: session.character.is_some(),
        file_path: session
            .current_file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        dirty: session.has_unsaved_changes(),
    })
}

#[tauri::command]
pub async fn has_unsaved_changes(state: State<'_, AppState>) -> CommandResult<bool> {
    let session = state.session.read();
    Ok(session.has_unsaved_changes())
}

#[tauri::command]
#[instrument(name = "export_to_localvault_command", skip(state))]
pub async fn export_to_localvault(state: State<'_, AppState>) -> CommandResult<String> {
    info!("Export to localvault command invoked");

    let session = state.session.read();
    match session.export_to_localvault() {
        Ok(path) => {
            info!("Character exported to vault: {}", path);
            Ok(path)
        }
        Err(e) => {
            error!("Failed to export to localvault: {}", e);
            Err(CommandError::FileError {
                message: e,
                path: None,
            })
        }
    }
}
