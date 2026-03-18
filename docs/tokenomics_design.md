# NFM Tokenomics & Circular Economy

Mekanisme ekonomi untuk menjaga nilai NFM Gold dan keberlanjutan insentif.

## 1. Dual-Token System
- **NFM GOLD (Max Supply: 100M)**: Token aset utama untuk staking dan governance.
- **NFM CREDIT**: Token utility (Gas) dengan harga stabil untuk operasional AI.

## 2. Recycling Reward Pool
- **Fee Recycling**: Biaya penggunaan AI dikumpulkan kembali ke Reward Pool.
- **Slashing Redistribution**: Potongan stake dari node curang dibagikan ke node yang jujur.
- **Sustainability**: Menjamin reward bagi node operator tersedia selamanya tanpa harus mencetak token baru terus-menerus.

## 3. Mekanisme Staking & Tiering (Deep Dive)

Staking di NFM bukan hanya untuk mendapatkan profit, melainkan fondasi keamanan dan kualitas jaringan.

### A. Cara Kerja Staking
1. **Locking**: Pengguna mengunci sejumlah **NFM Gold** di dalam Smart Contract melalui NFM Super-App. Token ini tidak bisa dipindahkan selama masa staking (misalnya minimal 30 hari).
2. **Identification**: Setelah staking, sistem secara otomatis memberikan "Badge" atau Tier pada **NFM-ID** pengguna. 
3. **Task Allocation**: Orchestrator (DSMO) akan memprioritaskan tugas-tugas penting dan memiliki reward tinggi kepada node dengan Tier Genesis dan Pro.

### B. Peran dalam Keamanan (Anti-Sybil)
- Tanpa staking, penyerang bisa membuat ribuan node palsu dengan biaya murah untuk memanipulasi konsensus (Sybil Attack).
- Dengan staking, penyerang harus memiliki modal besar (NFM Gold) untuk melakukan serangan. Jika mereka ketahuan curang, modal tersebut akan langsung di- **Slash** (dipotong) dan dibagikan ke node lain. Ini membuat biaya serangan jauh lebih mahal daripada potensi keuntungannya.

### C. Tier & Pembagian Reward (Dynamic Ranking)
Reward tidak diberikan murni hanya karena "Online", melainkan berdasarkan **Volume Kerja yang Diselesaikan (Proof of Computation)**:

| Tier | Syarat (Ranking Staker) | Multiplier Reward (Base) | Peran & Kapasitas Tugas |
| :--- | :--- | :--- | :--- |
| **Genesis** | **Top 5%** Teratas | 2.0x | **Server/Desktop High-End**. Fokus: Full Model Hosting. Reward Utama. |
| **Pro** | **Top 20%** Berikutnya | 1.5x | **Desktop/Laptop Mid-End**. Fokus: Parallel Inference. Reward Menengah. |
| **Lite** | Sisa Staker / HP | 1.0x | **Mobile (HP) / Perangkat IOT**. Fokus: Data Sharding & Validation. Reward Pendukung (Kecil). |

### D. Formula Reward: Work-Based (Fair Distribution)
- **FCM (Fair Compute Metric)**: Sistem secara otomatis mendeteksi kapasitas VRAM dan kecepatan Floating Point. Perangkat Mobile (HP) secara teknis tidak akan pernah masuk ke Tier Genesis karena keterbatasan hardware, sehingga alokasi reward HP memang dirancang sebagai "Passive Income" kecil dibanding Server/Desktop yang menjadi tulang punggung (Backbone) jaringan.

**Mengapa menggunakan Persentil?**
- **Adaptif**: Jika di masa depan 100 Gold sudah sangat mahal, pengguna tetap bisa bersaing masuk ke Tier Pro jika jumlah staking mereka masuk dalam 20% teratas secara global.
- **Kompetitif**: Mendorong node operator untuk terus meningkatkan kontribusi mereka guna mempertahankan posisi di tier atas.
- **Scarcity-Proof**: Tidak perlu melakukan hard-fork atau update manual setiap kali harga token naik drastis.

### D. Keuntungan Bagi Ekosistem
- **Mengurangi Inflasi**: Semakin banyak orang yang melakukan staking, semakin sedikit token yang beredar di pasar (Circulating Supply menurun), sehingga harga cenderung stabil atau naik.
- **Incentive Alignment**: Pemegang Gold (Investor) memiliki kepentingan yang sama dengan Node Operator (Pekerja) untuk menjaga kualitas jaringan agar harga token tetap tinggi.

## 4. Keberlanjutan Ekonomi (Stability Strategy)
Untuk memastikan nilai token tetap naik dan menarik minat orang:
- **Founder Buyback Program**: Menggunakan sebagian profit dari *Off-Chain Fiat Payment Gateway* (Xendit) yang dikelola Founder untuk melakukan buyback NFM Gold dari pasar secara berkala. Token hasil buyback dimasukkan kembali ke *Recycling Reward Pool* yang dikelola protokol.
- **Utility-First Focus**: Menjamin bahwa NFM Credit memiliki kegunaan nyata (seperti akses ke model AI premium) yang mendorong permintaan konstan terhadap ekosistem.
