use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub language: String,
    pub auto_backup: bool,
    pub backup_count: u32,
    pub last_save_path: Option<PathBuf>,
    pub recent_saves: Vec<PathBuf>,
    pub max_recent_saves: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            auto_backup: true,
            backup_count: 3,
            last_save_path: None,
            recent_saves: Vec::new(),
            max_recent_saves: 10,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Some(config_path) = Self::get_config_path()
            && config_path.exists()
                && let Ok(content) = std::fs::read_to_string(&config_path)
                    && let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(config_path) = Self::get_config_path() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            std::fs::write(config_path, json)?;
        }
        Ok(())
    }

    fn get_config_path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("nwn2_save_editor").join("app_config.json"))
    }

    pub fn add_recent_save(&mut self, path: PathBuf) {
        self.recent_saves.retain(|p| p != &path);
        self.recent_saves.insert(0, path.clone());
        if self.recent_saves.len() > self.max_recent_saves {
            self.recent_saves.truncate(self.max_recent_saves);
        }
        self.last_save_path = Some(path);
        let _ = self.save();
    }

    pub fn clear_recent_saves(&mut self) {
        self.recent_saves.clear();
        let _ = self.save();
    }
}
