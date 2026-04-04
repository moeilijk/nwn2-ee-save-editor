use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use crate::parsers::gff::{GffParser, GffValue, GffWriter};

use crate::character::{Character, FeatInfo};
use crate::loaders::GameData;
use crate::services::PlayerInfo;
use crate::services::resource_manager::ResourceManager;
use crate::services::savegame_handler::SaveGameHandler;

use crate::services::item_property_decoder::ItemPropertyDecoder;

pub struct SessionState {
    pub current_file_path: Option<PathBuf>,
    pub savegame_handler: Option<SaveGameHandler>,
    pub character: Option<Character>,
    pub selected_player_index: usize,
    pub primary_player_index: Option<usize>,
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
            selected_player_index: 0,
            primary_player_index: None,
            item_property_decoder,
            feat_cache: None,
        }
    }

    #[instrument(name = "SessionState::load_character", skip(self), fields(file_path = %file_path))]
    pub fn load_character(
        &mut self,
        file_path: &str,
        player_index: Option<usize>,
    ) -> Result<(), String> {
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

        debug!("Parsing playerlist.ifo GFF data");
        let gff = crate::parsers::gff::GffParser::from_bytes(playerlist_data).map_err(|e| {
            warn!("GFF parse error: {}", e);
            format!("GFF Parse error: {e}")
        })?;
        debug!("playerlist.ifo parsed successfully");

        let player_entries = read_playerlist_entries(gff)?;
        let player_bic_fields = match handler.extract_player_bic() {
            Ok(Some(player_bic_data)) => match read_player_bic_entry(player_bic_data) {
                Ok(fields) => Some(fields),
                Err(err) => {
                    warn!("Failed to parse player.bic while loading save: {}", err);
                    None
                }
            },
            Ok(None) => None,
            Err(err) => {
                warn!("Failed to extract player.bic while loading save: {}", err);
                None
            }
        };
        let primary_player_index =
            resolve_primary_player_index(&player_entries, player_bic_fields.as_ref());
        let selected_player_index = player_index.unwrap_or(primary_player_index.unwrap_or(0));

        let fields = if primary_player_index == Some(selected_player_index) {
            if let Some(fields) = player_bic_fields {
                debug!(
                    "Using player.bic as authoritative source for playerlist slot {}",
                    selected_player_index
                );
                fields
            } else {
                read_playerlist_entry_from_entries(&player_entries, selected_player_index)?
            }
        } else {
            read_playerlist_entry_from_entries(&player_entries, selected_player_index)?
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
        self.selected_player_index = selected_player_index;
        self.primary_player_index = primary_player_index;

        info!("Character loaded successfully");
        Ok(())
    }

    pub fn normalize_loaded_skill_points(&mut self, game_data: &GameData) {
        let Some(character) = self.character.as_mut() else {
            return;
        };

        let summary = character.get_skill_points_summary(game_data);
        if summary.mismatch == 0 {
            return;
        }

        let was_modified = character.is_modified();
        character.normalize_skill_points(game_data);

        if !was_modified {
            character.mark_saved();
        }
    }

    pub fn sync_primary_mirrors(&mut self, game_data: &GameData) -> Result<bool, String> {
        let Some(primary_player_index) = self.primary_player_index else {
            return Ok(false);
        };

        let (playerlist_data, player_bic_data) = {
            let handler = self
                .savegame_handler
                .as_ref()
                .ok_or("No active save handler".to_string())?;
            let playerlist_data = handler
                .extract_player_data()
                .map_err(|e| format!("Failed to read playerlist.ifo: {e}"))?;
            let player_bic_data = handler
                .extract_player_bic()
                .map_err(|e| format!("Failed to read player.bic: {e}"))?;
            (playerlist_data, player_bic_data)
        };

        let authoritative_fields = if self.selected_player_index == primary_player_index {
            let character = self
                .character
                .as_mut()
                .ok_or("No character loaded".to_string())?;
            character.normalize_race_fields_for_save(game_data);
            character.normalize_class_fields_for_save(game_data);
            character.normalize_background_fields_for_save(game_data);
            character.normalize_skill_points(game_data);
            character
                .recalculate_stats(game_data)
                .map_err(|e| format!("Failed to recalculate class-derived stats: {e}"))?;
            character.normalize_level_one_feat_history_for_save();
            character.normalize_single_level_skill_history_for_save();
            character.normalize_level_one_skill_history_for_save(game_data);
            character.clone_gff()
        } else if let Some(player_bic_data) = player_bic_data.as_ref() {
            let mut character = Character::from_gff(read_player_bic_entry(player_bic_data.clone())?);
            character.normalize_race_fields_for_save(game_data);
            character.normalize_class_fields_for_save(game_data);
            character.normalize_background_fields_for_save(game_data);
            character.normalize_skill_points(game_data);
            character
                .recalculate_stats(game_data)
                .map_err(|e| format!("Failed to recalculate class-derived stats: {e}"))?;
            character.normalize_level_one_feat_history_for_save();
            character.normalize_single_level_skill_history_for_save();
            character.normalize_level_one_skill_history_for_save(game_data);
            character.clone_gff()
        } else {
            return Ok(false);
        };

        let playerlist_bytes = serialize_playerlist_bytes(
            playerlist_data,
            &authoritative_fields,
            primary_player_index,
        )?;
        let player_bic_bytes = serialize_player_bic_bytes(player_bic_data, &authoritative_fields)?;

        self.savegame_handler
            .as_mut()
            .ok_or("No active save handler".to_string())?
            .update_player_complete(&playerlist_bytes, Some(&player_bic_bytes), None, None)
            .map_err(|e| format!("Failed to write synchronized save mirrors: {e}"))?;

        let primary_character = Character::from_gff(authoritative_fields);
        let save_dir = self.current_save_dir()?;
        write_playerinfo_for_character(&save_dir, &primary_character, game_data)?;
        sync_primary_localvault_mirror(&primary_character.clone_gff(), Some(player_bic_bytes))?;

        if self.selected_player_index == primary_player_index
            && let Some(character) = self.character.as_mut()
        {
            character.mark_saved();
        }

        info!(
            "Primary save mirrors synchronized for playerlist slot {}",
            primary_player_index
        );

        Ok(true)
    }

    pub fn save_character(&mut self, game_data: &GameData) -> Result<(), String> {
        if self.savegame_handler.is_none() {
            return Err("No active save handler".to_string());
        }
        if self.character.is_none() {
            return Err("No character loaded".to_string());
        }

        let char_fields = {
            let character = self.character.as_mut().unwrap();
            character.normalize_race_fields_for_save(game_data);
            character.normalize_class_fields_for_save(game_data);
            character.normalize_background_fields_for_save(game_data);
            character.normalize_skill_points(game_data);
            character
                .recalculate_stats(game_data)
                .map_err(|e| format!("Failed to recalculate class-derived stats: {e}"))?;
            character.normalize_level_one_feat_history_for_save();
            character.normalize_single_level_skill_history_for_save();
            character.normalize_level_one_skill_history_for_save(game_data);
            character.clone_gff()
        };

        let (playerlist_data, player_bic_data) = {
            let handler = self.savegame_handler.as_ref().unwrap();
            let playerlist_data = handler
                .extract_player_data()
                .map_err(|e| format!("Failed to read playerlist.ifo: {e}"))?;
            let player_bic_data = handler
                .extract_player_bic()
                .map_err(|e| format!("Failed to read player.bic: {e}"))?;
            (playerlist_data, player_bic_data)
        };

        let playerlist_bytes =
            serialize_playerlist_bytes(playerlist_data, &char_fields, self.selected_player_index)?;
        let update_primary_player_files =
            self.primary_player_index == Some(self.selected_player_index);
        let player_bic_bytes = if update_primary_player_files {
            Some(serialize_player_bic_bytes(player_bic_data, &char_fields)?)
        } else {
            None
        };

        self.savegame_handler
            .as_mut()
            .unwrap()
            .update_player_complete(&playerlist_bytes, player_bic_bytes.as_deref(), None, None)
            .map_err(|e| format!("Failed to write save file: {e}"))?;

        if update_primary_player_files {
            self.write_playerinfo(game_data)?;
            sync_primary_localvault_mirror(&char_fields, player_bic_bytes)?;
        }

        // Clear modified flag
        self.character.as_mut().unwrap().mark_saved();

        info!(
            "Character saved successfully (playerlist={} bytes, player.bic_updated={})",
            playerlist_bytes.len(),
            update_primary_player_files
        );
        Ok(())
    }

    pub fn close_character(&mut self) {
        self.character = None;
        self.savegame_handler = None;
        self.current_file_path = None;
        self.selected_player_index = 0;
        self.primary_player_index = None;
        self.feat_cache = None;
        crate::services::savegame_handler::backup::clear_backup_tracking();
    }

    pub fn invalidate_feat_cache(&mut self) {
        self.feat_cache = None;
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

    fn current_save_dir(&self) -> Result<PathBuf, String> {
        let current_path = self
            .current_file_path
            .as_ref()
            .ok_or("No current save path")?;

        if current_path.is_dir() {
            Ok(current_path.clone())
        } else {
            current_path
                .parent()
                .map(PathBuf::from)
                .ok_or_else(|| "Failed to determine save directory".to_string())
        }
    }

    fn write_playerinfo(&self, game_data: &GameData) -> Result<(), String> {
        let character = self.character.as_ref().ok_or("No character loaded")?;
        let save_dir = self.current_save_dir()?;
        write_playerinfo_for_character(&save_dir, character, game_data)
    }

    #[instrument(name = "SessionState::export_to_localvault", skip(self))]
    pub fn export_to_localvault(&self) -> Result<String, String> {
        let handler = self
            .savegame_handler
            .as_ref()
            .ok_or("No active save handler")?;
        let character = self.character.as_ref().ok_or("No character loaded")?;

        let player_bic_data = handler
            .extract_player_bic()
            .map_err(|e| format!("Failed to extract player.bic: {e}"))?;
        let current_character_fields = character.clone_gff();
        let dest_path = sync_primary_localvault_mirror(&current_character_fields, player_bic_data)?;

        info!("Exported character to vault: {}", dest_path.display());

        Ok(dest_path.to_string_lossy().to_string())
    }
}

fn sync_primary_localvault_mirror(
    character_fields: &IndexMap<String, GffValue<'static>>,
    source_player_bic: Option<Vec<u8>>,
) -> Result<PathBuf, String> {
    let nwn2_paths = crate::config::nwn2_paths::NWN2Paths::new();
    let vault_path = nwn2_paths
        .localvault()
        .ok_or("Could not determine NWN2 localvault path")?;
    write_localvault_character(&vault_path, character_fields, source_player_bic)
}

fn write_localvault_character(
    vault_path: &Path,
    character_fields: &IndexMap<String, GffValue<'static>>,
    source_player_bic: Option<Vec<u8>>,
) -> Result<PathBuf, String> {
    if !vault_path.exists() {
        std::fs::create_dir_all(vault_path)
            .map_err(|e| format!("Failed to create localvault directory: {e}"))?;
    }

    let standalone_fields = sanitize_localvault_character_fields(character_fields);
    let player_bic_data = serialize_player_bic_bytes(source_player_bic, &standalone_fields)?;

    let canonical_filename = canonical_localvault_filename(&standalone_fields);
    let dest_path = vault_path.join(&canonical_filename);

    std::fs::write(&dest_path, &player_bic_data)
        .map_err(|e| format!("Failed to write character to vault: {e}"))?;

    let legacy_filename = legacy_localvault_filename(&standalone_fields);
    if legacy_filename != canonical_filename {
        let legacy_path = vault_path.join(legacy_filename);
        if legacy_path.exists() {
            std::fs::remove_file(&legacy_path)
                .map_err(|e| format!("Failed to remove stale legacy localvault character: {e}"))?;
        }
    }

    quarantine_conflicting_localvault_variants(vault_path, &standalone_fields, &canonical_filename)?;

    Ok(dest_path)
}

fn sanitize_localvault_character_fields(
    character_fields: &IndexMap<String, GffValue<'static>>,
) -> IndexMap<String, GffValue<'static>> {
    let mut standalone_fields = character_fields.clone();
    standalone_fields.retain(|key, _| !key.starts_with("Mod_") && key != "ObjectId");
    standalone_fields
}

