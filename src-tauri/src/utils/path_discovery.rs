use dirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

const KNOWN_GAME_FOLDER_NAMES: &[&str] = &[
    "NWN2 Enhanced Edition",
    "Neverwinter Nights 2 Enhanced Edition",
    "Neverwinter Nights 2",
    "NWN2",
];

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

    let candidate_start = Instant::now();
    let candidate_paths = if let Some(custom_paths) = search_paths {
        build_candidate_paths_from_roots(custom_paths.into_iter().map(PathBuf::from).collect())
    } else {
        get_default_candidate_paths()
    };
    timing_breakdown.push(PathTiming {
        operation: "candidate_collection".to_string(),
        duration_ms: candidate_start.elapsed().as_millis() as u64,
        paths_checked: candidate_paths.len() as u32,
        paths_found: 0,
    });

    let validation_start = Instant::now();
    let mut nwn2_paths = Vec::new();
    let mut steam_paths = Vec::new();
    let mut gog_paths = Vec::new();
    let mut paths_found = 0;

    for candidate in &candidate_paths {
        if !candidate.exists() || !is_nwn2_installation(candidate) {
            continue;
        }

        paths_found += 1;
        let path_str = candidate.to_string_lossy().to_string();
        nwn2_paths.push(path_str.clone());

        let path_lower = path_str.to_lowercase();
        if path_lower.contains("steam") || path_lower.contains("steamapps") {
            steam_paths.push(path_str);
        } else if path_lower.contains("gog") {
            gog_paths.push(path_str);
        }
    }

    timing_breakdown.push(PathTiming {
        operation: "candidate_validation".to_string(),
        duration_ms: validation_start.elapsed().as_millis() as u64,
        paths_checked: candidate_paths.len() as u32,
        paths_found,
    });

    let total_time = start_time.elapsed();

    Ok(DiscoveryResult {
        nwn2_paths: dedupe_string_paths(nwn2_paths),
        steam_paths: dedupe_string_paths(steam_paths),
        gog_paths: dedupe_string_paths(gog_paths),
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

fn get_default_candidate_paths() -> Vec<PathBuf> {
    build_candidate_paths_from_roots(get_default_search_roots())
}

fn build_candidate_paths_from_roots(roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut candidates = HashSet::new();
    let mut steam_roots = HashSet::new();
    let mut steam_library_roots = HashSet::new();

    for root in roots {
        add_root_candidates(
            &root,
            &mut candidates,
            &mut steam_roots,
            &mut steam_library_roots,
        );
    }

    for steam_root in &steam_roots {
        add_steam_root_candidates(steam_root, &mut candidates, &mut steam_library_roots);
    }

    for library_root in &steam_library_roots {
        add_steam_library_candidates(library_root, &mut candidates);
    }

    for install_path in get_epic_manifest_install_paths() {
        candidates.insert(install_path);
    }

    let mut candidate_paths: Vec<_> = candidates.into_iter().collect();
    candidate_paths.sort();
    candidate_paths
}

fn add_root_candidates(
    root: &Path,
    candidates: &mut HashSet<PathBuf>,
    steam_roots: &mut HashSet<PathBuf>,
    steam_library_roots: &mut HashSet<PathBuf>,
) {
    candidates.insert(root.to_path_buf());

    for subdir in [
        root.to_path_buf(),
        root.join("Games"),
        root.join("Epic Games"),
        root.join("GOG Games"),
        root.join("Program Files").join("Epic Games"),
        root.join("Program Files").join("GOG Games"),
        root.join("Program Files (x86)").join("Epic Games"),
        root.join("Program Files (x86)").join("GOG Games"),
    ] {
        add_named_install_candidates(&subdir, candidates);
    }

    for steam_root in [
        root.to_path_buf(),
        root.join("Steam"),
        root.join("SteamLibrary"),
        root.join("Program Files").join("Steam"),
        root.join("Program Files (x86)").join("Steam"),
        root.join(".steam").join("steam"),
        root.join(".steam").join("root"),
        root.join(".local").join("share").join("Steam"),
        root.join("Library")
            .join("Application Support")
            .join("Steam"),
    ] {
        steam_roots.insert(steam_root.clone());
        if steam_root.join("steamapps").exists() {
            steam_library_roots.insert(steam_root);
        }
    }
}

fn add_steam_root_candidates(
    steam_root: &Path,
    candidates: &mut HashSet<PathBuf>,
    steam_library_roots: &mut HashSet<PathBuf>,
) {
    if steam_root.join("steamapps").exists() {
        steam_library_roots.insert(steam_root.to_path_buf());
    }

    if let Some(parent) = steam_root.parent()
        && parent.join("SteamLibrary").exists()
    {
        steam_library_roots.insert(parent.join("SteamLibrary"));
    }

    let libraryfolders_path = steam_root.join("steamapps").join("libraryfolders.vdf");
    for library_root in parse_steam_libraryfolders(&libraryfolders_path) {
        steam_library_roots.insert(library_root);
    }

    add_steam_library_candidates(steam_root, candidates);
}

fn add_steam_library_candidates(library_root: &Path, candidates: &mut HashSet<PathBuf>) {
    add_named_install_candidates(&library_root.join("steamapps").join("common"), candidates);
    add_named_install_candidates(&library_root.join("common"), candidates);
}

fn add_named_install_candidates(base: &Path, candidates: &mut HashSet<PathBuf>) {
    candidates.insert(base.to_path_buf());

    for folder_name in KNOWN_GAME_FOLDER_NAMES {
        candidates.insert(base.join(folder_name));
    }
}

fn dedupe_string_paths(paths: Vec<String>) -> Vec<String> {
    let mut unique_paths = Vec::new();
    let mut seen_paths = HashSet::new();

    for path in paths {
        let canonical_path = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
        let canonical_str = canonical_path.to_string_lossy().to_string();

        if seen_paths.insert(canonical_str) {
            unique_paths.push(path);
        }
    }

    unique_paths
}

fn get_default_search_roots() -> Vec<PathBuf> {
    let mut roots = HashSet::new();

    #[cfg(target_os = "windows")]
    {
        for drive_root in get_windows_drive_roots() {
            roots.insert(drive_root);
        }

        if let Ok(program_files) = std::env::var("ProgramFiles") {
            roots.insert(PathBuf::from(program_files));
        }

        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            roots.insert(PathBuf::from(program_files_x86));
        }
    }

    #[cfg(target_os = "macos")]
    {
        roots.insert(PathBuf::from("/Applications"));

        if let Some(home) = dirs::home_dir() {
            roots.insert(home.join("Applications"));
            roots.insert(
                home.join("Library")
                    .join("Application Support")
                    .join("Steam"),
            );
            roots.insert(home.join("Games"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            roots.insert(home.join(".steam").join("steam"));
            roots.insert(home.join(".steam").join("root"));
            roots.insert(home.join(".local").join("share").join("Steam"));
            roots.insert(home.join("Games"));
        }

        for drive_root in get_wsl_windows_drive_roots() {
            roots.insert(drive_root);
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        if let Some(home) = dirs::home_dir() {
            roots.insert(home.join("Games"));
        }
    }

    let mut root_paths: Vec<_> = roots.into_iter().collect();
    root_paths.sort();
    root_paths
}

#[cfg(target_os = "windows")]
fn get_windows_drive_roots() -> Vec<PathBuf> {
    ('A'..='Z')
        .map(|drive| PathBuf::from(format!("{drive}:/")))
        .filter(|path| path.exists())
        .collect()
}

#[cfg(target_os = "linux")]
fn get_wsl_windows_drive_roots() -> Vec<PathBuf> {
    let mut drive_roots = Vec::new();
    let mnt_root = PathBuf::from("/mnt");

    if let Ok(entries) = std::fs::read_dir(mnt_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_drive = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.len() == 1 && name.chars().all(|ch| ch.is_ascii_alphabetic())
                });

            if is_drive {
                drive_roots.push(path);
            }
        }
    }

    drive_roots.sort();
    drive_roots
}

fn parse_steam_libraryfolders(vdf_path: &Path) -> Vec<PathBuf> {
    let Ok(content) = std::fs::read_to_string(vdf_path) else {
        return Vec::new();
    };

    let mut libraries = Vec::new();

    for line in content.lines() {
        let Some(value) = parse_vdf_key_value(line, "path") else {
            continue;
        };

        let normalized = value.replace("\\\\", "\\");
        libraries.push(PathBuf::from(normalized));
    }

    libraries
}

fn parse_vdf_key_value(line: &str, key: &str) -> Option<String> {
    let tokens: Vec<&str> = line.split('"').collect();
    if tokens.len() >= 4 && tokens[1] == key {
        return Some(tokens[3].to_string());
    }

    None
}

fn get_epic_manifest_install_paths() -> Vec<PathBuf> {
    let mut install_paths = Vec::new();

    for manifest_dir in epic_manifest_dirs() {
        let Ok(entries) = std::fs::read_dir(manifest_dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("item") {
                continue;
            }

            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(manifest) = serde_json::from_str::<Value>(&content) else {
                continue;
            };
            let Some(install_location) = manifest.get("InstallLocation").and_then(Value::as_str)
            else {
                continue;
            };

            install_paths.push(PathBuf::from(install_location));
        }
    }

    install_paths
}

fn epic_manifest_dirs() -> Vec<PathBuf> {
    let mut manifest_dirs = HashSet::new();

    #[cfg(target_os = "windows")]
    {
        if let Ok(program_data) = std::env::var("ProgramData") {
            manifest_dirs.insert(
                PathBuf::from(program_data)
                    .join("Epic")
                    .join("EpicGamesLauncher")
                    .join("Data")
                    .join("Manifests"),
            );
        } else {
            manifest_dirs.insert(
                PathBuf::from("C:/ProgramData")
                    .join("Epic")
                    .join("EpicGamesLauncher")
                    .join("Data")
                    .join("Manifests"),
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        for drive_root in get_wsl_windows_drive_roots() {
            manifest_dirs.insert(
                drive_root
                    .join("ProgramData")
                    .join("Epic")
                    .join("EpicGamesLauncher")
                    .join("Data")
                    .join("Manifests"),
            );
        }
    }

    let mut manifest_paths: Vec<_> = manifest_dirs.into_iter().collect();
    manifest_paths.sort();
    manifest_paths
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

#[cfg(test)]
mod tests {
    use super::{build_candidate_paths_from_roots, parse_steam_libraryfolders};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_parse_steam_libraryfolders_reads_library_paths() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let steamapps_dir = temp_dir.path().join("steamapps");
        fs::create_dir_all(&steamapps_dir).expect("steamapps dir");

        let vdf_path = steamapps_dir.join("libraryfolders.vdf");
        fs::write(
            &vdf_path,
            "\"libraryfolders\"\n{\n    \"0\"\n    {\n        \"path\"        \"D:\\\\SteamLibrary\"\n    }\n    \"1\"\n    {\n        \"path\"        \"E:\\\\Games\"\n    }\n}\n",
        )
        .expect("write vdf");

        let libraries = parse_steam_libraryfolders(&vdf_path);

        assert_eq!(libraries.len(), 2);
        assert_eq!(libraries[0], PathBuf::from("D:\\SteamLibrary"));
        assert_eq!(libraries[1], PathBuf::from("E:\\Games"));
    }

    #[test]
    fn test_build_candidate_paths_uses_libraryfolders_without_directory_walk() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let steam_root = temp_dir.path().join("Steam");
        let steamapps_dir = steam_root.join("steamapps");
        fs::create_dir_all(&steamapps_dir).expect("steamapps dir");

        let vdf_path = steamapps_dir.join("libraryfolders.vdf");
        let library_root = temp_dir.path().join("SteamLibrary");
        fs::write(
            &vdf_path,
            format!(
                "\"libraryfolders\"\n{{\n    \"0\"\n    {{\n        \"path\"        \"{}\"\n    }}\n}}\n",
                library_root.to_string_lossy().replace('\\', "\\\\")
            ),
        )
        .expect("write vdf");

        let candidates = build_candidate_paths_from_roots(vec![steam_root]);
        let expected = library_root
            .join("steamapps")
            .join("common")
            .join("NWN2 Enhanced Edition");

        assert!(
            candidates.contains(&expected),
            "steam libraryfolders path should produce a direct candidate"
        );
    }
}
