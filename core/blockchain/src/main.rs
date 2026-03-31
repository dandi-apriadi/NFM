mod block;
mod identity;
mod reward;
mod coupon;
mod item;
mod auction;
mod security;
mod governance;
mod governance_api;
mod network;
mod transfer;
mod api;
mod admin;
mod wallet;
mod storage;
mod consensus;
mod contract;
mod mission;
mod config;

use crate::identity::NfmId;
use crate::reward::{EconomyPool, RewardEngine};
use crate::coupon::{CouponRegistry, CouponType, Coupon};
use crate::security::{NexusSecurity, DeviceFingerprint};
use crate::governance::GovernanceEngine;
use crate::storage::{BlockStorage, WalletStorage};
use crate::transfer::WalletEngine;
use crate::admin::AdminEngine;
use crate::wallet::CryptoWallet;
// use crate::storage::BlockStorage; // Removed duplicate
use crate::contract::ContractEngine;
use crate::mission::MissionEngine;
use block::Block;
use nfm_ai_engine::distributed_brain::{GeoDistributedBrainDb, NodeMeta};
use std::sync::{Arc, Mutex};

struct Blockchain {
    chain: Vec<Block>,
    storage: Option<BlockStorage>,
}

impl Blockchain {
    fn new(storage_path: Option<&str>) -> Self {
        let mut storage = None;
        let mut chain = Vec::new();

        if let Some(path) = storage_path {
            match BlockStorage::open(path) {
                Ok(s) => {
                    match s.load_chain() {
                        Ok(loaded_chain) => {
                            chain = loaded_chain;
                        }
                        Err(e) => {
                            println!("[CHAIN][WARN] Gagal memuat chain dari database: {}", e);
                            println!("[CHAIN][WARN] Menjalankan recovery dengan chain kosong (genesis akan dibuat jika perlu)");
                        }
                    }
                    storage = Some(s);
                }
                Err(e) => {
                    println!("[CHAIN][WARN] Gagal membuka database blockchain: {}", e);
                    println!("[CHAIN][WARN] Menjalankan node tanpa storage persisten untuk sesi ini");
                }
            }
        }

        if chain.is_empty() {
            println!("[CHAIN] No database found. Ready for Genesis.");
        } else {
            println!("[CHAIN] Loaded {} blocks from database", chain.len());
        }

        Blockchain { chain, storage }
    }

    fn create_genesis(&mut self, founder_address: &str) {
        if !self.chain.is_empty() { return; }

        println!("[CHAIN] Creating Structured Genesis Block...");
        let genesis_data = crate::block::BlockData {
            transactions: vec!["GENESIS_REWARD: @founder (+100 NVC)".to_string()],
            rewards: vec![crate::block::NodeRewardInfo { 
                address: founder_address.to_string(), 
                amount: 100.0,
                category: "Genesis Provision".to_string(),
            }],
            economy: crate::block::EconomySummary { 
                fees_collected: 0.0, 
                burned: 0.0, 
                epoch_number: 0 
            },
        };
        let genesis_json = serde_json::to_string(&genesis_data).unwrap_or_default();
        let genesis = Block::new(0, genesis_json, "0".to_string());
        
        if let Some(ref s) = self.storage {
            let _ = s.store_block(&genesis);
        }
        self.chain.push(genesis);
    }

    fn add_block(&mut self, mut new_block: Block) {
        new_block.previous_hash = self.get_latest_block().hash.clone();
        new_block.mine(2);
        
        if let Some(ref s) = self.storage {
            s.store_block(&new_block).ok();
        }
        self.chain.push(new_block);
    }

    fn next_index(&self) -> u32 { self.chain.len() as u32 }

    fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Chain should not be empty")
    }

    fn get_epoch_reward(&self) -> f64 {
        let height = self.chain.len() as u32;
        let halvings = height / 420480;
        if halvings >= 64 { return 0.0; }
        100.0 / (2.0f64.powi(halvings as i32))
    }
}

