#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Batas aturan Anti-Sybil (Sesuai docs/security_audit.md: "1 IP = 1 Account")
const MAX_ACCOUNTS_PER_DEVICE: usize = 1;
const MFA_THRESHOLD: f64 = 50.0; // Transaksi di atas 50 NVCoin butuh MFA
const MFA_CHALLENGE_VALIDITY_SECS: u64 = 60; // Challenge valid selama 60 detik
const POW_DIFFICULTY: usize = 4; // PoW ringan: 4 karakter prefix "0000"

// ======================================================================
// DEVICE FINGERPRINT [S-02 ENHANCED]
// ======================================================================

/// Sidik jari perangkat (Enhanced Multi-Factor)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceFingerprint {
    pub device_id: String,
    pub ip_address: String,
    pub user_agent: String,
}

impl DeviceFingerprint {
    /// Generate device ID dari IP + User-Agent (Legacy, backward-compatible)
    pub fn generate_id(ip: &str, user_agent: &str) -> String {
        Self::generate_enhanced_id(ip, user_agent, "", "", 0)
    }

    /// Generate device ID dari multiple faktor (Enhanced)
    pub fn generate_enhanced_id(
        ip: &str,
        user_agent: &str,
        screen_resolution: &str,
        timezone: &str,
        hardware_concurrency: u32,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}|{}|{}|{}|{}",
            ip, user_agent, screen_resolution, timezone, hardware_concurrency
        ).as_bytes());
        format!("dev_{}", &hex::encode(hasher.finalize())[..16])
    }
}

// ======================================================================
// REGISTRATION CHALLENGE (PoW Anti-Sybil)
// ======================================================================

/// Challenge PoW ringan untuk registrasi (mencegah spam bot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    pub challenge_id: String,
    pub prefix: String,
    pub difficulty: usize,
    pub created_at: u64,
}

impl RegistrationChallenge {
    /// Buat challenge baru
    pub fn new(device_id: &str) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut hasher = Sha256::new();
        hasher.update(format!("challenge_{}_{}", device_id, timestamp).as_bytes());
        let challenge_id = hex::encode(hasher.finalize())[..16].to_string();

        Self {
            challenge_id,
            prefix: "0".repeat(POW_DIFFICULTY),
            difficulty: POW_DIFFICULTY,
            created_at: timestamp,
        }
    }

    /// Verifikasi solusi PoW
    pub fn verify_solution(&self, nonce: u64) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", self.challenge_id, nonce).as_bytes());
        let hash = hex::encode(hasher.finalize());
        hash.starts_with(&self.prefix)
    }

    /// Mencari solusi PoW (untuk testing/demo)
    pub fn solve(&self) -> u64 {
        let mut nonce: u64 = 0;
        loop {
            if self.verify_solution(nonce) {
                return nonce;
            }
            nonce += 1;
        }
    }
}

// ======================================================================
// MFA (Enhanced Multi-Factor Authentication)
// ======================================================================

/// Metode MFA yang didukung
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MfaMethod {
    /// Simulasi sederhana (dev mode)
    Simulated,
    /// Hook biometrik (sidik jari / wajah melalui NFM Brain)
    Biometric,
    /// One-Time Password
    OTP,
}

/// Info sesi MFA
#[derive(Debug, Clone)]
struct MfaSession {
    pub verified: bool,
    pub method: MfaMethod,
    pub verified_at: u64,
}

/// Hasil verifikasi keamanan
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityVerdict {
    Approved,
    RequiresMfa,
    Blocked(String),
}

// ======================================================================
// NEXUS SECURITY ENGINE
// ======================================================================

