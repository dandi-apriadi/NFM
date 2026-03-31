//! Federated Aggregation (FedAvg) Coordinator
//!
//! Langkah 5 NFM Brain MVP:
//! - Collect gradient/model updates from local nodes
//! - Aggregate with Federated Averaging (FedAvg)
//! - Handle node dropout with minimum quorum
//! - Emit deterministic round summary for audit pipeline

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-node contribution for one training round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeUpdate {
    /// Unique node identifier.
    pub node_id: String,
    /// Number of samples used by this node for local training.
    pub sample_count: u32,
    /// Flattened model delta/weights for aggregation.
    pub values: Vec<f32>,
    /// Optional checksum/hash marker from node.
    pub checksum: String,
}

/// Runtime status of a federated round.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    Pending,
    Aggregated,
    SkippedLowQuorum,
}

/// Result summary for one aggregation round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub round_id: u64,
    pub status: RoundStatus,
    pub received_nodes: usize,
    pub min_required_nodes: usize,
    pub total_samples: u32,
    pub dropped_nodes: Vec<String>,
    pub aggregated_values: Vec<f32>,
}

/// FedAvg coordinator configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedConfig {
    /// Expected total nodes in this federation.
    pub expected_nodes: usize,
    /// Minimum updates needed to run aggregation.
    pub min_required_nodes: usize,
    /// Max wait time per round in seconds.
    pub round_timeout_secs: u64,
}

impl Default for FederatedConfig {
    fn default() -> Self {
        Self {
            expected_nodes: 10,
            min_required_nodes: 8,
            round_timeout_secs: 300,
        }
    }
}

/// Simple in-memory FedAvg coordinator for MVP.
pub struct FederatedCoordinator {
    config: FederatedConfig,
    current_round: u64,
    updates: HashMap<String, NodeUpdate>,
}

