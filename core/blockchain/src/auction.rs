#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use crate::item::Item;
use crate::reward::EconomyPool;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// Durasi perpanjangan untuk Anti-Sniping (5 menit)
const ANTI_SNIPING_WINDOW: i64 = 300; // detik
const EXTENSION_TIME: i64 = 300;     // detik

// ======================================================================
// AUCTION STATUS
// ======================================================================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AuctionStatus {
    Active,
    Settled,
    Cancelled,
}

// ======================================================================
// ESCROW VAULT
// ======================================================================

/// Brankas escrow: menahan dana bidder selama auction berlangsung
#[derive(Debug, Clone)]
pub struct EscrowVault {
    /// auction_id -> (bidder_address, locked_amount)
    locked_funds: HashMap<u32, (String, f64)>,
}

impl EscrowVault {
    pub fn new() -> Self {
        Self { locked_funds: HashMap::new() }
    }

    /// Lock dana bidder untuk auction tertentu
    pub fn lock(&mut self, auction_id: u32, bidder: &str, amount: f64) {
        self.locked_funds.insert(auction_id, (bidder.to_string(), amount));
    }

    /// Ambil info locked funds untuk auction
    pub fn get_locked(&self, auction_id: u32) -> Option<(String, f64)> {
        self.locked_funds.get(&auction_id).cloned()
    }

    /// Release (hapus) locked funds setelah settle/cancel
    pub fn release(&mut self, auction_id: u32) -> Option<(String, f64)> {
        self.locked_funds.remove(&auction_id)
    }
}

// ======================================================================
// AUCTION (Enhanced with Escrow)
// ======================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auction {
    pub auction_id: u32,
    pub seller: String,
    pub item: Item,
    pub starting_price: f64,
    pub highest_bid: f64,
    pub highest_bidder: Option<String>,
    pub end_time: DateTime<Utc>,
    pub status: AuctionStatus,
}

impl Auction {
    pub fn new(id: u32, seller: &str, item: Item, start_price: f64, duration_hours: i64) -> Self {
        Self {
            auction_id: id,
            seller: seller.to_string(),
            item,
            starting_price: start_price,
            highest_bid: 0.0,
            highest_bidder: None,
            end_time: Utc::now() + Duration::hours(duration_hours),
            status: AuctionStatus::Active,
        }
    }

    /// Melakukan penawaran (bid) DENGAN escrow
    /// Dana bidder dilock, bidder sebelumnya di-refund otomatis
    pub fn place_bid_with_escrow(
        &mut self,
        bidder: &str,
        amount: f64,
        wallets: &mut crate::transfer::WalletEngine,
        escrow: &mut EscrowVault,
    ) -> Result<String, String> {
        if self.status != AuctionStatus::Active || Utc::now() > self.end_time {
            return Err("Auction is not active".to_string());
        }

        if bidder == self.seller {
            return Err("Seller cannot bid on own auction".to_string());
        }

        let min_bid = if self.highest_bid == 0.0 { self.starting_price } else { self.highest_bid + 1.0 };
        if amount < min_bid {
            return Err(format!("Bid must be at least {:.2}", min_bid));
        }

        // Cek saldo bidder baru
        let balance = wallets.get_balance(bidder);
        if balance < amount {
            return Err(format!("Insufficient balance: have {:.2}, need {:.2}", balance, amount));
        }

        // Refund bidder sebelumnya (jika ada)
        if let Some(prev_bidder) = &self.highest_bidder {
            if let Some((prev_addr, prev_amount)) = escrow.release(self.auction_id) {
                wallets.add_balance(&prev_addr, prev_amount);
                println!("[ESCROW] Refunded {:.2} NVC to {} (outbid)", prev_amount, prev_bidder);
            }
        }

        // Lock dana bidder baru
        wallets.deduct_balance(bidder, amount)
            .map_err(|e| format!("Failed to lock funds: {}", e))?;
        escrow.lock(self.auction_id, bidder, amount);

        // Anti-Sniping
        let time_left = (self.end_time - Utc::now()).num_seconds();
        if time_left < ANTI_SNIPING_WINDOW {
            self.end_time = self.end_time + Duration::seconds(EXTENSION_TIME);
            println!("[ANTI-SNIPING] Auction #{} extended by 5 mins.", self.auction_id);
        }

        self.highest_bid = amount;
        self.highest_bidder = Some(bidder.to_string());

        Ok(format!("Bid of {:.2} NVC locked in escrow for {}", amount, bidder))
    }