/// Nexus Security Engine (Enhanced untuk Phase 2)
pub struct NexusSecurity {
    /// device_id -> list of NFM addresses registered from this device
    device_registry: HashMap<String, Vec<String>>,
    /// nfm_address -> MFA session info
    mfa_sessions: HashMap<String, MfaSession>,
    /// nfm_address -> preferred MFA method
    mfa_methods: HashMap<String, MfaMethod>,
    /// device_id -> reputation score (-100 to +100)
    pub reputation_scores: HashMap<String, i32>,
    /// Pending registration challenges
    pending_challenges: HashMap<String, RegistrationChallenge>,
}

impl NexusSecurity {
    pub fn new() -> Self {
        Self {
            device_registry: HashMap::new(),
            mfa_sessions: HashMap::new(),
            mfa_methods: HashMap::new(),
            reputation_scores: HashMap::new(),
            pending_challenges: HashMap::new(),
        }
    }

    /// Set metode MFA untuk address tertentu
    pub fn set_mfa_method(&mut self, nfm_address: &str, method: MfaMethod) {
        self.mfa_methods.insert(nfm_address.to_string(), method);
    }

    /// Get metode MFA untuk address (default: Simulated)
    pub fn get_mfa_method(&self, nfm_address: &str) -> MfaMethod {
        self.mfa_methods.get(nfm_address).cloned().unwrap_or(MfaMethod::Simulated)
    }

    // ------------------------------------------------------------------
    // REGISTRATION (dengan PoW Challenge)
    // ------------------------------------------------------------------

    /// Step 1: Request registration challenge
    pub fn request_registration_challenge(&mut self, device_id: &str) -> RegistrationChallenge {
        let challenge = RegistrationChallenge::new(device_id);
        self.pending_challenges.insert(device_id.to_string(), challenge.clone());
        challenge
    }

    /// Step 2: Verify PoW solution dan daftarkan akun
    pub fn register_with_challenge(
        &mut self,
        device_id: &str,
        nfm_address: &str,
        nonce: u64,
    ) -> SecurityVerdict {
        // Cek apakah ada challenge pending
        let challenge = match self.pending_challenges.get(device_id) {
            Some(c) => c.clone(),
            None => return SecurityVerdict::Blocked("No pending challenge for this device".to_string()),
        };

        // Verifikasi PoW
        if !challenge.verify_solution(nonce) {
            // Kurangi reputasi device
            let rep = self.reputation_scores.entry(device_id.to_string()).or_insert(0);
            *rep -= 10;
            return SecurityVerdict::Blocked("PoW solution invalid".to_string());
        }

        // Hapus challenge (sekali pakai)
        self.pending_challenges.remove(device_id);

        // Lanjut ke registrasi biasa
        let result = self.check_registration(device_id, nfm_address);

        // Tambah reputasi jika berhasil
        if result == SecurityVerdict::Approved {
            let rep = self.reputation_scores.entry(device_id.to_string()).or_insert(0);
            *rep += 5;
        }

        result
    }

    /// Cek apakah perangkat ini boleh mendaftarkan akun baru (Legacy, tanpa PoW)
    pub fn check_registration(&mut self, device_id: &str, nfm_address: &str) -> SecurityVerdict {
        let accounts = self.device_registry.entry(device_id.to_string()).or_default();

        // Apakah akun ini sudah terdaftar di device ini?
        if accounts.contains(&nfm_address.to_string()) {
            return SecurityVerdict::Approved; // Sudah terdaftar, tidak perlu dicek ulang
        }

        // Cek batas akun per perangkat
        if accounts.len() >= MAX_ACCOUNTS_PER_DEVICE {
            return SecurityVerdict::Blocked(
                format!("Device {} already has {} accounts (max {})", device_id, accounts.len(), MAX_ACCOUNTS_PER_DEVICE)
            );
        }

        // Daftarkan
        accounts.push(nfm_address.to_string());
        SecurityVerdict::Approved
    }

    // ------------------------------------------------------------------
    // MFA (Enhanced: Time-Based Challenge)
    // ------------------------------------------------------------------

