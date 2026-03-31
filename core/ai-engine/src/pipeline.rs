//! Backend Integration Pipeline (MVP)
//!
//! Coordinates non-frontend backend modules in one flow:
//! quantization -> federated aggregation -> model audit -> HR-AIS ingestion gate.

use crate::federated_aggregation::{AggregationResult, FederatedCoordinator, NodeUpdate, RoundStatus};
use crate::hr_ais::{HrAisService, IngestionDecision, NodeMetrics, ReputationResult};
use crate::model_audit::{AuditInput, AuditReport, AuditStatus, ModelAuditor};
use crate::quantization::{QuantizedModel, Quantizer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRequest {
    pub model_id: String,
    pub updates: Vec<NodeUpdate>,
    pub audit_input: AuditInput,
    pub node_metrics: NodeMetrics,
    pub requested_source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReport {
    pub quantized_model: QuantizedModel,
    pub aggregation: AggregationResult,
    pub audit_report: Option<AuditReport>,
    pub node_reputation: ReputationResult,
    pub ingestion_decision: Option<IngestionDecision>,
    pub success: bool,
    pub reason: String,
}

pub struct BackendPipeline {
    quantizer: Quantizer,
    federated: FederatedCoordinator,
    auditor: ModelAuditor,
    hr_ais: HrAisService,
}

impl BackendPipeline {
    pub fn new(
        quantizer: Quantizer,
        federated: FederatedCoordinator,
        auditor: ModelAuditor,
        hr_ais: HrAisService,
    ) -> Self {
        Self {
            quantizer,
            federated,
            auditor,
            hr_ais,
        }
    }

    pub fn default() -> Self {
        Self::new(
            Quantizer::default(),
            FederatedCoordinator::default(),
            ModelAuditor::default(),
            HrAisService::default(),
        )
    }

    pub fn execute_round(&mut self, request: PipelineRequest) -> Result<PipelineReport, String> {
        let quantized_model = self.quantizer.quantize_model(&request.model_id)?;

        for update in request.updates {
            self.federated.submit_update(update)?;
        }

        let aggregation = self.federated.finalize_round()?;
        let node_reputation = self.hr_ais.evaluate_node(&request.node_metrics);

        if aggregation.status != RoundStatus::Aggregated {
            return Ok(PipelineReport {
                quantized_model,
                aggregation,
                audit_report: None,
                node_reputation,
                ingestion_decision: None,
                success: false,
                reason: "aggregation skipped due to low quorum".to_string(),
            });
        }

        let audit_report = self.auditor.audit(&request.audit_input);
        if audit_report.status != AuditStatus::Passed {
            return Ok(PipelineReport {
                quantized_model,
                aggregation,
                audit_report: Some(audit_report),
                node_reputation,
                ingestion_decision: None,
                success: false,
                reason: "audit failed".to_string(),
            });
        }

        let ingestion_decision =
            self.hr_ais
                .decide_ingestion(&node_reputation, &request.requested_source_url);

        let success = ingestion_decision.allowed;
        let reason = if success {
            "pipeline completed successfully".to_string()
        } else {
            "ingestion blocked by HR-AIS policy".to_string()
        };

        Ok(PipelineReport {
            quantized_model,
            aggregation,
            audit_report: Some(audit_report),
            node_reputation,
            ingestion_decision: Some(ingestion_decision),
            success,
            reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_audit::{ModelMetadata, ShardRecord};

    fn updates_for_quorum(count: usize) -> Vec<NodeUpdate> {
        (1..=count)
            .map(|i| NodeUpdate {
                node_id: format!("node-{}", i),
                sample_count: 10,
                values: vec![1.0, 2.0, 3.0],
                checksum: "ok".to_string(),
            })
            .collect()
    }

    fn valid_audit_input() -> AuditInput {
        let metadata = ModelMetadata {
            model_id: "tinyllama-1.1b-int4".to_string(),
            version: "mvp-0.1".to_string(),
            total_size_bytes: 500_000_000,
            shard_count: 2,
            architecture: "tinyllama".to_string(),
        };
        let shards = vec![
            ShardRecord {
                shard_id: "s1".to_string(),
                size_bytes: 250_000_000,
                checksum: "a1".to_string(),
            },
            ShardRecord {
                shard_id: "s2".to_string(),
                size_bytes: 250_000_000,
                checksum: "b2".to_string(),
            },
        ];

        let auditor = ModelAuditor::default();
        let checksum = auditor.compute_model_checksum(&metadata, &shards);

        AuditInput {
            metadata,
            shards,
            expected_model_checksum: checksum,
        }
    }

    fn high_reputation_metrics() -> NodeMetrics {
        NodeMetrics {
            node_id: "node-1".to_string(),
            uptime_percent: 99.0,
            consensus_percent: 95.0,
            audit_pass_percent: 90.0,
            no_slash_percent: 100.0,
        }
    }

    #[test]
    fn test_pipeline_success() {
        let mut pipeline = BackendPipeline::default();

        let req = PipelineRequest {
            model_id: "tinyllama-1.1b".to_string(),
            updates: updates_for_quorum(8),
            audit_input: valid_audit_input(),
            node_metrics: high_reputation_metrics(),
            requested_source_url: "https://huggingface.co/docs".to_string(),
        };

        let report = pipeline.execute_round(req).unwrap();
        assert!(report.success);
        assert_eq!(report.aggregation.status, RoundStatus::Aggregated);
        assert!(report.audit_report.is_some());
        assert!(report.ingestion_decision.unwrap().allowed);
    }

    #[test]
    fn test_pipeline_fails_on_low_quorum() {
        let mut pipeline = BackendPipeline::default();

        let req = PipelineRequest {
            model_id: "tinyllama-1.1b".to_string(),
            updates: updates_for_quorum(5),
            audit_input: valid_audit_input(),
            node_metrics: high_reputation_metrics(),
            requested_source_url: "https://huggingface.co/docs".to_string(),
        };

        let report = pipeline.execute_round(req).unwrap();
        assert!(!report.success);
        assert_eq!(report.aggregation.status, RoundStatus::SkippedLowQuorum);
        assert!(report.audit_report.is_none());
        assert!(report.ingestion_decision.is_none());
    }
}
