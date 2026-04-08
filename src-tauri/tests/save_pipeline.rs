use std::path::PathBuf;

use indexmap::IndexMap;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gff")
        .join("sync")
}

fn parse_playerlist_ifo(
    data: Vec<u8>,
) -> IndexMap<String, app_lib::parsers::gff::GffValue<'static>> {
    use app_lib::parsers::gff::{GffParser, GffValue};

    let gff = GffParser::from_bytes(data).expect("Failed to parse IFO");
    let root = gff.read_struct_fields(0).expect("Failed to read root");

    let mod_player_list = root.get("Mod_PlayerList").expect("No Mod_PlayerList");
    if let GffValue::List(lazy_structs) = mod_player_list {
        let first = lazy_structs.first().expect("Mod_PlayerList is empty");
        let fields = first.force_load();
        fields
            .into_iter()
            .map(|(k, v)| (k, v.force_owned()))
            .collect()
    } else {
        panic!("Mod_PlayerList is not a list");
    }
}

#[test]
fn test_ifo_round_trip() {
    use app_lib::parsers::gff::{GffValue, GffWriter};

    let ifo_data = std::fs::read(fixtures_dir().join("playerlist.ifo")).expect("read ifo fixture");
    let original_fields = parse_playerlist_ifo(ifo_data);

    // Build IFO bytes from fields
    let mut root = IndexMap::new();
    root.insert(
        "Mod_PlayerList".to_string(),
        GffValue::ListOwned(vec![original_fields.clone()]),
    );
    let mut writer = GffWriter::new("IFO ", "V3.2");
    let ifo_bytes = writer.write(root).expect("Failed to write IFO");

    // Re-parse and compare key fields
    let reparsed = parse_playerlist_ifo(ifo_bytes);

    assert_eq!(
        original_fields.get("FirstName").map(|v| format!("{v:?}")),
        reparsed.get("FirstName").map(|v| format!("{v:?}")),
        "FirstName mismatch after round-trip"
    );
    assert_eq!(
        original_fields.get("Str").map(|v| format!("{v:?}")),
        reparsed.get("Str").map(|v| format!("{v:?}")),
        "Str mismatch after round-trip"
    );
    assert_eq!(
        original_fields.get("Race").map(|v| format!("{v:?}")),
        reparsed.get("Race").map(|v| format!("{v:?}")),
        "Race mismatch after round-trip"
    );

    let orig_count = original_fields
        .keys()
        .filter(|k| !k.starts_with("__"))
        .count();
    let reparse_count = reparsed.keys().filter(|k| !k.starts_with("__")).count();
    assert_eq!(
        orig_count, reparse_count,
        "Field count mismatch after round-trip"
    );
}

#[test]
fn test_bic_sync_preserves_bic_only_fields_and_updates_matching() {
    use app_lib::parsers::gff::{GffParser, GffValue, GffWriter};

    let bic_data = std::fs::read(fixtures_dir().join("player.bic")).expect("read bic fixture");
    let ifo_data = std::fs::read(fixtures_dir().join("playerlist.ifo")).expect("read ifo fixture");

    let bic_gff = GffParser::from_bytes(bic_data).expect("parse bic");
    let bic_fields = bic_gff.read_struct_fields(0).expect("read bic fields");
    let root_struct_id = bic_gff.get_struct_id(0).expect("get root struct_id");

    let char_fields = parse_playerlist_ifo(ifo_data);

    // Collect BIC-only keys before merge
    let bic_only_keys: Vec<String> = bic_fields
        .keys()
        .filter(|k| !k.starts_with("__") && !char_fields.contains_key(*k))
        .cloned()
        .collect();

    // Merge: overwrite matching fields from char_fields
    let mut merged: IndexMap<String, GffValue<'static>> = bic_fields
        .into_iter()
        .map(|(k, v)| (k, v.force_owned()))
        .collect();

    for (key, value) in &char_fields {
        if key.starts_with("__") {
            continue;
        }
        if merged.contains_key(key) {
            merged.insert(key.clone(), value.clone());
        }
    }

    // BIC-only fields should still be present
    for key in &bic_only_keys {
        assert!(
            merged.contains_key(key),
            "BIC-only field '{key}' was lost during merge"
        );
    }

    // Write and re-parse to verify serialization
    let mut writer = GffWriter::new("BIC ", "V3.2");
    let bic_bytes = writer
        .write_with_struct_id(merged, root_struct_id)
        .expect("Failed to write merged BIC");

    let reparsed_gff = GffParser::from_bytes(bic_bytes).expect("Failed to re-parse BIC");
    let reparsed = reparsed_gff
        .read_struct_fields(0)
        .expect("read reparsed bic");

    for key in &bic_only_keys {
        assert!(
            reparsed.contains_key(key),
            "BIC-only field '{key}' was lost after serialization"
        );
    }
}

