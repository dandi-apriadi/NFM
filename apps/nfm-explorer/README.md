# NFM Explorer (MVP)

NFM Explorer adalah frontend monitoring untuk node lokal NFM blockchain.
Aplikasi ini dibangun dengan React + TypeScript + Vite dan terhubung ke API node Rust di port `3000`.

## Fitur MVP

- Menampilkan status node (`/api/status`)
- Menampilkan daftar blok terbaru (`/api/blocks`)
- Menampilkan antrean mempool (`/api/mempool`)
- Wallet lookup sederhana (`/api/wallets`)
- Auto-refresh data berkala untuk observabilitas real-time

## Prasyarat

- Node.js 18+ (disarankan LTS terbaru)
- npm
- NFM blockchain API server aktif di `http://127.0.0.1:3000`

## Menjalankan Explorer

```bash
npm install
npm run dev
```

Setelah berjalan, buka:

- `http://localhost:5173`

## Build Produksi

```bash
npm run build
npm run preview
```

## Kontrak API Saat Ini

Explorer mengandalkan endpoint berikut dari backend blockchain:

- `GET /api/app/state`
- `GET /api/p2p/status`
- `POST /api/app/wallet/transfer`
- `POST /api/app/governance/proposal`
- `POST /api/app/governance/vote`
- `POST /api/app/quest/claim`
- `POST /api/app/mystery/extract`
- `POST /api/app/market/purchase`
- `GET /api/status`
- `GET /api/blocks`
- `GET /api/mempool`
- `GET /api/wallets`

Jika backend tidak aktif atau endpoint berubah, UI akan menampilkan error koneksi.

## Smoke Test Integrasi FE-BE

Setelah node aktif di port `3000`, jalankan smoke test berikut dari root repo:

```powershell
./scripts/app_actions_smoke.ps1
```

Untuk stress smoke ringan, jalankan beberapa iterasi sekaligus:

```powershell
./scripts/app_actions_smoke.ps1 -Repeat 20
```

Script ini mengetes flow action utama yang dipakai frontend:
- transfer wallet
- create proposal
- vote proposal
- claim quest
- mystery extract
- market purchase

## Catatan Progress

- Status saat ini: **MVP aktif**
- Fokus saat ini: stabilisasi dan observabilitas node
- Next: penyempurnaan UX, filtering block/mempool, dan mode konfigurasi endpoint API
