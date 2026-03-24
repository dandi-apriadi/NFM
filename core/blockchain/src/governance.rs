#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// Proposal untuk voting on-chain
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub voters: Vec<String>,
    pub is_active: bool,
}

/// Sistem Reputasi Node
#[derive(Debug, Clone)]
pub struct NodeReputation {
    pub address: String,
    pub reputation_score: u64,
    pub epochs_participated: u64,
    pub blocks_mined: u64,
}

// ======================================================================
// ELITE SHIELD (Mythic Holder Protection)
// ======================================================================

/// Elite Shield — perlindungan khusus bagi pemegang item Mythic
#[derive(Debug, Clone)]
pub struct EliteShield {
    /// Addresses yang memiliki shield aktif
    shielded: HashSet<String>,
    /// Super admins yang bisa bypass shield (untuk situasi darurat)
    super_admins: HashSet<String>,
}

impl EliteShield {
    pub fn new() -> Self {
        Self {
            shielded: HashSet::new(),
            super_admins: HashSet::new(),
        }
    }

    /// Daftarkan super admin (bisa bypass shield)
    pub fn register_super_admin(&mut self, address: &str) {
        self.super_admins.insert(address.to_string());
    }

    /// Aktifkan shield (dipanggil saat user memiliki item Mythic aktif)
    pub fn activate_shield(&mut self, address: &str) -> Result<String, String> {
        if self.shielded.contains(address) {
            return Err(format!("{} already has Elite Shield", address));
        }
        self.shielded.insert(address.to_string());
        Ok(format!("Elite Shield activated for {}", address))
    }

    /// Nonaktifkan shield (misalnya Mythic item expired)
    pub fn deactivate_shield(&mut self, address: &str) {
        self.shielded.remove(address);
    }

    /// Cek apakah address dilindungi
    pub fn is_protected(&self, address: &str) -> bool {
        self.shielded.contains(address)
    }

    /// Cek apakah aksi freeze diizinkan terhadap target
    /// Shield melindungi dari freeze KECUALI oleh super admin
    pub fn can_freeze(&self, admin: &str, target: &str) -> Result<(), String> {
        if self.is_protected(target) && !self.super_admins.contains(admin) {
            Err(format!(
                "Cannot freeze {}: protected by Elite Shield. Only super admins can override.",
                target
            ))
        } else {
            Ok(())
        }
    }

    /// Cek apakah slashing diizinkan (shield melindungi sepenuhnya)
    pub fn can_slash(&self, target: &str) -> Result<(), String> {
        if self.is_protected(target) {
            Err(format!("Cannot slash {}: protected by Elite Shield", target))
        } else {
            Ok(())
        }
    }

    /// Jumlah shield aktif
    pub fn shield_count(&self) -> usize {
        self.shielded.len()
    }
}

// ======================================================================
// GOVERNANCE ENGINE
// ======================================================================

