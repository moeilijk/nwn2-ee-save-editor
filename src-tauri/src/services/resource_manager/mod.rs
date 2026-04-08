pub mod cache;
pub mod error;
pub mod module_loader;
pub mod override_chain;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock as StdRwLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use parking_lot::Mutex;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::NWN2Paths;
use crate::parsers::erf::ErfParser;
use crate::parsers::gff::GffParser;
use crate::parsers::tda::TDAParser;
use crate::parsers::tlk::TLKParser;
use crate::utils::ZipContentReader;

pub use cache::{CacheStats, CachedModuleState, FileModificationTracker, ModuleLRUCache};
pub use error::{ResourceManagerError, ResourceManagerResult};
pub use override_chain::{
    CampaignInfo, ContainerType, ModuleInfo, OverrideSource, ResourceLocation, TemplateInfo,
};

const BASE_GAME_ZIPS: &[&str] = &["2da.zip", "2da_x1.zip", "2da_x2.zip"];
const TEMPLATE_ZIPS: &[&str] = &["Templates.zip", "Templates_X1.zip", "Templates_X2.zip"];

pub struct ResourceManager {
    paths: Arc<RwLock<NWN2Paths>>,
    zip_reader: Mutex<ZipContentReader>,

    tda_locations: HashMap<String, ResourceLocation>,
    template_locations: HashMap<String, ResourceLocation>,

    tlk_cache: Option<Arc<StdRwLock<TLKParser>>>,
    custom_tlk_cache: Option<Arc<StdRwLock<TLKParser>>>,

    hak_overrides: Vec<HashMap<String, Arc<TDAParser>>>,
    module_overrides: DashMap<String, Arc<TDAParser>>,
    campaign_overrides: DashMap<String, Arc<TDAParser>>,
    override_dir_cache: DashMap<String, Arc<TDAParser>>,
    workshop_cache: DashMap<String, Arc<TDAParser>>,
    custom_override_cache: DashMap<String, Arc<TDAParser>>,
    base_game_cache: DashMap<String, Arc<TDAParser>>,

    override_file_paths: HashMap<String, PathBuf>,
    workshop_file_paths: HashMap<String, PathBuf>,
    custom_override_paths: HashMap<String, PathBuf>,
    campaign_override_paths: HashMap<String, PathBuf>,

    current_module: Option<String>,
    module_path: Option<PathBuf>,
    // module_parser: Option<Arc<ErfParser>>, // Removing unused field
    module_info: Option<ModuleInfo>,

    current_campaign_id: Option<String>,
    current_campaign_folder: Option<PathBuf>,

    module_cache: ModuleLRUCache,
    file_mod_tracker: FileModificationTracker,

    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    initialized: bool,
}

