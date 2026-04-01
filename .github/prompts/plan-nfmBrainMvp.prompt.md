# Plan: NFM Brain MVP 8-10 Minggu

## Konteks & Metode

Fokus utama adalah membangun MVP NFM Brain yang bisa diuji end-to-end dalam resource terbatas (laptop/devbox), dengan pendekatan **Architecture-First Lean + Stage-Gate MVP**. Scope fase ini mencakup:
- TurboQuant quantization pipeline
- Learning Window governance
- Federated learning 10-node (simulasi lokal)
- Brain audit gateway
- HR-AIS minimal
- NLC intent-to-chain subset

**Timeline**: 8-10 minggu
**Governance Mode**: Founder-assisted sementara (cutover DAO bertahap setelah KPI stabil)
**Compute Constraint**: Terbatas (laptop/devbox, tanpa cluster besar)

---

## Step-by-Step Planning (Belum Eksekusi Kode)

### **Phase 0: Decision Lock & Contract-First Design** (Minggu 1)

**Langkah 1: Decision Lock** (Minggu 1, minggu pertama)
- **Owner**: Lead Architect + AI Engineer + Blockchain PM
- **Deliverables**:
  - [ ] Decision document (G1-G8):
    - G1 Model baseline untuk MVP: Phi-2 atau TinyLlama (8B max)
    - G2 TurboQuant library source: existing di [core/ai-engine/src/](core/ai-engine/src/) atau install external (PolarQuant + QJL)
    - G3 Federated aggregation algorithm: FedAvg untuk MVP
    - G5 DAO governance timeline: Founder-assisted phase 1 sampai KPI stabil, cutover schedule explicit
    - G7 Brain audit criteria: minimum rubric (metadata validation, integrity check, scoring pass/fail)
    - G8 AI hallucination mitigation: disable untuk MVP (manual curation dataset saja), add post-MVP
    - HR-AIS formula: reputation_score = (uptime × 0.4) + (consensus × 0.3) + (audit_pass × 0.2) + (no_slash × 0.1), threshold 80% untuk MVP
    - NLC intent whitelist untuk MVP: submit_proposal, vote, start_learning_window (3 intent saja)
  - [ ] Acceptance criteria per fitur (KPI minimum masing-masing langkah)
  - [ ] Risk register v2: tambahkan AI hallucination, centralization drift, model staleness
  - **Verification**: Stakeholder sign-off, team alignment session

**Langkah 2: Contract-First Design** (Minggu 1, parallel dengan langkah 1)
- **Owner**: Lead Architect + Lead Backend Engineer
- **Deliverables**: 
  - [ ] Data flow diagram: blockchain ↔ federated AI ↔ governance ↔ audit service
  - [ ] State machine Learning Window: proposal → voting → training → aggregation → deployment → audit → marketplace (optional)
  - [ ] API contracts (OpenAPI YAML):
    - Blockchain → AI nodes: event_learning_window_started(proposalID, datasetURL, trainingParams)
    - AI nodes → Blockchain: federated_aggregation_complete(modelHash, merkleRoot, signedByCoordinators)
    - Audit service: audit_model(modelShards, metadata) → auditScore + audit_log
    - HR-AIS: ingest_url(sourceURL, reputationThreshold) → validatedData + integrity_proof
    - NLC: intent_to_tx(utterance) → structuredIntent + confidence → blockchain_tx (kalau confidence > 80%)
  - [ ] Module boundaries dan dependencies (siapa call siapa)
  - **Acuan file**:
    - [core/blockchain/src/governance.rs](core/blockchain/src/governance.rs) — basis governance contract
    - [core/blockchain/src/contract.rs](core/blockchain/src/contract.rs) — contract infrastructure
    - [core/ai-engine/src/lib.rs](core/ai-engine/src/lib.rs) — entry point AI engine
    - [core/ai-engine/src/shredder.rs](core/ai-engine/src/shredder.rs) — model sharding reference
    - [core/shared/src/crypto.rs](core/shared/src/crypto.rs) — kriptografi utility
  - **Verification**: Architecture review dengan tech leads

---

### **Phase 1: Foundation** (Minggu 2-4)

