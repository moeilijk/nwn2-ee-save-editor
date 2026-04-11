use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use crate::utils::path_discovery::discover_nwn2_paths_rust;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PathSource {
    #[default]
    Discovery,
    Environment,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathSetupMode {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NWN2Paths {
    game_folder: Option<PathBuf>,
    documents_folder: Option<PathBuf>,
    steam_workshop_folder: Option<PathBuf>,

    #[serde(skip)]
    game_folder_source: PathSource,
    #[serde(skip)]
    documents_folder_source: PathSource,
    #[serde(skip)]
    steam_workshop_folder_source: PathSource,

    custom_override_folders: Vec<PathBuf>,
    custom_module_folders: Vec<PathBuf>,
    custom_hak_folders: Vec<PathBuf>,
    setup_mode: Option<PathSetupMode>,
}

impl Default for NWN2Paths {
    fn default() -> Self {
        Self::new()
    }
}

impl NWN2Paths {
    #[instrument(name = "NWN2Paths::new")]
    pub fn new() -> Self {
        info!("Initializing NWN2Paths");
        let mut paths = Self {
            game_folder: None,
            documents_folder: None,
            steam_workshop_folder: None,
            game_folder_source: PathSource::Discovery,
            documents_folder_source: PathSource::Discovery,
            steam_workshop_folder_source: PathSource::Discovery,
            custom_override_folders: Vec::new(),
            custom_module_folders: Vec::new(),
            custom_hak_folders: Vec::new(),
            setup_mode: None,
        };
        paths.load_config();
        info!(
            "NWN2Paths initialized: game={:?}, docs={:?}, workshop={:?}",
            paths.game_folder.as_ref().map(|p| p.display()),
            paths.documents_folder.as_ref().map(|p| p.display()),
            paths.steam_workshop_folder.as_ref().map(|p| p.display())
        );
        paths
    }

    #[instrument(name = "NWN2Paths::load_config", skip(self))]
    fn load_config(&mut self) {
        debug!("Loading NWN2 path configuration");
        let mut config_exists = false;
        if let Some(config_path) = Self::get_config_path()
            && config_path.exists()
            && let Ok(content) = std::fs::read_to_string(&config_path)
            && let Ok(config) = serde_json::from_str::<NWN2PathsConfig>(&content)
        {
            config_exists = true;
            self.setup_mode = config.setup_mode;

            if let Some(game) = config.game_folder {
                let path = PathBuf::from(&game);
                if path.exists() {
                    self.game_folder = Some(path);
                    self.game_folder_source = PathSource::Config;
                }
            }
            if let Some(docs) = config.documents_folder {
                let path = PathBuf::from(&docs);
                if path.exists() {
                    self.documents_folder = Some(path);
                    self.documents_folder_source = PathSource::Config;
                }
            }
            if let Some(workshop) = config.steam_workshop_folder {
                let path = PathBuf::from(&workshop);
                if path.exists() {
                    self.steam_workshop_folder = Some(path);
                    self.steam_workshop_folder_source = PathSource::Config;
                }
            }
            self.custom_override_folders = config
                .custom_override_folders
                .into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();
            self.custom_module_folders = config
                .custom_module_folders
                .into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();
            self.custom_hak_folders = config
                .custom_hak_folders
                .into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();

            if self.setup_mode.is_none()
                && (self.game_folder.is_some()
                    || self.documents_folder.is_some()
                    || self.steam_workshop_folder.is_some()
                    || !self.custom_override_folders.is_empty()
                    || !self.custom_module_folders.is_empty()
                    || !self.custom_hak_folders.is_empty())
            {
                self.setup_mode = Some(PathSetupMode::Manual);
            }
        }

        if config_exists && self.setup_mode.is_none() {
            self.setup_mode = Some(PathSetupMode::Auto);
        }

        let mut has_environment_override = false;
        if let Ok(game_folder) = std::env::var("NWN2_GAME_FOLDER") {
            let path = PathBuf::from(&game_folder);
            if path.exists() {
                self.game_folder = Some(path);
                self.game_folder_source = PathSource::Environment;
                has_environment_override = true;
            }
        }
        if let Ok(docs_folder) = std::env::var("NWN2_DOCUMENTS_FOLDER") {
            let path = PathBuf::from(&docs_folder);
            if path.exists() {
                self.documents_folder = Some(path);
                self.documents_folder_source = PathSource::Environment;
                has_environment_override = true;
            }
        }
        if let Ok(workshop) = std::env::var("NWN2_STEAM_WORKSHOP_FOLDER") {
            let path = PathBuf::from(&workshop);
            if path.exists() {
                self.steam_workshop_folder = Some(path);
                self.steam_workshop_folder_source = PathSource::Environment;
                has_environment_override = true;
            }
        }

        if has_environment_override && self.setup_mode.is_none() {
            self.setup_mode = Some(PathSetupMode::Auto);
        }

        if matches!(self.setup_mode, Some(PathSetupMode::Auto)) {
            self.auto_discover_missing();
        }
    }

    #[instrument(name = "NWN2Paths::auto_discover_missing", skip(self))]
    fn auto_discover_missing(&mut self) {
        debug!("Auto-discovering NWN2 installation paths");
        if let Ok(result) = discover_nwn2_paths_rust(None) {
            debug!(
                "Path discovery found {} candidates",
                result.nwn2_paths.len()
            );
            if self.game_folder.is_none() {
                for path in &result.nwn2_paths {
                    let path = PathBuf::from(path);
                    if !path.to_string_lossy().contains("Documents")
                        && !path.to_string_lossy().contains("My Documents")
                    {
                        self.game_folder = Some(path);
                        self.game_folder_source = PathSource::Discovery;
                        break;
                    }
                }

                if self.game_folder.is_none() && !result.nwn2_paths.is_empty() {
                    self.game_folder = Some(PathBuf::from(&result.nwn2_paths[0]));
                    self.game_folder_source = PathSource::Discovery;
                }
            }

            if self.documents_folder.is_none() {
                self.documents_folder = Self::find_documents_folder();
                if self.documents_folder.is_some() {
                    self.documents_folder_source = PathSource::Discovery;
                }
            }

            if self.steam_workshop_folder.is_none() && !result.steam_paths.is_empty() {
                self.steam_workshop_folder = Self::find_steam_workshop();
                if self.steam_workshop_folder.is_some() {
                    self.steam_workshop_folder_source = PathSource::Discovery;
                }
            }
        }
    }

    fn get_config_path() -> Option<PathBuf> {
        if let Ok(override_path) = std::env::var("NWN2EE_SETTINGS_PATH") {
            return Some(PathBuf::from(override_path));
        }
        dirs::data_dir().map(|d| d.join("nwn2_save_editor").join("settings.json"))
    }

    pub fn save_settings(&self) -> Result<(), std::io::Error> {
        if let Some(config_path) = Self::get_config_path() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let config = NWN2PathsConfig {
                game_folder: if self.game_folder_source == PathSource::Config {
                    self.game_folder
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    None
                },
                documents_folder: if self.documents_folder_source == PathSource::Config {
                    self.documents_folder
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    None
                },
                steam_workshop_folder: if self.steam_workshop_folder_source == PathSource::Config {
                    self.steam_workshop_folder
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    None
                },
                custom_override_folders: self
                    .custom_override_folders
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect(),
                custom_module_folders: self
                    .custom_module_folders
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect(),
                custom_hak_folders: self
                    .custom_hak_folders
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect(),
                setup_mode: self.setup_mode,
            };
            let json = serde_json::to_string_pretty(&config)?;
            std::fs::write(config_path, json)?;
        }
        Ok(())
    }

    fn find_documents_folder() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                let docs = PathBuf::from(&userprofile)
                    .join("Documents")
                    .join("Neverwinter Nights 2");
                if docs.exists() {
                    return Some(docs);
                }
                let my_docs = PathBuf::from(&userprofile)
                    .join("My Documents")
                    .join("Neverwinter Nights 2");
                if my_docs.exists() {
                    return Some(my_docs);
                }
            }
        }

        #[cfg(not(windows))]
        {
            for candidate in Self::non_windows_documents_candidates() {
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    #[cfg(not(windows))]
    fn non_windows_documents_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Some(home) = dirs::home_dir() {
            candidates.push(home.join("Documents").join("Neverwinter Nights 2"));
            candidates.push(home.join(".local/share/Neverwinter Nights 2"));
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(userprofile) = std::env::var("USERPROFILE")
                && let Some(wsl_home) = windows_profile_to_wsl_home(&userprofile)
            {
                candidates.push(wsl_home.join("Documents").join("Neverwinter Nights 2"));
                candidates.push(wsl_home.join("My Documents").join("Neverwinter Nights 2"));
            }

            if let Ok(username) = std::env::var("USER") {
                let wsl_home = PathBuf::from("/mnt/c/Users").join(username);
                candidates.push(wsl_home.join("Documents").join("Neverwinter Nights 2"));
                candidates.push(wsl_home.join("My Documents").join("Neverwinter Nights 2"));
            }
        }

        candidates
    }

    fn find_steam_workshop() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            let program_files =
                std::env::var("ProgramFiles").unwrap_or_else(|_| "C:/Program Files".to_string());
            let program_files_x86 = std::env::var("ProgramFiles(x86)")
                .unwrap_or_else(|_| "C:/Program Files (x86)".to_string());

            let candidates = [
                PathBuf::from(&program_files).join("Steam/steamapps/workshop/content/2738630"),
                PathBuf::from(&program_files_x86).join("Steam/steamapps/workshop/content/2738630"),
            ];

            for path in candidates {
                if path.exists() && path.is_dir() {
                    return Some(path);
                }
            }
        }

        #[cfg(not(windows))]
        {
            if let Some(home) = dirs::home_dir() {
                let path = home.join(".steam/steam/steamapps/workshop/content/2738630");
                if path.exists() && path.is_dir() {
                    return Some(path);
                }
            }
        }

        None
    }

    pub fn game_folder(&self) -> Option<&PathBuf> {
        self.game_folder.as_ref()
    }

    pub fn set_game_folder(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        self.game_folder = Some(path);
        self.game_folder_source = PathSource::Config;
        if self.setup_mode.is_none() {
            self.setup_mode = Some(PathSetupMode::Manual);
        }
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reset_game_folder(&mut self) -> Result<(), String> {
        self.game_folder = None;
        self.setup_mode = Some(PathSetupMode::Auto);
        self.game_folder_source = PathSource::Discovery;
        self.auto_discover_missing();
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn documents_folder(&self) -> Option<&PathBuf> {
        self.documents_folder.as_ref()
    }

    pub fn steam_workshop_folder(&self) -> Option<&PathBuf> {
        self.steam_workshop_folder.as_ref()
    }

    pub fn data(&self) -> Option<PathBuf> {
        self.game_folder.as_ref().map(|g| g.join("Data"))
    }

    pub fn enhanced(&self) -> Option<PathBuf> {
        self.game_folder.as_ref().map(|g| g.join("enhanced"))
    }

    pub fn enhanced_data(&self) -> Option<PathBuf> {
        self.game_folder.as_ref().map(|g| g.join("enhanced/Data"))
    }

    pub fn dialog_tlk(&self) -> Option<PathBuf> {
        self.game_folder.as_ref().map(|g| g.join("dialog.tlk"))
    }

    pub fn saves(&self) -> Option<PathBuf> {
        self.documents_folder.as_ref().map(|d| d.join("saves"))
    }

    pub fn localvault(&self) -> Option<PathBuf> {
        self.documents_folder.as_ref().map(|d| d.join("localvault"))
    }

    pub fn servervault(&self) -> Option<PathBuf> {
        self.documents_folder
            .as_ref()
            .map(|d| d.join("servervault"))
    }

    pub fn override_dir(&self) -> Option<PathBuf> {
        self.documents_folder.as_ref().map(|d| d.join("override"))
    }

    pub fn hak_dir(&self) -> Option<PathBuf> {
        self.documents_folder.as_ref().map(|d| d.join("hak"))
    }

    pub fn modules_dir(&self) -> Option<PathBuf> {
        self.documents_folder.as_ref().map(|d| d.join("modules"))
    }

    pub fn campaigns(&self) -> Option<PathBuf> {
        self.game_folder.as_ref().map(|g| g.join("Campaigns"))
    }

    pub fn is_enhanced_edition(&self) -> bool {
        self.game_folder
            .as_ref()
            .is_some_and(|g| g.join("enhanced").exists() || g.join("enhanced/Data").exists())
    }

    pub fn is_steam_installation(&self) -> bool {
        self.game_folder
            .as_ref()
            .is_some_and(|g| g.join("steam_api.dll").exists() || g.join("steam_api64.dll").exists())
    }

    pub fn is_gog_installation(&self) -> bool {
        self.game_folder.as_ref().is_some_and(|g| {
            g.join("goggame-1207658888.ico").exists()
                || g.join("goglog.ini").exists()
                || g.join("unins000.exe").exists() && !g.join("steam_api.dll").exists()
        })
    }

    pub fn get_game_version(&self) -> Option<String> {
        self.game_folder.as_ref().and_then(|g| {
            let version_file = g.join("version.txt");
            if version_file.exists() {
                std::fs::read_to_string(version_file)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
    }

    pub fn custom_override_folders(&self) -> &[PathBuf] {
        &self.custom_override_folders
    }

    pub fn custom_hak_folders(&self) -> &[PathBuf] {
        &self.custom_hak_folders
    }

    pub fn custom_module_folders(&self) -> &[PathBuf] {
        &self.custom_module_folders
    }

    pub fn user_campaigns(&self) -> Option<PathBuf> {
        self.documents_folder.as_ref().map(|d| d.join("campaigns"))
    }

    pub fn tlk_dir(&self) -> Option<PathBuf> {
        self.documents_folder.as_ref().map(|d| d.join("tlk"))
    }

    pub fn add_custom_override_folder(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        if !self.custom_override_folders.contains(&path) {
            self.custom_override_folders.push(path);
            self.save_settings().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn remove_custom_override_folder(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        self.custom_override_folders.retain(|p| p != &path);
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_documents_folder(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        self.documents_folder = Some(path);
        self.documents_folder_source = PathSource::Config;
        if self.setup_mode.is_none() {
            self.setup_mode = Some(PathSetupMode::Manual);
        }
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reset_documents_folder(&mut self) -> Result<(), String> {
        self.documents_folder = None;
        self.setup_mode = Some(PathSetupMode::Auto);
        self.documents_folder_source = PathSource::Discovery;
        self.documents_folder = Self::find_documents_folder();
        if self.documents_folder.is_some() {
            self.documents_folder_source = PathSource::Discovery;
        }
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_steam_workshop_folder(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        self.steam_workshop_folder = Some(path);
        self.steam_workshop_folder_source = PathSource::Config;
        if self.setup_mode.is_none() {
            self.setup_mode = Some(PathSetupMode::Manual);
        }
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reset_steam_workshop_folder(&mut self) -> Result<(), String> {
        self.steam_workshop_folder = None;
        self.setup_mode = Some(PathSetupMode::Auto);
        self.steam_workshop_folder_source = PathSource::Discovery;
        self.steam_workshop_folder = Self::find_steam_workshop();
        if self.steam_workshop_folder.is_some() {
            self.steam_workshop_folder_source = PathSource::Discovery;
        }
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_custom_hak_folder(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        if !self.custom_hak_folders.contains(&path) {
            self.custom_hak_folders.push(path);
            self.save_settings().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn remove_custom_hak_folder(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        self.custom_hak_folders.retain(|p| p != &path);
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn game_folder_source(&self) -> PathSource {
        self.game_folder_source
    }

    pub fn documents_folder_source(&self) -> PathSource {
        self.documents_folder_source
    }

    pub fn steam_workshop_folder_source(&self) -> PathSource {
        self.steam_workshop_folder_source
    }

    pub fn setup_mode(&self) -> Option<PathSetupMode> {
        self.setup_mode
    }

    pub fn needs_initial_setup(&self) -> bool {
        self.setup_mode.is_none()
    }

    pub fn enable_auto_discovery(&mut self) -> Result<(), String> {
        self.setup_mode = Some(PathSetupMode::Auto);
        self.auto_discover_missing();
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_manual_setup_mode(&mut self) -> Result<(), String> {
        self.setup_mode = Some(PathSetupMode::Manual);

        if self.game_folder_source == PathSource::Discovery {
            self.game_folder = None;
            self.game_folder_source = PathSource::Config;
        }
        if self.documents_folder_source == PathSource::Discovery {
            self.documents_folder = None;
            self.documents_folder_source = PathSource::Config;
        }
        if self.steam_workshop_folder_source == PathSource::Discovery {
            self.steam_workshop_folder = None;
            self.steam_workshop_folder_source = PathSource::Config;
        }

        if self.game_folder.is_none() && self.game_folder_source != PathSource::Environment {
            self.game_folder_source = PathSource::Config;
        }
        if self.documents_folder.is_none()
            && self.documents_folder_source != PathSource::Environment
        {
            self.documents_folder_source = PathSource::Config;
        }
        if self.steam_workshop_folder.is_none()
            && self.steam_workshop_folder_source != PathSource::Environment
        {
            self.steam_workshop_folder_source = PathSource::Config;
        }

        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(all(target_os = "linux", not(windows)))]
fn windows_profile_to_wsl_home(userprofile: &str) -> Option<PathBuf> {
    let normalized = userprofile.replace('\\', "/");
    let mut chars = normalized.chars();

    let drive = chars.next()?;
    if chars.next()? != ':' {
        return None;
    }

    let mut remainder = chars.as_str().trim_start_matches('/').to_string();
    if remainder.is_empty() {
        return None;
    }

    remainder = remainder.trim_end_matches('/').to_string();

    Some(PathBuf::from(format!(
        "/mnt/{}/{}",
        drive.to_ascii_lowercase(),
        remainder
    )))
}

#[derive(Debug, Serialize, Deserialize)]
struct NWN2PathsConfig {
    game_folder: Option<String>,
    documents_folder: Option<String>,
    steam_workshop_folder: Option<String>,
    #[serde(default)]
    custom_override_folders: Vec<String>,
    #[serde(default)]
    custom_module_folders: Vec<String>,
    #[serde(default)]
    custom_hak_folders: Vec<String>,
    #[serde(default)]
    setup_mode: Option<PathSetupMode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn with_test_settings_path<T>(temp_dir: &TempDir, test: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK.lock().expect("env mutex poisoned");
        let settings_path = temp_dir.path().join("settings.json");

        let old_settings = std::env::var("NWN2EE_SETTINGS_PATH").ok();
        let old_game = std::env::var("NWN2_GAME_FOLDER").ok();
        let old_docs = std::env::var("NWN2_DOCUMENTS_FOLDER").ok();
        let old_workshop = std::env::var("NWN2_STEAM_WORKSHOP_FOLDER").ok();

        unsafe {
            std::env::set_var("NWN2EE_SETTINGS_PATH", &settings_path);
            std::env::remove_var("NWN2_GAME_FOLDER");
            std::env::remove_var("NWN2_DOCUMENTS_FOLDER");
            std::env::remove_var("NWN2_STEAM_WORKSHOP_FOLDER");
        }

        let result = test();

        unsafe {
            if let Some(value) = old_settings {
                std::env::set_var("NWN2EE_SETTINGS_PATH", value);
            } else {
                std::env::remove_var("NWN2EE_SETTINGS_PATH");
            }

            if let Some(value) = old_game {
                std::env::set_var("NWN2_GAME_FOLDER", value);
            } else {
                std::env::remove_var("NWN2_GAME_FOLDER");
            }

            if let Some(value) = old_docs {
                std::env::set_var("NWN2_DOCUMENTS_FOLDER", value);
            } else {
                std::env::remove_var("NWN2_DOCUMENTS_FOLDER");
            }

            if let Some(value) = old_workshop {
                std::env::set_var("NWN2_STEAM_WORKSHOP_FOLDER", value);
            } else {
                std::env::remove_var("NWN2_STEAM_WORKSHOP_FOLDER");
            }
        }

        result
    }

    #[test]
    fn test_first_run_requires_explicit_setup_choice() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        with_test_settings_path(&temp_dir, || {
            let paths = NWN2Paths::new();
            assert!(paths.needs_initial_setup());
            assert_eq!(paths.setup_mode(), None);
            assert!(paths.game_folder().is_none());
            assert!(paths.documents_folder().is_none());
            assert!(paths.steam_workshop_folder().is_none());
        });
    }

    #[test]
    fn test_load_legacy_config_with_saved_paths_defaults_to_manual_mode() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let game_dir = temp_dir.path().join("game");
        std::fs::create_dir_all(&game_dir).expect("failed to create game dir");

        let config_path = temp_dir.path().join("settings.json");
        std::fs::write(
            &config_path,
            format!(
                r#"{{
  "game_folder": "{}",
  "documents_folder": null,
  "steam_workshop_folder": null,
  "custom_override_folders": [],
  "custom_module_folders": [],
  "custom_hak_folders": []
}}"#,
                game_dir.to_string_lossy()
            ),
        )
        .expect("failed to write legacy settings");

        with_test_settings_path(&temp_dir, || {
            let paths = NWN2Paths::new();
            assert_eq!(paths.setup_mode(), Some(PathSetupMode::Manual));
            assert_eq!(paths.game_folder(), Some(&game_dir));
        });
    }

    #[test]
    fn test_manual_setup_mode_clears_discovery_paths() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let game_dir = temp_dir.path().join("game");
        let docs_dir = temp_dir.path().join("docs");
        let workshop_dir = temp_dir.path().join("workshop");
        std::fs::create_dir_all(&game_dir).expect("failed to create game dir");
        std::fs::create_dir_all(&docs_dir).expect("failed to create docs dir");
        std::fs::create_dir_all(&workshop_dir).expect("failed to create workshop dir");

        with_test_settings_path(&temp_dir, || {
            let mut paths = NWN2Paths {
                game_folder: Some(game_dir.clone()),
                documents_folder: Some(docs_dir.clone()),
                steam_workshop_folder: Some(workshop_dir.clone()),
                game_folder_source: PathSource::Discovery,
                documents_folder_source: PathSource::Discovery,
                steam_workshop_folder_source: PathSource::Discovery,
                custom_override_folders: Vec::new(),
                custom_module_folders: Vec::new(),
                custom_hak_folders: Vec::new(),
                setup_mode: Some(PathSetupMode::Auto),
            };

            paths
                .set_manual_setup_mode()
                .expect("failed to switch to manual mode");

            assert_eq!(paths.setup_mode(), Some(PathSetupMode::Manual));
            assert!(paths.game_folder().is_none());
            assert!(paths.documents_folder().is_none());
            assert!(paths.steam_workshop_folder().is_none());
            assert_eq!(paths.game_folder_source(), PathSource::Config);
            assert_eq!(paths.documents_folder_source(), PathSource::Config);
            assert_eq!(paths.steam_workshop_folder_source(), PathSource::Config);
        });
    }
}
