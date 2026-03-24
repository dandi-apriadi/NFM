#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Tingkat kelangkaan item di ekosistem NFM
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd)]
pub enum Rarity {
    Common,    // 70%
    Rare,      // 20%
    Epic,      // 7%
    Legendary, // 2.5%
    Mythic,    // 0.5%
}

/// Item digital (NFT-like) dalam ekosistem NFM
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub name: String,
    pub rarity: Rarity,
    pub power_multiplier: f64,
}

// ======================================================================
// VRF MIXER (Enhanced Randomness)
// ======================================================================

/// Menggabungkan multiple entropy sources untuk randomness yang lebih kuat
pub struct VrfMixer;

impl VrfMixer {
    /// Gabungkan beberapa sumber entropy menjadi satu seed deterministik
    pub fn mix(sources: &[&str]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for (i, source) in sources.iter().enumerate() {
            hasher.update(format!("{}:{}:", i, source).as_bytes());
        }
        let result = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result);
        output
    }

    /// Dari hash 32 byte, hasilkan angka 0-999 untuk roll
    pub fn roll_from_hash(hash: &[u8; 32]) -> u32 {
        let seed_val = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        seed_val % 1000
    }

    /// Generate item dari roll value
    pub fn item_from_roll(roll: u32) -> Item {
        let (name, rarity, multiplier) = match roll {
            0..=4   => ("Neural Core Mythic".to_string(), Rarity::Mythic, 5.0),
            5..=29  => ("Fragment Shard Legendary".to_string(), Rarity::Legendary, 2.5),
            30..=99 => ("Data Crystal Epic".to_string(), Rarity::Epic, 1.8),
            100..=299 => ("System Booster Rare".to_string(), Rarity::Rare, 1.3),
            _ => ("Standard Fragment".to_string(), Rarity::Common, 1.0),
        };
        Item { name, rarity, power_multiplier: multiplier }
    }
}

// ======================================================================
// MYSTERY BOX (Legacy + Enhanced)
// ======================================================================

pub struct MysteryBox;

impl MysteryBox {
    /// Biaya satu kotak (NVCoin)
    pub const COST: f64 = 25.0;

    /// Membuka kotak menggunakan single entropy (Legacy, backward-compatible)
    pub fn open(entropy_seed: &str) -> Item {
        let hash = VrfMixer::mix(&[entropy_seed]);
        let roll = VrfMixer::roll_from_hash(&hash);
        VrfMixer::item_from_roll(roll)
    }

    /// Membuka kotak dengan multi-source entropy (Enhanced VRF)
    pub fn open_enhanced(block_hash: &str, user_address: &str, nonce: u64) -> Item {
        let nonce_str = nonce.to_string();
        let hash = VrfMixer::mix(&[block_hash, user_address, &nonce_str]);
        let roll = VrfMixer::roll_from_hash(&hash);
        VrfMixer::item_from_roll(roll)
    }
}

// ======================================================================
// MYSTERY BOX ENGINE (with Pity System)
// ======================================================================

/// Pity threshold: setelah N opening tanpa ≥ Epic, jamin drop Epic
const PITY_THRESHOLD: u32 = 15;

/// Engine yang melacak history per user untuk pity system
pub struct MysteryBoxEngine {
    /// address -> jumlah opening berturut-turut tanpa mendapat ≥ Epic
    pity_counters: HashMap<String, u32>,
    /// address -> total opening
    total_openings: HashMap<String, u32>,
}

impl MysteryBoxEngine {
    pub fn new() -> Self {
        Self {
            pity_counters: HashMap::new(),
            total_openings: HashMap::new(),
        }
    }

    /// Buka mystery box dengan pity system
    pub fn open_with_pity(
        &mut self,
        user_address: &str,
        block_hash: &str,
        nonce: u64,
    ) -> Item {
        let counter = self.pity_counters.entry(user_address.to_string()).or_insert(0);
        let total = self.total_openings.entry(user_address.to_string()).or_insert(0);
        *total += 1;

        // Cek apakah pity terpicu
        if *counter >= PITY_THRESHOLD {
            // Pity triggered! Jamin Epic atau lebih
            *counter = 0;
            println!("[PITY] Guaranteed Epic+ drop for {} after {} openings!", user_address, PITY_THRESHOLD);

            let nonce_str = nonce.to_string();
            let hash = VrfMixer::mix(&[block_hash, user_address, &nonce_str, "pity"]);
            let pity_roll = VrfMixer::roll_from_hash(&hash) % 100; // 0-99

            let (name, rarity, mult) = match pity_roll {
                0..=2   => ("Neural Core Mythic".to_string(), Rarity::Mythic, 5.0),
                3..=14  => ("Fragment Shard Legendary".to_string(), Rarity::Legendary, 2.5),
                _       => ("Data Crystal Epic".to_string(), Rarity::Epic, 1.8),
            };
            return Item { name, rarity, power_multiplier: mult };
        }

        // Normal roll
        let item = MysteryBox::open_enhanced(block_hash, user_address, nonce);

        // Update pity counter
        if item.rarity >= Rarity::Epic {
            *counter = 0; // Reset — user got a good drop
        } else {
            *counter += 1;
        }

        item
    }

