use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::SystemTime;

use chrono::Local;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::error::{SaveGameError, SaveGameResult};

static BACKUP_CREATED_FOR_SAVES: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static LEGACY_ARTIFACT_CLEANUP_DONE: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const BACKUP_TIMESTAMP_FORMAT: &str = "%Y%m%d_%H%M%S";
const DEFAULT_KEEP_COUNT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub timestamp: String,
    pub size_bytes: u64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub pre_restore_backup: Option<PathBuf>,
    pub files_restored: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub removed_count: usize,
    pub remaining_count: usize,
    pub freed_bytes: u64,
}

pub fn get_backups_dir(save_dir: &Path) -> PathBuf {
    if let Some(parent) = save_dir.parent()
        && let Some(parent_name) = parent.file_name().and_then(|n| n.to_str())
    {
        if parent_name.eq_ignore_ascii_case("multiplayer")
            && let Some(saves_dir) = parent.parent()
            && saves_dir
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case("saves"))
        {
            let docs_root = saves_dir.parent().unwrap_or(saves_dir);
            return docs_root
                .join(".nwn2ee-save-editor-backups")
                .join("multiplayer");
        }

        if parent_name.eq_ignore_ascii_case("saves") {
            let docs_root = parent.parent().unwrap_or(parent);
            return docs_root
                .join(".nwn2ee-save-editor-backups")
                .join("singleplayer");
        }
    }

    save_dir
        .parent()
        .map_or_else(|| save_dir.join("backups"), |p| p.join("backups"))
}

pub fn get_backup_dir_for_save(save_dir: &Path) -> PathBuf {
    let backups_root = get_backups_dir(save_dir);
    let save_name = save_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    backups_root.join(save_name)
}

pub fn has_backup_been_created(save_path: &Path) -> bool {
    BACKUP_CREATED_FOR_SAVES
        .lock()
        .map(|set| set.contains(save_path))
        .unwrap_or(false)
}

pub fn mark_backup_created(save_path: &Path) {
    if let Ok(mut set) = BACKUP_CREATED_FOR_SAVES.lock() {
        set.insert(save_path.to_path_buf());
    }
}

pub fn clear_backup_tracking() {
    if let Ok(mut set) = BACKUP_CREATED_FOR_SAVES.lock() {
        set.clear();
    }
}

pub fn cleanup_legacy_save_tree_artifacts(save_dir: &Path) -> SaveGameResult<()> {
    let Some(docs_root) = infer_docs_root_from_save(save_dir) else {
        return Ok(());
    };

    let already_done = LEGACY_ARTIFACT_CLEANUP_DONE
        .lock()
        .map(|set| set.contains(&docs_root))
        .unwrap_or(false);
    if already_done {
        return Ok(());
    }

    let quarantine_root = docs_root
        .join(".nwn2ee-save-editor-quarantine")
        .join("legacy-save-artifacts");
    fs::create_dir_all(&quarantine_root)?;

    let saves_root = docs_root.join("saves");
    if saves_root.exists() {
        quarantine_named_dirs(&saves_root, "resgff", &quarantine_root)?;

        for legacy_backups in [
            saves_root.join("backups"),
            saves_root.join("multiplayer").join("backups"),
        ] {
            if legacy_backups.exists() {
                move_to_quarantine(&legacy_backups, &quarantine_root)?;
            }
        }
    }

    if let Ok(mut set) = LEGACY_ARTIFACT_CLEANUP_DONE.lock() {
        set.insert(docs_root);
    }

    Ok(())
}

pub fn create_backup(save_dir: &Path) -> SaveGameResult<PathBuf> {
    if !save_dir.exists() {
        return Err(SaveGameError::NotFound {
            path: save_dir.to_path_buf(),
        });
    }

    let backup_dir = get_backup_dir_for_save(save_dir);
    fs::create_dir_all(&backup_dir)?;

    let timestamp = Local::now().format(BACKUP_TIMESTAMP_FORMAT).to_string();
    let backup_name = format!("backup_{timestamp}");
    let backup_path = backup_dir.join(&backup_name);

    copy_directory(save_dir, &backup_path)?;

    mark_backup_created(save_dir);

    info!(
        "Created backup: {} -> {}",
        save_dir.display(),
        backup_path.display()
    );
    Ok(backup_path)
}

