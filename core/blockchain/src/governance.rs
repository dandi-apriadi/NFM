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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
// LEARNING WINDOW (Langkah 4)
// ======================================================================

/// Learning Window — time-bound mechanism untuk model training phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningWindow {
    pub id: u32,
    pub epoch: u64,
    pub start_block: u64,
    pub end_block: u64,
    pub model_version: String,
    pub participants: Vec<String>,
    pub is_active: bool,
}

/// Learning Window Manager
#[derive(Debug, Clone)]
pub struct LearningWindowManager {
    windows: Vec<LearningWindow>,
    next_window_id: u32,
}

impl LearningWindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_window_id: 1,
        }
    }

    /// Buka learning window baru untuk epoch
    pub fn open_window(
        &mut self,
        epoch: u64,
        start_block: u64,
        end_block: u64,
        model_version: &str,
    ) -> u32 {
        let id = self.next_window_id;
        self.next_window_id += 1;

        self.windows.push(LearningWindow {
            id,
            epoch,
            start_block,
            end_block,
            model_version: model_version.to_string(),
            participants: Vec::new(),
            is_active: true,
        });

        id
    }

    /// Tambah participant ke learning window
    pub fn join_window(&mut self, window_id: u32, participant: &str) -> Result<String, String> {
        let window = self.windows.iter_mut()
            .find(|w| w.id == window_id && w.is_active)
            .ok_or("Learning window not found or closed")?;

        if window.participants.contains(&participant.to_string()) {
            return Err(format!("{} already joined window #{}", participant, window_id));
        }

        window.participants.push(participant.to_string());
        Ok(format!("{} joined learning window #{}", participant, window_id))
    }

    /// Close learning window (trigger aggregation)
    pub fn close_window(&mut self, window_id: u32) -> Result<u64, String> {
        let window = self.windows.iter_mut()
            .find(|w| w.id == window_id && w.is_active)
            .ok_or("Learning window not found")?;

        window.is_active = false;
        Ok(window.participants.len() as u64)
    }

    /// Get active window for epoch
    pub fn get_active_window(&self, epoch: u64) -> Option<&LearningWindow> {
        self.windows.iter()
            .find(|w| w.epoch == epoch && w.is_active)
    }

    /// Get all active windows
    pub fn active_windows(&self) -> Vec<&LearningWindow> {
        self.windows.iter().filter(|w| w.is_active).collect()
    }
}

// ======================================================================
// INTENT VOTING (Langkah 4)
// ======================================================================

/// Whitelist untuk NLC intents yang boleh di-vote
#[derive(Debug, Clone)]
pub struct IntentWhitelist {
    pub allowed_intents: HashSet<String>,
}

impl IntentWhitelist {
    pub fn new() -> Self {
        let mut allowed = HashSet::new();
        // MVP whitelisted intents (dari Langkah 3)
        allowed.insert("submit_proposal".to_string());
        allowed.insert("vote".to_string());
        allowed.insert("start_learning_window".to_string());

        Self {
            allowed_intents: allowed,
        }
    }

    /// Cek apakah intent whitelisted
    pub fn is_allowed(&self, intent: &str) -> bool {
        self.allowed_intents.contains(intent)
    }

    /// Tambah intent ke whitelist (requires governance approval)
    pub fn add_intent(&mut self, intent: &str) -> Result<(), String> {
        if self.allowed_intents.contains(intent) {
            return Err(format!("Intent '{}' already whitelisted", intent));
        }
        self.allowed_intents.insert(intent.to_string());
        Ok(())
    }

    /// Get all whitelisted intents
    pub fn get_whitelist(&self) -> Vec<&str> {
        self.allowed_intents.iter().map(|s| s.as_str()).collect()
    }
}

/// Intent voting proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentVote {
    pub proposal_id: u32,
    pub intent: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub participants: Vec<String>,
    pub requires_quorum: bool,
    pub is_approved: bool,
}

/// Intent Voting Engine
#[derive(Debug, Clone)]
pub struct IntentVotingEngine {
    votes: Vec<IntentVote>,
    whitelist: IntentWhitelist,
    next_vote_id: u32,
    quorum_threshold: u64, // Minimal votes needed (8/10 = 80%)
}

impl IntentVotingEngine {
    pub fn new() -> Self {
        Self {
            votes: Vec::new(),
            whitelist: IntentWhitelist::new(),
            next_vote_id: 1,
            quorum_threshold: 8, // MVP quorum = 80%
        }
    }

