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
    /// Membuat NFM-ID baru berdasarkan public key (simulated) dan nonce
    pub fn new(public_key: &str, handle: &str, nonce: u64) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", public_key, nonce).as_bytes());
        let hash = hasher.finalize();
        let address = format!("nfm_{}", &hex::encode(hash)[..32]);

        // ID #1 selalu Soulbound secara otomatis
        let is_soulbound = nonce == 1;

        Self {
            address,
            social_handle: format!("@{}", handle.trim_start_matches('@')),
            is_soulbound,
            nonce,
        }
    }

    /// Validasi keamanan: Apakah ID ini bisa ditransfer di Marketplace?
    pub fn can_be_traded(&self) -> bool {
        !self.is_soulbound
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
