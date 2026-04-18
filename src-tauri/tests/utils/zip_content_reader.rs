use app_lib::config::NWN2Paths;
use app_lib::utils::zip_scanner;
use app_lib::utils::{ZipContentReader, ZipReadRequest};

fn get_paths() -> NWN2Paths {
    NWN2Paths::new()
}

fn get_internal_path_for_file(zip_path: &std::path::Path, filename: &str) -> Option<String> {
    let entries = zip_scanner::scan_zip(zip_path).ok()?;
    let filename_lower = filename.to_lowercase();
    entries
        .into_iter()
        .find(|e| {
            let full = format!("{}.{}", e.stem, e.extension);
            full == filename_lower
        })
        .map(|e| e.internal_path)
}

#[test]
fn test_read_classes_2da_from_zip() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let internal_path = get_internal_path_for_file(&zip_path, "classes.2da")
                .expect("classes.2da should be in 2da.zip");

            let mut reader = ZipContentReader::new();
            let result = reader.read_file_from_zip(
                zip_path.to_string_lossy().to_string(),
                internal_path.clone(),
            );

            assert!(
                result.is_ok(),
                "Should read classes.2da: {:?}",
                result.err()
            );
            let data = result.unwrap();

            assert!(data.len() > 100, "classes.2da should have content");

            let content = String::from_utf8_lossy(&data);
            assert!(content.contains("2DA"), "Should be a valid 2DA file");

            println!(
                "Read {} bytes from {} (internal: {})",
                data.len(),
                zip_path.display(),
                internal_path
            );
            println!("First 200 chars:\n{}", &content[..content.len().min(200)]);
        }
    }
}

#[test]
fn test_archive_caching() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let classes_path = get_internal_path_for_file(&zip_path, "classes.2da")
                .expect("classes.2da should exist");
            let feat_path =
                get_internal_path_for_file(&zip_path, "feat.2da").expect("feat.2da should exist");

            let zip_str = zip_path.to_string_lossy().to_string();
            let mut reader = ZipContentReader::new();

            let _ = reader.read_file_from_zip(zip_str.clone(), classes_path);
            let stats1 = reader.get_stats();
            let opens1 = stats1
                .get("archives_opened")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let hits1 = stats1
                .get("cache_hits")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let _ = reader.read_file_from_zip(zip_str, feat_path);
            let stats2 = reader.get_stats();
            let opens2 = stats2
                .get("archives_opened")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let hits2 = stats2
                .get("cache_hits")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            assert_eq!(opens1, opens2, "Should not reopen archive");
            assert!(hits2 > hits1, "Should get cache hit on second read");

            println!("Archives opened: {opens2}, Cache hits: {hits2}");
        }
    }
}

#[test]
fn test_read_multiple_files_batch() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let zip_str = zip_path.to_string_lossy().to_string();

            let classes_path =
                get_internal_path_for_file(&zip_path, "classes.2da").expect("classes.2da");
            let feat_path = get_internal_path_for_file(&zip_path, "feat.2da").expect("feat.2da");
            let skills_path =
                get_internal_path_for_file(&zip_path, "skills.2da").expect("skills.2da");

            let mut reader = ZipContentReader::new();
            let requests = vec![
                ZipReadRequest {
                    zip_path: zip_str.clone(),
                    internal_path: classes_path,
                    request_id: "classes".to_string(),
                },
                ZipReadRequest {
                    zip_path: zip_str.clone(),
                    internal_path: feat_path,
                    request_id: "feats".to_string(),
                },
                ZipReadRequest {
                    zip_path: zip_str.clone(),
                    internal_path: skills_path,
                    request_id: "skills".to_string(),
                },
                ZipReadRequest {
                    zip_path: zip_str.clone(),
                    internal_path: "nonexistent_path/fake.2da".to_string(),
                    request_id: "missing".to_string(),
                },
            ];

            let results = reader.read_multiple_files(requests);

            assert_eq!(results.len(), 4);

            let classes_result = results.iter().find(|r| r.request_id == "classes").unwrap();
            assert!(classes_result.success, "classes.2da should succeed");
            assert!(classes_result.data.is_some());

            let missing_result = results.iter().find(|r| r.request_id == "missing").unwrap();
            assert!(!missing_result.success, "nonexistent file should fail");
            assert!(missing_result.error.is_some());

            println!("Batch read results:");
            for result in &results {
                if result.success {
                    println!(
                        "  {}: {} bytes",
                        result.request_id,
                        result.data.as_ref().map(|d| d.len()).unwrap_or(0)
                    );
                } else {
                    println!("  {}: FAILED - {:?}", result.request_id, result.error);
                }
            }
        }
    }
}