    /// Proposal untuk vote on intent (must be whitelisted)
    pub fn propose_intent_vote(
        &mut self,
        intent: &str,
        requires_quorum: bool,
    ) -> Result<u32, String> {
        // Check whitelist
        if !self.whitelist.is_allowed(intent) {
            return Err(format!(
                "Intent '{}' not whitelisted. Cannot propose vote.",
                intent
            ));
        }

        let id = self.next_vote_id;
        self.next_vote_id += 1;

        self.votes.push(IntentVote {
            proposal_id: id,
            intent: intent.to_string(),
            votes_for: 0,
            votes_against: 0,
            participants: Vec::new(),
            requires_quorum,
            is_approved: false,
        });

        Ok(id)
    }

    /// Cast vote pada intent (weighted by reputation)
    pub fn cast_intent_vote(
        &mut self,
        vote_id: u32,
        voter: &str,
        approve: bool,
        voter_reputation: u64,
    ) -> Result<String, String> {
        if voter_reputation == 0 {
            return Err("No reputation. Cannot vote on intent.".to_string());
        }

        let vote = self.votes.iter_mut()
            .find(|v| v.proposal_id == vote_id)
            .ok_or("Intent vote not found")?;

        if vote.participants.contains(&voter.to_string()) {
            return Err(format!("{} already voted on intent vote #{}", voter, vote_id));
        }

        vote.participants.push(voter.to_string());

        if approve {
            vote.votes_for += voter_reputation;
        } else {
            vote.votes_against += voter_reputation;
        }

        Ok(format!(
            "{} voted {} on intent '{}' with weight {}",
            voter,
            if approve { "FOR" } else { "AGAINST" },
            vote.intent,
            voter_reputation
        ))
    }

    /// Execute intent vote (finalize result & check quorum)
    pub fn execute_intent_vote(&mut self, vote_id: u32) -> Result<bool, String> {
        let vote = self.votes.iter_mut()
            .find(|v| v.proposal_id == vote_id)
            .ok_or("Intent vote not found")?;

        let total_votes = vote.votes_for + vote.votes_against;

        // Check quorum if required (MVP = 8 minimum votes)
        if vote.requires_quorum && total_votes < 8 {
            return Err(format!(
                "Quorum not met. Got {} votes, need {}",
                total_votes, self.quorum_threshold
            ));
        }

        let approved = vote.votes_for > vote.votes_against;
        vote.is_approved = approved;

        Ok(approved)
    }

    /// Get intent whitelist
    pub fn get_whitelist(&self) -> Vec<&str> {
        self.whitelist.get_whitelist()
    }

    /// Add/remove intent dari whitelist (requires governance call)
    pub fn update_whitelist(&mut self, intent: &str) -> Result<(), String> {
        self.whitelist.add_intent(intent)
    }

    /// Get voting summary for intent
    pub fn vote_summary(&self, vote_id: u32) -> Option<(String, u64, u64, bool)> {
        self.votes.iter()
            .find(|v| v.proposal_id == vote_id)
            .map(|v| (
                v.intent.clone(),
                v.votes_for,
                v.votes_against,
                v.is_approved,
            ))
    }
}

// ======================================================================
// SLASHING CONDITIONS (Langkah 4)
// ======================================================================

/// Slashing event untuk reputation penalties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    pub event_id: u32,
    pub target: String,
    pub reason: String,
    pub hr_ais_reputation_before: u64,
    pub slash_amount: u64,
    pub executed: bool,
}

/// Slashing Engine (HR-AIS integrated)
#[derive(Debug, Clone)]
pub struct SlashingEngine {
    events: Vec<SlashingEvent>,
    next_event_id: u32,
    reputation_map: HashMap<String, u64>, // HR-AIS reputation scores
}

