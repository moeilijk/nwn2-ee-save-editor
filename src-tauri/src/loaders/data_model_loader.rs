use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::parsers::tlk::TLKParser;
use crate::services::resource_manager::ResourceManager;
use crate::services::RuleDetector;

use super::constants::{is_priority_table, should_load_table, PRIORITY_TABLES};
use super::error::{LoaderError, LoaderResult};
use super::relationship_validator::RelationshipValidator;
use super::types::{
    GameData, LoadedTable, LoadingProgress, LoadingStats, TableMetadata, ValidationReport,
};

type ProgressCallback = Box<dyn Fn(&str, f32) + Send + Sync>;

pub struct DataModelLoader {
    resource_manager: Arc<RwLock<ResourceManager>>,
    rule_detector: RuleDetector,
    relationship_validator: RelationshipValidator,
    progress: LoadingProgress,
    validate_relationships: bool,
    priority_only: bool,
}

impl DataModelLoader {
    pub fn new(resource_manager: Arc<RwLock<ResourceManager>>) -> Self {
        Self {
            resource_manager,
            rule_detector: RuleDetector::new(),
            relationship_validator: RelationshipValidator::new(),
            progress: LoadingProgress::default(),
            validate_relationships: true,
            priority_only: false,
        }
    }

    pub fn with_options(
        resource_manager: Arc<RwLock<ResourceManager>>,
        validate_relationships: bool,
        priority_only: bool,
    ) -> Self {
        Self {
            resource_manager,
            rule_detector: RuleDetector::new(),
            relationship_validator: RelationshipValidator::new(),
            progress: LoadingProgress::default(),
            validate_relationships,
            priority_only,
        }
    }

    pub fn set_progress_callback(
        &mut self,
        callback: ProgressCallback,
    ) {
        self.progress = LoadingProgress::new(Some(callback));
    }

    pub async fn load_game_data(
        &mut self,
        tlk_parser: Arc<std::sync::RwLock<TLKParser>>,
    ) -> LoaderResult<GameData> {
        let start_time = Instant::now();
        let mut stats = LoadingStats::default();

        self.progress.update("Scanning 2DA files...", 5.0);
        let tables_to_load = self.scan_2da_files().await?;

        self.progress.update("Sorting by dependencies...", 10.0);
        let sorted_tables = self.sort_by_dependency_order(&tables_to_load);

        self.progress.update("Loading table data...", 15.0);
        let mut loaded_tables = HashMap::new();
        let total_tables = sorted_tables.len();

        for (idx, metadata) in sorted_tables.iter().enumerate() {
            let progress = 15.0 + (idx as f32 / total_tables as f32) * 70.0;
            if idx % 25 == 0 || is_priority_table(&metadata.name) {
                self.progress
                    .update(&format!("Loading {}...", metadata.name), progress);
            }

            match self.load_table(&metadata.name).await {
                Ok(table) => {
                    stats.total_rows += table.row_count();
                    if is_priority_table(&metadata.name) {
                        stats.priority_tables_loaded += 1;
                    }
                    loaded_tables.insert(metadata.name.clone(), table);
                }
                Err(e) => {
                    warn!("Failed to load table {}: {}", metadata.name, e);
                }
            }

            if idx % 50 == 0 {
                tokio::task::yield_now().await;
            }
        }

        stats.tables_loaded = loaded_tables.len();

        let mut game_data = GameData::new(tlk_parser);
        game_data.tables = loaded_tables;
        game_data.rule_detector = Some(self.rule_detector.clone());
        game_data.priority_tables = PRIORITY_TABLES.iter().map(|s| (*s).to_string()).collect();

        if self.validate_relationships {
            self.progress.update("Validating relationships...", 90.0);
            game_data.relationships = self.validate_relationships(&game_data.tables);
            stats.relationships_detected = game_data.relationships.total_relationships;
        }

        stats.load_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        self.progress.update("Loading complete", 100.0);

        info!(
            "Loaded {} tables ({} rows) in {:.1}ms",
            stats.tables_loaded, stats.total_rows, stats.load_time_ms
        );

        Ok(game_data)
    }

    async fn scan_2da_files(&self) -> LoaderResult<Vec<TableMetadata>> {
        let rm = self.resource_manager.read().await;

        let available_files = rm.get_available_2da_files();
        let mut tables = Vec::new();

        for name in available_files {
            let name_lower = name.to_lowercase().replace(".2da", "");

            if self.priority_only && !is_priority_table(&name_lower) {
                continue;
            }

            if !should_load_table(&name_lower) {
                continue;
            }

            match rm.get_2da_with_overrides(&name_lower) {
                Ok(parser) => {
                    tables.push(TableMetadata::new(
                        name_lower,
                        parser.row_count(),
                        parser.column_count(),
                    ));
                }
                Err(e) => {
                    debug!("Skipping {}: {}", name_lower, e);
                }
            }
        }

        info!("Found {} tables to load", tables.len());
        Ok(tables)
    }

    fn sort_by_dependency_order(&mut self, tables: &[TableMetadata]) -> Vec<TableMetadata> {
        let temp_loaded: HashMap<String, LoadedTable> = HashMap::new();
        let load_order = self
            .relationship_validator
            .calculate_load_order(&temp_loaded);

        let mut sorted = Vec::with_capacity(tables.len());
        let mut remaining: Vec<_> = tables.to_vec();

        for table_name in &load_order {
            if let Some(pos) = remaining.iter().position(|t| &t.name == table_name) {
                sorted.push(remaining.remove(pos));
            }
        }

        let mut priority_remaining: Vec<_> = remaining
            .iter()
            .filter(|t| is_priority_table(&t.name))
            .cloned()
            .collect();
        priority_remaining.sort_by(|a, b| a.name.cmp(&b.name));

        let mut other_remaining: Vec<_> = remaining
            .iter()
            .filter(|t| !is_priority_table(&t.name))
            .cloned()
            .collect();
        other_remaining.sort_by(|a, b| a.name.cmp(&b.name));

        sorted.extend(priority_remaining);
        sorted.extend(other_remaining);

        sorted
    }

    async fn load_table(&self, name: &str) -> LoaderResult<LoadedTable> {
        let rm = self.resource_manager.read().await;

        let parser = rm.get_2da_with_overrides(name).map_err(|e| {
            LoaderError::Parse(format!("Failed to get 2DA {name}: {e}"))
        })?;

        Ok(LoadedTable::new(name.to_string(), parser))
    }

    fn validate_relationships(
        &mut self,
        tables: &HashMap<String, LoadedTable>,
    ) -> ValidationReport {
        self.relationship_validator.detect_relationships(tables);
        self.relationship_validator
            .validate_relationships(tables, false)
    }

    pub fn get_stats(&self) -> LoadingStats {
        LoadingStats::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_table_check() {
        assert!(is_priority_table("classes"));
        assert!(is_priority_table("feat"));
        assert!(!is_priority_table("cls_feat_fighter"));
    }
}
