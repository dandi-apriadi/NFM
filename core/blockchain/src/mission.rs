#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

// ======================================================================
// MISSION LIFECYCLE: Available -> InProgress -> PendingVerification -> Completed
// Sesuai: docs/gamification_and_quests.md
// ======================================================================

/// Definisi tingkat kesulitan Misi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

/// Tipe pekerjaan yang diperlukan oleh misi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkType {
    /// Pekerjaan komputasi AI (menyumbangkan daya GPU/CPU)
    AiComputation { required_cycles: u64 },
    /// Verifikasi integritas shard di jaringan
    ShardVerification { required_shards: u32 },
    /// Partisipasi dalam konsensus antar-peer 
    ConsensusParticipation { required_rounds: u32 },
}

impl WorkType {
    /// Estimasi waktu minimum penyelesaian dalam detik
    pub fn min_duration_secs(&self) -> u64 {
        match self {
            WorkType::AiComputation { required_cycles } => required_cycles / 1000, // ~1s per 1000 cycles
            WorkType::ShardVerification { required_shards } => (*required_shards as u64) * 2,
            WorkType::ConsensusParticipation { required_rounds } => (*required_rounds as u64) * 5,
        }
    }

    /// Kebutuhan unit kerja aktual untuk misi
    pub fn required_units(&self) -> u64 {
        match self {
            WorkType::AiComputation { required_cycles } => *required_cycles,
            WorkType::ShardVerification { required_shards } => *required_shards as u64,
            WorkType::ConsensusParticipation { required_rounds } => *required_rounds as u64,
        }
    }
}

/// Status lifecycle sebuah assignment misi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MissionStatus {
    /// Misi tersedia untuk diambil
    Available,
    /// Sedang dikerjakan oleh node
    InProgress,
    /// Pekerjaan selesai, menunggu verifikasi proof
    PendingVerification,
    /// Sudah diverifikasi dan reward diklaim
    Completed,
}

/// Struktur data Misi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub reward_nvc: f64,
    pub reward_item: Option<String>,
    pub work_type: WorkType,
}

/// Bukti pekerjaan yang di-submit oleh node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProof {
    /// Hash dari hasil komputasi
    pub result_hash: String,
    /// Jumlah siklus yang di-klaim telah dikerjakan
    pub cycles_completed: u64,
    /// Timestamp mulai (unix secs)
    pub started_at: u64,
    /// Timestamp selesai (unix secs)
    pub completed_at: u64,
    /// Nonce yang digunakan untuk proof
    pub nonce: u64,
}

/// Tracking assignment misi per-user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionAssignment {
    pub mission_id: u32,
    pub address: String,
    pub status: MissionStatus,
    pub started_at: u64,       // Unix timestamp
    pub current_units: u64,
    pub required_units: u64,
    pub proof: Option<WorkProof>,
}

/// Mesin pengelola misi dengan verifikasi kerja
pub struct MissionEngine {
    pub available_missions: Vec<Mission>,
    pub completed_missions: HashMap<String, std::collections::HashSet<u32>>,
    /// Active assignments: key = "address:mission_id"
    pub active_assignments: HashMap<String, MissionAssignment>,
    /// Work units contributed per address since last epoch block
    /// key = address, value = total NVC earned from missions this epoch
    pub contribution_tracker: HashMap<String, f64>,
    /// User Inventory tracking won items
    pub user_inventory: HashMap<String, Vec<String>>,
}