    /// Cek apakah transaksi ini membutuhkan MFA
    pub fn check_transaction(&self, nfm_address: &str, amount: f64) -> SecurityVerdict {
        if amount < MFA_THRESHOLD {
            return SecurityVerdict::Approved;
        }

        // Transaksi besar: butuh MFA
        match self.mfa_sessions.get(nfm_address) {
            Some(session) if session.verified => {
                // Cek apakah sesi masih valid (10 menit)
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if now - session.verified_at <= 600 {
                    SecurityVerdict::Approved
                } else {
                    SecurityVerdict::RequiresMfa // Sesi expired
                }
            },
            _ => SecurityVerdict::RequiresMfa,
        }
    }

    /// Generate MFA challenge (Time-based, bukan statis lagi!) [R-01 FIX]
    pub fn generate_mfa_challenge(nfm_address: &str) -> String {
        let epoch_minute = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() / MFA_CHALLENGE_VALIDITY_SECS;

        let mut hasher = Sha256::new();
        hasher.update(format!("mfa:{}:{}", nfm_address, epoch_minute).as_bytes());
        hex::encode(hasher.finalize())[..8].to_string()
    }

    /// Verifikasi MFA response (memeriksa challenge saat ini DAN sebelumnya untuk toleransi waktu)
    pub fn verify_mfa(&mut self, nfm_address: &str, challenge_response: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Cek challenge saat ini dan 1 epoch sebelumnya (toleransi)
        for offset in 0..=1u64 {
            let epoch_minute = (now / MFA_CHALLENGE_VALIDITY_SECS).saturating_sub(offset);
            let mut hasher = Sha256::new();
            hasher.update(format!("mfa:{}:{}", nfm_address, epoch_minute).as_bytes());
            let expected = &hex::encode(hasher.finalize())[..8];

            if challenge_response == expected {
                let method = self.get_mfa_method(nfm_address);
                self.mfa_sessions.insert(nfm_address.to_string(), MfaSession {
                    verified: true,
                    method,
                    verified_at: now,
                });
                return true;
            }
        }

        false
    }

    /// Request biometric verification (hook untuk NFM Brain)
    /// Mengembalikan challenge token unik yang harus diverifikasi dalam 60 detik
    pub fn request_biometric_verification(&mut self, nfm_address: &str) -> String {
        self.set_mfa_method(nfm_address, MfaMethod::Biometric);
        let challenge = Self::generate_mfa_challenge(nfm_address);
        println!("[NEXUS] Biometric verification requested for {}: challenge={}", nfm_address, challenge);
        challenge
    }

    // ------------------------------------------------------------------
    // REPUTATION
    // ------------------------------------------------------------------

    /// Get reputasi device
    pub fn get_reputation(&self, device_id: &str) -> i32 {
        *self.reputation_scores.get(device_id).unwrap_or(&0)
    }

