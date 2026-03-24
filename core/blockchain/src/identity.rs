#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NfmId {
    pub address: String,       // Alamat teknis nfm_...
    pub social_handle: String, // Username @handle
    pub is_soulbound: bool,    // Status tidak bisa dipindah (ID #1)
    pub nonce: u64,            // Urutan akun
}

impl NfmId {
    /// Membuat NFM-ID baru berdasarkan public key dan nonce (Legacy/Simulated)
    #[allow(dead_code)]
    pub fn new(public_key: &str, handle: &str, nonce: u64) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", public_key, nonce).as_bytes());
        let hash = hasher.finalize();
        let address = format!("nfm_{}", &hex::encode(hash)[..32]);

        Self::new_with_address(&address, handle, nonce)
    }

    /// Membuat NFM-ID dengan alamat yang sudah ditentukan (Kriptografis)
    pub fn new_with_address(address: &str, handle: &str, nonce: u64) -> Self {
        let is_soulbound = nonce == 1;
        Self {
            address: address.to_string(),
            social_handle: format!("@{}", handle.trim_start_matches('@')),
            is_soulbound,
            nonce,
        }
    }

    /// Validasi keamanan: Apakah ID ini bisa ditransfer di Marketplace?
    #[allow(dead_code)]
    pub fn can_be_traded(&self) -> bool {
        !self.is_soulbound
    }
}

// ======================================================================
// NEWCOMER REGISTRY (Quest Cap: 20.000 pendaftar pertama)
// Sesuai: docs/gamification_and_quests.md "[Awal] The Newcomer"
// ======================================================================

/// Konfigurasi kuota Newcomer Quest
const NEWCOMER_CAP: u32 = 20_000;
/// Reward untuk Newcomer Quest (NVCoin)
const NEWCOMER_REWARD: f64 = 1_000.0;

/// Registry untuk melacak pendaftar Newcomer
#[allow(dead_code)]
pub struct NewcomerRegistry {
    /// Jumlah newcomer yang sudah terdaftar
    pub registered_count: u32,
    /// Addresses yang sudah klaim Newcomer Quest
    pub registered_addresses: std::collections::HashSet<String>,
}

#[allow(dead_code)]
impl NewcomerRegistry {
    pub fn new() -> Self {
        Self {
            registered_count: 0,
            registered_addresses: std::collections::HashSet::new(),
        }
    }

    /// Cek apakah kuota Newcomer masih tersedia
    pub fn has_slots(&self) -> bool {
        self.registered_count < NEWCOMER_CAP
    }

    /// Daftarkan newcomer baru. Mengembalikan reward jika berhasil.
    pub fn register_newcomer(&mut self, address: &str) -> Result<f64, String> {
        if !self.has_slots() {
            return Err(format!("Newcomer cap reached ({}/{})", self.registered_count, NEWCOMER_CAP));
        }

        if self.registered_addresses.contains(address) {
            return Err("Address already registered as newcomer".to_string());
        }

        self.registered_count += 1;
        self.registered_addresses.insert(address.to_string());

        println!("[NEWCOMER] #{} registered: {} (Reward: {:.0} NVC)", 
            self.registered_count, address, NEWCOMER_REWARD);

        Ok(NEWCOMER_REWARD)
    }

    /// Sisa slot yang tersedia
    pub fn remaining_slots(&self) -> u32 {
        NEWCOMER_CAP - self.registered_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfm_id_generation() {
        let id = NfmId::new("pub_key_test", "dandi", 1);
        assert!(id.address.starts_with("nfm_"));
        assert_eq!(id.social_handle, "@dandi");
        assert!(id.is_soulbound, "ID #1 must be soulbound");
    }

    #[test]
    fn test_soulbound_constraint() {
        let founder_id = NfmId::new("founder_key", "founder", 1);
        assert!(!founder_id.can_be_traded(), "Founder ID cannot be traded");

        let user_id = NfmId::new("user_key", "user", 2);
        assert!(user_id.can_be_traded(), "Normal user ID should be tradable");
    }
}

