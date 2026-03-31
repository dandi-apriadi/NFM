# Parallel Development Guide — Frontend (AG) & Backend (Backend-AI)

**Status**: Active parallel development
**Frontend Branch**: TBD (AG's choice)
**Backend Branch**: `feature/langkah3-turboq-training`
**Coordination**: Weekly syncs, zero hard conflicts

---

## 🎯 Development Isolation

### **Frontend (AG Team)**
```
📍 Location: apps/nfm-explorer/
📋 Activity: UI design with dummy data
🎨 Tools: React, Vite, TypeScript
🔌 API: Planned for Phase 2 (no integration yet)
```

**Touches**:
- ✅ `apps/nfm-explorer/src/` (UI components, styling)
- ✅ `apps/nfm-explorer/package.json` (dependencies)
- ✅ `docs/` (UI documentation, mockups)

**Avoids**:
- ❌ `core/` (backend code)
- ❌ `blueprint.txt` (engineering standards)

---

### **Backend (Backend-AI Team)**
```
📍 Location: core/ai-engine/
📋 Activity: TurboQuant + training → federated → governance
⚙️ Tools: Rust, Cargo, PyTorch FFI (Phase 2)
🔌 API: JSON schema defined in Phase 2
```

**Touches**:
- ✅ `core/ai-engine/src/` (quantization, training, federated, audit, etc.)
- ✅ `core/blockchain/src/` (Langkah 4: governance extension)
- ✅ `core/` (backend infrastructure)
- ✅ `docs/BACKEND_ARCHITECTURE.md` (backend specs)

**Avoids**:
- ❌ `apps/nfm-explorer/` (frontend UI)
- ❌ Frontend component imports
- ❌ HTML/CSS/React files

---

## 🔀 Git Workflow

### **Branch Strategy**

```
main (stable, shared)
├── releases/v0.1.0-maint (hotfixes)
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  
  feature/ui-phase1 (AG - FRONTEND)
  ├─ Commit: "ui: add learning window mockup"
  ├─ Files: apps/nfm-explorer/src/pages/LearningWindow.tsx
  └─ NO: core/ files, backend changes
  
  feature/langkah3-turboq-training (Backend - CURRENT)
  ├─ Commit: "feat: quantization + training"
  ├─ Files: core/ai-engine/src/quantization.rs
  └─ NO: apps/nfm-explorer/, UI files
```

### **Safe Merge Points**

**Phase 1** (Minggu 4): Separate gate reviews
```
Backend: L3+L4 ready → Merge to main
Frontend: UI components ready → Merge to main
Conflict risk: ZERO (different directories)
```

**Phase 2** (Minggu 8): Integration begins
```
Backend: API endpoints defined (json.rs)
Frontend: API consumer ready (hooks/)
Integration: Planned merge with API mediation
```

---

## 🚨 Conflict Prevention Checklist

### **Daily Check (Before Git Push)**

- [ ] **File scope isolation?**
  - Backend: only `core/**`, `docs/BACKEND_ARCHITECTURE.md`?
  - Frontend: only `apps/**`, mockup docs?
  
- [ ] **Import dependencies?**
  - Backend NOT importing from `apps/`?
  - Frontend NOT importing from `core/`?
  
- [ ] **Cargo.toml changes?**
  - Backend: only `core/ai-engine/Cargo.toml`, `core/blockchain/Cargo.toml`?
  - Frontend: only `apps/nfm-explorer/package.json`?
  
- [ ] **Docs changes?**
  - Backend: ONLY `docs/BACKEND_ARCHITECTURE.md` (dedicated)?
  - Frontend: mockup docs separate from technical docs?

### **Before Committing**

```bash
# Backend team:
git diff --stat
# Should show ONLY core/**

# Frontend team (AG):
git diff --stat
# Should show ONLY apps/**
```

### **Safe Merge**

```bash
# NEVER merge if:
git diff main...feature/langkah3-turboq-training -- apps/
# Returns ANY changes (= you touched frontend!)

# NEVER merge if:
git diff main...feature/ui-phase1 -- core/
# Returns ANY changes (= you touched backend!)
```

---

## 🤝 Communication Protocol

### **Weekly Sync (Every Monday 10:00 AM)**

**Participants**: Lead Architect + AG + Backend-AI Lead

**Agenda** (30 min):
1. Frontend progress: Which UI components done?
2. Backend progress: Which modules merged?
3. Integration plan: What API contracts needed?
4. Blockers: Any dependencies between teams?
5. Timeline alignment: Still on schedule?

**Example Sync Notes**:
```
🟢 Frontend (AG):
  - Learning Window mockup ✅
  - Wallet UI refactor 🔄 (next week)
  - Needs: API contract for proposal submission

🟢 Backend:
  - Quantization ✅
  - Training loop ✅  
  - Governance MVP 🔄 (next week)
  - Provides: /api/governance/submit_proposal (JSON schema v1)

🟡 Blockers:
  - None currently
  
📅 Next sync: April 7, 2026
```

### **Async Communication (Slack/Discord)**

**Channels**:
- `#nfm-backend`: Backend discussions
- `#nfm-frontend`: Frontend discussions  
- `#nfm-integration`: Cross-team API contracts
- `#nfm-general`: Announcements

**Protocol**:
- Tag engineers by role (e.g., @backend-lead)
- Prefix messages: `[BACKEND]`, `[FRONTEND]`, `[INTEGRATION]`
- PRs: Link to channel, request reviews only from relevant team

---

## 📦 Integration Checklist (Phase 2, Week 8)

When it's time to connect frontend to backend APIs:

### **Before First Integration Commit**

- [ ] **API Contracts Finalized**
  - [ ] JSON schema for all endpoints
  - [ ] Request/response examples
  - [ ] Error codes + handling

- [ ] **Version Compatibility**
  - [ ] Backend API v1.0 tagged
  - [ ] Frontend consumer v1.0 tagged
  - [ ] Backward compatibility plan

- [ ] **Testing**
  - [ ] Backend: E2E tests with mock frontend
  - [ ] Frontend: E2E tests with mock backend
  - [ ] Integration tests (both together)

- [ ] **Rollback Plan**
  - [ ] Feature flags for new API features
  - [ ] Fallback to dummy data if API down
  - [ ] Version negotiation in handshake

### **First Real Integration (Week 8)**

```bash
# 1. Create integration branch from main
git checkout -b feature/api-integration-phase2

# 2. Backend adds API layer (JSON)
# File: core/blockchain/src/api.rs (extend existing)
# + new types in core/shared/src/api_types.rs

# 3. Frontend adds API consumer
# File: apps/nfm-explorer/src/hooks/useLearnWindow.ts
# + api client in src/api/learned-window-client.ts

# 4. Run integration tests
cargo test --features integration
npm run test:integration

# 5. Merge to main (both teams sign off)
git merge main --no-ff
```

---

## 🚀 If We Need to Coordinate Mid-Phase

### **Scenario 1: Frontend Needs Backend Feature Early**

```
Problem: AG needs /api/learning-window/list before Week 8

Action:
1. Backend creates feature/api-preview branch
2. Implement API endpoint stub (returns mock data)
3. Merge ONLY the API layer to feature/langkah3-turboq-training
4. AG creates feature/ui-integration-preview
5. Integrate stub API for testing
6. Both branches stay separate (merge Week 8)
```

### **Scenario 2: Backend Wants UI for Testing**

```
Problem: Backend needs UI to manually test audit feature

Action:
1. Backend creates test CLI: nfm-core audit test-model.bin
2. Outputs JSON (for parsing, not UI)
3. AG can convert JSON to mockup dialog (separate)
4. No code dependency between teams
```

### **Scenario 3: Merge Conflict in docs/**

```
Problem: Both teams edit docs/implementation_roadmap.md

Action:
1. Create dedicated docs:
   - docs/BACKEND_ARCHITECTURE.md (Backend only)
   - docs/FRONTEND_COMPONENTS.md (Frontend only - if needed)
   - docs/implementation_roadmap.md (Shared, careful editing)
2. Use merge strategy: `-X ours` / `-X theirs` for road map
3. Manual review + rebase if conflict critical
```

---

## ✅ Checklist for New Team Member Joining

If another engineer joins:

- [ ] Read this file first
- [ ] Ensure they work on **separate** branch
- [ ] Verify no cross-imports
- [ ] Add to weekly sync calendar
- [ ] Set up Slack notifications (channel subscriptions)

---

## 📊 Progress Tracking

### **Frontend (AG)**
Progress tracked in: `apps/nfm-explorer/README.md`

### **Backend**
Progress tracked in: `docs/BACKEND_ARCHITECTURE.md` (updated per Langkah)

### **Shared**
Progress tracked in: `docs/implementation_roadmap.md`

**Update Frequency**: After each gate review (weekly)

---

## 🔒 Code Review Rules

### **Backend PR Review Checklist**

- ✅ No imports from `apps/`?
- ✅ No HTML/CSS/JSX files?
- ✅ Cargo tests passing?
- ✅ Documentation updated (BACKEND_ARCHITECTURE.md)?
- ✅ Commit message references Langkah number?

### **Frontend PR Review Checklist** (for AG)

- ✅ No imports from `core/`?
- ✅ No Rust files (.rs)?
- ✅ npm tests passing?
- ✅ Component isolation maintained?
- ✅ Dummy data only (no live API calls)?

---

## 🆘 Emergency: Unintended Conflict

If a conflict does occur:

1. **Identify scope**:
   ```bash
   git diff --stat main feature/langkah3-turboq-training -- apps/
   # If results non-empty: CONFLICT!
   ```

2. **Stop and communicate**:
   - Notify #nfm-integration channel immediately
   - Include git diff output
   - Tag both team leads

3. **Resolution options**:
   - **Option A**: Revert conflicting commit, redo carefully
   - **Option B**: Pause merges until Phase 2 integration
   - **Option C**: Manually mediate via code review

4. **Post-mortem**: Update this guide

---

## 📞 Escalation Path

| Issue | Owner | Channel | SLA |
|-------|-------|---------|-----|
| Branch conflict | Lead Architect | #nfm-integration | 2h |
| Merge blocker | PM | #nfm-general | 24h |
| Emergency hotfix | Backend Lead | #nfm-backend | 15m |

---

**Last Updated**: March 31, 2026
**Version**: 1.0
**Status**: Active (Weekly sync: Mondays 10:00 AM)
