use app_lib::services::playerinfo::{PlayerClassEntry, PlayerInfo, PlayerInfoData};
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

    data.classes
        .push(PlayerClassEntry::new("Fighter".to_string(), 5));
    data.classes
        .push(PlayerClassEntry::new("Wizard".to_string(), 3));

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
    fields.insert(
        "FirstName".to_string(),
        GffValue::String("UpdatedName".into()),
    );
    fields.insert(
        "LastName".to_string(),
        GffValue::String("UpdatedLast".into()),
    );
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

/// Parse known fixtures and confirm the decoded fields match Arbos' spec:
/// <https://gist.github.com/Arbos/225c724f91309d3f515e0f110524feee>
#[test]
fn test_playerinfo_fixtures_decode_alignment_and_background() {
    let fixtures_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/saves");

    // (relative path, alignment text, vertical, horizontal, background_id)
    let cases = [
        (
            "Classic_Campaign/playerinfo.bin",
            "Chaotic Neutral",
            1,
            3,
            3,
        ),
        ("STORM_Campaign/playerinfo.bin", "Chaotic Evil", 5, 3, 7),
        ("MOTB/playerinfo.bin", "Neutral Good", 4, 1, 9),
        ("Westgate_Campaign/playerinfo.bin", "Neutral Evil", 5, 1, 10),
    ];

    for (rel, alignment, vert, horiz, bg) in cases {
        let path = fixtures_dir.join(rel);
        let info = PlayerInfo::load(&path).unwrap_or_else(|e| panic!("load {rel}: {e}"));

        assert_eq!(info.data.alignment, alignment, "{rel}: alignment text");
        assert_eq!(info.data.alignment_vertical, vert, "{rel}: vertical");
        assert_eq!(info.data.alignment_horizontal, horiz, "{rel}: horizontal");
        assert_eq!(info.data.background_id, bg, "{rel}: background_id");
    }
}

/// Re-writing a parsed fixture must produce byte-identical output.
#[test]
fn test_playerinfo_round_trip_preserves_bytes() {
    let fixtures_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/saves");

    let fixtures = [
        "Classic_Campaign/playerinfo.bin",
        "MOTB/playerinfo.bin",
        "STORM_Campaign/playerinfo.bin",
        "Westgate_Campaign/playerinfo.bin",
        "cheatdebug/000061 - 16-04-2026-23-01/playerinfo.bin",
        "classdebug/000058 - 15-04-2026-16-56/playerinfo.bin",
    ];

    for rel in fixtures {
        let src = fixtures_dir.join(rel);
        let original = std::fs::read(&src).unwrap_or_else(|e| panic!("read {rel}: {e}"));

        let info = PlayerInfo::load(&src).unwrap_or_else(|e| panic!("parse {rel}: {e}"));

        let tmp = NamedTempFile::new().unwrap();
        info.save(tmp.path()).unwrap();
        let rewritten = std::fs::read(tmp.path()).unwrap();

        assert_eq!(
            original, rewritten,
            "{rel}: parse+write must preserve bytes exactly"
        );
    }
}

/// `update_from_gff_data` must pull CharBackground + LawfulChaotic + GoodEvil from the GFF
/// and recompute ability modifiers — these drive the load menu.
#[test]
fn test_playerinfo_update_from_gff_populates_background_alignment_mods() {
    use app_lib::parsers::gff::GffValue;
    use indexmap::IndexMap;

    let mut fields = IndexMap::new();
    fields.insert("FirstName".to_string(), GffValue::String("Hero".into()));
    fields.insert("Str".to_string(), GffValue::Byte(18));
    fields.insert("Dex".to_string(), GffValue::Byte(14));
    fields.insert("Con".to_string(), GffValue::Byte(13));
    fields.insert("Int".to_string(), GffValue::Byte(10));
    fields.insert("Wis".to_string(), GffValue::Byte(8));
    fields.insert("Cha".to_string(), GffValue::Byte(20));
    // Chaotic Good: law_chaos low -> Chaotic=3, good_evil high -> Good=4
    fields.insert("LawfulChaotic".to_string(), GffValue::Byte(10));
    fields.insert("GoodEvil".to_string(), GffValue::Byte(90));
    // CharBackground is Dword on the GFF root
    fields.insert("CharBackground".to_string(), GffValue::Dword(7));

    let mut info = PlayerInfo::new();
    info.update_from_gff_data(&fields, "Human", "Chaotic Good", &[]);

    assert_eq!(info.data.background_id, 7);
    assert_eq!(info.data.alignment_vertical, 4, "Good");
    assert_eq!(info.data.alignment_horizontal, 3, "Chaotic");

    // floor((score - 10) / 2)
    assert_eq!(info.data.str_mod, 4);
    assert_eq!(info.data.dex_mod, 2);
    assert_eq!(info.data.con_mod, 1);
    assert_eq!(info.data.int_mod, 0);
    assert_eq!(info.data.wis_mod, -1);
    assert_eq!(info.data.cha_mod, 5);
}
