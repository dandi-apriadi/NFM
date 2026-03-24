#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;

/// Struktur data transparan yang disimpan di dalam Block.data (JSON)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockData {
    pub transactions: Vec<String>,
    pub rewards: Vec<NodeRewardInfo>,
    pub economy: EconomySummary,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeRewardInfo {
    pub address: String,
    pub amount: f64,
    #[serde(default)]
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EconomySummary {
    pub fees_collected: f64,
    pub burned: f64,
    pub epoch_number: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u32,
    pub timestamp: i64,
    pub data: String, // Akan berisi JSON dari BlockData
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u32, data: String, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let input = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.data, self.previous_hash, self.nonce
        );
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Fungsi Penambangan (Mining): Mencari nonce agar hash dimulai dengan 'difficulty' jumlah nol
    pub fn mine(&mut self, difficulty: usize) {
        let prefix = "0".repeat(difficulty);
        println!("[MINING] Calculating hash for block {}...", self.index);
        
        while &self.hash[..difficulty] != prefix {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        
        println!("[MINING] Block {} mined! Nonce: {}, Hash: {}", self.index, self.nonce, self.hash);
    }

    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash()
    }

    /// Validasi blok secara menyeluruh (untuk P2P)
    pub fn validate_block(block: &Block, previous_block: &Block, difficulty: usize) -> Result<(), String> {
        // 1. Cek previous_hash cocok
        if block.previous_hash != previous_block.hash {
            return Err(format!("Previous hash mismatch: expected {}, got {}",
                previous_block.hash, block.previous_hash));
        }

        // 2. Cek index berurutan
        if block.index != previous_block.index + 1 {
            return Err(format!("Index mismatch: expected {}, got {}",
                previous_block.index + 1, block.index));
        }

        // 3. Cek hash valid (bukan data yang dimanipulasi)
        let recalculated = block.calculate_hash();
        if block.hash != recalculated {
            return Err(format!("Hash integrity failed: stored {}, recalculated {}",
                block.hash, recalculated));
        }

        // 4. Cek PoW difficulty
        let prefix = "0".repeat(difficulty);
        if !block.hash.starts_with(&prefix) {
            return Err(format!("PoW difficulty not met: hash {} doesn't start with {}",
                block.hash, prefix));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_validation_accepts_valid() {
        let mut b0 = Block::new(0, "Genesis".into(), "".into());
        b0.mine(2);
        let mut b1 = Block::new(1, "Block 1".into(), b0.hash.clone());
        b1.mine(2);

        assert!(Block::validate_block(&b1, &b0, 2).is_ok());
    }

    #[test]
    fn test_block_validation_rejects_tampered_data() {
        let mut b0 = Block::new(0, "Genesis".into(), "".into());
        b0.mine(2);
        let mut b1 = Block::new(1, "Block 1".into(), b0.hash.clone());
        b1.mine(2);

        // Tamper data setelah mining
        b1.data = "HACKED DATA".to_string();
        assert!(Block::validate_block(&b1, &b0, 2).is_err());
    }

    #[test]
    fn test_block_validation_rejects_wrong_previous_hash() {
        let mut b0 = Block::new(0, "Genesis".into(), "".into());
        b0.mine(2);
        let mut b1 = Block::new(1, "Block 1".into(), "wrong_hash".into());
        b1.mine(2);

        assert!(Block::validate_block(&b1, &b0, 2).is_err());
    }
}

