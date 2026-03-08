use dirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathTiming {
    pub operation: String,
    pub duration_ms: u64,
    pub paths_checked: u32,
    pub paths_found: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub nwn2_paths: Vec<String>,
    pub steam_paths: Vec<String>,
    pub gog_paths: Vec<String>,
    pub total_time_ms: u64,
    pub timing_breakdown: Vec<PathTiming>,
}

pub fn discover_nwn2_paths_rust(
    search_paths: Option<Vec<String>>,
) -> Result<DiscoveryResult, String> {
    let start_time = Instant::now();
    let mut timing_breakdown = Vec::new();

    let mut nwn2_paths = Vec::new();
    let mut steam_paths = Vec::new();
    let mut gog_paths = Vec::new();

    let paths_to_search = if let Some(custom_paths) = search_paths {
        custom_paths.into_iter().map(PathBuf::from).collect()
    } else {
        get_default_search_paths()
    };

    let search_start = Instant::now();
    let mut paths_checked = 0;
    let mut paths_found = 0;

    for search_path in paths_to_search {
        if !search_path.exists() {
            continue;
        }

        paths_checked += 1;

        let patterns = ["Neverwinter Nights 2", "NWN2", "nwn2"];

        if is_nwn2_installation(&search_path) {
            let path_str = search_path.to_string_lossy().to_string();
            nwn2_paths.push(path_str.clone());
            paths_found += 1;

            if path_str.contains("Steam") || path_str.contains("steamapps") {
                steam_paths.push(path_str);
            } else if path_str.contains("GOG") {
                gog_paths.push(path_str);
            }
        }

        if let Ok(entries) = std::fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_dir()
                    && let Some(name) = entry.file_name().to_str()
                {
                    let name_lower = name.to_lowercase();
                    let matches_pattern = patterns.iter().any(|pattern| {
                        let pattern_lower = pattern.to_lowercase();
                        name_lower.contains(&pattern_lower)
                            || name_lower.starts_with(&pattern_lower)
                    });

                    if matches_pattern && is_nwn2_installation(&entry.path()) {
                        let path_str = entry.path().to_string_lossy().to_string();
                        nwn2_paths.push(path_str.clone());
                        paths_found += 1;

                        if path_str.contains("Steam") || path_str.contains("steamapps") {
                            steam_paths.push(path_str);
                        } else if path_str.contains("GOG") {
                            gog_paths.push(path_str);
                        }
                    }
                }
            }
        }

        let search_name = search_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if (search_name == "common" || search_name == "steamapps")
            && let Ok(entries) = std::fs::read_dir(&search_path)
        {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_dir()
                    && let Some(name) = entry.file_name().to_str()
                    && name.to_uppercase().contains("NWN2")
                    && is_nwn2_installation(&entry.path())
                {
                    let path_str = entry.path().to_string_lossy().to_string();
                    nwn2_paths.push(path_str.clone());
                    paths_found += 1;

                    steam_paths.push(path_str);
                }
            }
        }
    }

    let search_time = search_start.elapsed();
    timing_breakdown.push(PathTiming {
        operation: "path_discovery".to_string(),
        duration_ms: search_time.as_millis() as u64,
        paths_checked,
        paths_found,
    });

    let mut unique_nwn2_paths = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for path in nwn2_paths {
        let canonical_path = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
        let canonical_str = canonical_path.to_string_lossy().to_string();

        if !seen_paths.contains(&canonical_str) {
            seen_paths.insert(canonical_str);
            unique_nwn2_paths.push(path);
        }
    }

    let mut unique_steam_paths = Vec::new();
    let mut seen_steam = std::collections::HashSet::new();
    for path in steam_paths {
        let canonical_path = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
        let canonical_str = canonical_path.to_string_lossy().to_string();
        if !seen_steam.contains(&canonical_str) {
            seen_steam.insert(canonical_str);
            unique_steam_paths.push(path);
        }
    }

    let mut unique_gog_paths = Vec::new();
    let mut seen_gog = std::collections::HashSet::new();
    for path in gog_paths {
        let canonical_path = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
        let canonical_str = canonical_path.to_string_lossy().to_string();
        if !seen_gog.contains(&canonical_str) {
            seen_gog.insert(canonical_str);
            unique_gog_paths.push(path);
        }
    }

    let total_time = start_time.elapsed();

    Ok(DiscoveryResult {
        nwn2_paths: unique_nwn2_paths,
        steam_paths: unique_steam_paths,
        gog_paths: unique_gog_paths,
        total_time_ms: total_time.as_millis() as u64,
        timing_breakdown,
    })
}

