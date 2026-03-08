use app_lib::services::playerinfo::{PlayerInfo, PlayerInfoData, PlayerClassEntry};
use tempfile::NamedTempFile;

#[test]
fn test_playerinfo_read_write() {
    let mut data = PlayerInfoData::new();
    data.first_name = "Test".to_string();
    data.last_name = "Hero".to_string();
    data.subrace = "Human".to_string();
    data.alignment = "True Neutral".to_string();
    data.deity = "None".to_string();
    data.str_score = 18;
    data.dex_score = 14;
    data.con_score = 12;
    data.int_score = 10;
    data.wis_score = 10;
    data.cha_score = 8;
    
    data.classes.push(PlayerClassEntry::new("Fighter".to_string(), 5));
    data.classes.push(PlayerClassEntry::new("Wizard".to_string(), 3));
    
    // Write to a temp file using public save API
    // PlayerInfo needs a path.
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    
    let info = PlayerInfo {
        file_path: Some(path.clone()),
        data: data.clone(),
    };
    
    info.save(&path).expect("Failed to save playerinfo");
    
    // Read back
    let loaded = PlayerInfo::load(&path).expect("Failed to load playerinfo");
    
    assert_eq!(loaded.data.first_name, "Test");
    assert_eq!(loaded.data.last_name, "Hero");
    assert_eq!(loaded.data.classes.len(), 2);
    assert_eq!(loaded.data.classes[0].name, "Fighter");
    assert_eq!(loaded.data.classes[0].level, 5);
    assert_eq!(loaded.data.classes[1].name, "Wizard");
    assert_eq!(loaded.data.classes[1].level, 3);
    
    // Verify scores
    assert_eq!(loaded.data.str_score, 18);
    assert_eq!(loaded.data.cha_score, 8);
}

#[test]
fn test_playerinfo_update_from_gff() {
    use app_lib::parsers::gff::GffValue;
    use indexmap::IndexMap;
    
    let mut fields = IndexMap::new();
    fields.insert("FirstName".to_string(), GffValue::String("UpdatedName".into()));
    fields.insert("LastName".to_string(), GffValue::String("UpdatedLast".into()));
    fields.insert("Str".to_string(), GffValue::Byte(20));
    
    let mut info = PlayerInfo::new();
    info.data.first_name = "Old".to_string();
    info.data.str_score = 10;
    
    let classes = vec![("Rogue".to_string(), 1u8)];
    
    info.update_from_gff_data(&fields, "Elf", "Chaotic Good", &classes);
    
    assert_eq!(info.data.first_name, "UpdatedName");
    assert_eq!(info.data.last_name, "UpdatedLast");
    // Name should be updated combination
    assert_eq!(info.data.display_name(), "UpdatedName UpdatedLast");
    assert_eq!(info.data.str_score, 20);
    assert_eq!(info.data.subrace, "Elf");
    assert_eq!(info.data.alignment, "Chaotic Good");
    assert_eq!(info.data.classes.len(), 1);
    assert_eq!(info.data.classes[0].name, "Rogue");
}
