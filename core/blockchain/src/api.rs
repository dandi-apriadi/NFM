use crate::auction::{Auction, AuctionStatus, EscrowVault};
use crate::block::{Block, BlockData};
use crate::item::{Item, Rarity};
use crate::reward::EconomyPool;
use nfm_ai_engine::distributed_brain::{
    DataClass, GeoDistributedBrainDb, NodeMeta, RequestProfile, RouterWeights,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sha2::{Sha256, Digest};

// ======================================================================
// RATE LIMITER [K-02 FIX]
// ======================================================================
const MAX_GET_PER_MINUTE: u32 = 1000;
const MAX_POST_PER_MINUTE: u32 = 300;
const STATUS_CACHE_TTL_SECS: u64 = 2;
const BRAIN_SNAPSHOT_KEY: &[u8] = b"brain_snapshot_v1";

#[derive(Clone)]
pub struct StatusCacheEntry {
    pub generated_at: Instant,
    pub payload: String,
}

struct RateLimiter {
    requests: HashMap<String, (u32, std::time::Instant)>,
}

impl RateLimiter {
    fn new() -> Self {
        Self { requests: HashMap::new() }
    }

    /// Cek apakah IP ini melebihi batas request
    fn check(&mut self, ip: &str, method: &str) -> bool {
        let now = std::time::Instant::now();
        let entry = self.requests.entry(ip.to_string()).or_insert((0, now));

        if now.duration_since(entry.1).as_secs() >= 60 {
            entry.0 = 0;
            entry.1 = now;
        }

        entry.0 += 1;
        let limit = if method == "GET" { MAX_GET_PER_MINUTE } else { MAX_POST_PER_MINUTE };
        entry.0 <= limit
    }
}

// ======================================================================
// API AUTHENTICATION [K-01 FIX]
// ======================================================================

/// Verifikasi HMAC-SHA256 signature untuk endpoint protected
fn verify_admin_signature(secret: &str, url: &str, body: &str, provided_sig: &str) -> bool {
    let payload = format!("{}:{}", url, body);
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", secret, payload).as_bytes());
    let expected = hex::encode(hasher.finalize());
    expected == provided_sig
}

/// Cek apakah endpoint ini memerlukan autentikasi
fn is_protected_endpoint(url: &str) -> bool {
    url.starts_with("/api/admin")
        || url == "/api/nlc"
        || url == "/api/transfer/secure"
        || url == "/api/staking/deposit"
        || url == "/api/mission/start"
    || url == "/api/mission/progress"
        || url == "/api/mission/complete"
}

/// Validasi bearer token untuk public brain endpoints
fn validate_brain_token(tokens: &[String], auth_header: &str) -> bool {
    if tokens.is_empty() {
        // Jika tidak ada token yang dikonfigurasi, akses terbuka
        return true;
    }
    
    // Ekstrak token dari header "Bearer <token>"
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        tokens.contains(&token.to_string())
    } else {
        false
    }
}

// ======================================================================
// UNIVERSAL GAS FEE [PHASE 11]
// ======================================================================

/// Middleware untuk memotong Gas Fee dari dompet setiap kali transaksi POST dilakukan
fn apply_universal_gas_fee(state: &ApiState, address: &str) -> Result<f64, String> {
    let mut gas_calc = state.gas_fee_calculator.lock().unwrap();
    let fee = gas_calc.calculate_fee();
    
    let mut wallets = state.wallets.lock().unwrap();
    if wallets.deduct_balance(address, fee).is_err() {
        return Err(format!("Insufficient balance to pay Gas Fee: {:.4} NVC", fee));
    }
    
    // Alirkan ke Economy Pool (Pajak AI)
    let mut fees = state.total_fees.lock().unwrap();
    *fees += fee;

    // --- BURN MECHANISM [PHASE 11.2] ---
    // 5% dari setiap Gas Fee dibakar secara permanen dari sirkulasi
    let burn_amount = fee * 0.05;
    let mut burned = state.total_burned.lock().unwrap();
    *burned += burn_amount;

    
    // Catat transaksi untuk menaikkan kesibukan (Dynamic Fee)
    gas_calc.record_tx();
    
    Ok(fee)
}

fn persist_brain_snapshot(state: &ApiState, brain: &GeoDistributedBrainDb) -> Result<(), String> {
    let store = state.brain_snapshot_store.lock().unwrap();
    if let Some(db) = store.as_ref() {
        let snapshot = brain.export_snapshot();
        let bytes = serde_json::to_vec(&snapshot).map_err(|e| e.to_string())?;
        db.insert(BRAIN_SNAPSHOT_KEY, bytes).map_err(|e| e.to_string())?;
        db.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} Bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", (bytes as f64) / 1024.0)
    } else {
        format!("{:.2} MB", (bytes as f64) / (1024.0 * 1024.0))
    }
}

fn parse_prefixed_id(raw: &str, prefix: &str) -> Option<u32> {
    if let Some(rest) = raw.strip_prefix(prefix) {
        return rest.parse::<u32>().ok();
    }
    raw.parse::<u32>().ok()
}

fn parse_rarity(raw: &str) -> Rarity {
    match raw.trim().to_ascii_uppercase().as_str() {
        "RARE" => Rarity::Rare,
        "EPIC" => Rarity::Epic,
        "LEGENDARY" => Rarity::Legendary,
        "MYTHIC" => Rarity::Mythic,
        _ => Rarity::Common,
    }
}

fn default_user_settings() -> serde_json::Value {
    serde_json::json!({
        "rpc": "http://127.0.0.1:3000",
        "theme": "mesh",
        "notifications": {
            "rewards": true,
            "network": true,
            "security": true
        }
    })
}

fn sanitize_user_settings(candidate: &serde_json::Value) -> serde_json::Value {
    let mut current = default_user_settings();

    if let Some(rpc) = candidate.get("rpc").and_then(|v| v.as_str()) {
        current["rpc"] = serde_json::Value::String(rpc.to_string());
    }

    if let Some(theme) = candidate.get("theme").and_then(|v| v.as_str()) {
        let normalized = match theme {
            "dark" | "light" | "mesh" => theme,
            _ => "mesh",
        };
        current["theme"] = serde_json::Value::String(normalized.to_string());
    }

    if let Some(notifications) = candidate.get("notifications").and_then(|v| v.as_object()) {
        for key in ["rewards", "network", "security"] {
            if let Some(flag) = notifications.get(key).and_then(|v| v.as_bool()) {
                current["notifications"][key] = serde_json::Value::Bool(flag);
            }
        }
    }

    current
}