fn canonical_localvault_filename(character_fields: &IndexMap<String, GffValue<'static>>) -> String {
    let first_name = extract_locstring(character_fields, "FirstName").unwrap_or_default();
    let last_name = extract_locstring(character_fields, "LastName").unwrap_or_default();
    let mut stem = String::new();

    for ch in first_name.chars().chain(last_name.chars()) {
        if ch.is_alphanumeric() {
            stem.extend(ch.to_lowercase());
        }
    }

    if stem.is_empty() {
        "character".to_string()
    } else {
        format!("{stem}.bic")
    }
}

fn legacy_localvault_filename(character_fields: &IndexMap<String, GffValue<'static>>) -> String {
    let first_name = extract_locstring(character_fields, "FirstName").unwrap_or_default();
    let last_name = extract_locstring(character_fields, "LastName").unwrap_or_default();
    let filename = if last_name.is_empty() {
        format!("{first_name}.bic")
    } else {
        format!("{first_name} {last_name}.bic")
    };

    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>()
}

fn canonical_localvault_stem(character_fields: &IndexMap<String, GffValue<'static>>) -> String {
    canonical_localvault_filename(character_fields)
        .strip_suffix(".bic")
        .unwrap_or("character")
        .to_string()
}

fn quarantine_conflicting_localvault_variants(
    vault_path: &Path,
    character_fields: &IndexMap<String, GffValue<'static>>,
    canonical_filename: &str,
) -> Result<(), String> {
    let canonical_stem = canonical_localvault_stem(character_fields);
    let quarantine_dir = localvault_conflict_quarantine_dir(vault_path);

    migrate_nested_localvault_conflicts(vault_path, &quarantine_dir)?;

    for entry in std::fs::read_dir(vault_path)
        .map_err(|e| format!("Failed to scan localvault directory: {e}"))?
    {
        let entry = entry.map_err(|e| format!("Failed to read localvault entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let file_name_lower = file_name.to_ascii_lowercase();
        if file_name_lower == canonical_filename.to_ascii_lowercase() {
            continue;
        }
        if !file_name_lower.ends_with(".bic") {
            continue;
        }

        let Some(stem) = file_name_lower.strip_suffix(".bic") else {
            continue;
        };
        let Some(suffix) = stem.strip_prefix(&canonical_stem) else {
            continue;
        };
        if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        if !quarantine_dir.exists() {
            std::fs::create_dir_all(&quarantine_dir).map_err(|e| {
                format!("Failed to create localvault conflict quarantine directory: {e}")
            })?;
        }

        let mut dest_path = quarantine_dir.join(file_name);
        if dest_path.exists() {
            let mut counter = 1usize;
            loop {
                let candidate = quarantine_dir.join(format!("{stem}_{counter}.bic"));
                if !candidate.exists() {
                    dest_path = candidate;
                    break;
                }
                counter += 1;
            }
        }

        std::fs::rename(&path, &dest_path).map_err(|e| {
            format!(
                "Failed to quarantine conflicting localvault variant {}: {e}",
                path.display()
            )
        })?;
    }

    Ok(())
}

fn localvault_conflict_quarantine_dir(vault_path: &Path) -> PathBuf {
    vault_path
        .parent()
        .unwrap_or(vault_path)
        .join(".nwn2ee-save-editor-conflicts")
        .join("localvault")
}

fn migrate_nested_localvault_conflicts(vault_path: &Path, quarantine_dir: &Path) -> Result<(), String> {
    let nested_quarantine_dir = vault_path.join(".nwn2ee-save-editor-conflicts");
    if !nested_quarantine_dir.exists() {
        return Ok(());
    }

    if !quarantine_dir.exists() {
        std::fs::create_dir_all(quarantine_dir)
            .map_err(|e| format!("Failed to create localvault conflict quarantine directory: {e}"))?;
    }

    for entry in std::fs::read_dir(&nested_quarantine_dir)
        .map_err(|e| format!("Failed to scan nested localvault conflict directory: {e}"))?
    {
        let entry = entry.map_err(|e| format!("Failed to read nested localvault conflict entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name() else {
            continue;
        };
        let mut dest_path = quarantine_dir.join(file_name);
        if dest_path.exists() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("character");
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("bic");
            let mut counter = 1usize;
            loop {
                let candidate = quarantine_dir.join(format!("{stem}_{counter}.{ext}"));
                if !candidate.exists() {
                    dest_path = candidate;
                    break;
                }
                counter += 1;
            }
        }

        std::fs::rename(&path, &dest_path).map_err(|e| {
            format!(
                "Failed to migrate nested localvault conflict {}: {e}",
                path.display()
            )
        })?;
    }

    if nested_quarantine_dir.exists()
        && std::fs::read_dir(&nested_quarantine_dir)
            .map_err(|e| format!("Failed to inspect nested localvault conflict directory: {e}"))?
            .next()
            .is_none()
    {
        std::fs::remove_dir(&nested_quarantine_dir).map_err(|e| {
            format!(
                "Failed to remove empty nested localvault conflict directory {}: {e}",
                nested_quarantine_dir.display()
            )
        })?;
    }

    Ok(())
}

fn write_playerinfo_for_character(
    save_dir: &std::path::Path,
    character: &Character,
    game_data: &GameData,
) -> Result<(), String> {
    let playerinfo_path = save_dir.join("playerinfo.bin");

    let mut player_info = if playerinfo_path.exists() {
        PlayerInfo::load(&playerinfo_path)
            .map_err(|e| format!("Failed to read playerinfo.bin: {e}"))?
    } else {
        PlayerInfo::new()
    };

    let race_label = character.race_name(game_data);
    let alignment_name = character.alignment().alignment_string();
    let classes = character
        .class_entries()
        .into_iter()
        .map(|entry| {
            let level = entry.level.clamp(0, i32::from(u8::MAX)) as u8;
            (character.get_class_name(entry.class_id, game_data), level)
        })
        .collect::<Vec<_>>();

    player_info.update_from_gff_data(character.gff(), &race_label, &alignment_name, &classes);
    player_info.data.unknown4 = character.background_id(game_data).unwrap_or(0) as u32;
    player_info
        .save(&playerinfo_path)
        .map_err(|e| format!("Failed to write playerinfo.bin: {e}"))?;

    Ok(())
}

fn extract_locstring(fields: &IndexMap<String, GffValue<'_>>, key: &str) -> Option<String> {
    match fields.get(key)? {
        GffValue::LocString(loc) => loc.substrings.first().map(|s| s.string.to_string()),
        GffValue::String(value) | GffValue::ResRef(value) => Some(value.to_string()),
        _ => None,
    }
}

fn serialize_playerlist_bytes(
    playerlist_data: Vec<u8>,
    character_fields: &IndexMap<String, GffValue<'static>>,
    player_index: usize,
) -> Result<Vec<u8>, String> {
    let gff = GffParser::from_bytes(playerlist_data)
        .map_err(|e| format!("playerlist.ifo parse error: {e}"))?;

    let file_type = gff.file_type.clone();
    let file_version = gff.file_version.clone();

    let root_fields_raw = gff
        .read_struct_fields(0)
        .map_err(|e| format!("Failed to read playerlist.ifo root struct: {e}"))?;

    let mut root_fields: IndexMap<String, GffValue<'static>> = root_fields_raw
        .into_iter()
        .map(|(k, v)| (k, v.force_owned()))
        .collect();

    match root_fields.get_mut("Mod_PlayerList") {
        Some(GffValue::ListOwned(players)) => {
            if players.is_empty() {
                if player_index == 0 {
                    players.push(character_fields.clone());
                } else {
                    return Err(format!(
                        "Selected player index {player_index} is out of range for an empty Mod_PlayerList"
                    ));
                }
            } else {
                let Some(player_entry) = players.get_mut(player_index) else {
                    return Err(format!(
                        "Selected player index {player_index} is out of range for Mod_PlayerList with {} entries",
                        players.len()
                    ));
                };
                *player_entry = character_fields.clone();
            }
        }
        Some(_) => {
            return Err("Mod_PlayerList in playerlist.ifo is not a list".to_string());
        }
        None => {
            if player_index == 0 {
                root_fields.insert(
                    "Mod_PlayerList".to_string(),
                    GffValue::ListOwned(vec![character_fields.clone()]),
                );
            } else {
                return Err(format!(
                    "Selected player index {player_index} is out of range because Mod_PlayerList is missing"
                ));
            }
        }
    }

    GffWriter::new(&file_type, &file_version)
        .write(root_fields)
        .map_err(|e| format!("playerlist.ifo serialization error: {e}"))
}

pub(crate) fn read_playerlist_entries(
    gff: Arc<GffParser>,
) -> Result<Vec<IndexMap<String, GffValue<'static>>>, String> {
    debug!("Reading playerlist.ifo root struct");
    let root_fields = gff.read_struct_fields(0).map_err(|e| {
        warn!("Failed to read root struct: {}", e);
        format!("Failed to read root struct: {e}")
    })?;

    use crate::parsers::gff::GffValue;
    let mod_player_list = root_fields.get("Mod_PlayerList").ok_or_else(|| {
        warn!("Mod_PlayerList not found in playerlist.ifo");
        "Mod_PlayerList not found in playerlist.ifo".to_string()
    })?;

    if let GffValue::List(lazy_structs) = mod_player_list {
        if lazy_structs.is_empty() {
            warn!("Mod_PlayerList is empty");
            return Err("Mod_PlayerList is empty".to_string());
        }

        Ok(lazy_structs
            .iter()
            .map(|entry| entry.force_load())
            .collect())
    } else {
        warn!("Mod_PlayerList is not a list");
        Err("Mod_PlayerList is not a list".to_string())
    }
}

fn read_playerlist_entry_from_entries(
    entries: &[IndexMap<String, GffValue<'static>>],
    player_index: usize,
) -> Result<IndexMap<String, GffValue<'static>>, String> {
    entries.get(player_index).cloned().ok_or_else(|| {
        format!(
            "Selected player index {player_index} is out of range for Mod_PlayerList with {} entries",
            entries.len()
        )
    })
}

pub(crate) fn read_player_bic_entry(
    player_bic_data: Vec<u8>,
) -> Result<IndexMap<String, GffValue<'static>>, String> {
    let gff = GffParser::from_bytes(player_bic_data).map_err(|e| {
        warn!("Failed to parse player.bic: {}", e);
        format!("Failed to parse player.bic: {e}")
    })?;

    let root_fields = gff.read_struct_fields(0).map_err(|e| {
        warn!("Failed to read player.bic root struct: {}", e);
        format!("Failed to read player.bic root struct: {e}")
    })?;

    Ok(root_fields
        .into_iter()
        .map(|(key, value)| (key, value.force_owned()))
        .collect())
}

