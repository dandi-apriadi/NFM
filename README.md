# NFM (Neural Fragment Mesh) - Project Documentation
Welcome to the NFM documentation repository. The `blueprint.txt` file in the root folder provides a high-level technical summary, while operational details and in-depth architecture are distributed across several specialized documents within the `docs/` folder.

## Document Guide

| File | Description |
| :--- | :--- |
| [blueprint.txt](./blueprint.txt) | Technical Summary v2.3 (Skeleton Overview). |
| [`docs/security_audit.md`](./docs/security_audit.md) | Data protection details (SDS, Bio-ZKP, PQC). |
| [`docs/tokenomics_design.md`](./docs/tokenomics_design.md) | Circular economy model, Recycling Pool, and Staking. |
| [`docs/sovereign_chain_design.md`](./docs/sovereign_chain_design.md) | L1 Blockchain Architecture and AI-to-Chain Bridge (NLC). |
| [`docs/app_suite_definition.md`](./docs/app_suite_definition.md) | Definition of the 5 main Apps in the NFM ecosystem. |
| [`docs/ai_model_deployment.md`](./docs/ai_model_deployment.md) | Mechanisms for uploading and sharding new AI models. |
| [`docs/native_brain_and_learning.md`](./docs/native_brain_and_learning.md) | NFM Native Brain concept and Collective Learning (Sentient). |
| [`docs/gamification_and_quests.md`](./docs/gamification_and_quests.md) | Quest system, Booster Items, and Social Syndicates. |
| [`docs/nfm_brain_curriculum.md`](./docs/nfm_brain_curriculum.md) | Foundational knowledge (Curriculum) for intelligent NFM Brain management. |
| [`docs/implementation_roadmap.md`](./docs/implementation_roadmap.md) | **Master To-Do List** for all features across the 5 NFM apps. |
| [`docs/folder_structure.md`](./docs/folder_structure.md) | Explanation of the NFM project **Folder Structure**. |
| [`docs/reward_simulation.md`](./docs/reward_simulation.md) | **Reward Pool Depression** simulation and Refill mechanisms. |

## Current Implementation Snapshot (2026-03-24)
- **Blockchain Core**: Stabilization hardening batch completed for transfer safety, consensus sorting safety, and startup DB recovery fallback.
- **Security Controls**: `/api/transfer/secure` now follows protected endpoint policy (HMAC check + admin transaction gate + POST rate-limit behavior).
- **Verification Status**: `87/87` tests passed in `core/blockchain` (including integration tests for secure transfer edge-cases).
- **Ecosystem Progress**: Node Runner and Block Explorer MVP are active; `super-app`, `web-portal`, `developer-portal`, and `cli` are still pending implementation.

## Near-Term Priorities
1. **Network Layer**: Implement P2P gossip propagation and dynamic peer discovery.
2. **App Suite Delivery**: Start phased implementation for `super-app`, `web-portal`, `developer-portal`, and `cli`.
3. **Security Continuation**: Continue hardening and add broader integration coverage beyond transfer flow.

For detailed progress and phase-level status, refer to [`docs/implementation_roadmap.md`](./docs/implementation_roadmap.md).

---
*Created by Antigravity AI for NFM Project Founder.*