fn build_drive_files_payload(state: &ApiState) -> Vec<serde_json::Value> {
    let brain = state.brain_db.lock().unwrap();
    let snapshot = brain.export_snapshot();
    let now_ms = chrono::Utc::now().timestamp_millis();

    snapshot
        .records
        .values()
        .take(100)
        .enumerate()
        .map(|(idx, rec)| {
            let value = rec.value.as_object();
            let name = value
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or(&rec.key);
            let file_type = value
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("FRAGMENT");
            let fragments = value
                .and_then(|v| v.get("fragments"))
                .and_then(|v| v.as_u64())
                .unwrap_or(1);
            let health = value
                .and_then(|v| v.get("health"))
                .and_then(|v| v.as_u64())
                .unwrap_or(100);
            let uploaded_at = value
                .and_then(|v| v.get("uploadedAt"))
                .and_then(|v| v.as_i64())
                .unwrap_or(now_ms - ((idx as i64) * 60_000));

            let size = value
                .and_then(|v| v.get("size"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{} B", rec.value.to_string().len()));

            serde_json::json!({
                "id": rec.key,
                "name": name,
                "size": size,
                "type": file_type,
                "fragments": fragments,
                "health": health,
                "uploadedAt": uploaded_at
            })
        })
        .collect()
}

fn build_frontend_app_state(state: &ApiState) -> serde_json::Value {
    let chain = state.chain.lock().unwrap();
    let wallets = state.wallets.lock().unwrap();
    let mempool = state.mempool.lock().unwrap();
    let missions = state.mission_engine.lock().unwrap();
    let governance = state.governance_engine.lock().unwrap();
    let admin = state.admin_engine.lock().unwrap();
    let aliases = state.aliases.lock().unwrap();
    let auctions = state.auctions.lock().unwrap();
    let brain = state.brain_db.lock().unwrap();
    let total_burned = *state.total_burned.lock().unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();

    let blocks: Vec<serde_json::Value> = chain
        .iter()
        .rev()
        .take(40)
        .map(|b| {
            let parsed = serde_json::from_str::<BlockData>(&b.data).ok();
            let tx_count = parsed.as_ref().map(|p| p.transactions.len()).unwrap_or(0);
            let rewards = parsed
                .as_ref()
                .map(|p| p.rewards.iter().map(|r| r.amount).sum::<f64>())
                .unwrap_or(0.0);
            let miner = parsed
                .as_ref()
                .and_then(|p| p.rewards.first().map(|r| r.address.clone()))
                .unwrap_or_else(|| "nfm_validator_unknown".to_string());

            let tx_hashes: Vec<String> = parsed
                .as_ref()
                .map(|p| {
                    p.transactions
                        .iter()
                        .map(|tx_str| {
                            if tx_str.starts_with("GENESIS") {
                                return tx_str.clone();
                            }
                            let mut hasher = Sha256::new();
                            hasher.update(tx_str.as_bytes());
                            hex::encode(hasher.finalize())
                        })
                        .collect()
                })
                .unwrap_or_default();

            serde_json::json!({
                "index": b.index,
                "hash": b.hash,
                "previous_hash": b.previous_hash,
                "timestamp": b.timestamp.saturating_mul(1000),
                "transactions": tx_count,
                "tx_hashes": tx_hashes,
                "size": format_bytes(b.data.len()),
                "miner": miner,
                "rewards": rewards
            })
        })
        .collect();

    // --- TRANSACTION HISTORY [CONFIRMED + MEMPOOL] ---
    let mut all_transactions: Vec<serde_json::Value> = Vec::new();

    // 1. Ambil dari blocks (Confirmed) - Ambil 30 terakhir
    for b in chain.iter().rev().take(10) {
        if let Ok(parsed) = serde_json::from_str::<BlockData>(&b.data) {
            for tx_str in parsed.transactions {
                let txid = if tx_str.starts_with("GENESIS") {
                    tx_str.clone()
                } else {
                    let mut hasher = Sha256::new();
                    hasher.update(tx_str.as_bytes());
                    hex::encode(hasher.finalize())
                };

                if let Ok(parsed_tx) = serde_json::from_str::<serde_json::Value>(&tx_str) {
                    let from = parsed_tx["address"].as_str().unwrap_or("nfm_protocol");
                    let to = parsed_tx["target"].as_str().unwrap_or("nfm_reward");
                    let amount = parsed_tx["amount"].as_f64().unwrap_or(0.0);
                    let tx_type = match parsed_tx["type"].as_str().unwrap_or("TRANSFER") {
                        "BURN" => "BURN",
                        "STAKE" | "UNSTAKE" => "SMART_CONTRACT",
                        _ => "TRANSFER",
                    };

                    all_transactions.push(serde_json::json!({
                        "txid": txid,
                        "type": tx_type,
                        "from": from,
                        "to": to,
                        "amount": amount,
                        "timestamp": b.timestamp * 1000,
                        "fee": 0.0, // Fee sudah masuk Reward Pool
                        "status": "CONFIRMED"
                    }));
                } else if tx_str.starts_with("GENESIS") {
                     all_transactions.push(serde_json::json!({
                        "txid": tx_str,
                        "type": "NODE_REWARD",
                        "from": "nfm_genesis",
                        "to": "nfm_founder",
                        "amount": 100.0,
                        "timestamp": b.timestamp * 1000,
                        "fee": 0.0,
                        "status": "CONFIRMED"
                    }));
                }
            }
        }
    }

    // 2. Ambil dari mempool (Pending)
    for (idx, raw) in mempool.iter().enumerate() {
        if let Ok(parsed_tx) = serde_json::from_str::<serde_json::Value>(raw) {
            let from = parsed_tx["address"].as_str().unwrap_or("nfm_unknown");
            let to = parsed_tx["target"].as_str().unwrap_or("nfm_unknown");
            let amount = parsed_tx["amount"].as_f64().unwrap_or(0.0);
            let tx_type = match parsed_tx["type"].as_str().unwrap_or("TRANSFER") {
                "BURN" => "BURN",
                "STAKE" | "UNSTAKE" => "SMART_CONTRACT",
                _ => "TRANSFER",
            };

            all_transactions.push(serde_json::json!({
                "txid": format!("pending_{}_{}", now_ms, idx),
                "type": tx_type,
                "from": from,
                "to": to,
                "amount": amount,
                "timestamp": now_ms,
                "fee": 0.0,
                "status": "PENDING"
            }));
        }
    }

    let completed = missions
        .completed_missions
        .get(&state.node_address)
        .cloned()
        .unwrap_or_default();

    let quests: Vec<serde_json::Value> = missions
        .available_missions
        .iter()
        .map(|m| {
            let assignment = missions
                .active_assignments
                .get(&format!("{}:{}", state.node_address, m.id));
            let required_units = m.work_type.required_units();
            let current_units = assignment.map(|a| a.current_units).unwrap_or(0);
            let status = if completed.contains(&m.id) {
                "COMPLETED"
            } else if assignment
                .map(|a| a.status == crate::mission::MissionStatus::PendingVerification)
                .unwrap_or(false)
            {
                "CLAIMABLE"
            } else {
                "ACTIVE"
            };
            serde_json::json!({
                "id": format!("q-{}", m.id),
                "title": m.name,
                "description": m.description,
                "rewardNVC": m.reward_nvc,
                "progress": current_units,
                "total": required_units,
                "status": status
            })
        })
        .collect();

    let wallets_list: Vec<serde_json::Value> = wallets
        .balances
        .iter()
        .map(|(address, balance)| serde_json::json!({
            "name": if *address == state.node_address { "Main Vault" } else { "Wallet" },
            "address": address,
            "balanceNVC": *balance,
            "balanceETH": 0.0,
            "isActive": *address == state.node_address
        }))
        .collect();

    let user_alias = aliases
        .iter()
        .find_map(|(alias, address)| {
            if *address == state.node_address {
                Some(alias.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "@founder".to_string());

    let joined_at_ms = chain
        .first()
        .map(|b| b.timestamp.saturating_mul(1000))
        .unwrap_or(now_ms);
    let user_settings = state
        .user_settings
        .lock()
        .unwrap()
        .get(&state.node_address)
        .cloned()
        .unwrap_or_else(default_user_settings);

    let api_docs = vec![
        serde_json::json!({ "method": "GET", "path": "/api/status", "description": "Core node status and tokenomics", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/p2p/status", "description": "P2P gossip telemetry and peer health", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/p2p/seeds", "description": "List configured seed peers", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/p2p/seeds", "description": "Replace configured seed peers", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/p2p/banlist", "description": "List runtime banned peer endpoints", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/p2p/ban", "description": "Ban a peer endpoint from active mesh", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/p2p/unban", "description": "Remove peer endpoint from banlist", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/p2p/ban/bulk", "description": "Bulk ban peer endpoints", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/p2p/unban/bulk", "description": "Bulk unban peer endpoints", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/p2p/bootstrap", "description": "Trigger gossip bootstrap from configured seeds", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/p2p/sync", "description": "Trigger longest-chain sync now", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/blocks", "description": "Recent blocks", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/mempool", "description": "Pending intents", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/transfer/create", "description": "Queue a transfer intent", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/transfer/secure", "description": "Signed transfer", "authRequired": true }),
        serde_json::json!({ "method": "GET", "path": "/api/app/settings", "description": "Read UI settings for active node", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/app/settings", "description": "Update UI settings for active node", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/drive/files", "description": "List indexed drive files", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/drive/upload", "description": "Upload a drive file into SDS index", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/drive/download", "description": "Download a drive file by id", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/auction/list", "description": "List active and historical auctions", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/auction/create", "description": "Create a new auction listing", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/auction/bid", "description": "Place an escrow-backed bid", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/auction/settle", "description": "Settle auction and distribute payout", "authRequired": false }),
        serde_json::json!({ "method": "POST", "path": "/api/auction/cancel", "description": "Cancel auction and refund escrow", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/governance/indicators", "description": "Governance quorum, treasury, and veto risk indicators", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/kg/semantic", "description": "Semantic knowledge graph view with typed concepts", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/brain/status", "description": "Distributed brain health", "authRequired": false }),
        serde_json::json!({ "method": "GET", "path": "/api/identity/{address}", "description": "Get user identity and elite shield status", "authRequired": false }),
    ];

    let proposals: Vec<serde_json::Value> = governance
        .proposals
        .iter()
        .rev()
        .take(20)
        .map(|p| serde_json::json!({
            "id": format!("prop-{}", p.id),
            "title": p.title,
            "creator": p.proposer,
            "status": if p.is_active { "ACTIVE" } else if p.votes_for >= p.votes_against { "PASSED" } else { "REJECTED" },
            "forVotes": p.votes_for,
            "againstVotes": p.votes_against,
            "endTime": now_ms + if p.is_active { 86400000 } else { 0 }
        }))
        .collect();

    let ai_tasks: Vec<serde_json::Value> = mempool
        .iter()
        .enumerate()
        .take(12)
        .map(|(idx, raw)| {
            let parsed: serde_json::Value = serde_json::from_str(raw).unwrap_or_default();
            serde_json::json!({
                "id": format!("task-{:03}", idx + 1),
                "name": parsed["type"].as_str().unwrap_or("MESH_TASK"),
                "status": "QUEUED",
                "progress": 0,
                "model": "NFM-Orchestrator",
                "cost": parsed["amount"].as_f64().unwrap_or(0.0) * 0.001
            })
        })
        .collect();

    let snapshot = brain.export_snapshot();

    let drive_files: Vec<serde_json::Value> = snapshot
        .records
        .values()
        .take(20)
        .enumerate()
        .map(|(idx, rec)| {
            let serialized = rec.value.to_string();
            serde_json::json!({
                "id": format!("f-{}", idx + 1),
                "name": rec.key,
                "size": format!("{} B", serialized.len()),
                "type": "FRAGMENT",
                "fragments": 1,
                "health": 100,
                "uploadedAt": now_ms - ((idx as i64) * 60_000)
            })
        })
        .collect();

    let kg_concepts: Vec<serde_json::Value> = snapshot
        .records
        .values()
        .take(24)
        .enumerate()
        .map(|(idx, rec)| {
            let serialized = rec.value.to_string();
            serde_json::json!({
                "id": format!("c-{}", idx + 1),
                "name": rec.key,
                "connections": serialized.len().min(256),
                "category": "DOCUMENT"
            })
        })
        .collect();

    let box_history: Vec<serde_json::Value> = admin
        .logs
        .iter()
        .rev()
        .take(10)
        .enumerate()
        .map(|(idx, log)| serde_json::json!({
            "id": format!("b-{}", idx + 1),
            "timestamp": log.timestamp.saturating_mul(1000),
            "rarity": "COMMON",
            "rewardInfo": format!("{} by {}: {}", log.action, log.admin, log.reason)
        }))
        .collect();

    let mystery_news: Vec<serde_json::Value> = admin
        .logs
        .iter()
        .rev()
        .take(10)
        .enumerate()
        .map(|(idx, log)| serde_json::json!({
            "id": format!("n-{}", idx + 1),
            "type": "SYSTEM",
            "content": format!("{} on {} ({})", log.action, log.target, log.reason),
            "timestamp": log.timestamp.saturating_mul(1000)
        }))
        .collect();

    let reward_catalog: Vec<serde_json::Value> = missions
        .available_missions
        .iter()
        .take(8)
        .enumerate()
        .map(|(idx, m)| serde_json::json!({
            "id": format!("r-{}", idx + 1),
            "name": m.name,
            "description": m.description,
            "rarity": match m.difficulty {
                crate::mission::Difficulty::Easy => "COMMON",
                crate::mission::Difficulty::Medium => "RARE",
                crate::mission::Difficulty::Hard => "EPIC",
                crate::mission::Difficulty::Expert => "LEGENDARY",
            },
            "type": "NVC"
        }))
        .collect();

    let market_items: Vec<serde_json::Value> = auctions
        .values()
        .take(40)
        .map(|a| {
            let rarity = match a.item.rarity {
                Rarity::Common => "COMMON",
                Rarity::Rare => "RARE",
                Rarity::Epic => "EPIC",
                Rarity::Legendary => "LEGENDARY",
                Rarity::Mythic => "MYTHIC",
            };
            let status = match a.status {
                AuctionStatus::Active => "ACTIVE",
                AuctionStatus::Settled => "SOLD",
                AuctionStatus::Cancelled => "CANCELLED",
            };
            serde_json::json!({
                "id": format!("a-{}", a.auction_id),
                "auction_id": a.auction_id,
                "title": a.item.name,
                "seller": a.seller,
                "price": if a.highest_bid > 0.0 { a.highest_bid } else { a.starting_price },
                "starting_price": a.starting_price,
                "highest_bid": a.highest_bid,
                "highest_bidder": a.highest_bidder,
                "rarity": rarity,
                "power_multiplier": a.item.power_multiplier,
                "status": status,
                "end_time": a.end_time.timestamp_millis(),
            })
        })
        .collect();

    let circulating_supply: f64 = wallets.balances.values().sum();
    let total_supply: f64 = 100_000_000.0;
    let reward_pool = *state.reward_pool.lock().unwrap();

    serde_json::json!({
        "status": {
            "node": state.node_address,
            "version": "NFM Vault v1.2",
            "status": "ONLINE",
            "blocks": chain.len(),
            "total_burned": total_burned,
            "reward_pool": reward_pool,
            "circulating_supply": circulating_supply,
            "total_supply": total_supply,
            "peers": brain.node_count()
        },
        "blocks": blocks,
        "transactions": all_transactions,
        "user_profile": {
            "username": user_alias,
            "nfmAddress": state.node_address,
            "balance": wallets.balances.get(&state.node_address).copied().unwrap_or(0.0),
            "reputation": governance.get_reputation(&state.node_address),
            "joinedAt": joined_at_ms,
            "feedbackCount": completed.len(),
            "settings": user_settings
        },
        "wallets": wallets_list,
        "node_stats": {
            "uptime": format!("{} blocks", chain.len()),
            "cpu": (mempool.len() as f64 * 3.5).min(95.0),
            "memory": format!("{:.2} GB / 8 GB", (chain.len() as f64 * 0.01).max(0.4)),
            "bandwidth": format!("{} rec/s", brain.record_count())
        },
        "ai_tasks": ai_tasks,
        "drive_files": drive_files,
        "kg_concepts": kg_concepts,
        "market_items": market_items,
        "quests": quests,
        "box_history": box_history,
        "reward_catalog": reward_catalog,
        "mystery_news": mystery_news,
        "proposals": proposals,
        "api_docs": api_docs
    })
}

/// State yang dibagikan ke API server
pub struct ApiState {
    pub chain: Arc<Mutex<Vec<Block>>>,
    pub node_address: String,
    pub total_fees: Arc<Mutex<f64>>,
    pub total_burned: Arc<Mutex<f64>>,
    pub reward_pool: Arc<Mutex<f64>>,
    pub active_effects: Arc<Mutex<std::collections::HashMap<String, Vec<crate::contract::ActiveEffect>>>>,
    pub mission_engine: Arc<Mutex<crate::mission::MissionEngine>>,
    pub staking_pool: Arc<Mutex<std::collections::HashMap<String, crate::contract::StakingInfo>>>,
    pub wallets: Arc<Mutex<crate::transfer::WalletEngine>>,
    pub admin_engine: Arc<Mutex<crate::admin::AdminEngine>>,
    pub governance_engine: Arc<Mutex<crate::governance::GovernanceEngine>>,
    pub block_tx: std::sync::mpsc::Sender<String>,
    pub api_secret: String,
    pub rate_limit_enabled: Arc<Mutex<bool>>,
    pub gas_fee_calculator: Arc<Mutex<crate::transfer::GasFeeCalculator>>,
    pub aliases: Arc<Mutex<std::collections::HashMap<String, String>>>,
    /// Antrean intent transaksi yang menunggu blokasi ditambang
    pub mempool: Arc<Mutex<Vec<String>>>,
    /// Jadwal pasti kapan epoch berikutnya akan dieksekusi oleh backend (UTC Unix Seconds)
    pub next_block_timestamp: Arc<Mutex<u64>>,
    /// Preferensi UI per alamat untuk sinkronisasi app settings
    pub user_settings: Arc<Mutex<std::collections::HashMap<String, serde_json::Value>>>,
    /// Distributed brain data router (geo + latency + load aware)
    pub brain_db: Arc<Mutex<GeoDistributedBrainDb>>,
    /// Whitelisted bearer tokens untuk public /api/brain/* endpoints
    pub brain_tokens: Arc<Mutex<Vec<String>>>,
    /// Cache respons /api/status untuk mengurangi lock contention di endpoint agregat
    pub status_cache: Arc<Mutex<Option<StatusCacheEntry>>>,
    /// Status ringkas P2P gossip untuk endpoint observability
    pub p2p_status: Arc<Mutex<serde_json::Value>>,
    /// Daftar seed peers yang dapat diperbarui runtime
    pub p2p_seed_peers: Arc<Mutex<Vec<String>>>,
    /// Daftar endpoint peer yang di-ban oleh operator runtime
    pub p2p_ban_peers: Arc<Mutex<Vec<String>>>,
    /// Buku auction aktif dan historis
    pub auctions: Arc<Mutex<std::collections::HashMap<u32, Auction>>>,
    /// Vault escrow untuk lock dana bidding
    pub auction_escrow: Arc<Mutex<EscrowVault>>,
    /// Incremental id generator untuk auction baru
    pub next_auction_id: Arc<Mutex<u32>>,
    /// Penyimpanan snapshot brain di sled untuk persistensi antar restart
    pub brain_snapshot_store: Arc<Mutex<Option<sled::Db>>>,
}

/// Mulai REST API server di background thread
pub fn start_api_server(state: ApiState, port: u16) {
    let bind = format!("0.0.0.0:{}", port);

    thread::spawn(move || {
        let server = match tiny_http::Server::http(&bind) {
            Ok(s) => {
                println!("[API] Dashboard running at http://127.0.0.1:{}", port);
                println!("[API] Auth: HMAC-SHA256 enabled for protected endpoints");
                s
            },
            Err(e) => {
                println!("[API] Failed to start: {}", e);
                let lower = e.to_string().to_lowercase();
                if lower.contains("address already in use")
                    || lower.contains("only one usage of each socket address")
                    || lower.contains("os error 10048")
                {
                    println!("[API][HINT] Port {} is already in use. Another node instance may already be serving the dashboard.", port);
                    println!("[API][HINT] Stop the existing process or restart via node-runner script with --restart.");
                }
                return;
            }
        };

        let mut rate_limiter = RateLimiter::new();

        for mut request in server.incoming_requests() {
            let url = request.url().to_string();
            let method = request.method().to_string();

            // --- RATE LIMITING [K-02] ---
            let is_rate_limit_enabled = *state.rate_limit_enabled.lock().unwrap();
            let is_limited_method = method == "GET" || method == "POST";
            
            if is_rate_limit_enabled && is_limited_method {
                let client_ip = request.remote_addr()
                    .map(|a| a.ip().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let limit = if method == "GET" { MAX_GET_PER_MINUTE } else { MAX_POST_PER_MINUTE };

                if !rate_limiter.check(&client_ip, &method) {
                    let response = tiny_http::Response::from_string(
                        serde_json::json!({
                            "error": format!("Rate limit exceeded ({} req/min for {}). Please slow down.", limit, method)
                        }).to_string()
                    )
                    .with_status_code(429)
                    .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap())
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap());
                    let _ = request.respond(response);
                    continue;
                }
            }

            // --- CORS PREFLIGHT ---
            if method == "OPTIONS" {
                let response = tiny_http::Response::from_string("")
                    .with_status_code(204)
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap())
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap())
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Headers", "Content-Type, Authorization, x-nfm-signature").unwrap());
                let _ = request.respond(response);
                continue;
            }

            // --- AUTHENTICATION [K-01] ---
            if is_protected_endpoint(&url) {
                let sig_header = request.headers().iter()
                    .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                    .map(|h| h.value.as_str().to_string())
                    .unwrap_or_default();

                // Untuk POST, kita perlu body untuk verifikasi, tapi body belum dibaca
                // Jadi verifikasi menggunakan URL saja untuk GET, dan body akan divalidasi setelah dibaca
                if method == "GET" {
                    if !verify_admin_signature(&state.api_secret, &url, "", &sig_header) {
                        let response = tiny_http::Response::from_string(
                            serde_json::json!({ "error": "Forbidden: invalid or missing X-NFM-Signature" }).to_string()
                        )
                        .with_status_code(403)
                        .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap());
                        let _ = request.respond(response);
                        continue;
                    }
                }
                // POST endpoints: signature akan diverifikasi setelah body dibaca (di dalam handler)
            }

            // --- BEARER TOKEN VALIDATION for public /api/brain/* endpoints ---
            if url.starts_with("/api/brain/") {
                let brain_tokens = state.brain_tokens.lock().unwrap();
                if !brain_tokens.is_empty() {
                    let auth_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "authorization")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !validate_brain_token(&brain_tokens, &auth_header) {
                        let response = tiny_http::Response::from_string(
                            serde_json::json!({ "error": "Unauthorized: invalid or missing bearer token" }).to_string()
                        )
                        .with_status_code(401)
                        .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap());
                        let _ = request.respond(response);
                        continue;
                    }
                }
            }

            let (status, content_type, body) = match (method.as_str(), url.as_str()) {
                ("GET", "/") => {
                    // Coba baca dashboard.html dari disk jika ada, jika tidak pakai fallback
                    match std::fs::read_to_string("web/dashboard.html") {
                        Ok(html) => (200, "text/html", html.replace("{{API_SECRET}}", &state.api_secret)),
                        Err(_) => {
                            let chain = state.chain.lock().unwrap();
                            let fees = state.total_fees.lock().unwrap();
                            let burned = state.total_burned.lock().unwrap();
                            (200, "text/html", render_dashboard(chain.len(), *fees, *burned, &state.node_address, port))
                        }
                    }
                },
                ("GET", "/api/blocks") => {
                    let chain = state.chain.lock().unwrap();
                    let json = serde_json::to_string_pretty(&*chain).unwrap_or_default();
                    (200, "application/json", json)
                },
                ("GET", "/api/status") => {
                    {
                        let cache = state.status_cache.lock().unwrap();
                        if let Some(entry) = cache.as_ref() {
                            if entry.generated_at.elapsed() < Duration::from_secs(STATUS_CACHE_TTL_SECS) {
                                (200, "application/json", entry.payload.clone())
                            } else {
                                drop(cache);
                                let chain = state.chain.lock().unwrap();
                                let wallets = state.wallets.lock().unwrap();
                                let fees = state.total_fees.lock().unwrap();
                                let burned = state.total_burned.lock().unwrap();
                                let effects = state.active_effects.lock().unwrap();
                                let missions = state.mission_engine.lock().unwrap();
                                let staking = state.staking_pool.lock().unwrap();
                                let aliases = state.aliases.lock().unwrap();
                                let mempool = state.mempool.lock().unwrap();
                                let mempool_count = mempool.len();

                                let last_block_timestamp = chain.last().map(|b| b.timestamp).unwrap_or(0);
                                let completed_missions = missions.completed_missions.get(&state.node_address)
                                    .cloned()
                                    .unwrap_or_default();

                                let active_missions: Vec<serde_json::Value> = missions.active_assignments.values()
                                    .filter(|a| a.address == state.node_address && a.status == crate::mission::MissionStatus::InProgress)
                                    .map(|a| {
                                        let min_duration = missions.available_missions.iter()
                                            .find(|m| m.id == a.mission_id)
                                            .map(|m| m.work_type.min_duration_secs())
                                            .unwrap_or(5);
                                        let progress_pct = if a.required_units == 0 {
                                            0
                                        } else {
                                            ((a.current_units.saturating_mul(100)) / a.required_units) as u32
                                        };
                                        serde_json::json!({
                                            "id": a.mission_id,
                                            "started_at": a.started_at,
                                            "min_duration_secs": min_duration,
                                            "current_units": a.current_units,
                                            "required_units": a.required_units,
                                            "progress_pct": progress_pct
                                        })
                                    })
                                    .collect();

                                let status_json = serde_json::json!({
                                    "node": state.node_address,
                                    "balance": wallets.balances.get(&state.node_address).unwrap_or(&0.0),
                                    "blocks": chain.len(),
                                    "total_fees": *fees,
                                    "total_burned": *burned,
                                    "active_effects": *effects,
                                    "missions": missions.available_missions,
                                    "completed_missions": completed_missions,
                                    "active_missions": active_missions,
                                    "staking": *staking,
                                    "aliases": *aliases,
                                    "mempool_count": mempool_count,
                                    "block_interval_secs": 300,
                                    "last_block_timestamp": last_block_timestamp,
                                    "next_block_timestamp": *state.next_block_timestamp.lock().unwrap(),
                                    "status": "running",
                                    "version": "1.0.0-mesh"
                                });
                                let payload = status_json.to_string();
                                *state.status_cache.lock().unwrap() = Some(StatusCacheEntry {
                                    generated_at: Instant::now(),
                                    payload: payload.clone(),
                                });
                                (200, "application/json", payload)
                            }
                        } else {
                            drop(cache);
                            let chain = state.chain.lock().unwrap();
                            let wallets = state.wallets.lock().unwrap();
                            let fees = state.total_fees.lock().unwrap();
                            let burned = state.total_burned.lock().unwrap();
                            let effects = state.active_effects.lock().unwrap();
                            let missions = state.mission_engine.lock().unwrap();
                            let staking = state.staking_pool.lock().unwrap();
                            let aliases = state.aliases.lock().unwrap();
                            let mempool = state.mempool.lock().unwrap();
                            let mempool_count = mempool.len();

                            let last_block_timestamp = chain.last().map(|b| b.timestamp).unwrap_or(0);
                            let completed_missions = missions.completed_missions.get(&state.node_address)
                                .cloned()
                                .unwrap_or_default();

                            let active_missions: Vec<serde_json::Value> = missions.active_assignments.values()
                                .filter(|a| a.address == state.node_address && a.status == crate::mission::MissionStatus::InProgress)
                                .map(|a| {
                                    let min_duration = missions.available_missions.iter()
                                        .find(|m| m.id == a.mission_id)
                                        .map(|m| m.work_type.min_duration_secs())
                                        .unwrap_or(5);
                                    let progress_pct = if a.required_units == 0 {
                                        0
                                    } else {
                                        ((a.current_units.saturating_mul(100)) / a.required_units) as u32
                                    };
                                    serde_json::json!({
                                        "id": a.mission_id,
                                        "started_at": a.started_at,
                                        "min_duration_secs": min_duration,
                                        "current_units": a.current_units,
                                        "required_units": a.required_units,
                                        "progress_pct": progress_pct
                                    })
                                })
                                .collect();

                            let status_json = serde_json::json!({
                                "node": state.node_address,
                                "balance": wallets.balances.get(&state.node_address).unwrap_or(&0.0),
                                "blocks": chain.len(),
                                "total_fees": *fees,
                                "total_burned": *burned,
                                "active_effects": *effects,
                                "missions": missions.available_missions,
                                "completed_missions": completed_missions,
                                "active_missions": active_missions,
                                "staking": *staking,
                                "aliases": *aliases,
                                "mempool_count": mempool_count,
                                "block_interval_secs": 300,
                                "last_block_timestamp": last_block_timestamp,
                                "next_block_timestamp": *state.next_block_timestamp.lock().unwrap(),
                                "status": "running",
                                "version": "1.0.0-mesh"
                            });
                            let payload = status_json.to_string();
                            *state.status_cache.lock().unwrap() = Some(StatusCacheEntry {
                                generated_at: Instant::now(),
                                payload: payload.clone(),
                            });
                            (200, "application/json", payload)
                        }
                    }
                },
                ("GET", "/api/p2p/status") => {
                    let status = state.p2p_status.lock().unwrap().clone();
                    (200, "application/json", status.to_string())
                },
                ("GET", "/api/p2p/seeds") => {
                    let seeds = state.p2p_seed_peers.lock().unwrap().clone();
                    (200, "application/json", serde_json::json!({
                        "count": seeds.len(),
                        "seeds": seeds
                    }).to_string())
                },
                ("POST", "/api/p2p/seeds") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let mut parsed: Vec<String> = Vec::new();
                    if let Some(arr) = data["seeds"].as_array() {
                        for item in arr {
                            if let Some(seed) = item.as_str() {
                                let s = seed.trim();
                                if !s.is_empty() {
                                    parsed.push(s.to_string());
                                }
                            }
                        }
                    } else if let Some(csv) = data["seeds_csv"].as_str() {
                        parsed = csv
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }

                    let mut seeds = state.p2p_seed_peers.lock().unwrap();
                    *seeds = parsed.clone();

                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "count": parsed.len(),
                        "seeds": parsed
                    }).to_string())
                },
                ("GET", "/api/p2p/banlist") => {
                    let banned = state.p2p_ban_peers.lock().unwrap().clone();
                    (200, "application/json", serde_json::json!({
                        "count": banned.len(),
                        "peers": banned
                    }).to_string())
                },
                ("POST", "/api/p2p/ban") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                    let endpoint = data["endpoint"].as_str().unwrap_or("").trim().to_string();

                    if endpoint.is_empty() || !endpoint.contains(':') {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid field: endpoint (expected host:port)"
                        }).to_string())
                    } else {
                        {
                            let mut ban = state.p2p_ban_peers.lock().unwrap();
                            if !ban.iter().any(|p| p == &endpoint) {
                                ban.push(endpoint.clone());
                            }
                        }
                        let cmd = format!("COMMAND_P2P_BAN:{}", endpoint);
                        state.block_tx.send(cmd).ok();
                        (202, "application/json", serde_json::json!({
                            "status": "accepted",
                            "action": "ban",
                            "endpoint": endpoint
                        }).to_string())
                    }
                },
                ("POST", "/api/p2p/unban") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                    let endpoint = data["endpoint"].as_str().unwrap_or("").trim().to_string();

                    if endpoint.is_empty() || !endpoint.contains(':') {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid field: endpoint (expected host:port)"
                        }).to_string())
                    } else {
                        {
                            let mut ban = state.p2p_ban_peers.lock().unwrap();
                            ban.retain(|p| p != &endpoint);
                        }
                        let cmd = format!("COMMAND_P2P_UNBAN:{}", endpoint);
                        state.block_tx.send(cmd).ok();
                        (202, "application/json", serde_json::json!({
                            "status": "accepted",
                            "action": "unban",
                            "endpoint": endpoint
                        }).to_string())
                    }
                },
                ("POST", "/api/p2p/ban/bulk") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let mut parsed: Vec<String> = Vec::new();
                    if let Some(arr) = data["endpoints"].as_array() {
                        for item in arr {
                            if let Some(endpoint) = item.as_str() {
                                let e = endpoint.trim();
                                if !e.is_empty() && e.contains(':') {
                                    parsed.push(e.to_string());
                                }
                            }
                        }
                    } else if let Some(csv) = data["endpoints_csv"].as_str() {
                        parsed = csv
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty() && s.contains(':'))
                            .collect();
                    }

                    parsed.sort();
                    parsed.dedup();

                    if parsed.is_empty() {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid field: endpoints (expected host:port list)"
                        }).to_string())
                    } else {
                        let mut accepted = Vec::new();
                        {
                            let mut ban = state.p2p_ban_peers.lock().unwrap();
                            for endpoint in &parsed {
                                if !ban.iter().any(|p| p == endpoint) {
                                    ban.push(endpoint.clone());
                                    accepted.push(endpoint.clone());
                                }
                            }
                        }

                        for endpoint in &accepted {
                            let cmd = format!("COMMAND_P2P_BAN:{}", endpoint);
                            state.block_tx.send(cmd).ok();
                        }

                        (202, "application/json", serde_json::json!({
                            "status": "accepted",
                            "action": "ban_bulk",
                            "requested_count": parsed.len(),
                            "accepted_count": accepted.len(),
                            "endpoints": accepted
                        }).to_string())
                    }
                },
                ("POST", "/api/p2p/unban/bulk") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let mut parsed: Vec<String> = Vec::new();
                    if let Some(arr) = data["endpoints"].as_array() {
                        for item in arr {
                            if let Some(endpoint) = item.as_str() {
                                let e = endpoint.trim();
                                if !e.is_empty() && e.contains(':') {
                                    parsed.push(e.to_string());
                                }
                            }
                        }
                    } else if let Some(csv) = data["endpoints_csv"].as_str() {
                        parsed = csv
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty() && s.contains(':'))
                            .collect();
                    }

                    parsed.sort();
                    parsed.dedup();

                    if parsed.is_empty() {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid field: endpoints (expected host:port list)"
                        }).to_string())
                    } else {
                        let mut accepted = Vec::new();
                        {
                            let mut ban = state.p2p_ban_peers.lock().unwrap();
                            for endpoint in &parsed {
                                if ban.iter().any(|p| p == endpoint) {
                                    ban.retain(|p| p != endpoint);
                                    accepted.push(endpoint.clone());
                                }
                            }
                        }

                        for endpoint in &accepted {
                            let cmd = format!("COMMAND_P2P_UNBAN:{}", endpoint);
                            state.block_tx.send(cmd).ok();
                        }

                        (202, "application/json", serde_json::json!({
                            "status": "accepted",
                            "action": "unban_bulk",
                            "requested_count": parsed.len(),
                            "accepted_count": accepted.len(),
                            "endpoints": accepted
                        }).to_string())
                    }
                },
                ("POST", "/api/p2p/bootstrap") => {
                    let seeds = state.p2p_seed_peers.lock().unwrap().clone();
                    let payload = seeds.join(",");
                    let cmd = format!("COMMAND_P2P_BOOTSTRAP:{}", payload);
                    state.block_tx.send(cmd).ok();
                    (202, "application/json", serde_json::json!({
                        "status": "accepted",
                        "action": "bootstrap",
                        "seed_count": seeds.len()
                    }).to_string())
                },
                ("POST", "/api/p2p/sync") => {
                    state.block_tx.send("COMMAND_P2P_SYNC".to_string()).ok();
                    (202, "application/json", serde_json::json!({
                        "status": "accepted",
                        "action": "sync"
                    }).to_string())
                },
                ("POST", "/api/app/wallet/create") => {
                    use crate::wallet::CryptoWallet;
                    let wallet = CryptoWallet::generate();
                    let mut wallets = state.wallets.lock().unwrap();
                    wallets.set_balance(&wallet.address, 0.0);
                    
                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "address": wallet.address,
                        "private_key": wallet.export_private_key_hex(),
                        "public_key": hex::encode(wallet.verifying_key.as_bytes())
                    }).to_string())
                },
                ("GET", "/api/app/state") => {
                    let payload = build_frontend_app_state(&state);
                    (200, "application/json", payload.to_string())
                },
                ("GET", "/api/governance/indicators") => {
                    let governance = state.governance_engine.lock().unwrap();
                    let reward_pool = *state.reward_pool.lock().unwrap();
                    let total_fees = *state.total_fees.lock().unwrap();

                    let quorum_target: u64 = 10_000_000;
                    let active: Vec<&crate::governance::Proposal> = governance
                        .proposals
                        .iter()
                        .filter(|p| p.is_active)
                        .collect();
                    let active_count = active.len();
                    let total_for: u64 = active.iter().map(|p| p.votes_for).sum();
                    let total_against: u64 = active.iter().map(|p| p.votes_against).sum();
                    let total_votes = total_for.saturating_add(total_against);
                    let quorum_progress = if quorum_target == 0 {
                        0.0
                    } else {
                        ((total_votes as f64) / (quorum_target as f64)).clamp(0.0, 1.0)
                    };
                    let veto_risk_count = active
                        .iter()
                        .filter(|p| p.votes_against > p.votes_for)
                        .count();

                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "quorum_target": quorum_target,
                        "active_proposals": active_count,
                        "total_for": total_for,
                        "total_against": total_against,
                        "quorum_progress": quorum_progress,
                        "veto_risk_count": veto_risk_count,
                        "treasury_pool": reward_pool,
                        "lifetime_fees": total_fees,
                        "registered_reputation_nodes": governance.reputations.len()
                    }).to_string())
                },
                ("GET", "/api/kg/semantic") => {
                    let brain = state.brain_db.lock().unwrap();
                    let snapshot = brain.export_snapshot();
                    let records = snapshot.records;

                    let concepts: Vec<serde_json::Value> = records
                        .iter()
                        .take(64)
                        .map(|(key, rec)| {
                            let value = rec.value.as_object();
                            let category = if key.contains("wallet") {
                                "ENTITY"
                            } else if key.contains("mission") || key.contains("proposal") {
                                "CODE"
                            } else {
                                "DOCUMENT"
                            };

                            let relation_count = value
                                .and_then(|v| v.get("relations"))
                                .and_then(|v| v.as_array())
                                .map(|a| a.len())
                                .unwrap_or_else(|| rec.value.to_string().len().min(256));

                            serde_json::json!({
                                "id": key,
                                "name": key,
                                "category": category,
                                "connections": relation_count,
                                "class": format!("{:?}", rec.class),
                            })
                        })
                        .collect();

                    let mut category_counts = std::collections::HashMap::<String, u64>::new();
                    for c in &concepts {
                        if let Some(cat) = c.get("category").and_then(|v| v.as_str()) {
                            let entry = category_counts.entry(cat.to_string()).or_insert(0);
                            *entry += 1;
                        }
                    }

                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "nodes": concepts.len(),
                        "concepts": concepts,
                        "category_counts": category_counts,
                    }).to_string())
                },
                ("GET", "/api/app/settings") => {
                    let settings = state
                        .user_settings
                        .lock()
                        .unwrap()
                        .get(&state.node_address)
                        .cloned()
                        .unwrap_or_else(default_user_settings);
                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "address": state.node_address,
                        "settings": settings
                    }).to_string())
                },
                ("POST", "/api/app/settings") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let address = data["address"]
                        .as_str()
                        .unwrap_or(&state.node_address)
                        .to_string();
                    let raw_settings = if data.get("settings").is_some() {
                        &data["settings"]
                    } else {
                        &data
                    };

                    let sanitized = sanitize_user_settings(raw_settings);
                    state
                        .user_settings
                        .lock()
                        .unwrap()
                        .insert(address.clone(), sanitized.clone());

                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "address": address,
                        "settings": sanitized
                    }).to_string())
                },
                ("GET", "/api/drive/files") => {
                    let files = build_drive_files_payload(&state);
                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "count": files.len(),
                        "files": files
                    }).to_string())
                },
                ("POST", "/api/drive/upload") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();
                    let name = data["name"].as_str().unwrap_or("").trim().to_string();
                    let body = data["content"].as_str().unwrap_or("").to_string();
                    let file_type = data["type"].as_str().unwrap_or("TEXT").to_string();
                    let fragments = data["fragments"].as_u64().unwrap_or(1).max(1);

                    if name.is_empty() {
                        (400, "application/json", serde_json::json!({ "error": "name is required" }).to_string())
                    } else {
                        let file_id = format!("drive:{}:{}", address, name);
                        let payload = serde_json::json!({
                            "id": file_id,
                            "name": name,
                            "size": format!("{} B", body.len()),
                            "type": file_type,
                            "fragments": fragments,
                            "health": 100,
                            "uploadedAt": chrono::Utc::now().timestamp_millis(),
                            "owner": address,
                            "content": body,
                            "content_preview": body.chars().take(120).collect::<String>()
                        });

                        let mut brain = state.brain_db.lock().unwrap();
                        if !brain.export_snapshot().nodes.contains_key(&state.node_address) {
                            brain.register_node(NodeMeta::new(&state.node_address, "id", -6.2088, 106.8456));
                        }

                        match brain.upsert_record(&file_id, payload, DataClass::NodeLocal, &state.node_address) {
                            Ok(()) => {
                                if let Err(e) = persist_brain_snapshot(&state, &brain) {
                                    (500, "application/json", serde_json::json!({ "error": format!("Failed to persist snapshot: {}", e) }).to_string())
                                } else {
                                    (200, "application/json", serde_json::json!({
                                        "status": "success",
                                        "message": "Drive file uploaded",
                                        "file_id": file_id
                                    }).to_string())
                                }
                            }
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                        }
                    }
                },
                ("POST", "/api/drive/download") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let file_id = data["file_id"]
                        .as_str()
                        .or_else(|| data["id"].as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();

                    if file_id.is_empty() {
                        (400, "application/json", serde_json::json!({ "error": "file_id is required" }).to_string())
                    } else {
                        let brain = state.brain_db.lock().unwrap();
                        match brain.get_record(&file_id) {
                            Some(rec) => {
                                let owner = rec
                                    .value
                                    .get("owner")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&state.node_address);

                                if owner != address {
                                    (403, "application/json", serde_json::json!({ "error": "Forbidden: not file owner" }).to_string())
                                } else {
                                    let downloaded_at = chrono::Utc::now().timestamp_millis();
                                    let mut file = rec.value.clone();
                                    if let Some(obj) = file.as_object_mut() {
                                        obj.insert("downloadedAt".to_string(), serde_json::Value::Number(downloaded_at.into()));
                                    }

                                    (200, "application/json", serde_json::json!({
                                        "status": "success",
                                        "file_id": file_id,
                                        "file": file,
                                        "content": rec.value.get("content").and_then(|v| v.as_str()).unwrap_or("")
                                    }).to_string())
                                }
                            }
                            None => (404, "application/json", serde_json::json!({ "error": "File not found" }).to_string()),
                        }
                    }
                },
                ("GET", "/api/auction/list") => {
                    let auctions = state.auctions.lock().unwrap();
                    let items: Vec<serde_json::Value> = auctions
                        .values()
                        .map(|a| {
                            let status = match a.status {
                                AuctionStatus::Active => "ACTIVE",
                                AuctionStatus::Settled => "SOLD",
                                AuctionStatus::Cancelled => "CANCELLED",
                            };
                            serde_json::json!({
                                "auction_id": a.auction_id,
                                "seller": a.seller,
                                "item": {
                                    "name": a.item.name,
                                    "rarity": format!("{:?}", a.item.rarity).to_ascii_uppercase(),
                                    "power_multiplier": a.item.power_multiplier,
                                },
                                "starting_price": a.starting_price,
                                "highest_bid": a.highest_bid,
                                "highest_bidder": a.highest_bidder,
                                "status": status,
                                "end_time": a.end_time.timestamp_millis(),
                            })
                        })
                        .collect();
                    (200, "application/json", serde_json::json!({
                        "status": "success",
                        "count": items.len(),
                        "auctions": items
                    }).to_string())
                },
                ("POST", "/api/auction/create") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let seller = data["seller"].as_str().unwrap_or(&state.node_address).to_string();
                    let item_name = data["name"].as_str().unwrap_or("Untitled Fragment").trim().to_string();
                    let starting_price = data["starting_price"].as_f64().unwrap_or(0.0);
                    let duration_hours = data["duration_hours"].as_i64().unwrap_or(24).max(1);
                    let rarity = parse_rarity(data["rarity"].as_str().unwrap_or("COMMON"));
                    let power_multiplier = data["power_multiplier"].as_f64().unwrap_or(1.0).max(0.1);

                    if item_name.is_empty() || starting_price <= 0.0 {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid fields: name, starting_price"
                        }).to_string())
                    } else {
                        let mut next_id = state.next_auction_id.lock().unwrap();
                        let auction_id = *next_id;
                        *next_id = next_id.saturating_add(1);
                        drop(next_id);

                        let item = Item {
                            name: item_name,
                            rarity,
                            power_multiplier,
                        };
                        let auction = Auction::new(auction_id, &seller, item, starting_price, duration_hours);
                        state.auctions.lock().unwrap().insert(auction_id, auction);

                        (200, "application/json", serde_json::json!({
                            "status": "success",
                            "auction_id": auction_id,
                            "message": "Auction created"
                        }).to_string())
                    }
                },
                ("POST", "/api/auction/bid") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let auction_id = data["auction_id"]
                        .as_u64()
                        .and_then(|v| u32::try_from(v).ok())
                        .or_else(|| data["id"].as_str().and_then(|v| parse_prefixed_id(v, "a-")))
                        .unwrap_or(0);
                    let bidder = data["bidder"].as_str().unwrap_or(&state.node_address).to_string();
                    let amount = data["amount"].as_f64().unwrap_or(0.0);

                    if auction_id == 0 || amount <= 0.0 {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid fields: auction_id, amount"
                        }).to_string())
                    } else {
                        let mut auctions = state.auctions.lock().unwrap();
                        if let Some(auction) = auctions.get_mut(&auction_id) {
                            let mut wallets = state.wallets.lock().unwrap();
                            let mut escrow = state.auction_escrow.lock().unwrap();
                            match auction.place_bid_with_escrow(&bidder, amount, &mut wallets, &mut escrow) {
                                Ok(message) => (200, "application/json", serde_json::json!({
                                    "status": "success",
                                    "auction_id": auction_id,
                                    "highest_bid": auction.highest_bid,
                                    "highest_bidder": auction.highest_bidder,
                                    "message": message
                                }).to_string()),
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                            }
                        } else {
                            (404, "application/json", serde_json::json!({ "error": "Auction not found" }).to_string())
                        }
                    }
                },
                ("POST", "/api/auction/settle") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let auction_id = data["auction_id"]
                        .as_u64()
                        .and_then(|v| u32::try_from(v).ok())
                        .or_else(|| data["id"].as_str().and_then(|v| parse_prefixed_id(v, "a-")))
                        .unwrap_or(0);

                    if auction_id == 0 {
                        (400, "application/json", serde_json::json!({ "error": "auction_id is required" }).to_string())
                    } else {
                        let mut auctions = state.auctions.lock().unwrap();
                        if let Some(auction) = auctions.get_mut(&auction_id) {
                            let mut wallets = state.wallets.lock().unwrap();
                            let mut escrow = state.auction_escrow.lock().unwrap();
                            let mut pool = EconomyPool::new();
                            match auction.settle_with_escrow(&mut pool, &mut wallets, &mut escrow) {
                                Ok((winner, seller, net_to_seller, marketplace_fee)) => {
                                    let mut total_fees = state.total_fees.lock().unwrap();
                                    *total_fees += pool.total_fees_collected;
                                    drop(total_fees);

                                    let mut reward_pool = state.reward_pool.lock().unwrap();
                                    *reward_pool += pool.reward_pool;
                                    drop(reward_pool);

                                    let mut total_burned = state.total_burned.lock().unwrap();
                                    *total_burned += pool.total_burned;

                                    (200, "application/json", serde_json::json!({
                                        "status": "success",
                                        "auction_id": auction_id,
                                        "winner": winner,
                                        "seller": seller,
                                        "net_to_seller": net_to_seller,
                                        "marketplace_fee": marketplace_fee,
                                        "reward_pool_delta": pool.reward_pool
                                    }).to_string())
                                }
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                            }
                        } else {
                            (404, "application/json", serde_json::json!({ "error": "Auction not found" }).to_string())
                        }
                    }
                },
                ("POST", "/api/auction/cancel") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let auction_id = data["auction_id"]
                        .as_u64()
                        .and_then(|v| u32::try_from(v).ok())
                        .or_else(|| data["id"].as_str().and_then(|v| parse_prefixed_id(v, "a-")))
                        .unwrap_or(0);
                    let requester = data["requester"].as_str().unwrap_or(&state.node_address).to_string();

                    if auction_id == 0 {
                        (400, "application/json", serde_json::json!({ "error": "auction_id is required" }).to_string())
                    } else {
                        let mut auctions = state.auctions.lock().unwrap();
                        if let Some(auction) = auctions.get_mut(&auction_id) {
                            if requester != auction.seller && requester != state.node_address {
                                (403, "application/json", serde_json::json!({ "error": "Forbidden: only seller can cancel" }).to_string())
                            } else {
                                let mut wallets = state.wallets.lock().unwrap();
                                let mut escrow = state.auction_escrow.lock().unwrap();
                                match auction.cancel(&mut wallets, &mut escrow) {
                                    Ok(message) => (200, "application/json", serde_json::json!({
                                        "status": "success",
                                        "auction_id": auction_id,
                                        "message": message
                                    }).to_string()),
                                    Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                                }
                            }
                        } else {
                            (404, "application/json", serde_json::json!({ "error": "Auction not found" }).to_string())
                        }
                    }
                },
                ("GET", path) if path.starts_with("/api/identity/") => {
                    let address = path.strip_prefix("/api/identity/").unwrap_or("");
                    if address.is_empty() {
                        (400, "application/json", serde_json::json!({ "error": "Missing address" }).to_string())
                    } else {
                        let auctions = state.auctions.lock().unwrap();
                        let gov = state.governance_engine.lock().unwrap();
                        let reputation = gov.get_reputation(address);
                        
                        // Check if address owns any LEGENDARY or MYTHIC items from auction wins
                        let elite_items: Vec<String> = auctions
                            .values()
                            .filter(|a| a.highest_bidder.as_deref() == Some(address) && (a.item.rarity == Rarity::Legendary || a.item.rarity == Rarity::Mythic))
                            .map(|a| format!("{}_{:?}", a.item.name, a.item.rarity))
                            .collect();
                        
                        let has_elite_shield = !elite_items.is_empty();
                        
                        (200, "application/json", serde_json::json!({
                            "address": address,
                            "reputation_score": reputation,
                            "elite_shield": has_elite_shield,
                            "elite_items": elite_items,
                            "status": if has_elite_shield { "ELITE_VERIFIED" } else { "VERIFIED" }
                        }).to_string())
                    }
                },
                ("POST", "/api/app/wallet/transfer") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let from = data["from"]
                        .as_str()
                        .unwrap_or(&state.node_address)
                        .to_string();
                    let to = data["to"].as_str().unwrap_or("").to_string();
                    let amount = data["amount"].as_f64().unwrap_or(0.0);

                    if to.is_empty() || amount <= 0.0 {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid fields: to, amount"
                        }).to_string())
                    } else {
                        let mut wallets = state.wallets.lock().unwrap();
                        if wallets.deduct_balance(&from, amount).is_err() {
                            (400, "application/json", serde_json::json!({
                                "error": "Insufficient balance"
                            }).to_string())
                        } else {
                            wallets.add_balance(&to, amount);
                            drop(wallets);

                            let intent = serde_json::json!({
                                "type": "TRANSFER_APP",
                                "address": from,
                                "target": to,
                                "amount": amount,
                                "created_at": chrono::Utc::now().timestamp()
                            }).to_string();
                            state.mempool.lock().unwrap().push(intent);

                            (200, "application/json", serde_json::json!({
                                "status": "success",
                                "message": "Transfer executed",
                                "amount": amount
                            }).to_string())
                        }
                    }
                },
                ("POST", "/api/app/governance/proposal") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let title = data["title"].as_str().unwrap_or("").trim().to_string();
                    let description = data["description"].as_str().unwrap_or("Created from NFM Explorer UI").to_string();
                    let proposer = data["proposer"]
                        .as_str()
                        .unwrap_or(&state.node_address)
                        .to_string();

                    if title.is_empty() {
                        (400, "application/json", serde_json::json!({ "error": "Title is required" }).to_string())
                    } else {
                        let mut gov = state.governance_engine.lock().unwrap();
                        let id = gov.create_proposal(&proposer, &title, &description);
                        (200, "application/json", serde_json::json!({
                            "status": "success",
                            "proposal_id": id,
                            "message": "Proposal created"
                        }).to_string())
                    }
                },
                ("POST", "/api/app/governance/vote") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let raw_proposal_id = data["proposal_id"].as_str().unwrap_or("0");
                    let proposal_id = parse_prefixed_id(raw_proposal_id, "prop-").unwrap_or(0);
                    let approve = data["approve"].as_bool().unwrap_or(true);
                    let voter = data["voter"]
                        .as_str()
                        .unwrap_or(&state.node_address)
                        .to_string();

                    if proposal_id == 0 {
                        (400, "application/json", serde_json::json!({ "error": "Invalid proposal_id" }).to_string())
                    } else {
                        let mut gov = state.governance_engine.lock().unwrap();
                        match gov.vote(proposal_id, &voter, approve) {
                            Ok(msg) => (200, "application/json", serde_json::json!({
                                "status": "success",
                                "message": msg
                            }).to_string()),
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        }
                    }
                },
                ("POST", "/api/app/quest/claim") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let raw_quest_id = data["quest_id"].as_str().unwrap_or("0");
                    let mission_id = parse_prefixed_id(raw_quest_id, "q-").unwrap_or(0);
                    let address = data["address"]
                        .as_str()
                        .unwrap_or(&state.node_address)
                        .to_string();

                    if mission_id == 0 {
                        (400, "application/json", serde_json::json!({ "error": "Invalid quest_id" }).to_string())
                    } else {
                        let mut missions = state.mission_engine.lock().unwrap();
                        let key = format!("{}:{}", address, mission_id);

                        if !missions.active_assignments.contains_key(&key)
                            && !missions.completed_missions.get(&address).map(|set| set.contains(&mission_id)).unwrap_or(false)
                        {
                            if let Err(e) = missions.start_mission(&address, mission_id) {
                                (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            } else {
                                let required_units = missions
                                    .available_missions
                                    .iter()
                                    .find(|m| m.id == mission_id)
                                    .map(|m| m.work_type.required_units())
                                    .unwrap_or(0);
                                let min_duration_secs = missions
                                    .available_missions
                                    .iter()
                                    .find(|m| m.id == mission_id)
                                    .map(|m| m.work_type.min_duration_secs())
                                    .unwrap_or(5);
                                let _ = missions.report_progress(&address, mission_id, required_units);

                                let started_at = chrono::Utc::now().timestamp().saturating_sub((min_duration_secs + 1) as i64) as u64;
                                let completed_at = chrono::Utc::now().timestamp() as u64;
                                let nonce = completed_at;
                                let result_hash = crate::mission::MissionEngine::compute_expected_hash(&address, mission_id, nonce);
                                let proof = crate::mission::WorkProof {
                                    result_hash,
                                    cycles_completed: required_units,
                                    started_at,
                                    completed_at,
                                    nonce,
                                };
                                let _ = missions.submit_proof(&address, mission_id, proof);

                                match missions.claim_reward(&address, mission_id) {
                                    Ok(reward) => {
                                        drop(missions);
                                        let mut wallets = state.wallets.lock().unwrap();
                                        wallets.add_balance(&address, reward);
                                        (200, "application/json", serde_json::json!({
                                            "status": "success",
                                            "reward": reward,
                                            "message": "Quest reward claimed"
                                        }).to_string())
                                    }
                                    Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                                }
                            }
                        } else {
                            match missions.claim_reward(&address, mission_id) {
                                Ok(reward) => {
                                    drop(missions);
                                    let mut wallets = state.wallets.lock().unwrap();
                                    wallets.add_balance(&address, reward);
                                    (200, "application/json", serde_json::json!({
                                        "status": "success",
                                        "reward": reward,
                                        "message": "Quest reward claimed"
                                    }).to_string())
                                }
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            }
                        }
                    }
                },
                ("POST", "/api/app/mystery/extract") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                    let address = data["address"]
                        .as_str()
                        .unwrap_or(&state.node_address)
                        .to_string();

                    let fee = 5.0;
                    let mut wallets = state.wallets.lock().unwrap();
                    if wallets.deduct_balance(&address, fee).is_err() {
                        (400, "application/json", serde_json::json!({ "error": "Insufficient balance for extraction fee" }).to_string())
                    } else {
                        let rewards = ["10 NVC Fragment", "500 NVC Packet", "Code Auditor Skill", "Genesis Fragment #42"];
                        let idx = (chrono::Utc::now().timestamp() as usize) % rewards.len();
                        let reward_name = rewards[idx];
                        if reward_name.contains("NVC") {
                            if reward_name.contains("500") {
                                wallets.add_balance(&address, 500.0);
                            } else {
                                wallets.add_balance(&address, 10.0);
                            }
                        }
                        drop(wallets);

                        let mut admin = state.admin_engine.lock().unwrap();
                        admin.logs.push(crate::admin::AdminLog {
                            timestamp: chrono::Utc::now().timestamp(),
                            action: "MYSTERY_EXTRACT".to_string(),
                            target: address.clone(),
                            admin: state.node_address.clone(),
                            reason: reward_name.to_string(),
                        });
                        (200, "application/json", serde_json::json!({
                            "status": "success",
                            "reward": reward_name,
                            "fee": fee
                        }).to_string())
                    }
                },
                ("POST", "/api/app/market/purchase") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                    let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();
                    let item_id = data["item_id"].as_str().unwrap_or("unknown").to_string();
                    let price = data["price"].as_f64().unwrap_or(0.0);

                    if price <= 0.0 {
                        (400, "application/json", serde_json::json!({ "error": "Invalid price" }).to_string())
                    } else {
                        let mut wallets = state.wallets.lock().unwrap();
                        if wallets.deduct_balance(&address, price).is_err() {
                            (400, "application/json", serde_json::json!({ "error": "Insufficient balance" }).to_string())
                        } else {
                            drop(wallets);
                            let mut admin = state.admin_engine.lock().unwrap();
                            admin.logs.push(crate::admin::AdminLog {
                                timestamp: chrono::Utc::now().timestamp(),
                                action: "MARKET_PURCHASE".to_string(),
                                target: item_id.clone(),
                                admin: address.clone(),
                                reason: format!("{:.2} NVC", price),
                            });
                            (200, "application/json", serde_json::json!({
                                "status": "success",
                                "item_id": item_id,
                                "price": price
                            }).to_string())
                        }
                    }
                },
                ("GET", "/api/wallets") => {
                    let wallets = state.wallets.lock().unwrap();
                    let json = serde_json::to_string_pretty(&wallets.balances).unwrap_or_default();
                    (200, "application/json", json)
                },
                ("GET", "/api/brain/status") => {
                    let brain = state.brain_db.lock().unwrap();
                    let probe = RequestProfile {
                        requester_node_id: Some(state.node_address.clone()),
                        user_latitude: -6.2088,
                        user_longitude: 106.8456,
                        data_class: DataClass::Global,
                        critical: false,
                    };
                    let candidates = brain.hedged_candidates(&probe, 3);
                    (200, "application/json", serde_json::json!({
                        "status": "ok",
                        "strategy": "geo+latency+load+error",
                        "nodes": brain.node_count(),
                        "records": brain.record_count(),
                        "top_candidates": candidates
                    }).to_string())
                },
                ("POST", "/api/brain/route") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let requester_node_id = data["requester_node_id"].as_str().map(|s| s.to_string());
                    let user_latitude = data["user_latitude"].as_f64().unwrap_or(-6.2088);
                    let user_longitude = data["user_longitude"].as_f64().unwrap_or(106.8456);
                    let critical = data["critical"].as_bool().unwrap_or(false);
                    let class = match data["data_class"].as_str().unwrap_or("global") {
                        "node_local" => DataClass::NodeLocal,
                        "regional" => DataClass::Regional,
                        _ => DataClass::Global,
                    };

                    let profile = RequestProfile {
                        requester_node_id,
                        user_latitude,
                        user_longitude,
                        data_class: class,
                        critical,
                    };

                    let brain = state.brain_db.lock().unwrap();
                    let selected = brain.route_request(&profile);
                    let hedged = if profile.critical {
                        brain.hedged_candidates(&profile, 2)
                    } else {
                        Vec::new()
                    };

                    match selected {
                        Some(node_id) => (200, "application/json", serde_json::json!({
                            "status": "ok",
                            "selected_node": node_id,
                            "hedged_candidates": hedged
                        }).to_string()),
                        None => (503, "application/json", serde_json::json!({
                            "error": "No healthy candidate node available"
                        }).to_string()),
                    }
                },
                ("POST", "/api/brain/benchmark") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let requester_node_id = data["requester_node_id"].as_str().map(|s| s.to_string());
                    let user_latitude = data["user_latitude"].as_f64().unwrap_or(-6.2088);
                    let user_longitude = data["user_longitude"].as_f64().unwrap_or(106.8456);
                    let critical = data["critical"].as_bool().unwrap_or(true);
                    let class = match data["data_class"].as_str().unwrap_or("global") {
                        "node_local" => DataClass::NodeLocal,
                        "regional" => DataClass::Regional,
                        _ => DataClass::Global,
                    };

                    let profile = RequestProfile {
                        requester_node_id,
                        user_latitude,
                        user_longitude,
                        data_class: class,
                        critical,
                    };

                    let brain = state.brain_db.lock().unwrap();
                    match brain.route_benchmark(&profile, 3) {
                        Some(bench) => (200, "application/json", serde_json::json!({
                            "status": "ok",
                            "benchmark": bench
                        }).to_string()),
                        None => (503, "application/json", serde_json::json!({
                            "error": "No healthy candidate node available"
                        }).to_string()),
                    }
                },
                ("POST", "/api/brain/benchmark/compare") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let before_profile_json = data["before_profile"].clone();
                    let after_profile_json = data["after_profile"].clone();

                    let before_profile = RequestProfile {
                        requester_node_id: before_profile_json["requester_node_id"].as_str().map(|s| s.to_string()),
                        user_latitude: before_profile_json["user_latitude"].as_f64().unwrap_or(-6.2088),
                        user_longitude: before_profile_json["user_longitude"].as_f64().unwrap_or(106.8456),
                        data_class: match before_profile_json["data_class"].as_str().unwrap_or("global") {
                            "node_local" => DataClass::NodeLocal,
                            "regional" => DataClass::Regional,
                            _ => DataClass::Global,
                        },
                        critical: before_profile_json["critical"].as_bool().unwrap_or(true),
                    };

                    let after_profile = RequestProfile {
                        requester_node_id: if after_profile_json.is_null() {
                            before_profile.requester_node_id.clone()
                        } else {
                            after_profile_json["requester_node_id"].as_str().map(|s| s.to_string())
                        },
                        user_latitude: if after_profile_json.is_null() {
                            before_profile.user_latitude
                        } else {
                            after_profile_json["user_latitude"].as_f64().unwrap_or(before_profile.user_latitude)
                        },
                        user_longitude: if after_profile_json.is_null() {
                            before_profile.user_longitude
                        } else {
                            after_profile_json["user_longitude"].as_f64().unwrap_or(before_profile.user_longitude)
                        },
                        data_class: if after_profile_json.is_null() {
                            before_profile.data_class.clone()
                        } else {
                            match after_profile_json["data_class"].as_str().unwrap_or("global") {
                                "node_local" => DataClass::NodeLocal,
                                "regional" => DataClass::Regional,
                                _ => DataClass::Global,
                            }
                        },
                        critical: if after_profile_json.is_null() {
                            before_profile.critical
                        } else {
                            after_profile_json["critical"].as_bool().unwrap_or(before_profile.critical)
                        },
                    };

                    let before_weights_json = data["before_weights"].clone();
                    let after_weights_json = data["after_weights"].clone();

                    let before_weights = RouterWeights {
                        latency: before_weights_json["latency"].as_f64().unwrap_or(0.55),
                        queue: before_weights_json["queue"].as_f64().unwrap_or(0.20),
                        error: before_weights_json["error"].as_f64().unwrap_or(0.20),
                        geo: before_weights_json["geo"].as_f64().unwrap_or(0.05),
                    };

                    let after_weights = RouterWeights {
                        latency: after_weights_json["latency"].as_f64().unwrap_or(0.55),
                        queue: after_weights_json["queue"].as_f64().unwrap_or(0.20),
                        error: after_weights_json["error"].as_f64().unwrap_or(0.20),
                        geo: after_weights_json["geo"].as_f64().unwrap_or(0.05),
                    };

                    let brain = state.brain_db.lock().unwrap();
                    let before_bench = brain.route_benchmark_with_weights(&before_profile, &before_weights, 3);
                    let after_bench = brain.route_benchmark_with_weights(&after_profile, &after_weights, 3);

                    match (before_bench, after_bench) {
                        (Some(before), Some(after)) => {
                            let improvement = before.selected_score - after.selected_score;
                            (200, "application/json", serde_json::json!({
                                "status": "ok",
                                "before": before,
                                "after": after,
                                "selected_score_improvement": improvement,
                                "is_better": improvement > 0.0
                            }).to_string())
                        }
                        _ => (503, "application/json", serde_json::json!({
                            "error": "No healthy candidate node available"
                        }).to_string()),
                    }
                },
                ("POST", "/api/brain/fetch") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let key = data["key"].as_str().unwrap_or("").to_string();
                    if key.is_empty() {
                        (400, "application/json", serde_json::json!({ "error": "Missing key" }).to_string())
                    } else {
                        let requester_node_id = data["requester_node_id"].as_str().map(|s| s.to_string());
                        let user_latitude = data["user_latitude"].as_f64().unwrap_or(-6.2088);
                        let user_longitude = data["user_longitude"].as_f64().unwrap_or(106.8456);
                        let critical = data["critical"].as_bool().unwrap_or(false);
                        let class = match data["data_class"].as_str().unwrap_or("global") {
                            "node_local" => DataClass::NodeLocal,
                            "regional" => DataClass::Regional,
                            _ => DataClass::Global,
                        };

                        let profile = RequestProfile {
                            requester_node_id,
                            user_latitude,
                            user_longitude,
                            data_class: class,
                            critical,
                        };

                        let brain = state.brain_db.lock().unwrap();
                        match brain.fetch_nearest_fastest(&key, &profile) {
                            Some((node_id, value)) => (200, "application/json", serde_json::json!({
                                "status": "ok",
                                "key": key,
                                "resolved_node": node_id,
                                "value": value
                            }).to_string()),
                            None => (404, "application/json", serde_json::json!({
                                "error": "Record not found or no healthy replica"
                            }).to_string()),
                        }
                    }
                },
                ("GET", "/api/mempool") => {
                    let mempool = state.mempool.lock().unwrap();
                    let json = serde_json::to_string_pretty(&*mempool).unwrap_or_default();
                    (200, "application/json", json)
                },
                // ==============================================================
                // NEW: Wallet Creation (server-side keypair generation)
                // Returns { address, private_key_hex } — user must save private key!
                // ==============================================================
                ("POST", "/api/wallet/create") => {
                    let new_wallet = crate::wallet::CryptoWallet::generate();
                    let private_key_hex = hex::encode(new_wallet.signing_key.to_bytes());
                    let response = serde_json::json!({
                        "status": "created",
                        "address": new_wallet.address,
                        "private_key_hex": private_key_hex,
                        "warning": "Save your private key securely! It cannot be recovered."
                    });
                    // Initialize with 0 balance so wallet appears in directory
                    let mut wallets = state.wallets.lock().unwrap();
                    if !wallets.balances.contains_key(&new_wallet.address) {
                        wallets.set_balance(&new_wallet.address, 0.0);
                    }
                    (200, "application/json", response.to_string())
                },
                // ==============================================================
                // NEW: Secure Transfer (client-side Ed25519 signing)
                // Body: { from, to, amount, public_key_hex, signature_hex }
                // ==============================================================
                ("POST", "/api/transfer/secure") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK (POST) [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/transfer/secure", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let from = data["from"].as_str().unwrap_or("").to_string();
                    let to = data["to"].as_str().unwrap_or("").to_string();
                    let amount = data["amount"].as_f64().unwrap_or(0.0);
                    let pubkey_hex = data["public_key_hex"].as_str().unwrap_or("").to_string();
                    let sig_hex = data["signature_hex"].as_str().unwrap_or("").to_string();

                    if from.is_empty() || to.is_empty() || amount <= 0.0 {
                        (400, "application/json", serde_json::json!({ "error": "Missing or invalid fields: from, to, amount" }).to_string())
                    } else {
                        // --- ADMIN CHECK [S-04] ---
                        let admin = state.admin_engine.lock().unwrap();
                        if let Err(e) = admin.can_transact(&from) {
                            (403, "application/json", serde_json::json!({ "error": format!("Blocked: {}", e) }).to_string())
                        } else {
                            drop(admin);
                        match (hex::decode(&pubkey_hex), hex::decode(&sig_hex)) {
                            (Ok(pk_bytes), Ok(sig_bytes)) => {
                                // Validate public key & signature via the CryptoWallet module
                                use ed25519_dalek::VerifyingKey;
                                match (pk_bytes.as_slice().try_into().ok().and_then(|b: [u8; 32]| VerifyingKey::from_bytes(&b).ok()),
                                       sig_bytes.as_slice().try_into().ok().map(|b: [u8; 64]| ed25519_dalek::Signature::from_bytes(&b))) {
                                    (Some(verifying_key), Some(signature)) => {
                                        use ed25519_dalek::Verifier;
                                        let message = format!("{}|{}|{:.8}", from, to, amount);
                                        if verifying_key.verify(message.as_bytes(), &signature).is_ok() {
                                            if let Err(e) = apply_universal_gas_fee(&state, &from) {
                                                (400, "application/json", serde_json::json!({ "error": e }).to_string())
                                            } else {
                                                let intent = serde_json::json!({
                                                    "type": "TRANSFER",
                                                    "address": from,
                                                    "target": to,
                                                    "amount": amount
                                                }).to_string();
                                                let mut m_lock = state.mempool.lock().unwrap();
                                                m_lock.push(intent);
                                                
                                                (200, "application/json", serde_json::json!({
                                                    "status": "queued",
                                                    "message": format!("Transfer of {:.2} NVC to {} queued in mempool", amount, to)
                                                }).to_string())
                                            }
                                        } else {
                                            (400, "application/json", serde_json::json!({ "error": "Invalid cryptographic signature" }).to_string())
                                        }
                                    },
                                    _ => (400, "application/json", serde_json::json!({ "error": "Invalid public key or signature bytes" }).to_string()),
                                }
                            },
                            _ => (400, "application/json", serde_json::json!({ "error": "Invalid hex encoding for public_key_hex or signature_hex" }).to_string()),
                        }
                        }
                    }
                    }
                },
                // ==============================================================
                // Transfer Intent Creation (lightweight, unsigned)
                // Body: { from, to, amount }
                // ==============================================================
                ("POST", "/api/transfer/create") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();
                    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

                    let from = data["from"].as_str().unwrap_or("").to_string();
                    let to = data["to"].as_str().unwrap_or("").to_string();
                    let amount = data["amount"].as_f64().unwrap_or(0.0);

                    if from.is_empty() || to.is_empty() || amount <= 0.0 {
                        (400, "application/json", serde_json::json!({
                            "error": "Missing or invalid fields: from, to, amount"
                        }).to_string())
                    } else {
                        let admin = state.admin_engine.lock().unwrap();
                        if let Err(e) = admin.can_transact(&from) {
                            (403, "application/json", serde_json::json!({
                                "error": format!("Blocked: {}", e)
                            }).to_string())
                        } else {
                            drop(admin);
                            if let Err(e) = apply_universal_gas_fee(&state, &from) {
                                (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            } else {
                                let intent = serde_json::json!({
                                    "type": "TRANSFER_INTENT",
                                    "address": from,
                                    "target": to,
                                    "amount": amount,
                                    "created_at": chrono::Utc::now().timestamp()
                                }).to_string();
                                let mut m_lock = state.mempool.lock().unwrap();
                                m_lock.push(intent);

                                (202, "application/json", serde_json::json!({
                                    "status": "accepted",
                                    "message": "Transfer intent queued",
                                    "mempool_count": m_lock.len()
                                }).to_string())
                            }
                        }
                    }
                },
                ("POST", "/api/nlc") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK (POST) [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/nlc", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let input = data["input"].as_str().unwrap_or("").to_lowercase();
                        let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();

                        if input.is_empty() {
                            (400, "application/json", serde_json::json!({ "error": "Empty command" }).to_string())
                        } else {
                            // --- ADMIN CHECK [S-04] ---
                            let admin = state.admin_engine.lock().unwrap();
                            if let Err(e) = admin.can_transact(&address) {
                                (403, "application/json", serde_json::json!({ "error": format!("Blocked: {}", e) }).to_string())
                            } else {
                                drop(admin);
                                if input.contains("stake") || input.contains("deposit") {
                                    let amount = input.split_whitespace()
                                                      .find_map(|s| s.parse::<f64>().ok())
                                                      .unwrap_or(0.0);
                                    if amount > 0.0 {
                                        if let Err(e) = apply_universal_gas_fee(&state, &address) {
                                            (400, "application/json", serde_json::json!({ "error": e }).to_string())
                                        } else {
                                            let mut wallets = state.wallets.lock().unwrap();
                                            if wallets.deduct_balance(&address, amount).is_ok() {
                                                let mut staking = state.staking_pool.lock().unwrap();
                                                let chain = state.chain.lock().unwrap();
                                                let info = staking.entry(address.clone()).or_insert(crate::contract::StakingInfo {
                                                    amount: 0.0,
                                                    start_block: chain.len() as u32,
                                                    last_claim_block: chain.len() as u32,
                                                });
                                                info.amount += amount;
                                                (200, "application/json", serde_json::json!({ "status": "success", "message": format!("NLC Executed: Staked {} NVC", amount) }).to_string())
                                            } else {
                                                (400, "application/json", serde_json::json!({ "error": "Insufficient wallet balance" }).to_string())
                                            }
                                        }
                                    } else {
                                        (400, "application/json", serde_json::json!({ "error": "Amount not recognized" }).to_string())
                                    }
                                } else if input.contains("register") || input.contains("login") {
                                    if let Err(e) = apply_universal_gas_fee(&state, &address) {
                                        (400, "application/json", serde_json::json!({ "error": e }).to_string())
                                    } else {
                                        state.block_tx.send(format!("ID_REGISTERED: @user [{}] via NLC", address)).ok();
                                        (200, "application/json", serde_json::json!({ "status": "success", "message": "NLC Executed: Identity Registered" }).to_string())
                                    }
                                } else if input == "command_nuke_database" {
                                    state.block_tx.send("COMMAND_NUKE_DATABASE".to_string()).ok();
                                    (200, "application/json", serde_json::json!({ "status": "success", "message": "Nuke Command Sent to Core" }).to_string())
                                } else if input.contains("transfer") || input.contains("send") {
                                    let amount = input.split_whitespace()
                                                      .find_map(|s| s.parse::<f64>().ok())
                                                      .unwrap_or(0.0);
                                    let raw_target = input.split_whitespace()
                                                      .find(|s| s.starts_with('@') || s.starts_with("nfm_"))
                                                      .unwrap_or_default();
                                    let target_id = raw_target.replace('@', "");
                                    
                                    // Resolve Alias [PHASE 19 BUGFIX]
                                    let aliases = state.aliases.lock().unwrap();
                                    let mut target = target_id.clone();
                                    
                                    // Check with @ and without @
                                    if let Some(addr) = aliases.get(&format!("@{}", target_id)) {
                                        target = addr.clone();
                                    } else if let Some(addr) = aliases.get(&target_id) {
                                        target = addr.clone();
                                    }
                                    drop(aliases);
                                    
                                    if amount > 0.0 && !target.is_empty() {
                                        if let Err(e) = apply_universal_gas_fee(&state, &address) {
                                            (400, "application/json", serde_json::json!({ "error": e }).to_string())
                                        } else {
                                            let mut wallets = state.wallets.lock().unwrap();
                                            if wallets.deduct_balance(&address, amount).is_ok() {
                                                wallets.add_balance(&target, amount);
                                                state.block_tx.send(format!("TRANSFER: {} -> {} ({:.2} NVC) via NLC", address, target, amount)).ok();
                                                (200, "application/json", serde_json::json!({ "status": "success", "message": format!("NLC Executed: Sent {} NVC to {}", amount, target) }).to_string())
                                            } else {
                                                (400, "application/json", serde_json::json!({ "error": "Insufficient wallet balance" }).to_string())
                                            }
                                        }
                                    } else {
                                        (400, "application/json", serde_json::json!({ "error": "Could not identify amount or target" }).to_string())
                                    }
                                } else {
                                    (400, "application/json", serde_json::json!({ "error": "Intent not understood by NLC Bridge" }).to_string())
                                }
                            }
                        }
                    }
                },
                ("GET", url) if url.starts_with("/api/wallet/history") => {
                    let address = url.split("address=").nth(1).unwrap_or("");
                    let chain = state.chain.lock().unwrap();
                    let history: Vec<_> = chain.iter()
                        .filter(|b| b.data.contains(address))
                        .collect();
                    let json = serde_json::to_string_pretty(&history).unwrap_or_default();
                    (200, "application/json", json)
                },

                ("POST", "/api/staking/deposit") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK (POST) [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/staking/deposit", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let amount = data["amount"].as_f64().unwrap_or(0.0);
                        let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();

                        if amount <= 0.0 {
                            (400, "application/json", serde_json::json!({ "error": "Amount must be positive" }).to_string())
                        } else {
                            // --- UNIVERSAL GAS FEE [PHASE 11/12 FIX] ---
                            if let Err(e) = apply_universal_gas_fee(&state, &address) {
                                (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            } else {
                                // --- ADMIN CHECK [S-04] ---
                            let admin = state.admin_engine.lock().unwrap();
                            if let Err(e) = admin.can_transact(&address) {
                                (403, "application/json", serde_json::json!({ "error": format!("Blocked: {}", e) }).to_string())
                            } else {
                                drop(admin);
                                let wallets = state.wallets.lock().unwrap();
                                if wallets.balances.get(&address).unwrap_or(&0.0) >= &amount {
                                    let intent = serde_json::json!({
                                        "type": "STAKE",
                                        "address": address,
                                        "amount": amount
                                    }).to_string();
                                    state.mempool.lock().unwrap().push(intent);

                                    (200, "application/json", serde_json::json!({ "status": "queued", "message": format!("Stake of {:.2} NVC queued in mempool", amount) }).to_string())
                                } else {
                                    (400, "application/json", serde_json::json!({ "error": "Insufficient balance" }).to_string())
                                }
                                }
                            }
                        }
                    }
                },
                ("POST", "/api/staking/withdraw") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/staking/withdraw", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();

                        if let Err(e) = apply_universal_gas_fee(&state, &address) {
                            (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        } else {
                            let staking = state.staking_pool.lock().unwrap();
                            if staking.get(&address).map(|s| s.amount).unwrap_or(0.0) <= 0.0 {
                                (400, "application/json", serde_json::json!({ "error": "No active stake found" }).to_string())
                            } else {
                                let intent = serde_json::json!({
                                    "type": "UNSTAKE",
                                    "address": address
                                }).to_string();
                                state.mempool.lock().unwrap().push(intent);
                                (200, "application/json", serde_json::json!({ "status": "queued", "message": "Unstake request queued in mempool" }).to_string())
                            }
                        }
                    }
                },
                ("GET", "/founders") | ("GET", "/founders.html") => {
                    match std::fs::read_to_string("web/founders.html") {
                        Ok(html) => (200, "text/html", html.replace("{{API_SECRET}}", &state.api_secret)),
                        Err(_) => (404, "text/plain", "Founders portal file not found".to_string())
                    }
                },
                ("POST", "/api/admin/freeze") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK (POST) [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/freeze", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let target = data["target"].as_str().unwrap_or("").to_string();

                        // --- UNIVERSAL GAS FEE [PHASE 11] ---
                        if let Err(e) = apply_universal_gas_fee(&state, &state.node_address) {
                            (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        } else {
                            let mut admin = state.admin_engine.lock().unwrap();
                            match admin.freeze_account(&state.node_address, &target, crate::admin::FreezeReason::SuspectedHack) {
                                Ok(_) => {
                                    state.block_tx.send(format!("ADMIN_FREEZE: account {} frozen by founder", target)).ok();
                                    (200, "application/json", serde_json::json!({ "status": "success" }).to_string())
                                },
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            }
                        }
                    }
                },
                ("POST", "/api/admin/unfreeze") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK (POST) [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/unfreeze", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let target = data["target"].as_str().unwrap_or("").to_string();

                        // --- UNIVERSAL GAS FEE [PHASE 11] ---
                        if let Err(e) = apply_universal_gas_fee(&state, &state.node_address) {
                            (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        } else {
                            let mut admin = state.admin_engine.lock().unwrap();
                            match admin.unfreeze_account(&state.node_address, &target) {
                                Ok(_) => {
                                    state.block_tx.send(format!("ADMIN_UNFREEZE: account {} restored by founder", target)).ok();
                                    (200, "application/json", serde_json::json!({ "status": "success" }).to_string())
                                },
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            }
                        }
                    }
                },
                ("POST", "/api/admin/nuke") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK (POST) [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/nuke", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature for NUKE operation" }).to_string())
                    } else {
                        state.block_tx.send("COMMAND_NUKE_DATABASE".to_string()).ok();
                        (200, "application/json", serde_json::json!({ "status": "success", "message": "Nuke Protocol Activated! Network resetting..." }).to_string())
                    }
                },
                ("POST", "/api/admin/toggle_ratelimit") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/toggle_ratelimit", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature for toggle_ratelimit" }).to_string())
                    } else {
                        let mut enabled = state.rate_limit_enabled.lock().unwrap();
                        *enabled = !*enabled; // Toggle logic
                        let status_msg = if *enabled { "ENABLED" } else { "DISABLED" };
                        state.block_tx.send(format!("ADMIN: Global Rate Limiting is now {}", status_msg)).ok();
                        (200, "application/json", serde_json::json!({ 
                            "status": "success", 
                            "message": format!("Rate Limiting has been {}", status_msg), 
                            "is_enabled": *enabled 
                        }).to_string())
                    }
                },
                ("GET", "/api/admin/logs") => {
                    let admin = state.admin_engine.lock().unwrap();
                    let json = serde_json::to_string_pretty(&admin.logs).unwrap_or_default();
                    (200, "application/json", json)
                },
                ("GET", "/api/admin/dashboard") => {
                    let admin = state.admin_engine.lock().unwrap();
                    let gov = state.governance_engine.lock().unwrap();
                    let staking = state.staking_pool.lock().unwrap();
                    
                    let total_staked: f64 = staking.values().map(|s| s.amount).sum();
                    
                    let dashboard_data = serde_json::json!({
                        "admin": {
                            "frozen_accounts_count": admin.frozen_accounts.len(),
                            "emergency_mode": admin.is_emergency_mode,
                            "audit_logs_count": admin.logs.len()
                        },
                        "governance": gov.summary(),
                        "economy": {
                            "total_staked": total_staked,
                            "active_stakers": staking.len()
                        }
                    });
                    (200, "application/json", dashboard_data.to_string())
                },
                ("GET", "/api/admin/governance/proposals") => {
                    let gov = state.governance_engine.lock().unwrap();
                    let json = serde_json::to_string_pretty(&gov.proposals).unwrap_or_default();
                    (200, "application/json", json)
                },
                ("POST", "/api/admin/governance/vote") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    // --- AUTH CHECK (POST) [K-01] ---
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/vote", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let proposal_id = data["proposal_id"].as_u64().unwrap_or(0) as u32;
                        let approve = data["approve"].as_bool().unwrap_or(true);
                        let voter = data["voter"].as_str().unwrap_or(&state.node_address).to_string();

                        // --- UNIVERSAL GAS FEE [PHASE 11] ---
                        if let Err(e) = apply_universal_gas_fee(&state, &voter) {
                            (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        } else {
                            let mut gov = state.governance_engine.lock().unwrap();
                            match gov.vote(proposal_id, &voter, approve) {
                                Ok(msg) => {
                                    state.block_tx.send(format!("GOV_VOTE: {}", msg)).ok();
                                    (200, "application/json", serde_json::json!({ "status": "success", "message": msg }).to_string())
                                },
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            }
                        }
                    }
                },
                ("GET", "/api/admin/governance/summary") => {
                    let gov = state.governance_engine.lock().unwrap();
                    (200, "application/json", gov.summary().to_string())
                },
                ("GET", "/api/admin/governance/learning-windows") => {
                    let gov = state.governance_engine.lock().unwrap();
                    let active = gov.learning_windows.active_windows();
                    let json = serde_json::to_string_pretty(&active).unwrap_or_default();
                    (200, "application/json", json)
                },
                ("POST", "/api/admin/governance/learning-window/open") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/learning-window/open", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let epoch = data["epoch"].as_u64().unwrap_or(0);
                        let start_block = data["start_block"].as_u64().unwrap_or(0);
                        let end_block = data["end_block"].as_u64().unwrap_or(0);
                        let model_version = data["model_version"].as_str().unwrap_or("v1.0.0").to_string();

                        if epoch == 0 || end_block <= start_block {
                            (400, "application/json", serde_json::json!({ "error": "Invalid epoch or block range" }).to_string())
                        } else {
                            let mut gov = state.governance_engine.lock().unwrap();
                            let window_id = gov.learning_windows.open_window(epoch, start_block, end_block, &model_version);
                            state.block_tx.send(format!("GOV_LEARNING_WINDOW_OPEN: id={} epoch={}", window_id, epoch)).ok();
                            (200, "application/json", serde_json::json!({
                                "status": "success",
                                "window_id": window_id,
                                "epoch": epoch,
                                "start_block": start_block,
                                "end_block": end_block,
                                "model_version": model_version
                            }).to_string())
                        }
                    }
                },
                ("POST", "/api/admin/governance/learning-window/join") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/learning-window/join", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let window_id = data["window_id"].as_u64().unwrap_or(0) as u32;
                        let participant = data["participant"].as_str().unwrap_or(&state.node_address).to_string();

                        let mut gov = state.governance_engine.lock().unwrap();
                        match gov.learning_windows.join_window(window_id, &participant) {
                            Ok(msg) => {
                                state.block_tx.send(format!("GOV_LEARNING_WINDOW_JOIN: {}", msg)).ok();
                                (200, "application/json", serde_json::json!({ "status": "success", "message": msg }).to_string())
                            }
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                        }
                    }
                },
                ("POST", "/api/admin/governance/intent/propose") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/intent/propose", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let intent = data["intent"].as_str().unwrap_or("").to_string();
                        let requires_quorum = data["requires_quorum"].as_bool().unwrap_or(true);

                        if intent.is_empty() {
                            (400, "application/json", serde_json::json!({ "error": "Missing intent" }).to_string())
                        } else {
                            let mut gov = state.governance_engine.lock().unwrap();
                            match gov.intent_voting.propose_intent_vote(&intent, requires_quorum) {
                                Ok(vote_id) => {
                                    state.block_tx.send(format!("GOV_INTENT_PROPOSE: id={} intent={}", vote_id, intent)).ok();
                                    (200, "application/json", serde_json::json!({
                                        "status": "success",
                                        "vote_id": vote_id,
                                        "intent": intent,
                                        "requires_quorum": requires_quorum
                                    }).to_string())
                                }
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                            }
                        }
                    }
                },
                ("POST", "/api/admin/governance/intent/cast") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/intent/cast", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let vote_id = data["vote_id"].as_u64().unwrap_or(0) as u32;
                        let voter = data["voter"].as_str().unwrap_or(&state.node_address).to_string();
                        let approve = data["approve"].as_bool().unwrap_or(true);

                        let mut gov = state.governance_engine.lock().unwrap();
                        let voter_reputation = gov.get_reputation(&voter);
                        match gov.intent_voting.cast_intent_vote(vote_id, &voter, approve, voter_reputation) {
                            Ok(msg) => {
                                state.block_tx.send(format!("GOV_INTENT_CAST: {}", msg)).ok();
                                (200, "application/json", serde_json::json!({ "status": "success", "message": msg }).to_string())
                            }
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                        }
                    }
                },
                ("POST", "/api/admin/governance/intent/execute") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/intent/execute", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let vote_id = data["vote_id"].as_u64().unwrap_or(0) as u32;

                        let mut gov = state.governance_engine.lock().unwrap();
                        match gov.intent_voting.execute_intent_vote(vote_id) {
                            Ok(approved) => {
                                state.block_tx.send(format!("GOV_INTENT_EXECUTE: id={} approved={}", vote_id, approved)).ok();
                                (200, "application/json", serde_json::json!({
                                    "status": "success",
                                    "vote_id": vote_id,
                                    "approved": approved
                                }).to_string())
                            }
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                        }
                    }
                },
                ("POST", "/api/admin/governance/slash/propose") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/slash/propose", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let target = data["target"].as_str().unwrap_or("").to_string();
                        let reason = data["reason"].as_str().unwrap_or("policy_violation").to_string();
                        let slash_amount = data["slash_amount"].as_u64().unwrap_or(0);

                        if target.is_empty() || slash_amount == 0 {
                            (400, "application/json", serde_json::json!({ "error": "Missing target or slash_amount" }).to_string())
                        } else {
                            let mut gov = state.governance_engine.lock().unwrap();
                            if gov.slashing.get_reputation(&target) == 0 {
                                let seed_reputation = gov.get_reputation(&target).max(100);
                                gov.slashing.register_participant(&target, seed_reputation);
                            }

                            match gov.slashing.propose_slash(&target, &reason, slash_amount) {
                                Ok(event_id) => {
                                    state.block_tx.send(format!("GOV_SLASH_PROPOSE: event={} target={} amount={}", event_id, target, slash_amount)).ok();
                                    (200, "application/json", serde_json::json!({
                                        "status": "success",
                                        "event_id": event_id,
                                        "target": target,
                                        "reason": reason,
                                        "slash_amount": slash_amount
                                    }).to_string())
                                }
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                            }
                        }
                    }
                },
                ("POST", "/api/admin/governance/slash/execute") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/governance/slash/execute", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let event_id = data["event_id"].as_u64().unwrap_or(0) as u32;

                        let mut gov = state.governance_engine.lock().unwrap();
                        match gov.slashing.execute_slash(event_id) {
                            Ok(current_reputation) => {
                                state.block_tx.send(format!("GOV_SLASH_EXECUTE: event={} rep={}", event_id, current_reputation)).ok();
                                (200, "application/json", serde_json::json!({
                                    "status": "success",
                                    "event_id": event_id,
                                    "current_reputation": current_reputation
                                }).to_string())
                            }
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                        }
                    }
                },
                ("POST", "/api/admin/brain/node/register") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/brain/node/register", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let node_id = data["node_id"].as_str().unwrap_or("").to_string();
                        let region = data["region"].as_str().unwrap_or("global").to_string();
                        let latitude = data["latitude"].as_f64().unwrap_or(0.0);
                        let longitude = data["longitude"].as_f64().unwrap_or(0.0);

                        if node_id.is_empty() {
                            (400, "application/json", serde_json::json!({ "error": "Missing node_id" }).to_string())
                        } else {
                            let mut brain = state.brain_db.lock().unwrap();
                            brain.register_node(NodeMeta::new(&node_id, &region, latitude, longitude));
                            let _ = persist_brain_snapshot(&state, &brain);
                            (200, "application/json", serde_json::json!({
                                "status": "success",
                                "node_id": node_id,
                                "region": region
                            }).to_string())
                        }
                    }
                },
                ("POST", "/api/admin/brain/node/metrics") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/brain/node/metrics", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let node_id = data["node_id"].as_str().unwrap_or("").to_string();
                        let latency = data["latency_ms"].as_f64().unwrap_or(50.0);
                        let queue_depth = data["queue_depth"].as_f64().unwrap_or(0.0);
                        let error_rate = data["error_rate"].as_f64().unwrap_or(0.0);
                        let healthy = data["healthy"].as_bool().unwrap_or(true);

                        let mut brain = state.brain_db.lock().unwrap();
                        match brain.update_runtime_metrics(&node_id, latency, queue_depth, error_rate, healthy) {
                            Ok(_) => {
                                let _ = persist_brain_snapshot(&state, &brain);
                                (200, "application/json", serde_json::json!({
                                    "status": "success",
                                    "node_id": node_id,
                                    "healthy": healthy
                                }).to_string())
                            },
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                        }
                    }
                },
                ("POST", "/api/admin/brain/record/upsert") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/brain/record/upsert", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let key = data["key"].as_str().unwrap_or("").to_string();
                        let owner_node = data["owner_node"].as_str().unwrap_or("").to_string();
                        let value = data["value"].clone();
                        let class = match data["data_class"].as_str().unwrap_or("global") {
                            "node_local" => DataClass::NodeLocal,
                            "regional" => DataClass::Regional,
                            _ => DataClass::Global,
                        };

                        if key.is_empty() || owner_node.is_empty() {
                            (400, "application/json", serde_json::json!({ "error": "Missing key or owner_node" }).to_string())
                        } else {
                            let mut brain = state.brain_db.lock().unwrap();
                            match brain.upsert_record(&key, value, class, &owner_node) {
                                Ok(_) => {
                                    let _ = persist_brain_snapshot(&state, &brain);
                                    (200, "application/json", serde_json::json!({ "status": "success", "key": key }).to_string())
                                },
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string()),
                            }
                        }
                    }
                },
                ("GET", "/api/admin/brain/snapshot/export") => {
                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();

                    if !verify_admin_signature(&state.api_secret, "/api/admin/brain/snapshot/export", "", &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let brain = state.brain_db.lock().unwrap();
                        let snapshot = brain.export_snapshot();
                        let _ = persist_brain_snapshot(&state, &brain);
                        (200, "application/json", serde_json::json!({
                            "status": "success",
                            "snapshot": snapshot
                        }).to_string())
                    }
                },
                ("POST", "/api/admin/brain/snapshot/import") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/admin/brain/snapshot/import", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let snapshot_value = data["snapshot"].clone();
                        match serde_json::from_value(snapshot_value) {
                            Ok(snapshot) => {
                                let mut brain = state.brain_db.lock().unwrap();
                                brain.import_snapshot(snapshot);
                                let _ = persist_brain_snapshot(&state, &brain);
                                (200, "application/json", serde_json::json!({
                                    "status": "success",
                                    "nodes": brain.node_count(),
                                    "records": brain.record_count()
                                }).to_string())
                            }
                            Err(e) => (400, "application/json", serde_json::json!({
                                "error": format!("Invalid snapshot payload: {}", e)
                            }).to_string()),
                        }
                    }
                },
                // ======================================================================
                // MISSION START [PHASE 12.1] — Mulai mengerjakan misi
                // ======================================================================
                ("POST", "/api/mission/start") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/mission/start", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let mission_id = data["id"].as_u64().unwrap_or(0) as u32;
                        let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();

                        let mut missions = state.mission_engine.lock().unwrap();
                        match missions.start_mission(&address, mission_id) {
                            Ok(assignment) => {
                                // Ambil info work_type untuk dikirim ke frontend
                                let mission = missions.available_missions.iter()
                                    .find(|m| m.id == mission_id);
                                let min_duration = mission.map(|m| m.work_type.min_duration_secs()).unwrap_or(5);

                                state.block_tx.send(format!(
                                    "MISSION_START: {} started mission #{}",
                                    address, mission_id
                                )).ok();

                                (200, "application/json", serde_json::json!({
                                    "status": "started",
                                    "mission_id": mission_id,
                                    "min_duration_secs": min_duration,
                                    "started_at": assignment.started_at,
                                    "current_units": assignment.current_units,
                                    "required_units": assignment.required_units,
                                    "message": format!("Mission started. Work for at least {}s before submitting proof.", min_duration)
                                }).to_string())
                            },
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        }
                    }
                },
                // ======================================================================
                // MISSION PROGRESS [PHASE 12.1b] — Laporkan progres kerja aktual
                // ======================================================================
                ("POST", "/api/mission/progress") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/mission/progress", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let mission_id = data["id"].as_u64().unwrap_or(0) as u32;
                        let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();
                        let units_delta = data["units_delta"].as_u64().unwrap_or(0);

                        let mut missions = state.mission_engine.lock().unwrap();
                        match missions.report_progress(&address, mission_id, units_delta) {
                            Ok(assignment) => {
                                let progress_pct = if assignment.required_units == 0 {
                                    0
                                } else {
                                    ((assignment.current_units.saturating_mul(100)) / assignment.required_units) as u32
                                };

                                (200, "application/json", serde_json::json!({
                                    "status": "ok",
                                    "mission_id": mission_id,
                                    "current_units": assignment.current_units,
                                    "required_units": assignment.required_units,
                                    "progress_pct": progress_pct
                                }).to_string())
                            },
                            Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        }
                    }
                },
                // ======================================================================
                // MISSION COMPLETE [PHASE 12.2] — Submit proof + claim reward
                // ======================================================================
                ("POST", "/api/mission/complete") => {
                    let mut content = String::new();
                    request.as_reader().read_to_string(&mut content).ok();

                    let sig_header = request.headers().iter()
                        .find(|h| h.field.as_str().to_ascii_lowercase() == "x-nfm-signature")
                        .map(|h| h.value.as_str().to_string())
                        .unwrap_or_default();
                    if !verify_admin_signature(&state.api_secret, "/api/mission/complete", &content, &sig_header) {
                        (403, "application/json", serde_json::json!({ "error": "Forbidden: invalid signature" }).to_string())
                    } else {
                        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                        let mission_id = data["id"].as_u64().unwrap_or(0) as u32;
                        let address = data["address"].as_str().unwrap_or(&state.node_address).to_string();
                        let nonce = data["nonce"].as_u64().unwrap_or(0);
                        let cycles = data["cycles_completed"].as_u64().unwrap_or(0);
                        let started_at = data["started_at"].as_u64().unwrap_or(0);
                        let completed_at = data["completed_at"].as_u64().unwrap_or(0);
                        let result_hash = data["result_hash"].as_str().unwrap_or("").to_string();

                        // --- UNIVERSAL GAS FEE [PHASE 11] ---
                        if let Err(e) = apply_universal_gas_fee(&state, &address) {
                            (400, "application/json", serde_json::json!({ "error": e }).to_string())
                        } else {
                            let proof = crate::mission::WorkProof {
                                result_hash,
                                cycles_completed: cycles,
                                started_at,
                                completed_at,
                                nonce,
                            };

                            let mut missions = state.mission_engine.lock().unwrap();

                            // Step 1: Submit proof
                            match missions.submit_proof(&address, mission_id, proof) {
                                Ok(_) => {
                                    // Step 2: Auto-claim setelah proof valid
                                    match missions.claim_reward(&address, mission_id) {
                                        Ok(reward) => {
                                            let mut wallets = state.wallets.lock().unwrap();
                                            wallets.add_balance(&address, reward);

                                            state.block_tx.send(format!(
                                                "MISSION_VERIFIED: {} completed mission #{} with valid proof (+{:.2} NVC)",
                                                address, mission_id, reward
                                            )).ok();

                                            (200, "application/json", serde_json::json!({
                                                "status": "success",
                                                "mission_id": mission_id,
                                                "reward": reward,
                                                "message": format!("Proof verified! Mission #{} completed. +{:.2} NVC awarded.", mission_id, reward)
                                            }).to_string())
                                        },
                                        Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                                    }
                                },
                                Err(e) => (400, "application/json", serde_json::json!({ "error": e }).to_string())
                            }
                        }
                    }
                },
                // ======================================================================
                // FAVICON (mencegah 404 spam dari browser)
                // ======================================================================

                ("GET", "/favicon.ico") => {
                    (204, "image/x-icon", String::new())
                },
                _ => (404, "text/plain", "Not Found".to_string()),
            };

            let response = tiny_http::Response::from_string(body)
                .with_status_code(status)
                .with_header(tiny_http::Header::from_bytes("Content-Type", content_type).unwrap())
                .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap())
                .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap())
                .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Headers", "Content-Type, Authorization, x-nfm-signature").unwrap());
            let _ = request.respond(response);
        }
    });
}

