//! Modul Kriptografi Bersama — ZKP Stub, PQC Placeholder, Hashing Utils
//!
//! Menyediakan utilitas kriptografi yang digunakan oleh seluruh ekosistem NFM.
//! Sesuai dengan: docs/security_audit.md (Bio-ZKP, PQC)

use sha2::{Sha256, Digest};

/// Simulasi Bio-ZKP (Zero-Knowledge Proof untuk biometrik)
///
/// Di production: bukti biometrik diproses lokal, hanya ZK-Proof yang dikirim ke chain.
/// Di alpha: simulasi menggunakan hash.
pub struct BioZkp;

impl BioZkp {
    /// Generate ZK-Proof dari data biometrik (simulasi)
    pub fn generate_proof(biometric_data: &[u8], nonce: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(biometric_data);
        hasher.update(nonce.to_be_bytes());
        format!("bio_zkp_{}", &hex::encode(hasher.finalize())[..32])
    }

    /// Verifikasi ZK-Proof (simulasi)
    pub fn verify_proof(biometric_data: &[u8], nonce: u64, proof: &str) -> bool {
        let expected = Self::generate_proof(biometric_data, nonce);
        expected == proof
    }
}

/// Placeholder untuk Post-Quantum Cryptography (PQC)
///
/// Akan diimplementasikan dengan library PQC sesungguhnya (misal: CRYSTALS-Kyber)
/// saat memasuki fase produksi.
pub struct PqcPlaceholder;

impl PqcPlaceholder {
    /// Simulasi enkripsi PQC
    pub fn encrypt(data: &[u8], public_key: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(public_key.as_bytes());
        hasher.finalize().to_vec()
    }

    /// Status: apakah PQC sudah di-enable
    pub fn is_enabled() -> bool {
        false // Akan diaktifkan di Mainnet
    }
}

/// Utilitas hashing umum
pub struct HashUtils;

impl HashUtils {
    /// SHA256 hash dari string
    pub fn sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// SHA256 hash dari bytes
    pub fn sha256_bytes(input: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input);
        hex::encode(hasher.finalize())
    }

    /// Generate ID deterministik dari multiple inputs
    pub fn generate_id(prefix: &str, inputs: &[&str]) -> String {
        let mut hasher = Sha256::new();
        for (i, input) in inputs.iter().enumerate() {
            hasher.update(format!("{}:{}", i, input).as_bytes());
        }
        format!("{}_{}", prefix, &hex::encode(hasher.finalize())[..24])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bio_zkp_proof_generation_and_verify() {
        let bio_data = b"fingerprint_scan_alice_001";
        let nonce = 12345u64;

        let proof = BioZkp::generate_proof(bio_data, nonce);
        assert!(proof.starts_with("bio_zkp_"));
        assert!(BioZkp::verify_proof(bio_data, nonce, &proof));

        // Wrong biometric should fail
        assert!(!BioZkp::verify_proof(b"wrong_fingerprint", nonce, &proof));
    }

    #[test]
    fn test_bio_zkp_deterministic() {
        let data = b"face_scan_bob";
        let p1 = BioZkp::generate_proof(data, 42);
        let p2 = BioZkp::generate_proof(data, 42);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_pqc_placeholder() {
        assert!(!PqcPlaceholder::is_enabled());
        let encrypted = PqcPlaceholder::encrypt(b"secret data", "pubkey_001");
        assert_eq!(encrypted.len(), 32); // SHA256 output
    }

    #[test]
    fn test_hash_utils() {
        let h1 = HashUtils::sha256("hello");
        let h2 = HashUtils::sha256("hello");
        assert_eq!(h1, h2);
        assert_ne!(h1, HashUtils::sha256("world"));
    }

    #[test]
    fn test_generate_id() {
        let id = HashUtils::generate_id("item", &["gold", "pulse", "42"]);
        assert!(id.starts_with("item_"));
        assert_eq!(id.len(), 5 + 24); // "item_" + 24 chars
    }
}
