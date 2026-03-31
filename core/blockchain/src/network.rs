#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use crate::block::Block;
use std::net::{TcpListener, TcpStream};
use std::io::{Write, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::{HashMap, HashSet};

/// Batas keamanan P2P [K-02]
const MAX_PEERS: usize = 50;
const CONNECTION_TIMEOUT_SECS: u64 = 5;
/// Jumlah infraksi sebelum auto-ban [S-01]
const BAN_THRESHOLD: u32 = 3;

// ======================================================================
// BAN LIST (P2P Reputation & Auto-Ban) [S-01]
// ======================================================================

/// Alasan ban
#[derive(Debug, Clone)]
pub enum BanReason {
    InvalidBlock,
    SpamMessages,
    MalformedData,
    ManualBan,
}

/// Entri infraksi per peer
#[derive(Debug, Clone)]
pub struct InfractionEntry {
    pub count: u32,
    pub reasons: Vec<String>,
    pub banned: bool,
}

/// Daftar ban P2P untuk mengelola reputasi node
pub struct BanList {
    /// IP -> infraction data
    infractions: HashMap<String, InfractionEntry>,
    /// Set of banned IPs for O(1) lookup
    banned_ips: HashSet<String>,
}

impl BanList {
    pub fn new() -> Self {
        Self {
            infractions: HashMap::new(),
            banned_ips: HashSet::new(),
        }
    }

    /// Catat infraksi untuk sebuah IP. Auto-ban jika melebihi threshold.
    pub fn record_infraction(&mut self, ip: &str, reason: BanReason) -> bool {
        let reason_str = format!("{:?}", reason);
        let entry = self.infractions.entry(ip.to_string()).or_insert(InfractionEntry {
            count: 0,
            reasons: Vec::new(),
            banned: false,
        });

        entry.count += 1;
        entry.reasons.push(reason_str);

        if entry.count >= BAN_THRESHOLD && !entry.banned {
            entry.banned = true;
            self.banned_ips.insert(ip.to_string());
            println!("[BAN] Auto-banned peer {} after {} infractions", ip, entry.count);
            return true; // Baru saja di-ban
        }

        false
    }

    /// Cek apakah IP di-ban
    pub fn is_banned(&self, ip: &str) -> bool {
        self.banned_ips.contains(ip)
    }

    /// Ban manual oleh admin
    pub fn manual_ban(&mut self, ip: &str) {
        self.record_infraction(ip, BanReason::ManualBan);
        let entry = self.infractions.entry(ip.to_string()).or_insert(InfractionEntry {
            count: BAN_THRESHOLD,
            reasons: vec!["ManualBan".to_string()],
            banned: true,
        });
        entry.banned = true;
        self.banned_ips.insert(ip.to_string());
    }

    /// Unban IP
    pub fn unban(&mut self, ip: &str) {
        self.banned_ips.remove(ip);
        if let Some(entry) = self.infractions.get_mut(ip) {
            entry.banned = false;
            entry.count = 0;
            entry.reasons.clear();
        }
    }

    /// Jumlah IP yang di-ban
    pub fn banned_count(&self) -> usize {
        self.banned_ips.len()
    }

    /// Get infraction count untuk IP
    pub fn get_infractions(&self, ip: &str) -> u32 {
        self.infractions.get(ip).map(|e| e.count).unwrap_or(0)
    }
}

/// Pesan yang dikirim antar node
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetMessage {
    /// Node baru bergabung ke jaringan
    Hello { address: String, port: u16 },
    /// Kirim blok baru ke semua peer
    NewBlock(Block),
    /// Minta seluruh chain (sync)
    RequestChain,
    /// Kirim seluruh chain (response sync)
    ChainResponse(Vec<Block>),
    /// Ping (heartbeat)
    Ping,
    /// Pong (heartbeat response)
    Pong,
    /// Exchange daftar alamat peer aktif [Gossip Protocol]
    PeerExchange(Vec<Peer>),
}

/// Peer yang terhubung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub address: String,
    pub port: u16,
}

