# NFM Frontend-Backend Integration Roadmap

Date: 2026-04-01
Status: Draft for execution

## Goal
Menyelesaikan seluruh gap integrasi fitur antara nfm-explorer frontend dan blockchain backend berdasarkan dokumen produk, kondisi endpoint aktual, dan wiring UI saat ini.

## Scope and Sources
Dokumen acuan:
- docs/app_suite_definition.md
- docs/marketplace_and_auctions.md
- docs/nfm_id_titles.md
- docs/native_brain_and_learning.md
- docs/gamification_and_quests.md
- docs/implementation_roadmap.md

Implementasi acuan:
- Frontend: apps/nfm-explorer/src/pages, apps/nfm-explorer/src/context/AppDataContext.tsx, apps/nfm-explorer/src/api/client.ts
- Backend: core/blockchain/src/api.rs dan modul domain terkait

## Current Integration Baseline
Sudah berjalan end-to-end:
- Core app state loading, governance proposal/vote, market purchase basic, quest claim basic
- Transfer fee guard dan secure auth guard tervalidasi oleh policy gate
- Pre-release gate tersedia: scripts/pre_release_policy_gate.ps1

Belum end-to-end atau masih parsial:
- Identity and title system
- Auction and escrow APIs
- Drive upload/download lifecycle
- Native brain curriculum governance loop
- Settings persistence endpoint
- Tokenomics transparency dashboard
- Knowledge graph semantic contract

## Gap Matrix (Prioritized)

| Feature | Backend | Frontend | Gap Type | Priority | Definition of Done |
|---|---|---|---|---|---|
| NFM-ID Titles and Elite Shield visibility | Struktur identity ada, endpoint publik belum jelas | UI badge/title belum ada | Missing wiring + API read model | High | Endpoint identity tersedia, badge/title tampil di profile/dashboard, elite shield state terlihat |
| Auction and Escrow Marketplace | Engine auction ada, route API auction belum expose penuh | Marketplace page masih listing-centric, belum bidding flow | Missing API + UI flow | Critical | Create/bid/cancel/settle berjalan, escrow state tampil, anti-sniping terbaca |
| NFM Drive SDS | Data drive tampil dari app state, upload endpoint belum tersedia | Tombol upload masih menampilkan error endpoint belum tersedia | Missing API | High | Upload/list/download API aktif, progress upload dan health fragment tampil |
| Native Brain Curriculum | Endpoint brain route/benchmark/fetch ada, curriculum governance belum expose | AIBrain masih command mock dan queue monitor | Missing governance loop | High | Proposal curriculum, voting, learning window, leaderboard tampil di UI |
| Settings persistence | client.ts memanggil api app settings | backend route app settings belum tersedia | Missing API | Medium | Settings tersimpan server-side, refresh state mengembalikan nilai konsisten |
| Governance advanced mechanics | Basic proposal/vote ada | Governance UI ada | Partial feature parity | Medium | Voting power berbasis stake, quorum/veto indicator, execution status tampil |
| Tokenomics dashboard | total_fees/total_burned ada pada status | visualisasi fee, burn rate, supply flow belum komplit | Partial UI integration | Medium | Panel tokenomics live dengan source dari endpoint status dan state |
| Knowledge Graph semantic view | Brain snapshot records ada | KG page butuh concept metadata terstruktur | Contract mismatch | Medium | Endpoint KG return schema kategori/relasi, UI render cluster and detail |
| NLC workflow UX | Endpoint api nlc ada dan protected | UI dedicated NLC builder belum ada | Missing frontend workflow | Medium | Intent builder, preview intent, result history tersedia |

## Execution Plan

### Phase 6A - API Parity Critical (Week 1)
1. Implement and expose auction API routes in backend API layer.
2. Implement app settings route for persistence.
3. Implement drive upload/list/download minimal viable endpoints.
4. Add integration tests for new routes and auth constraints.

Deliverables:
- core/blockchain/src/api.rs updated with new handlers
- Unit/integration tests for auction, settings, drive
- API docs surfaced in app state api_docs

### Phase 6B - Frontend Wiring Critical (Week 1-2)
1. Marketplace: add create auction, bid, settlement state, escrow badges.
2. Drive: replace upload placeholder with real upload and status polling.
3. Settings: persist and hydrate from backend route.

Deliverables:
- apps/nfm-explorer/src/pages/Marketplace.tsx wired to auction APIs
- apps/nfm-explorer/src/pages/Drive.tsx wired to drive APIs
- apps/nfm-explorer/src/pages/Settings.tsx sync success and fallback handling

### Phase 6C - Identity and Brain Governance (Week 2)
1. Add identity read endpoint and map title categories from docs.
2. Add frontend identity badge sections in profile/dashboard.
3. Add brain curriculum proposal and active learning window endpoint + UI panel.

Deliverables:
- Identity UI and endpoint contract finalized
- AIBrain curriculum workflow visible and actionable

### Phase 6D - Contract Alignment and Telemetry (Week 3)
1. Add KG semantic endpoint for typed concepts and relation count.
2. Add tokenomics dashboard sections: fees, burned, reward pool trend.
3. Add governance advanced indicators (stake weight, quorum, execution state).

Deliverables:
- Consistent schema between backend payload and frontend types
- Updated types in apps/nfm-explorer/src/types/index.ts

### Phase 6E - Stabilization and Release Gate (Week 3)
1. Expand pre-release gate to include new API regression checks.
2. Add frontend smoke validations for primary user flows.
3. Update report and implementation docs with final coverage matrix.

Deliverables:
- scripts/pre_release_policy_gate.ps1 includes new checks
- docs/report and roadmap updated with integration completion score

## Non-Negotiable Rules During Integration
1. Tidak boleh bypass secure endpoint auth/signature.
2. Tidak boleh bypass gas fee enforcement di transfer intent.
3. Semua endpoint baru wajib masuk policy gate check sebelum merge.

## API Backlog (Initial Contract Targets)

### Auction
- POST /api/auction/create
- POST /api/auction/bid
- POST /api/auction/cancel
- POST /api/auction/settle
- GET /api/auction/list
- GET /api/auction/:id

### Drive
- POST /api/drive/upload
- GET /api/drive/files
- GET /api/drive/file/:id
- POST /api/drive/download

### Identity
- GET /api/identity/:address
- GET /api/identity/titles

### Brain Curriculum
- POST /api/brain/curriculum/propose
- GET /api/brain/curriculum/active
- POST /api/brain/curriculum/vote
- GET /api/brain/reputation/leaderboard

### Settings
- POST /api/app/settings
- GET /api/app/settings

## Test Strategy
1. Backend unit and integration tests per route group.
2. Frontend component and interaction tests for pages with new wiring.
3. End-to-end smoke via scripts/integration_e2e_smoke.ps1 plus policy gate.
4. Mandatory gate command before release:
   powershell -ExecutionPolicy Bypass -File scripts/pre_release_policy_gate.ps1

## Progress Tracking Template
Gunakan checklist ini per phase:
- [ ] API contract finalized
- [ ] Backend handler implemented
- [ ] Frontend wiring completed
- [ ] Type contract aligned
- [ ] Tests added and passing
- [ ] Policy gate updated
- [ ] Documentation updated

## Success Criteria
Roadmap dianggap selesai jika:
1. Semua feature di Gap Matrix berstatus integrated atau intentionally deferred dengan alasan tertulis.
2. Policy gate lulus konsisten minimal 3 run berturut-turut.
3. Tidak ada regression pada auth guard dan transfer fee guard.
