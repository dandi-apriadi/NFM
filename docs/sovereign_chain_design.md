# NFM Sovereign Chain Design

Blockchain L1 sendiri yang berjalan beriringan dengan komputasi AI NFM.

## 1. Arsitektur L1
- **Consensus**: DPoS (Delegated Proof of Stake) + PoC (Proof of Computation).
- **State-Bridge**: Model AI memiliki hak akses "Read-only" langsung ke data on-chain.

## 2. NLC (Natural Language to Chain)
- **Intent Classifier**: Menerjemahkan bahasa chat menjadi blockchain call.
- **ABI Mapper**: Menghubungkan perintah user ke fungsi smart contract secara otomatis.

## 3. Professional API Gateway
- **Auth**: HMAC Signature + OAuth2.
- **Billing**: Pemotongan biaya NFM Credit dilakukan secara on-chain per inferensi.
