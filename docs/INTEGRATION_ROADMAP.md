# NFM Frontend-Backend Integration Roadmap

Date: 2026-04-01
Status: Active execution (Phase 6A-6E completed, Brain Curriculum baseline active)

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
- NLC workflow UX
- Brain curriculum advanced governance lifecycle (quorum/execution automation)

## Gap Matrix (Prioritized)

| Feature | Backend | Frontend | Gap Type | Priority | Definition of Done |
|---|---|---|---|---|---|
| NFM-ID Titles and Elite Shield visibility | Endpoint /api/identity/{address} + elite logic sudah aktif | Badge elite sudah tampil di Marketplace | Integrated + guarded | High | Endpoint identity tersedia, badge/title tampil di profile/dashboard, elite shield state terlihat |
| Auction and Escrow Marketplace | Endpoint create/list/bid/settle/cancel sudah aktif | Marketplace sudah mendukung create/bid/settle flow | Integrated + guarded | Critical | Create/bid/cancel/settle berjalan, escrow state tampil, anti-sniping terbaca |
| NFM Drive SDS | Upload/list/download endpoint aktif + ownership guard | UI upload/download aktif di Drive page | Integrated + guarded | High | Upload/list/download API aktif, progress upload dan health fragment tampil |
| Native Brain Curriculum | Endpoint curriculum propose/active/vote + leaderboard sudah aktif | AIBrain sudah wired ke curriculum governance flow | Integrated (baseline) + guarded | High | Proposal curriculum, voting, learning window, leaderboard tampil di UI |
| Settings persistence | GET/POST /api/app/settings aktif | Settings page sudah sinkron ke backend | Integrated | Medium | Settings tersimpan server-side, refresh state mengembalikan nilai konsisten |
| Governance advanced mechanics | GET /api/governance/indicators aktif | Governance page sudah konsumsi quorum/veto/treasury | Integrated (read model) | Medium | Voting power berbasis stake, quorum/veto indicator, execution status tampil |
| Tokenomics dashboard | status payload memuat reward_pool/circulating/total_supply | Explorer sudah tampilkan sebagian metrik tokenomics | Partial UI integration | Medium | Panel tokenomics live dengan source dari endpoint status dan state |
| Knowledge Graph semantic view | GET /api/kg/semantic aktif | KnowledgeGraph page sudah konsumsi endpoint semantic + fallback | Integrated (read model) | Medium | Endpoint KG return schema kategori/relasi, UI render cluster and detail |
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

## Execution Status Snapshot (2026-04-01)
- Phase 6A: Completed
- Phase 6B: Completed
- Phase 6C: Completed (identity + brain curriculum baseline)
- Phase 6D: Completed (governance indicators + KG semantic + frontend wiring)
- Phase 6E: Completed (guards expanded incl. identity + Phase6D + frontend flow + brain curriculum contracts)

Current gate coverage:
- Transfer fee guard
- Secure auth guard
- Drive ownership guard
- Identity elite shield guard
- Phase 6D contract guard (/api/governance/indicators + /api/kg/semantic)
- Frontend flow contract guard (/api/app/state + marketplace/drive/governance/kg read models)
- Brain curriculum contract guard (/api/brain/curriculum/propose + /api/brain/curriculum/active + /api/brain/curriculum/vote + /api/brain/reputation/leaderboard)

## Next Phase (Operational)
Phase berikutnya difokuskan pada NLC workflow UX dan brain curriculum lifecycle hardening:
1. NLC intent builder + preview + history di frontend.
2. Governance-grade execution lifecycle untuk curriculum (quorum, timeout, execution status).
3. Add dedicated NLC and curriculum execution guards into policy gate.

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
- [x] API contract finalized
- [x] Backend handler implemented
- [x] Frontend wiring completed
- [x] Type contract aligned
- [x] Tests added and passing
- [x] Policy gate updated
- [x] Documentation updated

## Success Criteria
Roadmap dianggap selesai jika:
1. Semua feature di Gap Matrix berstatus integrated atau intentionally deferred dengan alasan tertulis.
2. Policy gate lulus konsisten minimal 3 run berturut-turut.
3. Tidak ada regression pada auth guard dan transfer fee guard.
