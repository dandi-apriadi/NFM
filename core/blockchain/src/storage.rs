#![allow(dead_code)]
use crate::block::Block;
use sled::Db;

/// Persistent storage untuk blockchain menggunakan sled
pub struct BlockStorage {
    db: Db,
}

impl BlockStorage {
    /// Buka atau buat database di path tertentu
    pub fn open(path: &str) -> Result<Self, String> {
        let db = sled::open(path).map_err(|e| format!("Failed to open DB: {}", e))?;
        Ok(Self { db })
    }

    /// Simpan satu blok ke database (key = index)
    pub fn store_block(&self, block: &Block) -> Result<(), String> {
        let key = block.index.to_be_bytes();
        let value = serde_json::to_vec(block).map_err(|e| e.to_string())?;
        self.db.insert(key, value).map_err(|e| e.to_string())?;
        // self.db.flush() removed to unlock Jutaan TPS (MTPS) auto-batching
        Ok(())
    }

    /// Ambil blok berdasarkan index
    pub fn get_block(&self, index: u32) -> Result<Option<Block>, String> {
        let key = index.to_be_bytes();
        match self.db.get(key).map_err(|e| e.to_string())? {
            Some(data) => {
                let block: Block = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
                Ok(Some(block))
            },
            None => Ok(None),
        }
    }

    /// Ambil seluruh chain dari database
    pub fn load_chain(&self) -> Result<Vec<Block>, String> {
        let mut chain = Vec::new();
        for result in self.db.iter() {
            let (_, value) = result.map_err(|e| e.to_string())?;
            let block: Block = serde_json::from_slice(&value).map_err(|e| e.to_string())?;
            chain.push(block);
        }
        chain.sort_by_key(|b| b.index);
        Ok(chain)
    }

    /// Jumlah blok di database
    pub fn block_count(&self) -> usize {
        self.db.len()
    }

    /// Hapus database (untuk testing)
    pub fn clear(&self) -> Result<(), String> {
        self.db.clear().map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Persistent storage untuk saldo wallet
pub struct WalletStorage {
    db: Db,
}

impl WalletStorage {
    pub fn open(path: &str) -> Result<Self, String> {
        let db = sled::open(path).map_err(|e| format!("Failed to open Wallet DB: {}", e))?;
        Ok(Self { db })
    }

    pub fn store_balance(&self, address: &str, balance: f64) -> Result<(), String> {
        let key = address.as_bytes();
        let value = balance.to_be_bytes();
        self.db.insert(key, &value).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_balance(&self, address: &str) -> Result<f64, String> {
        let key = address.as_bytes();
        match self.db.get(key).map_err(|e| e.to_string())? {
            Some(value) => {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&value);
                Ok(f64::from_be_bytes(bytes))
            },
            None => Ok(0.0),
        }
    }

    pub fn load_all_balances(&self) -> Result<std::collections::HashMap<String, f64>, String> {
        let mut balances = std::collections::HashMap::new();
        for result in self.db.iter() {
            let (key, value) = result.map_err(|e| e.to_string())?;
            let address = String::from_utf8(key.to_vec()).map_err(|e| e.to_string())?;
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&value);
            let balance = f64::from_be_bytes(bytes);
            balances.insert(address, balance);
        }
        Ok(balances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_store_and_retrieve_block() {
        let path = "test_db_store";
        let _ = fs::remove_dir_all(path);

        let storage = BlockStorage::open(path).unwrap();
        let block = Block::new(0, "Genesis".to_string(), "".to_string());

        storage.store_block(&block).unwrap();
        let retrieved = storage.get_block(0).unwrap().unwrap();

        assert_eq!(retrieved.index, 0);
        assert_eq!(retrieved.data, "Genesis");

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_load_full_chain() {
        let path = "test_db_chain";
        let _ = fs::remove_dir_all(path);

        let storage = BlockStorage::open(path).unwrap();

        let b0 = Block::new(0, "Genesis".into(), "".into());
        let b1 = Block::new(1, "Block 1".into(), b0.hash.clone());
        let b2 = Block::new(2, "Block 2".into(), b1.hash.clone());

        storage.store_block(&b0).unwrap();
        storage.store_block(&b1).unwrap();
        storage.store_block(&b2).unwrap();

        let chain = storage.load_chain().unwrap();
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].index, 0);
        assert_eq!(chain[2].index, 2);

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_persistence_survives_reopen() {
        let path = "test_db_persist";
        let _ = fs::remove_dir_all(path);

        // Session 1: tulis data
        {
            let storage = BlockStorage::open(path).unwrap();
            let block = Block::new(0, "Persistent Block".into(), "".into());
            storage.store_block(&block).unwrap();
        }

        // Session 2: baca kembali (simulasi restart)
        {
            let storage = BlockStorage::open(path).unwrap();
            let block = storage.get_block(0).unwrap().unwrap();
            assert_eq!(block.data, "Persistent Block");
        }

        let _ = fs::remove_dir_all(path);
    }
}

