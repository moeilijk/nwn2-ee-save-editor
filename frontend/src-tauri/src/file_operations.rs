use tauri_plugin_dialog::DialogExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use base64::prelude::*;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct SaveFile {
    pub path: String,
    pub name: String,
    pub thumbnail: Option<String>,
    pub modified: Option<i64>,
}

#[tauri::command]
pub async fn select_save_file(app: tauri::AppHandle) -> Result<SaveFile, String> {
    log::info!("[Rust] The 'select_save_file' command has been invoked.");
    let dialog = app.dialog().file();
    
    // [FIX] Added logging to pinpoint the exact location of the freeze.
    log::info!("[Rust] About to call blocking_pick_folder. If the app freezes, this is the last log you will see.");
    
    let dir_path = dialog
        .blocking_pick_folder()
        .ok_or("No save directory selected or the dialog was cancelled.")?;

    // If the application doesn't freeze, you will see this log message.
    log::info!("[Rust] blocking_pick_folder completed successfully. A folder was selected.");

    let path_str = match &dir_path {
        tauri_plugin_dialog::FilePath::Path(p) => p.to_string_lossy().to_string(),
        _ => return Err("Invalid directory path format".to_string()),
    };
    
    let name = match &dir_path {
        tauri_plugin_dialog::FilePath::Path(p) => {
            // Try to read actual save name from savename.txt
            match std::fs::read_to_string(p.join("savename.txt")) {
                Ok(content) => content.trim().to_string(),
                Err(_) => {
                    // Fallback to folder name
                    p.file_name()
                     .and_then(|n| n.to_str())
                     .unwrap_or("Unknown")
                     .to_string()
                }
            }
        },
        _ => "Unknown".to_string(),
    };

    let save_path = PathBuf::from(&path_str);
    let resgff_path = save_path.join("resgff.zip");
    if !resgff_path.exists() {
        log::error!("[Rust] Validation failed: selected directory is missing resgff.zip");
        return Err("Selected directory doesn't appear to be a valid NWN2 save (missing resgff.zip)".to_string());
    }

    log::info!("[Rust] Save file validated. Returning path to frontend.");
    // Check for thumbnail in selected save
    let thumbnail_path = save_path.join("screen.tga");
    let thumbnail = if thumbnail_path.exists() {
        Some(thumbnail_path.to_string_lossy().to_string())
    } else {
        None
    };
    
    let modified = save_path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64);

    Ok(SaveFile { path: path_str, name, thumbnail, modified })
}

#[tauri::command]
pub async fn select_nwn2_directory(app: tauri::AppHandle) -> Result<String, String> {
    log::info!("[Rust] About to call blocking_pick_folder for NWN2 directory.");
    let dir_path = app.dialog()
        .file()
        .blocking_pick_folder()
        .ok_or("No directory selected or the dialog was cancelled.")?;
    log::info!("[Rust] blocking_pick_folder for NWN2 directory completed.");

    match dir_path {
        tauri_plugin_dialog::FilePath::Path(p) => Ok(p.to_string_lossy().to_string()),
        _ => Err("Invalid directory path format".to_string()),
    }
}

