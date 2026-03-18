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
## 4. DAO Governance (Decentralized Decision Making)
Untuk memastikan project tetap berjalan meskipun tanpa intervensi Founder secara langsung di masa depan:
- **Proposal System**: Pemegang NFM Gold (Staker) bisa mengajukan proposal untuk mengubah parameter jaringan (misal: penyesuaian multiplier reward atau fee marketplace).
- **Voting Power**: Kekuatan suara dihitung berdasarkan jumlah dan durasi staking NFM Gold.
- **On-Chain Execution**: Hasil voting yang disetujui akan otomatis dieksekusi oleh protokol (Hard-coded governance).
- **Founder Veto (Phase 1)**: Di fase awal, Founder memiliki hak veto untuk mencegah serangan tata kelola (governance attack) hingga jaringan cukup dewasa.