#[test]
fn test_playerinfo_bin_sync() {
    use app_lib::parsers::gff::{GffValue, LocalizedString, LocalizedSubstring};
    use app_lib::services::playerinfo::PlayerInfo;

    let fixture_path = fixtures_dir().join("playerinfo.bin");

    // Load original to capture unknown fields
    let original = PlayerInfo::load(&fixture_path).expect("load original playerinfo.bin");
    let original_unknown2 = original.data.unknown2;
    let original_unknown3 = original.data.unknown3;
    let original_unknown4 = original.data.unknown4;
    let original_unknown5 = original.data.unknown5;
    let original_unknown6 = original.data.unknown6;
    let original_unknown7 = original.data.unknown7;
    let original_unknown8 = original.data.unknown8;
    let original_unknown9 = original.data.unknown9;
    let original_unknown10 = original.data.unknown10;

    // Copy fixture to temp file for modification
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let temp_path = temp_dir.path().join("playerinfo.bin");
    std::fs::copy(&fixture_path, &temp_path).expect("copy fixture");

    // Load and update with test data
    let mut player_info = PlayerInfo::load(&temp_path).expect("load temp playerinfo.bin");

    let mut fields = IndexMap::new();
    fields.insert(
        "FirstName".to_string(),
        GffValue::LocString(LocalizedString {
            string_ref: -1,
            substrings: vec![LocalizedSubstring {
                language: 0,
                gender: 0,
                string: "TestFirst".into(),
            }],
        }),
    );
    fields.insert(
        "LastName".to_string(),
        GffValue::LocString(LocalizedString {
            string_ref: -1,
            substrings: vec![LocalizedSubstring {
                language: 0,
                gender: 0,
                string: "TestLast".into(),
            }],
        }),
    );
    fields.insert("Deity".to_string(), GffValue::String("Tyr".into()));
    fields.insert("Str".to_string(), GffValue::Byte(18));
    fields.insert("Dex".to_string(), GffValue::Byte(14));
    fields.insert("Con".to_string(), GffValue::Byte(16));
    fields.insert("Int".to_string(), GffValue::Byte(12));
    fields.insert("Wis".to_string(), GffValue::Byte(10));
    fields.insert("Cha".to_string(), GffValue::Byte(8));

    player_info.update_from_gff_data(
        &fields,
        "Tiefling",
        "Chaotic Good",
        &[("Fighter".to_string(), 10), ("Rogue".to_string(), 5)],
    );

    player_info
        .save(&temp_path)
        .expect("save updated playerinfo.bin");

    // Reload and verify
    let reloaded = PlayerInfo::load(&temp_path).expect("reload playerinfo.bin");

    assert_eq!(reloaded.data.first_name, "TestFirst");
    assert_eq!(reloaded.data.last_name, "TestLast");
    assert_eq!(reloaded.data.name, "TestFirst TestLast");
    assert_eq!(reloaded.data.deity, "Tyr");
    assert_eq!(reloaded.data.subrace, "Tiefling");
    assert_eq!(reloaded.data.alignment, "Chaotic Good");
    assert_eq!(reloaded.data.str_score, 18);
    assert_eq!(reloaded.data.dex_score, 14);
    assert_eq!(reloaded.data.con_score, 16);
    assert_eq!(reloaded.data.int_score, 12);
    assert_eq!(reloaded.data.wis_score, 10);
    assert_eq!(reloaded.data.cha_score, 8);
    assert_eq!(reloaded.data.classes.len(), 2);
    assert_eq!(reloaded.data.classes[0].name, "Fighter");
    assert_eq!(reloaded.data.classes[0].level, 10);
    assert_eq!(reloaded.data.classes[1].name, "Rogue");
    assert_eq!(reloaded.data.classes[1].level, 5);

    // Unknown fields must be preserved exactly
    assert_eq!(
        reloaded.data.unknown2, original_unknown2,
        "unknown2 changed"
    );
    assert_eq!(
        reloaded.data.unknown3, original_unknown3,
        "unknown3 changed"
    );
    assert_eq!(
        reloaded.data.unknown4, original_unknown4,
        "unknown4 changed"
    );
    assert_eq!(
        reloaded.data.unknown5, original_unknown5,
        "unknown5 changed"
    );
    assert_eq!(
        reloaded.data.unknown6, original_unknown6,
        "unknown6 changed"
    );
    assert_eq!(
        reloaded.data.unknown7, original_unknown7,
        "unknown7 changed"
    );
    assert_eq!(
        reloaded.data.unknown8, original_unknown8,
        "unknown8 changed"
    );
    assert_eq!(
        reloaded.data.unknown9, original_unknown9,
        "unknown9 changed"
    );
    assert_eq!(
        reloaded.data.unknown10, original_unknown10,
        "unknown10 changed"
    );
}