**Langkah 3: TurboQuant + Local Training Skeleton** (Minggu 2-3, depends on Decision Lock)
- **Owner**: AI Engineer (1-2 person)
- **Scope**: MVP quantization pipeline + single-node training loop
- **Deliverables**:
  - [ ] TurboQuant CLI tool:
    - `nfm-core quantize --model <path> --output-bits 4 --output-path <dir>`
    - Integration dengan existing [core/ai-engine/src/](core/ai-engine/src/)
    - Target: dari 8GB → 2GB model (4-bit compression)
  - [ ] Curriculum data loader: hardcoded 1 dataset (e.g., 1K finance Q&A untuk testing)
  - [ ] Single-node training loop:
    - Load data → forward pass → backward pass → gradient computation → weight update
    - Log convergence (loss per epoch)
  - [ ] Test suite:
    - Accuracy preservation after quantization (degradasi < 1%)
    - Training convergence (loss should decrease)
  - [ ] CLI: `nfm-core train --curriculum finance --epochs 5 --bits 4`
  - **Tech Stack**: PyTorch, Rust FFI (jika ada), atau pure Rust ML library
  - **Timeline**: 2-3 minggu
  - **Verification**:
    - Quantization loss < 1%
    - Training loop converges monotonic
    - CLI commands work end-to-end

**Langkah 4: Learning Window Governance MVP** (Minggu 2-4, parallel dengan langkah 3, depends on Decision Lock & Contract Design)
- **Owner**: Blockchain Engineer (2 person)
- **Scope**: Extend existing governance.rs dengan Learning Window lifecycle
- **Deliverables**:
  - [ ] Smart contract: `LearningWindowGovernance` (extend [core/blockchain/src/governance.rs](core/blockchain/src/governance.rs))
    - submitProposal(topic: str, datasetURL: str, duration: u64) → proposalID
    - vote(proposalID, yesNo: bool)
    - startLearningWindow(proposalID) → emits event to off-chain AI nodes
    - Storage: proposals, voting state, learning window history
    - Founder-assisted: submitProposal harus founder-approved dulu sebelum voting (MVP mode)
  - [ ] Event system: blockchain → AI nodes (via gossip atau webhook)
    - Event: LearningWindowStarted { proposalID, datasetURL, trainingParams }
  - [ ] Integration test: proposal lifecycle end-to-end on local testnet
  - [ ] API endpoint: GET /api/learning-window/active → returns current learning window
  - **Timeline**: 3-4 minggu
  - **Verification**:
    - Proposal dapat diceritakan, vote dihitung benar, window dimulai
    - Event terpropagasi ke listener
    - Atomicity: vote tidak bisa bertambah setelah window dimulai

---

### **Phase 2: Distributed Layer** (Minggu 4-7)

**Langkah 5: Federated 10-Node Simulation** (Minggu 4-6, depends on Langkah 3 & 4)
- **Owner**: Distributed Systems Engineer (1-2 person) + AI Engineer
- **Scope**: Multi-node federated training dengan agregasi FedAvg
- **Deliverables**:
  - [ ] Federated aggregator coordinator:
    - Collect gradients dari N nodes (lokal: spawn 10 subprocess atau VM)
    - FedAvg algorithm: weighted average gradients
    - Timebound: max 5 min per round (fallback jika node tidak respond)
  - [ ] Gradient transmission:
    - Serialize gradients → encrypt (optional, untuk MVP bisa skip)
    - Transmit via local socket/TCP
    - Deserialize + validate checksum
  - [ ] Aggregation verification: simple checksum atau signature confirm
  - [ ] Node dropout handling: if N < 8 nodes respond in 5 min, skip round
  - [ ] Test: 10-node simulation, verify convergence matches single-node baseline within 95%
  - [ ] Integration dengan Learning Window event: start aggregation saat event tiba
  - **Repository**: [core/ai-engine/src/](core/ai-engine/src/), new module `federated_aggregation.rs`
  - **Timeline**: 3-4 minggu
  - **Verification**:
    - 10-node konvergen, loss improvement konsisten
    - Dropout handling: jika 2 node mati, agregasi tetap jalan
    - Time-to-aggregation < 30 sec per round

