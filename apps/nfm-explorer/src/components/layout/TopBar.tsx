import { useState, useRef, useEffect } from 'react';
import { Search, Bell, Menu, Wallet, Copy, Plus, LogOut, ChevronDown, ExternalLink, Check, X } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../../context/AppDataContext';

const TopBar = () => {
  const navigate = useNavigate();
  const { data, refreshPaused, notifySuccess, notifyError } = useAppData();
  const DUMMY_STATUS = data.status;
  const DUMMY_USER = data.user_profile;
  const DUMMY_WALLETS = data.wallets;

  const [isWalletOpen, setIsWalletOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const walletRef = useRef<HTMLDivElement>(null);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Close on outside click
  useEffect(() => {
    const handleOutsideClick = (event: MouseEvent) => {
      if (walletRef.current && !walletRef.current.contains(event.target as Node)) {
        setIsWalletOpen(false);
      }
    };
    if (isWalletOpen) {
      document.addEventListener('mousedown', handleOutsideClick);
    }
    return () => document.removeEventListener('mousedown', handleOutsideClick);
  }, [isWalletOpen]);

  const submitSearch = () => {
    const query = searchQuery.trim();
    if (!query) {
      return;
    }
    sessionStorage.setItem('nfm.explorer.searchQuery', query);
    sessionStorage.setItem('nfm.marketplace.searchQuery', query);
    navigate('/explorer');
    notifySuccess(`Search applied: ${query}`);
  };

  const openAddressInExplorer = () => {
    sessionStorage.setItem('nfm.explorer.searchQuery', DUMMY_USER.nfmAddress);
    navigate('/explorer');
  };

  return (
    <header className="nfm-topbar">
      <div className="nfm-topbar__left">
        <button className="nfm-btn nfm-btn--ghost nfm-btn--sm hide-desktop">
          <Menu size={20} />
        </button>
        
        <div className="nfm-search nfm-topbar__search">
          <Search className="nfm-search__icon" size={18} />
          <input 
            type="text" 
            className="nfm-search__input" 
            placeholder="Search blocks, addresses, or assets..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                submitSearch();
              }
            }}
          />
        </div>
      </div>

      <div className="nfm-topbar__right">
        <div className="flex items-center gap-4 mr-4 hide-mobile">
          <div className="flex items-center gap-2 px-3 py-1 bg-surface-lowest-40 border border-white-05 rounded-full">
            <div className={`w-1.5 h-1.5 rounded-full ${!refreshPaused ? 'bg-success animate-pulse shadow-glow-success' : 'bg-warning'}`}></div>
            <span className={`text-9px font-bold uppercase tracking-widest ${!refreshPaused ? 'text-success' : 'text-warning'}`}>
              {!refreshPaused ? 'Telemetry Live' : 'Refresh Paused'}
            </span>
          </div>
          <div className="nfm-node-status">
            <div className={`nfm-status-dot nfm-status-dot--${DUMMY_STATUS.status.toLowerCase()}`}></div>
            <span className="status-text">{DUMMY_STATUS.node}</span>
          </div>
        </div>

        {/* Solana-Style Wallet Dropdown Container */}
        <div className="nfm-wallet-dropdown-container" ref={walletRef}>
          <button 
            className="nfm-wallet-btn"
            onClick={() => setIsWalletOpen(prev => !prev)}
          >
            <div className="p-1 px-2 bg-surface-highest rounded-full text-purple">
              <Wallet size={14} />
            </div>
            <div className="nfm-wallet-btn__info hide-mobile">
              <span className="nfm-wallet-btn__balance">{DUMMY_USER.balance.toLocaleString()} NVC</span>
              <span className="nfm-wallet-btn__address">{DUMMY_USER.nfmAddress.substring(0, 6)}...{DUMMY_USER.nfmAddress.slice(-4)}</span>
            </div>
            <ChevronDown size={14} className={`text-muted transition-transform ${isWalletOpen ? 'rotate-180' : ''}`} />
          </button>

          {/* Wallet Profile Dropdown */}
          {isWalletOpen && (
            <div className="nfm-wallet-dropdown">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-sm font-bold flex items-center gap-2">
                  <Wallet className="text-purple" size={16} /> Wallet Profile
                </h2>
                <button className="text-muted hover:text-white" onClick={() => setIsWalletOpen(false)}>
                  <X size={16} />
                </button>
              </div>

              <div className="flex-col gap-5">
                {/* Profile Header */}
                <div className="p-5 bg-surface-lowest rounded-lg border border-white-05 text-center relative overflow-hidden">
                  <div className="absolute top-0 right-0 p-2">
                    <div className="nfm-badge nfm-badge--success" style={{fontSize: '8px', padding: '1px 6px'}}>Active</div>
                  </div>
                  <div className="w-12 h-12 bg-gradient-to-br from-purple-600 to-cyan-400 rounded-xl mx-auto mb-3 flex items-center justify-center text-xl font-bold border border-white-10 shadow-lg">
                    {DUMMY_USER.username.substring(1, 2).toUpperCase()}
                  </div>
                  <h3 className="text-base font-bold mb-1">{DUMMY_USER.username}</h3>
                  <div className="flex items-center justify-center gap-2 text-muted text-xs mb-5">
                    <span className="font-mono">{DUMMY_USER.nfmAddress.substring(0, 12)}...</span>
                    <button 
                      className="p-1 hover:text-cyan transition-colors"
                      onClick={() => copyToClipboard(DUMMY_USER.nfmAddress)}
                    >
                      {copied ? <Check size={12} className="text-success" /> : <Copy size={12} />}
                    </button>
                    <button className="p-1 hover:text-cyan transition-colors" onClick={openAddressInExplorer}>
                      <ExternalLink size={12} className="cursor-pointer hover:text-cyan" />
                    </button>
                  </div>

                  <div className="grid grid-cols-2 gap-2">
                    <div className="p-2 bg-surface-low rounded-md border border-white-05 text-left">
                      <div className="text-[10px] text-muted uppercase tracking-widest mb-1">NVC Balance</div>
                      <div className="text-cyan font-bold text-xs">{DUMMY_USER.balance.toLocaleString()}</div>
                    </div>
                    <div className="p-2 bg-surface-low rounded-md border border-white-05 text-left">
                      <div className="text-[10px] text-muted uppercase tracking-widest mb-1">Network Pos</div>
                      <div className="text-purple font-bold text-xs">Founder</div>
                    </div>
                  </div>
                </div>

                {/* Wallet Switcher */}
                <div>
                  <div className="flex justify-between items-center mb-3 px-1">
                    <span className="text-[10px] font-bold text-muted uppercase tracking-wider">Your Wallets</span>
                    <button className="text-cyan text-[10px] flex items-center gap-1 hover:underline" onClick={() => notifyError('Add wallet flow is not available from backend yet')}>
                      <Plus size={10} /> Add Wallet
                    </button>
                  </div>
                  <div className="flex-col gap-2">
                    {DUMMY_WALLETS.map(w => (
                      <div key={w.address} className={`nfm-portfolio-item ${w.isActive ? 'border-purple-30 bg-surface-container' : 'opacity-60'}`} style={{padding: 'var(--space-2) var(--space-3)'}}>
                        <div className="nfm-portfolio-item__info">
                          <div className="nfm-portfolio-item__icon" style={{width: '24px', height: '24px', fontSize: '10px'}}>{w.name.substring(0, 1)}</div>
                          <div>
                            <div className="font-bold text-[10px]">{w.name}</div>
                            <div className="text-[8px] text-muted">{w.address.substring(0, 8)}...</div>
                          </div>
                        </div>
                        <div className="text-right">
                          <div className="text-[10px] font-bold">{w.balanceNVC.toLocaleString()} NVC</div>
                          <div className="text-[8px] text-muted">{w.balanceETH} ETH</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Footer Actions */}
                <button className="nfm-btn nfm-btn--danger w-full nfm-btn--sm" onClick={() => { setIsWalletOpen(false); notifySuccess('Wallet UI disconnected'); }}>
                  <LogOut size={14} /> Disconnect Wallet
                </button>
              </div>
            </div>
          )}
        </div>

        <button className="nfm-btn nfm-btn--ghost nfm-btn--sm notification-btn">
          <Bell size={20} />
          <span className="notification-badge"></span>
        </button>
      </div>
    </header>
  );
};

export default TopBar;