    /// Melakukan penawaran tanpa escrow (legacy, backward-compatible)
    pub fn place_bid(&mut self, bidder: &str, amount: f64) -> Result<String, String> {
        if self.status != AuctionStatus::Active || Utc::now() > self.end_time {
            return Err("Auction is already closed".to_string());
        }

        let min_bid = if self.highest_bid == 0.0 { self.starting_price } else { self.highest_bid + 1.0 };
        
        if amount < min_bid {
            return Err(format!("Bid must be at least {}", min_bid));
        }

        // Anti-Sniping
        let time_left = (self.end_time - Utc::now()).num_seconds();
        if time_left < ANTI_SNIPING_WINDOW {
            self.end_time = self.end_time + Duration::seconds(EXTENSION_TIME);
            println!("[ANTI-SNIPING] Bid placed in last 5 mins! Auction ID #{} extended by 5 mins.", self.auction_id);
        }

        self.highest_bid = amount;
        self.highest_bidder = Some(bidder.to_string());

        Ok(format!("Bid of {} successful for {}", amount, bidder))
    }

    /// Settle auction DENGAN escrow — transfer dana ke seller & fee pool
    pub fn settle_with_escrow(
        &mut self,
        pool: &mut EconomyPool,
        wallets: &mut crate::transfer::WalletEngine,
        escrow: &mut EscrowVault,
    ) -> Result<(String, String, f64, f64), String> {
        if self.status != AuctionStatus::Active {
            return Err("Auction is not active".to_string());
        }

        let winner = self.highest_bidder.as_ref()
            .ok_or("No bids placed")?
            .clone();

        let final_price = self.highest_bid;

        // Release escrow
        escrow.release(self.auction_id)
            .ok_or("No escrowed funds found")?;

        // Fee split: 5% marketplace, 95% seller
        let marketplace_fee = final_price * 0.05;
        let net_to_seller = final_price - marketplace_fee;

        pool.collect_ai_fee(marketplace_fee);
        wallets.add_balance(&self.seller, net_to_seller);

        self.status = AuctionStatus::Settled;

        Ok((winner, self.seller.clone(), net_to_seller, marketplace_fee))
    }

    /// Settle auction tanpa escrow (legacy)
    pub fn settle(&mut self, pool: &mut EconomyPool) -> Option<(String, String, f64, f64)> {
        if self.status != AuctionStatus::Active || Utc::now() <= self.end_time {
            return None;
        }

        self.status = AuctionStatus::Settled;

        if let Some(winner) = &self.highest_bidder {
            let final_price = self.highest_bid;
            let marketplace_fee = final_price * 0.05;
            let net_to_seller = final_price - marketplace_fee;
            pool.collect_ai_fee(marketplace_fee);
            Some((winner.clone(), self.seller.clone(), net_to_seller, marketplace_fee))
        } else {
            None
        }
    }

