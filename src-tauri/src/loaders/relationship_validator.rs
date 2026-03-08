use std::collections::{HashMap, HashSet};

use crate::services::rule_detector::RuleDetector;

use super::types::{
    BrokenReference, LoadedTable, RelationshipDefinition, RelationshipType, ValidationReport,
};

pub struct RelationshipValidator {
    rule_detector: RuleDetector,
    relationships: HashSet<RelationshipDefinition>,
    dependency_graph: HashMap<String, HashSet<String>>,
}

impl RelationshipValidator {
    pub fn new() -> Self {
        Self {
            rule_detector: RuleDetector::new(),
            relationships: HashSet::new(),
            dependency_graph: HashMap::new(),
        }
    }

    pub fn with_rule_detector(rule_detector: RuleDetector) -> Self {
        Self {
            rule_detector,
            relationships: HashSet::new(),
            dependency_graph: HashMap::new(),
        }
    }

    pub fn detect_relationships(&mut self, tables: &HashMap<String, LoadedTable>) {
        self.relationships.clear();
        self.dependency_graph.clear();

        for (table_name, table) in tables {
            let columns: Vec<String> = table
                .column_names()
                .iter()
                .map(|s| (*s).to_string())
                .collect();

            for column in &columns {
                self.check_column_for_relationship(table_name, column, tables);
            }
        }
    }

    fn check_column_for_relationship(
        &mut self,
        table_name: &str,
        column_name: &str,
        tables: &HashMap<String, LoadedTable>,
    ) {
        let purpose = self.rule_detector.detect_column_purpose(column_name);

        if let Some(target_table) = purpose.target_table() {
            if tables.contains_key(target_table) {
                self.add_lookup_reference(table_name, column_name, target_table);
            }
            return;
        }

        let col_lower = column_name.to_lowercase();

        if col_lower.ends_with("table") {
            self.add_table_reference(table_name, column_name, tables);
            return;
        }

        if col_lower.ends_with("id") || col_lower.ends_with("_id") {
            let potential_table = col_lower
                .trim_end_matches("_id")
                .trim_end_matches("id")
                .to_string();

            let target = Self::resolve_table_name(&potential_table, tables);
            if let Some(target_table) = target {
                self.add_lookup_reference(table_name, column_name, &target_table);
            }
        }
    }

    fn resolve_table_name(name: &str, tables: &HashMap<String, LoadedTable>) -> Option<String> {
        if tables.contains_key(name) {
            return Some(name.to_string());
        }

        let aliases = [
            ("spell", "spells"),
            ("feat", "feat"),
            ("class", "classes"),
            ("skill", "skills"),
            ("race", "racialtypes"),
            ("item", "baseitems"),
            ("domain", "domains"),
            ("school", "spellschools"),
        ];

        for (alias, target) in aliases {
            if name == alias && tables.contains_key(target) {
                return Some(target.to_string());
            }
        }

        None
    }

    fn add_lookup_reference(
        &mut self,
        source_table: &str,
        source_column: &str,
        target_table: &str,
    ) {
        let rel = RelationshipDefinition::new_lookup(
            source_table.to_string(),
            source_column.to_string(),
            target_table.to_string(),
        );
        self.relationships.insert(rel);
        self.dependency_graph
            .entry(source_table.to_string())
            .or_default()
            .insert(target_table.to_string());
    }

    fn add_table_reference(
        &mut self,
        source_table: &str,
        source_column: &str,
        tables: &HashMap<String, LoadedTable>,
    ) {
        let Some(table) = tables.get(source_table) else {
            return;
        };

        let mut referenced_tables: HashSet<String> = HashSet::new();

        let sample_count = table.row_count().min(10);
        for row_idx in 0..sample_count {
            if let Ok(Some(value)) = table.get_cell(row_idx, source_column) {
                let value_lower = value.to_lowercase();
                if tables.contains_key(&value_lower) {
                    referenced_tables.insert(value_lower);
                }
            }
        }

        for target_table in referenced_tables {
            let rel = RelationshipDefinition::new_table_reference(
                source_table.to_string(),
                source_column.to_string(),
                target_table.clone(),
            );
            self.relationships.insert(rel);
            self.dependency_graph
                .entry(source_table.to_string())
                .or_default()
                .insert(target_table);
        }
    }