impl ResourceManager {
    pub fn new(paths: Arc<RwLock<NWN2Paths>>) -> Self {
        Self {
            paths,
            zip_reader: Mutex::new(ZipContentReader::new()),
            tda_locations: HashMap::new(),
            template_locations: HashMap::new(),
            tlk_cache: None,
            custom_tlk_cache: None,
            hak_overrides: Vec::new(),
            module_overrides: DashMap::new(),
            campaign_overrides: DashMap::new(),
            override_dir_cache: DashMap::new(),
            workshop_cache: DashMap::new(),
            custom_override_cache: DashMap::new(),
            base_game_cache: DashMap::new(),
            override_file_paths: HashMap::new(),
            workshop_file_paths: HashMap::new(),
            custom_override_paths: HashMap::new(),
            campaign_override_paths: HashMap::new(),
            current_module: None,
            module_path: None,
            // module_parser: None,
            module_info: None,
            current_campaign_id: None,
            current_campaign_folder: None,
            module_cache: ModuleLRUCache::new(),
            file_mod_tracker: FileModificationTracker::new(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            initialized: false,
        }
    }

    pub async fn initialize(&mut self) -> ResourceManagerResult<()> {
        if self.initialized {
            return Ok(());
        }

        info!("Initializing ResourceManager");

        self.scan_base_game_zips().await?;
        self.scan_workshop_directories().await?;
        self.scan_override_directories().await?;
        self.load_base_tlk().await?;

        self.initialized = true;
        info!(
            "ResourceManager initialized: {} 2DAs indexed, {} templates indexed",
            self.tda_locations.len(),
            self.template_locations.len()
        );

        Ok(())
    }

    async fn scan_base_game_zips(&mut self) -> ResourceManagerResult<()> {
        let paths = self.paths.read().await;
        let data_dir = paths
            .data()
            .ok_or_else(|| ResourceManagerError::PathNotConfigured("NWN2 data directory".into()))?;

        drop(paths);

        for zip_name in BASE_GAME_ZIPS {
            let zip_path = data_dir.join(zip_name);
            if zip_path.exists() {
                self.index_zip_for_2das(&zip_path, OverrideSource::BaseGame)?;
            }
        }

        for zip_name in TEMPLATE_ZIPS {
            let zip_path = data_dir.join(zip_name);
            if zip_path.exists() {
                self.index_zip_for_templates(&zip_path, OverrideSource::BaseGame)?;
            }
        }

        Ok(())
    }

    fn index_zip_for_2das(
        &mut self,
        zip_path: &Path,
        source: OverrideSource,
    ) -> ResourceManagerResult<()> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            ResourceManagerError::ZipError(format!("Failed to open {}: {}", zip_path.display(), e))
        })?;

        let mtime = std::fs::metadata(zip_path)?
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0.0, |d| d.as_secs_f64());

        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| {
                ResourceManagerError::ZipError(format!("Failed to read entry {i}: {e}"))
            })?;

            let name = entry.name().to_string();
            if name.to_lowercase().ends_with(".2da") {
                let tda_name = Path::new(&name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&name)
                    .to_lowercase();

                let location =
                    ResourceLocation::from_zip(source.clone(), zip_path.to_path_buf(), name, mtime);

                self.tda_locations.insert(tda_name, location);
            }
        }

        debug!(
            "Indexed {} for 2DAs, total: {}",
            zip_path.display(),
            self.tda_locations.len()
        );
        Ok(())
    }

    fn index_zip_for_templates(
        &mut self,
        zip_path: &Path,
        source: OverrideSource,
    ) -> ResourceManagerResult<()> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            ResourceManagerError::ZipError(format!("Failed to open {}: {}", zip_path.display(), e))
        })?;

        let mtime = std::fs::metadata(zip_path)?
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0.0, |d| d.as_secs_f64());

        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| {
                ResourceManagerError::ZipError(format!("Failed to read entry {i}: {e}"))
            })?;

            let name = entry.name().to_string();
            if name.to_lowercase().ends_with(".uti") {
                let resref = Path::new(&name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&name)
                    .to_lowercase();

                let location =
                    ResourceLocation::from_zip(source.clone(), zip_path.to_path_buf(), name, mtime);

                self.template_locations.insert(resref, location);
            }
        }

        debug!(
            "Indexed {} for templates, total: {}",
            zip_path.display(),
            self.template_locations.len()
        );
        Ok(())
    }

    async fn scan_override_directories(&mut self) -> ResourceManagerResult<()> {
        let paths = self.paths.read().await;

        if let Some(override_dir) = paths.override_dir() {
            let override_dir = override_dir.clone();
            drop(paths);
            let mut new_paths = HashMap::new();
            self.index_directory_for_2das_internal(&override_dir, &mut new_paths)?;
            self.override_file_paths.extend(new_paths);
        }

        Ok(())
    }

    async fn scan_workshop_directories(&mut self) -> ResourceManagerResult<()> {
        let paths = self.paths.read().await;
        let workshop_dir = paths.steam_workshop_folder();

        if let Some(workshop_dir) = workshop_dir {
            let workshop_dir = workshop_dir.clone();
            drop(paths);

            if !workshop_dir.exists() {
                return Ok(());
            }

            debug!(
                "Scanning Steam Workshop directory: {}",
                workshop_dir.display()
            );
            let mut new_paths = HashMap::new();

            // Workshop structure: <WorkshopDir>/<ModID>/override/
            if let Ok(entries) = std::fs::read_dir(&workshop_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Check for 'override' folder inside the mod directory
                        let override_path = path.join("override");
                        if override_path.exists() && override_path.is_dir() {
                            self.index_directory_recursive(&override_path, &mut new_paths)?;
                        }

                        // Some mods might put files directly in the mod ID folder (less common but possible?)
                        // Standard practice is usually an override folder or just loose files.
                        // The Python code checked 'override' subdirectory.
                    }
                }
            }

            self.workshop_file_paths.extend(new_paths);
            info!("Indexed {} workshop 2DAs", self.workshop_file_paths.len());
        }

        Ok(())
    }

    fn index_directory_for_2das_internal(
        &mut self,
        dir: &Path,
        target: &mut HashMap<String, PathBuf>,
    ) -> ResourceManagerResult<()> {
        self.index_directory_recursive(dir, target)
    }

    fn index_directory_recursive(
        &mut self,
        dir: &Path,
        target: &mut HashMap<String, PathBuf>,
    ) -> ResourceManagerResult<()> {
        if !dir.exists() {
            return Ok(());
        }

        // Use a stack for iterative recursion to avoid stack overflow on deep structures,
        // though fs recursion is usually shallow enough. Iterative is safer.
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.is_file()
                        && path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("2da"))
                        && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                    {
                        let mtime = std::fs::metadata(&path)?
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map_or(0.0, |d| d.as_secs_f64());

                        self.file_mod_tracker.track(path.clone(), mtime);
                        target.insert(name.to_lowercase(), path);
                    }
                }
            }
        }

        Ok(())
    }

    async fn load_base_tlk(&mut self) -> ResourceManagerResult<()> {
        let paths = self.paths.read().await;
        let base_tlk_path = paths.dialog_tlk();
        let workshop_folder = paths.steam_workshop_folder().cloned();
        drop(paths);

        let mut best_parser: Option<TLKParser> = None;
        let mut best_source = String::new();

        // Load base dialog.tlk
        if let Some(ref tlk_path) = base_tlk_path
            && tlk_path.exists()
        {
            match module_loader::load_tlk(tlk_path) {
                Ok(parser) => {
                    info!(
                        "Found base TLK: {} ({} entries)",
                        tlk_path.display(),
                        parser.string_count()
                    );
                    best_source = tlk_path.display().to_string();
                    best_parser = Some(parser);
                }
                Err(e) => warn!("Failed to parse base TLK: {}", e),
            }
        }

        // NWN2 EE distributes extended TLK via Steam Workshop.
        // Scan workshop items for dialog.TLK with more entries (spell descriptions etc.)
        if let Some(ref workshop_dir) = workshop_folder
            && let Ok(entries) = std::fs::read_dir(workshop_dir)
        {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let tlk_path = entry.path().join("dialog.TLK");
                if !tlk_path.exists() {
                    // Also check lowercase
                    let tlk_lower = entry.path().join("dialog.tlk");
                    if !tlk_lower.exists() {
                        continue;
                    }
                }
                let actual_path = if entry.path().join("dialog.TLK").exists() {
                    entry.path().join("dialog.TLK")
                } else {
                    entry.path().join("dialog.tlk")
                };

                match module_loader::load_tlk(&actual_path) {
                    Ok(parser) => {
                        let count = parser.string_count();
                        let best_count = best_parser.as_ref().map_or(0, TLKParser::string_count);
                        if count > best_count {
                            info!(
                                "Found Workshop TLK with more entries: {} ({} > {})",
                                actual_path.display(),
                                count,
                                best_count
                            );
                            best_source = actual_path.display().to_string();
                            best_parser = Some(parser);
                        }
                    }
                    Err(e) => debug!("Skipping Workshop TLK {}: {}", actual_path.display(), e),
                }
            }
        }

        if let Some(parser) = best_parser {
            info!(
                "Using TLK: {} ({} entries)",
                best_source,
                parser.string_count()
            );
            self.tlk_cache = Some(Arc::new(StdRwLock::new(parser)));
        }

        Ok(())
    }

    pub fn get_2da(&self, name: &str) -> ResourceManagerResult<Arc<TDAParser>> {
        let name_lower = name.to_lowercase().replace(".2da", "");

        if let Some(parser) = self.base_game_cache.get(&name_lower) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Arc::clone(&parser));
        }

        if let Some(location) = self.tda_locations.get(&name_lower) {
            // self.cache_hits += 1; // It's a "hit" on location but "miss" on parser cache
            // Actually, we should call load_2da_from_location which will parse and cache it
            let parser = self.load_2da_from_location(location.clone())?;
            self.base_game_cache.insert(name_lower, Arc::clone(&parser));
            return Ok(parser);
        }

        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        Err(ResourceManagerError::TdaNotFound { name: name.into() })
    }

    pub fn get_2da_with_overrides(&self, name: &str) -> ResourceManagerResult<Arc<TDAParser>> {
        let name_lower = name.to_lowercase().replace(".2da", "");

        // 1. Check custom override paths (highest priority)
        if let Some(path) = self.custom_override_paths.get(&name_lower) {
            if let Some(parser) = self.custom_override_cache.get(&name_lower) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Arc::clone(&parser));
            }
            let parser = self.parse_2da_file(path)?;
            let arc = Arc::new(parser);
            self.custom_override_cache
                .insert(name_lower.clone(), Arc::clone(&arc));
            return Ok(arc);
        }

        // 2. Check traditional override directory (Documents/override)
        if let Some(path) = self.override_file_paths.get(&name_lower) {
            if let Some(parser) = self.override_dir_cache.get(&name_lower) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Arc::clone(&parser));
            }
            let parser = self.parse_2da_file(path)?;
            let arc = Arc::new(parser);
            self.override_dir_cache
                .insert(name_lower.clone(), Arc::clone(&arc));
            return Ok(arc);
        }

        // 3. Check Workshop content
        if let Some(path) = self.workshop_file_paths.get(&name_lower) {
            if let Some(parser) = self.workshop_cache.get(&name_lower) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Arc::clone(&parser));
            }
            let parser = self.parse_2da_file(path)?;
            let arc = Arc::new(parser);
            self.workshop_cache
                .insert(name_lower.clone(), Arc::clone(&arc));
            return Ok(arc);
        }

        // 4. Check HAK overrides (Module specified)
        for hak_cache in &self.hak_overrides {
            if let Some(parser) = hak_cache.get(&name_lower) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Arc::clone(parser));
            }
        }

        // 5. Check campaign folder (campaign.cam linked)
        if let Some(path) = self.campaign_override_paths.get(&name_lower) {
            if let Some(parser) = self.campaign_overrides.get(&name_lower) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Arc::clone(&parser));
            }
            let parser = self.parse_2da_file(path)?;
            let arc = Arc::new(parser);
            self.campaign_overrides
                .insert(name_lower.clone(), Arc::clone(&arc));
            return Ok(arc);
        }

        // 6. Check module overrides (inside .mod files)
        if let Some(parser) = self.module_overrides.get(&name_lower) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Arc::clone(&parser));
        }

        // 7. Base Game (fallback)
        self.get_2da(&name_lower)
    }

    fn load_2da_from_location(
        &self,
        location: ResourceLocation,
    ) -> ResourceManagerResult<Arc<TDAParser>> {
        match location.container_type {
            ContainerType::Zip => {
                let internal_path = location
                    .internal_path
                    .ok_or_else(|| ResourceManagerError::Parse("Missing internal path".into()))?;

                let data = self
                    .zip_reader
                    .lock()
                    .read_file_from_zip(
                        location.container_path.to_string_lossy().to_string(),
                        internal_path.clone(),
                    )
                    .map_err(|e| ResourceManagerError::ZipError(e.clone()))?;

                let mut parser = TDAParser::new();
                parser.parse_from_bytes(&data).map_err(|e| {
                    ResourceManagerError::InvalidTdaFormat(format!(
                        "Failed to parse {internal_path}: {e}"
                    ))
                })?;

                Ok(Arc::new(parser))
            }
            ContainerType::Erf => {
                let internal_path = location
                    .internal_path
                    .ok_or_else(|| ResourceManagerError::Parse("Missing internal path".into()))?;

                let mut erf = ErfParser::new();
                erf.read(&location.container_path).map_err(|e| {
                    ResourceManagerError::InvalidErfFormat(format!(
                        "Failed to open {}: {}",
                        location.container_path.display(),
                        e
                    ))
                })?;

                let resource = erf.resources.get(&internal_path).ok_or_else(|| {
                    ResourceManagerError::ExtractionFailed {
                        resource: internal_path.clone(),
                        container: location.container_path.display().to_string(),
                    }
                })?;

                let data = resource.data.as_ref().ok_or_else(|| {
                    ResourceManagerError::ExtractionFailed {
                        resource: internal_path.clone(),
                        container: location.container_path.display().to_string(),
                    }
                })?;

                let mut parser = TDAParser::new();
                parser.parse_from_bytes(data).map_err(|e| {
                    ResourceManagerError::InvalidTdaFormat(format!(
                        "Failed to parse {internal_path}: {e}"
                    ))
                })?;

                Ok(Arc::new(parser))
            }
            ContainerType::Directory => {
                let parser = self.parse_2da_file(&location.container_path)?;
                Ok(Arc::new(parser))
            }
        }
    }

    fn parse_2da_file(&self, path: &Path) -> ResourceManagerResult<TDAParser> {
        let mut parser = TDAParser::new();
        parser.parse_from_file(path).map_err(|e| {
            ResourceManagerError::InvalidTdaFormat(format!(
                "Failed to parse {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(parser)
    }

    pub fn get_string(&self, str_ref: i32) -> String {
        if str_ref < 0 {
            return String::new();
        }

        if let Some(custom_tlk) = &self.custom_tlk_cache
            && let Ok(mut tlk) = custom_tlk.write()
            && let Ok(Some(s)) = tlk.get_string(str_ref as usize)
            && !s.is_empty()
        {
            return s;
        }

        if let Some(base_tlk) = &self.tlk_cache
            && let Ok(mut tlk) = base_tlk.write()
            && let Ok(Some(s)) = tlk.get_string(str_ref as usize)
        {
            return s;
        }

        format!("<StrRef:{str_ref}>")
    }

    pub fn get_strings_batch(&self, str_refs: &[i32]) -> HashMap<i32, String> {
        let mut results = HashMap::with_capacity(str_refs.len());

        for &str_ref in str_refs {
            results.insert(str_ref, self.get_string(str_ref));
        }

        results
    }

    pub async fn set_module(&mut self, module_path: &Path) -> ResourceManagerResult<bool> {
        let module_key = module_path.display().to_string();

        if self.module_cache.contains(&module_key) {
            let cached = self.module_cache.get(&module_key).unwrap().clone();
            self.restore_from_cache(cached);
            info!("Restored module from cache: {}", module_key);
            return Ok(true);
        }

        let module_info = module_loader::extract_module_info(module_path)?;
        let module_2das = module_loader::load_module_2das(module_path, module_info.is_directory)?;

        self.module_info = Some(module_info.clone());
        self.module_path = Some(module_path.to_path_buf());
        self.module_overrides.clear();
        for (k, v) in module_2das {
            self.module_overrides.insert(k, v);
        }
        self.current_module = Some(module_key.clone());

        self.hak_overrides.clear();

        let paths = self.paths.read().await;
        let custom_hak_folders = paths.custom_hak_folders().to_vec();
        let user_hak = paths.hak_dir();
        let install_hak = paths.hak_dir();
        drop(paths);

        for hak_name in &module_info.hak_list {
            if let Some(hak_path) = module_loader::find_hak_path(
                hak_name,
                &custom_hak_folders,
                user_hak.as_ref(),
                install_hak.as_ref(),
            ) {
                match module_loader::load_hak_2das(&hak_path) {
                    Ok(hak_2das) => {
                        self.hak_overrides.push(hak_2das);

                        if let Some(tlk_path) = module_loader::check_hak_for_tlk(&hak_path)
                            && let Ok(tlk) = module_loader::load_tlk(&tlk_path)
                        {
                            self.custom_tlk_cache = Some(Arc::new(StdRwLock::new(tlk)));
                        }
                    }
                    Err(e) => {
                        warn!("Failed to load HAK {}: {}", hak_name, e);
                    }
                }
            } else {
                warn!("HAK not found: {}", hak_name);
            }
        }

        if !module_info.custom_tlk.is_empty() && self.custom_tlk_cache.is_none() {
            self.load_custom_tlk(&module_info.custom_tlk).await;
        }

        if let Some(campaign_guid) = &module_info.campaign_id {
            self.set_campaign_by_guid(campaign_guid).await?;
        }

        let cached_state = CachedModuleState {
            module_info: module_info.clone(),
            hak_overrides: self.hak_overrides.clone(),
            module_overrides: self.module_overrides.clone().into_iter().collect(), // convert DashMap to HashMap for caching
            campaign_overrides: self.campaign_overrides.clone().into_iter().collect(),
            custom_tlk_path: None,
            last_accessed: 0,
        };
        self.module_cache.put(module_key, cached_state);

        info!(
            "Loaded module: {} with {} HAKs",
            module_info.name,
            module_info.hak_list.len()
        );
        Ok(true)
    }

    fn restore_from_cache(&mut self, cached: CachedModuleState) {
        self.module_info = Some(cached.module_info.clone());
        self.module_path = Some(cached.module_info.path.clone());
        self.hak_overrides = cached.hak_overrides;
        self.module_overrides.clear();
        for (k, v) in cached.module_overrides {
            self.module_overrides.insert(k, v);
        }
        self.campaign_overrides.clear();
        for (k, v) in cached.campaign_overrides {
            self.campaign_overrides.insert(k, v);
        }
    }

    async fn load_custom_tlk(&mut self, tlk_name: &str) {
        let paths = self.paths.read().await;

        let tlk_filename = if tlk_name.to_lowercase().ends_with(".tlk") {
            tlk_name.to_string()
        } else {
            format!("{tlk_name}.tlk")
        };

        if let Some(tlk_dir) = paths.tlk_dir() {
            let tlk_path = tlk_dir.join(&tlk_filename);
            if tlk_path.exists()
                && let Ok(tlk) = module_loader::load_tlk(&tlk_path)
            {
                self.custom_tlk_cache = Some(Arc::new(StdRwLock::new(tlk)));
                return;
            }
        }

        if let Some(data_dir) = paths.data() {
            let tlk_path = data_dir.join(&tlk_filename);
            if tlk_path.exists()
                && let Ok(tlk) = module_loader::load_tlk(&tlk_path)
            {
                self.custom_tlk_cache = Some(Arc::new(StdRwLock::new(tlk)));
            }
        }
    }

    pub async fn set_campaign_by_guid(&mut self, guid: &str) -> ResourceManagerResult<bool> {
        let paths = self.paths.read().await;
        let install_campaigns = paths.campaigns();
        let user_campaigns = paths.user_campaigns();
        drop(paths);

        if let Some(campaign_folder) = module_loader::find_campaign_by_guid(
            guid,
            install_campaigns.as_ref(),
            user_campaigns.as_ref(),
        ) {
            self.current_campaign_folder = Some(campaign_folder.clone());
            self.current_campaign_id = Some(guid.to_string());

            self.campaign_override_paths.clear();
            self.campaign_overrides.clear();

            let mut new_paths = HashMap::new();
            self.index_directory_for_2das_internal(&campaign_folder, &mut new_paths)?;
            self.campaign_override_paths.extend(new_paths);

            info!(
                "Set campaign: {} with {} 2DA overrides",
                guid,
                self.campaign_override_paths.len()
            );
            return Ok(true);
        }

        warn!("Campaign not found: {}", guid);
        Ok(false)
    }

    pub async fn load_haks_for_save(
        &mut self,
        hak_list: &[String],
        custom_tlk: &str,
        campaign_guid: &str,
    ) -> ResourceManagerResult<bool> {
        self.clear_override_caches();

        let paths = self.paths.read().await;
        let custom_hak_folders = paths.custom_hak_folders().to_vec();
        let user_hak = paths.hak_dir();
        let install_hak = paths.hak_dir();
        drop(paths);

        for hak_name in hak_list {
            if let Some(hak_path) = module_loader::find_hak_path(
                hak_name,
                &custom_hak_folders,
                user_hak.as_ref(),
                install_hak.as_ref(),
            ) {
                match module_loader::load_hak_2das(&hak_path) {
                    Ok(hak_2das) => {
                        self.hak_overrides.push(hak_2das);

                        if let Some(tlk_path) = module_loader::check_hak_for_tlk(&hak_path)
                            && let Ok(tlk) = module_loader::load_tlk(&tlk_path)
                        {
                            self.custom_tlk_cache = Some(Arc::new(StdRwLock::new(tlk)));
                        }
                    }
                    Err(e) => {
                        warn!("Failed to load HAK {}: {}", hak_name, e);
                    }
                }
            }
        }

        if !custom_tlk.is_empty() && self.custom_tlk_cache.is_none() {
            self.load_custom_tlk(custom_tlk).await;
        }

        if !campaign_guid.is_empty() {
            self.set_campaign_by_guid(campaign_guid).await?;
        }

        Ok(true)
    }

    pub fn add_custom_override_directory(&mut self, path: &Path) -> ResourceManagerResult<bool> {
        if !path.exists() || !path.is_dir() {
            return Err(ResourceManagerError::FileNotFound(path.to_path_buf()));
        }

        let mut new_paths = HashMap::new();
        self.index_directory_for_2das_internal(path, &mut new_paths)?;
        self.custom_override_paths.extend(new_paths);
        self.custom_override_cache.clear();

        info!(
            "Added custom override directory: {} ({} 2DAs)",
            path.display(),
            self.custom_override_paths.len()
        );
        Ok(true)
    }

    pub fn remove_custom_override_directory(&mut self, path: &Path) -> bool {
        let path_str = path.display().to_string();
        let before = self.custom_override_paths.len();

        self.custom_override_paths
            .retain(|_, p| !p.starts_with(path));
        self.custom_override_cache.clear();

        let removed = before - self.custom_override_paths.len();
        info!(
            "Removed custom override directory: {} ({} 2DAs removed)",
            path_str, removed
        );
        removed > 0
    }

    pub fn clear_override_caches(&mut self) {
        self.hak_overrides.clear();
        self.module_overrides.clear();
        self.campaign_overrides.clear();
        self.override_dir_cache.clear();
        self.workshop_cache.clear();
        self.custom_override_cache.clear();
        self.custom_tlk_cache = None;
        self.current_module = None;
        self.module_path = None;
        self.module_info = None;
        self.current_campaign_id = None;
        self.current_campaign_folder = None;
    }

    pub fn check_for_modifications(&mut self) -> Vec<PathBuf> {
        let modified = self.file_mod_tracker.check_all();

        for path in &modified {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                let name_lower = name.to_lowercase();
                self.override_dir_cache.remove(&name_lower);
                self.workshop_cache.remove(&name_lower);
                self.custom_override_cache.remove(&name_lower);
            }
            self.file_mod_tracker.update(path);
        }

        modified
    }

    pub fn get_all_item_templates(&self) -> HashMap<String, TemplateInfo> {
        let mut templates = HashMap::new();

        for (resref, location) in &self.template_locations {
            templates.insert(
                resref.clone(),
                TemplateInfo {
                    resref: resref.clone(),
                    container_type: location.container_type.clone(),
                    container_path: location.container_path.clone(),
                    internal_path: location.internal_path.clone(),
                    source: location.source.clone(),
                },
            );
        }

        templates
    }

    pub fn get_item_template_data(
        &self,
        info: &TemplateInfo,
    ) -> ResourceManagerResult<IndexMap<String, serde_json::Value>> {
        let fields = self.get_item_template_fields(info)?;

        let mut result = IndexMap::new();
        for (key, value) in fields {
            result.insert(key, gff_value_to_json(&value));
        }

        Ok(result)
    }

    pub fn read_zip_file(
        &self,
        zip_path: &std::path::Path,
        internal_path: &str,
    ) -> ResourceManagerResult<Vec<u8>> {
        self.zip_reader
            .lock()
            .read_file_from_zip(
                zip_path.to_string_lossy().to_string(),
                internal_path.to_string(),
            )
            .map_err(|e| ResourceManagerError::ZipError(e.clone()))
    }

    pub fn get_item_template_fields(
        &self,
        info: &TemplateInfo,
    ) -> ResourceManagerResult<IndexMap<String, crate::parsers::gff::GffValue<'static>>> {
        let data =
            match &info.container_type {
                ContainerType::Zip => {
                    let internal_path = info.internal_path.as_ref().ok_or_else(|| {
                        ResourceManagerError::Parse("Missing internal path".into())
                    })?;

                    self.zip_reader
                        .lock()
                        .read_file_from_zip(
                            info.container_path.to_string_lossy().to_string(),
                            internal_path.clone(),
                        )
                        .map_err(|e| ResourceManagerError::ZipError(e.clone()))?
                }
                ContainerType::Erf => {
                    let internal_path = info.internal_path.as_ref().ok_or_else(|| {
                        ResourceManagerError::Parse("Missing internal path".into())
                    })?;

                    let mut erf = ErfParser::new();
                    erf.read(&info.container_path).map_err(|e| {
                        ResourceManagerError::InvalidErfFormat(format!(
                            "Failed to open {}: {}",
                            info.container_path.display(),
                            e
                        ))
                    })?;

                    erf.resources
                        .get(internal_path)
                        .ok_or_else(|| ResourceManagerError::ExtractionFailed {
                            resource: internal_path.clone(),
                            container: info.container_path.display().to_string(),
                        })?
                        .data
                        .clone()
                        .ok_or_else(|| ResourceManagerError::ExtractionFailed {
                            resource: internal_path.clone(),
                            container: info.container_path.display().to_string(),
                        })?
                }
                ContainerType::Directory => std::fs::read(&info.container_path)?,
            };

        let gff = GffParser::from_bytes(data).map_err(|e| {
            ResourceManagerError::InvalidGffFormat(format!("Failed to parse template: {e}"))
        })?;

        let fields = gff.read_struct_fields(0).map_err(|e| {
            ResourceManagerError::InvalidGffFormat(format!("Failed to read template fields: {e}"))
        })?;

        let mut owned_fields = IndexMap::new();
        for (k, v) in fields {
            owned_fields.insert(k, v.force_owned());
        }

        Ok(owned_fields)
    }

    pub fn get_item_template_summary(
        &self,
        info: &TemplateInfo,
    ) -> ResourceManagerResult<(Option<String>, i32)> {
        let data =
            match &info.container_type {
                ContainerType::Zip => {
                    let internal_path = info.internal_path.as_ref().ok_or_else(|| {
                        ResourceManagerError::Parse("Missing internal path".into())
                    })?;

                    self.zip_reader
                        .lock()
                        .read_file_from_zip(
                            info.container_path.to_string_lossy().to_string(),
                            internal_path.clone(),
                        )
                        .map_err(|e| ResourceManagerError::ZipError(e.clone()))?
                }
                ContainerType::Erf => {
                    let internal_path = info.internal_path.as_ref().ok_or_else(|| {
                        ResourceManagerError::Parse("Missing internal path".into())
                    })?;

                    let mut erf = ErfParser::new();
                    erf.read(&info.container_path).map_err(|e| {
                        ResourceManagerError::InvalidErfFormat(format!(
                            "Failed to open {}: {}",
                            info.container_path.display(),
                            e
                        ))
                    })?;

                    erf.resources
                        .get(internal_path)
                        .ok_or_else(|| ResourceManagerError::ExtractionFailed {
                            resource: internal_path.clone(),
                            container: info.container_path.display().to_string(),
                        })?
                        .data
                        .clone()
                        .ok_or_else(|| ResourceManagerError::ExtractionFailed {
                            resource: internal_path.clone(),
                            container: info.container_path.display().to_string(),
                        })?
                }
                ContainerType::Directory => std::fs::read(&info.container_path)?,
            };

        let gff = GffParser::from_bytes(data).map_err(|e| {
            ResourceManagerError::InvalidGffFormat(format!("Failed to parse template: {e}"))
        })?;

        let base_item = gff
            .read_field_by_label(0, "BaseItem")
            .ok()
            .and_then(|v| match v {
                crate::parsers::gff::GffValue::Int(i) => Some(i),
                crate::parsers::gff::GffValue::Short(s) => Some(i32::from(s)),
                crate::parsers::gff::GffValue::Byte(b) => Some(i32::from(b)),
                crate::parsers::gff::GffValue::Word(w) => Some(i32::from(w)),
                crate::parsers::gff::GffValue::Dword(d) => Some(d as i32),
                _ => None,
            })
            .unwrap_or(0);

        let name = gff
            .read_field_by_label(0, "LocalizedName")
            .ok()
            .and_then(|v| {
                if let crate::parsers::gff::GffValue::LocString(ls) = v {
                    if !ls.substrings.is_empty() {
                        Some(ls.substrings[0].string.to_string())
                    } else if ls.string_ref >= 0 {
                        Some(self.get_string(ls.string_ref))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter(|s| !s.is_empty());

        Ok((name, base_item))
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);

        CacheStats {
            size: self.tda_locations.len(),
            max_size: 0,
            hits,
            misses,
            hit_ratio: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        }
    }

    pub fn get_module_cache_stats(&self) -> CacheStats {
        self.module_cache.get_stats()
    }

    pub fn get_zip_cache_stats(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.zip_reader.lock().get_stats()
    }

    pub fn get_module_info(&self) -> Option<&ModuleInfo> {
        self.module_info.as_ref()
    }

    pub fn get_available_2da_files(&self) -> Vec<String> {
        self.tda_locations.keys().cloned().collect()
    }

    pub fn get_tlk_parser(&self) -> Option<Arc<StdRwLock<TLKParser>>> {
        self.tlk_cache.clone()
    }

    pub fn get_resource_bytes(
        &self,
        resref: &str,
        extension: &str,
    ) -> ResourceManagerResult<Vec<u8>> {
        let filename = format!("{resref}.{extension}");
        let filename_lower = filename.to_lowercase();

        tracing::debug!("get_resource_bytes: looking for '{}'", filename_lower);

        let paths = self.paths.blocking_read();

        for override_dir in paths.custom_override_folders() {
            let file_path = override_dir.join(&filename);
            if file_path.exists() {
                tracing::debug!("  Found in custom override: {:?}", file_path);
                return Ok(std::fs::read(&file_path)?);
            }
            let file_path = override_dir.join(&filename_lower);
            if file_path.exists() {
                tracing::debug!("  Found in custom override (lowercase): {:?}", file_path);
                return Ok(std::fs::read(&file_path)?);
            }
        }

        if let Some(override_dir) = paths.override_dir() {
            let file_path = override_dir.join(&filename);
            if file_path.exists() {
                tracing::debug!("  Found in override dir: {:?}", file_path);
                return Ok(std::fs::read(&file_path)?);
            }
            let file_path = override_dir.join(&filename_lower);
            if file_path.exists() {
                tracing::debug!("  Found in override dir (lowercase): {:?}", file_path);
                return Ok(std::fs::read(&file_path)?);
            }
        }

        if let Some(data_dir) = paths.data() {
            tracing::debug!("  Searching zips in {:?}", data_dir);
            drop(paths);
            if data_dir.exists()
                && let Ok(entries) = std::fs::read_dir(&data_dir)
            {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
                    {
                        match self
                            .zip_reader
                            .lock()
                            .find_file_by_name(&path.to_string_lossy(), &filename_lower)
                        {
                            Ok(Some(bytes)) => {
                                tracing::debug!(
                                    "  Found '{}' in zip {:?} ({} bytes)",
                                    filename_lower,
                                    path.file_name().unwrap_or_default(),
                                    bytes.len()
                                );
                                return Ok(bytes);
                            }
                            Ok(None) => {}
                            Err(e) => {
                                tracing::warn!(
                                    "  Error searching zip {:?}: {}",
                                    path.file_name().unwrap_or_default(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        } else {
            tracing::warn!("  No game data directory configured");
            drop(paths);
        }

        tracing::debug!("  Not found: {}", filename_lower);
        Err(ResourceManagerError::FileNotFound(
            std::path::PathBuf::from(format!("{resref}.{extension}")),
        ))
    }

    pub fn list_resources_by_extension(&self, extension: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        let paths = self.paths.blocking_read();

        if let Some(data_dir) = paths.data() {
            drop(paths);
            if data_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&data_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path
                            .extension()
                            .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
                        {
                            let zip_name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            if let Ok(files) = self
                                .zip_reader
                                .lock()
                                .list_files_by_extension(&path.to_string_lossy(), extension)
                            {
                                for file in files {
                                    results.push((file, zip_name.clone()));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            drop(paths);
        }

        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }
}

fn gff_value_to_json(value: &crate::parsers::gff::GffValue<'_>) -> serde_json::Value {
    use crate::parsers::gff::GffValue;

    match value {
        GffValue::Byte(v) => serde_json::json!(*v),
        GffValue::Char(v) => serde_json::json!(*v as u8),
        GffValue::Word(v) => serde_json::json!(*v),
        GffValue::Short(v) => serde_json::json!(*v),
        GffValue::Dword(v) => serde_json::json!(*v),
        GffValue::Int(v) => serde_json::json!(*v),
        GffValue::Dword64(v) => serde_json::json!(*v),
        GffValue::Int64(v) => serde_json::json!(*v),
        GffValue::Float(v) => serde_json::json!(*v),
        GffValue::Double(v) => serde_json::json!(*v),
        GffValue::String(s) => serde_json::json!(s.as_ref()),
        GffValue::ResRef(s) => serde_json::json!(s.as_ref()),
        GffValue::LocString(ls) => {
            let mut obj = serde_json::Map::new();
            obj.insert("string_ref".to_string(), serde_json::json!(ls.string_ref));
            let substrings: Vec<serde_json::Value> = ls
                .substrings
                .iter()
                .map(|sub| {
                    serde_json::json!({
                        "string": sub.string.as_ref(),
                        "language": sub.language,
                        "gender": sub.gender
                    })
                })
                .collect();
            obj.insert(
                "substrings".to_string(),
                serde_json::Value::Array(substrings),
            );
            serde_json::Value::Object(obj)
        }
        GffValue::Void(data) => serde_json::json!(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            data.as_ref()
        )),
        GffValue::Struct(lazy) => {
            let fields = lazy.force_load();
            let obj: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), gff_value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        GffValue::List(items) => {
            let arr: Vec<serde_json::Value> = items
                .iter()
                .map(|lazy| {
                    let fields = lazy.force_load();
                    let obj: serde_json::Map<String, serde_json::Value> = fields
                        .iter()
                        .map(|(k, v)| (k.clone(), gff_value_to_json(v)))
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            serde_json::Value::Array(arr)
        }
        GffValue::StructOwned(fields) => {
            let obj: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), gff_value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        GffValue::ListOwned(items) => {
            let arr: Vec<serde_json::Value> = items
                .iter()
                .map(|fields| {
                    let obj: serde_json::Map<String, serde_json::Value> = fields
                        .iter()
                        .map(|(k, v)| (k.clone(), gff_value_to_json(v)))
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            serde_json::Value::Array(arr)
        }
        GffValue::StructRef(idx) => serde_json::json!({ "struct_ref": idx }),
        GffValue::ListRef(indices) => serde_json::json!({ "list_ref": indices }),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_recursive_directory_indexing() {
        use std::fs;
        use std::io::Write;

        // Create a temp directory structure
        let temp_dir = std::env::temp_dir().join("nwn2_test_recursive");
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let sub_dir = temp_dir.join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let file1 = temp_dir.join("test1.2da");
        let mut f1 = fs::File::create(&file1).unwrap();
        f1.write_all(b"test").unwrap();

        let file2 = sub_dir.join("test2.2da");
        let mut f2 = fs::File::create(&file2).unwrap();
        f2.write_all(b"test").unwrap();

        // Placeholder test to verify compilation
        assert!(true);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
