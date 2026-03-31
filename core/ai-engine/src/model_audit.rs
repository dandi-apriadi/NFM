//! Model Audit Service (MVP)
//!
//! Langkah 6 NFM Brain MVP:
//! - Validate model metadata integrity
//! - Validate shard count and size constraints
//! - Verify model checksum marker
//! - Produce deterministic pass/fail report for blockchain gateway integration

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub version: String,
    pub total_size_bytes: u64,
    pub shard_count: u32,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardRecord {
    pub shard_id: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInput {
    pub metadata: ModelMetadata,
    pub shards: Vec<ShardRecord>,
    /// Expected model checksum registered before deployment.
    pub expected_model_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub model_id: String,
    pub status: AuditStatus,
    pub score: u8,
    pub reasons: Vec<String>,
    pub computed_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPolicy {
    pub min_score_to_pass: u8,
    pub max_shard_size_bytes: u64,
    pub min_shard_count: u32,
}

impl Default for AuditPolicy {
    fn default() -> Self {
        Self {
            min_score_to_pass: 80,
            max_shard_size_bytes: 300_000_000,
            min_shard_count: 1,
        }
    }
}

pub struct ModelAuditor {
    policy: AuditPolicy,
}

impl ModelAuditor {
    pub fn new(policy: AuditPolicy) -> Self {
        Self { policy }
    }

    pub fn default() -> Self {
        Self::new(AuditPolicy::default())
    }

    pub fn audit(&self, input: &AuditInput) -> AuditReport {
        let mut score: i32 = 100;
        let mut reasons = Vec::new();

        // 1) Metadata sanity checks
        if input.metadata.model_id.trim().is_empty() {
            score -= 30;
            reasons.push("model_id is empty".to_string());
        }
        if input.metadata.version.trim().is_empty() {
            score -= 10;
            reasons.push("version is empty".to_string());
        }
        if input.metadata.total_size_bytes == 0 {
            score -= 20;
            reasons.push("total_size_bytes is zero".to_string());
        }

        // 2) Shard constraints
        if input.metadata.shard_count < self.policy.min_shard_count {
            score -= 20;
            reasons.push("shard_count below minimum".to_string());
        }
        if input.shards.len() as u32 != input.metadata.shard_count {
            score -= 25;
            reasons.push("shard list count mismatch metadata.shard_count".to_string());
        }

        for shard in &input.shards {
            if shard.size_bytes == 0 {
                score -= 10;
                reasons.push(format!("shard {} has zero size", shard.shard_id));
            }
            if shard.size_bytes > self.policy.max_shard_size_bytes {
                score -= 10;
                reasons.push(format!(
                    "shard {} exceeds max shard size policy",
                    shard.shard_id
                ));
            }
            if shard.checksum.trim().is_empty() {
                score -= 10;
                reasons.push(format!("shard {} checksum is empty", shard.shard_id));
            }
        }

        // 3) Checksum verification
        let computed_checksum = self.compute_model_checksum(&input.metadata, &input.shards);
        if computed_checksum != input.expected_model_checksum {
            score -= 35;
            reasons.push("model checksum mismatch".to_string());
        }

        let score = score.clamp(0, 100) as u8;
        let status = if score >= self.policy.min_score_to_pass {
            AuditStatus::Passed
        } else {
            AuditStatus::Failed
        };

        AuditReport {
            model_id: input.metadata.model_id.clone(),
            status,
            score,
            reasons,
            computed_checksum,
        }
    }

    pub fn compute_model_checksum(&self, metadata: &ModelMetadata, shards: &[ShardRecord]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(metadata.model_id.as_bytes());
        hasher.update(metadata.version.as_bytes());
        hasher.update(metadata.total_size_bytes.to_le_bytes());
        hasher.update(metadata.shard_count.to_le_bytes());
        hasher.update(metadata.architecture.as_bytes());

        for shard in shards {
            hasher.update(shard.shard_id.as_bytes());
            hasher.update(shard.size_bytes.to_le_bytes());
            hasher.update(shard.checksum.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> AuditInput {
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
        let expected = auditor.compute_model_checksum(&metadata, &shards);

        AuditInput {
            metadata,
            shards,
            expected_model_checksum: expected,
        }
    }

    #[test]
    fn test_audit_passes_for_valid_input() {
        let auditor = ModelAuditor::default();
        let input = valid_input();
        let report = auditor.audit(&input);

        assert_eq!(report.status, AuditStatus::Passed);
        assert!(report.score >= 80);
        assert!(report.reasons.is_empty());
    }

    #[test]
    fn test_audit_fails_for_checksum_mismatch() {
        let auditor = ModelAuditor::default();
        let mut input = valid_input();
        input.expected_model_checksum = "bad".to_string();

        let report = auditor.audit(&input);
        assert_eq!(report.status, AuditStatus::Failed);
        assert!(report.reasons.iter().any(|r| r.contains("checksum mismatch")));
    }

    #[test]
    fn test_audit_fails_for_shard_count_mismatch() {
        let auditor = ModelAuditor::default();
        let mut input = valid_input();
        input.metadata.shard_count = 3;

        let report = auditor.audit(&input);
        assert_eq!(report.status, AuditStatus::Failed);
        assert!(
            report
                .reasons
                .iter()
                .any(|r| r.contains("shard list count mismatch"))
        );
    }
}
