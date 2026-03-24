#![allow(dead_code)]
use ed25519_dalek::{VerifyingKey, Signature};
use sha2::Digest;
use crate::reward::EconomyPool;
use crate::wallet::CryptoWallet;
use crate::storage::WalletStorage;
use std::collections::HashMap;
use std::sync::Arc;

/// Dynamic Gas Fee berdasarkan kesibukan jaringan
#[derive(Debug, Clone)]
pub struct GasFeeCalculator {
    pub tx_count_this_epoch: u32,
}

impl GasFeeCalculator {
    pub fn new() -> Self {
        Self { tx_count_this_epoch: 0 }
    }

    /// Hitung gas fee berdasarkan jumlah transaksi di epoch ini
    pub fn calculate_fee(&self) -> f64 {
        match self.tx_count_this_epoch {
            0..=9     => 0.01,   // Sepi: hampir gratis
            10..=49   => 0.05,   // Normal
            50..=99   => 0.10,   // Sibuk
            _         => 0.25,   // Sangat sibuk (batas maksimum)
        }
    }

    /// Catat transaksi baru
    pub fn record_tx(&mut self) {
        self.tx_count_this_epoch += 1;
    }

    /// Reset di awal epoch baru
    pub fn reset_epoch(&mut self) {
        self.tx_count_this_epoch = 0;
    }
}

/// Hasil transfer
#[derive(Debug, Clone)]
pub struct TransferResult {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub gas_fee: f64,
    pub total_deducted: f64,
}

/// Wallet Engine (saldo pengguna)
#[derive(Clone)]
pub struct WalletEngine {
    pub balances: HashMap<String, f64>,
    pub gas: GasFeeCalculator,
    pub storage: Option<Arc<WalletStorage>>,
}

