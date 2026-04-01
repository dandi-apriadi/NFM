# Blockchain Reset Feature - Implementation Summary

## What Was Created

A complete **development-only blockchain reset feature** that allows resetting all blockchain state back to genesis conditions. Designed to be easily removed before production.

## Files Modified/Created

### Backend Changes
- ✅ **Modified:** [`core/blockchain/src/api.rs`](../core/blockchain/src/api.rs)
  - Added POST `/api/admin/reset` endpoint (lines 2856-2915)
  - Clears all state: chain, balances, governance, missions, auctions, etc.
  - Requires HMAC-SHA256 authentication
  - Logs reset action to block transaction channel
  - Returns success message with timestamp

### Frontend Changes
- ✅ **Modified:** [`apps/nfm-explorer/src/pages/Settings.tsx`](../apps/nfm-explorer/src/pages/Settings.tsx)
  - Added import for AlertTriangle icon
  - Added `appAdminResetBlockchain` API import
  - Created `handleBlockchainReset()` async handler:
    - Prompts for API secret
    - Calls reset endpoint with HMAC signature
    - Shows success/error notifications
    - Reloads page on success
  - Added "Development Tools" section in Preferences tab:
    - Yellow warning banner with disclaimer
    - Red action button to trigger reset
    - Warning text explaining temporary nature

- ✅ **Modified:** [`apps/nfm-explorer/src/api/client.ts`](../apps/nfm-explorer/src/api/client.ts)
  - Exported `appAdminResetBlockchain(adminSecret: string)` function
  - Makes POST request to `/api/admin/reset`

### Tests & Documentation
- ✅ **Created:** [`test_blockchain_reset.ps1`](../test_blockchain_reset.ps1)
  - PowerShell test script for endpoint validation
  - Tests invalid signature rejection
  - Tests valid HMAC-SHA256 signature acceptance
  - Verifies chain is actually reset to 0 blocks

- ✅ **Created:** [`docs/BLOCKCHAIN_RESET_DEV.md`](../docs/BLOCKCHAIN_RESET_DEV.md)
  - Complete documentation
  - Security considerations
  - Clear removal procedure for production
  - Code markers for finding all reset-related code

## How It Works

### User Flow
```
1. User opens Settings → Preferences tab
2. Scrolls to "Development Tools" section
3. Clicks "Reset Blockchain to Genesis" button
4. Enters API Secret in prompt dialog
5. System validates signature and resets blockchain
6. Shows success message and reloads page
```

### Technical Flow
```
Frontend (TypeScript)
    ↓
  [User clicks reset button]
    ↓
  [Prompt for API secret]
    ↓
  appAdminResetBlockchain(secret)
    ↓
POST /api/admin/reset
    ↓
Backend (Rust)
    ↓
  [Verify HMAC-SHA256 signature]
    ↓
  IF invalid: Return 403 Forbidden
  IF valid: Clear all state
    ↓
  [ Return 200 Success ]
    ↓
Frontend
    ↓
  [ Show success notification ]
  [ Reload page after 2 seconds ]
```

## State That Gets Reset

✅ Blockchain chain (all blocks)
✅ Transaction fees
✅ Burned coins
✅ Reward pool
✅ Mission engine (completed, active, inventory)
✅ Staking pools
✅ Wallet balances
✅ Governance proposals
✅ Transaction aliases
✅ Mempool
✅ User settings
✅ Auctions (all auctions + escrow vault)
✅ Block timestamp

## Security

- **Authentication:** HMAC-SHA256 signature validation (same as other admin endpoints)
- **No Default Secret:** API secret required from operator
- **Rate Limiting:** Subject to global rate limits
- **Audit Trail:** All resets logged to block transaction channel
- **Protected Endpoint:** Listed in `is_protected_endpoint()` function

## Build Status

✅ **Backend:** Compiles without errors (`cargo check --release`)
✅ **Frontend:** Builds without errors (`npm run build`)
- Backend build time: ~5 seconds
- Frontend build time: ~529ms
- Final bundle: ~499KB (gzipped to ~140KB)

## Removal for Production

**Before deploying to production:**

1. Search codebase for `[DEV-ONLY]` and `DEVELOPMENT-ONLY` markers
2. Delete endpoint handler from `api.rs` (lines 2856-2915)
3. Delete Settings button, handler, and section from `Settings.tsx`
4. Delete API client function from `client.ts`
5. Delete test script `test_blockchain_reset.ps1`
6. Delete or archive documentation

**Estimated removal time:** ~5 minutes

## Key Design Decisions

1. **Marked as Development-Only:** All code clearly marked with [DEV-ONLY] comments for easy tracking
2. **No Hidden Functionality:** Visible warning banner in UI about temporary nature
3. **Same Auth Pattern:** Uses existing HMAC-SHA256 signature validation for consistency
4. **Complete Reset:** Clears absolutely all state, ensuring clean genesis on next block
5. **Audit Trail:** Logs action for operator review
6. **User Confirmation:** Requires API secret - not a single-click action
7. **Page Reload:** Ensures fresh UI state after reset

## Usage Example

```bash
# Using the frontend:
1. Open http://localhost:5173
2. Navigate to Settings (gear icon)
3. Click "Preferences" tab
4. Scroll to "Development Tools"
5. Click "Reset Blockchain to Genesis"
6. Enter your API secret: nfm_admin_secret_key
7. Confirm the reset
8. System resets and page reloads

# Using the test script:
powershell -ExecutionPolicy Bypass -File test_blockchain_reset.ps1 -ApiPort 3000 -ApiSecret "nfm_admin_secret_key"
```

## Notes

- Reset is **irreversible** - all blockchain state is permanently cleared
- Only resets in-memory state, does not affect local browser storage
- Frontend user settings (seed phrase, RPC URL, theme) are preserved
- Reset can be performed multiple times
- Ideal for rapid development iteration and testing
- Will be completely removed before mainnet deployment

---

**Created:** April 1, 2026
**Status:** ✅ Implementation Complete | ✅ Testing Ready | ⏳ Production Removal Pending