**Langkah 6: Brain Audit Gateway MVP** (Minggu 5-7, parallel dengan Langkah 5, depends on Decision Lock & Langkah 4)
- **Owner**: Security Engineer (1 person) + AI Engineer
- **Scope**: Autonomous audit validation sebelum model dianggap lolos
- **Deliverables**:
  - [ ] Audit contract: `ModelAudit` (extend [core/blockchain/src/contract.rs](core/blockchain/src/contract.rs))
    - auditModel(modelHash: hash, metadata: json, shards: vec) → auditScore + auditLog
    - Audit checks (MVP rubric minimum):
      - Check 1: Metadata validation (model_type, version, size dalam batas)
      - Check 2: Integrity check (merkle root atau hash match input)
      - Check 3: Scoring (hardcoded: pass if all checks OK, fail otherwise)
    - Returns: { passed: bool, score: 0-100, log: string }
  - [ ] Audit executor: sequential shard validation (dapat parallelize nanti)
  - [ ] Integration dengan deployment: model hanya masuk marketplace kalau audit.passed == true
  - [ ] Test: 50+ test models (mix valid + corrupted), detect 100% corrupted
  - [ ] API: POST /api/audit/submit, GET /api/audit/{modelID}
  - **Timeline**: 3 minggu
  - **Verification**:
    - 100% valid models lolos audit
    - 100% corrupted models ditolak audit
    - Audit runtime < 1 min per model

---

### **Phase 3: Governance & Integration Layer** (Minggu 6-9)

**Langkah 7: HR-AIS Minimal + NLC Snippet** (Minggu 6-8, parallel, depends on Langkah 4 & 6)
- **Owner**: Data Engineer (1 person) + ML Engineer (1 person)
- **Scope**: Reputation filtering + intent-to-chain basic
- **Deliverables**:
  - **HR-AIS (High-Reputation Autonomous Internet Search) Minimal**:
    - [ ] Reputation contract (extend governance.rs):
      - Calculation: reputation_score = (uptime_pct × 0.4) + (consensus_pct × 0.3) + (audit_pass × 0.2) + (no_slash × 0.1)
      - Threshold: rep > 80% to ingest
    - [ ] Data ingestion allowlist:
      - MVP: hardcoded 2-3 whitelist URLs (e.g., Hugging Face docs, arxiv subset)
      - Fetch data → format check → duplicate filter → store
    - [ ] Integrity proof (simple hash proof untuk MVP)
    - [ ] Integration: LearningWindow request HR-AIS → filter high-rep nodes → ingest → return validated dataset
    - [ ] Test: ingest 3 sources, verify only high-rep nodes participate, data format valid
    - **Timeline**: 2-3 minggu
  - **NLC Intent-to-Chain (Natural Language Commands)**:
    - [ ] Intent classifier (tiny model):
      - Train on 500 hardcoded examples (submit_proposal, vote, start_learning_window intent)
      - Confidence threshold: > 80%
    - [ ] ABI mapper:
      - intent → blockchain function: e.g., "I propose AI training" → submitProposal(topic="AI training", dataset=..., duration=...)
    - [ ] Safety guard:
      - Whitelist 3 intents saja untuk MVP (submit, vote, start)
      - Blacklist everything else (reject unknown intents)
    - [ ] Integration: Super-app chat → NLC → blockchain tx (kalau confidence > 80% dan dalam whitelist)
    - [ ] Test: 100 sample utterances, intent accuracy > 90%, false activation 0
    - **Timeline**: 2-3 minggu
  - **Verification**:
    - HR-AIS: hanya high-rep nodes ingest, data valid
    - NLC: intent whitelist 3 saja, accuracy > 90%, no false positives

**Langkah 8: E2E Hardening & Demo Scenario** (Minggu 8-9, depends on Langkah 5, 6, 7)
- **Owner**: QA Lead (1-2 person) + All engineers (review)
- **Scope**: Full lifecycle testing + reliability tuning
- **Deliverables**:
  - [ ] Integration test suite:
    - **Test 1**: Proposal submission via NLC (utterance → intent → blockchain) OK
    - **Test 2**: Voting (30 sec window, 5+ nodes vote, tally correct)
    - **Test 3**: Learning window start (event triggered, 10 nodes start federated training)
    - **Test 4**: Federated training 3 rounds (weights aggregate, loss decreases)
    - **Test 5**: Model submission to audit (model produced by training → audit contract → pass if OK)
    - **Test 6**: HR-AIS filters: low-rep node tries ingest → rejected, high-rep node ingest → OK
    - **Test 7**: Node dropout (kill 2/10 nodes mid-training) → agregasi tetap jalan
  - [ ] Performance benchmarks:
    - Quantization speed: model 8GB → 2GB < 1 min
    - Training throughput: samples/sec baseline
    - Aggregation latency: collect + FedAvg + broadcast < 30 sec
    - Audit time: < 1 min per model
    - NLC latency: utterance → blockchain tx < 5 sec
  - [ ] Reliability tuning:
    - Timeout handling (node tidak respond → skip)
    - Error logging (di mana error terjadi, apa causnya)
    - Recovery (dapat resume federated training jika network hiccup)
  - [ ] Demo scenario script:
    - 1. Founder submit proposal via UI: "Train on recent finance data"
    - 2. Mock 10 nodes vote "yes"
    - 3. Learning window starts
    - 4. Federated training runs 3 rounds
    - 5. Coordinator aggregates, produces final model
    - 6. Model sent to audit → passes → ready for marketplace
    - 7. Document outcome (loss trajectory, timing, success rate)
  - **Timeline**: 2 minggu
  - **Verification**:
    - All 7 tests pass 10/10 runs
    - Performance targets met
    - Demo scenario reproducible, results documented

