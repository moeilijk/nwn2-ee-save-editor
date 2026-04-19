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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::unsafe_derive_deserialize)]
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
        if let Some(config_path) = Self::get_config_path()
            && config_path.exists()
            && let Ok(content) = std::fs::read_to_string(&config_path)
            && let Ok(config) = serde_json::from_str::<NWN2PathsConfig>(&content)
        {
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
        }

        if let Ok(game_folder) = std::env::var("NWN2_GAME_FOLDER") {
            let path = PathBuf::from(&game_folder);
            if path.exists() {
                self.game_folder = Some(path);
                self.game_folder_source = PathSource::Environment;
            }
        }
        if let Ok(docs_folder) = std::env::var("NWN2_DOCUMENTS_FOLDER") {
            let path = PathBuf::from(&docs_folder);
            if path.exists() {
                self.documents_folder = Some(path);
                self.documents_folder_source = PathSource::Environment;
            }
        }
        if let Ok(workshop) = std::env::var("NWN2_STEAM_WORKSHOP_FOLDER") {
            let path = PathBuf::from(&workshop);
            if path.exists() {
                self.steam_workshop_folder = Some(path);
                self.steam_workshop_folder_source = PathSource::Environment;
            }
        }

        if self.game_folder.is_none() {
            self.auto_discover();
        }
    }

    #[instrument(name = "NWN2Paths::auto_discover", skip(self))]
    fn auto_discover(&mut self) {
        debug!("Auto-discovering NWN2 installation paths");
        if let Ok(result) = discover_nwn2_paths_rust(None) {
            debug!(
                "Path discovery found {} candidates",
                result.nwn2_paths.len()
            );
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
            if let Some(home) = dirs::home_dir() {
                let docs = home.join("Documents").join("Neverwinter Nights 2");
                if docs.exists() {
                    return Some(docs);
                }
                let local = home.join(".local/share/Neverwinter Nights 2");
                if local.exists() {
                    return Some(local);
                }
            }
        }

        None
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
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reset_game_folder(&mut self) -> Result<(), String> {
        self.game_folder = None;
        self.game_folder_source = PathSource::Discovery;
        self.auto_discover();
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
            if version_file.exists()
                && let Some(v) = std::fs::read_to_string(version_file)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            {
                return Some(v);
            }
            Self::read_exe_version(&g.join("nwn2.exe"))
        })
    }

    #[cfg(target_os = "windows")]
    fn read_exe_version(exe_path: &std::path::Path) -> Option<String> {
        use std::os::windows::ffi::OsStrExt;

        #[allow(non_snake_case)]
        #[repr(C)]
        struct VS_FIXEDFILEINFO {
            dwSignature: u32,
            dwStrucVersion: u32,
            dwFileVersionMS: u32,
            dwFileVersionLS: u32,
            _rest: [u32; 9],
        }

        unsafe extern "system" {
            fn GetFileVersionInfoSizeW(file: *const u16, handle: *mut u32) -> u32;
            fn GetFileVersionInfoW(file: *const u16, handle: u32, len: u32, data: *mut u8) -> i32;
            fn VerQueryValueW(
                block: *const u8,
                sub_block: *const u16,
                buffer: *mut *const u8,
                len: *mut u32,
            ) -> i32;
        }

        if !exe_path.exists() {
            return None;
        }

        let wide_path: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();

        unsafe {
            let size = GetFileVersionInfoSizeW(wide_path.as_ptr(), std::ptr::null_mut());
            if size == 0 {
                return None;
            }

            let mut buffer = vec![0u8; size as usize];
            if GetFileVersionInfoW(wide_path.as_ptr(), 0, size, buffer.as_mut_ptr()) == 0 {
                return None;
            }

            let mut info_ptr: *const u8 = std::ptr::null();
            let mut info_len: u32 = 0;
            let query: Vec<u16> = "\\".encode_utf16().chain(Some(0)).collect();

            if VerQueryValueW(
                buffer.as_ptr(),
                query.as_ptr(),
                &raw mut info_ptr,
                &raw mut info_len,
            ) == 0
            {
                return None;
            }

            #[allow(clippy::cast_ptr_alignment)]
            let info = &*info_ptr.cast::<VS_FIXEDFILEINFO>();
            Some(format!(
                "{}.{}.{}.{}",
                (info.dwFileVersionMS >> 16) & 0xFFFF,
                info.dwFileVersionMS & 0xFFFF,
                (info.dwFileVersionLS >> 16) & 0xFFFF,
                info.dwFileVersionLS & 0xFFFF,
            ))
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn read_exe_version(_exe_path: &std::path::Path) -> Option<String> {
        None
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
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reset_documents_folder(&mut self) -> Result<(), String> {
        self.documents_folder = None;
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
        self.save_settings().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reset_steam_workshop_folder(&mut self) -> Result<(), String> {
        self.steam_workshop_folder = None;
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

    #[cfg(test)]
    pub fn clear_game_folder(&mut self) {
        self.game_folder = None;
    }

    #[cfg(test)]
    pub fn set_game_folder_for_test(&mut self, path: std::path::PathBuf) {
        self.game_folder = Some(path);
    }
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
}
