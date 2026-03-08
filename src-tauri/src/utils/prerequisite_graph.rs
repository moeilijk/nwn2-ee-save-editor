use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct PrerequisiteGraph {
    feat_requirements: Vec<Vec<u32>>,
    direct_prerequisites: Vec<Prerequisites>,
    stats: GraphStats,
    build_time_ms: f64,
    is_built: bool,
}

#[derive(Clone, Debug, Default)]
struct Prerequisites {
    feats: Vec<u32>,
    abilities: HashMap<String, u32>,
    class: Option<u32>,
    level: u32,
    bab: u32,
    spell_level: u32,
}

#[derive(Clone, Debug, Default)]
struct GraphStats {
    total_feats: usize,
    feats_with_prereqs: usize,
    max_chain_depth: usize,
    circular_dependencies: Vec<u32>,
}

impl PrerequisiteGraph {
    pub fn new() -> Self {
        PrerequisiteGraph {
            feat_requirements: Vec::new(),
            direct_prerequisites: Vec::new(),
            stats: GraphStats::default(),
            build_time_ms: 0.0,
            is_built: false,
        }
    }

    pub fn build_from_data(
        &mut self,
        feat_data: &[HashMap<String, serde_json::Value>],
    ) -> Result<(), String> {
        let start = Instant::now();

        let total_feats = feat_data.len();
        self.stats.total_feats = total_feats;

        let mut direct_prereqs: Vec<Prerequisites> = vec![Prerequisites::default(); total_feats];

        for (index, feat_dict) in feat_data.iter().enumerate() {
            let mut prereqs = Prerequisites::default();

            if let Some(feat1) = feat_dict
                .get("prereqfeat1")
                .and_then(serde_json::Value::as_i64)
                && feat1 >= 0
            {
                prereqs.feats.push(feat1 as u32);
            }
            if let Some(feat2) = feat_dict
                .get("prereqfeat2")
                .and_then(serde_json::Value::as_i64)
                && feat2 >= 0
            {
                prereqs.feats.push(feat2 as u32);
            }

            if let Some(min_str) = feat_dict.get("minstr").and_then(serde_json::Value::as_u64)
                && min_str > 0
            {
                prereqs
                    .abilities
                    .insert("strength".to_string(), min_str as u32);
            }
            if let Some(min_dex) = feat_dict.get("mindex").and_then(serde_json::Value::as_u64)
                && min_dex > 0
            {
                prereqs
                    .abilities
                    .insert("dexterity".to_string(), min_dex as u32);
            }
            if let Some(min_con) = feat_dict.get("mincon").and_then(serde_json::Value::as_u64)
                && min_con > 0
            {
                prereqs
                    .abilities
                    .insert("constitution".to_string(), min_con as u32);
            }
            if let Some(min_int) = feat_dict.get("minint").and_then(serde_json::Value::as_u64)
                && min_int > 0
            {
                prereqs
                    .abilities
                    .insert("intelligence".to_string(), min_int as u32);
            }
            if let Some(min_wis) = feat_dict.get("minwis").and_then(serde_json::Value::as_u64)
                && min_wis > 0
            {
                prereqs
                    .abilities
                    .insert("wisdom".to_string(), min_wis as u32);
            }
            if let Some(min_cha) = feat_dict.get("mincha").and_then(serde_json::Value::as_u64)
                && min_cha > 0
            {
                prereqs
                    .abilities
                    .insert("charisma".to_string(), min_cha as u32);
            }

            if let Some(min_level) = feat_dict
                .get("minlevel")
                .and_then(serde_json::Value::as_u64)
            {
                prereqs.level = min_level as u32;
            }
            if let Some(min_bab) = feat_dict
                .get("minattackbonus")
                .and_then(serde_json::Value::as_u64)
            {
                prereqs.bab = min_bab as u32;
            }
            if let Some(spell_level) = feat_dict
                .get("minspelllvl")
                .and_then(serde_json::Value::as_u64)
            {
                prereqs.spell_level = spell_level as u32;
            }

            let has_prereqs = !prereqs.feats.is_empty()
                || !prereqs.abilities.is_empty()
                || prereqs.class.is_some()
                || prereqs.level > 0
                || prereqs.bab > 0
                || prereqs.spell_level > 0;

            if has_prereqs {
                self.stats.feats_with_prereqs += 1;
            }

            direct_prereqs[index] = prereqs;
        }

        self.direct_prerequisites = direct_prereqs.clone();

        let max_depth = Arc::new(RwLock::new(0usize));
        let circular_deps = Arc::new(RwLock::new(Vec::new()));

        let flattened_results: Vec<Vec<u32>> = (0..total_feats)
            .into_par_iter()
            .map(|feat_id| {
                let mut visited = vec![false; total_feats];
                Self::flatten_prerequisites_internal(
                    feat_id as u32,
                    &mut visited,
                    1,
                    &direct_prereqs,
                    &max_depth,
                    &circular_deps,
                )
            })
            .collect();

        self.feat_requirements = flattened_results;
        self.stats.max_chain_depth = *max_depth.read();
        self.stats.circular_dependencies = circular_deps.read().clone();

        self.build_time_ms = start.elapsed().as_millis() as f64;
        self.is_built = true;

        Ok(())
    }

