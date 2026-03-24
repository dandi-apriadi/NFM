//! Proof of Computation (PoC) — Validasi Bukti Kerja AI
//!
//! Mekanisme untuk memverifikasi bahwa node benar-benar melakukan
//! komputasi AI (bukan hanya staking token).
//!
//! Sesuai dengan: docs/sovereign_chain_design.md (DPoS + PoC hybrid)

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Bukti komputasi yang dihasilkan oleh node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationProof {
    /// Address node yang melakukan komputasi
    pub node_address: String,
    /// ID shard yang diproses
    pub shard_id: String,
    /// Hash dari hasil komputasi (output inference/training)
    pub result_hash: String,
    /// Waktu komputasi dalam milidetik
    pub compute_time_ms: u64,
    /// Nonce untuk PoC
    pub nonce: u64,
    /// Timestamp (epoch seconds)
    pub timestamp: i64,
}

impl ComputationProof {
    /// Hitung hash bukti komputasi
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}:{}:{}:{}:{}",
            self.node_address, self.shard_id, self.result_hash, self.compute_time_ms, self.nonce
        ).as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Hasil verifikasi PoC
#[derive(Debug, Clone, PartialEq)]
pub enum PocVerdict {
    Valid { work_score: u64 },
    InvalidProof,
    SuspiciouslyFast,    // Terlalu cepat → kemungkinan cheating
    DuplicateSubmission,
}

/// Engine PoC untuk memverifikasi bukti komputasi node
pub struct PocEngine {
    /// Submitted proofs per epoch: node_address -> Vec<proof_hash>
    submitted_proofs: HashMap<String, Vec<String>>,
    /// Minimum waktu komputasi yang dianggap valid (ms)
    pub min_compute_time_ms: u64,
    /// Work score per proof yang valid
    pub base_work_score: u64,
}

impl PocEngine {
    pub fn new() -> Self {
        Self {
            submitted_proofs: HashMap::new(),
            min_compute_time_ms: 100,  // Minimal 100ms per komputasi
            base_work_score: 10,
        }
    }

    /// Verifikasi bukti komputasi dari node
    pub fn verify_proof(&mut self, proof: &ComputationProof) -> PocVerdict {
        let proof_hash = proof.compute_hash();

        // 1. Cek duplikasi
        let proofs = self.submitted_proofs.entry(proof.node_address.clone()).or_default();
        if proofs.contains(&proof_hash) {
            return PocVerdict::DuplicateSubmission;
        }

        // 2. Cek waktu komputasi (terlalu cepat = curang)
        if proof.compute_time_ms < self.min_compute_time_ms {
            return PocVerdict::SuspiciouslyFast;
        }

        // 3. Verifikasi result_hash tidak kosong
        if proof.result_hash.is_empty() || proof.shard_id.is_empty() {
            return PocVerdict::InvalidProof;
        }

        // 4. Hitung work score berdasarkan waktu komputasi
        // Semakin lama (hingga batas wajar), semakin tinggi score
        let time_factor = (proof.compute_time_ms as f64 / 1000.0).min(10.0); // Cap 10x
        let work_score = (self.base_work_score as f64 * (1.0 + time_factor)) as u64;

        // Catat proof
        proofs.push(proof_hash);

        PocVerdict::Valid { work_score }
    }

    /// Hitung total work score untuk sebuah node di epoch ini
    pub fn get_total_work_score(&self, node_address: &str) -> u64 {
        self.submitted_proofs.get(node_address)
            .map(|proofs| proofs.len() as u64 * self.base_work_score)
            .unwrap_or(0)
    }

    /// Reset untuk epoch baru
    pub fn reset_epoch(&mut self) {
        self.submitted_proofs.clear();
    }

    /// Jumlah proof yang disubmit oleh node
    pub fn proof_count(&self, node_address: &str) -> usize {
        self.submitted_proofs.get(node_address).map(|p| p.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proof(address: &str, shard: &str, time_ms: u64) -> ComputationProof {
        ComputationProof {
            node_address: address.to_string(),
            shard_id: shard.to_string(),
            result_hash: format!("result_{}", shard),
            compute_time_ms: time_ms,
            nonce: 42,
            timestamp: 1700000000,
        }
    }

    #[test]
    fn test_valid_proof() {
        let mut engine = PocEngine::new();
        let proof = make_proof("nfm_node_1", "shard_001", 500);

        match engine.verify_proof(&proof) {
            PocVerdict::Valid { work_score } => {
                assert!(work_score > 0);
            },
            other => panic!("Expected Valid, got {:?}", other),
        }
    }

    #[test]
    fn test_suspiciously_fast_rejected() {
        let mut engine = PocEngine::new();
        let proof = make_proof("nfm_cheater", "shard_001", 10); // Too fast

        assert_eq!(engine.verify_proof(&proof), PocVerdict::SuspiciouslyFast);
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut engine = PocEngine::new();
        let proof = make_proof("nfm_node_1", "shard_001", 500);

        engine.verify_proof(&proof); // First submission
        assert_eq!(engine.verify_proof(&proof), PocVerdict::DuplicateSubmission);
    }

    #[test]
    fn test_epoch_reset() {
        let mut engine = PocEngine::new();
        let proof = make_proof("nfm_node_1", "shard_001", 500);
        engine.verify_proof(&proof);
        assert_eq!(engine.proof_count("nfm_node_1"), 1);

        engine.reset_epoch();
        assert_eq!(engine.proof_count("nfm_node_1"), 0);
    }

    #[test]
    fn test_work_score_increases_with_time() {
        let mut engine = PocEngine::new();
        let fast = make_proof("nfm_fast", "shard_a", 200);
        let slow = make_proof("nfm_slow", "shard_b", 5000);

        let fast_score = match engine.verify_proof(&fast) {
            PocVerdict::Valid { work_score } => work_score,
            _ => panic!("Expected Valid"),
        };

        let slow_score = match engine.verify_proof(&slow) {
            PocVerdict::Valid { work_score } => work_score,
            _ => panic!("Expected Valid"),
        };

        assert!(slow_score > fast_score, "Longer compute should yield higher score");
    }
}
