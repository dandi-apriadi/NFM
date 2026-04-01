use sled::Db;
use crate::governance::{Proposal, NodeReputation};
use std::collections::HashMap;

pub struct GovernanceStorage {
    db: Db,
}

impl GovernanceStorage {
    pub fn open(path: &str) -> Result<Self, String> {
        let db = sled::open(path).map_err(|e| e.to_string())?;
        Ok(Self { db })
    }

    // --- PROPOSALS ---

    pub fn save_proposal(&self, proposal: &Proposal) -> Result<(), String> {
        let key = format!("prop:{}", proposal.id);
        let value = serde_json::to_vec(proposal).map_err(|e| e.to_string())?;
        self.db.insert(key.as_bytes(), value).map_err(|e| e.to_string())?;
        self.db.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_proposals(&self) -> Vec<Proposal> {
        let mut proposals = Vec::new();
        for item in self.db.scan_prefix("prop:") {
            if let Ok((_, value)) = item {
                if let Ok(p) = serde_json::from_slice::<Proposal>(&value) {
                    proposals.push(p);
                }
            }
        }
        proposals.sort_by_key(|p| p.id);
        proposals
    }

    // --- REPUTATIONS ---

    pub fn save_reputation(&self, rep: &NodeReputation) -> Result<(), String> {
        let key = format!("rep:{}", rep.address);
        let value = serde_json::to_vec(rep).map_err(|e| e.to_string())?;
        self.db.insert(key.as_bytes(), value).map_err(|e| e.to_string())?;
        self.db.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_reputations(&self) -> HashMap<String, NodeReputation> {
        let mut reputations = HashMap::new();
        for item in self.db.scan_prefix("rep:") {
            if let Ok((_, value)) = item {
                if let Ok(r) = serde_json::from_slice::<NodeReputation>(&value) {
                    reputations.insert(r.address.clone(), r);
                }
            }
        }
        reputations
    }
}
