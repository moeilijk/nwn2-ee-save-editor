use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::parsers::tda::TDAParser;

use super::override_chain::ModuleInfo;

const DEFAULT_MAX_MODULES: usize = 5;

#[derive(Debug, Clone)]
pub struct CachedModuleState {
    pub module_info: ModuleInfo,
    pub hak_overrides: Vec<HashMap<String, Arc<TDAParser>>>,
    pub module_overrides: HashMap<String, Arc<TDAParser>>,
    pub campaign_overrides: HashMap<String, Arc<TDAParser>>,
    pub custom_tlk_path: Option<PathBuf>,
    pub last_accessed: u64,
}

pub struct ModuleLRUCache {
    cache: IndexMap<String, CachedModuleState>,
    max_size: usize,
    hits: u64,
    misses: u64,
}

impl ModuleLRUCache {
    pub fn new() -> Self {
        Self::with_max_size(DEFAULT_MAX_MODULES)
    }

    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            cache: IndexMap::with_capacity(max_size),
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&CachedModuleState> {
        if self.cache.contains_key(key) {
            self.hits += 1;
            if let Some(entry) = self.cache.get_mut(key) {
                entry.last_accessed = current_timestamp();
            }
            self.cache
                .move_index(self.cache.get_index_of(key).unwrap(), self.cache.len() - 1);
            self.cache.get(key)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut CachedModuleState> {
        if self.cache.contains_key(key) {
            self.hits += 1;
            let idx = self.cache.get_index_of(key).unwrap();
            self.cache.move_index(idx, self.cache.len() - 1);
            if let Some(entry) = self.cache.get_mut(key) {
                entry.last_accessed = current_timestamp();
            }
            self.cache.get_mut(key)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn put(&mut self, key: String, value: CachedModuleState) {
        if self.cache.contains_key(&key) {
            self.cache.shift_remove(&key);
        }

        while self.cache.len() >= self.max_size {
            self.cache.shift_remove_index(0);
        }

        self.cache.insert(key, value);
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<CachedModuleState> {
        self.cache.shift_remove(key)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            hits: self.hits,
            misses: self.misses,
            hit_ratio: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for ModuleLRUCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_ratio: f64,
}

#[derive(Debug, Default)]
pub struct FileModificationTracker {
    mod_times: HashMap<PathBuf, f64>,
}

impl FileModificationTracker {
    pub fn new() -> Self {
        Self {
            mod_times: HashMap::new(),
        }
    }

    pub fn track(&mut self, path: PathBuf, mtime: f64) {
        self.mod_times.insert(path, mtime);
    }

    pub fn is_modified(&self, path: &PathBuf) -> bool {
        if let Some(&cached_mtime) = self.mod_times.get(path)
            && let Ok(current_mtime) = get_file_mtime(path)
        {
            return (current_mtime - cached_mtime).abs() > 0.001;
        }
        true
    }

    pub fn update(&mut self, path: &PathBuf) -> bool {
        if let Ok(mtime) = get_file_mtime(path) {
            let was_modified = self.is_modified(path);
            self.mod_times.insert(path.clone(), mtime);
            was_modified
        } else {
            false
        }
    }

    pub fn check_all(&self) -> Vec<PathBuf> {
        self.mod_times
            .keys()
            .filter(|path| self.is_modified(path))
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.mod_times.clear();
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.mod_times.remove(path);
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn get_file_mtime(path: &PathBuf) -> Result<f64, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    let mtime = metadata.modified()?;
    Ok(mtime
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = ModuleLRUCache::with_max_size(3);

        let state = CachedModuleState {
            module_info: ModuleInfo::default(),
            hak_overrides: Vec::new(),
            module_overrides: HashMap::new(),
            campaign_overrides: HashMap::new(),
            custom_tlk_path: None,
            last_accessed: 0,
        };

        cache.put("mod1".to_string(), state.clone());
        cache.put("mod2".to_string(), state.clone());
        cache.put("mod3".to_string(), state.clone());

        assert_eq!(cache.len(), 3);
        assert!(cache.contains("mod1"));
        assert!(cache.contains("mod2"));
        assert!(cache.contains("mod3"));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = ModuleLRUCache::with_max_size(2);

        let state = CachedModuleState {
            module_info: ModuleInfo::default(),
            hak_overrides: Vec::new(),
            module_overrides: HashMap::new(),
            campaign_overrides: HashMap::new(),
            custom_tlk_path: None,
            last_accessed: 0,
        };

        cache.put("mod1".to_string(), state.clone());
        cache.put("mod2".to_string(), state.clone());
        cache.put("mod3".to_string(), state.clone());

        assert_eq!(cache.len(), 2);
        assert!(!cache.contains("mod1"));
        assert!(cache.contains("mod2"));
        assert!(cache.contains("mod3"));
    }

    #[test]
    fn test_lru_cache_access_order() {
        let mut cache = ModuleLRUCache::with_max_size(2);

        let state = CachedModuleState {
            module_info: ModuleInfo::default(),
            hak_overrides: Vec::new(),
            module_overrides: HashMap::new(),
            campaign_overrides: HashMap::new(),
            custom_tlk_path: None,
            last_accessed: 0,
        };

        cache.put("mod1".to_string(), state.clone());
        cache.put("mod2".to_string(), state.clone());

        let _ = cache.get("mod1");

        cache.put("mod3".to_string(), state.clone());

        assert!(cache.contains("mod1"));
        assert!(!cache.contains("mod2"));
        assert!(cache.contains("mod3"));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ModuleLRUCache::with_max_size(2);

        let state = CachedModuleState {
            module_info: ModuleInfo::default(),
            hak_overrides: Vec::new(),
            module_overrides: HashMap::new(),
            campaign_overrides: HashMap::new(),
            custom_tlk_path: None,
            last_accessed: 0,
        };

        cache.put("mod1".to_string(), state);
        let _ = cache.get("mod1");
        let _ = cache.get("mod2");

        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_ratio - 0.5).abs() < 0.01);
    }
}