/// Engine Governance (Enhanced with Elite Shield)
pub struct GovernanceEngine {
    pub proposals: Vec<Proposal>,
    pub reputations: HashMap<String, NodeReputation>,
    pub elite_shield: EliteShield,
    next_proposal_id: u32,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        Self {
            proposals: Vec::new(),
            reputations: HashMap::new(),
            elite_shield: EliteShield::new(),
            next_proposal_id: 1,
        }
    }

    /// Daftarkan reputasi awal sebuah node
    pub fn register_node(&mut self, address: &str) {
        self.reputations.insert(address.to_string(), NodeReputation {
            address: address.to_string(),
            reputation_score: 10,
            epochs_participated: 0,
            blocks_mined: 0,
        });
    }

    /// Tingkatkan reputasi
    pub fn add_reputation(&mut self, address: &str, points: u64) {
        if let Some(rep) = self.reputations.get_mut(address) {
            rep.reputation_score += points;
            rep.epochs_participated += 1;
        }
    }

    /// Get reputation score
    pub fn get_reputation(&self, address: &str) -> u64 {
        self.reputations.get(address).map(|r| r.reputation_score).unwrap_or(0)
    }

    /// Buat proposal baru
    pub fn create_proposal(&mut self, proposer: &str, title: &str, description: &str) -> u32 {
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;

        self.proposals.push(Proposal {
            id,
            title: title.to_string(),
            description: description.to_string(),
            proposer: proposer.to_string(),
            votes_for: 0,
            votes_against: 0,
            voters: Vec::new(),
            is_active: true,
        });

        id
    }

    /// Vote pada proposal (weighted by reputation)
    pub fn vote(&mut self, proposal_id: u32, voter: &str, approve: bool) -> Result<String, String> {
        let rep_score = self.reputations.get(voter)
            .map(|r| r.reputation_score)
            .unwrap_or(0);

        if rep_score == 0 {
            return Err("No reputation. Cannot vote.".to_string());
        }

        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id && p.is_active)
            .ok_or("Proposal not found or closed")?;

        if proposal.voters.contains(&voter.to_string()) {
            return Err("Already voted on this proposal".to_string());
        }

        proposal.voters.push(voter.to_string());

        if approve {
            proposal.votes_for += rep_score;
        } else {
            proposal.votes_against += rep_score;
        }

        Ok(format!("{} voted {} with weight {} on proposal #{}", 
            voter, if approve { "FOR" } else { "AGAINST" }, rep_score, proposal_id))
    }

    /// Periksa hasil voting
    pub fn check_result(&self, proposal_id: u32) -> Option<(bool, u64, u64)> {
        self.proposals.iter()
            .find(|p| p.id == proposal_id)
            .map(|p| (p.votes_for > p.votes_against, p.votes_for, p.votes_against))
    }

    /// Get active proposals count
    pub fn active_proposal_count(&self) -> usize {
        self.proposals.iter().filter(|p| p.is_active).count()
    }

    /// Get governance summary (untuk dashboard)
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({
            "total_proposals": self.proposals.len(),
            "active_proposals": self.active_proposal_count(),
            "registered_nodes": self.reputations.len(),
            "elite_shields_active": self.elite_shield.shield_count(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Legacy tests ---

    #[test]
    fn test_weighted_voting() {
        let mut gov = GovernanceEngine::new();
        gov.register_node("nfm_founder");
        gov.register_node("nfm_user_2");

        gov.add_reputation("nfm_founder", 90);
        gov.add_reputation("nfm_user_2", 10);

        let pid = gov.create_proposal("nfm_founder", "Reduce Fee", "Change marketplace fee to 3%");

        gov.vote(pid, "nfm_founder", true).unwrap();
        gov.vote(pid, "nfm_user_2", false).unwrap();

        let (passed, votes_for, votes_against) = gov.check_result(pid).unwrap();
        assert!(passed, "Proposal should pass (100 > 20)");
        assert_eq!(votes_for, 100);
        assert_eq!(votes_against, 20);
    }

    #[test]
    fn test_double_vote_prevention() {
        let mut gov = GovernanceEngine::new();
        gov.register_node("nfm_voter");
        let pid = gov.create_proposal("nfm_voter", "Test", "Test proposal");

        gov.vote(pid, "nfm_voter", true).unwrap();
        let result = gov.vote(pid, "nfm_voter", true);
        assert!(result.is_err(), "Should not allow double voting");
    }

    #[test]
    fn test_no_reputation_cannot_vote() {
        let mut gov = GovernanceEngine::new();
        let pid = gov.create_proposal("someone", "Test", "Test");
        let result = gov.vote(pid, "nfm_unknown", true);
        assert!(result.is_err(), "Unregistered node should not vote");
    }

    // --- Elite Shield tests ---

    #[test]
    fn test_elite_shield_protects_from_freeze() {
        let mut shield = EliteShield::new();
        shield.activate_shield("nfm_mythic_holder").unwrap();

        // Normal admin cannot freeze shielded user
        assert!(shield.can_freeze("nfm_regular_admin", "nfm_mythic_holder").is_err());

        // Unshielded user can still be frozen
        assert!(shield.can_freeze("nfm_regular_admin", "nfm_normal_user").is_ok());
    }

    #[test]
    fn test_super_admin_bypasses_shield() {
        let mut shield = EliteShield::new();
        shield.register_super_admin("nfm_super");
        shield.activate_shield("nfm_mythic_holder").unwrap();

        // Super admin CAN freeze shielded user
        assert!(shield.can_freeze("nfm_super", "nfm_mythic_holder").is_ok());
    }

    #[test]
    fn test_shield_prevents_slashing() {
        let mut shield = EliteShield::new();
        shield.activate_shield("nfm_protected").unwrap();

        assert!(shield.can_slash("nfm_protected").is_err());
        assert!(shield.can_slash("nfm_unprotected").is_ok());
    }

    #[test]
    fn test_shield_deactivation() {
        let mut shield = EliteShield::new();
        shield.activate_shield("nfm_temp").unwrap();
        assert!(shield.is_protected("nfm_temp"));

        shield.deactivate_shield("nfm_temp");
        assert!(!shield.is_protected("nfm_temp"));
    }
}

