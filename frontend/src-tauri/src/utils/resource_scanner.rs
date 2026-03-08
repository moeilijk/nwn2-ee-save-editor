use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::zip_indexer::ZipIndexer;
use super::directory_walker::DirectoryWalker;

#[derive(Error, Debug)]
pub enum ResourceScanError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Path error: {0}")]
    Path(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLocation {
    pub source_type: String,
    pub source_path: String,
    pub internal_path: Option<String>,
    pub size: u64,
    pub modified_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    pub scan_time_ms: u64,
    pub resources_found: u32,
    pub zip_files_scanned: u32,
    pub directories_scanned: u32,
    pub workshop_items_found: u32,
    pub resource_locations: HashMap<String, ResourceLocation>,
}

pub struct ResourceScanner {
    zip_indexer: ZipIndexer,
    directory_walker: DirectoryWalker,
}

impl ResourceScanner {
    pub fn new() -> Self {
        Self {
            zip_indexer: ZipIndexer::new(),
            directory_walker: DirectoryWalker::new(),
        }
    }

    pub fn scan_zip_files(&mut self, zip_paths: Vec<String>) -> Result<HashMap<String, ResourceLocation>, ResourceScanError> {
        let valid_paths: Vec<&Path> = zip_paths.iter()
            .map(Path::new)
            .filter(|p| p.exists())
            .collect();

        if valid_paths.is_empty() {
            return Ok(HashMap::new());
        }

        match self.zip_indexer.index_zips_parallel(valid_paths) {
            Ok(results) => Ok(results),
            Err(e) => {
                eprintln!("Warning: ZIP parallel scanning failed, falling back to sequential: {e}");
                let mut results = HashMap::new();
                for zip_path_str in zip_paths {
                    let zip_path = Path::new(&zip_path_str);
                    if !zip_path.exists() {
                        continue;
                    }

                    match self.zip_indexer.index_zip(zip_path) {
                        Ok(zip_resources) => {
                            results.extend(zip_resources);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to index ZIP {zip_path_str}: {e}");
                        }
                    }
                }
                Ok(results)
            }
        }
    }

    pub fn scan_workshop_directories(&mut self, workshop_dirs: Vec<String>) -> Result<HashMap<String, ResourceLocation>, ResourceScanError> {
        let mut results = HashMap::new();

        for workshop_dir_str in workshop_dirs {
            let workshop_dir = Path::new(&workshop_dir_str);

            if !workshop_dir.exists() {
                continue;
            }

            match self.directory_walker.scan_workshop_directory(workshop_dir) {
                Ok(workshop_resources) => {
                    for (resource_name, location) in workshop_resources {
                        results.insert(resource_name, location);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to scan workshop directory {workshop_dir_str}: {e}");
                }
            }
        }

        Ok(results)
    }

    pub fn index_directory(&mut self, directory_path: String, recursive: Option<bool>) -> Result<HashMap<String, ResourceLocation>, ResourceScanError> {
        let dir_path = Path::new(&directory_path);
        let is_recursive = recursive.unwrap_or(true);

        if !dir_path.exists() {
            return Ok(HashMap::new());
        }

        match self.directory_walker.index_directory(dir_path, is_recursive) {
            Ok(resources) => Ok(resources),
            Err(e) => {
                eprintln!("Warning: Failed to index directory {directory_path}: {e}");
                Ok(HashMap::new())
            }
        }
    }

    pub fn comprehensive_scan(
        &mut self,
        nwn2_data_dir: String,
        workshop_dirs: Vec<String>,
        custom_override_dirs: Vec<String>,
        enhanced_data_dir: Option<String>,
    ) -> Result<ScanResults, ResourceScanError> {
        let start_time = Instant::now();
        let mut all_resources = HashMap::new();
        let mut zip_files_scanned = 0;
        let mut directories_scanned = 0;
        let mut workshop_items_found = 0;

        let zip_files = vec!["2da.zip", "2da_x1.zip", "2da_x2.zip"];
        let mut zip_paths = Vec::new();

        let data_dir = Path::new(&nwn2_data_dir);
        if data_dir.exists() {
            for zip_name in &zip_files {
                let zip_path = data_dir.join(zip_name);
                if zip_path.exists() {
                    zip_paths.push(zip_path.to_string_lossy().to_string());
                }
            }
        }

        if let Some(enhanced_dir) = enhanced_data_dir {
            let enhanced_path = Path::new(&enhanced_dir);
            if enhanced_path.exists() {
                for zip_name in &zip_files {
                    let zip_path = enhanced_path.join(zip_name);
                    if zip_path.exists() {
                        zip_paths.push(zip_path.to_string_lossy().to_string());
                    }
                }
            }
        }

        match self.scan_zip_files(zip_paths.clone()) {
            Ok(zip_resources) => {
                zip_files_scanned = zip_paths.len() as u32;
                for (name, location) in zip_resources {
                    all_resources.insert(name, location);
                }
            }
            Err(e) => {
                eprintln!("Warning: ZIP scanning failed: {e}");
            }
        }

        match self.scan_workshop_directories(workshop_dirs.clone()) {
            Ok(workshop_resources) => {
                workshop_items_found = workshop_resources.len() as u32;
                for (name, location) in workshop_resources {
                    all_resources.insert(name, location);
                }
            }
            Err(e) => {
                eprintln!("Warning: Workshop scanning failed: {e}");
            }
        }

        for override_dir in custom_override_dirs {
            match self.index_directory(override_dir, Some(true)) {
                Ok(override_resources) => {
                    directories_scanned += 1;
                    for (name, location) in override_resources {
                        all_resources.insert(name, location);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Override directory scanning failed: {e}");
                }
            }
        }

        let scan_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ScanResults {
            scan_time_ms,
            resources_found: all_resources.len() as u32,
            zip_files_scanned,
            directories_scanned,
            workshop_items_found,
            resource_locations: all_resources,
        })
    }

    pub fn get_performance_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();

        stats.extend(self.zip_indexer.get_stats());
        stats.extend(self.directory_walker.get_stats());

        stats
    }
}

impl Default for ResourceScanner {
    fn default() -> Self {
        Self::new()
    }
}
