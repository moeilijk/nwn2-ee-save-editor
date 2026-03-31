use indexmap::IndexMap;
use std::path::PathBuf;
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
            item_property_decoder,
            feat_cache: None,
        }
    }

    #[instrument(name = "SessionState::load_character", skip(self), fields(file_path = %file_path))]
    pub fn load_character(&mut self, file_path: &str, player_index: Option<usize>) -> Result<(), String> {
        info!("Loading character from save file");
        let path = PathBuf::from(file_path);
        let selected_player_index = player_index.unwrap_or(0);

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

        let fields = read_playerlist_entry(gff, selected_player_index)?;
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
            character.normalize_skill_points(game_data);
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
        let player_bic_bytes = serialize_player_bic_bytes(player_bic_data, &char_fields)?;

        self.savegame_handler
            .as_mut()
            .unwrap()
            .update_player_complete(&playerlist_bytes, &player_bic_bytes, None, None)
            .map_err(|e| format!("Failed to write save file: {e}"))?;

        self.write_playerinfo(game_data)?;

        // Clear modified flag
        self.character.as_mut().unwrap().mark_saved();

        info!(
            "Character saved successfully (playerlist={} bytes, player.bic={} bytes)",
            playerlist_bytes.len(),
            player_bic_bytes.len()
        );
        Ok(())
    }

    pub fn close_character(&mut self) {
        self.character = None;
        self.savegame_handler = None;
        self.current_file_path = None;
        self.selected_player_index = 0;
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

    #[instrument(name = "SessionState::export_to_localvault", skip(self))]
    pub fn export_to_localvault(&self) -> Result<String, String> {
        let handler = self
            .savegame_handler
            .as_ref()
            .ok_or("No active save handler")?;
        let character = self.character.as_ref().ok_or("No character loaded")?;

        let nwn2_paths = crate::config::nwn2_paths::NWN2Paths::new();
        let vault_path = nwn2_paths
            .localvault()
            .ok_or("Could not determine NWN2 localvault path")?;

        if !vault_path.exists() {
            std::fs::create_dir_all(&vault_path)
                .map_err(|e| format!("Failed to create localvault directory: {e}"))?;
        }

        let player_bic_data = handler
            .extract_player_bic()
            .map_err(|e| format!("Failed to extract player.bic: {e}"))?;
        let current_character_fields = character.clone_gff();
        let player_bic_data =
            serialize_player_bic_bytes(player_bic_data, &current_character_fields)?;

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

        Ok(lazy_structs.iter().map(|entry| entry.force_load()).collect())
    } else {
        warn!("Mod_PlayerList is not a list");
        Err("Mod_PlayerList is not a list".to_string())
    }
}

pub(crate) fn read_playerlist_entry(
    gff: Arc<GffParser>,
    player_index: usize,
) -> Result<IndexMap<String, GffValue<'static>>, String> {
    let entries = read_playerlist_entries(gff)?;
    entries.get(player_index).cloned().ok_or_else(|| {
        format!(
            "Selected player index {player_index} is out of range for Mod_PlayerList with {} entries",
            entries.len()
        )
    })
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
    use std::fs::File;
    use std::path::Path;
    use std::sync::Arc as StdArc;
    use tempfile::TempDir;
    use zip::CompressionMethod;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use crate::config::NWN2Paths;
    use crate::loaders::{GameData, LoadedTable};
    use crate::parsers::gff::GffValue;
    use crate::parsers::tda::TDAParser;
    use crate::parsers::tlk::TLKParser;
    use crate::services::resource_manager::ResourceManager;
    use crate::services::{PlayerClassEntry, PlayerInfoData};

    fn create_test_character_fields(age: i32, xp: u32) -> IndexMap<String, GffValue<'static>> {
        let mut fields = IndexMap::new();
        fields.insert("Age".to_string(), GffValue::Int(age));
        fields.insert("Experience".to_string(), GffValue::Dword(xp));
        fields.insert("ClassList".to_string(), GffValue::ListOwned(vec![]));
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

    #[test]
    fn test_save_character_updates_playerlist_and_player_bic() {
        let player_one = create_test_character_fields(20, 1000);
        let player_two = create_test_character_fields(33, 2000);
        let (_temp_dir, save_dir) = create_test_save(
            vec![player_one, player_two],
            create_test_character_fields(20, 1000),
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
    fn test_load_and_save_character_can_target_second_playerlist_entry() {
        let player_one = create_test_character_fields(20, 1000);
        let player_two = create_test_character_fields(33, 2000);
        let (_temp_dir, save_dir) = create_test_save(
            vec![player_one, player_two],
            create_test_character_fields(20, 1000),
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
            33
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
        assert!(matches!(
            class_list[0].get("Class"),
            Some(GffValue::Int(3))
        ));
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