**Langkah 9: Gate Review & Go/No-Go Decision** (Minggu 9-10, depends on Langkah 8)
- **Owner**: Product Manager + Lead Architect + Stakeholders
- **Scope**: MVP completeness assessment + decision untuk fase selanjutnya
- **Deliverables**:
  - [ ] KPI assessment:
    | KPI | Target | Actual | Status |
    |---|---|---|---|
    | Quantization loss | < 1% | _% | ✅/❌ |
    | Governance proposal lifecycle | < 1 hour | _ sec | ✅/❌ |
    | Federated convergence | within 95% baseline | _% | ✅/❌ |
    | 10-node dropout resilience | 2 nodes fail, agregasi OK | _/10 runs | ✅/❌ |
    | Audit gate detection | 100% corrupted models rejected | _% | ✅/❌ |
    | HR-AIS reputation filtering | 0 low-rep ingests | _/100 tests | ✅/❌ |
    | NLC accuracy | > 90% on whitelist intents | _% | ✅/❌ |
    | E2E scenario success rate | 100% | _% | ✅/❌ |
  - [ ] Risk residual analysis:
    - [Defer to Phase 2] Model poisoning detection (not in MVP, flagged for future)
    - [Defer to Phase 2] Full DAO governance (founder veto still active)
    - [Defer to Phase 2] Multi-domain curriculum (only finance domain MVP)
    - [Defer to Phase 2] 50+ node production mesh (10-node simulation for now)
  - [ ] Go decision: **IF all KPI green → Go to Phase 2 (scale-up)**, ELSE **fallback**:
    - Fallback option A: Fix critical modul (remain on MVP for 2 more weeks)
    - Fallback option B: Simplify scope (remove NLC atau HR-AIS advanced features)
    - Fallback option C: Pivot to semi-centralized mode (single-node training + governance only, defer federated learning)
  - [ ] Deliverables:
    - KPI report (Excel atau MD table)
    - Risk register updated
    - Go/No-Go decision log + rationale
  - **Timeline**: 1-2 minggu
  - **Verification**: Stakeholder approval on decision

---

## Relevant Files & Acuan

### **Dokumentasi Utama**
| File | Penggunaan | Langkah |
|---|---|---|
| [docs/native_brain_and_learning.md](docs/native_brain_and_learning.md) | Learning window protocol, federated lifecycle | L1, L4, L5 |
| [docs/nfm_brain_curriculum.md](docs/nfm_brain_curriculum.md) | 6 knowledge domains, audit rubric | L1, L6 |
| [docs/ai_model_deployment.md](docs/ai_model_deployment.md) | Model uplink, sharding, watermark | L1, L6, L9 |
| [docs/sovereign_chain_design.md](docs/sovereign_chain_design.md) | DAO governance, NLC architecture | L1, L2, L7 |
| [docs/security_audit.md](docs/security_audit.md) | Reputation, slashing, elite shield | L1, L7 |
| [docs/risk_and_mitigation.md](docs/risk_and_mitigation.md) | Risk register, operational mitigations | L1, L9 |
| [docs/implementation_roadmap.md](docs/implementation_roadmap.md) | Phase checklist, status tracking | All |
| [blueprint.txt](blueprint.txt) | Engineering standards, token economics | Code review |

