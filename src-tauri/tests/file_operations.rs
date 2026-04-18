use app_lib::file_operations::browse_backups;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;

#[tokio::test]
async fn browse_backups_root_populates_character_and_save_name_from_newest_inner_backup() {
    let temp = TempDir::new().expect("tempdir");
    let backups_root = temp.path().join("backups");
    let save_folder = backups_root.join("000001 - Save");
    let older = save_folder.join("backup_20260101_000000");
    let newer = save_folder.join("backup_20260401_120000");

    fs::create_dir_all(&older).expect("create older");
    // Ensure the newer directory is created with a strictly later mtime/ctime.
    thread::sleep(Duration::from_millis(20));
    fs::create_dir_all(&newer).expect("create newer");

    // Older backup has a different savename and no playerinfo.bin so we can detect
    // the wrong one being picked up.
    fs::write(older.join("savename.txt"), "Old Save").expect("write old savename");

    // Copy a real playerinfo.bin from fixtures so PlayerInfo::get_player_name parses cleanly.
    let fixture_player_info = common::fixtures_path().join("saves/Classic_Campaign/playerinfo.bin");
    fs::copy(&fixture_player_info, newer.join("playerinfo.bin"))
        .expect("copy playerinfo.bin fixture");
    fs::write(newer.join("savename.txt"), "My Newest Save").expect("write new savename");
    // Add one small file so size > 0 is reported.
    fs::write(newer.join("player.bic"), b"dummy bytes").expect("write player.bic");

    let result = browse_backups(backups_root.to_string_lossy().to_string())
        .await
        .expect("browse_backups ok");

    assert_eq!(result.len(), 1, "expected one save folder at root");
    let entry = &result[0];
    assert_eq!(entry.name, "000001 - Save");
    assert_eq!(
        entry.save_name.as_deref(),
        Some("My Newest Save"),
        "save_name should come from the newer backup, not the older one"
    );
    assert!(
        entry.character_name.is_some(),
        "character_name should be read from the newer backup's playerinfo.bin"
    );
    assert!(
        entry.size > 0,
        "size should reflect files in the newer backup"
    );
}
