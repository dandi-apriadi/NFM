#![allow(dead_code)]
use serde::{Serialize, Deserialize};

/// Konfigurasi konstan untuk Revenue Split
const RECYCLING_RATIO: f64 = 0.60;       // 60% -> Reward Pool (untuk node)
const BONUS_RATIO: f64 = 0.10;           // 10% -> Bonus Pool (Legacy Core owners)
const PROTOCOL_GROWTH_RATIO: f64 = 0.30; // 30% -> Protocol (15% Founder, 10% Hub, 5% Burn)

/// Pool ekonomi NFM
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EconomyPool {
    pub reward_pool: f64,      // Kumpulan dari 60% fee AI (dibagikan saat epoch)
    pub bonus_pool: f64,       // Kumpulan dari 10% fee AI (untuk Legacy Core owners)
    pub protocol_growth: f64,  // Kumpulan dari 30% fee AI (Founder/Hub/Burn)
    pub total_fees_collected: f64,
    pub total_burned: f64,
}

impl EconomyPool {
    pub fn new() -> Self {
        Self {
            reward_pool: 0.0,
            bonus_pool: 0.0,
            protocol_growth: 0.0,
            total_fees_collected: 0.0,
            total_burned: 0.0,
        }
    }

    /// Dipanggil setiap kali user membayar fee AI (misal: bertanya ke AI)
    /// Fee langsung dipecah 60/10/30 ke masing-masing pool
    pub fn collect_ai_fee(&mut self, fee_amount: f64) -> FeeSplitResult {
        let to_reward = fee_amount * RECYCLING_RATIO;
        let to_bonus = fee_amount * BONUS_RATIO;
        let to_protocol = fee_amount * PROTOCOL_GROWTH_RATIO;

        // Pecah protocol growth: 15% Founder, 10% Hub, 5% Burn
        let to_founder = fee_amount * 0.15;
        let to_hub = fee_amount * 0.10;
        let to_burn = fee_amount * 0.05;

        self.reward_pool += to_reward;
        self.bonus_pool += to_bonus;
        self.protocol_growth += to_protocol;
        self.total_fees_collected += fee_amount;
        self.total_burned += to_burn;

        FeeSplitResult {
            fee_paid: fee_amount,
            to_reward_pool: to_reward,
            to_bonus_pool: to_bonus,
            to_founder: to_founder,
            to_hub: to_hub,
            to_burn: to_burn,
        }
    }
}

/// Hasil pemecahan fee AI
#[derive(Debug, Clone)]
pub struct FeeSplitResult {
    pub fee_paid: f64,
    pub to_reward_pool: f64,
    pub to_bonus_pool: f64,
    pub to_founder: f64,
    pub to_hub: f64,
    pub to_burn: f64,
}

/// Node aktif di jaringan
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveNode {
    pub address: String,
    pub work_score: u64,
}

/// Hasil distribusi per node
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeReward {
    pub address: String,
    pub earned: f64,
}

/// Hasil distribusi satu epoch
#[derive(Debug, Clone)]
pub struct EpochResult {
    pub epoch_number: u64,
    pub pool_distributed: f64,
    pub node_rewards: Vec<NodeReward>,
}

/// Mesin distribusi reward per epoch
pub struct RewardEngine {
    pub current_epoch: u64,
}

impl RewardEngine {
    pub fn new() -> Self {
        Self { current_epoch: 0 }
    }

    /// Dipanggil setiap 5 menit: ambil isi reward_pool dan bagikan ke node aktif
    pub fn distribute_epoch(&mut self, pool: &mut EconomyPool, active_nodes: &[ActiveNode]) -> Option<EpochResult> {
        if active_nodes.is_empty() || pool.reward_pool <= 0.0 {
            return None;
        }

        self.current_epoch += 1;
        let distributable = pool.reward_pool;
        pool.reward_pool = 0.0; // Kosongkan pool setelah distribusi

        let total_work: u64 = active_nodes.iter().map(|n| n.work_score).sum();
        let node_rewards: Vec<NodeReward> = active_nodes.iter().map(|node| {
            let share = if total_work > 0 {
                (node.work_score as f64 / total_work as f64) * distributable
            } else {
                0.0
            };
            NodeReward {
                address: node.address.clone(),
                earned: (share * 10000.0).round() / 10000.0,
            }
        }).collect();

        Some(EpochResult {
            epoch_number: self.current_epoch,
            pool_distributed: distributable,
            node_rewards,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_split_60_10_30() {
        let mut pool = EconomyPool::new();
        let result = pool.collect_ai_fee(10.0); // User bayar 10 NVCoin

        assert_eq!(result.to_reward_pool, 6.0);  // 60%
        assert_eq!(result.to_bonus_pool, 1.0);    // 10%
        assert_eq!(result.to_founder, 1.5);        // 15%
        assert_eq!(result.to_hub, 1.0);            // 10%
        assert_eq!(result.to_burn, 0.5);           // 5%
        assert_eq!(pool.reward_pool, 6.0);
    }

    #[test]
    fn test_epoch_distributes_accumulated_pool() {
        let mut pool = EconomyPool::new();
        let mut engine = RewardEngine::new();

        // 5 user masing-masing bayar 10 NVCoin -> pool terkumpul 30 NVCoin (60% dari 50)
        for _ in 0..5 {
            pool.collect_ai_fee(10.0);
        }
        assert_eq!(pool.reward_pool, 30.0);

        let nodes = vec![
            ActiveNode { address: "nfm_node_a".to_string(), work_score: 100 },
            ActiveNode { address: "nfm_node_b".to_string(), work_score: 50 },
        ];

        let result = engine.distribute_epoch(&mut pool, &nodes).unwrap();
        assert_eq!(result.pool_distributed, 30.0);
        assert_eq!(pool.reward_pool, 0.0); // Pool habis setelah distribusi

        // Node A (100/150) mendapat 2/3 dari 30 = 20
        assert_eq!(result.node_rewards[0].earned, 20.0);
        // Node B (50/150) mendapat 1/3 dari 30 = 10
        assert_eq!(result.node_rewards[1].earned, 10.0);
    }

    #[test]
    fn test_no_distribution_on_empty_pool() {
        let mut pool = EconomyPool::new();
        let mut engine = RewardEngine::new();
        let nodes = vec![ActiveNode { address: "nfm_test".to_string(), work_score: 10 }];

        // Pool kosong, tidak ada fee masuk
        let result = engine.distribute_epoch(&mut pool, &nodes);
        assert!(result.is_none(), "Should not distribute when pool is empty");
    }
}