impl MissionEngine {
    pub fn new() -> Self {
        let missions = vec![
            Mission {
                id: 1,
                name: "[Awal] The Newcomer".into(),
                description: "Selesaikan 10 task pertama Anda. Kuota: 20.000 pendaftar pertama.".into(),
                difficulty: Difficulty::Easy,
                reward_nvc: 100.0,
                reward_item: None,
                work_type: WorkType::AiComputation { required_cycles: 10_000 },
            },
            Mission {
                id: 2,
                name: "[Awal] The First Fragment".into(),
                description: "Selesaikan komputasi shard pertama Anda.".into(),
                difficulty: Difficulty::Easy,
                reward_nvc: 10.0,
                reward_item: Some("Badge Pioneer".to_string()),
                work_type: WorkType::ShardVerification { required_shards: 1 },
            },
            Mission {
                id: 3,
                name: "[HARD] Eternal Guardian".into(),
                description: "Node nonstop 30 hari (dikonversi ke 8640 siklus proof). Reward: Legacy Core!".into(),
                difficulty: Difficulty::Expert,
                reward_nvc: 500.0,
                reward_item: Some("Legacy Core".to_string()),
                work_type: WorkType::ConsensusParticipation { required_rounds: 8640 },
            },
            Mission {
                id: 4,
                name: "Brain: P2P Orchestration & Shard Balancing".into(),
                description: "Verifikasi integritas shard yang terfragmentasi secara massif.".into(),
                difficulty: Difficulty::Medium,
                reward_nvc: 150.0,
                reward_item: None,
                work_type: WorkType::ShardVerification { required_shards: 50 },
            },
            Mission {
                id: 5,
                name: "System Defense: Attack Detection".into(),
                description: "Identifikasi pola serangan Sybil dengan komputasi AI mendalam.".into(),
                difficulty: Difficulty::Hard,
                reward_nvc: 300.0,
                reward_item: None,
                work_type: WorkType::AiComputation { required_cycles: 50_000 },
            },
            Mission {
                id: 6,
                name: "PoUW: Dataset Sanitization".into(),
                description: "Bantu membedakan data internet berkualitas tinggi dari noise/spam.".into(),
                difficulty: Difficulty::Medium,
                reward_nvc: 200.0,
                reward_item: None,
                work_type: WorkType::AiComputation { required_cycles: 25_000 },
            },
        ];

        Self {
            available_missions: missions,
            completed_missions: HashMap::new(),
            active_assignments: HashMap::new(),
            contribution_tracker: HashMap::new(),
            user_inventory: HashMap::new(),
        }
    }

    /// Kunci assignment unik
    fn assignment_key(address: &str, mission_id: u32) -> String {
        format!("{}:{}", address, mission_id)
    }

