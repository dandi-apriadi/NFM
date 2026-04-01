#![allow(dead_code)]

//! Langkah 4 Governance API Module
//! Provides REST/gRPC-ready interfaces for learning windows, intent voting, and slashing

use serde::{Serialize, Deserialize};
use crate::governance::{
    GovernanceEngine,
};

// ======================================================================
// API REQUEST/RESPONSE TYPES
// ======================================================================

/// Create learning window request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLearningWindowRequest {
    pub epoch: u64,
    pub start_block: u64,
    pub end_block: u64,
    pub model_version: String,
}

/// Learning window response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningWindowResponse {
    pub window_id: u32,
    pub epoch: u64,
    pub start_block: u64,
    pub end_block: u64,
    pub model_version: String,
    pub participant_count: usize,
    pub is_active: bool,
}

/// Join learning window request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinLearningWindowRequest {
    pub window_id: u32,
    pub participant: String,
}

/// Propose intent vote request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposeIntentVoteRequest {
    pub intent: String,
    pub requires_quorum: bool,
}

/// Intent vote response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntentVoteResponse {
    pub vote_id: u32,
    pub intent: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub participant_count: usize,
    pub is_approved: Option<bool>, // None if not yet executed
}

/// Cast intent vote request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastIntentVoteRequest {
    pub vote_id: u32,
    pub voter: String,
    pub approve: bool,
    pub voter_reputation: u64,
}

/// Propose slashing request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposeSlashingRequest {
    pub target: String,
    pub reason: String,
    pub slash_amount: u64,
}

/// Slashing event response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlashingEventResponse {
    pub event_id: u32,
    pub target: String,
    pub reason: String,
    pub hr_ais_reputation_before: u64,
    pub slash_amount: u64,
    pub current_reputation: u64,
    pub executed: bool,
    pub should_eject: bool,
}

/// Governance status summary
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GovernanceStatusResponse {
    pub active_learning_windows: usize,
    pub pending_intent_votes: usize,
    pub active_slashing_events: usize,
}

// ======================================================================
// GOVERNANCE API SERVICE
// ======================================================================

/// Governance API Service Layer
pub struct GovernanceApiService {
    governance: GovernanceEngine,
}

impl GovernanceApiService {
    pub fn new() -> Self {
        Self {
            governance: GovernanceEngine::new(),
        }
    }

    // --- Learning Window API ---

    pub fn create_learning_window(
        &mut self,
        req: CreateLearningWindowRequest,
    ) -> Result<LearningWindowResponse, String> {
        let window_id = self.governance.learning_windows.open_window(
            req.epoch,
            req.start_block,
            req.end_block,
            &req.model_version,
        );

        let window = self.governance
            .learning_windows
            .get_active_window(req.epoch)
            .ok_or("Failed to retrieve created window")?;

        Ok(LearningWindowResponse {
            window_id,
            epoch: window.epoch,
            start_block: window.start_block,
            end_block: window.end_block,
            model_version: window.model_version.clone(),
            participant_count: window.participants.len(),
            is_active: window.is_active,
        })
    }

    pub fn join_learning_window(
        &mut self,
        req: JoinLearningWindowRequest,
    ) -> Result<LearningWindowResponse, String> {
        self.governance
            .learning_windows
            .join_window(req.window_id, &req.participant)?;

        // Find the window
        let window = self.governance
            .learning_windows
            .active_windows()
            .into_iter()
            .find(|w| w.id == req.window_id)
            .ok_or("Window not found")?;

        Ok(LearningWindowResponse {
            window_id: window.id,
            epoch: window.epoch,
            start_block: window.start_block,
            end_block: window.end_block,
            model_version: window.model_version.clone(),
            participant_count: window.participants.len(),
            is_active: window.is_active,
        })
    }

    pub fn close_learning_window(
        &mut self,
        window_id: u32,
    ) -> Result<u64, String> {
        self.governance.learning_windows.close_window(window_id)
    }

    // --- Intent Voting API ---

    pub fn propose_intent_vote(
        &mut self,
        req: ProposeIntentVoteRequest,
    ) -> Result<IntentVoteResponse, String> {
        let vote_id = self.governance
            .intent_voting
            .propose_intent_vote(&req.intent, req.requires_quorum)?;

        let summary = self.governance
            .intent_voting
            .vote_summary(vote_id)
            .ok_or("Failed to retrieve created vote")?;

        Ok(IntentVoteResponse {
            vote_id,
            intent: summary.0,
            votes_for: summary.1,
            votes_against: summary.2,
            participant_count: 0,
            is_approved: None,
        })
    }

    pub fn cast_intent_vote(
        &mut self,
        req: CastIntentVoteRequest,
    ) -> Result<IntentVoteResponse, String> {
        self.governance.intent_voting.cast_intent_vote(
            req.vote_id,
            &req.voter,
            req.approve,
            req.voter_reputation,
        )?;

        let summary = self.governance
            .intent_voting
            .vote_summary(req.vote_id)
            .ok_or("Vote not found")?;

        Ok(IntentVoteResponse {
            vote_id: req.vote_id,
            intent: summary.0,
            votes_for: summary.1,
            votes_against: summary.2,
            participant_count: summary.1 as usize + summary.2 as usize,
            is_approved: None,
        })
    }

