# Technical Recommendations for Agent VS (NFM Explorer)

This document outlines the high-priority technical Debt and UI/UX enhancements required to elevate the NFM Explorer from "Functional Integration" to "Premium Production-Ready".

## 1. UI Architecture: Refinement of System Interactivity
> [!IMPORTANT]
> The current use of `window.alert`, `window.prompt`, and `window.confirm` breaks the immersive "Dark Mesh" aesthetic.

- **Action**: Implement a centralized `ModalManager` within `AppDataContext`.
- **Target**: Replace prompts in `Wallet.tsx` (amount), `Governance.tsx` (proposal text), and `NodeRunner.tsx` (peer endpoints).
- **Style**: Use the `nfm-modal` class from `components.css` with entry/exit animations from `framer-motion` or standard CSS transitions.

## 2. Notification System: Global Toast Hoisting
- **Observation**: The `Dashboard.tsx` contains an excellent Toast system that is currently local to that page only.
- **Action**: Move the Toast state and UI to `App.tsx` or `AppDataProvider`.
- **Effect**: Allows `api/client.ts` or any page to trigger `setToast({ type: 'success', message: '...' })` upon successful API calls like Quest claims or Peer bans.

## 3. Data Integrity & Types
- **Improvement**: Create more specialized types for API error responses. 
- **Action**: Update `request` handler in `api/client.ts` to return typed error objects (e.g., `{ code: 'INSUFFICIENT_BALANCE', message: '...' }`) to allow the UI to react specifically to certain failure modes.

## 4. UX: Enhanced Feedback Loops
- **Mystery Box**: Currently, the "Extraction" is a simple spinner. 
- **Recommendation**: Map the `isExtracting` state to a multi-stage visual process: 
  1. `CONNECTING_GATEWAY` 
  2. `SCANNING_SHARD` 
  3. `DECRYPTING_CARGO`
  4. `SUCCESS_FLASH`.
- **Latency Indicators**: In `NodeRunner.tsx`, add a "Signal Strength" icon (bars) next to latency ms to give users a quicker visual understanding of peer quality.

## 5. Security & State Persistence
- **Action**: Implement `sessionStorage` or `localStorage` persistence for:
  - `activeTab` on all paged modules (Quest, MysteryBox, Settings).
  - Search query strings in Explorer and Marketplace to prevent data loss on page refresh.

---
**Signed**: *Antigravity AI (Strategic Audit Lead)*
