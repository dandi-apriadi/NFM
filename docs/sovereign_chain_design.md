# NFM Sovereign Chain Design

A self-sovereign L1 Blockchain that runs in tandem with NFM AI computation.

## 1. L1 Architecture
- **Consensus**: DPoS (Delegated Proof of Stake) + PoC (Proof of Computation).
- **State-Bridge**: AI models have direct "Read-only" access to on-chain data.

## 2. NLC (Natural Language to Chain)
- **Intent Classifier**: Translates chat language into blockchain calls.
- **ABI Mapper**: Automatically maps user commands to smart contract functions.

## 3. Professional API Gateway
- **Auth**: HMAC Signature + OAuth2.
- **Billing**: NFM Credit deductions are performed on-chain per inference.

## 4. DAO Governance (Decentralized Decision Making)
Ensuring the project remains operational even without direct Founder intervention in the future:
- **Proposal System**: NFM Gold holders (Stakers) can submit proposals to change network parameters (e.g., adjusting reward multipliers or marketplace fees).
- **Voting Power**: Voting strength is calculated based on the amount and duration of NFM Gold staked.
- **On-Chain Execution**: Approved voting results are automatically executed by the protocol (Hard-coded governance).
- **Founder Veto (Phase 1)**: In the early stages, the Founder holds veto rights to prevent governance attacks until the network attains sufficient maturity.