#[tauri::command]
pub async fn find_nwn2_saves(_app: tauri::AppHandle) -> Result<Vec<SaveFile>, String> {
    use std::time::Instant;
    let start_time = Instant::now();
    log::info!("[Rust] Finding available NWN2 saves.");

    // Use NWN2Paths to get saves directory
    let nwn2_paths = crate::config::nwn2_paths::NWN2Paths::new();
    let saves_path = nwn2_paths.saves().ok_or("Could not determine NWN2 saves path")?;
    let mut saves = Vec::new();
    
    let scan_start = Instant::now();
    if saves_path.is_dir()
        && let Ok(entries) = std::fs::read_dir(&saves_path) {
            // Collect save directory entries for sorting
            let mut save_entries: Vec<_> = entries.flatten()
                .filter(|entry| entry.path().is_dir() && entry.path().join("resgff.zip").exists())
                .collect();
            
            // Sort by directory modification time (newest first)
            save_entries.sort_by(|a, b| {
                let time_a = a.path().metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let time_b = b.path().metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                time_b.cmp(&time_a) // Reverse order for newest first
            });
            
            for entry in save_entries {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                let save_path = entry.path().to_string_lossy().to_string();
                
                // Try to read actual save name from savename.txt
                let save_name = match std::fs::read_to_string(entry.path().join("savename.txt")) {
                    Ok(content) => content.trim().to_string(),
                    Err(_) => folder_name, // Fallback to folder name if savename.txt doesn't exist
                };
                
                // Check for thumbnail
                let thumbnail_path = entry.path().join("screen.tga");
                let thumbnail = if thumbnail_path.exists() {
                    Some(thumbnail_path.to_string_lossy().to_string())
                } else {
                    None
                };
                
                let modified = entry.path().metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64);

                saves.push(SaveFile {
                    name: save_name,
                    path: save_path,
                    thumbnail,
                    modified,
                });
                
                // Limit to 3 saves
                if saves.len() >= 3 {
                    log::info!("[Rust] Limited scan to first 3 saves (newest by modification time)");
                    break;
                }
            }
        }
    log::info!("[Rust] Directory scan took: {:?}", scan_start.elapsed());
    
    log::info!("[Rust] Total function took: {:?}", start_time.elapsed());
    log::info!("[Rust] Found {} potential save(s) in {}", saves.len(), saves_path.display());
    Ok(saves)
}