impl WalletEngine {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            gas: GasFeeCalculator::new(),
            storage: None,
        }
    }

    pub fn with_storage(storage: Arc<WalletStorage>) -> Self {
        let mut engine = Self::new();
        if let Ok(loaded) = storage.load_all_balances() {
            engine.balances = loaded;
        }
        engine.storage = Some(storage);
        engine
    }

    /// Set saldo awal
    pub fn set_balance(&mut self, address: &str, amount: f64) {
        self.balances.insert(address.to_string(), amount);
        if let Some(s) = &self.storage {
            s.store_balance(address, amount).ok();
        }
    }

    /// Tambah saldo
    pub fn add_balance(&mut self, address: &str, amount: f64) {
        let current = self.get_balance(address);
        let new_balance = current + amount;
        self.balances.insert(address.to_string(), new_balance);
        if let Some(s) = &self.storage {
            s.store_balance(address, new_balance).ok();
        }
    }

    /// Kurangi saldo
    pub fn deduct_balance(&mut self, address: &str, amount: f64) -> Result<(), String> {
        let current = self.get_balance(address);
        if current < amount {
            return Err("Insufficient balance".to_string());
        }
        let new_balance = current - amount;
        self.balances.insert(address.to_string(), new_balance);
        if let Some(s) = &self.storage {
            s.store_balance(address, new_balance).ok();
        }
        Ok(())
    }

    /// Cek saldo
    pub fn get_balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
    }

    /// Transfer NVCoin dari A ke B (WAJIB Tanda Tangan Digital)
    pub fn transfer(
        &mut self,
        from: &str,
        to: &str,
        amount: f64,
        pool: &mut EconomyPool,
        verifying_key: &VerifyingKey,
        signature: &Signature,
    ) -> Result<TransferResult, String> {
        // 1. Verifikasi Tanda Tangan Kriptografis
        if !CryptoWallet::verify_transfer(verifying_key, from, to, amount, signature) {
            return Err("Kriptografi Gagal: Tanda tangan transaksi tidak valid!".to_string());
        }

        // 2. Cek apakah Public Key sesuai dengan alamat pengirim
        // (Alamat harus diturunkan dari public key tersebut)
        let derived_addr = format!("nfm_{}", hex::encode(&sha2::Sha256::digest(verifying_key.as_bytes())[..16]));
        if derived_addr != from {
            return Err("Kriptografi Gagal: Public key tidak cocok dengan alamat pengirim!".to_string());
        }

        let gas_fee = self.gas.calculate_fee();
        let total_needed = amount + gas_fee;

        let sender_balance = self.get_balance(from);
        if sender_balance < total_needed {
            return Err(format!("Insufficient balance: has {:.2}, needs {:.2} (amount {} + gas {})",
                sender_balance, total_needed, amount, gas_fee));
        }

        // Potong dari pengirim
        *self.balances.get_mut(from).unwrap() -= total_needed;

        // Tambah ke penerima
        *self.balances.entry(to.to_string()).or_insert(0.0) += amount;

        // Gas fee masuk ke pool ekonomi (split 60/10/30)
        pool.collect_ai_fee(gas_fee);

        // Catat transaksi
        self.gas.record_tx();

        Ok(TransferResult {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            gas_fee,
            total_deducted: total_needed,
        })
    }

    /// Menghapus entri wallet yang bukan alamat NFM valid (Pembersihan Bug Phase 19)
    pub fn cleanup_malformed_wallets(&mut self) {
        let malformed: Vec<String> = self.balances.keys()
            .filter(|addr| !addr.starts_with("nfm_"))
            .cloned()
            .collect();

        for addr in malformed {
            println!("  [DB] Cleaning up malformed wallet: {}", addr);
            self.balances.remove(&addr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_gas_fee() {
        let mut gas = GasFeeCalculator::new();
        assert_eq!(gas.calculate_fee(), 0.01); // Sepi

        for _ in 0..10 { gas.record_tx(); }
        assert_eq!(gas.calculate_fee(), 0.05); // Normal

        for _ in 0..40 { gas.record_tx(); }
        assert_eq!(gas.calculate_fee(), 0.10); // Sibuk

        for _ in 0..50 { gas.record_tx(); }
        assert_eq!(gas.calculate_fee(), 0.25); // Sangat sibuk

        gas.reset_epoch();
        assert_eq!(gas.calculate_fee(), 0.01); // Kembali sepi
    }

    #[test]
    fn test_transfer_success() {
        let mut wallet_engine = WalletEngine::new();
        let mut pool = EconomyPool::new();

        let sender = CryptoWallet::generate();
        let receiver = CryptoWallet::generate();

        wallet_engine.set_balance(&sender.address, 100.0);
        
        let (_, signature) = sender.sign_transfer(&receiver.address, 30.0);

        let result = wallet_engine.transfer(
            &sender.address, 
            &receiver.address, 
            30.0, 
            &mut pool, 
            &sender.verifying_key, 
            &signature
        ).unwrap();

        assert_eq!(result.amount, 30.0);
        assert!((wallet_engine.get_balance(&sender.address) - 69.99).abs() < 0.001);
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let mut wallet_engine = WalletEngine::new();
        let mut pool = EconomyPool::new();

        let sender = CryptoWallet::generate();
        let receiver = CryptoWallet::generate();

        wallet_engine.set_balance(&sender.address, 5.0);
        let (_, signature) = sender.sign_transfer(&receiver.address, 100.0);

        let result = wallet_engine.transfer(
            &sender.address, 
            &receiver.address, 
            100.0, 
            &mut pool, 
            &sender.verifying_key, 
            &signature
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_gas_fee_goes_to_pool() {
        let mut wallet_engine = WalletEngine::new();
        let mut pool = EconomyPool::new();

        let sender = CryptoWallet::generate();
        let receiver = CryptoWallet::generate();

        wallet_engine.set_balance(&sender.address, 1000.0);
        let (_, signature) = sender.sign_transfer(&receiver.address, 10.0);

        wallet_engine.transfer(
            &sender.address, 
            &receiver.address, 
            10.0, 
            &mut pool, 
            &sender.verifying_key, 
            &signature
        ).unwrap();

        assert!(pool.reward_pool > 0.0);
    }
}