impl FederatedCoordinator {
    pub fn new(config: FederatedConfig) -> Self {
        Self {
            config,
            current_round: 1,
            updates: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        Self::new(FederatedConfig::default())
    }

    pub fn round_id(&self) -> u64 {
        self.current_round
    }

    pub fn config(&self) -> &FederatedConfig {
        &self.config
    }

    /// Accept/replace node update in the active round.
    pub fn submit_update(&mut self, update: NodeUpdate) -> Result<(), String> {
        if update.sample_count == 0 {
            return Err("sample_count must be > 0".to_string());
        }
        if update.values.is_empty() {
            return Err("values cannot be empty".to_string());
        }

        // Allow overwrite from same node to support retry/resubmission.
        self.updates.insert(update.node_id.clone(), update);
        Ok(())
    }

    pub fn received_node_count(&self) -> usize {
        self.updates.len()
    }

    /// Aggregate with FedAvg if quorum is met.
    pub fn finalize_round(&mut self) -> Result<AggregationResult, String> {
        let received_nodes = self.updates.len();

        if received_nodes < self.config.min_required_nodes {
            let dropped = self.derive_dropped_nodes();
            let result = AggregationResult {
                round_id: self.current_round,
                status: RoundStatus::SkippedLowQuorum,
                received_nodes,
                min_required_nodes: self.config.min_required_nodes,
                total_samples: self.total_samples(),
                dropped_nodes: dropped,
                aggregated_values: Vec::new(),
            };

            self.advance_round();
            return Ok(result);
        }

        let aggregated = self.compute_fedavg()?;
        let result = AggregationResult {
            round_id: self.current_round,
            status: RoundStatus::Aggregated,
            received_nodes,
            min_required_nodes: self.config.min_required_nodes,
            total_samples: self.total_samples(),
            dropped_nodes: self.derive_dropped_nodes(),
            aggregated_values: aggregated,
        };

        self.advance_round();
        Ok(result)
    }

    fn total_samples(&self) -> u32 {
        self.updates.values().map(|u| u.sample_count).sum()
    }

    fn derive_dropped_nodes(&self) -> Vec<String> {
        let mut dropped = Vec::new();
        for i in 0..self.config.expected_nodes {
            let node_id = format!("node-{}", i + 1);
            if !self.updates.contains_key(&node_id) {
                dropped.push(node_id);
            }
        }
        dropped
    }

    fn compute_fedavg(&self) -> Result<Vec<f32>, String> {
        let first = self
            .updates
            .values()
            .next()
            .ok_or_else(|| "No updates available".to_string())?;

        let dim = first.values.len();
        if dim == 0 {
            return Err("Invalid update dimension".to_string());
        }

        for upd in self.updates.values() {
            if upd.values.len() != dim {
                return Err("Inconsistent update dimensions among nodes".to_string());
            }
        }

        let total_samples = self.total_samples() as f32;
        if total_samples <= 0.0 {
            return Err("Total samples must be > 0".to_string());
        }

        let mut out = vec![0.0_f32; dim];
        for upd in self.updates.values() {
            let weight = upd.sample_count as f32 / total_samples;
            for (i, value) in upd.values.iter().enumerate() {
                out[i] += weight * value;
            }
        }

        Ok(out)
    }

    fn advance_round(&mut self) {
        self.current_round += 1;
        self.updates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_update(node_id: &str, sample_count: u32, values: Vec<f32>) -> NodeUpdate {
        NodeUpdate {
            node_id: node_id.to_string(),
            sample_count,
            values,
            checksum: "ok".to_string(),
        }
    }

    #[test]
    fn test_submit_update_validation() {
        let mut coord = FederatedCoordinator::default();

        let err_zero = coord.submit_update(mk_update("node-1", 0, vec![1.0]));
        assert!(err_zero.is_err());

        let err_empty = coord.submit_update(mk_update("node-1", 1, vec![]));
        assert!(err_empty.is_err());
    }

    #[test]
    fn test_skip_when_low_quorum() {
        let mut coord = FederatedCoordinator::default();
        for i in 1..=5 {
            coord
                .submit_update(mk_update(&format!("node-{}", i), 10, vec![1.0, 2.0]))
                .unwrap();
        }

        let result = coord.finalize_round().unwrap();
        assert_eq!(result.status, RoundStatus::SkippedLowQuorum);
        assert_eq!(result.received_nodes, 5);
        assert!(result.aggregated_values.is_empty());
        assert_eq!(coord.round_id(), 2);
    }

    #[test]
    fn test_fedavg_aggregation_success() {
        let mut coord = FederatedCoordinator::default();

        // Weighted average expected:
        // total = 10 + 30 = 40
        // out[0] = 0.25*2 + 0.75*6 = 5
        // out[1] = 0.25*4 + 0.75*8 = 7
        coord
            .submit_update(mk_update("node-1", 10, vec![2.0, 4.0]))
            .unwrap();
        coord
            .submit_update(mk_update("node-2", 30, vec![6.0, 8.0]))
            .unwrap();

        // Add 6 more nodes to satisfy quorum 8/10.
        for i in 3..=8 {
            coord
                .submit_update(mk_update(&format!("node-{}", i), 5, vec![5.0, 7.0]))
                .unwrap();
        }

        let result = coord.finalize_round().unwrap();
        assert_eq!(result.status, RoundStatus::Aggregated);
        assert_eq!(result.received_nodes, 8);
        assert_eq!(result.aggregated_values.len(), 2);

        // Should be around stable weighted values.
        assert!(result.aggregated_values[0] > 4.5 && result.aggregated_values[0] < 5.5);
        assert!(result.aggregated_values[1] > 6.5 && result.aggregated_values[1] < 7.5);
    }

    #[test]
    fn test_dimension_mismatch_rejected() {
        let mut coord = FederatedCoordinator::default();

        for i in 1..=7 {
            coord
                .submit_update(mk_update(&format!("node-{}", i), 10, vec![1.0, 2.0]))
                .unwrap();
        }
        coord
            .submit_update(mk_update("node-8", 10, vec![1.0]))
            .unwrap();

        let err = coord.finalize_round();
        assert!(err.is_err());
    }
}