impl SlashingEngine {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_event_id: 1,
            reputation_map: HashMap::new(),
        }
    }

    /// Register participant with initial HR-AIS reputation
    pub fn register_participant(&mut self, address: &str, initial_reputation: u64) {
        self.reputation_map.insert(address.to_string(), initial_reputation);
    }

    /// Propose slashing (Byzantine behavior, failed validation, etc)
    pub fn propose_slash(
        &mut self,
        target: &str,
        reason: &str,
        slash_amount: u64,
    ) -> Result<u32, String> {
        let current_rep = self.reputation_map.get(target)
            .copied()
            .ok_or(format!("Participant {} not found", target))?;

        if current_rep < slash_amount {
            return Err(format!(
                "Cannot slash {} by {} (current reputation: {})",
                target, slash_amount, current_rep
            ));
        }

        let event_id = self.next_event_id;
        self.next_event_id += 1;

        self.events.push(SlashingEvent {
            event_id,
            target: target.to_string(),
            reason: reason.to_string(),
            hr_ais_reputation_before: current_rep,
            slash_amount,
            executed: false,
        });

        Ok(event_id)
    }

    /// Execute slashing (finalize penalty)
    pub fn execute_slash(&mut self, event_id: u32) -> Result<u64, String> {
        let event = self.events.iter_mut()
            .find(|e| e.event_id == event_id && !e.executed)
            .ok_or("Slashing event not found or already executed")?;

        let target = event.target.clone();
        let slash_amount = event.slash_amount;

        // Apply slash
        if let Some(rep) = self.reputation_map.get_mut(&target) {
            *rep = rep.saturating_sub(slash_amount);
        }

        event.executed = true;

        Ok(self.reputation_map.get(&target).copied().unwrap_or(0))
    }

    /// Get current HR-AIS reputation
    pub fn get_reputation(&self, address: &str) -> u64 {
        self.reputation_map.get(address).copied().unwrap_or(0)
    }

    /// Get slashing event details
    pub fn get_event(&self, event_id: u32) -> Option<&SlashingEvent> {
        self.events.iter().find(|e| e.event_id == event_id)
    }

    /// Get all slashing events for participant
    pub fn get_participant_slashes(&self, address: &str) -> Vec<&SlashingEvent> {
        self.events.iter()
            .filter(|e| e.target == address)
            .collect()
    }

    /// Check if participant should be ejected (reputation too low)
    pub fn should_eject(&self, address: &str) -> bool {
        let rep = self.get_reputation(address);
        rep < 10 // Ejection threshold = 10 (MVP)
    }
}

// ======================================================================
// GOVERNANCE ENGINE
// ======================================================================