    /// Cancel auction — refund escrow ke bidder tertinggi
    pub fn cancel(
        &mut self,
        wallets: &mut crate::transfer::WalletEngine,
        escrow: &mut EscrowVault,
    ) -> Result<String, String> {
        if self.status != AuctionStatus::Active {
            return Err("Auction is not active".to_string());
        }

        // Refund escrowed funds
        if let Some((bidder, amount)) = escrow.release(self.auction_id) {
            wallets.add_balance(&bidder, amount);
            println!("[ESCROW] Cancelled auction #{}: refunded {:.2} NVC to {}", self.auction_id, amount, bidder);
        }

        self.status = AuctionStatus::Cancelled;
        Ok(format!("Auction #{} cancelled", self.auction_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Rarity;
    use crate::transfer::WalletEngine;

    fn test_item() -> Item {
        Item { name: "Test Fragment".to_string(), rarity: Rarity::Epic, power_multiplier: 1.8 }
    }

    // --- Legacy test (must still pass) ---

    #[test]
    fn test_bid_extension_anti_sniping() {
        let mut auction = Auction::new(1, "seller", test_item(), 10.0, 1);
        auction.end_time = Utc::now() + Duration::seconds(10);
        
        let original_end = auction.end_time;
        auction.place_bid("bidder1", 15.0).unwrap();
        
        assert!(auction.end_time > original_end, "End time should be extended");
        assert_eq!(auction.highest_bid, 15.0);
    }

    // --- New Escrow tests ---

    #[test]
    fn test_escrow_locks_bidder_funds() {
        let mut auction = Auction::new(100, "@alice", test_item(), 10.0, 24);
        let mut wallets = WalletEngine::new();
        let mut escrow = EscrowVault::new();

        wallets.set_balance("@bob", 100.0);
        auction.place_bid_with_escrow("@bob", 20.0, &mut wallets, &mut escrow).unwrap();

        // Dana @bob harus dikurangi
        assert_eq!(wallets.get_balance("@bob"), 80.0);
        // Dana harus ada di escrow
        let (bidder, amount) = escrow.get_locked(100).unwrap();
        assert_eq!(bidder, "@bob");
        assert_eq!(amount, 20.0);
    }

    #[test]
    fn test_escrow_refunds_outbid() {
        let mut auction = Auction::new(101, "@alice", test_item(), 10.0, 24);
        let mut wallets = WalletEngine::new();
        let mut escrow = EscrowVault::new();

        wallets.set_balance("@bob", 100.0);
        wallets.set_balance("@charlie", 200.0);

        // Bob bids 20
        auction.place_bid_with_escrow("@bob", 20.0, &mut wallets, &mut escrow).unwrap();
        assert_eq!(wallets.get_balance("@bob"), 80.0);

        // Charlie outbids with 50 — Bob should be refunded
        auction.place_bid_with_escrow("@charlie", 50.0, &mut wallets, &mut escrow).unwrap();
        assert_eq!(wallets.get_balance("@bob"), 100.0); // Refunded!
        assert_eq!(wallets.get_balance("@charlie"), 150.0); // 200 - 50 locked
    }

    #[test]
    fn test_escrow_settle_transfers() {
        let mut auction = Auction::new(102, "@alice", test_item(), 10.0, 24);
        let mut wallets = WalletEngine::new();
        let mut escrow = EscrowVault::new();
        let mut pool = EconomyPool::new();

        wallets.set_balance("@alice", 0.0);
        wallets.set_balance("@bob", 100.0);

        auction.place_bid_with_escrow("@bob", 50.0, &mut wallets, &mut escrow).unwrap();

        // Settle
        let (winner, seller, net, fee) = auction.settle_with_escrow(&mut pool, &mut wallets, &mut escrow).unwrap();
        
        assert_eq!(winner, "@bob");
        assert_eq!(seller, "@alice");
        assert_eq!(fee, 2.5);  // 5% of 50
        assert_eq!(net, 47.5); // 95% of 50
        assert_eq!(wallets.get_balance("@alice"), 47.5); // Seller gets net
        assert_eq!(auction.status, AuctionStatus::Settled);
    }

    #[test]
    fn test_escrow_cancel_refunds() {
        let mut auction = Auction::new(103, "@alice", test_item(), 10.0, 24);
        let mut wallets = WalletEngine::new();
        let mut escrow = EscrowVault::new();

        wallets.set_balance("@bob", 100.0);
        auction.place_bid_with_escrow("@bob", 30.0, &mut wallets, &mut escrow).unwrap();
        assert_eq!(wallets.get_balance("@bob"), 70.0);

        // Cancel — bob gets refund
        auction.cancel(&mut wallets, &mut escrow).unwrap();
        assert_eq!(wallets.get_balance("@bob"), 100.0); // Full refund
        assert_eq!(auction.status, AuctionStatus::Cancelled);
    }

    #[test]
    fn test_seller_cannot_bid_own_auction() {
        let mut auction = Auction::new(104, "@alice", test_item(), 10.0, 24);
        let mut wallets = WalletEngine::new();
        let mut escrow = EscrowVault::new();

        wallets.set_balance("@alice", 100.0);
        let result = auction.place_bid_with_escrow("@alice", 20.0, &mut wallets, &mut escrow);
        assert!(result.is_err());
    }

    #[test]
    fn test_insufficient_balance_rejected() {
        let mut auction = Auction::new(105, "@alice", test_item(), 10.0, 24);
        let mut wallets = WalletEngine::new();
        let mut escrow = EscrowVault::new();

        wallets.set_balance("@bob", 5.0);
        let result = auction.place_bid_with_escrow("@bob", 20.0, &mut wallets, &mut escrow);
        assert!(result.is_err());
        assert_eq!(wallets.get_balance("@bob"), 5.0); // Balance unchanged
    }
}