### **Kode Existing**
| Path | Modul | Fungsi | Acuan untuk |
|---|---|---|---|
| [core/blockchain/src/governance.rs](core/blockchain/src/governance.rs) | DAO governance contracts | Extend L4 governance | L2, L4 |
| [core/blockchain/src/contract.rs](core/blockchain/src/contract.rs) | Smart contract base | Extend L6 audit | L2, L6 |
| [core/ai-engine/src/lib.rs](core/ai-engine/src/lib.rs) | AI engine entry point | Integration hub | L3, L4, L5 |
| [core/ai-engine/src/shredder.rs](core/ai-engine/src/shredder.rs) | Model sharding | Gradient serialization (L5) | L3, L5 |
| [core/shared/src/crypto.rs](core/shared/src/crypto.rs) | Crypto utilities | Encryption, proof (L5, L6) | L5, L6, L7 |
| [apps/nfm-explorer/src/](apps/nfm-explorer/src/) | React/Vite UI | Learning window UI | L4, L9 |
| [apps/node-runner/](apps/node-runner/) | Node launcher | Hardware detection, training mode | L5 |

---

## Verification & Gates

### **Per-Langkah Acceptance Criteria**

**L1 Decision Lock**
- [ ] Decision document signed off by stakeholders
- [ ] No ambiguity in G1-G8
- [ ] Team alignment (architects, leads, PMs agree)

**L2 Contract Design**
- [ ] Data flow diagram approved by team
- [ ] API contracts consistent across modules
- [ ] No missing integration points

**L3 Quantization**
- [ ] Quantization loss < 1%
- [ ] Training convergence verified
- [ ] CLI commands functional

**L4 Governance MVP**
- [ ] Proposal lifecycle atomic
- [ ] Event propagation works
- [ ] Integration test passes on local testnet

**L5 Federated 10-Node**
- [ ] Convergence within 95% baseline
- [ ] Dropout resilience: 2 nodes fail, success still achieved
- [ ] Aggregation time < 30 sec per round

**L6 Audit Gateway**
- [ ] 100% valid models pass
- [ ] 100% corrupted models fail
- [ ] Audit < 1 min

**L7 HR-AIS + NLC**
- [ ] HR-AIS: only high-rep nodes ingest (verified via test)
- [ ] NLC: accuracy > 90%, false positive 0
- [ ] Integration with governance + audit works

**L8 E2E Hardening**
- [ ] All 7 integration tests pass 10/10 runs
- [ ] Performance benchmarks met
- [ ] Demo scenario reproducible

**L9 Gate Review**
- [ ] KPI report completed with all metrics
- [ ] Go/No-Go decision documented
- [ ] Risk residual registered for Phase 2

---

## Timeline Summary

| Phase | Minggu | Owner | Deliverable | Gate |
|---|---|---|---|---|
| **Phase 0** | 1 | Arch + Tech leads | Decision lock + Contract design | Stakeholder sign-off |
| **Phase 1** | 2-4 | AI + Blockchain engineers | Quantization + Governance MVP | Governance proposal lifecycle works |
| **Phase 2** | 4-7 | Distributed sys + Security | Federated 10-node + Audit | 10-node convergence OK, audit detects poison |
| **Phase 3** | 6-9 | Data + ML + QA | HR-AIS + NLC + E2E | Demo scenario completes |
| **Gate** | 9-10 | PM + Architect | KPI report + decision | Go/No-Go approved |

Total: **10 minggu** (dari minggu 1-10)

---

## Critical Success Factors

1. **Decision Lock (Minggu 1)** — Jangan mulai koding sebelum G1-G8 locked. Scope creep di akhir akan buang waktu.
2. **Model Kecil (MVP tidak harus state-of-art)** — Phi-2 atau TinyLlama OK, bukan harus Llama-70B. Ini untuk test concept, bukan production inference.
3. **Parallelization** — L3, L4, L5, L6, L7 bisa jalan bersamaan di minggu 2-7 (teams terpisah).
4. **Simple ≠ Bad** — Audit MVP boleh hardcoded rubric, HR-AIS boleh whitelist URL, NLC boleh 3 intent saja. Nanti kompleksitas ditambah Phase 2.
5. **Early Demo (Minggu 8-9)** — Demo scenario end-to-end penting untuk confidence stakeholder dan tim.
6. **Risk Register Living Document** — Update risk minggu-minggu terakhir sesuai findings real dari koding.

---

## Next Steps (Setelah Planning Approved)

1. Mulai Langkah 1 (Decision Lock): Designate decision-makers, set 1-minggu deadline untuk decision document + contract design.
2. Spin-up teams per langkah: assign owners, setup repos/branches, prepare development environments.
3. Establish CI/CD pipeline untuk automated testing (test suite harus siap minggu 2).
4. Weekly syncs: 30 min standup, gate review setiap akhir langkah.
5. Post-gate retrospective: document learnings, update risk register untuk Phase 2.