pub fn create_pre_restore_backup(save_dir: &Path) -> SaveGameResult<PathBuf> {
    let backup_dir = get_backup_dir_for_save(save_dir);
    fs::create_dir_all(&backup_dir)?;

    let timestamp = Local::now().format(BACKUP_TIMESTAMP_FORMAT).to_string();
    let backup_name = format!("pre_restore_{timestamp}");
    let backup_path = backup_dir.join(&backup_name);

    copy_directory(save_dir, &backup_path)?;

    info!(
        "Created pre-restore backup: {} -> {}",
        save_dir.display(),
        backup_path.display()
    );
    Ok(backup_path)
}

pub fn list_backups(save_dir: &Path) -> SaveGameResult<Vec<BackupInfo>> {
    let backup_dir = get_backup_dir_for_save(save_dir);

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if name.starts_with("backup_") || name.starts_with("pre_restore_") {
                let timestamp = name
                    .strip_prefix("backup_")
                    .or_else(|| name.strip_prefix("pre_restore_"))
                    .unwrap_or(name)
                    .to_string();

                let metadata = fs::metadata(&path)?;
                let created_at = metadata
                    .created()
                    .or_else(|_| metadata.modified())
                    .ok()
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_secs() as i64);

                let size_bytes = calculate_dir_size(&path);

                backups.push(BackupInfo {
                    path,
                    timestamp,
                    size_bytes,
                    created_at,
                });
            }
        }
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(backups)
}

pub fn restore_from_backup(
    backup_path: &Path,
    save_dir: &Path,
    create_pre_restore: bool,
) -> SaveGameResult<RestoreResult> {
    if !backup_path.exists() {
        return Err(SaveGameError::NotFound {
            path: backup_path.to_path_buf(),
        });
    }

    let pre_restore_backup = if create_pre_restore && save_dir.exists() {
        Some(create_pre_restore_backup(save_dir)?)
    } else {
        None
    };

    if save_dir.exists() {
        fs::remove_dir_all(save_dir)?;
    }

    let files_restored = copy_directory(backup_path, save_dir)?;

    info!(
        "Restored backup: {} -> {} ({} files)",
        backup_path.display(),
        save_dir.display(),
        files_restored
    );

    Ok(RestoreResult {
        success: true,
        pre_restore_backup,
        files_restored,
        message: format!("Successfully restored {files_restored} files"),
    })
}

pub fn restore_modules_from_backup(
    backup_path: &Path,
    save_dir: &Path,
) -> SaveGameResult<RestoreResult> {
    if !backup_path.exists() {
        return Err(SaveGameError::NotFound {
            path: backup_path.to_path_buf(),
        });
    }

    let mut files_restored = 0usize;

    for entry in fs::read_dir(backup_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "z") {
            let filename = path.file_name().unwrap();
            let dest = save_dir.join(filename);
            fs::copy(&path, &dest)?;
            files_restored += 1;
            debug!("Restored module file: {}", filename.to_string_lossy());
        }
    }

    info!(
        "Restored {} module .z files from {} -> {}",
        files_restored,
        backup_path.display(),
        save_dir.display()
    );

    Ok(RestoreResult {
        success: true,
        pre_restore_backup: None,
        files_restored,
        message: format!("Successfully restored {files_restored} module files"),
    })
}

