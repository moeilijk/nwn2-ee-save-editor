use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, debug, warn, instrument};

use crate::character::{Character, FeatInfo};
use crate::services::resource_manager::ResourceManager;
use crate::services::savegame_handler::SaveGameHandler;

use crate::services::item_property_decoder::ItemPropertyDecoder;

pub struct SessionState {
    pub current_file_path: Option<PathBuf>,
    pub savegame_handler: Option<SaveGameHandler>,
    pub character: Option<Character>,
    pub item_property_decoder: ItemPropertyDecoder,
    pub feat_cache: Option<Vec<FeatInfo>>,
}

impl SessionState {
    #[instrument(name = "SessionState::new", skip_all)]
    pub fn new(resource_manager: Arc<tokio::sync::RwLock<ResourceManager>>) -> Self {
        debug!("Creating SessionState");

        debug!("Initializing ItemPropertyDecoder");
        let item_property_decoder = ItemPropertyDecoder::new(resource_manager);
        debug!("ItemPropertyDecoder created");

        info!("SessionState created successfully");

        Self {
            current_file_path: None,
            savegame_handler: None,
            character: None,
            item_property_decoder,
            feat_cache: None,
        }
    }

    #[instrument(name = "SessionState::load_character", skip(self), fields(file_path = %file_path))]
    pub fn load_character(&mut self, file_path: &str) -> Result<(), String> {
        info!("Loading character from save file");
        let path = PathBuf::from(file_path);

        debug!("Creating SaveGameHandler");
        let handler = SaveGameHandler::new(&path, false, false)
            .map_err(|e| {
                warn!("Failed to create save handler: {}", e);
                format!("Failed to create save handler: {e}")
            })?;
        debug!("SaveGameHandler created");

        debug!("Extracting playerlist.ifo from save archive");
        let playerlist_data = handler.extract_player_data()
            .map_err(|e| {
                warn!("Failed to extract playerlist.ifo: {}", e);
                format!("Failed to extract playerlist.ifo: {e}")
            })?;
        info!("playerlist.ifo extracted ({} bytes)", playerlist_data.len());

        debug!("Parsing GFF data");
        let gff = crate::parsers::gff::GffParser::from_bytes(playerlist_data)
             .map_err(|e| {
                 warn!("GFF parse error: {}", e);
                 format!("GFF Parse error: {e}")
             })?;
        debug!("GFF parsed successfully");

        debug!("Reading playerlist.ifo root struct");
        let root_fields = gff.read_struct_fields(0)
             .map_err(|e| {
                 warn!("Failed to read root struct: {}", e);
                 format!("Failed to read root struct: {e}")
             })?;

        debug!("Extracting Mod_PlayerList[0] (character data)");
        let fields = {
            use crate::parsers::gff::GffValue;
            let mod_player_list = root_fields.get("Mod_PlayerList")
                .ok_or_else(|| {
                    warn!("Mod_PlayerList not found in playerlist.ifo");
                    "Mod_PlayerList not found in playerlist.ifo".to_string()
                })?;

            if let GffValue::List(lazy_structs) = mod_player_list {
                let first = lazy_structs.first()
                    .ok_or_else(|| {
                        warn!("Mod_PlayerList is empty");
                        "Mod_PlayerList is empty".to_string()
                    })?;
                first.force_load()
            } else {
                warn!("Mod_PlayerList is not a list");
                return Err("Mod_PlayerList is not a list".to_string());
            }
        };
        info!("Character data extracted ({} fields)", fields.len());

        debug!("Creating Character from GFF fields");
        let character = Character::from_gff(fields);
        info!("Character created: {} (Level {})",
              character.full_name(),
              character.total_level());

        self.character = Some(character);
        self.savegame_handler = Some(handler);
        self.current_file_path = Some(path);

        info!("Character loaded successfully");
        Ok(())
    }

    pub fn save_character(&mut self) -> Result<(), String> {
        let _handler = self.savegame_handler.as_mut().ok_or("No active save handler")?;
        let _character = self.character.as_ref().ok_or("No character loaded")?;

        // TODO: serialize character data to GFF and write to save file
        // This will be implemented when we need the save functionality
        Ok(())
    }

    pub fn close_character(&mut self) {
        self.character = None;
        self.savegame_handler = None;
        self.current_file_path = None;
        self.feat_cache = None;
    }

    pub fn invalidate_feat_cache(&mut self) {
        self.feat_cache = None;
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.character.as_ref().is_some_and(super::super::character::Character::is_modified)
    }

    pub fn character(&self) -> Option<&Character> {
        self.character.as_ref()
    }

    pub fn character_mut(&mut self) -> Option<&mut Character> {
        self.character.as_mut()
    }

    #[instrument(name = "SessionState::export_to_localvault", skip(self))]
    pub fn export_to_localvault(&self) -> Result<String, String> {
        let handler = self.savegame_handler.as_ref().ok_or("No active save handler")?;
        let character = self.character.as_ref().ok_or("No character loaded")?;

        let nwn2_paths = crate::config::nwn2_paths::NWN2Paths::new();
        let vault_path = nwn2_paths.localvault().ok_or("Could not determine NWN2 localvault path")?;

        if !vault_path.exists() {
            std::fs::create_dir_all(&vault_path)
                .map_err(|e| format!("Failed to create localvault directory: {e}"))?;
        }

        let player_bic_data = handler.extract_player_bic()
            .map_err(|e| format!("Failed to extract player.bic: {e}"))?
            .ok_or("No player.bic found in save")?;

        let first_name = character.first_name();
        let last_name = character.last_name();
        let filename = if last_name.is_empty() {
            format!("{}.bic", first_name)
        } else {
            format!("{} {}.bic", first_name, last_name)
        };

        let sanitized_filename = filename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '.' || *c == '-' || *c == '_')
            .collect::<String>();

        let dest_path = vault_path.join(&sanitized_filename);

        std::fs::write(&dest_path, &player_bic_data)
            .map_err(|e| format!("Failed to write character to vault: {e}"))?;

        info!("Exported character to vault: {}", dest_path.display());

        Ok(dest_path.to_string_lossy().to_string())
    }
}
