use serde::Serialize;
use specta::Type;
use tauri::State;
use tracing::info;

use crate::config::nwn2_paths::PathSource;
use crate::state::AppState;

use super::{CommandError, CommandResult};

#[derive(Debug, Serialize, Type)]
pub struct DebugLog {
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub timestamp: String,
    pub paths: PathsDebug,
    pub session: SessionDebug,
}

#[derive(Debug, Serialize, Type)]
pub struct PathsDebug {
    pub game_folder: Option<String>,
    pub game_folder_source: String,
    pub documents_folder: Option<String>,
    pub documents_folder_source: String,
    pub steam_workshop_folder: Option<String>,
    pub steam_workshop_folder_source: String,
    pub custom_override_folders: Vec<String>,
    pub custom_hak_folders: Vec<String>,
    pub game_version: Option<String>,
    pub is_enhanced_edition: bool,
    pub is_steam_installation: bool,
    pub is_gog_installation: bool,
}

#[derive(Debug, Serialize, Type)]
pub struct SessionDebug {
    pub character_loaded: bool,
    pub file_path: Option<String>,
    pub character_name: Option<String>,
    pub has_unsaved_changes: bool,
}

fn path_source_to_string(source: PathSource) -> String {
    match source {
        PathSource::Discovery => "auto-detected".to_string(),
        PathSource::Environment => "environment".to_string(),
        PathSource::Config => "manual".to_string(),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn export_debug_log(state: State<'_, AppState>) -> CommandResult<String> {
    info!("Exporting debug log");

    let paths = state.paths.read();
    let session = state.session.read();

    let paths_debug = PathsDebug {
        game_folder: paths.game_folder().map(|p| p.to_string_lossy().to_string()),
        game_folder_source: path_source_to_string(paths.game_folder_source()),
        documents_folder: paths
            .documents_folder()
            .map(|p| p.to_string_lossy().to_string()),
        documents_folder_source: path_source_to_string(paths.documents_folder_source()),
        steam_workshop_folder: paths
            .steam_workshop_folder()
            .map(|p| p.to_string_lossy().to_string()),
        steam_workshop_folder_source: path_source_to_string(paths.steam_workshop_folder_source()),
        custom_override_folders: paths
            .custom_override_folders()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        custom_hak_folders: paths
            .custom_hak_folders()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        game_version: paths.get_game_version(),
        is_enhanced_edition: paths.is_enhanced_edition(),
        is_steam_installation: paths.is_steam_installation(),
        is_gog_installation: paths.is_gog_installation(),
    };

    let session_debug = SessionDebug {
        character_loaded: session.character.is_some(),
        file_path: session
            .current_file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        character_name: session.character.as_ref().map(|c| c.full_name()),
        has_unsaved_changes: session.has_unsaved_changes(),
    };

    let debug_log = DebugLog {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        paths: paths_debug,
        session: session_debug,
    };

    let json = serde_json::to_string_pretty(&debug_log)
        .map_err(|e| CommandError::Internal(format!("Failed to serialize debug log: {e}")))?;

    let downloads_path = dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .ok_or_else(|| CommandError::Internal("Could not find Downloads folder".to_string()))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("nwn2ee-debug-{timestamp}.json");
    let file_path = downloads_path.join(&filename);

    std::fs::write(&file_path, &json).map_err(|e| CommandError::FileError {
        message: format!("Failed to write debug log: {e}"),
        path: Some(file_path.to_string_lossy().to_string()),
    })?;

    info!("Debug log exported to {}", file_path.display());

    Ok(file_path.to_string_lossy().to_string())
}
