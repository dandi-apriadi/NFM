use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataClass {
    NodeLocal,
    Regional,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMeta {
    pub node_id: String,
    pub region: String,
    pub latitude: f64,
    pub longitude: f64,
    pub ewma_latency_ms: f64,
    pub queue_depth: f64,
    pub error_rate: f64,
    pub healthy: bool,
}

impl NodeMeta {
    pub fn new(node_id: &str, region: &str, latitude: f64, longitude: f64) -> Self {
        Self {
            node_id: node_id.to_string(),
            region: region.to_string(),
            latitude,
            longitude,
            ewma_latency_ms: 50.0,
            queue_depth: 0.0,
            error_rate: 0.0,
            healthy: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestProfile {
    pub requester_node_id: Option<String>,
    pub user_latitude: f64,
    pub user_longitude: f64,
    pub data_class: DataClass,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScore {
    pub node_id: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteBenchmark {
    pub selected_node: String,
    pub selected_score: f64,
    pub fallback_node: Option<String>,
    pub fallback_score: Option<f64>,
    pub projected_score_gain: f64,
    pub top_candidates: Vec<CandidateScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedRecord {
    pub key: String,
    pub value: serde_json::Value,
    pub class: DataClass,
    pub owner_node: String,
    pub replica_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSnapshot {
    pub nodes: HashMap<String, NodeMeta>,
    pub records: HashMap<String, RoutedRecord>,
    pub weights: RouterWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterWeights {
    pub latency: f64,
    pub queue: f64,
    pub error: f64,
    pub geo: f64,
}

impl Default for RouterWeights {
    fn default() -> Self {
        Self {
            latency: 0.55,
            queue: 0.20,
            error: 0.20,
            geo: 0.05,
        }
    }
}

#[derive(Debug)]
pub struct GeoDistributedBrainDb {
    nodes: HashMap<String, NodeMeta>,
    records: HashMap<String, RoutedRecord>,
    weights: RouterWeights,
}

impl Default for GeoDistributedBrainDb {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoDistributedBrainDb {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            records: HashMap::new(),
            weights: RouterWeights::default(),
        }
    }

    pub fn set_weights(&mut self, weights: RouterWeights) {
        self.weights = weights;
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn export_snapshot(&self) -> BrainSnapshot {
        BrainSnapshot {
            nodes: self.nodes.clone(),
            records: self.records.clone(),
            weights: self.weights.clone(),
        }
    }

    pub fn import_snapshot(&mut self, snapshot: BrainSnapshot) {
        self.nodes = snapshot.nodes;
        self.records = snapshot.records;
        self.weights = snapshot.weights;
    }

    pub fn register_node(&mut self, meta: NodeMeta) {
        self.nodes.insert(meta.node_id.clone(), meta);
    }

    pub fn update_runtime_metrics(
        &mut self,
        node_id: &str,
        ewma_latency_ms: f64,
        queue_depth: f64,
        error_rate: f64,
        healthy: bool,
    ) -> Result<(), String> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| format!("node not found: {}", node_id))?;

        node.ewma_latency_ms = ewma_latency_ms.max(0.0);
        node.queue_depth = queue_depth.max(0.0);
        node.error_rate = error_rate.clamp(0.0, 1.0);
        node.healthy = healthy;
        Ok(())
    }

    pub fn upsert_record(
        &mut self,
        key: &str,
        value: serde_json::Value,
        class: DataClass,
        owner_node: &str,
    ) -> Result<(), String> {
        if !self.nodes.contains_key(owner_node) {
            return Err(format!("owner node not found: {}", owner_node));
        }

        let replica_nodes = self.select_replica_nodes(owner_node, &class);

        self.records.insert(
            key.to_string(),
            RoutedRecord {
                key: key.to_string(),
                value,
                class,
                owner_node: owner_node.to_string(),
                replica_nodes,
            },
        );

        Ok(())
    }

    pub fn get_record(&self, key: &str) -> Option<&RoutedRecord> {
        self.records.get(key)
    }

    pub fn route_request(&self, profile: &RequestProfile) -> Option<String> {
        let best = self
            .nodes
            .values()
            .filter(|n| n.healthy)
            .filter(|n| self.filter_by_class(profile, n))
            .min_by(|a, b| {
                let sa = self.node_score(profile, a);
                let sb = self.node_score(profile, b);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })?;

        Some(best.node_id.clone())
    }

    pub fn fetch_nearest_fastest(
        &self,
        key: &str,
        profile: &RequestProfile,
    ) -> Option<(String, serde_json::Value)> {
        let record = self.records.get(key)?;

        let mut candidates = Vec::new();
        candidates.push(record.owner_node.clone());
        candidates.extend(record.replica_nodes.iter().cloned());

        let best = candidates
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .filter(|n| n.healthy)
            .filter(|n| self.filter_by_class(profile, n))
            .min_by(|a, b| {
                let sa = self.node_score(profile, a);
                let sb = self.node_score(profile, b);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })?;

        Some((best.node_id.clone(), record.value.clone()))
    }

    pub fn hedged_candidates(&self, profile: &RequestProfile, count: usize) -> Vec<String> {
        let mut scored: Vec<(&NodeMeta, f64)> = self
            .nodes
            .values()
            .filter(|n| n.healthy)
            .filter(|n| self.filter_by_class(profile, n))
            .map(|n| (n, self.node_score(profile, n)))
            .collect();

        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(count.max(1))
            .map(|(n, _)| n.node_id.clone())
            .collect()
    }

    pub fn route_benchmark(&self, profile: &RequestProfile, count: usize) -> Option<RouteBenchmark> {
        let mut scored = self.scored_candidates(profile, &self.weights);

        if scored.is_empty() {
            return None;
        }

        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let selected_node = scored[0].0.node_id.clone();
        let selected_score = scored[0].1;
        let fallback_node = scored.get(1).map(|(n, _)| n.node_id.clone());
        let fallback_score = scored.get(1).map(|(_, s)| *s);
        let projected_score_gain = fallback_score.map(|s| s - selected_score).unwrap_or(0.0);

        let top_candidates = scored
            .iter()
            .take(count.max(1))
            .map(|(n, s)| CandidateScore {
                node_id: n.node_id.clone(),
                score: *s,
            })
            .collect();

        Some(RouteBenchmark {
            selected_node,
            selected_score,
            fallback_node,
            fallback_score,
            projected_score_gain,
            top_candidates,
        })
    }

    pub fn route_benchmark_with_weights(
        &self,
        profile: &RequestProfile,
        weights: &RouterWeights,
        count: usize,
    ) -> Option<RouteBenchmark> {
        let mut scored = self.scored_candidates(profile, weights);

        if scored.is_empty() {
            return None;
        }

        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let selected_node = scored[0].0.node_id.clone();
        let selected_score = scored[0].1;
        let fallback_node = scored.get(1).map(|(n, _)| n.node_id.clone());
        let fallback_score = scored.get(1).map(|(_, s)| *s);
        let projected_score_gain = fallback_score.map(|s| s - selected_score).unwrap_or(0.0);

        let top_candidates = scored
            .iter()
            .take(count.max(1))
            .map(|(n, s)| CandidateScore {
                node_id: n.node_id.clone(),
                score: *s,
            })
            .collect();

        Some(RouteBenchmark {
            selected_node,
            selected_score,
            fallback_node,
            fallback_score,
            projected_score_gain,
            top_candidates,
        })
    }

    fn node_score_with_weights(
        &self,
        profile: &RequestProfile,
        node: &NodeMeta,
        weights: &RouterWeights,
    ) -> f64 {
        let geo_distance_km = haversine_km(
            profile.user_latitude,
            profile.user_longitude,
            node.latitude,
            node.longitude,
        );

        weights.latency * node.ewma_latency_ms
            + weights.queue * node.queue_depth
            + weights.error * (node.error_rate * 1000.0)
            + weights.geo * geo_distance_km
    }

    fn node_score(&self, profile: &RequestProfile, node: &NodeMeta) -> f64 {
        self.node_score_with_weights(profile, node, &self.weights)
    }

    fn scored_candidates(&self, profile: &RequestProfile, weights: &RouterWeights) -> Vec<(&NodeMeta, f64)> {
        self.nodes
            .values()
            .filter(|n| n.healthy)
            .filter(|n| self.filter_by_class(profile, n))
            .map(|n| (n, self.node_score_with_weights(profile, n, weights)))
            .collect()
    }

    fn filter_by_class(&self, profile: &RequestProfile, node: &NodeMeta) -> bool {
        match profile.data_class {
            DataClass::Global => true,
            DataClass::Regional => {
                // Regional traffic should prefer same region if requester known.
                if let Some(requester) = &profile.requester_node_id {
                    if let Some(requester_meta) = self.nodes.get(requester) {
                        return requester_meta.region == node.region;
                    }
                }
                true
            }
            DataClass::NodeLocal => {
                if let Some(requester) = &profile.requester_node_id {
                    return requester == &node.node_id;
                }
                false
            }
        }
    }

    fn select_replica_nodes(&self, owner_node: &str, class: &DataClass) -> Vec<String> {
        let owner = match self.nodes.get(owner_node) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let mut peers: Vec<&NodeMeta> = self
            .nodes
            .values()
            .filter(|n| n.healthy && n.node_id != owner.node_id)
            .collect();

        peers.sort_by(|a, b| {
            let da = haversine_km(owner.latitude, owner.longitude, a.latitude, a.longitude);
            let db = haversine_km(owner.latitude, owner.longitude, b.latitude, b.longitude);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });

        let replica_count = match class {
            DataClass::NodeLocal => 1,
            DataClass::Regional => 2,
            DataClass::Global => 3,
        };

        peers
            .into_iter()
            .take(replica_count)
            .map(|n| n.node_id.clone())
            .collect()
    }
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> GeoDistributedBrainDb {
        let mut db = GeoDistributedBrainDb::new();

        db.register_node(NodeMeta::new("id-jkt-a", "id", -6.2088, 106.8456));
        db.register_node(NodeMeta::new("id-sby-a", "id", -7.2575, 112.7521));
        db.register_node(NodeMeta::new("sg-sin-a", "sg", 1.3521, 103.8198));
        db.register_node(NodeMeta::new("jp-tyo-a", "jp", 35.6762, 139.6503));

        db.update_runtime_metrics("id-jkt-a", 22.0, 8.0, 0.01, true)
            .expect("failed to update metrics for id-jkt-a");
        db.update_runtime_metrics("id-sby-a", 18.0, 16.0, 0.01, true)
            .expect("failed to update metrics for id-sby-a");
        db.update_runtime_metrics("sg-sin-a", 45.0, 2.0, 0.00, true)
            .expect("failed to update metrics for sg-sin-a");
        db.update_runtime_metrics("jp-tyo-a", 80.0, 1.0, 0.00, true)
            .expect("failed to update metrics for jp-tyo-a");

        db
    }

    #[test]
    fn routes_to_fastest_nearby_healthy_node() {
        let db = setup_db();
        let profile = RequestProfile {
            requester_node_id: Some("id-jkt-a".to_string()),
            user_latitude: -6.2,
            user_longitude: 106.8,
            data_class: DataClass::Regional,
            critical: false,
        };

        let selected = db.route_request(&profile);
        assert_eq!(selected, Some("id-jkt-a".to_string()));
    }

    #[test]
    fn avoids_unhealthy_node_even_if_geographically_close() {
        let mut db = setup_db();
        db.update_runtime_metrics("id-jkt-a", 10.0, 2.0, 0.0, false)
            .expect("failed to update metrics for id-jkt-a");

        let profile = RequestProfile {
            requester_node_id: Some("id-jkt-a".to_string()),
            user_latitude: -6.2,
            user_longitude: 106.8,
            data_class: DataClass::Regional,
            critical: false,
        };

        let selected = db.route_request(&profile);
        assert_eq!(selected, Some("id-sby-a".to_string()));
    }

    #[test]
    fn stores_data_with_class_aware_replica_policy() {
        let mut db = setup_db();
        db.upsert_record(
            "user:42:profile",
            serde_json::json!({"name": "Dandi"}),
            DataClass::Regional,
            "id-jkt-a",
        )
        .expect("failed to upsert regional record");

        let record = db
            .get_record("user:42:profile")
            .expect("record should exist after upsert");
        assert_eq!(record.class, DataClass::Regional);
        assert_eq!(record.owner_node, "id-jkt-a");
        assert_eq!(record.replica_nodes.len(), 2);
    }

    #[test]
    fn fetches_from_best_owner_or_replica() {
        let mut db = setup_db();
        db.upsert_record(
            "policy:global:1",
            serde_json::json!({"mode": "strict"}),
            DataClass::Global,
            "sg-sin-a",
        )
        .expect("failed to upsert global policy record");

        // Make owner slower to force replica selection.
        db.update_runtime_metrics("sg-sin-a", 180.0, 80.0, 0.10, true)
            .expect("failed to update metrics for sg-sin-a");

        let profile = RequestProfile {
            requester_node_id: Some("id-jkt-a".to_string()),
            user_latitude: -6.2,
            user_longitude: 106.8,
            data_class: DataClass::Global,
            critical: true,
        };

        let fetched = db.fetch_nearest_fastest("policy:global:1", &profile);
        assert!(fetched.is_some());
        let (node, value) = fetched.expect("fetched result should be present");
        assert_ne!(node, "sg-sin-a");
        assert_eq!(value["mode"], "strict");
    }

    #[test]
    fn returns_top_two_candidates_for_critical_hedged_request() {
        let db = setup_db();
        let profile = RequestProfile {
            requester_node_id: Some("id-jkt-a".to_string()),
            user_latitude: -6.2,
            user_longitude: 106.8,
            data_class: DataClass::Global,
            critical: true,
        };

        let candidates = db.hedged_candidates(&profile, 2);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0], "id-jkt-a");
    }

    #[test]
    fn benchmark_exposes_selected_and_fallback_scores() {
        let db = setup_db();
        let profile = RequestProfile {
            requester_node_id: Some("id-jkt-a".to_string()),
            user_latitude: -6.2,
            user_longitude: 106.8,
            data_class: DataClass::Global,
            critical: true,
        };

        let bench = db
            .route_benchmark(&profile, 3)
            .expect("benchmark should produce candidates");

        assert_eq!(bench.selected_node, "id-jkt-a");
        assert!(bench.fallback_node.is_some());
        assert!(bench.projected_score_gain >= 0.0);
        assert_eq!(bench.top_candidates.len(), 3);
    }

    #[test]
    fn benchmark_with_custom_weights_changes_scoring_strategy() {
        let db = setup_db();
        let profile = RequestProfile {
            requester_node_id: Some("id-jkt-a".to_string()),
            user_latitude: -6.2,
            user_longitude: 106.8,
            data_class: DataClass::Global,
            critical: true,
        };

        let default_bench = db
            .route_benchmark(&profile, 3)
            .expect("default benchmark should be available");

        let latency_heavy = RouterWeights {
            latency: 0.9,
            queue: 0.05,
            error: 0.04,
            geo: 0.01,
        };

        let tuned_bench = db
            .route_benchmark_with_weights(&profile, &latency_heavy, 3)
            .expect("tuned benchmark should be available");

        assert_eq!(default_bench.selected_node, tuned_bench.selected_node);
        assert!(tuned_bench.selected_score <= default_bench.selected_score * 2.0);
    }

    #[test]
    fn snapshot_roundtrip_restores_nodes_and_records() {
        let mut db = setup_db();
        db.upsert_record(
            "policy:backup:1",
            serde_json::json!({"mode": "resilient"}),
            DataClass::Global,
            "id-jkt-a",
        )
        .expect("failed to upsert record for snapshot test");

        let snapshot = db.export_snapshot();

        let mut restored = GeoDistributedBrainDb::new();
        restored.import_snapshot(snapshot);

        assert_eq!(restored.node_count(), 4);
        assert_eq!(restored.record_count(), 1);
        let rec = restored
            .get_record("policy:backup:1")
            .expect("restored record should exist");
        assert_eq!(rec.value["mode"], "resilient");
    }
}
