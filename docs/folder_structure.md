# NFM Project Folder Structure

Struktur folder terorganisir untuk seluruh ekosistem NFM v4.3.

```text
NFM/
├── apps/                       # Aplikasi User-Facing
│   ├── node-runner/            # Mesin Node (Desktop/Mobile)
│   ├── super-app/              # App Chat, Wallet, & Marketplace (L1)
│   ├── web-portal/             # Explorer & Dashboard Governance (Web)
│   ├── developer-portal/       # Portal API & Dev Documentation
│   └── cli/                    # Terminal Tool untuk Developer
│
├── core/                       # Infrastruktur Utama & Logika Protokol
│   ├── blockchain/             # Native L1 Chain (DPoS & PoC)
│   ├── ai-engine/              # Shared Model Sharding & Native Brain Logic
│   ├── storage/                # Protokol SDS & NFM Drive
│   └── shared/                 # Kumpulan library (ZKP, PQC, Encryption)
│
├── docs/                       # Arsitektur & Dokumentasi (Markdown)
├── scripts/                    # Script otomatisasi, bootstrap, & deployment
├── README.md                   # Pintu masuk informasi proyek
└── blueprint.txt               # Blueprint teknis utama
```

## Deskripsi Folder

### `apps/`
- **node-runner**: Berisi kode engine untuk validasi transaksi dan sharding.
- **super-app**: Kode frontend mobile (React Native/Flutter).
- **web-portal**: Frontend berbasis web (Next.js) untuk akses explorer.

### `core/`
- **blockchain**: Node client untuk kedaulatan chain NFM.
- **ai-engine**: Implementasi NKM (Neural Knowledge Migration) dan Federated Learning.
- **shared**: Berisi kontrak pintar (smart contracts) dan library kriptografi.
