import { 
  ArrowUpRight, ArrowDownLeft, History, ArrowRight, Plus, Copy, Check, X, Lock, TrendingUp, 
  Repeat, Filter, ChevronDown, Edit2, Heart, Tag, Gift, DollarSign
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';
import { appTransfer, appCreateWallet } from '../api/client';
import { useState } from 'react';

const Wallet = () => {
  const navigate = useNavigate();
  const { data, refresh, requestPrompt, notifySuccess, notifyError } = useAppData();
  const [newWallet, setNewWallet] = useState<{ address: string; private_key: string } | null>(null);
  const [copied, setCopied] = useState(false);
  const [selectedWallet, setSelectedWallet] = useState<string | null>(null);
  const [showWalletMenu, setShowWalletMenu] = useState(false);
  const [transactionFilter, setTransactionFilter] = useState<'ALL' | 'TRANSFER' | 'BURN' | 'SMART_CONTRACT' | 'NODE_REWARD'>('ALL');
  const [swapMode, setSwapMode] = useState<'NVC_TO_ETH' | 'ETH_TO_NVC' | null>(null);
  const [walletAliases, setWalletAliases] = useState<Record<string, string>>({});
  
  const DUMMY_USER = data.user_profile;
  const DUMMY_TRANSACTIONS = data.transactions;
  const WALLETS = data.wallets.length > 0 ? data.wallets : [];
  const ACTIVE_WALLET = selectedWallet ? WALLETS.find(w => w.address === selectedWallet) : null;
  const CURRENT_WALLET = ACTIVE_WALLET ? {
    nfmAddress: ACTIVE_WALLET.address,
    balanceNVC: ACTIVE_WALLET.balanceNVC,
    balanceETH: ACTIVE_WALLET.balanceETH,
  } : {
    nfmAddress: DUMMY_USER.nfmAddress,
    balanceNVC: DUMMY_USER.balance,
    balanceETH: 0,
  };

  const handleReceive = async () => {
    const address = CURRENT_WALLET.nfmAddress;
    try {
      await navigator.clipboard.writeText(address);
      notifySuccess('Address copied to clipboard');
    } catch {
      notifySuccess(`Your receive address: ${address}`);
    }
  };

  const handleOpenLedger = () => {
    const address = CURRENT_WALLET.nfmAddress;
    sessionStorage.setItem('nfm_explorer_query', address);
    navigate('/explorer');
  };

  const handleSend = async () => {
    const to = await requestPrompt({
      title: 'Send NVC',
      message: 'Target address (nfm_...)',
      placeholder: 'nfm_xxxxx',
      confirmText: 'Next',
    });
    if (!to) return;

    const amountRaw = await requestPrompt({
      title: 'Send NVC',
      message: 'Amount NVC',
      placeholder: '10',
      confirmText: 'Send',
    });
    if (!amountRaw) return;
    const amount = Number(amountRaw);
    if (!Number.isFinite(amount) || amount <= 0) {
      notifyError('Invalid amount');
      return;
    }

    try {
      await appTransfer(to, amount, CURRENT_WALLET.nfmAddress);
      await refresh();
      notifySuccess('Transfer success');
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Transfer failed');
    }
  };

  const handleAliasTransfer = async () => {
    const alias = await requestPrompt({
      title: 'Send via Alias',
      message: 'Enter @alias (e.g. @alice)',
      placeholder: '@alias',
      confirmText: 'Next',
    });
    if (!alias) return;

    const amountRaw = await requestPrompt({
      title: 'Send via Alias',
      message: 'Amount NVC',
      placeholder: '10',
      confirmText: 'Send',
    });
    if (!amountRaw) return;
    const amount = Number(amountRaw);
    if (!Number.isFinite(amount) || amount <= 0) {
      notifyError('Invalid amount');
      return;
    }

    try {
      notifySuccess(`Transfer via ${alias} queued for AI verification`);
      await refresh();
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Alias transfer failed');
    }
  };

  const handleStake = async () => {
    const amountRaw = await requestPrompt({
      title: 'Stake NVC',
      message: 'Amount to stake',
      placeholder: '100',
      confirmText: 'Stake',
    });
    if (!amountRaw) return;
    const amount = Number(amountRaw);
    if (!Number.isFinite(amount) || amount <= 0) {
      notifyError('Invalid amount');
      return;
    }

    if (amount > CURRENT_WALLET.balanceNVC) {
      notifyError('Insufficient balance');
      return;
    }

    try {
      notifySuccess(`Staked ${amount} NVC successfully`);
      await refresh();
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Staking failed');
    }
  };

  const handleSwap = async (from: 'NVC' | 'ETH') => {
    const fromBalance = from === 'NVC' ? CURRENT_WALLET.balanceNVC : CURRENT_WALLET.balanceETH;
    const amountRaw = await requestPrompt({
      title: `Swap ${from}`,
      message: `Amount ${from} to swap`,
      placeholder: '10',
      confirmText: 'Swap',
    });
    if (!amountRaw) return;
    const amount = Number(amountRaw);
    if (!Number.isFinite(amount) || amount <= 0) {
      notifyError('Invalid amount');
      return;
    }

    if (amount > fromBalance) {
      notifyError('Insufficient balance');
      return;
    }

    try {
      const to = from === 'NVC' ? 'ETH' : 'NVC';
      notifySuccess(`Swapped ${amount} ${from} to ${to}`);
      await refresh();
      setSwapMode(null);
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Swap failed');
    }
  };

  const handleCreateWallet = async () => {
    try {
      const res = await appCreateWallet();
      setNewWallet({
        address: res.address,
        private_key: res.private_key,
      });
      await refresh();
      notifySuccess('New wallet created and registered');
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Failed to create wallet');
    }
  };

  const handleCopyPrivateKey = async () => {
    if (!newWallet) return;
    try {
      await navigator.clipboard.writeText(newWallet.private_key);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      notifySuccess('Private key copied to clipboard');
    } catch {
      notifyError('Failed to copy to clipboard');
    }
  };

  return (
    <div className="animate-in">
      {/* Header with Wallet Selector */}
      <div className="flex items-center justify-between mb-8 gap-4 wrap">
        <div>
          <h1 className="text-purple">Wallet Management</h1>
          <p className="text-xs text-muted mt-1">Manage NFM identities and asset transfers.</p>
        </div>
        <div className="flex items-center gap-3">
          <button className="nfm-btn nfm-btn--secondary" style={{ height: '38px', padding: '0 16px' }} onClick={handleCreateWallet}>
            <Plus size={16} /> Create New Wallet
          </button>
          
          {/* Wallet Selector Dropdown */}
          <div className="relative">
            <button 
              className="nfm-badge nfm-badge--cyan flex items-center gap-2"
              onClick={() => setShowWalletMenu(!showWalletMenu)}
            >
              <div className="nfm-badge__dot"></div>
              <span className="hide-mobile">Connected:</span> {CURRENT_WALLET.nfmAddress.substring(0, 10)}...
              <ChevronDown size={14} />
            </button>
            
            {showWalletMenu && WALLETS.length > 0 && (
              <div className="absolute top-full right-0 mt-2 bg-nfm-dark border border-white/10 rounded-lg shadow-lg z-50 min-w-max">
                {WALLETS.map((w) => (
                  <button
                    key={w.address}
                    onClick={() => {
                      setSelectedWallet(w.address);
                      setShowWalletMenu(false);
                    }}
                    className={`block w-full text-left px-4 py-2 text-sm ${
                      selectedWallet === w.address ? 'bg-white/10 text-cyan' : 'text-white hover:bg-white/5'
                    }`}
                  >
                    <span className={w.isActive ? 'text-success' : 'text-muted'}>●</span> {w.name || 'Wallet'} ({w.address.substring(0, 8)}...)
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Multi-Wallet List */}
      {WALLETS.length > 0 && (
        <div className="nfm-glass-card mb-8">
          <h3 className="text-lg mb-4">Your Wallets</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {WALLETS.map((w) => (
              <div 
                key={w.address}
                onClick={() => setSelectedWallet(w.address)}
                className={`p-4 rounded-lg border-2 cursor-pointer transition-all ${
                  selectedWallet === w.address 
                    ? 'border-cyan bg-cyan/10' 
                    : 'border-white/10 hover:border-cyan/50'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="text-xs font-semibold text-muted">{w.name || 'Wallet'}</div>
                  {w.isActive && <span className="text-10px bg-success/20 text-success px-2 py-1 rounded">ACTIVE</span>}
                </div>
                <div className="font-mono text-10px text-muted mb-2 truncate">{w.address}</div>
                <div className="flex justify-between items-baseline">
                  <div className="text-lg font-bold text-cyan">{w.balanceNVC.toLocaleString()}</div>
                  <div className="text-xs text-muted">NVC</div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        {/* Main Balance Card */}
        <div className="nfm-glass-card nfm-glass-card--glow-purple nfm-wallet-balance-card" style={{ flex: '1 1 400px' }}>
          <div className="text-muted text-xs uppercase tracking-widest mb-2 font-semibold">Vault Equilibrium</div>
          <div className="flex items-baseline gap-3 mb-8">
            <span className="font-display text-5xl font-bold">{CURRENT_WALLET.balanceNVC.toLocaleString('en-US')}</span>
            <span className="text-xl text-cyan font-bold tracking-tighter">NVC</span>
          </div>
          
          <div className="flex gap-4 wrap">
            <button className="nfm-btn nfm-btn--primary" style={{ flex: 1, minWidth: '120px' }} onClick={() => void handleReceive()}>
              <ArrowDownLeft size={16} /> Receive
            </button>
            <button className="nfm-btn nfm-btn--secondary" style={{ flex: 1, minWidth: '120px' }} onClick={handleSend}>
              <ArrowUpRight size={16} /> Send
            </button>
            <button className="nfm-btn nfm-btn--secondary" style={{ flex: 1, minWidth: '120px' }} onClick={handleAliasTransfer}>
              @Alias Transfer
            </button>
          </div>
        </div>

        {/* Portfolio Stats */}
        <div className="nfm-glass-card" style={{ flex: '1 1 300px' }}>
          <h3 className="text-lg mb-6">Asset Portfolio</h3>
          <div className="flex-col gap-3">
            <div className="nfm-portfolio-item cursor-pointer hover:bg-white/5 p-2 rounded" onClick={() => setSwapMode(swapMode === 'NVC_TO_ETH' ? null : 'NVC_TO_ETH')}>
              <div className="nfm-portfolio-item__info">
                <div className="nfm-portfolio-item__icon">NVC</div>
                <div>
                  <div className="font-bold text-sm">Neural Vault Coin</div>
                  <div className="text-10px text-muted tracking-wide uppercase">Core Intelligence Asset</div>
                </div>
              </div>
              <div className="font-mono text-cyan text-sm">{CURRENT_WALLET.balanceNVC.toLocaleString()}</div>
            </div>
            
            <div className="nfm-portfolio-item cursor-pointer hover:bg-white/5 p-2 rounded opacity-80" onClick={() => setSwapMode(swapMode === 'ETH_TO_NVC' ? null : 'ETH_TO_NVC')}>
              <div className="nfm-portfolio-item__info">
                <div className="nfm-portfolio-item__icon" style={{color: 'var(--hyper-pink)'}}>ETH</div>
                <div>
                  <div className="font-bold text-sm">Ethereum</div>
                  <div className="text-10px text-muted tracking-wide uppercase">L1 Settlement</div>
                </div>
              </div>
              <div className="font-mono text-sm">{CURRENT_WALLET.balanceETH.toFixed(4)}</div>
            </div>

            {swapMode && (
              <div className="mt-4 pt-4 border-t border-white/10">
                <div className="flex items-center gap-2 mb-3">
                  <Repeat size={16} className="text-cyan" />
                  <span className="text-sm font-semibold">Quick Swap</span>
                </div>
                <button 
                  className="nfm-btn nfm-btn--secondary w-full"
                  onClick={() => handleSwap(swapMode === 'NVC_TO_ETH' ? 'NVC' : 'ETH')}
                >
                  Swap {swapMode === 'NVC_TO_ETH' ? 'NVC → ETH' : 'ETH → NVC'}
                </button>
              </div>
            )}
          </div>
        </div>

        {/* Staking Panel */}
        <div className="nfm-glass-card nfm-glass-card--glow-pink" style={{ flex: '1 1 300px' }}>
          <div className="flex items-center gap-2 mb-6">
            <Lock size={20} className="text-pink" />
            <h3 className="text-lg">Staking Vault</h3>
          </div>
          <div className="flex-col gap-4">
            <div className="bg-white/5 p-4 rounded-lg">
              <div className="text-10px text-muted uppercase tracking-widest mb-1">APY</div>
              <div className="text-2xl font-bold text-pink">12.5%</div>
            </div>
            <div className="bg-white/5 p-4 rounded-lg">
              <div className="text-10px text-muted uppercase tracking-widest mb-1">Staked NVC</div>
              <div className="text-xl font-bold text-success">250,000</div>
            </div>
            <button className="nfm-btn nfm-btn--primary w-full" onClick={handleStake}>
              <TrendingUp size={16} /> Stake NVC
            </button>
            <button className="nfm-btn nfm-btn--ghost w-full">
              Unstake
            </button>
          </div>
        </div>
      </div>

      {/* Transaction History with Filter */}
      <div className="nfm-glass-card mt-8">
        <div className="flex items-center justify-between gap-4 mb-6 wrap">
          <div className="flex items-center gap-2 text-xl">
            <History className="text-cyan" /> <h3>Recent Transactions</h3>
          </div>
          <div className="flex items-center gap-2">
            <Filter size={16} className="text-muted" />
            <select 
              value={transactionFilter}
              onChange={(e) => setTransactionFilter(e.target.value as any)}
              className="nfm-input text-sm p-2"
              style={{ width: '140px' }}
            >
              <option value="ALL">All Types</option>
              <option value="TRANSFER">Transfers</option>
              <option value="BURN">Burns</option>
              <option value="NODE_REWARD">Rewards</option>
              <option value="SMART_CONTRACT">Contracts</option>
            </select>
          </div>
        </div>
        
        <table className="nfm-table">
          <thead>
            <tr>
              <th>Type</th>
              <th>Status</th>
              <th>TX Hash</th>
              <th>Amount</th>
              <th>Time</th>
            </tr>
          </thead>
          <tbody>
            {DUMMY_TRANSACTIONS
              .filter(tx => transactionFilter === 'ALL' || tx.type === transactionFilter)
              .map((tx) => (
              <tr key={tx.txid}>
                <td>
                  <span className={`nfm-badge nfm-badge--${
                    tx.type === 'TRANSFER' ? 'cyan' : 
                    tx.type === 'BURN' ? 'pink' : 
                    tx.type === 'NODE_REWARD' ? 'purple' : 
                    'gray'
                  }`}>
                    {tx.type}
                  </span>
                </td>
                <td>
                  <div className="nfm-tx-status">
                    <span className={`nfm-status-dot nfm-status-dot--${tx.status === 'CONFIRMED' ? 'online' : 'syncing'}`}></span>
                    {tx.status}
                  </div>
                </td>
                <td className="font-mono text-xs text-muted">{tx.txid.substring(0, 16)}...</td>
                <td className={`font-mono text-xs font-bold ${tx.type === 'BURN' ? 'text-pink' : 'text-success'}`}>
                  {tx.type === 'BURN' ? '-' : '+'}{tx.amount.toFixed(2)} NVC
                </td>
                <td className="text-muted text-xs">
                  {Math.floor((Date.now() - tx.timestamp) / 3600000)}h ago
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <button className="nfm-btn-more" onClick={handleOpenLedger}>
          <ArrowRight size={14} /> Full Transaction Ledger
        </button>
      </div>

      {newWallet && (
        <div className="nfm-modal-overlay" onClick={() => setNewWallet(null)}>
          <div className="nfm-modal animate-in" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '500px' }}>
            <div className="nfm-modal__header">
              <h3 className="nfm-modal__title text-cyan flex items-center gap-2">
                <Plus size={20} /> New Identity Generated
              </h3>
              <button className="nfm-modal-close" onClick={() => setNewWallet(null)}>
                <X size={18} />
              </button>
            </div>
            
            <div className="flex-col gap-6 p-2">
              <div className="nfm-alert nfm-alert--warning" style={{ background: 'rgba(245, 158, 11, 0.08)', borderColor: 'rgba(245, 158, 11, 0.2)' }}>
                <p className="text-xs" style={{ color: 'var(--warning)' }}>
                  <strong>CRITICAL:</strong> Save your Private Key now. It will not be shown again and cannot be recovered if lost.
                </p>
              </div>

              <div>
                <label className="text-10px uppercase tracking-widest text-muted block mb-2">Public Address</label>
                <div className="flex items-center gap-2 nfm-input-group">
                  <input className="nfm-input font-mono text-xs" value={newWallet.address} readOnly />
                </div>
              </div>

              <div>
                <label className="text-10px uppercase tracking-widest text-muted block mb-2">Private Key (Ed25519 Hex)</label>
                <div className="flex items-center gap-2 nfm-input-group">
                  <input 
                    type="password" 
                    className="nfm-input font-mono text-xs" 
                    value={newWallet.private_key} 
                    readOnly 
                    style={{ letterSpacing: '0.1em' }}
                  />
                  <button className="nfm-btn nfm-btn--ghost" onClick={handleCopyPrivateKey} style={{ width: '40px', padding: 0 }}>
                    {copied ? <Check size={16} className="text-success" /> : <Copy size={16} />}
                  </button>
                </div>
              </div>

              <div className="mt-4 pt-4 border-t border-white/5 flex justify-end">
                <button className="nfm-btn nfm-btn--primary" onClick={() => setNewWallet(null)}>
                  I have saved my keys
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Wallet;
