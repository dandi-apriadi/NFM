// ========================================
// BLOCKCHAIN RESET FEATURE - QUICK REFERENCE
// ========================================

// 🎯 WHAT IT DOES
// Resets entire blockchain to genesis state
// Clears all blocks, balances, governance, missions, auctions
// Designed for development/testing only

// 📍 WHERE TO FIND IT
UI Button:      Settings → Preferences → Development Tools → Reset Blockchain
Backend:        core/blockchain/src/api.rs (lines 2856-2915)
Frontend:       apps/nfm-explorer/src/pages/Settings.tsx
API Client:     apps/nfm-explorer/src/api/client.ts
Tests:          test_blockchain_reset.ps1
Docs:           docs/BLOCKCHAIN_RESET_DEV.md

// 🔐 AUTHENTICATION
Method:         POST /api/admin/reset
Auth Type:      HMAC-SHA256 signature
Required Secret: Your API secret (e.g., "nfm_admin_secret_key")
Response:       200 = Success | 403 = Invalid signature

// 📋 WHAT GETS RESET
✅ Blockchain chain (all blocks)
✅ Wallet balances
✅ Governance proposals
✅ Mission engine state
✅ Staking pools
✅ Auctions
✅ Transaction fees/burned
✅ User settings (rewards, etc.)
✅ Mempool

// 💾 WHAT DOESN'T GET RESET
❌ Browser local storage (seed phrase, RPC URL, theme)
❌ Node configuration
❌ API secret

// 🚀 HOW TO USE

// Via Frontend UI:
1. Open http://localhost:5173
2. Click Settings (gear icon)
3. Go to Preferences tab
4. Scroll to "Development Tools" section
5. Click "Reset Blockchain to Genesis"
6. Enter API secret
7. Confirm
8. Wait for page reload

// Via Test Script:
.\test_blockchain_reset.ps1 -ApiPort 3000 -ApiSecret "your-secret"

// Via cURL:
$payload = "http://127.0.0.1:3000/api/admin/reset:{}"
$hmac = ComputeHMAC256($secret, $payload)
curl -X POST http://127.0.0.1:3000/api/admin/reset \
  -H "X-NFM-Signature: $hmac" \
  -H "Content-Type: application/json" \
  -d "{}"

// ⚠️  WARNINGS
- IRREVERSIBLE: All blockchain state is permanently cleared
- DEVELOPMENT ONLY: Will be removed before production
- SINGLE USER: Only reset one blockchain at a time
- NO AUTOMATIC RECOVERY: Once reset, blocks are gone forever

// 🗑️  REMOVING FOR PRODUCTION

Search for: [DEV-ONLY] and DEVELOPMENT-ONLY
Delete:
  1. /api/admin/reset endpoint from api.rs (lines 2856-2915)
  2. handleBlockchainReset() from Settings.tsx
  3. "Development Tools" section from Settings.tsx
  4. appAdminResetBlockchain() from client.ts
  5. test_blockchain_reset.ps1
  6. docs/BLOCKCHAIN_RESET_DEV.md

Estimated time: ~5 minutes

// 📊 TESTING CHECKLIST
✅ Backend compiles without errors
✅ Frontend builds without TypeScript errors
✅ Invalid signature returns 403
✅ Valid signature resets blockchain
✅ Block count goes to 0 after reset
✅ Multiple resets work correctly
✅ Reset logs to audit trail
✅ Page reloads successfully after reset

// 💡 TIPS
- Keep API secret secure during testing
- Run reset before stress testing to ensure clean state
- Use test script to automate reset between test runs
- Reset is deterministic - same result every time
- Ideal for clearing bad state during development

// 🔍 DEBUGGING
If reset fails:
  - Check API secret is correct
  - Verify server is running on correct port
  - Check HMAC signature computation matches backend
  - Look for "ADMIN_RESET" in block transaction logs

If state doesn't clear:
  - Check /api/status endpoint shows 0 blocks
  - Check wallet balances are 0
  - Check governance has no proposals
  - Refresh page to reload UI state

// 🎓 LEARN MORE
See BLOCKCHAIN_RESET_DEV.md for complete documentation
See BLOCKCHAIN_RESET_SUMMARY.md for implementation details
See test_blockchain_reset.ps1 for working code examples

// ========================================
