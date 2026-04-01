# Blockchain Reset Feature [DEVELOPMENT-ONLY]

## Overview

Added a development-only **blockchain reset endpoint** that allows resetting all blockchain state back to genesis conditions. This feature is intended for testing and development and **will be removed before production deployment**.

## Components

### 1. Backend Endpoint: `/api/admin/reset`

**Location:** [`core/blockchain/src/api.rs`](../core/blockchain/src/api.rs#L2856) (lines 2856-2915)

**Method:** `POST`

**Authentication:** Required (HMAC-SHA256 signature validation)

**Request:**
```json
{}
```

**Response (Success):**
```json
{
  "status": "success",
  "message": "Blockchain reset to genesis. All state cleared.",
  "timestamp": 1712043600
}
```

**Response (Auth Failure - 403):**
```json
{
  "error": "Forbidden: invalid signature"
}
```

### 2. What Gets Reset

The endpoint performs a complete reset of:
- ✅ Blockchain chain (all blocks)
- ✅ Transaction fees collected
- ✅ Burned coins
- ✅ Reward pool
- ✅ Active effects
- ✅ Missions (completed, active, inventory)
- ✅ Staking pools
- ✅ Wallet balances
- ✅ Governance proposals
- ✅ Transaction aliases
- ✅ Mempool
- ✅ User settings
- ✅ Auctions
- ✅ Next auction ID (reset to 1)

### 3. Frontend Integration

**Location:** [`apps/nfm-explorer/src/pages/Settings.tsx`](../apps/nfm-explorer/src/pages/Settings.tsx)

Added "Development Tools" section to the **Preferences (prefs) tab** in Settings:

- **Section Label:** "DEVELOPMENT TOOLS (TEMPORARY)"
- **Warning Banner:** Yellow/warning background with AlertTriangle icon
- **Button:** "Reset Blockchain to Genesis" (red action button)
- **Disclaimer:** "These tools are for development/testing only and will be removed before production."

**User Flow:**
1. User opens Settings → Preferences tab
2. Scrolls to "Development Tools" section
3. Clicks "Reset Blockchain to Genesis" button
4. Prompted to enter API Secret
5. On confirmation:
   - Shows success notification: "✅ Blockchain reset complete!"
   - Reloads page after 2 seconds to reflect empty state

### 4. API Client Function

**Location:** [`apps/nfm-explorer/src/api/client.ts`](../apps/nfm-explorer/src/api/client.ts) (exported function)

```typescript
export async function appAdminResetBlockchain(adminSecret: string) {
  return request('/api/admin/reset', 'POST', { secret: adminSecret });
}
```

## Security Considerations

1. **HMAC-SHA256 Signature Required:** Endpoint is protected with the same authentication mechanism as other admin endpoints.
2. **No Default Secret:** Users must know the API secret to perform reset.
3. **Rate Limiting:** Subject to global rate limiting if enabled.
4. **Audit Trail:** Reset operations are logged to the block transaction channel: `"ADMIN_RESET: Blockchain reset to genesis conditions"`

## Testing

A test script has been provided: [`test_blockchain_reset.ps1`](../test_blockchain_reset.ps1)

**Test Coverage:**
- ✅ Invalid signature rejection (403)
- ✅ Valid HMAC-SHA256 signature acceptance (200)
- ✅ Verification that blocks are actually cleared to 0

**Run the test:**
```powershell
.\test_blockchain_reset.ps1 -ApiPort 3000 -ApiSecret "nfm_admin_secret_key"
```

## Removal for Production

Before deploying to production, **MUST remove:**

1. **Backend:**
   - Delete the `/api/admin/reset` endpoint handler from `api.rs` (lines 2856-2915)
   - Remove the endpoint from `is_protected_endpoint()` function if listed

2. **Frontend:**
   - Remove the "Development Tools" section from Settings.tsx
   - Remove the `handleBlockchainReset()` function
   - Remove the `appAdminResetBlockchain()` function from `client.ts`

3. **Tests:**
   - Delete `test_blockchain_reset.ps1`

4. **Documentation:**
   - Delete this file or archive under "archived-dev-docs"

## Code Markers

All reset-related code is marked with `[DEV-ONLY]` comments:
- Backend endpoint: `// --- DEVELOPMENT-ONLY: BLOCKCHAIN RESET TO GENESIS [DEV-ONLY]`
- Frontend function: `// ========== DEVELOPMENT-ONLY: BLOCKCHAIN RESET ==========`
- API client: `// ========== DEVELOPMENT-ONLY: BLOCKCHAIN RESET ==========`

**Search for `[DEV-ONLY]` and `DEVELOPMENT-ONLY` to find all removal points.**

## Implementation Details

### Backend Logic (api.rs)

```rust
("POST", "/api/admin/reset") => {
    // [AUTH CHECK - HMAC-SHA256]
    if !verify_admin_signature(&state.api_secret, "/api/admin/reset", &content, &sig_header) {
        return (403, "application/json", ...) // Forbidden
    }
    
    // [RESET ALL STATE]
    *state.chain.lock().unwrap() = Vec::new();  // Clear blocks
    // ... reset all Arc<Mutex> fields
    
    // [AUDIT LOG]
    state.block_tx.send("ADMIN_RESET: ...").ok();
    
    // [RETURN SUCCESS]
    return (200, "application/json", { status: "success", ... })
}
```

### Frontend Logic (Settings.tsx)

```typescript
const handleBlockchainReset = async () => {
  // [PROMPT FOR SECRET]
  const secret = await requestPrompt({ ... });
  
  if (secret) {
    try {
      // [CALL API]
      await client.appAdminResetBlockchain(secret);
      
      // [SUCCESS FEEDBACK]
      notifySuccess('✅ Blockchain reset complete!');
      
      // [RELOAD UI]
      setTimeout(() => window.location.reload(), 2000);
    } catch (err) {
      notifyError(`⚠️ Reset failed: ${err.message}`);
    }
  }
}
```

## Status

- ✅ **Backend:** Implemented and tested (compiles without errors)
- ✅ **Frontend:** Implemented and tested (builds without TypeScript errors)
- ✅ **Test Script:** Created for automated validation
- ✅ **Documentation:** Complete with removal procedure

## Future Improvements (Optional)

- [ ] Add confirmation dialog with warning about irreversibility
- [ ] Log reset operations to persistent audit log
- [ ] Add optional parameter to reset only specific components (e.g., just balance, just governance)
- [ ] Add timestamp for when genesis state was created

---

**Note:** This feature has been specifically designed to be ephemeral and easily removable when transitioning to production.
