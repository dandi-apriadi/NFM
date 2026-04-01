# Langkah 4: Blockchain Governance MVP Design

**Date**: March 31, 2026  
**Status**: Implementation Complete  
**Branch**: `feature/langkah4-governance-mvp`

## 📋 Overview

Langkah 4 implements three core governance mechanisms for on-chain coordination:

1. **Learning Windows** - Time-bound model training phases with quorum tracking
2. **Intent Voting** - Whitelistable NLC intent voting with reputation-weighted consensus
3. **Slashing** - HR-AIS reputation-based penalties for Byzantine behavior

Integrates deeply with Langkah 3 AI gating (model audit, NLC intent validation, HR-AIS reputation).

---

## 🏗️ Architecture

### System Diagram
```
┌─────────────────────────────────────────────────────────┐
│                   Governance Engine                      │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────────┐  ┌──────────────────────┐          │
│  │ LearningWindow   │  │ IntentVoting         │          │
│  │  Manager         │  │  Engine              │          │
│  ├──────────────────┤  ├──────────────────────┤          │
│  │ - open_window    │  │ - propose_vote       │          │
│  │ - join_window    │  │ - cast_vote          │          │
│  │ - close_window   │  │ - execute_vote       │          │
│  └──────────────────┘  └──────────────────────┘          │
│                                                           │
│  ┌──────────────────────────┐                            │
│  │ SlashingEngine           │                            │
│  ├──────────────────────────┤                            │
│  │ - register_participant   │                            │
│  │ - propose_slash          │                            │
│  │ - execute_slash          │                            │
│  │ - should_eject           │                            │
│  └──────────────────────────┘                            │
│                                                           │
└─────────────────────────────────────────────────────────┘
          ↓                    ↓                    ↓
     ┌────────────────────────────────────────────────┐
     │    GovernanceApiService (REST/gRPC Layer)      │
     │  Ready for Phase 2 Frontend Integration        │
     └────────────────────────────────────────────────┘
```

---

## 🪟 Learning Windows

### Purpose
Coordinate model training phases across distributed participants. Each window is time-bound and can trigger federated aggregation.

### Core Types

```rust
struct LearningWindow {
    id: u32,
    epoch: u64,
    start_block: u64,
    end_block: u64,
    model_version: String,
    participants: Vec<String>,
    is_active: bool,
}
```

### Lifecycle

1. **Open** - Governance creates learning window for new epoch
   ```
   LearningWindowManager::open_window(
       epoch: 1, 
       start_block: 100, 
       end_block: 150,
       model_version: "v1.0.0"
   ) → window_id: 1
   ```

2. **Join** - Participants register for model training
   ```
   LearningWindowManager::join_window(1, "node1") → OK
   LearningWindowManager::join_window(1, "node2") → OK
   ```

3. **Close** - Window closes, triggers aggregation
   ```
   LearningWindowManager::close_window(1) → 2 participants
   ```

### Integration with AI Engine

Learning windows coordinate with Langkah 3's pipeline:
- Window closure signals FedAvg aggregation start
- Participant list feeds quorum calculation
- Model version ensures consistency across training

---

## 🗳️ Intent Voting

### Purpose
Enforce on-chain governance for whitelisted NLC intents. Prevents unauthorized actions through community voting.

### Whitelist (MVP)

Only 3 intents allowed by default:
- `submit_proposal` - Create governance proposals
- `vote` - Participate in voting
- `start_learning_window` - Initiate new training phases

### Core Types

```rust
struct IntentVote {
    proposal_id: u32,
    intent: String,
    votes_for: u64,
    votes_against: u64,
    participants: Vec<String>,
    requires_quorum: bool,
    is_approved: bool,
}
```

### Voting Mechanism

1. **Propose** - Create vote for whitelisted intent
   ```
   IntentVotingEngine::propose_intent_vote(
       intent: "submit_proposal",
       requires_quorum: false
   ) → vote_id: 1
   ```

2. **Cast** - Reputation-weighted voting
   ```
   IntentVotingEngine::cast_intent_vote(
       vote_id: 1,
       voter: "node1",
       approve: true,
       voter_reputation: 50  // From Langkah 3 HR-AIS
   )
   ```
   - Voter weight = HR-AIS reputation score
   - Double-voting prevented
   - No reputation = cannot vote

3. **Execute** - Finalize with quorum check
   ```
   IntentVotingEngine::execute_intent_vote(1)
   // If requires_quorum: need 8+ votes (80% MVP)
   // Result: approved if votes_for > votes_against
   ```

