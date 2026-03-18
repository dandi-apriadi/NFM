mod block;
mod identity;
mod reward;

use crate::identity::NfmId;
use crate::reward::{EconomyPool, RewardEngine, ActiveNode};
use block::Block;
use std::fs;

struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, "NFM GENESIS BLOCK - The Birth of Sovereign AI".to_string(), "0".to_string());
        Blockchain {
            chain: vec![genesis_block],
        }
    }

    fn add_block(&mut self, mut new_block: Block) {
        new_block.previous_hash = self.get_latest_block().hash.clone();
        new_block.hash = new_block.calculate_hash();
        self.chain.push(new_block);
    }

    fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Chain should not be empty")
    }

    /// Export seluruh blockchain ke file JSON
    fn export_json(&self, path: &str) {
        let json = serde_json::to_string_pretty(&self.chain)
            .expect("Failed to serialize blockchain");
        fs::write(path, &json).expect("Failed to write JSON file");
        println!("[EXPORT] Blockchain saved to: {}", path);
    }
}

fn main() {
    println!("==========================================");
    println!("NFM ALPHA CORE [RUST] - Starting Blockchain");
    println!("==========================================");

    // 1. Inisialisasi Genesis ID (Founder)
    let founder_id = NfmId::new("genesis_pub_key", "founder", 1);
    println!("========================================");
    println!("NFM NODE STARTED");
    println!("Genesis ID: {}", founder_id.address);
    println!("Handle: {}", founder_id.social_handle);
    println!("Status: {}", if founder_id.is_soulbound { "SOULBOUND (LOCKED)" } else { "TRADABLE" });
    println!("========================================");

    // 2. Inisialisasi Blockchain
    let mut blockchain = Blockchain::new();

    // 3. Inisialisasi Economy
    let mut pool = EconomyPool::new();
    let mut engine = RewardEngine::new();

    // 4. Simulasi: Beberapa user menggunakan AI (membayar fee)
    println!("\n[ECONOMY] Simulating AI usage...");
    let users = vec![
        ("@alice", 10.0),
        ("@bob", 15.0),
        ("@charlie", 8.0),
        ("@dave", 20.0),
        ("@eve", 5.0),
    ];

    for (user, fee) in &users {
        let split = pool.collect_ai_fee(*fee);
        println!("  {} paid {} NVCoin -> Pool+{:.1} | Bonus+{:.1} | Founder+{:.1} | Burn+{:.2}",
            user, fee, split.to_reward_pool, split.to_bonus_pool, split.to_founder, split.to_burn);

        blockchain.add_block(Block::new(
            blockchain.chain.len() as u32,
            format!("AI_FEE: {} paid {} NVCoin", user, fee),
            "".to_string(),
        ));
    }

    println!("\n[POOL STATUS] Reward Pool: {:.1} NVCoin | Bonus Pool: {:.1} NVCoin | Total Burned: {:.2} NVCoin",
        pool.reward_pool, pool.bonus_pool, pool.total_burned);

    // 5. Epoch: Distribusi reward pool ke node aktif
    println!("\n[EPOCH] Distributing reward pool to active nodes...");
    let active_nodes = vec![
        ActiveNode { address: founder_id.address.clone(), work_score: 100 },
        ActiveNode { address: "nfm_user_002_abcdef1234567890".to_string(), work_score: 50 },
        ActiveNode { address: "nfm_user_003_fedcba0987654321".to_string(), work_score: 30 },
    ];

    if let Some(result) = engine.distribute_epoch(&mut pool, &active_nodes) {
        println!("--- EPOCH {} ---", result.epoch_number);
        println!("Pool Distributed: {:.1} NVCoin", result.pool_distributed);
        for nr in &result.node_rewards {
            println!("  {} -> {:.4} NVCoin", nr.address, nr.earned);
        }

        blockchain.add_block(Block::new(
            blockchain.chain.len() as u32,
            format!("EPOCH_{}: Distributed {:.1} NVCoin to {} nodes", result.epoch_number, result.pool_distributed, result.node_rewards.len()),
            "".to_string(),
        ));
    }

    // 6. Export blockchain ke JSON
    blockchain.export_json("blockchain_output.json");

    println!("\n==========================================");
    println!("NFM Alpha Core - Shutdown");
    println!("==========================================");
}