fn main() {
    println!("==========================================");
    println!("  NFM ALPHA CORE [RUST] v1.0 - MESH");
    println!("==========================================");

    // Load centralized configuration
    let node_config = config::NodeConfig::from_env();
    node_config.print_summary();

    // PERSISTENCE: Gunakan database sled (Data tersimpan di disk!)
    let db_path = &node_config.db_path;
    let mut blockchain = Blockchain::new(Some(db_path));
    
    let mut pool = EconomyPool::new();
    let mut reward_engine = RewardEngine::new();
    let mut registry = CouponRegistry::new();
    let mut security = NexusSecurity::new();
    let mut gov = GovernanceEngine::new();
    let mut admin_engine = AdminEngine::new();
    let mut contracts = ContractEngine::new();
    let missions = MissionEngine::new();
    
    // --- WALLET PERSISTENCE [PHASE 18 FIX] ---
    let wallets = match WalletStorage::open("nfm_wallets.db") {
        Ok(storage) => Arc::new(Mutex::new(WalletEngine::with_storage(Arc::new(storage)))),
        Err(e) => {
            println!("[WALLET][WARN] Gagal membuka wallet storage: {}", e);
            println!("[WALLET][WARN] Menjalankan wallet engine in-memory untuk sesi ini");
            Arc::new(Mutex::new(WalletEngine::new()))
        }
    };
    
    let mut consensus = consensus::ConsensusEngine::new();

    // =============================================
    // PHASE 1: IDENTITY & CRYPTO WALLETS
    // =============================================
    println!("\n--- PHASE 1: CRYPTO WALLETS ---");

    // Generate Founder wallet deterministically from a fixed seed
    let founder_wallet = CryptoWallet::from_seed(&[1u8; 32]);
    let founder = NfmId::new_with_address(&founder_wallet.address, "founder", 1);
    
    // Ensure Genesis block exists if chain is fresh
    blockchain.create_genesis(&founder.address);

    println!("  @founder [NODE] Wallet: {} [ED25519]", founder.address);
    if blockchain.chain.len() <= 1 {
        println!("  [CRITICAL SECURITY] FOUNDER PRIVATE KEY: {}", founder_wallet.export_private_key_hex());
        println!("  Note: Keep this key safe. You need it to sign transactions from the dashboard.");
    }
    println!("  Note: Additional participants join by creating wallets and contributing.");

    let device_id = DeviceFingerprint::generate_id("192.168.1.1", "NFM-Client/1.0");
    security.check_registration(&device_id, &founder.address);

    // Register initial settings — Founder only
    registry.coupons.push(Coupon::new(CouponType::Founder, &founder.address));
    admin_engine.register_admin(&founder.address);
    gov.register_node(&founder.address);
    
    // --- START DATA SIMULATION (Only if enabled) ---
    if node_config.nfm_simulation {
        // =============================================
        // PHASE 13: THE GAME LOOP (MISSIONS & STAKING)
        // =============================================
        println!("\n--- PHASE 13: THE GAME LOOP ---");
        
        // Founder stakes NVC (simulation demo)
        if let Ok(msg) = contracts.stake_nvc(&founder.address, 100.0, blockchain.next_index()) {
            println!("  {}", msg);
        }

        // Founder pays an AI fee
        let actual_fee = registry.apply_discount(&founder.address, 10.0);
        pool.collect_ai_fee(actual_fee);
        println!("  @founder paid {:.2} NVCoin (AI Fee)", actual_fee);

        blockchain.add_block(Block::new(blockchain.next_index(), format!("SIMULATION: @founder demo tx"), "".into()));
    } else {
        println!("\n[INFO] Simulation Mode DISABLED. Starting with a clean real state.");
        let mut w_lock = wallets.lock().unwrap();
        if w_lock.balances.is_empty() {
            println!("  [DB] Initializing Organic Economy with Genesis Reward (100 NVC to Founder)...");
            w_lock.set_balance(&founder.address, 100.0);
            // No alice/bob initial balances — only Founder at genesis
        } else {
            println!("  [DB] Loaded persistent wallet state.");
        }
    }

    // =============================================
    // PHASE 9: REST API & GAMIFIED DASHBOARD
    // =============================================

    println!("\n--- PHASE 9: REST API DASHBOARD ---");
    let (block_tx, block_rx) = std::sync::mpsc::channel::<String>();

    let api_chain = Arc::new(Mutex::new(blockchain.chain.clone()));
    let api_wallets = wallets.clone(); // Re-use the SAME SHARED state
    let api_fees = Arc::new(Mutex::new(pool.total_fees_collected));
    let api_burned = Arc::new(Mutex::new(pool.total_burned));
    let api_effects = Arc::new(Mutex::new(contracts.active_effects.clone()));
    let api_missions = Arc::new(Mutex::new(missions));
    let shared_staking = Arc::new(Mutex::new(contracts.staking_pool.clone()));
    let api_admin = Arc::new(Mutex::new(admin_engine));
    let api_gov = Arc::new(Mutex::new(gov));

    // API Secret: lire dari environment variable melalui config
    let api_secret = node_config.api_secret.clone();
    let api_rate_limit_enabled = Arc::new(Mutex::new(true));
    let shared_gas = Arc::new(Mutex::new(crate::transfer::GasFeeCalculator::new()));

    let mut alias_map = std::collections::HashMap::new();
    alias_map.insert("@founder".to_string(), founder.address.clone());
    let api_aliases = Arc::new(Mutex::new(alias_map));

    let shared_mempool: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let shared_next_block = Arc::new(Mutex::new(chrono::Utc::now().timestamp() as u64 + 300));
    let shared_brain_db = Arc::new(Mutex::new(GeoDistributedBrainDb::new()));
    {
        let mut brain = shared_brain_db.lock().unwrap();
        brain.register_node(NodeMeta::new(&founder.address, "id", -6.2088, 106.8456));
    }

    let api_state = api::ApiState {
        chain: api_chain.clone(),
        wallets: api_wallets.clone(),
        node_address: founder.address.clone(),
        total_fees: api_fees.clone(),
        total_burned: api_burned.clone(),
        active_effects: api_effects.clone(),
        mission_engine: api_missions.clone(),
        staking_pool: shared_staking.clone(),
        admin_engine: api_admin.clone(),
        governance_engine: api_gov.clone(),
        block_tx: block_tx,
        api_secret,
        rate_limit_enabled: api_rate_limit_enabled,
        gas_fee_calculator: shared_gas.clone(),
        aliases: api_aliases,
        mempool: shared_mempool.clone(),
        next_block_timestamp: shared_next_block.clone(),
        brain_db: shared_brain_db.clone(),
        brain_tokens: Arc::new(Mutex::new(Vec::new())), // Whitelist tokens—empty means open access
    };
    api::start_api_server(api_state, node_config.api_port);
    println!("  Dashboard running at http://127.0.0.1:{}", node_config.api_port);

    // =============================================
    // FINAL: EXPORT & SUMMARY
    // =============================================
    println!("\n--- SYSTEM SUMMARY ---");
    println!("  Total Blocks: {}", blockchain.chain.len());
    println!("  Database: {} (Persistent Sled DB)", db_path);
    println!("  Security: Ed25519 Signatures + P2P Block Verification + HMAC Auth ACTIVE");

    println!("\n==========================================");
    println!("  NFM Mesh v1.0.0 - ALL SYSTEMS ONLINE");
    println!("==========================================");
    println!("  Press Ctrl+C to shutdown.\n");

    let _last_auto_mine = std::time::Instant::now();
    let mut last_epoch_time = std::time::Instant::now();
    let _last_batch_mine = std::time::Instant::now(); // PHASE 20: Batch counter
    // Fase 19: Bersihkan alamat malformed (ID yang terekam sebagai alamat)
    wallets.lock().unwrap().cleanup_malformed_wallets();

    loop {
        // 0. BUFFER API MESSAGES INTO MEMPOOL [PHASE 20: MEMPOOL]
        while let Ok(msg) = block_rx.try_recv() {
            if msg == "COMMAND_NUKE_DATABASE" {
                 println!("\n[SYSTEM] ☢️ NUKE PROTOCOL ACTIVATED! Wiping all databases...");
                 if let Some(ref s) = blockchain.storage { let _ = s.clear(); }
                 blockchain.chain.clear();
                 let genesis_data = crate::block::BlockData {
                     transactions: vec!["GENESIS_REWARD: @founder (+500 NVC)".to_string()],
                     rewards: vec![crate::block::NodeRewardInfo { address: founder.address.clone(), amount: 500.0, category: "Genesis Provision".to_string() }],
                     economy: crate::block::EconomySummary { fees_collected: 0.0, burned: 0.0, epoch_number: 0 },
                 };
                 let genesis_json = serde_json::to_string(&genesis_data).unwrap_or_default();
                 let genesis = Block::new(0, genesis_json, "0".to_string());
                 if let Some(ref s) = blockchain.storage { let _ = s.store_block(&genesis); }
                 blockchain.chain.push(genesis);
                 let mut w_lock = wallets.lock().unwrap();
                 w_lock.balances.clear();
                 w_lock.set_balance(&founder.address, 500.0);
                 drop(w_lock);
                 if let Ok(mut chain_lock) = api_chain.lock() { *chain_lock = blockchain.chain.clone(); }
                 println!("[SYSTEM] Nuke complete. Economy reset to Genesis with 500 NVC Payout.");
            } else {
                let mut m_lock = shared_mempool.lock().unwrap();
                println!("[MEMPOOL] Buffering TX ({} pending): {}", m_lock.len() + 1, msg);
                m_lock.push(msg);
            }
        }

        // 0.1 PERIODIC BATCH MINE REMOVED - Consolidation into 5-min Epoch [PHASE 20]


        // NO LONGER NEEDED: Overwriting api_wallets with stale local state was the bug

        // 1. UPDATE CONSENSUS STATE (DPoS Refresh)

        {
            let staking_lock = shared_staking.lock().unwrap();
            consensus.refresh_validator_set(&*staking_lock);
        }
        let is_authorized = consensus.is_authorized_validator(&founder.address);
        let _voting_power = consensus.get_voting_power(&founder.address);

        // 2. Automated DPoS Mining - Consolidation into 5-min Epoch [PHASE 20]
        if is_authorized {
            // Kita sudah melakukan konsolidasi ke Epoch Distribution di bawah
        }

        // 3. Epoch Distribution & Structured Block Production (Every 5 minutes)
        if last_epoch_time.elapsed().as_secs_f64() >= 300.0 {
            println!("\n[EPOCH] 5 Minutes elapsed. Producing Unified Structured Block...");
            
            // Sync API Fees to Economy Pool [B-01 FIX]
            let fees_this_period;
            {
                let mut fees_lock = api_fees.lock().unwrap();
                fees_this_period = *fees_lock;
                pool.collect_ai_fee(fees_this_period);
                *fees_lock = 0.0;
            }
            
            // Sync Total Burn Telemetry for Explorer
            if let Ok(mut burn_lock) = api_burned.lock() {
                *burn_lock = pool.total_burned;
            }

            // [PHASE 21] 100 NVC Base Inflation with Halving Support
            let base_inflation = blockchain.get_epoch_reward();
            pool.reward_pool += base_inflation;
            
            let mut active_nodes = std::vec::Vec::new();
            {
                let gov = api_gov.lock().unwrap();
                let staking = shared_staking.lock().unwrap();
                
                for address in gov.reputations.keys() {
                    let stake = staking.get(address).map(|s| s.amount).unwrap_or(0.0);
                    let work_score = (1.0 + stake) as u64; 
                    active_nodes.push(crate::reward::ActiveNode {
                        address: address.clone(),
                        work_score,
                    });
                }
            }

            // === PROOF-OF-CONTRIBUTION REWARD DISTRIBUTION ===
            // Reward is split proportionally based on mission work completed this epoch.
            // Only addresses that completed missions get a share.
            let epoch_inflation = pool.reward_pool; // Includes base_inflation + 60% System Fees
            pool.reward_pool = 0.0; // Reset after processing
            let mut node_rewards_info = Vec::new();

            {
                let missions_lock = api_missions.lock().unwrap();
                let total_contribution = missions_lock.total_contribution();
                
                if total_contribution > 0.0 {
                    // Distribute proportionally to contributors
                    let mut wallets = api_wallets.lock().unwrap();
                    for (addr, contrib) in &missions_lock.contribution_tracker {
                        let share = (contrib / total_contribution) * epoch_inflation;
                        wallets.add_balance(addr, share);
                        node_rewards_info.push(crate::block::NodeRewardInfo {
                            address: addr.clone(),
                            amount: share,
                            category: "Base Epoch Reward".to_string(),
                        });
                        println!("[REWARD] {} earned {:.4} NVC (contribution: {:.2}/{:.2})", 
                            &addr[..16.min(addr.len())], share, contrib, total_contribution);
                    }
                } else {
                    // No contributors this epoch: full inflation goes to Founder (genesis node)
                    let mut wallets = api_wallets.lock().unwrap();
                    wallets.add_balance(&founder.address, epoch_inflation);
                    node_rewards_info.push(crate::block::NodeRewardInfo {
                        address: founder.address.clone(),
                        amount: epoch_inflation,
                        category: "Base Epoch Reward".to_string(),
                    });
                    println!("[REWARD] No contributors this epoch. {} NVC to Founder (genesis node).", epoch_inflation);
                }
            }

            // === PROTOCOL GROWTH DISTRIBUTION (30% System Fees) ===
            if pool.protocol_growth > 0.0 {
                let to_founder = pool.protocol_growth * (0.15 / 0.30); // 15% out of 30%
                let to_hub = pool.protocol_growth * (0.10 / 0.30);     // 10% out of 30%
                let total_protocol = to_founder + to_hub;
                
                let mut wallets = api_wallets.lock().unwrap();
                wallets.add_balance(&founder.address, total_protocol); 
                node_rewards_info.push(crate::block::NodeRewardInfo {
                    address: founder.address.clone(),
                    amount: total_protocol,
                    category: "Protocol Growth (Founder & Hub)".to_string(),
                });
                println!("[REWARD] Protocol Allocation (Founder 15%, Hub 10%): {:.4} NVC", total_protocol);
                pool.protocol_growth = 0.0;
            }

            // === BONUS POOL DISTRIBUTION (10% System Fees) ===
            // Splits the 10% pool among Legacy Core holders, or Founder Treasury if none
            if pool.bonus_pool > 0.0 {
                let missions_lock = api_missions.lock().unwrap();
                let legacy_owners: Vec<String> = missions_lock.user_inventory.iter()
                    .filter(|(_, items)| items.contains(&"Legacy Core".to_string()))
                    .map(|(addr, _)| addr.clone())
                    .collect();
                
                let mut wallets = api_wallets.lock().unwrap();
                if legacy_owners.is_empty() {
                    wallets.add_balance(&founder.address, pool.bonus_pool); 
                    node_rewards_info.push(crate::block::NodeRewardInfo {
                        address: founder.address.clone(),
                        amount: pool.bonus_pool,
                        category: "System Fee Bonus".to_string(),
                    });
                    println!("[REWARD] Bonus Pool (Legacy Core) reserved to Founder Treasury: {:.4} NVC", pool.bonus_pool);
                } else {
                    let share = pool.bonus_pool / legacy_owners.len() as f64;
                    for owner in legacy_owners {
                        wallets.add_balance(&owner, share);
                        node_rewards_info.push(crate::block::NodeRewardInfo {
                            address: owner.clone(),
                            amount: share,
                            category: "Legacy Core Reward".to_string(),
                        });
                        println!("[REWARD] Bonus Pool => @{} received {:.4} NVC (Legacy Core Bonus)", owner, share);
                    }
                }
                pool.bonus_pool = 0.0;
            }

            // Reset contribution tracker for next epoch
            {
                let mut missions_lock = api_missions.lock().unwrap();
                missions_lock.clear_contributions();
            }
            reward_engine.current_epoch += 1;

            // === MEMPOOL PROCESSING LAYER ===
            let mut snapshot_mempool;
            {
                let mut m_lock = shared_mempool.lock().unwrap();
                snapshot_mempool = m_lock.clone();
                m_lock.clear();
            }

            {
                let mut wallets = api_wallets.lock().unwrap();
                let mut staking = shared_staking.lock().unwrap();
                let mut valid_txs = Vec::new();

                for tx_str in &snapshot_mempool {
                    // Try parsing as JSON intent
                    if let Ok(intent) = serde_json::from_str::<serde_json::Value>(tx_str) {
                        let tx_type = intent["type"].as_str().unwrap_or("");
                        let addr = intent["address"].as_str().unwrap_or("").to_string();
                        let amount = intent["amount"].as_f64().unwrap_or(0.0);

                        match tx_type {
                            "STAKE" => {
                                if wallets.deduct_balance(&addr, amount).is_ok() {
                                    let info = staking.entry(addr.clone()).or_insert(crate::contract::StakingInfo {
                                        amount: 0.0,
                                        start_block: blockchain.chain.len() as u32,
                                        last_claim_block: blockchain.chain.len() as u32,
                                    });
                                    info.amount += amount;
                                    valid_txs.push(tx_str.clone());
                                    println!("[MEMPOOL EXEC] Staked {:.2} NVC for {}", amount, addr);
                                } else {
                                    println!("[MEMPOOL REJECT] {} insufficient balance for staking {:.2}", addr, amount);
                                }
                            },
                            "UNSTAKE" => {
                                if let Some(info) = staking.remove(&addr) {
                                    let returned_amount = info.amount;
                                    wallets.add_balance(&addr, returned_amount);
                                    valid_txs.push(tx_str.clone());
                                    println!("[MEMPOOL EXEC] Unstaked {:.2} NVC for {}", returned_amount, addr);
                                } else {
                                    println!("[MEMPOOL REJECT] {} has no active stake", addr);
                                }
                            },
                            "TRANSFER" => {
                                let target = intent["target"].as_str().unwrap_or("").to_string();
                                if wallets.deduct_balance(&addr, amount).is_ok() {
                                    wallets.add_balance(&target, amount);
                                    valid_txs.push(tx_str.clone());
                                    println!("[MEMPOOL EXEC] Transfer {:.2} NVC from {} to {}", amount, addr, target);
                                } else {
                                    println!("[MEMPOOL REJECT] {} insufficient balance for transfer of {:.2}", addr, amount);
                                }
                            },
                            _ => { valid_txs.push(tx_str.clone()); } // other JSON intents
                        }
                    } else {
                        // Keep legacy string formats (e.g. ID_REGISTERED, GOV_VOTE)
                        valid_txs.push(tx_str.clone());
                    }
                }
                snapshot_mempool = valid_txs;
            }

            // BUILD STRUCTURED BLOCK DATA [PHASE 20: TRANSPARENCY]
            let block_data = crate::block::BlockData {
                transactions: snapshot_mempool,
                rewards: node_rewards_info,
                economy: crate::block::EconomySummary {
                    fees_collected: fees_this_period,
                    burned: pool.total_burned,
                    epoch_number: reward_engine.current_epoch,
                },
            };

            let data_json = serde_json::to_string(&block_data).unwrap_or_else(|_| "{}".to_string());
            let new_index = blockchain.next_index();
            
            // Add block to chain & persistent storage
            blockchain.add_block(crate::block::Block::new(
                new_index, 
                data_json, 
                blockchain.get_latest_block().hash.clone()
            ));

            println!("[CHAIN] Block #{} created with {} transactions and {} reward entries.", 
                new_index, block_data.transactions.len(), block_data.rewards.len());
            // Sync dashboard chain
            if let Ok(mut chain_lock) = api_chain.lock() {
                *chain_lock = blockchain.chain.clone();
            }

            // Reset Dynamic Gas Fee counter for the new epoch
            {
                let mut gas_lock = shared_gas.lock().unwrap();
                gas_lock.reset_epoch();
            }

            last_epoch_time = std::time::Instant::now();
            if let Ok(mut next_lock) = shared_next_block.lock() {
                *next_lock = chrono::Utc::now().timestamp() as u64 + 300;
            }
        }

        // 4. Sleep to prevent CPU hogging
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // No need to manual update shared_staking as it's already shared

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
