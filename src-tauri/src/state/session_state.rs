use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use crate::character::{Character, FeatInfo};
use crate::services::campaign::content::{ModuleInfo, ModuleVariables};
use crate::services::resource_manager::ResourceManager;
use crate::services::savegame_handler::SaveGameHandler;

use crate::services::item_property_decoder::ItemPropertyDecoder;

pub struct SessionState {
    pub current_file_path: Option<PathBuf>,
    pub savegame_handler: Option<SaveGameHandler>,
    pub character: Option<Character>,
    pub item_property_decoder: ItemPropertyDecoder,
    pub feat_cache: Option<Vec<FeatInfo>>,
    pub module_info_cache: Option<(ModuleInfo, ModuleVariables)>,
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
            module_info_cache: None,
        }
    }

    #[instrument(name = "SessionState::load_character", skip(self), fields(file_path = %file_path))]
    pub fn load_character(&mut self, file_path: &str) -> Result<(), String> {
        info!("Loading character from save file");
        let path = PathBuf::from(file_path);

        crate::services::savegame_handler::backup::clear_backup_tracking();

        debug!("Creating SaveGameHandler");
        let handler = SaveGameHandler::new(&path, false, true).map_err(|e| {
            warn!("Failed to create save handler: {}", e);
            format!("Failed to create save handler: {e}")
        })?;
        debug!("SaveGameHandler created");

        debug!("Extracting playerlist.ifo from save archive");
        let playerlist_data = handler.extract_player_data().map_err(|e| {
            warn!("Failed to extract playerlist.ifo: {}", e);
            format!("Failed to extract playerlist.ifo: {e}")
        })?;
        info!("playerlist.ifo extracted ({} bytes)", playerlist_data.len());

        debug!("Parsing GFF data");
        let gff = crate::parsers::gff::GffParser::from_bytes(playerlist_data).map_err(|e| {
            warn!("GFF parse error: {}", e);
            format!("GFF Parse error: {e}")
        })?;
        debug!("GFF parsed successfully");

        debug!("Reading playerlist.ifo root struct");
        let root_fields = gff.read_struct_fields(0).map_err(|e| {
            warn!("Failed to read root struct: {}", e);
            format!("Failed to read root struct: {e}")
        })?;

        debug!("Extracting Mod_PlayerList[0] (character data)");
        let fields = {
            use crate::parsers::gff::GffValue;
            let mod_player_list = root_fields.get("Mod_PlayerList").ok_or_else(|| {
                warn!("Mod_PlayerList not found in playerlist.ifo");
                "Mod_PlayerList not found in playerlist.ifo".to_string()
            })?;

            if let GffValue::List(lazy_structs) = mod_player_list {
                let first = lazy_structs.first().ok_or_else(|| {
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
        info!(
            "Character created: {} (Level {})",
            character.full_name(),
            character.total_level()
        );

        self.character = Some(character);
        self.savegame_handler = Some(handler);
        self.current_file_path = Some(path);
        self.module_info_cache = None;

        info!("Character loaded successfully");
        Ok(())
    }

    pub fn save_character(&mut self, game_data: &crate::loaders::GameData) -> Result<(), String> {
        let handler = self
            .savegame_handler
            .as_mut()
            .ok_or("No active save handler")?;
        let character = self.character.as_ref().ok_or("No character loaded")?;

        if !character.is_modified() {
            info!("No changes to save");
            return Ok(());
        }

        let char_fields = character.clone_gff();

        // Step 1: Build playerlist.ifo
        let ifo_bytes = Self::build_playerlist_ifo(&char_fields)
            .map_err(|e| format!("Failed to build playerlist.ifo: {e}"))?;

        // Step 2: Sync player.bic
        let bic_bytes = Self::build_synced_player_bic(handler, &char_fields)
            .map_err(|e| format!("Failed to sync player.bic: {e}"))?;

        // Step 3: Sync playerinfo.bin
        // playerinfo.bin's subrace field is the load-menu display text; NWN2
        // matches the icon by TLK name, so resolve labels/indices here.
        let subrace = character.race_display_name(game_data);
        let alignment_name = character.alignment().alignment_string();
        let class_entries = character.class_entries();
        let classes: Vec<(String, u8)> = class_entries
            .iter()
            .map(|e| {
                let name = character.get_class_name(e.class_id, game_data);
                (name, e.level as u8)
            })
            .collect();

        handler
            .sync_playerinfo_bin(&char_fields, &subrace, &alignment_name, &classes)
            .map_err(|e| format!("Failed to sync playerinfo.bin: {e}"))?;

        // Step 4: Atomic write of IFO + BIC to zip
        handler
            .update_player_complete(&ifo_bytes, &bic_bytes, None, None)
            .map_err(|e| format!("Failed to write save files: {e}"))?;

        let character = self.character.as_mut().ok_or("No character loaded")?;
        character.mark_saved();

        info!("Character saved successfully");
        Ok(())
    }

    fn build_playerlist_ifo(
        char_fields: &indexmap::IndexMap<String, crate::parsers::gff::GffValue<'static>>,
    ) -> Result<Vec<u8>, String> {
        use crate::parsers::gff::GffValue;

        let mut root = indexmap::IndexMap::new();
        root.insert(
            "Mod_PlayerList".to_string(),
            GffValue::ListOwned(vec![char_fields.clone()]),
        );

        let mut writer = crate::parsers::gff::GffWriter::new("IFO ", "V3.2");
        writer
            .write(root)
            .map_err(|e| format!("GFF write error: {e}"))
    }

    fn build_synced_player_bic(
        handler: &crate::services::savegame_handler::SaveGameHandler,
        char_fields: &indexmap::IndexMap<String, crate::parsers::gff::GffValue<'static>>,
    ) -> Result<Vec<u8>, String> {
        use crate::parsers::gff::{GffParser, GffValue, GffWriter};

        let bic_data = handler
            .extract_player_bic()
            .map_err(|e| format!("Failed to extract player.bic: {e}"))?
            .ok_or("No player.bic found in save")?;

        let bic_gff = GffParser::from_bytes(bic_data)
            .map_err(|e| format!("Failed to parse player.bic: {e}"))?;

        let bic_fields = bic_gff
            .read_struct_fields(0)
            .map_err(|e| format!("Failed to read player.bic fields: {e}"))?;

        let root_struct_id = bic_gff
            .get_struct_id(0)
            .map_err(|e| format!("Failed to get BIC root struct_id: {e}"))?;

        // Merge: for each key in BIC, if it exists in char_fields, overwrite it
        // Preserves BIC-only fields, updates matching fields from character data
        let mut merged: indexmap::IndexMap<String, GffValue<'static>> = bic_fields
            .into_iter()
            .map(|(k, v)| (k, v.force_owned()))
            .collect();

        for (key, value) in char_fields {
            if key.starts_with("__") {
                continue;
            }
            if merged.contains_key(key) {
                merged.insert(key.clone(), value.clone());
            }
        }

        let mut writer = GffWriter::new("BIC ", "V3.2");
        writer
            .write_with_struct_id(merged, root_struct_id)
            .map_err(|e| format!("GFF write error for player.bic: {e}"))
    }

    pub fn close_character(&mut self) {
        self.character = None;
        self.savegame_handler = None;
        self.current_file_path = None;
        self.feat_cache = None;
        self.module_info_cache = None;
        crate::services::savegame_handler::backup::clear_backup_tracking();
    }

    pub fn invalidate_feat_cache(&mut self) {
        self.feat_cache = None;
    }

    pub fn invalidate_module_info_cache(&mut self) {
        self.module_info_cache = None;
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.character
            .as_ref()
            .is_some_and(super::super::character::Character::is_modified)
    }

    pub fn character(&self) -> Option<&Character> {
        self.character.as_ref()
    }

    pub fn character_mut(&mut self) -> Option<&mut Character> {
        self.character.as_mut()
    }

    #[instrument(name = "SessionState::export_to_localvault", skip(self, paths))]
    pub fn export_to_localvault(
        &self,
        paths: &crate::config::nwn2_paths::NWN2Paths,
    ) -> Result<String, String> {
        let handler = self
            .savegame_handler
            .as_ref()
            .ok_or("No active save handler")?;
        let character = self.character.as_ref().ok_or("No character loaded")?;

        let vault_path = paths
            .localvault()
            .ok_or("Could not determine NWN2 localvault path")?;

        if !vault_path.exists() {
            std::fs::create_dir_all(&vault_path)
                .map_err(|e| format!("Failed to create localvault directory: {e}"))?;
        }

        let player_bic_data = handler
            .extract_player_bic()
            .map_err(|e| format!("Failed to extract player.bic: {e}"))?
            .ok_or("No player.bic found in save")?;

        let first_name = character.first_name();
        let last_name = character.last_name();
        let filename = if last_name.is_empty() {
            format!("{first_name}.bic")
        } else {
            format!("{first_name} {last_name}.bic")
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