    /// STEP 1: Mulai mengerjakan misi
    pub fn start_mission(&mut self, address: &str, mission_id: u32) -> Result<MissionAssignment, String> {
        // Cek apakah misi ada
        let mission = self.available_missions.iter()
            .find(|m| m.id == mission_id)
            .ok_or("Misi tidak ditemukan")?;

        // Cek apakah sudah pernah selesai
        if let Some(completed) = self.completed_missions.get(address) {
            if completed.contains(&mission_id) {
                return Err("Misi sudah pernah diselesaikan oleh address ini".into());
            }
        }

        let key = Self::assignment_key(address, mission_id);

        // Cek apakah sudah sedang dikerjakan
        if let Some(existing) = self.active_assignments.get(&key) {
            if existing.status == MissionStatus::InProgress || existing.status == MissionStatus::PendingVerification {
                return Err(format!("Misi '{}' sudah sedang dikerjakan", mission.name));
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let assignment = MissionAssignment {
            mission_id,
            address: address.to_string(),
            status: MissionStatus::InProgress,
            started_at: now,
            current_units: 0,
            required_units: mission.work_type.required_units(),
            proof: None,
        };

        self.active_assignments.insert(key, assignment.clone());

        println!("[MISSION] @{} mulai mengerjakan: {} (min {}s)", 
            address, mission.name, mission.work_type.min_duration_secs());

        Ok(assignment)
    }

    /// STEP 1.5: Laporkan progres kerja aktual
    pub fn report_progress(&mut self, address: &str, mission_id: u32, units_delta: u64) -> Result<MissionAssignment, String> {
        if units_delta == 0 {
            return Err("units_delta harus > 0".into());
        }

        let key = Self::assignment_key(address, mission_id);
        let assignment = self.active_assignments.get_mut(&key)
            .ok_or("Belum memulai misi ini. Gunakan start_mission terlebih dahulu.")?;

        if assignment.status != MissionStatus::InProgress {
            return Err(format!(
                "Status misi tidak valid: {:?}. Harus InProgress.",
                assignment.status
            ));
        }

        assignment.current_units = assignment.current_units.saturating_add(units_delta);
        if assignment.current_units > assignment.required_units {
            assignment.current_units = assignment.required_units;
        }

        Ok(assignment.clone())
    }

    /// STEP 2: Submit bukti pekerjaan untuk verifikasi
    pub fn submit_proof(&mut self, address: &str, mission_id: u32, proof: WorkProof) -> Result<String, String> {
        let key = Self::assignment_key(address, mission_id);

        let assignment = self.active_assignments.get_mut(&key)
            .ok_or("Belum memulai misi ini. Gunakan start_mission terlebih dahulu.")?;

        if assignment.status != MissionStatus::InProgress {
            return Err(format!("Status misi tidak valid: {:?}. Harus InProgress.", assignment.status));
        }

        // Verifikasi proof
        let mission = self.available_missions.iter()
            .find(|m| m.id == mission_id)
            .ok_or("Misi tidak ditemukan")?;

        // --- CHEAT DETECTION ---

        // 1. Cek durasi minimum (anti speed-hack)
        let min_duration = mission.work_type.min_duration_secs();
        let actual_duration = proof.completed_at.saturating_sub(proof.started_at);
        if actual_duration < min_duration {
            return Err(format!(
                "Suspiciously fast! Completed in {}s, minimum is {}s. Proof rejected.",
                actual_duration, min_duration
            ));
        }

        // 2. Cek apakah cycles yang diklaim cukup
        match &mission.work_type {
            WorkType::AiComputation { required_cycles } => {
                if proof.cycles_completed < *required_cycles {
                    return Err(format!(
                        "Insufficient computation: {}/{} cycles completed",
                        proof.cycles_completed, required_cycles
                    ));
                }
            },
            WorkType::ShardVerification { required_shards } => {
                if proof.cycles_completed < (*required_shards as u64) {
                    return Err(format!(
                        "Insufficient shards verified: {}/{}", 
                        proof.cycles_completed, required_shards
                    ));
                }
            },
            WorkType::ConsensusParticipation { required_rounds } => {
                if proof.cycles_completed < (*required_rounds as u64) {
                    return Err(format!(
                        "Insufficient rounds: {}/{}", 
                        proof.cycles_completed, required_rounds
                    ));
                }
            },
        }

        // 3. Verifikasi result_hash (proof-of-work style)
        let expected_prefix = Self::compute_expected_hash(address, mission_id, proof.nonce);
        if proof.result_hash != expected_prefix {
            return Err("Invalid proof hash. Computation result does not match.".into());
        }

        // Proof valid! Update status
        assignment.proof = Some(proof);
        assignment.status = MissionStatus::PendingVerification;

        println!("[MISSION] @{} submitted valid proof for: {}", address, mission.name);

        Ok(format!("Proof accepted for '{}'. Ready to claim.", mission.name))
    }

    /// STEP 3: Klaim reward setelah proof terverifikasi
    pub fn claim_reward(&mut self, address: &str, mission_id: u32) -> Result<f64, String> {
        let key = Self::assignment_key(address, mission_id);

        let assignment = self.active_assignments.get_mut(&key)
            .ok_or("Tidak ada assignment aktif untuk misi ini")?;

        if assignment.status != MissionStatus::PendingVerification {
            return Err(format!(
                "Tidak bisa klaim: status = {:?}. Harus PendingVerification (submit proof dulu).",
                assignment.status
            ));
        }

        let mission = self.available_missions.iter()
            .find(|m| m.id == mission_id)
            .ok_or("Misi tidak ditemukan")?;

        // Mark completed
        assignment.status = MissionStatus::Completed;

        // Record in completed set
        let completed_set = self.completed_missions
            .entry(address.to_string())
            .or_insert_with(std::collections::HashSet::new);
        completed_set.insert(mission_id);

        // Record contribution for epoch reward distribution (Proof-of-Contribution)
        *self.contribution_tracker.entry(address.to_string()).or_insert(0.0) += mission.reward_nvc;

        println!("[MISSION] @{} claimed reward: {} (+{:.2} NVC) | Contribution tracked for epoch.", 
            address, mission.name, mission.reward_nvc);

        Ok(mission.reward_nvc)
    }

    /// Legacy: complete_mission sekarang mengarahkan ke error
    pub fn complete_mission(&mut self, address: &str, mission_id: u32) -> Result<f64, String> {
        // Cek apakah ada proof yang sudah PendingVerification
        let key = Self::assignment_key(address, mission_id);
        if let Some(assignment) = self.active_assignments.get(&key) {
            if assignment.status == MissionStatus::PendingVerification {
                return self.claim_reward(address, mission_id);
            }
        }
        Err("Tidak bisa langsung klaim reward. Gunakan alur: start_mission -> submit_proof -> claim_reward".into())
    }

    /// Compute expected proof hash (deterministic dari address + mission_id + nonce)
    pub fn compute_expected_hash(address: &str, mission_id: u32, nonce: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("nfm_work:{}:{}:{}", address, mission_id, nonce).as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Cek status assignment untuk address tertentu
    pub fn get_assignment_status(&self, address: &str, mission_id: u32) -> Option<&MissionAssignment> {
        let key = Self::assignment_key(address, mission_id);
        self.active_assignments.get(&key)
    }

    /// Dapatkan semua assignment aktif untuk address
    pub fn get_user_assignments(&self, address: &str) -> Vec<&MissionAssignment> {
        self.active_assignments.values()
            .filter(|a| a.address == address)
            .collect()
    }

    /// Reset kontribusi setelah epoch block diproduksi
    /// Dipanggil dari main loop setiap 5 menit
    pub fn clear_contributions(&mut self) {
        let count = self.contribution_tracker.len();
        self.contribution_tracker.clear();
        if count > 0 {
            println!("[MISSION] Contribution tracker reset for new epoch ({} contributors).", count);
        }
    }

    /// Total contribution units this epoch (untuk kalkulasi proporsi)
    pub fn total_contribution(&self) -> f64 {
        self.contribution_tracker.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cannot_complete_without_starting() {
        let mut engine = MissionEngine::new();
        let result = engine.complete_mission("nfm_test_addr", 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Tidak bisa langsung klaim"));
    }

    #[test]
    fn test_cannot_claim_without_proof() {
        let mut engine = MissionEngine::new();

        // Start misi
        engine.start_mission("nfm_test", 1).unwrap();

        // Coba langsung klaim tanpa submit proof
        let result = engine.claim_reward("nfm_test", 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Harus PendingVerification"));
    }

    #[test]
    fn test_speed_hack_rejected() {
        let mut engine = MissionEngine::new();
        engine.start_mission("nfm_hacker", 1).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs();

        // Submit proof yang selesai terlalu cepat (0 detik)
        let nonce = 42u64;
        let hash = MissionEngine::compute_expected_hash("nfm_hacker", 1, nonce);
        let proof = WorkProof {
            result_hash: hash,
            cycles_completed: 5000,
            started_at: now,
            completed_at: now, // 0 detik! Terlalu cepat
            nonce,
        };

        let result = engine.submit_proof("nfm_hacker", 1, proof);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Suspiciously fast"));
    }

    #[test]
    fn test_insufficient_cycles_rejected() {
        let mut engine = MissionEngine::new();
        engine.start_mission("nfm_lazy", 1).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs();

        let nonce = 99u64;
        let hash = MissionEngine::compute_expected_hash("nfm_lazy", 1, nonce);
        let proof = WorkProof {
            result_hash: hash,
            cycles_completed: 100, // Butuh 10000, cuma submit 100
            started_at: now - 60,
            completed_at: now,
            nonce,
        };

        let result = engine.submit_proof("nfm_lazy", 1, proof);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient computation"));
    }

    #[test]
    fn test_invalid_proof_hash_rejected() {
        let mut engine = MissionEngine::new();
        engine.start_mission("nfm_faker", 1).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs();

        let proof = WorkProof {
            result_hash: "fakehash1234567890".into(), // Hash palsu
            cycles_completed: 10000,
            started_at: now - 60,
            completed_at: now,
            nonce: 42,
        };

        let result = engine.submit_proof("nfm_faker", 1, proof);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid proof hash"));
    }

    #[test]
    fn test_successful_full_flow() {
        let mut engine = MissionEngine::new();

        // Step 1: Start
        let assignment = engine.start_mission("nfm_worker", 1).unwrap();
        assert_eq!(assignment.status, MissionStatus::InProgress);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs();

        // Step 2: Submit valid proof
        let nonce = 777u64;
        let hash = MissionEngine::compute_expected_hash("nfm_worker", 1, nonce);
        let proof = WorkProof {
            result_hash: hash,
            cycles_completed: 12000, // > 10000 required
            started_at: now - 30,   // 30 detik lalu (> 5s minimum)
            completed_at: now,
            nonce,
        };

        let msg = engine.submit_proof("nfm_worker", 1, proof).unwrap();
        assert!(msg.contains("Proof accepted"));

        // Step 3: Claim reward
        let reward = engine.claim_reward("nfm_worker", 1).unwrap();
        assert_eq!(reward, 100.0);

        // Cek sudah masuk completed
        assert!(engine.completed_missions.get("nfm_worker").unwrap().contains(&1));
    }

    #[test]
    fn test_cannot_redo_completed_mission() {
        let mut engine = MissionEngine::new();
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs();

        // Complete mission 1
        engine.start_mission("nfm_done", 1).unwrap();
        let nonce = 1u64;
        let hash = MissionEngine::compute_expected_hash("nfm_done", 1, nonce);
        engine.submit_proof("nfm_done", 1, WorkProof {
            result_hash: hash, cycles_completed: 10000,
            started_at: now - 30, completed_at: now, nonce,
        }).unwrap();
        engine.claim_reward("nfm_done", 1).unwrap();

        // Try to start again
        let result = engine.start_mission("nfm_done", 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sudah pernah diselesaikan"));
    }
}


