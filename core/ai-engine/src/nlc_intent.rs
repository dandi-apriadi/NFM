//! NLC Intent Guard (MVP)
//!
//! Langkah 7 NFM Brain MVP:
//! - Parse natural language command into whitelisted intents
//! - Enforce confidence threshold before chain mapping
//! - Reject unknown/high-risk commands by default

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WhitelistedIntent {
    SubmitProposal,
    Vote,
    StartLearningWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResolution {
    pub accepted: bool,
    pub intent: Option<WhitelistedIntent>,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentGuardConfig {
    pub min_confidence: f32,
}

impl Default for IntentGuardConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.80,
        }
    }
}

pub struct IntentGuard {
    config: IntentGuardConfig,
}

impl IntentGuard {
    pub fn new(config: IntentGuardConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(IntentGuardConfig::default())
    }

    pub fn resolve(&self, utterance: &str) -> IntentResolution {
        let normalized = utterance.trim().to_lowercase();
        if normalized.is_empty() {
            return IntentResolution {
                accepted: false,
                intent: None,
                confidence: 0.0,
                reason: "empty utterance".to_string(),
            };
        }

        let scored = self.score_intents(&normalized);
        if let Some((intent, confidence)) = scored {
            if confidence >= self.config.min_confidence {
                return IntentResolution {
                    accepted: true,
                    intent: Some(intent),
                    confidence,
                    reason: "accepted".to_string(),
                };
            }

            return IntentResolution {
                accepted: false,
                intent: None,
                confidence,
                reason: "confidence below threshold".to_string(),
            };
        }

        IntentResolution {
            accepted: false,
            intent: None,
            confidence: 0.0,
            reason: "intent not whitelisted".to_string(),
        }
    }

    fn score_intents(&self, normalized: &str) -> Option<(WhitelistedIntent, f32)> {
        // Lightweight heuristic scorer for MVP. Replace with classifier in Phase 2.
        let submit_score = if normalized.contains("submit proposal")
            || normalized.contains("propose proposal")
        {
            0.95
        } else {
            keyword_score(normalized, &["submit", "proposal", "propose"])
        };

        let vote_score = if normalized.contains("vote yes")
            || normalized.contains("vote no")
            || normalized.contains("cast vote")
        {
            0.95
        } else {
            keyword_score(normalized, &["vote", "yes", "no", "approve"])
        };

        let start_score = if normalized.contains("start learning window")
            || normalized.contains("begin learning window")
        {
            0.95
        } else {
            keyword_score(normalized, &["start", "learning", "window", "begin"])
        };

        let mut candidates = vec![
            (WhitelistedIntent::SubmitProposal, submit_score),
            (WhitelistedIntent::Vote, vote_score),
            (WhitelistedIntent::StartLearningWindow, start_score),
        ];

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (intent, score) = candidates[0].clone();

        if score <= 0.0 {
            None
        } else {
            Some((intent, score.min(1.0)))
        }
    }
}

fn keyword_score(input: &str, keywords: &[&str]) -> f32 {
    let mut hits = 0u32;
    for kw in keywords {
        if input.contains(kw) {
            hits += 1;
        }
    }
    hits as f32 / keywords.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_proposal_intent_accepted() {
        let guard = IntentGuard::default();
        let res = guard.resolve("please submit proposal for finance curriculum");
        assert!(res.accepted);
        assert_eq!(res.intent, Some(WhitelistedIntent::SubmitProposal));
        assert!(res.confidence >= 0.80);
    }

    #[test]
    fn test_vote_intent_accepted() {
        let guard = IntentGuard::default();
        let res = guard.resolve("vote yes on proposal 12");
        assert!(res.accepted);
        assert_eq!(res.intent, Some(WhitelistedIntent::Vote));
    }

    #[test]
    fn test_start_learning_window_accepted() {
        let guard = IntentGuard::default();
        let res = guard.resolve("start learning window now");
        assert!(res.accepted);
        assert_eq!(res.intent, Some(WhitelistedIntent::StartLearningWindow));
    }

    #[test]
    fn test_non_whitelisted_intent_rejected() {
        let guard = IntentGuard::default();
        let res = guard.resolve("delete all wallets and nuke chain");
        assert!(!res.accepted);
        assert_eq!(res.intent, None);
    }

    #[test]
    fn test_empty_utterance_rejected() {
        let guard = IntentGuard::default();
        let res = guard.resolve("   ");
        assert!(!res.accepted);
        assert!(res.reason.contains("empty"));
    }
}