    /// Berapa akun yang terdaftar dari device ini?
    pub fn accounts_on_device(&self, device_id: &str) -> usize {
        self.device_registry.get(device_id).map_or(0, |v| v.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Legacy tests (masih harus lulus) ---

    #[test]
    fn test_sybil_defense_blocks_excess_accounts() {
        let mut security = NexusSecurity::new();
        let device = "dev_laptop_001";

        // 1 akun per device (sesuai docs/security_audit.md: "1 IP = 1 Account")
        assert_eq!(security.check_registration(device, "nfm_user_1"), SecurityVerdict::Approved);

        // Akun ke-2 harus DIBLOKIR
        match security.check_registration(device, "nfm_user_2") {
            SecurityVerdict::Blocked(_) => {} // Expected
            other => panic!("Expected Blocked, got {:?}", other),
        }
    }

    #[test]
    fn test_mfa_required_for_large_transactions() {
        let security = NexusSecurity::new();

        // Transaksi kecil: OK
        assert_eq!(security.check_transaction("nfm_user_1", 10.0), SecurityVerdict::Approved);

        // Transaksi besar: Butuh MFA
        assert_eq!(security.check_transaction("nfm_user_1", 100.0), SecurityVerdict::RequiresMfa);
    }

    #[test]
    fn test_mfa_verification_flow() {
        let mut security = NexusSecurity::new();
        let address = "nfm_test_mfa";

        // Generate challenge
        let challenge = NexusSecurity::generate_mfa_challenge(address);

        // Verifikasi dengan response yang benar
        assert!(security.verify_mfa(address, &challenge));

        // Sekarang transaksi besar harus disetujui
        assert_eq!(security.check_transaction(address, 100.0), SecurityVerdict::Approved);
    }

    #[test]
    fn test_mfa_rejection_on_wrong_response() {
        let mut security = NexusSecurity::new();
        assert!(!security.verify_mfa("nfm_test", "wrong_code"));
    }

    // --- New Phase 2 tests ---

    #[test]
    fn test_enhanced_device_fingerprint() {
        let id1 = DeviceFingerprint::generate_enhanced_id("192.168.1.1", "Chrome", "1920x1080", "UTC+7", 8);
        let id2 = DeviceFingerprint::generate_enhanced_id("192.168.1.1", "Chrome", "1920x1080", "UTC+7", 8);
        let id3 = DeviceFingerprint::generate_enhanced_id("192.168.1.1", "Chrome", "1366x768", "UTC+7", 4);

        assert_eq!(id1, id2); // Same device = same ID
        assert_ne!(id1, id3); // Different resolution/cores = different ID
    }

    #[test]
    fn test_pow_registration_challenge() {
        let challenge = RegistrationChallenge::new("dev_test_001");
        assert_eq!(challenge.difficulty, POW_DIFFICULTY);

        // Solve the challenge
        let nonce = challenge.solve();
        assert!(challenge.verify_solution(nonce));

        // Wrong nonce should fail
        assert!(!challenge.verify_solution(nonce.wrapping_add(1)));
    }

    #[test]
    fn test_register_with_challenge_flow() {
        let mut security = NexusSecurity::new();
        let device = "dev_challenge_test";

        // Step 1: Request challenge
        let challenge = security.request_registration_challenge(device);

        // Step 2: Solve PoW
        let nonce = challenge.solve();

        // Step 3: Register with solution
        let result = security.register_with_challenge(device, "nfm_new_user", nonce);
        assert_eq!(result, SecurityVerdict::Approved);
        assert!(security.get_reputation(device) > 0);
    }

    #[test]
    fn test_register_with_invalid_pow_blocked() {
        let mut security = NexusSecurity::new();
        let device = "dev_bad_actor";

        security.request_registration_challenge(device);
        let result = security.register_with_challenge(device, "nfm_bad", 12345);
        match result {
            SecurityVerdict::Blocked(_) => {},
            other => panic!("Expected Blocked, got {:?}", other),
        }
        assert!(security.get_reputation(device) < 0);
    }

    #[test]
    fn test_biometric_hook() {
        let mut security = NexusSecurity::new();
        let address = "nfm_biometric_user";

        // Request biometric verification
        let challenge = security.request_biometric_verification(address);
        assert_eq!(security.get_mfa_method(address), MfaMethod::Biometric);

        // Verify with the challenge
        assert!(security.verify_mfa(address, &challenge));
        assert_eq!(security.check_transaction(address, 100.0), SecurityVerdict::Approved);
    }

    #[test]
    fn test_mfa_method_default() {
        let security = NexusSecurity::new();
        assert_eq!(security.get_mfa_method("nfm_unknown"), MfaMethod::Simulated);
    }

    #[test]
    fn test_legacy_fingerprint_backward_compatible() {
        // Legacy generate_id harus tetap menghasilkan ID yang konsisten
        let id1 = DeviceFingerprint::generate_id("192.168.1.1", "NFM-Client/1.0");
        let id2 = DeviceFingerprint::generate_id("192.168.1.1", "NFM-Client/1.0");
        assert_eq!(id1, id2);
        assert!(id1.starts_with("dev_"));
    }
}

