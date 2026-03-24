#![allow(dead_code)]
use std::collections::HashMap;

/// Centralized node configuration [R-02 FIX]
/// Semua nilai dibaca dari environment variable dengan fallback ke defaults.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Port REST API (default: 3000)
    pub api_port: u16,
    /// Port P2P (default: 9000)
    pub p2p_port: u16,
    /// Mining difficulty (jumlah leading zeros, default: 2)
    pub mining_difficulty: u32,
    /// API secret untuk HMAC auth
    pub api_secret: String,
    /// Rate limit per menit per IP
    pub rate_limit_per_minute: u32,
    /// Max peers P2P
    pub max_peers: usize,
    /// Database path
    pub db_path: String,
    /// Bind address
    pub bind_address: String,
    /// Simulation mode (default: false)
    pub nfm_simulation: bool,
}

impl NodeConfig {
    /// Load config dari environment variables dengan fallback ke defaults
    pub fn from_env() -> Self {
        Self {
            api_port: Self::env_u16("NFM_API_PORT", 3000),
            p2p_port: Self::env_u16("NFM_P2P_PORT", 9000),
            mining_difficulty: Self::env_u32("NFM_DIFFICULTY", 2),
            api_secret: Self::env_string("NFM_API_SECRET", "nfm_dev_secret_v0.5"),
            rate_limit_per_minute: Self::env_u32("NFM_RATE_LIMIT", 60),
            max_peers: Self::env_u16("NFM_MAX_PEERS", 50) as usize,
            db_path: Self::env_string("NFM_DB_PATH", "nfm_main.db"),
            bind_address: Self::env_string("NFM_BIND_ADDR", "0.0.0.0"),
            nfm_simulation: Self::env_bool("NFM_SIMULATION", false),
        }
    }

    /// Print config ke stdout (hide secret)
    pub fn print_summary(&self) {
        println!("[CONFIG] API Port: {}", self.api_port);
        println!("[CONFIG] P2P Port: {}", self.p2p_port);
        println!("[CONFIG] Difficulty: {}", self.mining_difficulty);
        println!("[CONFIG] Rate Limit: {}/min", self.rate_limit_per_minute);
        println!("[CONFIG] Max Peers: {}", self.max_peers);
        println!("[CONFIG] DB Path: {}", self.db_path);
        println!("[CONFIG] Simulation: {}", self.nfm_simulation);
        println!("[CONFIG] API Secret: {}...", &self.api_secret[..4.min(self.api_secret.len())]);
    }

    /// Export config sebagai HashMap (untuk API status)
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("api_port".into(), self.api_port.to_string());
        map.insert("p2p_port".into(), self.p2p_port.to_string());
        map.insert("difficulty".into(), self.mining_difficulty.to_string());
        map.insert("rate_limit".into(), self.rate_limit_per_minute.to_string());
        map.insert("max_peers".into(), self.max_peers.to_string());
        map.insert("db_path".into(), self.db_path.clone());
        map
    }

    // --- Helpers ---

    fn env_string(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16(key: &str, default: u16) -> u16 {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    fn env_u32(key: &str, default: u32) -> u32 {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    fn env_bool(key: &str, default: bool) -> bool {
        std::env::var(key)
            .ok()
            .and_then(|v| v.to_lowercase().parse().ok())
            .unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::from_env();
        assert_eq!(config.api_port, 3000);
        assert_eq!(config.p2p_port, 9000);
        assert_eq!(config.mining_difficulty, 2);
        assert_eq!(config.rate_limit_per_minute, 60);
        assert_eq!(config.max_peers, 50);
    }

    #[test]
    fn test_config_to_map() {
        let config = NodeConfig::from_env();
        let map = config.to_map();
        assert_eq!(map.get("api_port").unwrap(), "3000");
        assert_eq!(map.get("difficulty").unwrap(), "2");
    }
}

