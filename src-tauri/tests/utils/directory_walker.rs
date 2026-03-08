use app_lib::config::NWN2Paths;
use app_lib::utils::DirectoryWalker;

fn get_paths() -> NWN2Paths {
    NWN2Paths::new()
}

#[test]
fn test_scan_nwn2_data_directory() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        if data_dir.exists() {
            let mut walker = DirectoryWalker::new();
            let result = walker.index_directory(&data_dir, true);

            assert!(result.is_ok(), "Should scan NWN2 data directory");
            let resources = result.unwrap();

            println!("Found {} resources in NWN2 data directory", resources.len());

            let stats = walker.get_stats();
            println!("Scan stats: {:?}", stats);

            assert!(
                stats.contains_key("last_dir_index_time_ms"),
                "Should track timing"
            );
        } else {
            println!("Data directory not accessible, skipping");
        }
    } else {
        panic!("NWN2 data directory not found - is NWN2 installed?");
    }
}

#[test]
fn test_scan_nwn2_override_directory() {
    let paths = get_paths();

    if let Some(override_dir) = paths.override_dir() {
        if override_dir.exists() {
            let mut walker = DirectoryWalker::new();
            let result = walker.index_directory(&override_dir, true);

            assert!(result.is_ok());
            let resources = result.unwrap();

            println!("Found {} override resources", resources.len());

            for (name, location) in resources.iter().take(10) {
                println!("  {}: {} bytes", name, location.size);
            }
        } else {
            println!("Override directory doesn't exist (user hasn't created overrides)");
        }
    }
}

#[test]
fn test_directory_walker_finds_2da_files_in_data() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        if data_dir.exists() {
            let mut walker = DirectoryWalker::new();
            let result = walker.index_directory(&data_dir, false);

            if let Ok(resources) = result {
                let tda_count = resources.keys().filter(|k| k.ends_with(".2da")).count();

                println!("Found {} .2da files in root data directory", tda_count);
            }
        }
    }
}

#[test]
fn test_directory_walker_resource_locations() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        if data_dir.exists() {
            let mut walker = DirectoryWalker::new();
            let result = walker.index_directory(&data_dir, false);

            if let Ok(resources) = result {
                for (name, location) in resources.iter().take(5) {
                    assert_eq!(location.source_type, "file");
                    assert!(!location.source_path.is_empty());
                    assert!(location.internal_path.is_none());
                    assert!(location.modified_time > 0);

                    println!(
                        "{}: {} bytes, modified {}",
                        name, location.size, location.modified_time
                    );
                }
            }
        }
    }
}

#[test]
fn test_scan_workshop_directory_if_exists() {
    let paths = get_paths();

    if let Some(workshop_dir) = paths.steam_workshop_folder() {
        if workshop_dir.exists() {
            let mut walker = DirectoryWalker::new();
            let result = walker.scan_workshop_directory(workshop_dir);

            assert!(result.is_ok(), "Should scan workshop directory");
            let resources = result.unwrap();

            println!("Found {} resources in Steam Workshop", resources.len());

            let stats = walker.get_stats();
            if let Some(items_scanned) = stats.get("last_workshop_items_scanned") {
                println!("Workshop items scanned: {}", items_scanned);
            }
        } else {
            println!("Steam Workshop directory doesn't exist, skipping");
        }
    } else {
        println!("Steam Workshop path not configured, skipping");
    }
}

#[test]
fn test_stats_reset() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        if data_dir.exists() {
            let mut walker = DirectoryWalker::new();
            let _ = walker.index_directory(&data_dir, false);

            assert!(
                !walker.get_stats().is_empty(),
                "Should have stats after scan"
            );

            walker.reset_stats();

            assert!(walker.get_stats().is_empty(), "Stats should be cleared");
        }
    }
}

#[test]
fn test_hak_directory_scanning() {
    let paths = get_paths();

    if let Some(hak_dir) = paths.hak_dir() {
        if hak_dir.exists() {
            let mut walker = DirectoryWalker::new();
            let result = walker.index_directory(&hak_dir, true);

            assert!(result.is_ok());
            let resources = result.unwrap();

            println!("Found {} resources in HAK directory", resources.len());
        }
    }
}
