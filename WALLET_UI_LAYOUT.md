# Wallet Page UI Layout Structure

## New Wallet Page Layout (http://localhost:5173/wallet)

```
┌─────────────────────────────────────────────────────────────────► Header Section
│  Wallet Management                          [+ Create New Wallet] [Connected: nfm_xxx▼]
│  Manage NFM identities and asset transfers
│
├─────────────────────────────────────────────────────────────────► Wallet List (if wallets exist)
│  Your Wallets
│  ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│  │  Wallet          │ │  Wallet 2 (ACTIVE)
│ │  Wallet 3        │
│  │ nfm_abc123...   │ │ nfm_def456... ✓ │ │ nfm_ghi789...   │
│  │                 │ │                  │ │                 │
│  │ 1,000,000 NVC   │ │ 500,000 NVC   │ │ 250,000 NVC    │
│  └──────────────────┘ └──────────────────┘ └──────────────────┘
│
├─────────────────────────────────────────────────────────────────► Main Actions Section (3-column wrap)
│
│  ┌──────────────────────────────┐  ┌─────────────────────┐  ┌──────────────────┐
│  │  VAULT EQUILIBRIUM           │  │ ASSET PORTFOLIO     │  │ STAKING VAULT    │
│  │                              │  │                     │  │ 💎               │
│  │  1,234,567 NVC              │  │ ● NVC              │  │ APY: 12.5%      │
│  │                              │  │   Neural Vault Coin │  │ Staked: 250K NVC │
│  │ [Receive] [Send] [@Alias]  │  │   1,234,567 NVC    │  │                  │
│  │                              │  │                     │  │ [Stake NVC]      │
│  │                              │  │ ● ETH (swap-able) │  │ [Unstake]        │
│  │                              │  │   Ethereum          │  │                  │
│  │                              │  │   0.5432 ETH       │  │                  │
│  │                              │  │                     │  │                  │
│  │                              │  │ ┌─────────────────┐ │  │                  │
│  │                              │  │ │ ⟲ Quick Swap    │ │  │                  │
│  │                              │  │ │ Swap NVC → ETH  │ │  │                  │
│  │                              │  │ └─────────────────┘ │  │                  │
│  └──────────────────────────────┘  └─────────────────────┘  └──────────────────┘
│
├─────────────────────────────────────────────────────────────────► Transaction History
│  📋 Recent Transactions                        [Filter ▼ All Types]
│
│  ┌────────┬──────────┬──────────────┬───────────────┬────────────┐
│  │ Type   │ Status   │ TX Hash      │ Amount        │ Time       │
├────────┼──────────┼──────────────┼───────────────┼────────────┤
│  │TRANSFER│ ✓CONFIRMED│ abcd123456ef...│ +1,000.00 NVC │ 2h ago     │
│  │BURN    │ ✓CONFIRMED│ bcde234567fg...│ -500.00 NVC   │ 4h ago     │
│  │REWARD  │ ✓CONFIRMED│ cdef345678gh...│ +250.00 NVC   │ 6h ago     │
│  │TRANSFER│ ⟳PENDING  │ defg456789hi...│ +100.00 NVC   │ 1h ago     │
│  └────────┴──────────┴──────────────┴───────────────┴────────────┘
│
│  [→ Full Transaction Ledger]
│
└─────────────────────────────────────────────────────────────────► End

```

## Responsive Behavior

### Desktop (> 1024px)
- Wallet grid: 3 columns
- Main sections: 3-column layout side-by-side
- Full-width transaction table

### Tablet (768px - 1024px)
- Wallet grid: 2 columns
- Main sections: Responsive wrapping
- Transaction table with horizontal scroll

### Mobile (< 768px)
- Wallet grid: 1 column (full width)
- Main sections: Stack vertically
- Transaction table condensed with horizontal scroll

## Color Scheme Integration