impl Peer {
    pub fn endpoint(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

/// P2P Node Engine
pub struct P2PNode {
    pub node_address: String,
    pub port: u16,
    pub peers: Arc<Mutex<Vec<Peer>>>,
    pub chain: Arc<Mutex<Vec<Block>>>,
    pub seen_blocks: Arc<Mutex<std::collections::HashSet<String>>>,
}

impl P2PNode {
    pub fn new(node_address: &str, port: u16) -> Self {
        Self {
            node_address: node_address.to_string(),
            port,
            peers: Arc::new(Mutex::new(Vec::new())),
            chain: Arc::new(Mutex::new(Vec::new())),
            seen_blocks: Arc::new(Mutex::new(std::collections::HashSet::new())),
        }
    }

    /// Tambah peer secara manual (dengan batas MAX_PEERS)
    pub fn add_peer(&self, address: &str, port: u16) -> bool {
        let mut peers = self.peers.lock().unwrap();
        if peers.iter().any(|p| p.address == address && p.port == port) {
            return false;
        }
        if peers.len() >= MAX_PEERS {
            println!("[P2P] Peer limit reached ({}), rejecting {}:{}", MAX_PEERS, address, port);
            return false;
        }
        peers.push(Peer { address: address.to_string(), port });
        println!("[P2P] Added peer: {}:{} ({}/{})", address, port, peers.len(), MAX_PEERS);
        true
    }

    fn merge_discovered_peers(&self, discovered: Vec<Peer>) -> usize {
        let mut peers = self.peers.lock().unwrap();
        let mut added = 0;

        for candidate in discovered {
            if peers.len() >= MAX_PEERS {
                break;
            }

            // Hindari menambah diri sendiri
            if candidate.port == self.port
                && (candidate.address == "127.0.0.1"
                    || candidate.address == "localhost"
                    || candidate.address == self.node_address)
            {
                continue;
            }

            if !peers
                .iter()
                .any(|p| p.address == candidate.address && p.port == candidate.port)
            {
                peers.push(candidate);
                added += 1;
            }
        }

        added
    }

    /// Bootstrap gossip discovery dari daftar seed peers.
    pub fn bootstrap_gossip(&self, seeds: &[String]) {
        for seed in seeds {
            let trimmed = seed.trim();
            if trimmed.is_empty() {
                continue;
            }

            let mut parts = trimmed.split(':');
            let address = parts.next().unwrap_or("").trim();
            let port = parts
                .next()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(0);

            if address.is_empty() || port == 0 {
                println!("[P2P] Ignoring invalid seed peer: {}", seed);
                continue;
            }

            let _ = self.add_peer(address, port);
        }

        let peers = self.peers.lock().unwrap().clone();
        for peer in peers {
            let hello = NetMessage::Hello {
                address: self.node_address.clone(),
                port: self.port,
            };
            match Self::send_message(&peer, &hello) {
                Ok(response) => {
                    if let Ok(NetMessage::PeerExchange(received)) = serde_json::from_str::<NetMessage>(&response) {
                        let added = self.merge_discovered_peers(received);
                        if added > 0 {
                            println!("[P2P] Added {} discovered peers from {}", added, peer.endpoint());
                        }
                    }
                }
                Err(e) => {
                    println!("[P2P] Seed handshake failed with {}: {}", peer.endpoint(), e);
                }
            }
        }
    }

    fn is_chain_valid(candidate: &[Block]) -> bool {
        if candidate.is_empty() {
            return false;
        }

        if !candidate[0].is_valid() {
            return false;
        }

        for idx in 1..candidate.len() {
            if Block::validate_block(&candidate[idx], &candidate[idx - 1], 2).is_err() {
                return false;
            }
        }

        true
    }

    /// Ambil chain terpanjang yang valid dari peers yang terhubung.
    pub fn sync_longest_chain(&self) -> Result<usize, String> {
        let peers = self.peers.lock().unwrap().clone();
        if peers.is_empty() {
            return Ok(self.chain.lock().unwrap().len());
        }

        let current_chain = self.chain.lock().unwrap().clone();
        let mut best_chain = current_chain;

        for peer in peers {
            if let Ok(candidate) = self.sync_from_peer(&peer) {
                if candidate.len() > best_chain.len() && Self::is_chain_valid(&candidate) {
                    best_chain = candidate;
                }
            }
        }

        let best_len = best_chain.len();
        let mut chain_lock = self.chain.lock().unwrap();
        if best_len > chain_lock.len() {
            *chain_lock = best_chain;
            println!("[P2P] Chain updated from gossip sync: {} blocks", best_len);
        }

        Ok(chain_lock.len())
    }

    /// Kirim pesan ke satu peer (dengan timeout)
    pub fn send_message(peer: &Peer, message: &NetMessage) -> Result<String, String> {
        let stream = TcpStream::connect_timeout(
            &peer.endpoint().parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
            Duration::from_secs(CONNECTION_TIMEOUT_SECS)
        ).map_err(|e| format!("Connection failed to {}: {}", peer.endpoint(), e))?;

        // Set read/write timeouts [K-02]
        stream.set_read_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT_SECS))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT_SECS))).ok();

        let json = serde_json::to_string(message).unwrap();
        let mut writer = std::io::BufWriter::new(&stream);
        writer.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        writer.write_all(b"\n").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;

        // Baca response
        let mut reader = BufReader::new(&stream);
        let mut response = String::new();
        reader.read_line(&mut response).map_err(|e| e.to_string())?;

        Ok(response.trim().to_string())
    }

    /// Broadcast blok baru ke semua peer
    pub fn broadcast_block(&self, block: &Block) {
        let peers = self.peers.lock().unwrap().clone();
        let msg = NetMessage::NewBlock(block.clone());

        for peer in &peers {
            match Self::send_message(peer, &msg) {
                Ok(_) => println!("[P2P] Block #{} sent to {}", block.index, peer.endpoint()),
                Err(e) => println!("[P2P] Failed to send to {}: {}", peer.endpoint(), e),
            }
        }
    }

    /// Mulai mendengarkan koneksi masuk (blocking, jalankan di thread terpisah)
    pub fn start_listener(&self) {
        let bind_addr = format!("0.0.0.0:{}", self.port);
        let listener = match TcpListener::bind(&bind_addr) {
            Ok(l) => {
                println!("[P2P] Listening on {}", bind_addr);
                l
            },
            Err(e) => {
                println!("[P2P] Failed to bind {}: {}", bind_addr, e);
                return;
            }
        };

        let chain = self.chain.clone();
        let peers = self.peers.clone();
        let seen_blocks = self.seen_blocks.clone();
        let node_port = self.port;

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        // Set timeouts pada koneksi masuk [K-02]
                        stream.set_read_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT_SECS))).ok();
                        stream.set_write_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT_SECS))).ok();

                        let chain_clone = chain.clone();
                        let peers_clone = peers.clone();
                        let seen_clone = seen_blocks.clone();
                        
                        thread::spawn(move || {
                            let rebroadcast_block = Self::handle_connection(stream, chain_clone, peers_clone.clone(), seen_clone, node_port);
                            
                            // GOSSIP PROTOCOL: Re-broadcast jika ada block baru yang divalidasi
                            if let Some(block) = rebroadcast_block {
                                let current_peers = peers_clone.lock().unwrap().clone();
                                let msg = NetMessage::NewBlock(block.clone());
                                for peer in current_peers {
                                    let _ = Self::send_message(&peer, &msg);
                                }
                            }
                        });
                    },
                    Err(e) => println!("[P2P] Connection error: {}", e),
                }
            }
        });
    }

    /// Handle koneksi masuk dari peer
    fn handle_connection(
        stream: TcpStream,
        chain: Arc<Mutex<Vec<Block>>>,
        peers: Arc<Mutex<Vec<Peer>>>,
        seen_blocks: Arc<Mutex<std::collections::HashSet<String>>>,
        node_port: u16,
    ) -> Option<Block> {
        let peer_addr = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();

        if reader.read_line(&mut line).is_err() {
            return None;
        }

        let message: NetMessage = match serde_json::from_str(line.trim()) {
            Ok(m) => m,
            Err(_) => return None,
        };

        let mut rebroadcast = None;

        let response = match message {
            NetMessage::Hello { address, port } => {
                let mut p = peers.lock().unwrap();
                // Kirim daftar peer yang kita tahu ke node baru (Peer Exchange)
                if p.len() < MAX_PEERS {
                    p.push(Peer { address: address.clone(), port });
                    println!("[P2P] New peer joined: {}:{} ({}/{})", address, port, p.len(), MAX_PEERS);
                }
                let peer_list = p.clone();
                serde_json::to_string(&NetMessage::PeerExchange(peer_list)).unwrap_or_else(|_| "WELCOME".to_string())
            },
            NetMessage::PeerExchange(received_peers) => {
                let mut p = peers.lock().unwrap();
                let mut added = 0;
                for rp in received_peers {
                    if p.len() >= MAX_PEERS { break; }
                    // Hindari menambah diri sendiri
                    if rp.port == node_port && (rp.address == "127.0.0.1" || rp.address == "localhost") { continue; }
                    
                    if !p.iter().any(|existing| existing.address == rp.address && existing.port == rp.port) {
                        p.push(rp);
                        added += 1;
                    }
                }
                if added > 0 {
                    println!("[P2P] Added {} new peers from exchange", added);
                }
                "PEER_EXCHANGE_OK".to_string()
            },
            NetMessage::NewBlock(block) => {
                let mut seen = seen_blocks.lock().unwrap();
                if seen.contains(&block.hash) {
                    "ALREADY_SEEN".to_string() // Skip gossip loop
                } else {
                    seen.insert(block.hash.clone());
                    println!("[P2P] Received block #{} from {}", block.index, peer_addr);
                    let mut c = chain.lock().unwrap();
                    
                    // VALIDASI: Hanya terima jika index cocok dan hash valid
                    if let Some(last_block) = c.last() {
                        // Gunakan difficulty 2 (sesuai target kita saat ini)
                        match Block::validate_block(&block, last_block, 2) {
                            Ok(_) => {
                                c.push(block.clone());
                                rebroadcast = Some(block); // Trigger rebroadcast
                                "BLOCK_ACCEPTED".to_string()
                            },
                            Err(e) => {
                                println!("[P2P] Block REJECTED: {}", e);
                                format!("BLOCK_REJECTED: {}", e)
                            }
                        }
                    } else {
                        // Jika chain kosong (hanya terjadi jika tidak ada genesis)
                        c.push(block.clone());
                        rebroadcast = Some(block);
                        "BLOCK_ACCEPTED".to_string()
                    }
                }
            },
            NetMessage::RequestChain => {
                let c = chain.lock().unwrap();
                serde_json::to_string(&NetMessage::ChainResponse(c.clone())).unwrap()
            },
            NetMessage::Ping => "PONG".to_string(),
            _ => "OK".to_string(),
        };

        let mut writer = std::io::BufWriter::new(&stream);
        let _ = writer.write_all(response.as_bytes());
        let _ = writer.write_all(b"\n");
        let _ = writer.flush();

        rebroadcast
    }

    /// Sinkronisasi chain dari peer (ambil chain terpanjang)
    pub fn sync_from_peer(&self, peer: &Peer) -> Result<Vec<Block>, String> {
        let response = Self::send_message(peer, &NetMessage::RequestChain)?;
        let msg: NetMessage = serde_json::from_str(&response).map_err(|e| e.to_string())?;

        match msg {
            NetMessage::ChainResponse(blocks) => {
                println!("[P2P] Received chain of {} blocks from {}", blocks.len(), peer.endpoint());
                Ok(blocks)
            },
            _ => Err("Unexpected response".to_string()),
        }
    }

    /// Jumlah peer yang terhubung
    pub fn peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_management() {
        let node = P2PNode::new("nfm_test_node", 9000);
        node.add_peer("127.0.0.1", 9001);
        node.add_peer("127.0.0.1", 9002);
        assert_eq!(node.peer_count(), 2);
    }

    #[test]
    fn test_add_peer_prevents_duplicate() {
        let node = P2PNode::new("nfm_test_node", 9000);
        assert!(node.add_peer("127.0.0.1", 9001));
        assert!(!node.add_peer("127.0.0.1", 9001));
        assert_eq!(node.peer_count(), 1);
    }

    #[test]
    fn test_message_serialization() {
        let msg = NetMessage::Hello { address: "nfm_node_1".to_string(), port: 9000 };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: NetMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            NetMessage::Hello { address, port } => {
                assert_eq!(address, "nfm_node_1");
                assert_eq!(port, 9000);
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_tcp_listener_and_communication() {
        // Buat node server
        let server = P2PNode::new("nfm_server", 19876);
        server.start_listener();

        // Beri waktu listener aktif
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Kirim Ping dari "client"
        let peer = Peer { address: "127.0.0.1".to_string(), port: 19876 };
        let result = P2PNode::send_message(&peer, &NetMessage::Ping);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "PONG");
    }

    // --- BanList Tests [S-01] ---

    #[test]
    fn test_banlist_auto_ban_after_threshold() {
        let mut banlist = BanList::new();
        let ip = "192.168.1.100";

        // 2 infractions: not banned yet
        banlist.record_infraction(ip, BanReason::InvalidBlock);
        banlist.record_infraction(ip, BanReason::MalformedData);
        assert!(!banlist.is_banned(ip));
        assert_eq!(banlist.get_infractions(ip), 2);

        // 3rd infraction: auto-ban triggered
        let was_banned = banlist.record_infraction(ip, BanReason::SpamMessages);
        assert!(was_banned);
        assert!(banlist.is_banned(ip));
        assert_eq!(banlist.banned_count(), 1);
    }

    #[test]
    fn test_banlist_manual_ban() {
        let mut banlist = BanList::new();
        banlist.manual_ban("10.0.0.5");
        assert!(banlist.is_banned("10.0.0.5"));
    }

    #[test]
    fn test_banlist_unban() {
        let mut banlist = BanList::new();
        banlist.manual_ban("10.0.0.5");
        assert!(banlist.is_banned("10.0.0.5"));

        banlist.unban("10.0.0.5");
        assert!(!banlist.is_banned("10.0.0.5"));
        assert_eq!(banlist.get_infractions("10.0.0.5"), 0);
    }

    #[test]
    fn test_banlist_isolation_per_ip() {
        let mut banlist = BanList::new();
        banlist.manual_ban("bad_peer");
        assert!(banlist.is_banned("bad_peer"));
        assert!(!banlist.is_banned("good_peer"));
    }

    #[test]
    fn test_chain_validation_rejects_broken_previous_hash() {
        let mut b0 = Block::new(0, "genesis".to_string(), "".to_string());
        b0.mine(2);
        let mut b1 = Block::new(1, "b1".to_string(), b0.hash.clone());
        b1.mine(2);
        let mut b2 = Block::new(2, "b2".to_string(), "invalid_prev".to_string());
        b2.mine(2);

        let chain = vec![b0, b1, b2];
        assert!(!P2PNode::is_chain_valid(&chain));
    }
}