#[tauri::command]
pub async fn get_steam_workshop_path() -> Result<Option<String>, String> {
    // This function is unchanged
    let mut steam_paths = vec![
        PathBuf::from("C:/Program Files (x86)/Steam/steamapps/workshop/content/2760"),
    ];
    if let Ok(home) = std::env::var("HOME") {
        steam_paths.push(PathBuf::from(&home).join(".steam/steam/steamapps/workshop/content/2760"));
    }
    for path in steam_paths {
        if path.exists() {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn validate_nwn2_installation(path: String) -> Result<bool, String> {
    // This function is unchanged
    let base_path = PathBuf::from(path);
    let required_items = vec!["Data", "dialog.tlk"];
    for item in required_items {
        if !base_path.join(item).exists() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[tauri::command]
pub async fn get_save_thumbnail(thumbnail_path: String) -> Result<String, String> {
    log::info!("[Rust] Starting thumbnail conversion process for: {thumbnail_path}");
    
    let path = PathBuf::from(&thumbnail_path);
    
    // Open and decode TGA file
    log::debug!("[Rust] Attempting to decode TGA file at: {}", path.display());
    
    let dynamic_image = image::open(&path).map_err(|e| {
        log::error!("Failed to open/decode TGA file at '{}': {}", path.display(), e);
        "Failed to process thumbnail. The file may be corrupt or inaccessible.".to_string()
    })?;
    
    log::debug!("[Rust] TGA decoded successfully: {}x{}", dynamic_image.width(), dynamic_image.height());
    
    // Convert to WebP with quality control using webp crate
    let encoder = webp::Encoder::from_image(&dynamic_image)
        .map_err(|e| {
            log::error!("Failed to create WebP encoder: {e}");
            "Failed to create WebP encoder from image.".to_string()
        })?;
    
    // Encode with quality setting of 85.0 (out of 100) for good balance of quality and size
    let webp_memory = encoder.encode(85.0);
    let webp_data = webp_memory.to_vec();
    
    log::debug!("[Rust] Successfully converted TGA to WebP ({} bytes)", webp_data.len());
    
    // Debug: Save a test file to verify WebP is valid (debug builds only)
    #[cfg(debug_assertions)]
    {
        if let Some(parent) = path.parent() {
            let test_path = parent.join("debug_thumbnail.webp");
            if let Err(e) = std::fs::write(&test_path, &webp_data) {
                log::warn!("[Rust] Could not save debug WebP: {e}");
            } else {
                log::debug!("[Rust] Saved debug WebP to: {}", test_path.display());
            }
        }
    }
    
    // Encode as base64 for safe transfer
    let base64_data = base64::prelude::BASE64_STANDARD.encode(&webp_data);
    log::debug!("[Rust] Base64 encoding complete ({} chars), WebP size: {} bytes", base64_data.len(), webp_data.len());
    
    Ok(base64_data)
}

#[tauri::command]
pub async fn detect_nwn2_installation(_app: tauri::AppHandle) -> Result<Option<String>, String> {
    log::info!("[Rust] Detecting NWN2:EE installation");

    let nwn2_paths = crate::config::nwn2_paths::NWN2Paths::new();

    if let Some(game_folder) = nwn2_paths.game_folder()
        && game_folder.exists() {
            let path_str = game_folder.to_string_lossy().to_string();
            log::info!("[Rust] Found NWN2 installation: {path_str}");
            return Ok(Some(path_str));
        }

    log::info!("[Rust] No NWN2 installation found");
    Ok(None)
}

#[tauri::command]
pub async fn open_folder_in_explorer(app: tauri::AppHandle, folder_path: String) -> Result<(), String> {
    log::info!("[Rust] Opening folder in file explorer: {folder_path}");
    
    let path = PathBuf::from(&folder_path);
    
    // Check if path exists
    if !path.exists() {
        return Err(format!("Folder does not exist: {folder_path}"));
    }
    
    let shell = app.shell();
    
    if cfg!(windows) {
        // On Windows, use explorer.exe
        shell.command("explorer")
            .args([&folder_path])
            .spawn()
            .map_err(|e| format!("Failed to open folder: {e}"))?;
    } else if cfg!(target_os = "macos") {
        // On macOS, use open command
        shell.command("open")
            .args([&folder_path])
            .spawn()
            .map_err(|e| format!("Failed to open folder: {e}"))?;
    } else {
        // On Linux, try xdg-open
        shell.command("xdg-open")
            .args([&folder_path])
            .spawn()
            .map_err(|e| format!("Failed to open folder: {e}"))?;
    }
    
    log::info!("[Rust] Successfully opened folder in file explorer");
    Ok(())
}

#[tauri::command]
pub async fn launch_nwn2_game(app: tauri::AppHandle, game_path: Option<String>) -> Result<(), String> {
    log::info!("[Rust] Launching NWN2:EE game");
    
    let installation_path = match game_path {
        Some(path) => path,
        None => {
            match detect_nwn2_installation(app.clone()).await? {
                Some(path) => path,
                None => return Err("NWN2:EE installation not found. Please set the game path in settings.".to_string()),
            }
        }
    };
    
    let base_path = PathBuf::from(&installation_path);
    
    // Determine which executable to use (prefer NWN2Player.exe if available)
    let exe_path = if base_path.join("NWN2Player.exe").exists() {
        base_path.join("NWN2Player.exe")
    } else if base_path.join("NWN2.exe").exists() {
        base_path.join("NWN2.exe") 
    } else {
        return Err(format!("No NWN2 executable found in: {installation_path}"));
    };
    
    log::info!("[Rust] Launching game executable: {}", exe_path.display());
    
    // Launch the game
    let shell = app.shell();
    
    if cfg!(windows) {
        // On Windows, launch directly
        shell.command(&exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch NWN2: {e}"))?;
    } else {
        // On Linux/WSL, might need to use wine or different approach
        // For now, try direct execution
        shell.command(&exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch NWN2: {e}. You may need to configure Wine or use Windows."))?;
    }
    
    log::info!("[Rust] NWN2 game launched successfully");
    Ok(())
}