    pub fn validate_relationships(
        &self,
        tables: &HashMap<String, LoadedTable>,
        strict: bool,
    ) -> ValidationReport {
        let mut report = ValidationReport::new();
        report.total_relationships = self.relationships.len();
        report.dependency_order = self.calculate_load_order(tables);

        for rel in &self.relationships {
            if self.validate_single_relationship(rel, tables, &mut report, strict) {
                report.valid_relationships += 1;
            }
        }

        report
    }

    fn validate_single_relationship(
        &self,
        rel: &RelationshipDefinition,
        tables: &HashMap<String, LoadedTable>,
        report: &mut ValidationReport,
        _strict: bool,
    ) -> bool {
        let Some(target_table) = tables.get(&rel.target_table) else {
            report.add_missing_table(rel.target_table.clone());
            return false;
        };

        let target_ids: HashSet<i32> = (0..target_table.row_count() as i32).collect();

        let Some(source_table) = tables.get(&rel.source_table) else {
            return false;
        };

        let mut valid = true;

        for row_idx in 0..source_table.row_count() {
            let Ok(value) = source_table.get_cell(row_idx, &rel.source_column) else {
                continue;
            };

            let Some(value_str) = value else {
                continue;
            };

            if RuleDetector::is_null_value(Some(&value_str)) {
                continue;
            }

            match rel.relationship_type {
                RelationshipType::TableReference => {
                    let value_lower = value_str.to_lowercase();
                    if !tables.contains_key(&value_lower) {
                        report.add_broken_reference(BrokenReference::new(
                            rel.source_table.clone(),
                            rel.source_column.clone(),
                            row_idx,
                            rel.target_table.clone(),
                            -1,
                        ));
                        valid = false;
                    }
                }
                RelationshipType::Lookup => {
                    if let Some(id) = RuleDetector::parse_int(Some(&value_str))
                        && id >= 0
                        && !target_ids.contains(&id)
                    {
                        report.add_broken_reference(BrokenReference::new(
                            rel.source_table.clone(),
                            rel.source_column.clone(),
                            row_idx,
                            rel.target_table.clone(),
                            id,
                        ));
                        valid = false;
                    }
                }
            }
        }

        valid
    }

    pub fn calculate_load_order(&self, tables: &HashMap<String, LoadedTable>) -> Vec<String> {
        let all_tables: Vec<String> = tables.keys().cloned().collect();

        let mut no_deps: Vec<String> = Vec::new();
        let mut has_deps: Vec<String> = Vec::new();

        for table in &all_tables {
            if self
                .dependency_graph
                .get(table)
                .is_none_or(std::collections::HashSet::is_empty)
            {
                no_deps.push(table.clone());
            } else {
                has_deps.push(table.clone());
            }
        }

        no_deps.sort();

        let mut load_order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for table in &no_deps {
            self.visit_table(table, &mut load_order, &mut visited, &mut visiting);
        }

        for table in &has_deps {
            self.visit_table(table, &mut load_order, &mut visited, &mut visiting);
        }

        for table in all_tables {
            if !visited.contains(&table) {
                load_order.push(table.clone());
                visited.insert(table);
            }
        }

        load_order
    }

    fn visit_table(
        &self,
        table: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) {
        if visited.contains(table) {
            return;
        }

        if visiting.contains(table) {
            // Cycle detected, break recursion
            return;
        }

        visiting.insert(table.to_string());

        if let Some(deps) = self.dependency_graph.get(table) {
            for dep in deps {
                self.visit_table(dep, order, visited, visiting);
            }
        }

        visiting.remove(table);

        if !visited.contains(table) {
            order.push(table.to_string());
            visited.insert(table.to_string());
        }
    }

    pub fn get_table_dependencies(&self, table_name: &str) -> HashSet<String> {
        self.dependency_graph
            .get(table_name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_table_dependents(&self, table_name: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        for (source, deps) in &self.dependency_graph {
            if deps.contains(table_name) {
                dependents.insert(source.clone());
            }
        }
        dependents
    }

    pub fn relationships(&self) -> &HashSet<RelationshipDefinition> {
        &self.relationships
    }

    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }
}

impl Default for RelationshipValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_load_order_empty() {
        let validator = RelationshipValidator::new();
        let tables = HashMap::new();
        let order = validator.calculate_load_order(&tables);
        assert!(order.is_empty());
    }

    #[test]
    fn test_get_table_dependencies_empty() {
        let validator = RelationshipValidator::new();
        let deps = validator.get_table_dependencies("nonexistent");
        assert!(deps.is_empty());
    }
}
