//! HR-AIS (High-Reputation Autonomous Internet Search) — Minimal MVP
//!
//! Langkah 7 NFM Brain MVP:
//! - Compute deterministic node reputation score
//! - Enforce reputation threshold for data ingestion
//! - Keep allowlist source model for safe MVP execution

use serde::{Deserialize, Serialize};

/// Reputation scoring configuration locked in planning:
/// score = uptime*0.4 + consensus*0.3 + audit_pass*0.2 + no_slash*0.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    pub uptime_weight: f32,
    pub consensus_weight: f32,
    pub audit_pass_weight: f32,
    pub no_slash_weight: f32,
    pub threshold_percent: f32,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            uptime_weight: 0.4,
            consensus_weight: 0.3,
            audit_pass_weight: 0.2,
            no_slash_weight: 0.1,
            threshold_percent: 80.0,
        }
    }
}

/// Node operational metrics normalized to 0..100.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub uptime_percent: f32,
    pub consensus_percent: f32,
    pub audit_pass_percent: f32,
    pub no_slash_percent: f32,
}

/// Result of node reputation evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationResult {
    pub node_id: String,
    pub score_percent: f32,
    pub eligible: bool,
    pub reason: String,
}

/// Source record ingested by HR-AIS pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub url: String,
    pub category: String,
    pub min_reputation_percent: f32,
}

/// Ingestion decision for one source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionDecision {
    pub node_id: String,
    pub url: String,
    pub allowed: bool,
    pub reason: String,
}

pub struct HrAisService {
    config: ReputationConfig,
    allowlist_sources: Vec<SourceRecord>,
}

impl HrAisService {
    pub fn new(config: ReputationConfig, allowlist_sources: Vec<SourceRecord>) -> Self {
        Self {
            config,
            allowlist_sources,
        }
    }

    pub fn default() -> Self {
        // Safe MVP allowlist from curated domains only.
        Self::new(
            ReputationConfig::default(),
            vec![
                SourceRecord {
                    url: "https://huggingface.co/docs".to_string(),
                    category: "ml-docs".to_string(),
                    min_reputation_percent: 80.0,
                },
                SourceRecord {
                    url: "https://arxiv.org/list/cs.AI/recent".to_string(),
                    category: "research".to_string(),
                    min_reputation_percent: 85.0,
                },
                SourceRecord {
                    url: "https://www.imf.org/en/Publications".to_string(),
                    category: "finance".to_string(),
                    min_reputation_percent: 90.0,
                },
            ],
        )
    }

    pub fn evaluate_node(&self, metrics: &NodeMetrics) -> ReputationResult {
        let uptime = clamp_percent(metrics.uptime_percent);
        let consensus = clamp_percent(metrics.consensus_percent);
        let audit = clamp_percent(metrics.audit_pass_percent);
        let no_slash = clamp_percent(metrics.no_slash_percent);

        let score = uptime * self.config.uptime_weight
            + consensus * self.config.consensus_weight
            + audit * self.config.audit_pass_weight
            + no_slash * self.config.no_slash_weight;

        let eligible = score >= self.config.threshold_percent;
        let reason = if eligible {
            "eligible for ingestion".to_string()
        } else {
            format!(
                "score {:.2}% below threshold {:.2}%",
                score, self.config.threshold_percent
            )
        };

        ReputationResult {
            node_id: metrics.node_id.clone(),
            score_percent: score,
            eligible,
            reason,
        }
    }

    pub fn decide_ingestion(
        &self,
        node_eval: &ReputationResult,
        requested_url: &str,
    ) -> IngestionDecision {
        let source = self.allowlist_sources.iter().find(|s| s.url == requested_url);

        if source.is_none() {
            return IngestionDecision {
                node_id: node_eval.node_id.clone(),
                url: requested_url.to_string(),
                allowed: false,
                reason: "source not in allowlist".to_string(),
            };
        }

        let source = source.unwrap();
        if !node_eval.eligible {
            return IngestionDecision {
                node_id: node_eval.node_id.clone(),
                url: requested_url.to_string(),
                allowed: false,
                reason: "node reputation not eligible".to_string(),
            };
        }

        if node_eval.score_percent < source.min_reputation_percent {
            return IngestionDecision {
                node_id: node_eval.node_id.clone(),
                url: requested_url.to_string(),
                allowed: false,
                reason: format!(
                    "source requires {:.2}% reputation, node has {:.2}%",
                    source.min_reputation_percent, node_eval.score_percent
                ),
            };
        }

        IngestionDecision {
            node_id: node_eval.node_id.clone(),
            url: requested_url.to_string(),
            allowed: true,
            reason: "ingestion approved".to_string(),
        }
    }

    pub fn allowlist(&self) -> &[SourceRecord] {
        &self.allowlist_sources
    }
}

fn clamp_percent(value: f32) -> f32 {
    value.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_formula_matches_locked_weights() {
        let service = HrAisService::default();
        let metrics = NodeMetrics {
            node_id: "node-1".to_string(),
            uptime_percent: 100.0,
            consensus_percent: 80.0,
            audit_pass_percent: 90.0,
            no_slash_percent: 100.0,
        };

        // 100*0.4 + 80*0.3 + 90*0.2 + 100*0.1 = 92
        let result = service.evaluate_node(&metrics);
        assert!((result.score_percent - 92.0).abs() < 0.001);
        assert!(result.eligible);
    }

    #[test]
    fn test_low_reputation_not_eligible() {
        let service = HrAisService::default();
        let metrics = NodeMetrics {
            node_id: "node-2".to_string(),
            uptime_percent: 60.0,
            consensus_percent: 60.0,
            audit_pass_percent: 60.0,
            no_slash_percent: 100.0,
        };

        let result = service.evaluate_node(&metrics);
        assert!(result.score_percent < 80.0);
        assert!(!result.eligible);
    }

    #[test]
    fn test_allowlist_source_required() {
        let service = HrAisService::default();
        let eval = ReputationResult {
            node_id: "node-3".to_string(),
            score_percent: 95.0,
            eligible: true,
            reason: "ok".to_string(),
        };

        let decision = service.decide_ingestion(&eval, "https://example.com/random");
        assert!(!decision.allowed);
        assert!(decision.reason.contains("allowlist"));
    }

    #[test]
    fn test_reputation_and_source_threshold_for_ingestion() {
        let service = HrAisService::default();
        let eval = ReputationResult {
            node_id: "node-4".to_string(),
            score_percent: 92.0,
            eligible: true,
            reason: "ok".to_string(),
        };

        let approved = service.decide_ingestion(&eval, "https://huggingface.co/docs");
        assert!(approved.allowed);

        let denied = service.decide_ingestion(&eval, "https://www.imf.org/en/Publications");
        assert!(denied.allowed); // 92 >= 90
    }

    #[test]
    fn test_not_eligible_node_cannot_ingest() {
        let service = HrAisService::default();
        let eval = ReputationResult {
            node_id: "node-5".to_string(),
            score_percent: 75.0,
            eligible: false,
            reason: "below threshold".to_string(),
        };

        let decision = service.decide_ingestion(&eval, "https://huggingface.co/docs");
        assert!(!decision.allowed);
        assert!(decision.reason.contains("not eligible"));
    }
}