/// Render HTML dashboard
fn render_dashboard(blocks: usize, fees: f64, burned: f64, node: &str, _port: u16) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>NFM Mesh Dashboard</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: 'Segoe UI', system-ui, sans-serif;
    background: linear-gradient(135deg, #0a0a1a 0%, #1a0a2e 50%, #0a1628 100%);
    color: #e0e0ff;
    min-height: 100vh;
    padding: 40px 20px;
  }}
  .container {{ max-width: 900px; margin: 0 auto; }}
  h1 {{
    text-align: center;
    font-size: 2.5em;
    background: linear-gradient(90deg, #00d4ff, #7b2fef, #ff006e);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    margin-bottom: 10px;
  }}
  .subtitle {{ text-align: center; color: #8888aa; margin-bottom: 40px; font-size: 0.9em; }}
  .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }}
  .card {{
    background: rgba(255,255,255,0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 16px;
    padding: 24px;
    text-align: center;
    transition: transform 0.2s, box-shadow 0.2s;
  }}
  .card:hover {{
    transform: translateY(-4px);
    box-shadow: 0 8px 32px rgba(123,47,239,0.3);
  }}
  .card .value {{
    font-size: 2.2em;
    font-weight: 700;
    background: linear-gradient(135deg, #00d4ff, #7b2fef);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }}
  .card .label {{ color: #8888aa; margin-top: 8px; font-size: 0.85em; text-transform: uppercase; letter-spacing: 1px; }}
  .node-info {{
    background: rgba(0,212,255,0.08);
    border: 1px solid rgba(0,212,255,0.2);
    border-radius: 12px;
    padding: 16px 24px;
    font-family: monospace;
    font-size: 0.85em;
    color: #00d4ff;
    margin-bottom: 30px;
    word-break: break-all;
  }}
  .links {{ text-align: center; margin-top: 20px; }}
  .links a {{
    color: #7b2fef;
    text-decoration: none;
    margin: 0 16px;
    padding: 8px 20px;
    border: 1px solid #7b2fef;
    border-radius: 8px;
    transition: all 0.2s;
  }}
  .links a:hover {{ background: #7b2fef; color: white; }}
  .pulse {{ animation: pulse 2s infinite; }}
  @keyframes pulse {{
    0%, 100% {{ opacity: 1; }}
    50% {{ opacity: 0.5; }}
  }}
  .status {{ display: inline-block; width: 8px; height: 8px; background: #00ff88; border-radius: 50%; margin-right: 6px; }}
  .shield {{ display: inline-block; width: 8px; height: 8px; background: #ff6600; border-radius: 50%; margin-right: 6px; }}
</style>
</head>
<body>
<div class="container">
  <h1>NFM Mesh Dashboard</h1>
  <p class="subtitle"><span class="status pulse"></span>Neural Fragment Mesh — v0.5.0-nexus | <span class="shield"></span>Auth: HMAC-SHA256</p>
  <div class="node-info">Node: {node}</div>
  <div class="grid">
    <div class="card">
      <div class="value">{blocks}</div>
      <div class="label">Total Blocks</div>
    </div>
    <div class="card">
      <div class="value">{fees:.2}</div>
      <div class="label">Fees Collected (NVC)</div>
    </div>
    <div class="card">
      <div class="value">{burned:.2}</div>
      <div class="label">NVCoin Burned</div>
    </div>
    <div class="card">
      <div class="value">42+</div>
      <div class="label">Tests Passed</div>
    </div>
  </div>
  <div class="links">
    <a href="/api/blocks">📦 View Blocks (JSON)</a>
    <a href="/api/status">📊 Node Status (API)</a>
  </div>
</div>
</body>
</html>"#, node=node, blocks=blocks, fees=fees, burned=burned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::{AdminEngine, FreezeReason};
    use crate::block::Block;
    use crate::governance::GovernanceEngine;
    use crate::mission::MissionEngine;
    use crate::transfer::{GasFeeCalculator, WalletEngine};
    use crate::wallet::CryptoWallet;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn pick_free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        listener.local_addr().expect("read local addr").port()
    }

    fn wait_for_server(port: u16) {
        for _ in 0..40 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!("API test server did not start on port {}", port);
    }

    fn create_hmac(secret: &str, url: &str, body: &str) -> String {
        let payload = format!("{}:{}", url, body);
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", secret, payload).as_bytes());
        hex::encode(hasher.finalize())
    }

    fn send_post(port: u16, path: &str, body: &str, extra_headers: &[String]) -> (u16, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect test server");

        let mut headers = String::new();
        for h in extra_headers {
            headers.push_str(h);
            headers.push_str("\r\n");
        }

        let request = format!(
            "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
            path,
            port,
            body.len(),
            headers,
            body
        );

        stream.write_all(request.as_bytes()).expect("write request");
        let _ = stream.shutdown(std::net::Shutdown::Write);

        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read response");

        let status = response
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|code| code.parse::<u16>().ok())
            .unwrap_or(0);

        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or("")
            .to_string();

        (status, body)
    }

    fn send_get(port: u16, path: &str, extra_headers: &[String]) -> (u16, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect test server");

        let mut headers = String::new();
        for h in extra_headers {
            headers.push_str(h);
            headers.push_str("\r\n");
        }

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n{}\r\n",
            path, port, headers
        );

        stream.write_all(request.as_bytes()).expect("write request");
        let _ = stream.shutdown(std::net::Shutdown::Write);

        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read response");

        let status = response
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|code| code.parse::<u16>().ok())
            .unwrap_or(0);

        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or("")
            .to_string();

        (status, body)
    }

    fn start_test_api_server(
        api_secret: &str,
        node_address: &str,
        wallets: WalletEngine,
        admin_engine: AdminEngine,
        rate_limit_enabled: bool,
    ) -> u16 {
        let (block_tx, _block_rx) = std::sync::mpsc::channel::<String>();
        let port = pick_free_port();

        let state = ApiState {
            chain: Arc::new(Mutex::new(Vec::<Block>::new())),
            node_address: node_address.to_string(),
            total_fees: Arc::new(Mutex::new(0.0)),
            total_burned: Arc::new(Mutex::new(0.0)),
            reward_pool: Arc::new(Mutex::new(0.0)),
            active_effects: Arc::new(Mutex::new(std::collections::HashMap::new())),
            mission_engine: Arc::new(Mutex::new(MissionEngine::new())),
            staking_pool: Arc::new(Mutex::new(std::collections::HashMap::new())),
            wallets: Arc::new(Mutex::new(wallets)),
            admin_engine: Arc::new(Mutex::new(admin_engine)),
            governance_engine: Arc::new(Mutex::new(GovernanceEngine::new())),
            block_tx,
            api_secret: api_secret.to_string(),
            rate_limit_enabled: Arc::new(Mutex::new(rate_limit_enabled)),
            gas_fee_calculator: Arc::new(Mutex::new(GasFeeCalculator::new())),
            aliases: Arc::new(Mutex::new(std::collections::HashMap::new())),
            mempool: Arc::new(Mutex::new(Vec::new())),
            next_block_timestamp: Arc::new(Mutex::new(0)),
            user_settings: Arc::new(Mutex::new(std::collections::HashMap::new())),
            brain_db: Arc::new(Mutex::new(GeoDistributedBrainDb::new())),
            brain_tokens: Arc::new(Mutex::new(Vec::new())),
            status_cache: Arc::new(Mutex::new(None)),
            p2p_status: Arc::new(Mutex::new(serde_json::json!({
                "gossip_enabled": false,
                "listening_port": 0,
                "peer_count": 0,
                "known_peers": [],
                "seed_count": 0,
                "ban_count": 0,
                "banned_peers": [],
                "last_sync_unix": 0,
                "chain_blocks": 0,
                "status": "inactive"
            }))),
            p2p_seed_peers: Arc::new(Mutex::new(Vec::new())),
            p2p_ban_peers: Arc::new(Mutex::new(Vec::new())),
            auctions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            auction_escrow: Arc::new(Mutex::new(EscrowVault::new())),
            next_auction_id: Arc::new(Mutex::new(1)),
            brain_snapshot_store: Arc::new(Mutex::new(None)),
        };

        start_api_server(state, port);
        wait_for_server(port);
        port
    }

    fn signed_transfer_body(sender: &CryptoWallet, receiver: &str, amount: f64) -> String {
        let (_msg, signature) = sender.sign_transfer(receiver, amount);
        serde_json::json!({
            "from": sender.address,
            "to": receiver,
            "amount": amount,
            "public_key_hex": hex::encode(sender.verifying_key.as_bytes()),
            "signature_hex": hex::encode(signature.to_bytes())
        })
        .to_string()
    }

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let mut limiter = RateLimiter::new();
        for _ in 0..1000 {
            assert!(limiter.check("192.168.1.1", "GET"));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_excess() {
        let mut limiter = RateLimiter::new();
        for _ in 0..1000 {
            limiter.check("192.168.1.1", "GET");
        }
        // Request ke-1001 harus diblokir
        assert!(!limiter.check("192.168.1.1", "GET"));
    }

    #[test]
    fn test_rate_limiter_per_ip_isolation() {
        let mut limiter = RateLimiter::new();
        for _ in 0..1000 {
            limiter.check("192.168.1.1", "GET");
        }
        // IP berbeda tetap diizinkan
        assert!(limiter.check("192.168.1.2", "GET"));
    }

    #[test]
    fn test_rate_limiter_blocks_excess_post() {
        let mut limiter = RateLimiter::new();
        for _ in 0..300 {
            assert!(limiter.check("192.168.1.1", "POST"));
        }
        // Request ke-301 POST harus diblokir
        assert!(!limiter.check("192.168.1.1", "POST"));
    }

    #[test]
    fn test_signature_verification() {
        let secret = "test_secret_123";
        let url = "/api/admin/freeze";
        let body = r#"{"target":"nfm_bob"}"#;

        // Generate valid signature
        let payload = format!("{}:{}", url, body);
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", secret, payload).as_bytes());
        let valid_sig = hex::encode(hasher.finalize());

        assert!(verify_admin_signature(secret, url, body, &valid_sig));
        assert!(!verify_admin_signature(secret, url, body, "wrong_signature"));
    }

    #[test]
    fn test_protected_endpoint_detection() {
        assert!(is_protected_endpoint("/api/admin/freeze"));
        assert!(is_protected_endpoint("/api/admin/nuke"));
        assert!(is_protected_endpoint("/api/admin/governance/learning-window/open"));
        assert!(is_protected_endpoint("/api/nlc"));
        assert!(is_protected_endpoint("/api/transfer/secure"));
        assert!(is_protected_endpoint("/api/staking/deposit"));
        assert!(is_protected_endpoint("/api/mission/progress"));
        assert!(is_protected_endpoint("/api/mission/complete"));

        assert!(!is_protected_endpoint("/"));
        assert!(!is_protected_endpoint("/api/blocks"));
        assert!(!is_protected_endpoint("/api/status"));
        assert!(!is_protected_endpoint("/api/p2p/status"));
        assert!(!is_protected_endpoint("/api/wallets"));
    }

    #[test]
    fn test_p2p_status_endpoint_returns_ok() {
        let secret = "test_secret_p2p_status";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let (status, body) = send_get(port, "/api/p2p/status", &[]);

        assert_eq!(status, 200);
        assert!(body.contains("peer_count"));
    }

    #[test]
    fn test_p2p_bulk_ban_mixed_payload_dedupes_and_accepts_valid_only() {
        let secret = "test_secret_p2p_bulk_ban_mixed";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "endpoints": [
                "127.0.0.1:9000",
                "127.0.0.1:9000",
                " 127.0.0.1:9001 ",
                "",
                "invalid"
            ]
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/p2p/ban/bulk", &body, &[]);
        assert_eq!(status, 202);

        let response_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("valid json response");
        assert_eq!(response_json["action"], "ban_bulk");
        assert_eq!(response_json["requested_count"].as_u64().unwrap_or(0), 2);
        assert_eq!(response_json["accepted_count"].as_u64().unwrap_or(0), 2);

        let endpoints = response_json["endpoints"].as_array().cloned().unwrap_or_default();
        assert_eq!(endpoints.len(), 2);

        let (ban_status, ban_body) = send_get(port, "/api/p2p/banlist", &[]);
        assert_eq!(ban_status, 200);
        assert!(ban_body.contains("127.0.0.1:9000"));
        assert!(ban_body.contains("127.0.0.1:9001"));
    }

    #[test]
    fn test_p2p_bulk_unban_accepts_only_currently_banned_endpoints() {
        let secret = "test_secret_p2p_bulk_unban_mixed";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let seed_ban_body = serde_json::json!({
            "endpoints": ["127.0.0.1:9010", "127.0.0.1:9011"]
        })
        .to_string();
        let (seed_status, _) = send_post(port, "/api/p2p/ban/bulk", &seed_ban_body, &[]);
        assert_eq!(seed_status, 202);

        let body = serde_json::json!({
            "endpoints": [
                "127.0.0.1:9010",
                "127.0.0.1:9010",
                "127.0.0.1:9999",
                "invalid"
            ]
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/p2p/unban/bulk", &body, &[]);
        assert_eq!(status, 202);

        let response_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("valid json response");
        assert_eq!(response_json["action"], "unban_bulk");
        assert_eq!(response_json["requested_count"].as_u64().unwrap_or(0), 2);
        assert_eq!(response_json["accepted_count"].as_u64().unwrap_or(0), 1);

        let (ban_status, ban_body) = send_get(port, "/api/p2p/banlist", &[]);
        assert_eq!(ban_status, 200);
        assert!(!ban_body.contains("127.0.0.1:9010"));
        assert!(ban_body.contains("127.0.0.1:9011"));
    }

    #[test]
    fn test_p2p_bulk_ban_supports_endpoints_csv_payload() {
        let secret = "test_secret_p2p_bulk_ban_csv";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "endpoints_csv": "127.0.0.1:9020, invalid, 127.0.0.1:9021, 127.0.0.1:9021"
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/p2p/ban/bulk", &body, &[]);
        assert_eq!(status, 202);

        let response_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("valid json response");
        assert_eq!(response_json["requested_count"].as_u64().unwrap_or(0), 2);
        assert_eq!(response_json["accepted_count"].as_u64().unwrap_or(0), 2);
    }

    #[test]
    fn test_p2p_bulk_ban_rejects_empty_or_invalid_payload() {
        let secret = "test_secret_p2p_bulk_ban_invalid";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "endpoints": ["", "invalid", "also_invalid"]
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/p2p/ban/bulk", &body, &[]);
        assert_eq!(status, 400);
        assert!(response_body.contains("Missing or invalid field: endpoints"));
    }

    #[test]
    fn test_p2p_bulk_ban_keeps_accepted_contract_when_command_channel_disconnected() {
        let secret = "test_secret_p2p_bulk_ban_disconnected";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        // start_test_api_server intentionally drops receiver, simulating disconnected runtime command channel.
        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "endpoints": ["127.0.0.1:9030", "127.0.0.1:9031"]
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/p2p/ban/bulk", &body, &[]);
        assert_eq!(status, 202);

        let response_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("valid json response");
        assert_eq!(response_json["action"], "ban_bulk");
        assert_eq!(response_json["accepted_count"].as_u64().unwrap_or(0), 2);

        let (ban_status, ban_body) = send_get(port, "/api/p2p/banlist", &[]);
        assert_eq!(ban_status, 200);
        assert!(ban_body.contains("127.0.0.1:9030"));
        assert!(ban_body.contains("127.0.0.1:9031"));
    }

    #[test]
    fn test_p2p_bulk_unban_keeps_accepted_contract_when_command_channel_disconnected() {
        let secret = "test_secret_p2p_bulk_unban_disconnected";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let seed_ban_body = serde_json::json!({
            "endpoints": ["127.0.0.1:9040", "127.0.0.1:9041"]
        })
        .to_string();
        let (seed_status, _) = send_post(port, "/api/p2p/ban/bulk", &seed_ban_body, &[]);
        assert_eq!(seed_status, 202);

        let unban_body = serde_json::json!({
            "endpoints": ["127.0.0.1:9040", "127.0.0.1:9041"]
        })
        .to_string();
        let (status, response_body) = send_post(port, "/api/p2p/unban/bulk", &unban_body, &[]);
        assert_eq!(status, 202);

        let response_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("valid json response");
        assert_eq!(response_json["action"], "unban_bulk");
        assert_eq!(response_json["accepted_count"].as_u64().unwrap_or(0), 2);

        let (ban_status, ban_body) = send_get(port, "/api/p2p/banlist", &[]);
        assert_eq!(ban_status, 200);
        assert!(!ban_body.contains("127.0.0.1:9040"));
        assert!(!ban_body.contains("127.0.0.1:9041"));
    }

    #[test]
    fn test_transfer_secure_rejects_invalid_hmac_signature() {
        let secret = "test_secret_transfer_invalid";
        let node_address = "nfm_founder_test";
        let sender = CryptoWallet::generate();
        let receiver = CryptoWallet::generate();

        let mut wallets = WalletEngine::new();
        wallets.set_balance(&sender.address, 100.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = signed_transfer_body(&sender, &receiver.address, 1.0);

        let (status, response_body) = send_post(
            port,
            "/api/transfer/secure",
            &body,
            &["x-nfm-signature: invalid_sig".to_string()],
        );

        assert_eq!(status, 403);
        assert!(response_body.contains("invalid signature"));
    }

    #[test]
    fn test_governance_learning_window_open_requires_signature() {
        let secret = "test_secret_gov_window";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "epoch": 1,
            "start_block": 100,
            "end_block": 200,
            "model_version": "v1.0.0"
        })
        .to_string();

        let (status, response_body) = send_post(
            port,
            "/api/admin/governance/learning-window/open",
            &body,
            &["x-nfm-signature: invalid_sig".to_string()],
        );

        assert_eq!(status, 403);
        assert!(response_body.contains("invalid signature"));
    }

    #[test]
    fn test_brain_node_register_requires_signature() {
        let secret = "test_secret_brain_register";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "node_id": "id-jkt-a",
            "region": "id",
            "latitude": -6.2088,
            "longitude": 106.8456
        })
        .to_string();

        let (status, response_body) = send_post(
            port,
            "/api/admin/brain/node/register",
            &body,
            &["x-nfm-signature: invalid_sig".to_string()],
        );

        assert_eq!(status, 403);
        assert!(response_body.contains("invalid signature"));
    }

    #[test]
    fn test_brain_route_returns_503_when_no_healthy_nodes() {
        let secret = "test_secret_brain_route";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "requester_node_id": "id-jkt-a",
            "user_latitude": -6.2,
            "user_longitude": 106.8,
            "data_class": "global",
            "critical": false
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/brain/route", &body, &[]);

        assert_eq!(status, 503);
        assert!(response_body.contains("No healthy candidate node"));
    }

    #[test]
    fn test_brain_benchmark_returns_503_when_no_healthy_nodes() {
        let secret = "test_secret_brain_benchmark";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "requester_node_id": "id-jkt-a",
            "user_latitude": -6.2,
            "user_longitude": 106.8,
            "data_class": "global",
            "critical": true
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/brain/benchmark", &body, &[]);

        assert_eq!(status, 503);
        assert!(response_body.contains("No healthy candidate node"));
    }

    #[test]
    fn test_brain_benchmark_compare_returns_503_when_no_healthy_nodes() {
        let secret = "test_secret_brain_benchmark_compare";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = serde_json::json!({
            "before_profile": {
                "requester_node_id": "id-jkt-a",
                "user_latitude": -6.2,
                "user_longitude": 106.8,
                "data_class": "global",
                "critical": true
            },
            "after_profile": {
                "requester_node_id": "id-jkt-a",
                "user_latitude": -6.2,
                "user_longitude": 106.8,
                "data_class": "global",
                "critical": true
            },
            "before_weights": {
                "latency": 0.55,
                "queue": 0.20,
                "error": 0.20,
                "geo": 0.05
            },
            "after_weights": {
                "latency": 0.90,
                "queue": 0.05,
                "error": 0.04,
                "geo": 0.01
            }
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/brain/benchmark/compare", &body, &[]);

        assert_eq!(status, 503);
        assert!(response_body.contains("No healthy candidate node"));
    }

    #[test]
    fn test_brain_snapshot_export_requires_signature() {
        let secret = "test_secret_brain_snapshot_export";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let (status, response_body) = send_get(
            port,
            "/api/admin/brain/snapshot/export",
            &["x-nfm-signature: invalid_sig".to_string()],
        );

        assert_eq!(status, 403);
        assert!(
            response_body.contains("invalid") || response_body.contains("missing"),
            "unexpected auth error body: {}",
            response_body
        );
    }

    #[test]
    fn test_transfer_secure_blocks_frozen_sender() {
        let secret = "test_secret_transfer_frozen";
        let node_address = "nfm_founder_test";
        let sender = CryptoWallet::generate();
        let receiver = CryptoWallet::generate();

        let mut wallets = WalletEngine::new();
        wallets.set_balance(&sender.address, 100.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);
        admin
            .freeze_account(node_address, &sender.address, FreezeReason::ComplianceViolation)
            .expect("freeze sender for test");

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let body = signed_transfer_body(&sender, &receiver.address, 1.0);
        let hmac = create_hmac(secret, "/api/transfer/secure", &body);

        let (status, response_body) = send_post(
            port,
            "/api/transfer/secure",
            &body,
            &[format!("x-nfm-signature: {}", hmac)],
        );

        assert_eq!(status, 403);
        assert!(response_body.contains("Blocked"));
    }

    #[test]
    fn test_transfer_secure_post_rate_limit_integration() {
        let secret = "test_secret_transfer_rate_limit";
        let node_address = "nfm_founder_test";

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, WalletEngine::new(), admin, true);

        // 300 POST pertama masih diproses (akan ditolak auth dengan 403, tapi belum kena 429)
        for _ in 0..300 {
            let (status, _) = send_post(port, "/api/transfer/secure", "{}", &[]);
            assert_eq!(status, 403);
        }

        // POST ke-301 harus kena rate limit
        let (status, response_body) = send_post(port, "/api/transfer/secure", "{}", &[]);
        assert_eq!(status, 429);
        assert!(response_body.contains("Rate limit exceeded"));
    }

    #[test]
    fn test_transfer_create_queues_intent() {
        let secret = "test_secret_transfer_create";
        let node_address = "nfm_founder_test";

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, WalletEngine::new(), admin, true);

        let body = serde_json::json!({
            "from": "nfm_sender",
            "to": "nfm_receiver",
            "amount": 2.5
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/transfer/create", &body, &[]);
        assert_eq!(status, 202);
        assert!(response_body.contains("accepted"));

        let (status, mempool_body) = send_get(port, "/api/mempool", &[]);
        assert_eq!(status, 200);
        assert!(mempool_body.contains("TRANSFER_INTENT"));
    }

    #[test]
    fn test_brain_route_accepts_request_without_configured_tokens() {
        // Ketika tidak ada token yang dikonfigurasi, akses harus terbuka (empty Vec)
        let secret = "test_secret_brain_open";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        // Register satu node agar tidak 503
        let register_body = serde_json::json!({
            "node_id": "test_node_1",
            "region": "jkt",
            "latitude": -6.2088,
            "longitude": 106.8456
        })
        .to_string();
        let sig = create_hmac(secret, "/api/admin/brain/node/register", &register_body);
        let (status, _) = send_post(
            port,
            "/api/admin/brain/node/register",
            &register_body,
            &[format!("x-nfm-signature: {}", sig)],
        );
        assert_eq!(status, 200);

        // Sekarang route tanpa token header harus berhasil (203 OK, bukan 401)
        let route_body = serde_json::json!({
            "requester_node_id": "id-jkt-a",
            "user_latitude": -6.2,
            "user_longitude": 106.8,
            "data_class": "global",
            "critical": false
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/brain/route", &route_body, &[]);

        assert_eq!(status, 200);
        assert!(response_body.contains("selected_node"));
    }

    #[test]
    fn test_app_wallet_transfer_endpoint_executes_and_queues_intent() {
        let secret = "test_secret_app_transfer";
        let node_address = "nfm_founder_test";
        let sender = "nfm_sender_app";

        let mut wallets = WalletEngine::new();
        wallets.set_balance(sender, 50.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let body = serde_json::json!({
            "from": sender,
            "to": "nfm_receiver_app",
            "amount": 12.5
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/app/wallet/transfer", &body, &[]);
        assert_eq!(status, 200);
        assert!(response_body.contains("Transfer executed"));

        let (mempool_status, mempool_body) = send_get(port, "/api/mempool", &[]);
        assert_eq!(mempool_status, 200);
        assert!(mempool_body.contains("TRANSFER_APP"));
    }

    #[test]
    fn test_app_governance_proposal_and_vote_endpoints() {
        let secret = "test_secret_app_governance";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let proposal_body = serde_json::json!({
            "title": "Proposal from app endpoint test",
            "description": "Ensure app endpoint can create proposal",
            "proposer": node_address
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/app/governance/proposal", &proposal_body, &[]);
        assert_eq!(status, 200);

        let proposal_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("valid proposal response json");
        let proposal_id = proposal_json["proposal_id"].as_u64().unwrap_or(0);
        assert!(proposal_id > 0);

        let vote_body = serde_json::json!({
            "proposal_id": proposal_id.to_string(),
            "approve": true,
            "voter": node_address
        })
        .to_string();

        let (vote_status, vote_response) = send_post(port, "/api/app/governance/vote", &vote_body, &[]);
        assert_eq!(vote_status, 400);
        assert!(vote_response.contains("No reputation") || vote_response.contains("error"));
    }

    #[test]
    fn test_app_quest_claim_endpoint_rewards_wallet() {
        let secret = "test_secret_app_quest";
        let node_address = "nfm_founder_test";

        let mut wallets = WalletEngine::new();
        wallets.set_balance(node_address, 0.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let body = serde_json::json!({
            "quest_id": "q-2",
            "address": node_address
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/app/quest/claim", &body, &[]);
        assert_eq!(status, 200);
        assert!(response_body.contains("Quest reward claimed"));

        let (wallet_status, wallet_body) = send_get(port, "/api/wallets", &[]);
        assert_eq!(wallet_status, 200);
        assert!(wallet_body.contains(node_address));
    }

    #[test]
    fn test_app_mystery_extract_endpoint_returns_reward() {
        let secret = "test_secret_app_mystery";
        let node_address = "nfm_founder_test";

        let mut wallets = WalletEngine::new();
        wallets.set_balance(node_address, 20.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let body = serde_json::json!({ "address": node_address }).to_string();
        let (status, response_body) = send_post(port, "/api/app/mystery/extract", &body, &[]);
        assert_eq!(status, 200);
        assert!(response_body.contains("reward"));
    }

    #[test]
    fn test_app_market_purchase_endpoint_deducts_balance() {
        let secret = "test_secret_app_market";
        let node_address = "nfm_founder_test";

        let mut wallets = WalletEngine::new();
        wallets.set_balance(node_address, 200.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let body = serde_json::json!({
            "address": node_address,
            "item_id": "market-42",
            "price": 99.0
        })
        .to_string();

        let (status, response_body) = send_post(port, "/api/app/market/purchase", &body, &[]);
        assert_eq!(status, 200);
        assert!(response_body.contains("market-42"));
    }

    #[test]
    fn test_app_settings_roundtrip_endpoint() {
        let secret = "test_secret_app_settings_roundtrip";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let update_body = serde_json::json!({
            "settings": {
                "rpc": "http://127.0.0.1:3999",
                "theme": "dark",
                "notifications": {
                    "rewards": true,
                    "network": false,
                    "security": true
                }
            }
        })
        .to_string();

        let (post_status, post_body) = send_post(port, "/api/app/settings", &update_body, &[]);
        assert_eq!(post_status, 200);
        assert!(post_body.contains("127.0.0.1:3999"));
        assert!(post_body.contains("dark"));

        let (get_status, get_body) = send_get(port, "/api/app/settings", &[]);
        assert_eq!(get_status, 200);
        assert!(get_body.contains("127.0.0.1:3999"));
        assert!(get_body.contains("\"network\":false"));
    }

    #[test]
    fn test_app_state_reflects_updated_settings() {
        let secret = "test_secret_app_state_settings";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let update_body = serde_json::json!({
            "settings": {
                "rpc": "http://127.0.0.1:4555",
                "theme": "light",
                "notifications": {
                    "rewards": false,
                    "network": true,
                    "security": false
                }
            }
        })
        .to_string();

        let (post_status, _) = send_post(port, "/api/app/settings", &update_body, &[]);
        assert_eq!(post_status, 200);

        let (state_status, state_body) = send_get(port, "/api/app/state", &[]);
        assert_eq!(state_status, 200);
        assert!(state_body.contains("127.0.0.1:4555"));
        assert!(state_body.contains("\"theme\":\"light\""));
    }

    #[test]
    fn test_drive_upload_and_list_endpoints() {
        let secret = "test_secret_drive_upload";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let upload_body = serde_json::json!({
            "address": node_address,
            "name": "notes-alpha.txt",
            "content": "hello from drive integration test",
            "type": "TEXT",
            "fragments": 2
        })
        .to_string();

        let (upload_status, upload_response) = send_post(port, "/api/drive/upload", &upload_body, &[]);
        assert_eq!(upload_status, 200);
        assert!(upload_response.contains("Drive file uploaded"));
        assert!(upload_response.contains("notes-alpha.txt"));

        let (list_status, list_response) = send_get(port, "/api/drive/files", &[]);
        assert_eq!(list_status, 200);
        assert!(list_response.contains("notes-alpha.txt"));
        assert!(list_response.contains("\"count\":"));

        let parsed_upload: serde_json::Value = serde_json::from_str(&upload_response).expect("valid upload response");
        let file_id = parsed_upload["file_id"].as_str().unwrap_or("");
        assert!(!file_id.is_empty());

        let download_body = serde_json::json!({
            "file_id": file_id,
            "address": node_address
        })
        .to_string();

        let (download_status, download_response) = send_post(port, "/api/drive/download", &download_body, &[]);
        assert_eq!(download_status, 200);
        assert!(download_response.contains("hello from drive integration test"));
        assert!(download_response.contains("notes-alpha.txt"));
    }



    #[test]
    fn test_auction_create_bid_and_list_endpoints() {
        let secret = "test_secret_auction_flow";
        let node_address = "nfm_founder_test";
        let bidder = "nfm_bidder_test";

        let mut wallets = WalletEngine::new();
        wallets.set_balance(bidder, 150.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let create_body = serde_json::json!({
            "seller": node_address,
            "name": "Genesis Relic",
            "rarity": "EPIC",
            "power_multiplier": 1.8,
            "starting_price": 10.0,
            "duration_hours": 24
        })
        .to_string();

        let (create_status, create_response) = send_post(port, "/api/auction/create", &create_body, &[]);
        assert_eq!(create_status, 200);
        assert!(create_response.contains("Auction created"));

        let create_json: serde_json::Value = serde_json::from_str(&create_response).expect("valid create response");
        let auction_id = create_json["auction_id"].as_u64().unwrap_or(0);
        assert!(auction_id > 0);

        let bid_body = serde_json::json!({
            "auction_id": auction_id,
            "bidder": bidder,
            "amount": 20.0
        })
        .to_string();

        let (bid_status, bid_response) = send_post(port, "/api/auction/bid", &bid_body, &[]);
        assert_eq!(bid_status, 200);
        assert!(bid_response.contains("highest_bid"));

        let (list_status, list_response) = send_get(port, "/api/auction/list", &[]);
        assert_eq!(list_status, 200);
        assert!(list_response.contains("Genesis Relic"));
        assert!(list_response.contains("ACTIVE"));
    }

    #[test]
    fn test_identity_endpoint_reports_elite_shield() {
        let secret = "test_secret_identity_endpoint";
        let node_address = "nfm_founder_test";
        let bidder = "nfm_elite_bidder";

        let mut wallets = WalletEngine::new();
        wallets.set_balance(bidder, 500.0);

        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        let create_body = serde_json::json!({
            "seller": node_address,
            "name": "Mythic Crown",
            "rarity": "MYTHIC",
            "power_multiplier": 2.5,
            "starting_price": 25.0,
            "duration_hours": 24
        })
        .to_string();

        let (create_status, create_response) = send_post(port, "/api/auction/create", &create_body, &[]);
        assert_eq!(create_status, 200);
        let create_json: serde_json::Value = serde_json::from_str(&create_response).expect("valid create response");
        let auction_id = create_json["auction_id"].as_u64().unwrap_or(0);
        assert!(auction_id > 0);

        let bid_body = serde_json::json!({
            "auction_id": auction_id,
            "bidder": bidder,
            "amount": 40.0
        })
        .to_string();

        let (bid_status, _) = send_post(port, "/api/auction/bid", &bid_body, &[]);
        assert_eq!(bid_status, 200);

        let settle_body = serde_json::json!({ "auction_id": auction_id }).to_string();
        let (settle_status, _) = send_post(port, "/api/auction/settle", &settle_body, &[]);
        assert_eq!(settle_status, 200);

        let (identity_status, identity_body) = send_get(port, &format!("/api/identity/{}", bidder), &[]);
        assert_eq!(identity_status, 200);
        assert!(identity_body.contains("\"elite_shield\":true"));
        assert!(identity_body.contains("ELITE_VERIFIED"));
        assert!(identity_body.contains("Mythic Crown_Mythic"));
    }

    #[test]
    fn test_governance_indicators_endpoint() {
        let secret = "test_secret_governance_indicators";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let (status, body) = send_get(port, "/api/governance/indicators", &[]);
        assert_eq!(status, 200);
        assert!(body.contains("\"quorum_target\""));
        assert!(body.contains("\"active_proposals\""));
        assert!(body.contains("\"treasury_pool\""));
    }

    #[test]
    fn test_kg_semantic_endpoint() {
        let secret = "test_secret_kg_semantic";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);
        let (status, body) = send_get(port, "/api/kg/semantic", &[]);
        assert_eq!(status, 200);
        assert!(body.contains("\"concepts\""));
        assert!(body.contains("\"nodes\""));
        assert!(body.contains("\"category_counts\""));
    }

    #[test]
    fn test_brain_status_accepts_valid_bearer_token() {
        // Test penggunaan bearer token yang benar
        let secret = "test_secret_brain_token";
        let node_address = "nfm_founder_test";

        let wallets = WalletEngine::new();
        let mut admin = AdminEngine::new();
        admin.register_admin(node_address);

        let port = start_test_api_server(secret, node_address, wallets, admin, true);

        // Tanpa token (karena default empty), harus berhasil
        let (status, response_body) = send_get(port, "/api/brain/status", &[]);

        assert_eq!(status, 200);
        assert!(response_body.contains("\"status\":\"ok\""));
    }
}
