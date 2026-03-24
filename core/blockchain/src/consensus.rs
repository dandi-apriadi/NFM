use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Definisi Validator dalam sistem DPoS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub total_stake: f64,
    pub is_active: bool,
    pub commission_rate: f64, // Persentase fee yang diambil validator
}

/// Mesin Konsensus DPoS
pub struct ConsensusEngine {
    pub validators: HashMap<String, Validator>,
    pub min_stake_for_validator: f64,
    pub max_validator_slots: usize,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            min_stake_for_validator: 100.0, // Minimal 100 NVC untuk jadi kandidat
            max_validator_slots: 21,       // Standar DPoS (misal 21 slot)
        }
    }

    /// Update status validator berdasarkan snapshot staking terbaru
    pub fn refresh_validator_set(&mut self, staking_pool: &HashMap<String, crate::contract::StakingInfo>) {
        self.validators.clear();
        
        for (addr, info) in staking_pool {
            if info.amount >= self.min_stake_for_validator {
                self.validators.insert(addr.clone(), Validator {
                    address: addr.clone(),
                    total_stake: info.amount,
                    is_active: true,
                    commission_rate: 0.05, // Default 5%
                });
            }
        }
        
        // Sorting berdasarkan stake tertinggi (hanya menyisakan top slots)
        let mut sorted_validators: Vec<_> = self.validators.values().cloned().collect();
        sorted_validators.sort_by(|a, b| b.total_stake.partial_cmp(&a.total_stake).unwrap());
        
        self.validators.clear();
        for v in sorted_validators.into_iter().take(self.max_validator_slots) {
            self.validators.insert(v.address.clone(), v);
        }
    }

    /// Verifikasi apakah sebuah address berhak memvalidasi blok
    pub fn is_authorized_validator(&self, address: &str) -> bool {
        self.validators.contains_key(address)
    }

    /// Hitung "Probability Weight" (Simulasi voting power)
    pub fn get_voting_power(&self, address: &str) -> f64 {
        self.validators.get(address).map(|v| v.total_stake).unwrap_or(0.0)
    }
}