    /// Get pity counter saat ini
    pub fn get_pity_counter(&self, user_address: &str) -> u32 {
        *self.pity_counters.get(user_address).unwrap_or(&0)
    }

    /// Get total openings
    pub fn get_total_openings(&self, user_address: &str) -> u32 {
        *self.total_openings.get(user_address).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Legacy tests (must still pass) ---

    #[test]
    fn test_mystery_box_fairness_distribution() {
        let mut mythic_count = 0;
        let mut common_count = 0;
        let iterations = 2000;

        for i in 0..iterations {
            let item = MysteryBox::open(&format!("seed_{}", i));
            match item.rarity {
                Rarity::Mythic => mythic_count += 1,
                Rarity::Common => common_count += 1,
                _ => {}
            }
        }

        println!("Mythics found: {}/{}", mythic_count, iterations);
        assert!(mythic_count > 0, "At least one mythic should be found in 2000 rolls");
        assert!(common_count > mythic_count);
    }

    #[test]
    fn test_determinism() {
        let item1 = MysteryBox::open("fixed_seed_123");
        let item2 = MysteryBox::open("fixed_seed_123");
        assert_eq!(item1.rarity, item2.rarity);
        assert_eq!(item1.name, item2.name);
    }

    // --- New Phase 3 tests ---

    #[test]
    fn test_vrf_mixer_deterministic() {
        let hash1 = VrfMixer::mix(&["block_abc", "user_123", "42"]);
        let hash2 = VrfMixer::mix(&["block_abc", "user_123", "42"]);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_vrf_mixer_different_sources() {
        let hash1 = VrfMixer::mix(&["block_abc", "user_123", "42"]);
        let hash2 = VrfMixer::mix(&["block_abc", "user_456", "42"]);
        assert_ne!(hash1, hash2); // Different user = different result
    }

    #[test]
    fn test_enhanced_open_deterministic() {
        let item1 = MysteryBox::open_enhanced("block_hash_999", "nfm_alice", 7);
        let item2 = MysteryBox::open_enhanced("block_hash_999", "nfm_alice", 7);
        assert_eq!(item1.rarity, item2.rarity);
        assert_eq!(item1.name, item2.name);
    }

    #[test]
    fn test_pity_system_triggers() {
        let mut engine = MysteryBoxEngine::new();
        let user = "nfm_unlucky_user";

        // Force PITY_THRESHOLD openings with Common results
        // We use different seeds each time. Most will be Common.
        // We need to find seeds that produce Common results.
        let mut common_count = 0;
        let mut nonce = 0u64;
        while common_count < PITY_THRESHOLD {
            let item = MysteryBox::open_enhanced("block_pity", user, nonce);
            if item.rarity < Rarity::Epic {
                // Simulate the counter manually (matching engine logic)
                common_count += 1;
            } else {
                common_count = 0; // Would reset in real engine
            }
            nonce += 1;
        }

        // Now use the engine directly: set counter to threshold
        engine.pity_counters.insert(user.to_string(), PITY_THRESHOLD);

        // Next opening should guarantee >= Epic
        let item = engine.open_with_pity(user, "block_pity_final", 999);
        assert!(item.rarity >= Rarity::Epic, "Pity should guarantee Epic+, got {:?}", item.rarity);
        assert_eq!(engine.get_pity_counter(user), 0); // Counter reset
    }

    #[test]
    fn test_pity_counter_resets_on_epic() {
        let mut engine = MysteryBoxEngine::new();
        let user = "nfm_pity_reset_user";

        // Set counter to 10 (below threshold)
        engine.pity_counters.insert(user.to_string(), 10);

        // Find a nonce that gives >= Epic
        for nonce in 0..10000u64 {
            let test_item = MysteryBox::open_enhanced("block_reset", user, nonce);
            if test_item.rarity >= Rarity::Epic {
                // Use this nonce with engine
                engine.pity_counters.insert(user.to_string(), 10);
                engine.open_with_pity(user, "block_reset", nonce);
                assert_eq!(engine.get_pity_counter(user), 0, "Counter should reset on Epic+ drop");
                return;
            }
        }
        // If somehow no Epic in 10000 tries (extremely unlikely), skip
        println!("No Epic found in 10000 tries, test effectively skipped");
    }

    #[test]
    fn test_total_openings_tracked() {
        let mut engine = MysteryBoxEngine::new();
        let user = "nfm_tracker";

        for i in 0..5 {
            engine.open_with_pity(user, "block_track", i);
        }
        assert_eq!(engine.get_total_openings(user), 5);
    }
}