    fn flatten_prerequisites_internal(
        feat_id: u32,
        visited: &mut Vec<bool>,
        depth: usize,
        direct_prereqs: &[Prerequisites],
        max_depth: &Arc<RwLock<usize>>,
        circular_deps: &Arc<RwLock<Vec<u32>>>,
    ) -> Vec<u32> {
        let idx = feat_id as usize;

        if idx >= visited.len() || visited[idx] {
            if idx < visited.len() {
                circular_deps.write().push(feat_id);
            }
            return Vec::new();
        }

        {
            let mut max = max_depth.write();
            if depth > *max {
                *max = depth;
            }
        }

        visited[idx] = true;

        let mut all_requirements = Vec::new();
        if idx < direct_prereqs.len() {
            let prereqs = &direct_prereqs[idx];
            for &req_feat in &prereqs.feats {
                if !all_requirements.contains(&req_feat) {
                    all_requirements.push(req_feat);
                }

                let nested = Self::flatten_prerequisites_internal(
                    req_feat,
                    visited,
                    depth + 1,
                    direct_prereqs,
                    max_depth,
                    circular_deps,
                );
                for req in nested {
                    if !all_requirements.contains(&req) {
                        all_requirements.push(req);
                    }
                }
            }
        }

        visited[idx] = false;
        all_requirements
    }

    pub fn get_all_feat_requirements(&self, feat_id: u32) -> Vec<u32> {
        if !self.is_built {
            return Vec::new();
        }

        let idx = feat_id as usize;
        if idx < self.feat_requirements.len() {
            self.feat_requirements[idx].clone()
        } else {
            Vec::new()
        }
    }

    pub fn get_direct_prerequisites(&self, feat_id: u32) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();
        let idx = feat_id as usize;

        if idx < self.direct_prerequisites.len() {
            let prereqs = &self.direct_prerequisites[idx];
            result.insert("feats".to_string(), serde_json::json!(prereqs.feats));
            result.insert(
                "abilities".to_string(),
                serde_json::json!(prereqs.abilities),
            );
            result.insert("class".to_string(), serde_json::json!(prereqs.class));
            result.insert("level".to_string(), serde_json::json!(prereqs.level));
            result.insert("bab".to_string(), serde_json::json!(prereqs.bab));
            result.insert(
                "spell_level".to_string(),
                serde_json::json!(prereqs.spell_level),
            );
        } else {
            result.insert("feats".to_string(), serde_json::json!(Vec::<u32>::new()));
            result.insert(
                "abilities".to_string(),
                serde_json::json!(HashMap::<String, u32>::new()),
            );
            result.insert("class".to_string(), serde_json::Value::Null);
            result.insert("level".to_string(), serde_json::json!(0));
            result.insert("bab".to_string(), serde_json::json!(0));
            result.insert("spell_level".to_string(), serde_json::json!(0));
        }

