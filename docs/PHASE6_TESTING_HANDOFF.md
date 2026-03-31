# Phase 6 Testing Handoff (for QA Agent)

## Scope
Dokumen ini untuk agent tester menjalankan verifikasi fitur Phase 6 terbaru:
- P2P telemetry endpoint
- Runtime P2P operator endpoints
- Dashboard/NodeRunner live observability
- Multi-node gossip bootstrap/sync behavior

## New/Updated Backend Endpoints
- `GET /api/p2p/status`
- `GET /api/p2p/seeds`
- `POST /api/p2p/seeds`
- `POST /api/p2p/bootstrap`
- `POST /api/p2p/sync`

## Quick Start
1. Start node:
```powershell
cd core/blockchain
cargo run --release
```
2. Start explorer:
```powershell
cd apps/nfm-explorer
npm run dev
```

### Optional automated E2E smoke (recommended)
From repo root:
```powershell
.\scripts\integration_e2e_smoke.ps1
```
Expected:
- Node starts successfully
- Health-check endpoints (`/api/status`, `/api/p2p/status`, `/api/app/state`) return `200`
- App action smoke flow passes
- Node process is stopped automatically at end

### CI smoke gate
- Workflow: `.github/workflows/integration-smoke.yml`
- Trigger: `pull_request`, push to `main/master`, or manual `workflow_dispatch`
- Gate behavior: run `scripts/integration_e2e_smoke.ps1`; PR/build fails if health-check or smoke test fails
- Artifacts: workflow uploads `artifacts/integration-smoke/summary.json` and `summary.txt` for post-failure diagnosis

## API Test Cases

### TC-API-01: Read P2P status
Request:
```http
GET /api/p2p/status
```
Expected:
- HTTP `200`
- JSON fields exist:
  - `gossip_enabled`
  - `listening_port`
  - `peer_count`
  - `healthy_peers`
  - `unhealthy_peers`
  - `known_peers`
  - `peer_health`
    - each peer should include `endpoint`, `healthy`, `latency_ms`, `score`, `quality`, `error`
  - `seed_count`
  - `ban_count`
  - `banned_peers`
  - `reconnect_attempts`
  - `reconnect_backoff_secs`
  - `next_reconnect_unix`
  - `last_reconnect_unix`
  - `last_sync_unix`
  - `chain_blocks`
  - `status`

### TC-API-02: Set seed peers
Request:
```http
POST /api/p2p/seeds
Content-Type: application/json

{
  "seeds": ["127.0.0.1:9100", "127.0.0.1:9200"]
}
```
Expected:
- HTTP `200`
- Response `status=success`
- `count=2`

### TC-API-03: Read seeds after update
Request:
```http
GET /api/p2p/seeds
```
Expected:
- HTTP `200`
- `count` sesuai data sebelumnya

### TC-API-04: Trigger bootstrap
Request:
```http
POST /api/p2p/bootstrap
```
Expected:
- HTTP `202`
- `status=accepted`
- `action=bootstrap`

### TC-API-05: Ban a peer endpoint
Request:
```http
POST /api/p2p/ban
Content-Type: application/json

{
  "endpoint": "127.0.0.1:9000"
}
```
Expected:
- HTTP `202`
- `status=accepted`
- `action=ban`

### TC-API-06: Unban a peer endpoint
Request:
```http
POST /api/p2p/unban
Content-Type: application/json

{
  "endpoint": "127.0.0.1:9000"
}
```
Expected:
- HTTP `202`
- `status=accepted`
- `action=unban`

### TC-API-06B: Bulk ban endpoints
Request:
```http
POST /api/p2p/ban/bulk
Content-Type: application/json

{
  "endpoints": ["127.0.0.1:9000", "127.0.0.1:9001"]
}
```
Expected:
- HTTP `202`
- `status=accepted`
- `action=ban_bulk`
- `accepted_count` tersedia

### TC-API-06C: Bulk unban endpoints
Request:
```http
POST /api/p2p/unban/bulk
Content-Type: application/json

{
  "endpoints": ["127.0.0.1:9000", "127.0.0.1:9001"]
}
```
Expected:
- HTTP `202`
- `status=accepted`
- `action=unban_bulk`
- `accepted_count` tersedia

