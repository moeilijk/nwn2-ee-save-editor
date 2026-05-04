use app_lib::config::NWN2Paths;
use app_lib::utils::ZipIndexer;

fn get_paths() -> NWN2Paths {
    NWN2Paths::new()
}

#[test]
fn test_index_nwn2_base_zip() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let mut indexer = ZipIndexer::new();
            let result = indexer.index_zip(&zip_path);

            assert!(result.is_ok(), "Should index 2da.zip");
            let resources = result.unwrap();

            println!("Found {} resources in 2da.zip", resources.len());
            assert!(resources.len() > 50, "2da.zip should have many 2DA files");

            assert!(
                resources.contains_key("classes.2da"),
                "Should find classes.2da"
            );
            assert!(resources.contains_key("feat.2da"), "Should find feat.2da");
            assert!(
                resources.contains_key("skills.2da"),
                "Should find skills.2da"
            );

            let stats = indexer.get_stats();
            println!(
                "Index time: {}ms",
                stats.get("last_zip_index_time_ms").unwrap_or(&0)
            );
            println!(
                "Files found: {}",
                stats.get("last_zip_2da_files_found").unwrap_or(&0)
            );
        } else {
            panic!("2da.zip not found at {:?}", zip_path);
        }
    } else {
        panic!("NWN2 data directory not found");
    }
}

#[test]
fn test_index_expansion_zips() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let mut indexer = ZipIndexer::new();
        let mut total_resources = 0;

        for zip_name in ["2da.zip", "2da_x1.zip", "2da_x2.zip"] {
            let zip_path = data_dir.join(zip_name);

            if zip_path.exists() {
                let result = indexer.index_zip(&zip_path);
                if let Ok(resources) = result {
                    println!("{}: {} resources", zip_name, resources.len());
                    total_resources += resources.len();
                }
            }
        }

        println!("Total resources across all data zips: {}", total_resources);

        let stats = indexer.get_stats();
        assert!(stats.get("total_zips_indexed").unwrap_or(&0) > &0);
    }
}

#[test]
fn test_resource_location_from_zip() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let mut indexer = ZipIndexer::new();
            let resources = indexer.index_zip(&zip_path).unwrap();

            if let Some(location) = resources.get("classes.2da") {
                assert_eq!(location.source_type, "zip");
                assert!(location.source_path.contains("2da.zip"));
                assert!(location.internal_path.is_some());
                assert!(location.size > 0);
                assert!(location.modified_time > 0);

                println!("classes.2da location:");
                println!("  Source: {}", location.source_path);
                println!("  Internal path: {:?}", location.internal_path);
                println!("  Size: {} bytes", location.size);
            }
        }
    }
}

#[test]
fn test_parallel_zip_indexing() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let mut zip_paths = Vec::new();

        for zip_name in ["2da.zip", "2da_x1.zip", "2da_x2.zip"] {
            let zip_path = data_dir.join(zip_name);
            if zip_path.exists() {
                zip_paths.push(zip_path);
            }
        }

        if zip_paths.len() > 1 {
            let mut indexer = ZipIndexer::new();
            let path_refs: Vec<_> = zip_paths.iter().map(|p| p.as_path()).collect();

            let result = indexer.index_zips_parallel(path_refs);

            assert!(result.is_ok(), "Parallel indexing should succeed");
            let resources = result.unwrap();

            println!("Parallel indexed {} total resources", resources.len());

            let stats = indexer.get_stats();
            println!(
                "Parallel index time: {}ms",
                stats.get("last_parallel_zip_time_ms").unwrap_or(&0)
            );
            println!(
                "Zips processed: {}",
                stats.get("last_parallel_zip_count").unwrap_or(&0)
            );
        }
    }
}

#[test]
fn test_cumulative_stats_tracking() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let mut indexer = ZipIndexer::new();

        for zip_name in ["2da.zip", "2da_x1.zip"] {
            let zip_path = data_dir.join(zip_name);
            if zip_path.exists() {
                let _ = indexer.index_zip(&zip_path);
            }
        }

        let stats = indexer.get_stats();
        let total_indexed = *stats.get("total_zips_indexed").unwrap_or(&0);
        let total_2das = *stats.get("total_2da_files_indexed").unwrap_or(&0);

        println!("Total zips indexed: {}", total_indexed);
        println!("Total 2DA files indexed: {}", total_2das);

        assert!(total_indexed > 0, "Should track cumulative zips");
        assert!(total_2das > 0, "Should track cumulative files");
    }
}

#[test]
fn test_index_enhanced_data_zip() {
    let paths = get_paths();

    if let Some(enhanced_data) = paths.enhanced_data() {
        let zip_path = enhanced_data.join("2da.zip");

        if zip_path.exists() {
            let mut indexer = ZipIndexer::new();
            let result = indexer.index_zip(&zip_path);

            assert!(result.is_ok());
            let resources = result.unwrap();

            println!("Found {} resources in enhanced 2da.zip", resources.len());
        } else {
            println!("Enhanced 2da.zip not found, skipping");
        }
    } else {
        println!("Enhanced Edition not installed, skipping");
    }
}