        result
    }

    pub fn validate_feat_prerequisites_fast(
        &self,
        feat_id: u32,
        character_feats: &[u32],
        character_data: Option<&HashMap<String, serde_json::Value>>,
    ) -> (bool, Vec<String>) {
        if !self.is_built {
            return (true, Vec::new());
        }

        let mut errors = Vec::new();

        let mut char_has_feat = vec![false; self.feat_requirements.len()];
        for &feat in character_feats {
            let idx = feat as usize;
            if idx < char_has_feat.len() {
                char_has_feat[idx] = true;
            }
        }

        let idx = feat_id as usize;
        if idx < self.feat_requirements.len() {
            for &req_feat in &self.feat_requirements[idx] {
                let req_idx = req_feat as usize;
                if req_idx >= char_has_feat.len() || !char_has_feat[req_idx] {
                    errors.push(format!("Requires Feat {req_feat}"));
                }
            }
        }

        if let Some(data) = character_data
            && idx < self.direct_prerequisites.len()
        {
            let prereqs = &self.direct_prerequisites[idx];

            for (ability, min_score) in &prereqs.abilities {
                if let Some(current) = data.get(ability).and_then(serde_json::Value::as_u64)
                    && (current as u32) < *min_score
                {
                    errors.push(format!("Requires {} {}", ability.to_uppercase(), min_score));
                }
            }

            if prereqs.level > 0
                && let Some(level) = data.get("level").and_then(serde_json::Value::as_u64)
                && (level as u32) < prereqs.level
            {
                errors.push(format!("Requires character level {}", prereqs.level));
            }

            if prereqs.bab > 0
                && let Some(bab) = data.get("bab").and_then(serde_json::Value::as_u64)
                && (bab as u32) < prereqs.bab
            {
                errors.push(format!("Requires base attack bonus +{}", prereqs.bab));
            }
        }

        (errors.is_empty(), errors)
    }

    pub fn validate_batch_fast(
        &self,
        feat_ids: Vec<u32>,
        character_feats: &[u32],
        character_data: Option<&HashMap<String, serde_json::Value>>,
    ) -> HashMap<u32, (bool, Vec<String>)> {
        if !self.is_built {
            return HashMap::new();
        }

        let mut char_has_feat = vec![false; self.feat_requirements.len()];
        for &feat in character_feats {
            let idx = feat as usize;
            if idx < char_has_feat.len() {
                char_has_feat[idx] = true;
            }
        }

        let mut results = HashMap::new();

        for feat_id in feat_ids {
            let mut errors = Vec::new();
            let idx = feat_id as usize;

            if idx < self.feat_requirements.len() {
                for &required in &self.feat_requirements[idx] {
                    let req_idx = required as usize;
                    if req_idx >= char_has_feat.len() || !char_has_feat[req_idx] {
                        errors.push(format!("Requires Feat {required}"));
                    }
                }
            }

            if let Some(data) = character_data
                && idx < self.direct_prerequisites.len()
            {
                let prereqs = &self.direct_prerequisites[idx];

                for (ability, min_score) in &prereqs.abilities {
                    if let Some(current) = data.get(ability).and_then(serde_json::Value::as_u64)
                        && (current as u32) < *min_score
                    {
                        errors.push(format!("Requires {} {}", ability.to_uppercase(), min_score));
                    }
                }

                if prereqs.level > 0
                    && let Some(level) = data.get("level").and_then(serde_json::Value::as_u64)
                    && (level as u32) < prereqs.level
                {
                    errors.push(format!("Requires character level {}", prereqs.level));
                }

                if prereqs.bab > 0
                    && let Some(bab) = data.get("bab").and_then(serde_json::Value::as_u64)
                    && (bab as u32) < prereqs.bab
                {
                    errors.push(format!("Requires base attack bonus +{}", prereqs.bab));
                }
            }

            results.insert(feat_id, (errors.is_empty(), errors));
        }

        results
    }

    pub fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert("is_built".to_string(), serde_json::json!(self.is_built));
        stats.insert(
            "build_time_ms".to_string(),
            serde_json::json!(self.build_time_ms),
        );
        stats.insert(
            "total_feats".to_string(),
            serde_json::json!(self.stats.total_feats),
        );
        stats.insert(
            "feats_with_prerequisites".to_string(),
            serde_json::json!(self.stats.feats_with_prereqs),
        );
        stats.insert(
            "max_chain_depth".to_string(),
            serde_json::json!(self.stats.max_chain_depth),
        );
        stats.insert(
            "circular_dependencies_count".to_string(),
            serde_json::json!(self.stats.circular_dependencies.len()),
        );
        stats.insert(
            "memory_estimate_mb".to_string(),
            serde_json::json!((self.feat_requirements.len() * 100) as f64 / (1024.0 * 1024.0)),
        );
        stats
    }
}

impl Default for PrerequisiteGraph {
    fn default() -> Self {
        Self::new()
    }
}
