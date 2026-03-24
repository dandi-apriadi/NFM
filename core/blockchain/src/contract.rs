#![allow(dead_code)]
use std::collections::HashMap;

/// Enum untuk jenis efek yang bisa diberikan oleh Fragment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EffectType {
    ReputationBoost,    // Meningkatkan voting power
    RewardMultiplier,   // Mengalikan hasil staking/mining
    FeeDiscount,        // Diskon biaya transaksi tambahan
}

/// Struktur data untuk efek aktif
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActiveEffect {
    pub effect_type: EffectType,
    pub magnitude: f64,
    pub expiry_block: u32,
}

/// Struktur untuk mengelola Staking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StakingInfo {
    pub amount: f64,
    pub start_block: u32,
    pub last_claim_block: u32,
}

/// Modul kontrak pintar untuk Fragment & Staking
pub struct ContractEngine {
    pub active_effects: HashMap<String, Vec<ActiveEffect>>, // address -> effects
    pub staking_pool: HashMap<String, StakingInfo>,        // address -> staking
}

impl ContractEngine {
    pub fn new() -> Self {
        Self {
            active_effects: HashMap::new(),
            staking_pool: HashMap::new(),
        }
    }

    /// Stake NVCoin ke dalam kontrak
    pub fn stake_nvc(&mut self, address: &str, amount: f64, current_block: u32) -> Result<String, String> {
        if amount <= 0.0 { return Err("Jumlah stake harus > 0".into()); }
        
        let info = self.staking_pool.entry(address.to_string()).or_insert(StakingInfo {
            amount: 0.0,
            start_block: current_block,
            last_claim_block: current_block,
        });

        info.amount += amount;
        Ok(format!("@{} berhasil stake {:.2} NVC. Total stake: {:.2} NVC", address, amount, info.amount))
    }

    /// Hitung yield yang bisa diklaim
    pub fn calculate_yield(&self, address: &str, current_block: u32) -> f64 {
        if let Some(info) = self.staking_pool.get(address) {
            let blocks_elapsed = current_block - info.last_claim_block;
            // Simulasi bunga: 0.01% per blok dari jumlah stake
            return info.amount * 0.0001 * (blocks_elapsed as f64);
        }
        0.0
    }

    /// Aktivasi fragment (Smart Contract Hook)
    pub fn activate_fragment(
        &mut self,
        address: &str,
        fragment_name: &str,
        current_block: u32,
    ) -> Result<String, String> {
        let effect = match fragment_name {
            "System Booster Rare" => ActiveEffect {
                effect_type: EffectType::RewardMultiplier,
                magnitude: 1.5,
                expiry_block: current_block + 10, // Berlaku selama 10 blok
            },
            "Neural Link" => ActiveEffect {
                effect_type: EffectType::ReputationBoost,
                magnitude: 2.0,
                expiry_block: current_block + 5,
            },
            _ => return Err("Fragment ini tidak memiliki kontrak pintar aktif.".into()),
        };

        self.active_effects.entry(address.to_string()).or_insert(Vec::new()).push(effect);
        
        Ok(format!("Fragment '{}' diaktifkan! Efek berlaku hingga blok {}.", fragment_name, current_block + 10))
    }

    /// Bersihkan efek yang sudah kadaluarsa
    pub fn cleanup_expired(&mut self, current_block: u32) {
        for effects in self.active_effects.values_mut() {
            effects.retain(|e| e.expiry_block > current_block);
        }
    }

    /// Dapatkan multiplier reward untuk user tertentu
    pub fn get_reward_multiplier(&self, address: &str) -> f64 {
        let mut mult = 1.0;
        if let Some(effects) = self.active_effects.get(address) {
            for e in effects {
                if let EffectType::RewardMultiplier = e.effect_type {
                    mult *= e.magnitude;
                }
            }
        }
        mult
    }
}