### Quorum Enforcement

- **MVP Threshold**: 8 minimum votes (8/10 = 80%)
- **Bypass**: Superadmin can execute without quorum
- **Dynamic**: Can be adjusted via governance proposal

---

## ⚔️ Slashing

### Purpose
Penalize Byzantine participants (failed validation, malicious behavior) by reducing HR-AIS reputation.

### Core Types

```rust
struct SlashingEvent {
    event_id: u32,
    target: String,
    reason: String,
    hr_ais_reputation_before: u64,
    slash_amount: u64,
    executed: bool,
}
```

### Slashing Flow

1. **Register** - Participant begins with base reputation (100)
   ```
   SlashingEngine::register_participant("node1", 100)
   ```

2. **Propose** - Slash for Byzantine behavior
   ```
   SlashingEngine::propose_slash(
       target: "node1",
       reason: "Failed model audit validation",
       slash_amount: 25
   ) → event_id: 1
   ```
   - Cannot slash more than current reputation
   - Returns error if insufficient reputation

3. **Execute** - Apply penalty
   ```
   SlashingEngine::execute_slash(1)
   // result: 75 // new reputation
   ```
   - Reputation is immutable after execution
   - Prevents double-execution

4. **Eject** - Auto-remove if reputation too low
   ```
   SlashingEngine::should_eject("node1")
   // true if reputation < 10 (MVP threshold)
   ```

### Slashing Triggers (Examples)

| Trigger | Slash Amount | Reason |
|---------|--------------|--------|
| Failed model audit | 20-30 | Checksum mismatch, shard mismatch |
| Non-whitelisted intent | 15-25 | Attempted dangerous action |
| Byzantine aggregation | 30-40 | Poisoned gradient, consensus deviation |
| Repeated failures | 20-50 | Pattern of violations |
| Consensus deviation | 10-20 | Slightest infraction |

---

## 🔗 Integration Points

### With Langkah 3 (AI Engine)

```
┌─────────────────────────┐
│   Langkah 3: AI Engine  │
├─────────────────────────┤
│ - TurboQuant            │ ──┐
│ - Training Loop         │   │
│ - FedAvg Aggregation    │   │
│ - Model Audit Gate      │   │
│ - NLC Intent Validation │   │  Reputation Scores, Audit Results
│ - HR-AIS Reputation     │   │  NLC Intent Approvals
└─────────────────────────┘   │
          ↓                    │
      ┌──────────────────────────┐
      │  Langkah 4: Governance   │
      ├──────────────────────────┤
      │ - Learning Windows       │
      │ - Intent Voting          │ ← Uses Langkah 3 reputation
      │ - Slashing               │ ← Votes on NLC intents
      └──────────────────────────┘
```

### With Langkah 5+ (Scaling)

- Langkah 5 extends slashing to chain-wide consensus
- Langkah 6-7 add dynamic intent whitelist management
- Future phases may implement reputation recovery mechanisms

---

## 📊 Test Coverage

### Learning Windows (4 tests)
- ✅ Window creation with epoch/blocks/version
- ✅ Participant join (prevents duplicates)
- ✅ Window closure returns participant count
- ✅ Active window retrieval

### Intent Voting (5 tests)
- ✅ Whitelist enforcement (3 intents only)
- ✅ Vote proposal creation
- ✅ Non-whitelisted intent rejection
- ✅ Vote casting with reputation weighting
- ✅ Quorum validation before execution

### Slashing (4 tests)
- ✅ Event creation and validation
- ✅ Over-slash prevention (reputation ceiling)
- ✅ Reputation reduction and finalization
- ✅ Auto-eject on low reputation

### Integration (4 tests)
- ✅ Learning windows in governance engine
- ✅ Intent voting in governance engine
- ✅ Slashing in governance engine
- ✅ Full multi-module flow

**Total**: 30 tests (17 new Langkah 4 + 13 existing legacy/API)

---

## 🌐 API Endpoints (Phase 2)

### Learning Windows
```
POST   /governance/learning-windows        → Create window
POST   /governance/learning-windows/{id}/join → Join window
PATCH  /governance/learning-windows/{id}/close → Close window
GET    /governance/learning-windows/{id}   → Get window status
```