### TC-API-06D: Bulk ban mixed payload (valid+invalid+duplicate)
Request:
```http
POST /api/p2p/ban/bulk
Content-Type: application/json

{
  "endpoints": ["127.0.0.1:9000", "127.0.0.1:9000", "invalid", " 127.0.0.1:9001 ", ""]
}
```
Expected:
- HTTP `202`
- `requested_count` hanya menghitung endpoint valid unik
- `accepted_count` <= `requested_count`

### TC-API-06E: Bulk ban via CSV payload
Request:
```http
POST /api/p2p/ban/bulk
Content-Type: application/json

{
  "endpoints_csv": "127.0.0.1:9000, invalid, 127.0.0.1:9001, 127.0.0.1:9001"
}
```
Expected:
- HTTP `202`
- endpoint valid unik saja yang diproses
- `accepted_count` sesuai jumlah endpoint valid yang belum diban

### TC-API-06F: Bulk ban reject when all endpoints invalid
Request:
```http
POST /api/p2p/ban/bulk
Content-Type: application/json

{
  "endpoints": ["invalid", "", "no_port"]
}
```
Expected:
- HTTP `400`
- error `Missing or invalid field: endpoints`

### TC-API-06G: Bulk endpoints when runtime command channel unavailable
Request flow:
1. Simulasikan runtime command processor tidak aktif/terputus.
2. Kirim `POST /api/p2p/ban/bulk` dengan endpoint valid.
3. Kirim `POST /api/p2p/unban/bulk` untuk endpoint yang sama.
Expected:
- Endpoint tetap merespons HTTP `202` (`action=ban_bulk` / `action=unban_bulk`).
- `accepted_count` tetap konsisten terhadap perubahan banlist.
- `GET /api/p2p/banlist` mencerminkan state terbaru sesuai request terakhir.

### TC-API-07: Trigger sync
Request:
```http
POST /api/p2p/sync
```
Expected:
- HTTP `202`
- `status=accepted`
- `action=sync`

## UI Test Cases (Explorer)

### TC-UI-01: Dashboard peer observability
Page: `/`
Expected:
- Tile `Connected Peers` mengikuti `peer_count` dari `/api/p2p/status`
- Tile `Connected Peers` menampilkan ringkasan `At-risk` ratio dari kualitas peer (`degraded/poor/critical`)
- Section `Node Connectivity` menampilkan `known_peers`
- Section `Node Connectivity` menampilkan quality badge per peer (`excellent/good/degraded/poor/critical`) jika `peer_health` tersedia
- Tombol `Sort` mengubah urutan peer berdasarkan score (high/low)
- Tombol `Filter` bisa switch `All` -> `Risk Only` -> `Banned`
- Tersedia toggle `Pause Refresh` / `Resume Refresh` di Dashboard operator controls
- Badge header Dashboard menampilkan status `LIVE` saat refresh aktif dan `PAUSED` saat refresh dijeda
- Preferensi `Sort` dan `Filter` tetap sama setelah reload page (localStorage)
- Tiap peer memiliki tombol `Ban` yang trigger command ban tanpa pindah ke NodeRunner
- Ada panel `Banned Peers` dengan tombol `Unban` per endpoint untuk rollback cepat dari Dashboard
- Tombol `Ban` disabled jika peer sudah ada di `banned_peers`
- Saat request ban/unban berjalan, tombol action terkait disabled (hindari double click)
- Status sukses/gagal muncul sebagai toast non-blocking (bukan modal alert)
- Tombol batch `Ban All Risk` mengeksekusi ban endpoint kualitas `degraded/poor/critical` yang belum diban
- Sebelum batch ban dijalankan, UI menampilkan `Dry Run Preview` endpoint target (termasuk jumlah total kandidat)
- Batch ban menggunakan threshold score operator (default 40, rentang 0-100)
- Tersedia preset threshold cepat `10/25/40/60` untuk mempercepat tuning
- Tombol `Export Risk List` menyalin daftar endpoint risk ke clipboard
- Tombol `Import Ban List` menerima daftar endpoint multi-line/comma dan mengeksekusi bulk ban endpoint valid
- Batch action Dashboard memanggil endpoint bulk (`/api/p2p/ban/bulk`, `/api/p2p/unban/bulk`) bukan loop request single endpoint
- Tombol `Undo Last Batch` mengeksekusi inverse bulk action (ban <-> unban) berdasarkan snapshot batch terakhir yang sukses
- Panel `Operator Activity` mencatat aksi ban/unban/batch/export terbaru
- Panel `Operator Activity` menyediakan `Export Log` (download txt) dan `Clear Log`
- Batch action (`Ban All Risk` / `Unban All`) menerima reason opsional yang tercatat di `Operator Activity`
- Panel `Operator Activity` menampilkan ringkasan `Last pause duration` dan hint shortcut keyboard `P` (pause) / `R` (resume)
- Tombol batch `Unban All` menghapus seluruh endpoint dari `banned_peers`
- Footer menampilkan `P2P status`, `Port`, `Last sync`