/// Engine Governance (Enhanced with Elite Shield + Langkah 4)
pub struct GovernanceEngine {
    pub proposals: Vec<Proposal>,
    pub reputations: HashMap<String, NodeReputation>,
    pub elite_shield: EliteShield,
    pub learning_windows: LearningWindowManager,        // Langkah 4
    pub intent_voting: IntentVotingEngine,               // Langkah 4
    pub slashing: SlashingEngine,                        // Langkah 4
    pub next_proposal_id: u32,
    pub storage: Option<crate::governance_storage::GovernanceStorage>,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        Self {
            proposals: Vec::new(),
            reputations: HashMap::new(),
            elite_shield: EliteShield::new(),
            learning_windows: LearningWindowManager::new(),  // Langkah 4
            intent_voting: IntentVotingEngine::new(),         // Langkah 4
            slashing: SlashingEngine::new(),                  // Langkah 4
            next_proposal_id: 1,
            storage: None,
        }
    }

    /// PHASE 11: Inisialisasi dari storage (Persistent!)
    pub fn with_storage(storage: crate::governance_storage::GovernanceStorage) -> Self {
        let mut gov = Self::new();
        gov.proposals = storage.load_proposals();
        gov.reputations = storage.load_reputations();
        
        // Update next_proposal_id
        if let Some(max_id) = gov.proposals.iter().map(|p| p.id).max() {
            gov.next_proposal_id = max_id + 1;
        }
        
        gov.storage = Some(storage);
        gov
    }

    /// Daftarkan reputasi awal sebuah node
    pub fn register_node(&mut self, address: &str) {
        let rep = NodeReputation {
            address: address.to_string(),
            reputation_score: 10,
            epochs_participated: 0,
            blocks_mined: 0,
        };
        
        self.reputations.insert(address.to_string(), rep.clone());

        if let Some(ref s) = self.storage {
            let _ = s.save_reputation(&rep);
        }
    }

    /// Tingkatkan reputasi
    pub fn add_reputation(&mut self, address: &str, points: u64) {
        if let Some(rep) = self.reputations.get_mut(address) {
            rep.reputation_score += points;
            rep.epochs_participated += 1;
            
            if let Some(ref s) = self.storage {
                let _ = s.save_reputation(rep);
            }
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

        let proposal = Proposal {
            id,
            title: title.to_string(),
            description: description.to_string(),
            proposer: proposer.to_string(),
            votes_for: 0,
            votes_against: 0,
            voters: Vec::new(),
            is_active: true,
        };

        if let Some(ref s) = self.storage {
            let _ = s.save_proposal(&proposal);
        }

        self.proposals.push(proposal);
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

        if let Some(ref s) = self.storage {
            let _ = s.save_proposal(proposal);
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
        let active_windows = self.learning_windows.active_windows().len();
        let intent_whitelist_size = self.intent_voting.get_whitelist().len();

        serde_json::json!({
            "total_proposals": self.proposals.len(),
            "active_proposals": self.active_proposal_count(),
            "registered_nodes": self.reputations.len(),
            "elite_shields_active": self.elite_shield.shield_count(),
            "learning_windows_active": active_windows,
            "intent_whitelist_size": intent_whitelist_size,
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

    // --- Langkah 4: Learning Window tests ---

    #[test]
    fn test_learning_window_creation() {
        let mut mgr = LearningWindowManager::new();
        let window_id = mgr.open_window(1, 100, 200, "v1.0.0");
        
        assert_eq!(window_id, 1);
        let window = mgr.get_active_window(1).unwrap();
        assert_eq!(window.model_version, "v1.0.0");
        assert_eq!(window.start_block, 100);
        assert_eq!(window.end_block, 200);
    }

    #[test]
    fn test_learning_window_participant_join() {
        let mut mgr = LearningWindowManager::new();
        let window_id = mgr.open_window(1, 100, 200, "v1.0.0");
        
        let result = mgr.join_window(window_id, "node1");
        assert!(result.is_ok());
        
        let window = mgr.get_active_window(1).unwrap();
        assert!(window.participants.contains(&"node1".to_string()));
    }

    #[test]
    fn test_learning_window_prevents_double_join() {
        let mut mgr = LearningWindowManager::new();
        let window_id = mgr.open_window(1, 100, 200, "v1.0.0");
        
        mgr.join_window(window_id, "node1").unwrap();
        let result = mgr.join_window(window_id, "node1");
        assert!(result.is_err(), "Should prevent double join");
    }

    #[test]
    fn test_learning_window_closure() {
        let mut mgr = LearningWindowManager::new();
        let window_id = mgr.open_window(1, 100, 200, "v1.0.0");
        
        mgr.join_window(window_id, "node1").unwrap();
        mgr.join_window(window_id, "node2").unwrap();
        
        let participant_count = mgr.close_window(window_id).unwrap();
        assert_eq!(participant_count, 2);
        assert!(mgr.get_active_window(1).is_none());
    }

    // --- Langkah 4: Intent Voting tests ---

    #[test]
    fn test_intent_whitelist_mvp() {
        let whitelist = IntentWhitelist::new();
        assert!(whitelist.is_allowed("submit_proposal"));
        assert!(whitelist.is_allowed("vote"));
        assert!(whitelist.is_allowed("start_learning_window"));
        assert!(!whitelist.is_allowed("malicious_intent"));
    }

    #[test]
    fn test_intent_vote_proposal_creation() {
        let mut voting = IntentVotingEngine::new();
        let vote_id = voting.propose_intent_vote("submit_proposal", false).unwrap();
        
        assert_eq!(vote_id, 1);
        let summary = voting.vote_summary(vote_id).unwrap();
        assert_eq!(summary.0, "submit_proposal");
    }

    #[test]
    fn test_intent_vote_rejects_non_whitelisted() {
        let mut voting = IntentVotingEngine::new();
        let result = voting.propose_intent_vote("dangerous_intent", false);
        assert!(result.is_err(), "Should reject non-whitelisted intent");
    }

    #[test]
    fn test_intent_vote_casting() {
        let mut voting = IntentVotingEngine::new();
        let vote_id = voting.propose_intent_vote("vote", false).unwrap();
        
        voting.cast_intent_vote(vote_id, "voter1", true, 50).unwrap();
        voting.cast_intent_vote(vote_id, "voter2", false, 30).unwrap();
        
        let summary = voting.vote_summary(vote_id).unwrap();
        assert_eq!(summary.1, 50);  // votes_for
        assert_eq!(summary.2, 30);  // votes_against
    }

    #[test]
    fn test_intent_vote_execution() {
        let mut voting = IntentVotingEngine::new();
        let vote_id = voting.propose_intent_vote("submit_proposal", false).unwrap();
        
        voting.cast_intent_vote(vote_id, "voter1", true, 100).unwrap();
        voting.cast_intent_vote(vote_id, "voter2", false, 30).unwrap();
        
        let result = voting.execute_intent_vote(vote_id).unwrap();
        assert!(result, "Should approve (100 > 30)");
    }

    #[test]
    fn test_intent_vote_quorum_check() {
        let mut voting = IntentVotingEngine::new();
        let vote_id = voting.propose_intent_vote("submit_proposal", true).unwrap();
        
        voting.cast_intent_vote(vote_id, "voter1", true, 5).unwrap();
        
        let result = voting.execute_intent_vote(vote_id);
        assert!(result.is_err(), "Should fail quorum (5 < 8)");
    }

    // --- Langkah 4: Slashing tests ---

    #[test]
    fn test_slashing_event_creation() {
        let mut slashing = SlashingEngine::new();
        slashing.register_participant("node1", 100);
        
        let event_id = slashing.propose_slash("node1", "Byzantine behavior", 30).unwrap();
        assert_eq!(event_id, 1);
        
        let event = slashing.get_event(event_id).unwrap();
        assert_eq!(event.hr_ais_reputation_before, 100);
        assert_eq!(event.slash_amount, 30);
    }

    #[test]
    fn test_slashing_prevents_over_slash() {
        let mut slashing = SlashingEngine::new();
        slashing.register_participant("node1", 50);
        
        let result = slashing.propose_slash("node1", "Test", 100);
        assert!(result.is_err(), "Should not allow slashing more than reputation");
    }

    #[test]
    fn test_slashing_execution() {
        let mut slashing = SlashingEngine::new();
        slashing.register_participant("node1", 100);
        
        let event_id = slashing.propose_slash("node1", "Byzantine behavior", 30).unwrap();
        slashing.execute_slash(event_id).unwrap();
        
        assert_eq!(slashing.get_reputation("node1"), 70);
    }

    #[test]
    fn test_slashing_eject_threshold() {
        let mut slashing = SlashingEngine::new();
        slashing.register_participant("node1", 50);
        
        let event_id = slashing.propose_slash("node1", "Repeated failures", 45).unwrap();
        slashing.execute_slash(event_id).unwrap();
        
        assert!(slashing.should_eject("node1"), "Should eject (reputation = 5 < 10)");
    }

    // --- Langkah 4: Full Governance Integration tests ---

    #[test]
    fn test_governance_with_learning_windows() {
        let mut gov = GovernanceEngine::new();
        
        let window_id = gov.learning_windows.open_window(1, 100, 200, "v1.0.0");
        gov.learning_windows.join_window(window_id, "node1").unwrap();
        gov.learning_windows.join_window(window_id, "node2").unwrap();
        
        assert_eq!(gov.learning_windows.active_windows().len(), 1);
    }

    #[test]
    fn test_governance_with_intent_voting() {
        let mut gov = GovernanceEngine::new();
        
        let vote_id = gov.intent_voting.propose_intent_vote("submit_proposal", false).unwrap();
        gov.intent_voting.cast_intent_vote(vote_id, "voter1", true, 80).unwrap();
        
        let approved = gov.intent_voting.execute_intent_vote(vote_id).unwrap();
        assert!(approved);
    }

    #[test]
    fn test_governance_with_slashing() {
        let mut gov = GovernanceEngine::new();
        
        gov.slashing.register_participant("node1", 100);
        let event_id = gov.slashing.propose_slash("node1", "Test failure", 25).unwrap();
        gov.slashing.execute_slash(event_id).unwrap();
        
        assert_eq!(gov.slashing.get_reputation("node1"), 75);
        assert!(!gov.slashing.should_eject("node1"));
    }

    #[test]
    fn test_governance_integrated_flow() {
        let mut gov = GovernanceEngine::new();
        gov.register_node("participant1");
        
        // Open learning window
        let window_id = gov.learning_windows.open_window(1, 100, 200, "v1.0.0");
        gov.learning_windows.join_window(window_id, "participant1").unwrap();
        
        // Vote on intent
        let vote_id = gov.intent_voting.propose_intent_vote("submit_proposal", false).unwrap();
        gov.intent_voting.cast_intent_vote(vote_id, "participant1", true, 50).unwrap();
        
        // Register for slashing
        gov.slashing.register_participant("participant1", 100);
        
        // Execute vote
        let approved = gov.intent_voting.execute_intent_vote(vote_id).unwrap();
        assert!(approved);
        
        // Close window
        let count = gov.learning_windows.close_window(window_id).unwrap();
        assert_eq!(count, 1);
    }
}