### Intent Voting
```
POST   /governance/intent-votes            → Propose vote
POST   /governance/intent-votes/{id}/cast  → Cast vote
PATCH  /governance/intent-votes/{id}/execute → Execute vote
GET    /governance/intent-votes/{id}       → Check result
GET    /governance/whitelist               → Get allowed intents
```

### Slashing
```
POST   /governance/slashing                → Propose slash
PATCH  /governance/slashing/{id}/execute   → Execute slash
GET    /governance/slashing/{id}           → Check event
GET    /governance/participants/{addr}/reputation → Get reputation
```

### Status
```
GET    /governance/status                  → Overall health
```

---

## 🔄 Workflows

### Model Training Coordination

```
1. Governance opens learning window
    └─ LearningWindowManager::open_window(epoch=1, ...)
    
2. Participants join training phase
    └─ LearningWindowManager::join_window(window_id, "node1/2/3...")
    
3. AI Engine runs training + FedAvg aggregation
    └─ Uses participant list for quorum calculation
    
4. Governance closes window
    └─ LearningWindowManager::close_window(window_id)
    └─ Signal FedAvg to finalize aggregation
```

### Intent-Based Governance

```
1. NLC attempts dangerous action (e.g., "delete_all_models")
    └─ Not whitelisted → Intent Voting Engine rejects
    
2. For whitelisted intent:
    └─ ProposeIntentVote("submit_proposal", requires_quorum=true)
    
3. Community votes with reputation weighting
    └─ Node1 (rep=80) votes FOR
    └─ Node2 (rep=20) votes AGAINST
    └─ Total: FOR=80, AGAINST=20
    
4. Execute vote (if quorum met: 8+ votes)
    └─ Result = approved (80 > 20)
    └─ Allow action execution
```

### Penalty & Ejection Flow

```
1. Byzantine behavior detected (model audit fails)
    └─ SlashingEngine::propose_slash("node1", "Failed audit", 30)
    
2. Governance votes to execute slash
    └─ SlashingEngine::execute_slash(event_id)
    └─ Reputation: 100 → 70
    
3. If reputation drops below 10:
    └─ SlashingEngine::should_eject("node1") = true
    └─ Remove node from network
```

---

## 📝 Configuration (MVP)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Intent Whitelist | 3 intents | Expandable via governance |
| Quorum Threshold | 8 votes | 80% of 10 nodes |
| Eject Threshold | < 10 reputation | Can be adjusted |
| Base Reputation | 100 | Initial value for new nodes |
| Initial Slash | 30 | Average penalty |

---

## 🚀 Future Extensions

### Langkah 5+
- [ ] Dynamic intent whitelist management via voting
- [ ] Reputation recovery mechanisms (good behavior)
- [ ] Tiered slash amounts (minor/major/critical)
- [ ] Multi-signature slashing approvals
- [ ] Governance proposal voting (separate from intent voting)
- [ ] Delegation (reputation + voting power transfer)

### Phase 2 Integration
- [ ] REST/gRPC API deployment
- [ ] Dashboard for governance monitoring
- [ ] Real-time voting UI
- [ ] Slashing event alerts
- [ ] Reputation leaderboard

---

## 🔐 Security Considerations

### Double-Voting Prevention
✅ Participants field tracks all voters per vote
✅ Join window prevents duplicate participation

### Reputation Integrity
✅ Slashing amounts checked against current reputation
✅ Executed events immutable (cannot re-execute)
✅ HR-AIS reputation source is single source of truth

### Intent Enforcement
✅ Whitelist hardcoded in MVP (governance expansion in 5+)
✅ Non-whitelisted intents immediately rejected
✅ Empty utterances blocked

### Byzantine Resilience
✅ Quorum prevents minority rule (80% threshold)
✅ Reputation weighting prevents sybil attacks
✅ Slashing deters coordinated attacks

---

## 📦 Deliverables Checklist

- [x] LearningWindowManager (open, join, close)
- [x] IntentVotingEngine (whitelist, propose, vote, execute)
- [x] SlashingEngine (register, propose, execute, eject)
- [x] GovernanceEngine integration (3 modules)
- [x] GovernanceApiService (REST layer)
- [x] 30 comprehensive tests
- [x] Architecture documentation (this file)
- [x] API specifications
- [x] Workflow examples

---

## 📞 Contact & Questions

**Implementer**: @dandi-apriadi  
**Architecture Review**: [Pending Tech Lead Approval]  
**Phase 2 Kickoff**: Post-Langkah 3 merge & approval  

---

**Status**: 🟢 READY FOR GATE REVIEW & MERGE