pub(crate) fn resolve_primary_player_index(
    player_entries: &[IndexMap<String, GffValue<'static>>],
    player_bic_fields: Option<&IndexMap<String, GffValue<'static>>>,
) -> Option<usize> {
    if player_entries.len() == 1 {
        return Some(0);
    }

    let Some(player_bic_fields) = player_bic_fields else {
        return None;
    };

    let player_bic_name = Character::from_gff(player_bic_fields.clone()).full_name();
    if player_bic_name.trim().is_empty() {
        warn!("player.bic has no character name; refusing to infer a primary multiplayer slot");
        return None;
    }

    let matching_indices = player_entries
        .iter()
        .enumerate()
        .filter_map(|(index, fields)| {
            (Character::from_gff(fields.clone()).full_name() == player_bic_name).then_some(index)
        })
        .collect::<Vec<_>>();

    match matching_indices.as_slice() {
        [index] => Some(*index),
        [] => {
            warn!(
                "player.bic name '{}' did not match any Mod_PlayerList entry; refusing to infer a primary multiplayer slot",
                player_bic_name
            );
            None
        }
        _ => {
            warn!(
                "player.bic name '{}' matched multiple Mod_PlayerList entries; refusing to infer a primary multiplayer slot",
                player_bic_name
            );
            None
        }
    }
}

fn serialize_player_bic_bytes(
    player_bic_data: Option<Vec<u8>>,
    character_fields: &IndexMap<String, GffValue<'static>>,
) -> Result<Vec<u8>, String> {
    let (file_type, file_version) = if let Some(player_bic_data) = player_bic_data {
        let gff = GffParser::from_bytes(player_bic_data)
            .map_err(|e| format!("player.bic parse error: {e}"))?;
        let file_type = gff.file_type.clone();
        let file_version = gff.file_version.clone();
        (file_type, file_version)
    } else {
        ("BIC ".to_string(), "V3.2".to_string())
    };

    GffWriter::new(&file_type, &file_version)
        .write(character_fields.clone())
        .map_err(|e| format!("player.bic serialization error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::fs::File;
    use std::path::Path;
    use std::sync::Mutex;
    use std::sync::Arc as StdArc;
    use tempfile::TempDir;
    use zip::CompressionMethod;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use crate::config::NWN2Paths;
    use crate::loaders::{GameData, LoadedTable};
    use crate::parsers::gff::{GffValue, LocalizedString, LocalizedSubstring};
    use crate::parsers::tda::TDAParser;
    use crate::parsers::tlk::TLKParser;
    use crate::services::resource_manager::ResourceManager;
    use crate::services::{PlayerClassEntry, PlayerInfoData};

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn create_test_character_fields(age: i32, xp: u32) -> IndexMap<String, GffValue<'static>> {
        let mut fields = IndexMap::new();
        fields.insert("Age".to_string(), GffValue::Int(age));
        fields.insert("Experience".to_string(), GffValue::Dword(xp));
        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![]));
        fields
    }

    fn create_test_locstring(value: &str) -> GffValue<'static> {
        GffValue::LocString(LocalizedString {
            string_ref: -1,
            substrings: vec![LocalizedSubstring {
                string: value.to_string().into(),
                language: 0,
                gender: 0,
            }],
        })
    }

    fn create_named_test_character_fields(
        first_name: &str,
        last_name: &str,
        age: i32,
        xp: u32,
    ) -> IndexMap<String, GffValue<'static>> {
        let mut fields = create_test_character_fields(age, xp);
        fields.insert("FirstName".to_string(), create_test_locstring(first_name));
        fields.insert("LastName".to_string(), create_test_locstring(last_name));
        fields
    }

    fn create_named_test_character_fields_with_classes(
        first_name: &str,
        last_name: &str,
        age: i32,
        xp: u32,
        classes: &[(u8, i16)],
    ) -> IndexMap<String, GffValue<'static>> {
        let mut fields = create_named_test_character_fields(first_name, last_name, age, xp);
        let class_list = classes
            .iter()
            .map(|(class_id, class_level)| {
                let mut class_entry = IndexMap::new();
                class_entry.insert("Class".to_string(), GffValue::Byte(*class_id));
                class_entry.insert("ClassLevel".to_string(), GffValue::Short(*class_level));
                class_entry
            })
            .collect();
        fields.insert("ClassList".to_string(), GffValue::ListOwned(class_list));
        fields
    }

    fn create_test_character_fields_with_classes(
        age: i32,
        xp: u32,
        classes: &[(u8, i16)],
    ) -> IndexMap<String, GffValue<'static>> {
        let mut fields = create_test_character_fields(age, xp);
        let class_list = classes
            .iter()
            .map(|(class_id, class_level)| {
                let mut class_entry = IndexMap::new();
                class_entry.insert("Class".to_string(), GffValue::Byte(*class_id));
                class_entry.insert("ClassLevel".to_string(), GffValue::Short(*class_level));
                class_entry
            })
            .collect();
        fields.insert("ClassList".to_string(), GffValue::ListOwned(class_list));
        fields
    }

    fn zero_rank_skill_list(skill_count: usize) -> Vec<IndexMap<String, GffValue<'static>>> {
        let mut skill_list = Vec::with_capacity(skill_count);
        for _ in 0..skill_count {
            let mut skill = IndexMap::new();
            skill.insert("Rank".to_string(), GffValue::Byte(0));
            skill_list.push(skill);
        }
        skill_list
    }

    fn create_history_entry(
        class_id: u8,
        hit_die: u8,
        skill_points: i16,
        skill_count: usize,
    ) -> IndexMap<String, GffValue<'static>> {
        let mut entry = IndexMap::new();
        entry.insert("LvlStatClass".to_string(), GffValue::Byte(class_id));
        entry.insert("LvlStatHitDie".to_string(), GffValue::Byte(hit_die));
        entry.insert("EpicLevel".to_string(), GffValue::Byte(0));
        entry.insert("SkillPoints".to_string(), GffValue::Short(skill_points));
        entry.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(zero_rank_skill_list(skill_count)),
        );
        entry.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));
        entry
    }

    fn write_gff_bytes(
        file_type: &str,
        file_version: &str,
        fields: IndexMap<String, GffValue<'static>>,
    ) -> Vec<u8> {
        GffWriter::new(file_type, file_version)
            .write(fields)
            .expect("Failed to write test GFF")
    }

    fn create_test_save(
        playerlist_entries: Vec<IndexMap<String, GffValue<'static>>>,
        player_bic_fields: IndexMap<String, GffValue<'static>>,
        playerinfo: Option<PlayerInfoData>,
    ) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let save_dir = temp_dir.path().join("TestSave");
        std::fs::create_dir_all(&save_dir).expect("Failed to create save dir");

        let mut playerlist_root = IndexMap::new();
        playerlist_root.insert(
            "Mod_PlayerList".to_string(),
            GffValue::ListOwned(playerlist_entries),
        );
        playerlist_root.insert(
            "SaveName".to_string(),
            GffValue::String("SyntheticSave".into()),
        );

        let playerlist_bytes = write_gff_bytes("IFO ", "V3.2", playerlist_root);
        let player_bic_bytes = write_gff_bytes("BIC ", "V3.2", player_bic_fields);

        let zip_path = save_dir.join("resgff.zip");
        let file = File::create(&zip_path).expect("Failed to create resgff.zip");
        let mut archive = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        archive
            .start_file("playerlist.ifo", options)
            .expect("Failed to create playerlist entry");
        use std::io::Write as _;
        archive
            .write_all(&playerlist_bytes)
            .expect("Failed to write playerlist.ifo");
        archive
            .start_file("player.bic", options)
            .expect("Failed to create player.bic entry");
        archive
            .write_all(&player_bic_bytes)
            .expect("Failed to write player.bic");
        archive.finish().expect("Failed to finalize save zip");

        if let Some(playerinfo_data) = playerinfo {
            let mut player_info = PlayerInfo::new();
            player_info.data = playerinfo_data;
            player_info
                .save(save_dir.join("playerinfo.bin"))
                .expect("Failed to write playerinfo.bin");
        }

        (temp_dir, save_dir)
    }

    fn create_session_state() -> SessionState {
        let paths = Arc::new(tokio::sync::RwLock::new(NWN2Paths::new()));
        let resource_manager = Arc::new(tokio::sync::RwLock::new(ResourceManager::new(paths)));
        SessionState::new(resource_manager)
    }

    fn create_empty_game_data() -> GameData {
        GameData::new(StdArc::new(std::sync::RwLock::new(TLKParser::default())))
    }

    fn create_game_data_with_classes() -> GameData {
        let mut game_data = create_empty_game_data();
        let mut parser = TDAParser::new();
        parser
            .parse_from_string(
                "2DA V2.0\n\nLabel HitDie SkillPointBase AttackBonusTable SavingThrowTable Package SpellCaster\n\
0 Barbarian 12 4 **** **** 0 0\n\
1 Bard 6 6 **** **** 1 1\n\
2 Cleric 8 2 **** **** 2 1\n\
3 Fighter 10 2 **** **** 3 0\n",
            )
            .expect("Failed to parse test classes 2DA");
        game_data.tables.insert(
            "classes".to_string(),
            LoadedTable::new("classes".to_string(), StdArc::new(parser)),
        );
        game_data
    }

    fn create_game_data_with_classes_and_backgrounds() -> GameData {
        let mut game_data = create_game_data_with_classes();
        let mut parser = TDAParser::new();
        parser
            .parse_from_string(
                "2DA V2.0\n\nlabel DisplayFeat REMOVED\n\
0 None 0 1\n\
1 Bully 1717 0\n\
8 NaturalLeader 1724 0\n",
            )
            .expect("Failed to parse test backgrounds 2DA");
        game_data.tables.insert(
            "backgrounds".to_string(),
            LoadedTable::new("backgrounds".to_string(), StdArc::new(parser)),
        );
        game_data
    }

    fn read_playerlist_entry_ages(save_dir: &Path) -> Vec<i32> {
        let handler =
            SaveGameHandler::new(save_dir, false, false).expect("Failed to open saved test save");
        let playerlist_data = handler
            .extract_player_data()
            .expect("Failed to read playerlist.ifo");
        let parser = GffParser::from_bytes(playerlist_data).expect("Failed to parse playerlist");
        let root = parser
            .read_struct_fields(0)
            .expect("Failed to read playerlist root");
        let GffValue::List(players) = root
            .get("Mod_PlayerList")
            .expect("Mod_PlayerList should exist")
        else {
            panic!("Mod_PlayerList should be a list");
        };

        players
            .iter()
            .map(|entry| {
                let fields = entry.force_load();
                match fields.get("Age") {
                    Some(GffValue::Int(age)) => *age,
                    Some(other) => panic!("Unexpected Age value: {other:?}"),
                    None => panic!("Age field missing"),
                }
            })
            .collect()
    }

    fn read_player_bic_age(save_dir: &Path) -> i32 {
        let handler =
            SaveGameHandler::new(save_dir, false, false).expect("Failed to open saved test save");
        let player_bic_data = handler
            .extract_player_bic()
            .expect("Failed to read player.bic")
            .expect("player.bic should exist");
        let parser = GffParser::from_bytes(player_bic_data).expect("Failed to parse player.bic");
        let root = parser
            .read_struct_fields(0)
            .expect("Failed to read player.bic root");
        match root.get("Age") {
            Some(GffValue::Int(age)) => *age,
            Some(other) => panic!("Unexpected player.bic Age value: {other:?}"),
            None => panic!("Age field missing in player.bic"),
        }
    }

    fn read_player_bic_skill_points(save_dir: &Path) -> i32 {
        let handler =
            SaveGameHandler::new(save_dir, false, false).expect("Failed to open saved test save");
        let player_bic_data = handler
            .extract_player_bic()
            .expect("Failed to read player.bic")
            .expect("player.bic should exist");
        let parser = GffParser::from_bytes(player_bic_data).expect("Failed to parse player.bic");
        let root = parser
            .read_struct_fields(0)
            .expect("Failed to read player.bic root");
        root.get("SkillPoints")
            .and_then(crate::character::gff_helpers::gff_value_to_i32)
            .expect("SkillPoints should exist in player.bic")
    }

    fn read_player_bic_char_background(save_dir: &Path) -> Option<i32> {
        let handler =
            SaveGameHandler::new(save_dir, false, false).expect("Failed to open saved test save");
        let player_bic_data = handler
            .extract_player_bic()
            .expect("Failed to read player.bic")
            .expect("player.bic should exist");
        let parser = GffParser::from_bytes(player_bic_data).expect("Failed to parse player.bic");
        let root = parser
            .read_struct_fields(0)
            .expect("Failed to read player.bic root");
        root.get("CharBackground")
            .and_then(crate::character::gff_helpers::gff_value_to_i32)
    }

    fn read_playerlist_entry_char_background(save_dir: &Path, player_index: usize) -> Option<i32> {
        let handler =
            SaveGameHandler::new(save_dir, false, false).expect("Failed to open saved test save");
        let playerlist_data = handler
            .extract_player_data()
            .expect("Failed to read playerlist.ifo");
        let parser = GffParser::from_bytes(playerlist_data).expect("Failed to parse playerlist.ifo");
        let entries = read_playerlist_entries(parser).expect("Failed to read playerlist entries");

        entries
            .get(player_index)
            .and_then(|fields| fields.get("CharBackground"))
            .and_then(crate::character::gff_helpers::gff_value_to_i32)
    }

    fn read_playerinfo_classes(save_dir: &Path) -> Vec<PlayerClassEntry> {
        PlayerInfo::load(save_dir.join("playerinfo.bin"))
            .expect("Failed to load playerinfo.bin")
            .data
            .classes
    }

    fn read_playerinfo_background_row(save_dir: &Path) -> u32 {
        PlayerInfo::load(save_dir.join("playerinfo.bin"))
            .expect("Failed to load playerinfo.bin")
            .data
            .unknown4
    }

    fn read_playerinfo_name(save_dir: &Path) -> String {
        PlayerInfo::load(save_dir.join("playerinfo.bin"))
            .expect("Failed to load playerinfo.bin")
            .data
            .display_name()
    }

    fn read_gff_root_fields(path: &Path) -> IndexMap<String, GffValue<'static>> {
        let data = std::fs::read(path).expect("Failed to read GFF file");
        let parser = GffParser::from_bytes(data).expect("Failed to parse GFF file");
        parser
            .read_struct_fields(0)
            .expect("Failed to read GFF root")
            .into_iter()
            .map(|(k, v)| (k, v.force_owned()))
            .collect()
    }

    fn with_temp_documents_folder<T>(f: impl FnOnce(&Path) -> T) -> T {
        let _guard = ENV_LOCK.lock().expect("Failed to lock env mutex");
        let temp_dir = TempDir::new().expect("Failed to create temp docs dir");
        let docs_dir = temp_dir.path().join("Neverwinter Nights 2");
        std::fs::create_dir_all(docs_dir.join("localvault")).expect("Failed to create localvault");

        let old_documents = std::env::var_os("NWN2_DOCUMENTS_FOLDER");
        let old_settings = std::env::var_os("NWN2EE_SETTINGS_PATH");
        unsafe {
            std::env::set_var("NWN2_DOCUMENTS_FOLDER", &docs_dir);
            std::env::remove_var("NWN2EE_SETTINGS_PATH");
        }

        let result = f(&docs_dir);

        match old_documents {
            Some(value) => unsafe { std::env::set_var("NWN2_DOCUMENTS_FOLDER", value) },
            None => unsafe { std::env::remove_var("NWN2_DOCUMENTS_FOLDER") },
        }
        match old_settings {
            Some(value) => unsafe { std::env::set_var("NWN2EE_SETTINGS_PATH", value) },
            None => unsafe { std::env::remove_var("NWN2EE_SETTINGS_PATH") },
        }

        result
    }

    #[test]
    fn test_save_character_updates_playerlist_and_player_bic() {
        let player_one = create_named_test_character_fields("Garrick", "Ironheart", 20, 1000);
        let player_two = create_named_test_character_fields("Craven", "Tyrell", 33, 2000);
        let (_temp_dir, save_dir) = create_test_save(
            vec![player_one, player_two],
            create_named_test_character_fields("Garrick", "Ironheart", 20, 1000),
            None,
        );
        let game_data = create_empty_game_data();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");

        session
            .character_mut()
            .expect("Character should be loaded")
            .set_age(42)
            .expect("Failed to update age");
        session
            .save_character(&game_data)
            .expect("Failed to save test character");

        assert_eq!(read_playerlist_entry_ages(&save_dir), vec![42, 33]);
        assert_eq!(read_player_bic_age(&save_dir), 42);

        let mut reloaded = create_session_state();
        reloaded
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to reload saved character");
        assert_eq!(
            reloaded
                .character()
                .expect("Reloaded character should exist")
                .age(),
            42
        );
    }

    #[test]
    fn test_save_character_syncs_primary_localvault_with_canonical_filename() {
        with_temp_documents_folder(|docs_dir| {
            let player_one = create_named_test_character_fields("Garrick", "Ironheart", 20, 1000);
            let player_two = create_named_test_character_fields("Craven", "Tyrell", 33, 2000);
            let (_temp_dir, save_dir) = create_test_save(
                vec![player_one, player_two],
                create_named_test_character_fields("Garrick", "Ironheart", 20, 1000),
                None,
            );
            let game_data = create_empty_game_data();

            let legacy_path = docs_dir.join("localvault").join("Garrick Ironheart.bic");
            std::fs::write(&legacy_path, b"legacy").expect("Failed to seed legacy localvault file");
            let duplicate_one = docs_dir.join("localvault").join("garrickironheart1.bic");
            let duplicate_two = docs_dir.join("localvault").join("garrickironheart2.bic");
            std::fs::write(&duplicate_one, b"stale1")
                .expect("Failed to seed first duplicate localvault file");
            std::fs::write(&duplicate_two, b"stale2")
                .expect("Failed to seed second duplicate localvault file");

            let mut session = create_session_state();
            session
                .load_character(
                    save_dir
                        .to_str()
                        .expect("Test save path should be valid UTF-8"),
                    None,
                )
                .expect("Failed to load test save");

            session
                .character_mut()
                .expect("Character should be loaded")
                .set_age(42)
                .expect("Failed to update age");
            session
                .save_character(&game_data)
                .expect("Failed to save test character");

            let canonical_path = docs_dir.join("localvault").join("garrickironheart.bic");
            assert!(canonical_path.exists(), "canonical localvault BIC should be written");
            assert!(
                !legacy_path.exists(),
                "legacy spaced localvault filename should be removed"
            );
            assert!(
                !duplicate_one.exists(),
                "numbered duplicate localvault variant should be removed from vault root"
            );
            assert!(
                !duplicate_two.exists(),
                "all numbered duplicate localvault variants should be removed from vault root"
            );
            assert!(
                docs_dir
                    .join(".nwn2ee-save-editor-conflicts")
                    .join("localvault")
                    .join("garrickironheart1.bic")
                    .exists(),
                "first duplicate localvault variant should be quarantined"
            );
            assert!(
                docs_dir
                    .join(".nwn2ee-save-editor-conflicts")
                    .join("localvault")
                    .join("garrickironheart2.bic")
                    .exists(),
                "second duplicate localvault variant should be quarantined"
            );

            let localvault_fields = read_gff_root_fields(&canonical_path);
            match localvault_fields.get("Age") {
                Some(GffValue::Int(age)) => assert_eq!(*age, 42),
                other => panic!("Unexpected localvault Age value: {other:?}"),
            }
            assert!(!localvault_fields.contains_key("Mod_IsPrimaryPlr"));
            assert!(!localvault_fields.contains_key("ObjectId"));
            assert!(!localvault_fields.contains_key("Mod_LastModId"));
        });
    }

    #[test]
    fn test_export_to_localvault_strips_save_only_fields() {
        with_temp_documents_folder(|docs_dir| {
            let mut player_fields =
                create_named_test_character_fields("Garrick", "Ironheart", 20, 1000);
            player_fields.insert("Mod_IsPrimaryPlr".to_string(), GffValue::Byte(1));
            player_fields.insert("ObjectId".to_string(), GffValue::Dword(2147483647));
            player_fields.insert(
                "Mod_LastModId".to_string(),
                GffValue::Void(vec![1, 2, 3, 4].into()),
            );
            player_fields.insert("AreaId".to_string(), GffValue::Dword(1250));

            let (_temp_dir, save_dir) =
                create_test_save(vec![player_fields.clone()], player_fields.clone(), None);

            let mut session = create_session_state();
            session
                .load_character(
                    save_dir
                        .to_str()
                        .expect("Test save path should be valid UTF-8"),
                    None,
                )
                .expect("Failed to load test save");

            let exported_path = session
                .export_to_localvault()
                .expect("Failed to export to localvault");
            let exported_path = PathBuf::from(exported_path);

            assert_eq!(
                exported_path,
                docs_dir.join("localvault").join("garrickironheart.bic")
            );

            let localvault_fields = read_gff_root_fields(&exported_path);
            assert!(!localvault_fields.contains_key("Mod_IsPrimaryPlr"));
            assert!(!localvault_fields.contains_key("ObjectId"));
            assert!(!localvault_fields.contains_key("Mod_LastModId"));
            match localvault_fields.get("AreaId") {
                Some(GffValue::Dword(area_id)) => assert_eq!(*area_id, 1250),
                other => panic!("Unexpected AreaId value: {other:?}"),
            }
        });
    }

    #[test]
    fn test_load_character_prefers_player_bic_for_primary_player() {
        let playerlist_entry = create_test_character_fields(20, 1000);
        let player_bic_fields = create_test_character_fields(42, 1000);
        let (_temp_dir, save_dir) =
            create_test_save(vec![playerlist_entry], player_bic_fields, None);

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");

        assert_eq!(
            session
                .character()
                .expect("Character should be loaded")
                .age(),
            42
        );
    }

    #[test]
    fn test_load_character_prefers_player_bic_background_for_primary_player() {
        let mut playerlist_entry = create_test_character_fields_with_classes(20, 1000, &[(3, 1)]);
        playerlist_entry.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![{
                let mut feat = IndexMap::new();
                feat.insert("Feat".to_string(), GffValue::Word(1724));
                feat
            }]),
        );

        let mut player_bic_fields = create_test_character_fields_with_classes(20, 1000, &[(3, 1)]);
        player_bic_fields.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![{
                let mut feat = IndexMap::new();
                feat.insert("Feat".to_string(), GffValue::Word(1717));
                feat
            }]),
        );

        let (_temp_dir, save_dir) =
            create_test_save(vec![playerlist_entry], player_bic_fields, None);
        let game_data = create_game_data_with_classes_and_backgrounds();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");

        let character = session.character().expect("Character should be loaded");
        assert_eq!(character.background(&game_data).as_deref(), Some("Bully"));
        assert!(character.has_feat(crate::character::FeatId(1717)));
        assert!(!character.has_feat(crate::character::FeatId(1724)));
    }

    #[test]
    fn test_load_character_prefers_player_bic_skills_for_primary_player() {
        let mut playerlist_entry = create_test_character_fields_with_classes(20, 1000, &[(3, 1)]);
        playerlist_entry.insert("SkillPoints".to_string(), GffValue::Word(9));
        let mut playerlist_skills = zero_rank_skill_list(5);
        playerlist_skills[0].insert("Rank".to_string(), GffValue::Byte(1));
        playerlist_entry.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(playerlist_skills),
        );

        let mut player_bic_fields = create_test_character_fields_with_classes(20, 1000, &[(3, 1)]);
        player_bic_fields.insert("SkillPoints".to_string(), GffValue::Word(2));
        let mut player_bic_skills = zero_rank_skill_list(5);
        player_bic_skills[0].insert("Rank".to_string(), GffValue::Byte(4));
        player_bic_skills[3].insert("Rank".to_string(), GffValue::Byte(2));
        player_bic_fields.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(player_bic_skills),
        );

        let (_temp_dir, save_dir) =
            create_test_save(vec![playerlist_entry], player_bic_fields, None);

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");

        let character = session.character().expect("Character should be loaded");
        assert_eq!(character.skill_rank(crate::character::SkillId(0)), 4);
        assert_eq!(character.skill_rank(crate::character::SkillId(3)), 2);
        assert_eq!(character.get_available_skill_points(), 2);
    }

    #[test]
    fn test_load_character_defaults_to_matched_primary_multiplayer_slot() {
        let player_one = create_named_test_character_fields("Craven", "Tyrell", 20, 1000);
        let player_two = create_named_test_character_fields("Garrick", "Ironheart", 33, 2000);
        let (_temp_dir, save_dir) = create_test_save(
            vec![player_one, player_two],
            create_named_test_character_fields("Garrick", "Ironheart", 42, 2000),
            None,
        );

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load matched primary player");

        assert_eq!(session.selected_player_index, 1);
        assert_eq!(session.primary_player_index, Some(1));
        assert_eq!(
            session
                .character()
                .expect("Character should be loaded")
                .full_name(),
            "Garrick Ironheart"
        );
        assert_eq!(
            session
                .character()
                .expect("Character should be loaded")
                .age(),
            42
        );
    }

    #[test]
    fn test_load_and_save_character_can_target_second_playerlist_entry() {
        let player_one = create_named_test_character_fields("Craven", "Tyrell", 20, 1000);
        let player_two = create_named_test_character_fields("Garrick", "Ironheart", 33, 2000);
        let (_temp_dir, save_dir) = create_test_save(
            vec![player_one, player_two],
            create_named_test_character_fields("Garrick", "Ironheart", 42, 2000),
            None,
        );
        let game_data = create_empty_game_data();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                Some(1),
            )
            .expect("Failed to load second player from test save");

        assert_eq!(
            session
                .character()
                .expect("Character should be loaded")
                .age(),
            42
        );

        session
            .character_mut()
            .expect("Character should be loaded")
            .set_age(55)
            .expect("Failed to update age");
        session
            .save_character(&game_data)
            .expect("Failed to save selected player");

        assert_eq!(read_playerlist_entry_ages(&save_dir), vec![20, 55]);
        assert_eq!(read_player_bic_age(&save_dir), 55);
    }

    #[test]
    fn test_save_character_does_not_update_player_bic_for_other_multiplayer_player() {
        let player_one = create_named_test_character_fields("Craven", "Tyrell", 20, 1000);
        let player_two = create_named_test_character_fields("Garrick", "Ironheart", 33, 2000);

        let mut playerinfo = PlayerInfoData::new();
        playerinfo.first_name = "Garrick".to_string();
        playerinfo.name = "Garrick Ironheart".to_string();

        let (_temp_dir, save_dir) = create_test_save(
            vec![player_one, player_two],
            create_named_test_character_fields("Garrick", "Ironheart", 33, 2000),
            Some(playerinfo),
        );
        let game_data = create_empty_game_data();
        let original_playerinfo =
            std::fs::read(save_dir.join("playerinfo.bin")).expect("Failed to read playerinfo.bin");

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                Some(0),
            )
            .expect("Failed to load first player from test save");

        assert_eq!(
            session
                .character()
                .expect("Character should be loaded")
                .full_name(),
            "Craven Tyrell"
        );

        session
            .character_mut()
            .expect("Character should be loaded")
            .set_age(55)
            .expect("Failed to update age");
        session
            .save_character(&game_data)
            .expect("Failed to save selected player");

        assert_eq!(read_playerlist_entry_ages(&save_dir), vec![55, 33]);
        assert_eq!(read_player_bic_age(&save_dir), 33);
        assert_eq!(
            std::fs::read(save_dir.join("playerinfo.bin")).expect("Failed to read playerinfo.bin"),
            original_playerinfo
        );
    }

    #[test]
    fn test_sync_primary_mirrors_repairs_playerlist_and_playerinfo_from_player_bic() {
        let player_one = create_named_test_character_fields("Craven", "Tyrell", 20, 1000);
        let player_two = create_named_test_character_fields_with_classes(
            "Garrick",
            "Ironheart",
            33,
            2000,
            &[(0, 4)],
        );
        let mut player_bic = create_named_test_character_fields_with_classes(
            "Garrick",
            "Ironheart",
            42,
            2400,
            &[(0, 5), (3, 1)],
        );
        player_bic.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![{
                let mut feat = IndexMap::new();
                feat.insert("Feat".to_string(), GffValue::Word(1717));
                feat
            }]),
        );
        player_bic.insert("CharBackground".to_string(), GffValue::Dword(8));

        let mut playerinfo = PlayerInfoData::new();
        playerinfo.first_name = "Stale".to_string();
        playerinfo.last_name = "Mirror".to_string();
        playerinfo.name = "Stale Mirror".to_string();
        playerinfo.subrace = "Wrong".to_string();
        playerinfo.alignment = "Wrong".to_string();
        playerinfo.classes = vec![PlayerClassEntry::new("Fighter", 1)];
        playerinfo.deity = "Wrong".to_string();
        playerinfo.unknown4 = 99;

        let (_temp_dir, save_dir) =
            create_test_save(vec![player_one, player_two], player_bic, Some(playerinfo));
        let game_data = create_game_data_with_classes_and_backgrounds();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                Some(0),
            )
            .expect("Failed to load first player from test save");

        session
            .sync_primary_mirrors(&game_data)
            .expect("Failed to synchronize primary mirrors");

        assert_eq!(read_playerlist_entry_ages(&save_dir), vec![20, 42]);
        assert_eq!(read_player_bic_age(&save_dir), 42);
        assert_eq!(read_playerinfo_name(&save_dir), "Garrick Ironheart");
        assert_eq!(
            read_playerinfo_classes(&save_dir),
            vec![
                PlayerClassEntry::new("Barbarian", 5),
                PlayerClassEntry::new("Fighter", 1),
            ]
        );
        assert_eq!(read_playerinfo_background_row(&save_dir), 1);
        assert_eq!(read_player_bic_char_background(&save_dir), Some(1));
        assert_eq!(read_playerlist_entry_char_background(&save_dir, 1), Some(1));
    }

    #[test]
    fn test_save_character_updates_playerinfo_class_summary() {
        let player_fields = create_test_character_fields_with_classes(20, 1000, &[(0, 5), (3, 1)]);
        let mut playerinfo = PlayerInfoData::new();
        playerinfo.first_name = "Garrick".to_string();
        playerinfo.name = "Garrick Ironheart".to_string();
        playerinfo.subrace = "Yuan-ti Pureblood".to_string();
        playerinfo.alignment = "Chaotic Good".to_string();
        playerinfo.classes = vec![
            PlayerClassEntry::new("Barbarian", 5),
            PlayerClassEntry::new("Fighter", 1),
        ];
        playerinfo.deity = "Tempus".to_string();

        let (_temp_dir, save_dir) =
            create_test_save(vec![player_fields.clone()], player_fields, Some(playerinfo));
        let game_data = create_game_data_with_classes();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");

        session
            .character_mut()
            .expect("Character should be loaded")
            .gff_mut()
            .insert(
                "ClassList".to_string(),
                GffValue::ListOwned(vec![
                    {
                        let mut class_entry = IndexMap::new();
                        class_entry.insert("Class".to_string(), GffValue::Byte(0));
                        class_entry.insert("ClassLevel".to_string(), GffValue::Short(5));
                        class_entry
                    },
                    {
                        let mut class_entry = IndexMap::new();
                        class_entry.insert("Class".to_string(), GffValue::Byte(3));
                        class_entry.insert("ClassLevel".to_string(), GffValue::Short(2));
                        class_entry
                    },
                ]),
            );

        session
            .save_character(&game_data)
            .expect("Failed to save updated class data");

        assert_eq!(
            read_playerinfo_classes(&save_dir),
            vec![
                PlayerClassEntry::new("Barbarian", 5),
                PlayerClassEntry::new("Fighter", 2),
            ]
        );
    }

    #[test]
    fn test_save_character_normalizes_class_fields_before_write() {
        let mut player_fields = create_test_character_fields_with_classes(20, 1000, &[(3, 2)]);
        player_fields.insert("Class".to_string(), GffValue::Byte(3));
        player_fields.insert("MClassLevUpIn".to_string(), GffValue::Byte(1));
        player_fields.insert("StartingPackage".to_string(), GffValue::Byte(0));
        player_fields.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(zero_rank_skill_list(30)),
        );

        let mut level_one = IndexMap::new();
        level_one.insert("LvlStatClass".to_string(), GffValue::Byte(3));
        level_one.insert("LvlStatHitDie".to_string(), GffValue::Byte(10));
        level_one.insert("EpicLevel".to_string(), GffValue::Byte(0));
        level_one.insert("SkillPoints".to_string(), GffValue::Short(0));
        level_one.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        level_one.insert(
            "SkillList".to_string(),
            GffValue::ListOwned(zero_rank_skill_list(30)),
        );
        level_one.insert("FeatList".to_string(), GffValue::ListOwned(vec![]));

        let mut level_two = IndexMap::new();
        level_two.insert("LvlStatClass".to_string(), GffValue::Byte(3));
        level_two.insert("LvlStatHitDie".to_string(), GffValue::Byte(13));
        level_two.insert("EpicLevel".to_string(), GffValue::Byte(0));
        level_two.insert("SkillPoints".to_string(), GffValue::Short(1));
        level_two.insert("LvlStatAbility".to_string(), GffValue::Byte(255));
        level_two.insert("SkillList".to_string(), GffValue::ListOwned(vec![]));
        level_two.insert("KnownList0".to_string(), GffValue::ListOwned(vec![]));
        level_two.insert("KnownRemoveList0".to_string(), GffValue::ListOwned(vec![]));

        player_fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![level_one, level_two]),
        );

        let (_temp_dir, save_dir) =
            create_test_save(vec![player_fields.clone()], player_fields, None);
        let game_data = create_game_data_with_classes();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");
        session
            .save_character(&game_data)
            .expect("Failed to save normalized class data");

        let handler = SaveGameHandler::new(&save_dir, false, false)
            .expect("Failed to reopen saved test save");
        let player_bic_data = handler
            .extract_player_bic()
            .expect("Failed to read player.bic")
            .expect("player.bic should exist");
        let parser = GffParser::from_bytes(player_bic_data).expect("Failed to parse player.bic");
        let root = parser
            .read_struct_fields(0)
            .expect("Failed to read player.bic root");

        assert_eq!(
            root.get("MClassLevUpIn")
                .and_then(crate::character::gff_helpers::gff_value_to_i32),
            Some(0)
        );
        assert_eq!(
            root.get("StartingPackage")
                .and_then(crate::character::gff_helpers::gff_value_to_i32),
            Some(3)
        );
        assert!(!root.contains_key("Class"));

        let class_list = match root.get("ClassList") {
            Some(GffValue::List(list)) => list
                .iter()
                .map(|entry| entry.force_load())
                .collect::<Vec<_>>(),
            Some(other) => panic!("Unexpected ClassList value: {other:?}"),
            None => panic!("ClassList missing from player.bic"),
        };
        assert!(matches!(class_list[0].get("Class"), Some(GffValue::Int(3))));
        assert!(matches!(
            class_list[0].get("ClassLevel"),
            Some(GffValue::Short(2))
        ));

        let history = match root.get("LvlStatList") {
            Some(GffValue::List(list)) => list
                .iter()
                .map(|entry| entry.force_load())
                .collect::<Vec<_>>(),
            Some(other) => panic!("Unexpected LvlStatList value: {other:?}"),
            None => panic!("LvlStatList missing from player.bic"),
        };
        assert_eq!(history.len(), 2);
        assert_eq!(
            history[1]
                .get("LvlStatHitDie")
                .and_then(crate::character::gff_helpers::gff_value_to_i32),
            Some(6)
        );
        assert_eq!(
            history[1].get("SkillList").and_then(|value| match value {
                GffValue::List(list) => Some(list.len()),
                GffValue::ListOwned(list) => Some(list.len()),
                _ => None,
            }),
            Some(30)
        );
        assert!(matches!(
            history[1].get("FeatList"),
            Some(GffValue::List(list)) if list.is_empty()
        ));
        assert!(!history[1].contains_key("KnownList0"));
        assert!(!history[1].contains_key("KnownRemoveList0"));
    }

    #[test]
    fn test_save_character_updates_playerinfo_background_row() {
        let mut player_fields = create_test_character_fields_with_classes(20, 1000, &[(3, 1)]);
        player_fields.insert(
            "FeatList".to_string(),
            GffValue::ListOwned(vec![{
                let mut feat = IndexMap::new();
                feat.insert("Feat".to_string(), GffValue::Word(1717));
                feat
            }]),
        );
        player_fields.insert("CharBackground".to_string(), GffValue::Dword(8));

        let mut playerinfo = PlayerInfoData::new();
        playerinfo.first_name = "Garrick".to_string();
        playerinfo.name = "Garrick Ironheart".to_string();
        playerinfo.subrace = "Shield Dwarf".to_string();
        playerinfo.alignment = "Chaotic Good".to_string();
        playerinfo.classes = vec![PlayerClassEntry::new("Fighter", 1)];
        playerinfo.deity = "Tempus".to_string();
        playerinfo.unknown4 = 8;

        let (_temp_dir, save_dir) =
            create_test_save(vec![player_fields.clone()], player_fields, Some(playerinfo));
        let game_data = create_game_data_with_classes_and_backgrounds();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");
        session
            .save_character(&game_data)
            .expect("Failed to save playerinfo background update");

        assert_eq!(read_playerinfo_background_row(&save_dir), 1);
        assert_eq!(read_player_bic_char_background(&save_dir), Some(1));
        assert_eq!(read_playerlist_entry_char_background(&save_dir, 0), Some(1));
    }

    #[test]
    fn test_load_and_save_character_normalize_stale_skill_points() {
        let mut player_fields = create_test_character_fields_with_classes(20, 1000, &[(3, 1)]);
        player_fields.insert("Int".to_string(), GffValue::Byte(10));
        player_fields.insert("SkillPoints".to_string(), GffValue::Word(6));

        let mut skill_list = zero_rank_skill_list(30);
        skill_list[0].insert("Rank".to_string(), GffValue::Byte(2));
        player_fields.insert("SkillList".to_string(), GffValue::ListOwned(skill_list));
        player_fields.insert(
            "LvlStatList".to_string(),
            GffValue::ListOwned(vec![create_history_entry(3, 10, 0, 30)]),
        );

        let (_temp_dir, save_dir) =
            create_test_save(vec![player_fields.clone()], player_fields, None);
        let game_data = create_game_data_with_classes();

        let mut session = create_session_state();
        session
            .load_character(
                save_dir
                    .to_str()
                    .expect("Test save path should be valid UTF-8"),
                None,
            )
            .expect("Failed to load test save");

        session.normalize_loaded_skill_points(&game_data);

        assert_eq!(
            session
                .character()
                .expect("Character should be loaded")
                .get_available_skill_points(),
            4
        );
        assert!(!session.has_unsaved_changes());

        session
            .save_character(&game_data)
            .expect("Failed to save normalized skill points");

        assert_eq!(read_player_bic_skill_points(&save_dir), 4);
    }
}