    pub fn execute_intent_vote(
        &mut self,
        vote_id: u32,
    ) -> Result<IntentVoteResponse, String> {
        let approved = self.governance.intent_voting.execute_intent_vote(vote_id)?;

        let summary = self.governance
            .intent_voting
            .vote_summary(vote_id)
            .ok_or("Vote not found")?;

        Ok(IntentVoteResponse {
            vote_id,
            intent: summary.0,
            votes_for: summary.1,
            votes_against: summary.2,
            participant_count: summary.1 as usize + summary.2 as usize,
            is_approved: Some(approved),
        })
    }

    // --- Slashing API ---

    pub fn propose_slashing(
        &mut self,
        req: ProposeSlashingRequest,
    ) -> Result<SlashingEventResponse, String> {
        self.governance.slashing.register_participant(&req.target, 100); // Default reputation

        let rep_before = self.governance.slashing.get_reputation(&req.target);
        let event_id =
            self.governance
                .slashing
                .propose_slash(&req.target, &req.reason, req.slash_amount)?;

        Ok(SlashingEventResponse {
            event_id,
            target: req.target,
            reason: req.reason,
            hr_ais_reputation_before: rep_before,
            slash_amount: req.slash_amount,
            current_reputation: rep_before,
            executed: false,
            should_eject: false,
        })
    }

    pub fn execute_slashing(
        &mut self,
        event_id: u32,
    ) -> Result<SlashingEventResponse, String> {
        let event = self.governance
            .slashing
            .get_event(event_id)
            .ok_or("Slashing event not found")?
            .clone();

        let new_rep = self.governance.slashing.execute_slash(event_id)?;
        let should_eject = self.governance.slashing.should_eject(&event.target);

        Ok(SlashingEventResponse {
            event_id,
            target: event.target,
            reason: event.reason,
            hr_ais_reputation_before: event.hr_ais_reputation_before,
            slash_amount: event.slash_amount,
            current_reputation: new_rep,
            executed: true,
            should_eject,
        })
    }

    // --- Status & Summary ---

    pub fn governance_status(&self) -> GovernanceStatusResponse {
        GovernanceStatusResponse {
            active_learning_windows: self.governance.learning_windows.active_windows().len(),
            pending_intent_votes: self.governance.intent_voting.get_whitelist().len(),
            active_slashing_events: 0, // Would track pending slashing events
        }
    }
}

// ======================================================================
// INTEGRATION WITH AI ENGINE (Langkah 3)
// ======================================================================

/// Governance integration report (for AI Engine pipeline)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GovernanceIntegrationReport {
    pub learning_windows_active: usize,
    pub intent_votes_approved: usize,
    pub slashing_events_executed: usize,
    pub governance_status: String, // "healthy", "degraded", "critical"
}

impl GovernanceIntegrationReport {
    pub fn from_service(service: &GovernanceApiService) -> Self {
        let status = service.governance_status();
        let health = if status.active_slashing_events > 5 {
            "critical".to_string()
        } else if status.active_slashing_events > 2 {
            "degraded".to_string()
        } else {
            "healthy".to_string()
        };

        Self {
            learning_windows_active: status.active_learning_windows,
            intent_votes_approved: status.pending_intent_votes,
            slashing_events_executed: status.active_slashing_events,
            governance_status: health,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_create_learning_window() {
        let mut service = GovernanceApiService::new();
        let req = CreateLearningWindowRequest {
            epoch: 1,
            start_block: 100,
            end_block: 200,
            model_version: "v1.0.0".to_string(),
        };

        let resp = service.create_learning_window(req).unwrap();
        assert_eq!(resp.epoch, 1);
        assert_eq!(resp.participant_count, 0);
    }

    #[test]
    fn test_api_join_learning_window() {
        let mut service = GovernanceApiService::new();
        let window_req = CreateLearningWindowRequest {
            epoch: 1,
            start_block: 100,
            end_block: 200,
            model_version: "v1.0.0".to_string(),
        };
        let window_resp = service.create_learning_window(window_req).unwrap();

        let join_req = JoinLearningWindowRequest {
            window_id: window_resp.window_id,
            participant: "node1".to_string(),
        };
        let join_resp = service.join_learning_window(join_req).unwrap();

        assert_eq!(join_resp.participant_count, 1);
    }

    #[test]
    fn test_api_propose_intent_vote() {
        let mut service = GovernanceApiService::new();
        let req = ProposeIntentVoteRequest {
            intent: "submit_proposal".to_string(),
            requires_quorum: false,
        };

        let resp = service.propose_intent_vote(req).unwrap();
        assert_eq!(resp.intent, "submit_proposal");
        assert_eq!(resp.is_approved, None);
    }

    #[test]
    fn test_api_cast_and_execute_intent_vote() {
        let mut service = GovernanceApiService::new();
        let propose_req = ProposeIntentVoteRequest {
            intent: "vote".to_string(),
            requires_quorum: false,
        };
        let propose_resp = service.propose_intent_vote(propose_req).unwrap();

        let cast_req = CastIntentVoteRequest {
            vote_id: propose_resp.vote_id,
            voter: "voter1".to_string(),
            approve: true,
            voter_reputation: 80,
        };
        service.cast_intent_vote(cast_req).unwrap();

        let exec_resp = service.execute_intent_vote(propose_resp.vote_id).unwrap();
        assert_eq!(exec_resp.is_approved, Some(true));
    }

    #[test]
    fn test_api_governance_status() {
        let service = GovernanceApiService::new();
        let status = service.governance_status();

        assert_eq!(status.active_learning_windows, 0);
        assert_eq!(status.active_slashing_events, 0);
    }
}