### Badges by Transaction Type:
- **TRANSFER**: Cyan badge (#06B6D4)
- **BURN**: Pink badge (#EC4899)
- **NODE_REWARD**: Purple badge (#9333EA)
- **SMART_CONTRACT**: Gray badge

### Card Styling:
- **Balance Card**: Purple glow (nfm-glass-card--glow-purple)
- **Staking Card**: Pink glow (nfm-glass-card--glow-pink)
- **Portfolio Card**: Default glass (nfm-glass-card)
- **Wallet Selector**: Cyan badge (nfm-badge--cyan)

### Status Indicators:
- Active Wallet: Green status dot (text-success) with "ACTIVE" badge
- Online Status: Blue/green dot
- Syncing Status: Yellow dot

## Interactive Elements

### Wallet Dropdown Menu
```
Connected: nfm_abcd... ▼
├─ ● Wallet 1 (nfm_abcd...)     ← Hover highlights
├─ ● Wallet 2 [ACTIVE] (nfm_efgh...)
└─ ● Wallet 3 (nfm_ijkl...)
```

### Portfolio Asset Interaction
- **Click NVC**: Activates NVC ↔ ETH swap panel
- **Click ETH**: Activates ETH ↔ NVC swap panel
- Visual feedback: hover background change

### Button Actions
- **[Receive]**: Copy address to clipboard
- **[Send]**: Prompt for recipient address and amount
- **[@Alias Transfer]**: Prompt for alias (@username) and amount
- **[Stake NVC]**: Prompt for stake amount
- **[Unstake]**: Withdraw staked NVC
- **[Swap]**: Execute NVC/ETH conversion

### Transaction Filtering
```
Filter: [All Types ▼]
├─ All Types (default)
├─ Transfers
├─ Burns
├─ Node Rewards
└─ Smart Contracts
```

## Modal: New Wallet Creation

```
╔════════════════════════════════════════╗
║ ✨ New Identity Generated         [✕] ║
╟────────────────────────────────────────╢
║ ⚠️  CRITICAL: Save your Private Key   ║
║    now. It will not be shown again    ║
║    and cannot be recovered if lost.   ║
║                                        ║
║ PUBLIC ADDRESS                         ║
║ ┌────────────────────────────────────┐ ║
║ │ nfm_a1b2c3d4e5f6g7h8i9j0k1l2m3... │ ║
║ └────────────────────────────────────┘ ║
║                                        ║
║ PRIVATE KEY (Ed25519 Hex)             ║
║ ┌────────────────────────────────────┐ ║
║ │ •••••••••••••••••••••••••••••••• │[📋]║
║ └────────────────────────────────────┘ ║
║                                        ║
║                    [I have saved my keys]║
╚════════════════════════════════════════╝
```

## State Flow Diagram

```
┌─────────────────────┐
│  Initial Load       │
│  data.wallets       │
│  data.transactions  │
└──────────┬──────────┘
           │
           ▼
   ┌──────────────────────┐
   │ User selects wallet  │
   │ setSelectedWallet()  │
   └──────────┬───────────┘
              │
              ▼
   ┌──────────────────────────────┐
   │ CURRENT_WALLET updates       │
   │ (balance, address display)   │
   └────────────────────────────────┘
           │
           ├─────────────────────────────────────────┐
           │                                         │
           ▼                                         ▼
   ┌──────────────────┐                  ┌────────────────┐
   │ User exchanges   │                  │ User filters   │
   │ setSwapMode()    │                  │ setTransFilter │
   └─────────────────┘                      └────────────────┘
```

## Key Features Breakdown

### 1. Multi-Wallet Display
```typescript
{WALLETS.length > 0 && (
  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {WALLETS.map((w) => (
      // Each wallet is a clickable card
    ))}
  </div>
)}
```

### 2. Dynamic Balance Display
```typescript
const CURRENT_WALLET = ACTIVE_WALLET ? {
  nfmAddress: ACTIVE_WALLET.address,
  balanceNVC: ACTIVE_WALLET.balanceNVC,
  balanceETH: ACTIVE_WALLET.balanceETH,
} : {
  nfmAddress: DUMMY_USER.nfmAddress,
  balanceNVC: DUMMY_USER.balance,
  balanceETH: 0,
};
```

### 3. Transaction Filtering
```typescript
DUMMY_TRANSACTIONS
  .filter(tx => transactionFilter === 'ALL' || tx.type === transactionFilter)
  .map((tx) => ...)
```

### 4. Asset Swap Toggle
```typescript
{swapMode && (
  <div className="mt-4 pt-4 border-t border-white/10">
    <button onClick={() => handleSwap(...)}>
      Swap {swapMode === 'NVC_TO_ETH' ? 'NVC → ETH' : 'ETH → NVC'}
    </button>
  </div>
)}
```

## Accessibility Considerations

- Semantic HTML structure
- Proper button elements for all actions
- Clear visual hierarchy with heading tags
- Sufficient color contrast for badges
- Keyboard navigation support via tab
- ARIA labels on interactive elements (future enhancement)
- Responsive text sizing for readability

## Performance Optimizations

1. **Lazy Rendering**: Transaction table only filters on display
2. **Efficient Updates**: useState hooks for minimal re-renders
3. **CSS Grid**: Native layout without extra computation
4. **Memoization Ready**: Can add React.memo() for wallet cards
5. **Event Delegation**: Button clicks handled at component level

---

## Testing Coverage Map

| Feature | Test Type | Status |
|---------|-----------|--------|
| Wallet List Display | Visual | ✅ Compiled |
| Wallet Switching | Functional | ✅ State Handler Ready |
| Balance Display Update | Functional | ✅ Dynamic compute |
| Send/Receive | Functional | ✅ Handler Ready |
| Alias Transfer | Functional | ✅ Handler Ready |
| Swap Toggle | Functional | ✅ State Handler Ready |
| Staking | Functional | ✅ Handler Ready |
| Transaction Filter | Functional | ✅ Array filter logic |
| Responsive Layout | Visual | ✅ Grid-based |
| Modal Creation | Functional | ✅ Modal handler ready |

---

Last Updated: April 1, 2026