#[test]
fn test_read_from_multiple_zips() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let mut reader = ZipContentReader::new();
        let mut requests = Vec::new();

        for zip_name in ["2da.zip", "2da_x1.zip", "2da_x2.zip"] {
            let zip_path = data_dir.join(zip_name);
            if zip_path.exists()
                && let Some(internal_path) = get_internal_path_for_file(&zip_path, "classes.2da")
            {
                requests.push(ZipReadRequest {
                    zip_path: zip_path.to_string_lossy().to_string(),
                    internal_path,
                    request_id: zip_name.to_string(),
                });
            }
        }

        if !requests.is_empty() {
            let results = reader.read_multiple_files(requests);

            println!("Read from {} zips:", results.len());
            for result in &results {
                println!("  {}: success={}", result.request_id, result.success);
            }

            let stats = reader.get_stats();
            println!(
                "Open archives: {}",
                stats
                    .get("open_archives")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            );
        }
    }
}

#[test]
fn test_file_exists_in_zip() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let zip_str = zip_path.to_string_lossy().to_string();

            let classes_path = get_internal_path_for_file(&zip_path, "classes.2da")
                .expect("classes.2da should exist");

            let mut reader = ZipContentReader::new();

            let exists = reader.file_exists_in_zip(zip_str.clone(), classes_path);
            assert!(exists.is_ok());
            assert!(exists.unwrap(), "classes.2da should exist");

            let not_exists = reader.file_exists_in_zip(zip_str, "fake/nonexistent.2da".to_string());
            assert!(not_exists.is_ok());
            assert!(!not_exists.unwrap(), "nonexistent.2da should not exist");
        }
    }
}

#[test]
fn test_preopen_and_close_archives() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let mut zip_paths = Vec::new();

        for zip_name in ["2da.zip", "2da_x1.zip"] {
            let zip_path = data_dir.join(zip_name);
            if zip_path.exists() {
                zip_paths.push(zip_path.to_string_lossy().to_string());
            }
        }

        if !zip_paths.is_empty() {
            let mut reader = ZipContentReader::new();

            let result = reader.preopen_zip_archives(zip_paths.clone());
            assert!(result.is_ok());

            let stats = reader.get_stats();
            let open_count = stats
                .get("open_archives")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            println!("Pre-opened {open_count} archives");
            assert!(open_count > 0);

            reader.close_all_archives();

            let stats = reader.get_stats();
            let open_count = stats
                .get("open_archives")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            assert_eq!(open_count, 0, "All archives should be closed");
        }
    }
}

#[test]
fn test_stats_tracking() {
    let paths = get_paths();

    if let Some(data_dir) = paths.data() {
        let zip_path = data_dir.join("2da.zip");

        if zip_path.exists() {
            let zip_str = zip_path.to_string_lossy().to_string();

            let classes_path =
                get_internal_path_for_file(&zip_path, "classes.2da").expect("classes.2da");
            let feat_path = get_internal_path_for_file(&zip_path, "feat.2da").expect("feat.2da");

            let mut reader = ZipContentReader::new();

            let _ = reader.read_file_from_zip(zip_str.clone(), classes_path);
            let _ = reader.read_file_from_zip(zip_str, feat_path);

            let stats = reader.get_stats();

            let files_read = stats
                .get("files_read")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let bytes_read = stats
                .get("bytes_read")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            assert_eq!(files_read, 2, "Should have read 2 files");
            assert!(bytes_read > 0, "Should have read bytes");

            println!("Stats: files={files_read}, bytes={bytes_read}");
        }
    }
}

#[test]
fn test_read_from_enhanced_data() {
    let paths = get_paths();

    if let Some(enhanced_data) = paths.enhanced_data() {
        let zip_path = enhanced_data.join("2da.zip");

        if zip_path.exists() {
            if let Some(internal_path) = get_internal_path_for_file(&zip_path, "classes.2da") {
                let mut reader = ZipContentReader::new();
                let result = reader
                    .read_file_from_zip(zip_path.to_string_lossy().to_string(), internal_path);

                if let Ok(data) = result {
                    println!("Read {} bytes from enhanced classes.2da", data.len());
                }
            }
        } else {
            println!("Enhanced 2da.zip not found, skipping");
        }
    }
}