pub fn cleanup_old_backups(save_dir: &Path, keep_count: usize) -> SaveGameResult<CleanupResult> {
    let keep_count = if keep_count == 0 {
        DEFAULT_KEEP_COUNT
    } else {
        keep_count
    };

    let mut backups = list_backups(save_dir)?;

    if backups.len() <= keep_count {
        return Ok(CleanupResult {
            removed_count: 0,
            remaining_count: backups.len(),
            freed_bytes: 0,
        });
    }

    let mut removed_count = 0;
    let mut freed_bytes = 0u64;

    while backups.len() > keep_count {
        if let Some(oldest) = backups.pop()
            && oldest.path.exists()
        {
            freed_bytes += oldest.size_bytes;
            fs::remove_dir_all(&oldest.path)?;
            removed_count += 1;
            debug!("Removed old backup: {}", oldest.path.display());
        }
    }

    info!(
        "Cleaned up {} old backups, freed {} bytes",
        removed_count, freed_bytes
    );

    Ok(CleanupResult {
        removed_count,
        remaining_count: backups.len(),
        freed_bytes,
    })
}

pub fn infer_save_path_from_backup(backup_path: &Path) -> Option<PathBuf> {
    let backup_dir = backup_path.parent()?;
    let save_name = backup_dir.file_name()?;

    let backups_root = backup_dir.parent()?;

    if backups_root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("multiplayer"))
        && backups_root
            .parent()?
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case(".nwn2ee-save-editor-backups"))
    {
        let docs_root = backups_root.parent()?.parent()?;
        let inferred = docs_root.join("saves").join("multiplayer").join(save_name);
        if inferred.exists() {
            return Some(inferred);
        }
    }

    if backups_root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("singleplayer"))
        && backups_root
            .parent()?
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case(".nwn2ee-save-editor-backups"))
    {
        let docs_root = backups_root.parent()?.parent()?;
        let inferred = docs_root.join("saves").join(save_name);
        if inferred.exists() {
            return Some(inferred);
        }
    }

    let saves_dir = backups_root.parent()?.join("saves");
    if saves_dir.exists() {
        let inferred = saves_dir.join(save_name);
        if inferred.exists() {
            return Some(inferred);
        }
    }

    let sibling_save = backups_root.parent()?.join(save_name);
    if sibling_save.exists() {
        return Some(sibling_save);
    }

    None
}

fn infer_docs_root_from_save(save_dir: &Path) -> Option<PathBuf> {
    if let Some(parent) = save_dir.parent()
        && let Some(parent_name) = parent.file_name().and_then(|n| n.to_str())
    {
        if parent_name.eq_ignore_ascii_case("multiplayer")
            && let Some(saves_dir) = parent.parent()
            && saves_dir
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case("saves"))
        {
            return Some(saves_dir.parent().unwrap_or(saves_dir).to_path_buf());
        }

        if parent_name.eq_ignore_ascii_case("saves") {
            return Some(parent.parent().unwrap_or(parent).to_path_buf());
        }
    }

    None
}

fn quarantine_named_dirs(root: &Path, dir_name: &str, quarantine_root: &Path) -> SaveGameResult<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case(dir_name))
        {
            move_to_quarantine(&path, quarantine_root)?;
            continue;
        }

        quarantine_named_dirs(&path, dir_name, quarantine_root)?;
    }

    Ok(())
}

fn move_to_quarantine(path: &Path, quarantine_root: &Path) -> SaveGameResult<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::create_dir_all(quarantine_root)?;

    let mut relative_name = path
        .to_string_lossy()
        .replace(':', "")
        .replace('\\', "/")
        .trim_start_matches('/')
        .replace('/', "__");
    if relative_name.is_empty() {
        relative_name = "artifact".to_string();
    }

    let mut destination = quarantine_root.join(&relative_name);
    let mut counter = 1usize;
    while destination.exists() {
        destination = quarantine_root.join(format!("{relative_name}_{counter}"));
        counter += 1;
    }

    fs::rename(path, &destination)?;
    Ok(())
}