pub fn profile_path_discovery_rust(iterations: u32) -> Result<HashMap<String, f64>, String> {
    let mut results = HashMap::new();
    let mut total_times = Vec::new();

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = discover_nwn2_paths_rust(None)?;
        let duration = start.elapsed();
        total_times.push(duration.as_secs_f64());
    }

    if !total_times.is_empty() {
        let mean_time = total_times.iter().sum::<f64>() / total_times.len() as f64;
        let min_time = total_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_time = total_times.iter().fold(0.0f64, |a, &b| a.max(b));

        results.insert("mean_seconds".to_string(), mean_time);
        results.insert("min_seconds".to_string(), min_time);
        results.insert("max_seconds".to_string(), max_time);
        results.insert("iterations".to_string(), f64::from(iterations));
    }

    Ok(results)
}

fn get_default_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Documents"));
        paths.push(home.join("Games"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            let pf = PathBuf::from(program_files);
            paths.push(pf.clone());
            paths.push(pf.join("Steam").join("steamapps").join("common"));
            paths.push(pf.join("GOG Games"));
        } else {
            paths.push(PathBuf::from("C:/Program Files"));
            paths.push(PathBuf::from("C:/Program Files/Steam/steamapps/common"));
            paths.push(PathBuf::from("C:/Program Files/GOG Games"));
        }

        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            let pf86 = PathBuf::from(program_files_x86);
            paths.push(pf86.clone());
            paths.push(pf86.join("Steam").join("steamapps").join("common"));
            paths.push(pf86.join("GOG Games"));
        } else {
            paths.push(PathBuf::from("C:/Program Files (x86)"));
            paths.push(PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common",
            ));
            paths.push(PathBuf::from("C:/Program Files (x86)/GOG Games"));
        }

        paths.push(PathBuf::from("C:/GOG Games"));

        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Games"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.clone());
            paths.push(home.join("Games"));
        }
        paths.push(PathBuf::from("/Applications"));
        paths.push(PathBuf::from("/opt"));
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.clone());
            paths.push(
                home.join(".steam")
                    .join("steam")
                    .join("steamapps")
                    .join("common"),
            );
            paths.push(
                home.join(".local")
                    .join("share")
                    .join("Steam")
                    .join("steamapps")
                    .join("common"),
            );
            paths.push(home.join("Games"));
        }

        paths.push(PathBuf::from("/Applications"));
        paths.push(PathBuf::from("/opt"));
        paths.push(PathBuf::from("/usr/local/games"));

        if PathBuf::from("/mnt/c").exists() {
            paths.push(PathBuf::from("/mnt/c/Program Files"));
            paths.push(PathBuf::from("/mnt/c/Program Files (x86)"));
            paths.push(PathBuf::from("/mnt/c/Program Files/Steam/steamapps/common"));
            paths.push(PathBuf::from(
                "/mnt/c/Program Files (x86)/Steam/steamapps/common",
            ));
            paths.push(PathBuf::from("/mnt/c/GOG Games"));
        }
    }

    paths
}

fn is_nwn2_installation(path: &Path) -> bool {
    let indicators = ["data", "dialog.tlk", "nwn2main.exe", "nwn2.exe", "enhanced"];

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                let name_lower = name.to_lowercase();
                for indicator in &indicators {
                    if name_lower == indicator.to_lowercase() {
                        return true;
                    }
                }
            }
        }
    }

    false
}
