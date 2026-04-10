use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub struct ScannedFile {
    pub stem: String,
    pub extension: String,
    pub path: PathBuf,
    pub mtime: f64,
}

/// Scan a directory for all files, optionally recursive.
/// Returns lowercase stem and extension for each file found.
pub fn scan_directory(dir: &Path, recursive: bool) -> Vec<ScannedFile> {
    if !dir.exists() {
        return Vec::new();
    }

    let walker = if recursive {
        WalkDir::new(dir).into_iter()
    } else {
        WalkDir::new(dir).max_depth(1).into_iter()
    };

    walker
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let mtime = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0.0, |d| d.as_secs_f64());
            let path = entry.into_path();
            let stem = path.file_stem()?.to_str()?.to_lowercase();
            let ext = path.extension()?.to_str()?.to_lowercase();

            Some(ScannedFile {
                stem,
                extension: ext,
                path,
                mtime,
            })
        })
        .collect()
}

/// Scan Steam Workshop directory structure.
/// Expects: `<workshop_dir>/<mod_id>/override/` layout.
/// Recursively scans each mod's override subdirectory.
pub fn scan_workshop(workshop_dir: &Path) -> Vec<ScannedFile> {
    if !workshop_dir.exists() {
        return Vec::new();
    }

    let Ok(entries) = std::fs::read_dir(workshop_dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .flat_map(|mod_entry| {
            let override_dir = mod_entry.path().join("override");
            if override_dir.is_dir() {
                scan_directory(&override_dir, true)
            } else {
                Vec::new()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_directory_finds_all_extensions() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("classes.2da"), b"2DA").unwrap();
        fs::write(temp.path().join("fireball.dds"), b"DDS").unwrap();
        fs::write(temp.path().join("readme.txt"), b"txt").unwrap();

        let results = scan_directory(temp.path(), true);
        assert_eq!(results.len(), 3);

        let tda = results.iter().find(|r| r.stem == "classes").unwrap();
        assert_eq!(tda.extension, "2da");

        let dds = results.iter().find(|r| r.stem == "fireball").unwrap();
        assert_eq!(dds.extension, "dds");
    }

    #[test]
    fn test_scan_directory_recursive() {
        let temp = TempDir::new().unwrap();
        let sub = temp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(temp.path().join("root.2da"), b"2DA").unwrap();
        fs::write(sub.join("nested.dds"), b"DDS").unwrap();

        let results = scan_directory(temp.path(), true);
        assert_eq!(results.len(), 2);

        let results_flat = scan_directory(temp.path(), false);
        assert_eq!(results_flat.len(), 1);
    }

    #[test]
    fn test_scan_directory_lowercases_stem_and_ext() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("MyIcon.DDS"), b"DDS").unwrap();

        let results = scan_directory(temp.path(), false);
        assert_eq!(results[0].stem, "myicon");
        assert_eq!(results[0].extension, "dds");
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let results = scan_directory(Path::new("/nonexistent/path"), true);
        assert!(results.is_empty());
    }

    #[test]
    fn test_scan_workshop_directory() {
        let temp = TempDir::new().unwrap();
        let mod1 = temp.path().join("12345").join("override");
        fs::create_dir_all(&mod1).unwrap();
        fs::write(mod1.join("custom.2da"), b"2DA").unwrap();
        fs::write(mod1.join("icon.dds"), b"DDS").unwrap();
        let mod2 = temp.path().join("67890");
        fs::create_dir_all(&mod2).unwrap();
        fs::write(mod2.join("loose.tga"), b"TGA").unwrap();

        let results = scan_workshop(temp.path());
        assert_eq!(results.len(), 2);
    }
}
