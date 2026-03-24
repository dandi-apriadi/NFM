#![allow(dead_code)]
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

/// Wallet kriptografis dengan Ed25519 Key Pair
#[derive(Clone)]
pub struct CryptoWallet {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub address: String,
}

impl CryptoWallet {
    /// Buat wallet baru dengan key pair acak
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let address = Self::derive_address(&verifying_key);

        Self { signing_key, verifying_key, address }
    }

    /// Buat wallet dari seed (deterministik, untuk testing)
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        let address = Self::derive_address(&verifying_key);

        Self { signing_key, verifying_key, address }
    }

    /// Derive alamat NFM dari public key (hash SHA-256, ambil 16 karakter pertama)
    fn derive_address(verifying_key: &VerifyingKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.as_bytes());
        let hash = hasher.finalize();
        format!("nfm_{}", hex::encode(&hash[..16]))
    }

    /// Tanda tangani data (bytes)
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Verifikasi tanda tangan
    pub fn verify(verifying_key: &VerifyingKey, message: &[u8], signature: &Signature) -> bool {
        verifying_key.verify(message, signature).is_ok()
    }

    /// Tanda tangani transaksi transfer
    pub fn sign_transfer(&self, to: &str, amount: f64) -> (Vec<u8>, Signature) {
        let payload = format!("TRANSFER:{}:{}:{}", self.address, to, amount);
        let payload_bytes = payload.as_bytes().to_vec();
        let signature = self.sign(&payload_bytes);
        (payload_bytes, signature)
    }

    /// Verifikasi tanda tangan transfer
    pub fn verify_transfer(
        verifying_key: &VerifyingKey,
        from: &str,
        to: &str,
        amount: f64,
        signature: &Signature,
    ) -> bool {
        let payload = format!("TRANSFER:{}:{}:{}", from, to, amount);
        Self::verify(verifying_key, payload.as_bytes(), signature)
    }
}

impl std::fmt::Debug for CryptoWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryptoWallet")
            .field("address", &self.address)
            .field("public_key", &hex::encode(self.verifying_key.as_bytes()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_generation() {
        let wallet = CryptoWallet::generate();
        assert!(wallet.address.starts_with("nfm_"));
        assert_eq!(wallet.address.len(), 4 + 32); // "nfm_" + 32 hex chars
    }

    #[test]
    fn test_deterministic_from_seed() {
        let seed = [42u8; 32];
        let w1 = CryptoWallet::from_seed(&seed);
        let w2 = CryptoWallet::from_seed(&seed);
        assert_eq!(w1.address, w2.address);
    }

    #[test]
    fn test_sign_and_verify() {
        let wallet = CryptoWallet::generate();
        let message = b"Hello NFM!";
        let signature = wallet.sign(message);
        assert!(CryptoWallet::verify(&wallet.verifying_key, message, &signature));
    }

    #[test]
    fn test_forged_signature_rejected() {
        let wallet_a = CryptoWallet::generate();
        let wallet_b = CryptoWallet::generate();

        let message = b"Transfer 100 NVCoin";
        let sig_a = wallet_a.sign(message);

        // Coba verifikasi dengan public key B -> harus GAGAL
        assert!(!CryptoWallet::verify(&wallet_b.verifying_key, message, &sig_a));
    }

    #[test]
    fn test_transfer_signature_flow() {
        let sender = CryptoWallet::generate();
        let receiver = CryptoWallet::generate();

        let (_, signature) = sender.sign_transfer(&receiver.address, 50.0);

        // Verifikasi valid
        assert!(CryptoWallet::verify_transfer(
            &sender.verifying_key, &sender.address, &receiver.address, 50.0, &signature
        ));

        // Verifikasi dengan jumlah diubah -> DITOLAK
        assert!(!CryptoWallet::verify_transfer(
            &sender.verifying_key, &sender.address, &receiver.address, 99.0, &signature
        ));
    }
}

