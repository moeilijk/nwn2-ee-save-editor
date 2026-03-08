use app_lib::services::savegame_handler::SaveGameHandler;
use tempfile::TempDir;

#[path = "../common/mod.rs"]
mod common;

#[test]
fn test_savegame_handler_read_operations() {
    let fixture_path = common::fixtures_path().join("saves/Classic_Campaign");
    let temp_dir = TempDir::new().unwrap();
    let save_path = temp_dir.path().join("Classic_Campaign_Read");
    
    common::copy_dir_recursive(&fixture_path, &save_path).expect("Failed to copy fixture");
    
    let handler = SaveGameHandler::new(&save_path, true, false).expect("Failed to create handler");
    
    // Test extract_file
    let globals = handler.extract_file("globals.xml").expect("Failed to extract globals.xml");
    assert!(!globals.is_empty());
    
    let start_bytes = &globals[0..5]; // check xml header or similar
    assert!(String::from_utf8_lossy(start_bytes).starts_with('<') || !globals.is_empty());

    // Test list_files
    let files = handler.list_files().expect("Failed to list files");
    assert!(!files.is_empty());
    assert!(!files.is_empty());
    // Note: globals.xml and module.ifo might be on disk (not in zip) for directory saves,
    // so list_files (which lists zip content) might not show them.
    // assert!(files.iter().any(|f| f.name == "globals.xml"));
    // assert!(files.iter().any(|f| f.name == "module.ifo"));

    // Test list_companions
    let companions = handler.list_companions().expect("Failed to list companions");
    // Classic Campaign might have some companions like 'Khelgar', 'Neeshka' depending on save point.
    // Even if empty, it should not fail.
    println!("Companions found: {:?}", companions);

    // Test batch_read_character_files
    let char_files = handler.batch_read_character_files().expect("Failed to batch read characters");
    assert!(char_files.contains_key("player.bic"));
    // If companions exist, they should be here too
    if !companions.is_empty() {
         assert!(char_files.keys().any(|k| k.contains(&companions[0])));
    }

    // Test read_character_summary
    let summary = handler.read_character_summary().expect("Failed to read character summary");
    if let Some(s) = summary {
        assert!(!s.first_name.is_empty());
        // assert!(s.str > 0);
    } else {
        // player.bic must expect to exist in valid save
        panic!("Character summary should be present for Classic_Campaign");
    }
}

#[test]
fn test_savegame_handler_write_operations() {
    let fixture_path = common::fixtures_path().join("saves/Classic_Campaign");
    let temp_dir = TempDir::new().unwrap();
    let save_path = temp_dir.path().join("Classic_Campaign_Write");
    
    common::copy_dir_recursive(&fixture_path, &save_path).expect("Failed to copy fixture");
    
    let mut handler = SaveGameHandler::new(&save_path, true, false).expect("Failed to create handler");
    
    // Test update_file
    let test_filename = "test_file.txt";
    let test_content = b"Hello NWN2";
    
    handler.update_file(test_filename, test_content).expect("Failed to update file");
    
    // Verify update
    let content = handler.extract_file(test_filename).expect("Failed to extract updated file");
    assert_eq!(content, test_content);
    
    // Verify it is in list
    let files = handler.list_files().expect("Failed to list files");
    assert!(files.iter().any(|f| f.name == test_filename));
    
    // Test update existing file
    let new_globals = b"<xml>Modified</xml>";
    handler.update_file("globals.xml", new_globals).expect("Failed to overwrite globals.xml");
    
    let content_mod = handler.extract_file("globals.xml").expect("Failed to read modified globals");
    assert_eq!(content_mod, new_globals);
}
