import { ArrowUpRight, ArrowDownLeft, History, ArrowRight, Plus, Copy, Check, X } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';
import { appTransfer, appCreateWallet } from '../api/client';
import { useState } from 'react';

const Wallet = () => {
  const navigate = useNavigate();
  const { data, refresh, requestPrompt, notifySuccess, notifyError } = useAppData();
  const [newWallet, setNewWallet] = useState<{ address: string; private_key: string } | null>(null);
  const [copied, setCopied] = useState(false);
  
  const DUMMY_USER = data.user_profile;
  const DUMMY_TRANSACTIONS = data.transactions;

  const handleReceive = async () => {
    const address = DUMMY_USER.nfmAddress;
    try {
      await navigator.clipboard.writeText(address);
      notifySuccess('Address copied to clipboard');
    } catch {
      notifySuccess(`Your receive address: ${address}`);
    }
  };

  const handleOpenLedger = () => {
    sessionStorage.setItem('nfm_explorer_query', DUMMY_USER.nfmAddress);
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
      await appTransfer(to, amount, DUMMY_USER.nfmAddress);
      await refresh();
      notifySuccess('Transfer success');
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Transfer failed');
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
      <div className="flex items-center justify-between mb-8 gap-4 wrap">
        <div>
          <h1 className="text-purple">Wallet Management</h1>
          <p className="text-xs text-muted mt-1">Manage core NFM identity and asset transfers.</p>
        </div>
        <div className="flex items-center gap-3">
          <button className="nfm-btn nfm-btn--secondary" style={{ height: '38px', padding: '0 16px' }} onClick={handleCreateWallet}>
            <Plus size={16} /> Create New Wallet
          </button>
          <div className="nfm-badge nfm-badge--cyan">
            <div className="nfm-badge__dot"></div>
            <span className="hide-mobile">Connected:</span> {DUMMY_USER.nfmAddress.substring(0, 10)}...
          </div>
        </div>
      </div>

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        {/* Main Balance Card */}
        <div className="nfm-glass-card nfm-glass-card--glow-purple nfm-wallet-balance-card" style={{ flex: '1 1 400px' }}>
          <div className="text-muted text-xs uppercase tracking-widest mb-2 font-semibold">Vault Equilibrium</div>
          <div className="flex items-baseline gap-3 mb-8">
            <span className="font-display text-5xl font-bold">{DUMMY_USER.balance.toLocaleString('en-US')}</span>
            <span className="text-xl text-cyan font-bold tracking-tighter">NVC</span>
          </div>
          
          <div className="flex gap-4">
            <button className="nfm-btn nfm-btn--primary" style={{ flex: 1 }} onClick={() => void handleReceive()}>
              <ArrowDownLeft size={16} /> Receive
            </button>
            <button className="nfm-btn nfm-btn--secondary" style={{ flex: 1 }} onClick={handleSend}>
              <ArrowUpRight size={16} /> Send
            </button>
          </div>
        </div>

        {/* Portfolio Stats */}
        <div className="nfm-glass-card" style={{ flex: '1 1 300px' }}>
          <h3 className="text-lg mb-6">Asset Portfolio</h3>
          <div className="flex-col gap-3">
            <div className="nfm-portfolio-item">
              <div className="nfm-portfolio-item__info">
                <div className="nfm-portfolio-item__icon">NVC</div>
                <div>
                  <div className="font-bold text-sm">Neural Vault Coin</div>
                  <div className="text-10px text-muted tracking-wide uppercase">Core Intelligence Asset</div>
                </div>
              </div>
              <div className="font-mono text-cyan text-sm">{DUMMY_USER.balance.toLocaleString()}</div>
            </div>
            
            <div className="nfm-portfolio-item opacity-60">
              <div className="nfm-portfolio-item__info">
                <div className="nfm-portfolio-item__icon" style={{color: 'var(--hyper-pink)'}}>ETH</div>
                <div>
                  <div className="font-bold text-sm">Ethereum</div>
                  <div className="text-10px text-muted tracking-wide uppercase">L1 Settlement</div>
                </div>
              </div>
              <div className="font-mono text-sm">0.00</div>
            </div>
          </div>
        </div>
      </div>

      {/* Transaction History */}
      <div className="nfm-glass-card mt-8">
        <div className="flex items-center gap-2 mb-6 text-xl">
          <History className="text-cyan" /> <h3>Recent Transactions</h3>
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
            {DUMMY_TRANSACTIONS.map((tx) => (
              <tr key={tx.txid}>
                <td>
                  <span className={`nfm-badge nfm-badge--${tx.type === 'TRANSFER' ? 'cyan' : tx.type === 'BURN' ? 'pink' : 'purple'}`}>
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
