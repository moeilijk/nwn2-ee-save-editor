use app_lib::config::NWN2Paths;
use app_lib::utils::ResourceScanner;

fn get_paths() -> NWN2Paths {
    NWN2Paths::new()
}

#[test]
fn test_scan_nwn2_data_zips() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let mut zip_paths = Vec::new();

        for zip_name in ["2da.zip", "2da_x1.zip", "2da_x2.zip"] {
            let zip_path = data_dir.join(zip_name);
            if zip_path.exists() {
                zip_paths.push(zip_path.to_string_lossy().to_string());
            }
        }

        assert!(!zip_paths.is_empty(), "Should have at least one data zip");

        let mut scanner = ResourceScanner::new();
        let result = scanner.scan_zip_files(zip_paths);

        assert!(result.is_ok());
        let resources = result.unwrap();

        println!("Scanned {} resources from data zips", resources.len());

        assert!(
            resources.contains_key("classes.2da"),
            "Should find classes.2da"
        );
        assert!(resources.contains_key("feat.2da"), "Should find feat.2da");
        assert!(
            resources.contains_key("skills.2da"),
            "Should find skills.2da"
        );
    }
}

#[test]
fn test_scan_workshop_directories() {
    let paths = get_paths();

    if let Some(workshop_dir) = paths.steam_workshop_folder() {
        if workshop_dir.exists() {
            let mut scanner = ResourceScanner::new();
            let result =
                scanner.scan_workshop_directories(vec![workshop_dir.to_string_lossy().to_string()]);

            assert!(result.is_ok());
            let resources = result.unwrap();

            println!("Found {} workshop resources", resources.len());

            for (name, location) in resources.iter().take(10) {
                println!("  {}: {}", name, location.source_path);
            }
        } else {
            println!("Steam Workshop not installed, skipping");
        }
    } else {
        println!("Steam Workshop path not configured, skipping");
    }
}

#[test]
fn test_index_override_directory() {
    let paths = get_paths();

    if let Some(override_dir) = paths.override_dir() {
        let mut scanner = ResourceScanner::new();
        let result =
            scanner.index_directory(override_dir.to_string_lossy().to_string(), Some(true));

        assert!(result.is_ok());
        let resources = result.unwrap();

        println!("Found {} override resources", resources.len());

        let stats = scanner.get_performance_stats();
        println!("Performance stats: {:?}", stats);
    }
}

#[test]
fn test_comprehensive_scan() {
    let paths = get_paths();

    let data_dir = paths
        .data()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let workshop_dirs: Vec<String> = paths
        .steam_workshop_folder()
        .filter(|p| p.exists())
        .map(|p| vec![p.to_string_lossy().to_string()])
        .unwrap_or_default();

    let override_dirs: Vec<String> = paths
        .override_dir()
        .filter(|p| p.exists())
        .map(|p| vec![p.to_string_lossy().to_string()])
        .unwrap_or_default();

    let enhanced_dir = paths
        .enhanced_data()
        .map(|p| p.to_string_lossy().to_string());

    let mut scanner = ResourceScanner::new();
    let result = scanner.comprehensive_scan(data_dir, workshop_dirs, override_dirs, enhanced_dir);

    assert!(result.is_ok());
    let scan_results = result.unwrap();

    println!("=== Comprehensive Scan Results ===");
    println!("Scan time: {}ms", scan_results.scan_time_ms);
    println!("Resources found: {}", scan_results.resources_found);
    println!("ZIP files scanned: {}", scan_results.zip_files_scanned);
    println!("Directories scanned: {}", scan_results.directories_scanned);
    println!("Workshop items: {}", scan_results.workshop_items_found);

    assert!(scan_results.resources_found > 0, "Should find resources");
    assert!(scan_results.zip_files_scanned > 0, "Should scan zip files");
}

#[test]
fn test_resource_location_types() {
    let paths = get_paths();

    let data_dir = paths
        .data()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut scanner = ResourceScanner::new();
    let result = scanner.comprehensive_scan(data_dir, vec![], vec![], None);

    if let Ok(scan_results) = result {
        let zip_resources = scan_results
            .resource_locations
            .values()
            .filter(|loc| loc.source_type == "zip")
            .count();

        let file_resources = scan_results
            .resource_locations
            .values()
            .filter(|loc| loc.source_type == "file")
            .count();

        println!("Resources from ZIP: {}", zip_resources);
        println!("Resources from files: {}", file_resources);

        assert!(zip_resources > 0, "Should have ZIP resources");
    }
}

#[test]
fn test_performance_stats_populated() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let mut scanner = ResourceScanner::new();
            let _ = scanner.scan_zip_files(vec![zip_path.to_string_lossy().to_string()]);

            let stats = scanner.get_performance_stats();

            assert!(!stats.is_empty(), "Should have performance stats");

            println!("Performance stats:");
            for (key, value) in &stats {
                println!("  {}: {}", key, value);
            }
        }
    }
}

#[test]
fn test_scan_finds_core_2da_tables() {
    let paths = get_paths();

    let data_dir = paths
        .data()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut scanner = ResourceScanner::new();
    let result = scanner.comprehensive_scan(data_dir, vec![], vec![], None);

    if let Ok(scan_results) = result {
        let core_tables = [
            "classes.2da",
            "feat.2da",
            "skills.2da",
            "spells.2da",
            "racialtypes.2da",
            "baseitems.2da",
            "appearance.2da",
        ];

        println!("Checking core 2DA tables:");
        for table in core_tables {
            let found = scan_results.resource_locations.contains_key(table);
            println!("  {}: {}", table, if found { "FOUND" } else { "MISSING" });
            assert!(found, "{} should be present", table);
        }
    }
}

#[test]
fn test_enhanced_edition_scanning() {
    let paths = get_paths();

    if let Some(enhanced_data) = paths.enhanced_data() {
        if enhanced_data.exists() {
            let data_dir = paths
                .data()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let mut scanner = ResourceScanner::new();
            let result = scanner.comprehensive_scan(
                data_dir,
                vec![],
                vec![],
                Some(enhanced_data.to_string_lossy().to_string()),
            );

            assert!(result.is_ok());
            let scan_results = result.unwrap();

            println!(
                "Enhanced Edition scan found {} resources",
                scan_results.resources_found
            );
        } else {
            println!("Enhanced data directory doesn't exist, skipping");
        }
    } else {
        println!("Not Enhanced Edition, skipping");
    }
}