### TC-UI-02: NodeRunner peer topology live
Page: `/node`
Expected:
- Metric `CONNECTED PEERS` mengikuti telemetry
- Label health menampilkan status P2P (`ONLINE`/`SYNCING`/`DISABLED`)
- Tabel peer menampilkan data `known_peers`

### TC-UI-03: NodeRunner operator actions
Page: `/node`
Actions:
- Click `Force Chain Resync`
- Click `Bootstrap from Seeds`
Expected:
- Menampilkan alert sukses/gagal
- Setelah refresh data, status/peer list bergerak sesuai command

### TC-UI-04: Ban/unban peer from NodeRunner
Page: `/node`
Actions:
- Click `Ban Peer Endpoint`, input valid `host:port`
- Click `Unban Peer Endpoint`, input endpoint yang sama
Expected:
- Menampilkan alert command accepted
- Ringkasan P2P menampilkan `Banned` count sesuai update
- `GET /api/p2p/banlist` konsisten dengan UI

### TC-UI-05: Undo last dashboard batch
Page: `/`
Actions:
- Jalankan salah satu batch action sukses (`Import Ban List`, `Ban All Risk`, atau `Unban All`)
- Click `Undo Last Batch`
Expected:
- Menampilkan toast sukses undo dengan rasio `accepted/total`
- `Operator Activity` mencatat `UNDO_BATCH`
- Snapshot batch terakhir dihapus setelah undo sukses (tombol undo disabled)

## Multi-Node Scenario (Manual)

### Topology
- Node A: `NFM_API_PORT=3000`, `NFM_P2P_PORT=9000`
- Node B: `NFM_API_PORT=3001`, `NFM_P2P_PORT=9001`, seed ke A

Run example:
```powershell
# terminal 1
$env:NFM_API_PORT="3000"; $env:NFM_P2P_PORT="9000"; $env:NFM_P2P_GOSSIP="true"; $env:NFM_P2P_SEEDS=""; cargo run --release

# terminal 2
$env:NFM_API_PORT="3001"; $env:NFM_P2P_PORT="9001"; $env:NFM_P2P_GOSSIP="true"; $env:NFM_P2P_SEEDS="127.0.0.1:9000"; cargo run --release
```

### TC-MESH-01: Seed discovery
- On Node B, call `POST /api/p2p/bootstrap`
- Validate Node B `GET /api/p2p/status` shows non-zero peers and includes Node A endpoint in `known_peers`

### TC-MESH-02: Cross-node sync trigger
- On Node B, call `POST /api/p2p/sync`
- Validate `last_sync_unix` updated
- Validate `chain_blocks` does not regress

### TC-MESH-03: Health probe + auto-reconnect telemetry
- Start Node B with invalid seed first (e.g. `127.0.0.1:9999`) and valid seed second (`127.0.0.1:9000`)
- Trigger `POST /api/p2p/bootstrap`
- Observe `GET /api/p2p/status` on Node B over ~15s
Expected:
- `unhealthy_peers` may become > 0 before pruning
- `reconnect_attempts` may increment when peers drop to 0 and re-bootstrap runs
- `status` can temporarily be `reconnecting` or `backoff_wait` then back to `online`
- `next_reconnect_unix` should move forward and `reconnect_backoff_secs` can increase up to capped value when repeated reconnect attempts happen
- For healthy peers, `peer_health[].score` should be in range 1..100 and `quality` should map roughly to latency (`excellent/good/degraded/poor`)

## Regression Checklist
- Existing app endpoints still work:
  - `/api/app/state`
  - `/api/app/wallet/transfer`
  - `/api/app/governance/proposal`
  - `/api/app/governance/vote`
  - `/api/app/quest/claim`
  - `/api/app/mystery/extract`
  - `/api/app/market/purchase`

## Notes for Tester
- Endpoint `POST /api/p2p/bootstrap` and `POST /api/p2p/sync` are asynchronous command triggers (expect `202`, not immediate final state).
- On a single-node local setup without seeds, `known_peers` can remain empty and this is valid behavior.
