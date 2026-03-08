use crate::commands::{CommandError, CommandResult};
use crate::state::AppState;
use tauri::State;
use tracing::{info, error, instrument};

#[tauri::command]
#[instrument(name = "load_character_command", skip(state), fields(file_path = %file_path))]
pub async fn load_character(
    state: State<'_, AppState>,
    file_path: String,
) -> CommandResult<bool> {
    info!("Load character command invoked");

    let mut session = state.session.write();
    match session.load_character(&file_path) {
        Ok(()) => {
            info!("Character loaded successfully via command");
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

    let mut session = state.session.write();
    match session.save_character() {
        Ok(()) => {
            info!("Character saved successfully via command");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to save character: {}", e);
            Err(CommandError::FileError {
                message: e.clone(),
                path: session.current_file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
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
        file_path: session.current_file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
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
pub async fn export_to_localvault(
    state: State<'_, AppState>,
) -> CommandResult<String> {
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
