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

- `GET /api/status`
- `GET /api/blocks`
- `GET /api/mempool`
- `GET /api/wallets`

Jika backend tidak aktif atau endpoint berubah, UI akan menampilkan error koneksi.

## Catatan Progress

- Status saat ini: **MVP aktif**
- Fokus saat ini: stabilisasi dan observabilitas node
- Next: penyempurnaan UX, filtering block/mempool, dan mode konfigurasi endpoint API