fn copy_directory(src: &Path, dst: &Path) -> SaveGameResult<usize> {
    fs::create_dir_all(dst)?;

    let mut count = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            count += copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
            count += 1;
        }
    }

    Ok(count)
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut size = 0u64;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                size += calculate_dir_size(&path);
            } else if let Ok(metadata) = fs::metadata(&path) {
                size += metadata.len();
            }
        }
    }

    size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_dir_structure() {
        let save_dir = PathBuf::from("/saves/mysave");
        let backup_dir = get_backup_dir_for_save(&save_dir);

        assert!(backup_dir.to_string_lossy().contains("backups"));
        assert!(backup_dir.to_string_lossy().contains("mysave"));
    }

    #[test]
    fn test_multiplayer_backup_dir_is_outside_save_tree() {
        let save_dir =
            PathBuf::from("/docs/Neverwinter Nights 2/saves/multiplayer/000040 - 01-04-2026-21-31");
        let backup_dir = get_backup_dir_for_save(&save_dir);

        assert_eq!(
            backup_dir,
            PathBuf::from(
                "/docs/Neverwinter Nights 2/.nwn2ee-save-editor-backups/multiplayer/000040 - 01-04-2026-21-31"
            )
        );
    }

    #[test]
    fn test_singleplayer_backup_dir_is_outside_save_tree() {
        let save_dir =
            PathBuf::from("/docs/Neverwinter Nights 2/saves/000003 - 22-03-2026-19-45");
        let backup_dir = get_backup_dir_for_save(&save_dir);

        assert_eq!(
            backup_dir,
            PathBuf::from(
                "/docs/Neverwinter Nights 2/.nwn2ee-save-editor-backups/singleplayer/000003 - 22-03-2026-19-45"
            )
        );
    }

    #[test]
    fn test_cleanup_legacy_save_tree_artifacts_moves_resgff_and_backups_out_of_save_tree() {
        let temp = tempfile::tempdir().expect("tempdir");
        let docs = temp.path().join("Neverwinter Nights 2");
        let save_dir = docs.join("saves").join("multiplayer").join("000041 - 04-04-2026-15-24");
        let legacy_resgff = docs
            .join("saves")
            .join("multiplayer")
            .join("000026 - 29-03-2026-21-10")
            .join("resgff");
        let legacy_backups = docs.join("saves").join("multiplayer").join("backups");

        fs::create_dir_all(&save_dir).expect("create save dir");
        fs::create_dir_all(&legacy_resgff).expect("create resgff dir");
        fs::create_dir_all(&legacy_backups).expect("create backups dir");
        fs::write(legacy_resgff.join("player.bic"), b"BIC ").expect("write bic");
        fs::write(legacy_backups.join("dummy.txt"), b"x").expect("write backup file");

        cleanup_legacy_save_tree_artifacts(&save_dir).expect("cleanup should succeed");

        assert!(
            !legacy_resgff.exists(),
            "legacy resgff dir should be moved out of save tree"
        );
        assert!(
            !legacy_backups.exists(),
            "legacy backups dir should be moved out of save tree"
        );

        let quarantine_root = docs
            .join(".nwn2ee-save-editor-quarantine")
            .join("legacy-save-artifacts");
        assert!(quarantine_root.exists(), "quarantine root should be created");

        let entries: Vec<_> = fs::read_dir(&quarantine_root)
            .expect("read quarantine")
            .map(|entry| entry.expect("entry").file_name().to_string_lossy().to_string())
            .collect();
        assert!(
            entries.iter().any(|name| name.contains("resgff")),
            "resgff artifact should be moved into quarantine"
        );
        assert!(
            entries.iter().any(|name| name.contains("backups")),
            "backups artifact should be moved into quarantine"
        );
    }

    #[test]
    fn test_backup_tracking() {
        clear_backup_tracking();

        let path = PathBuf::from("/test/save");
        assert!(!has_backup_been_created(&path));

        mark_backup_created(&path);
        assert!(has_backup_been_created(&path));

        clear_backup_tracking();
        assert!(!has_backup_been_created(&path));
    }
}
