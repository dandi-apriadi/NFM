import { useEffect, useMemo, useState } from 'react';
import { Search, Activity, Server, Cpu, Globe, Zap, ShieldCheck, X, ArrowRight, Hourglass, Flame, Trophy, TrendingDown } from 'lucide-react';
import type { Block } from '../types';
import { useAppData } from '../context/AppDataContext';

const EXPLORER_SEARCH_KEY = 'nfm.explorer.searchQuery';

const formatAgo = (timestampMs: number) => {
  const sec = Math.max(0, Math.floor((Date.now() - timestampMs) / 1000));
  if (sec < 60) return `${sec}s ago`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m ago`;
  const hour = Math.floor(min / 60);
  return `${hour}h ago`;
};

const truncateHash = (hash: string, length = 8) => {
  if (!hash) return '';
  if (hash.length <= length * 2) return hash;
  return `${hash.slice(0, length)}...${hash.slice(-length)}`;
};

const Explorer = () => {
  const { data, p2p } = useAppData();
  const HALVING_INTERVAL = 420000;
  const blocksToHalving = HALVING_INTERVAL - (data.status.blocks % HALVING_INTERVAL);
  const halvingProgressPct = ((data.status.blocks % HALVING_INTERVAL) / HALVING_INTERVAL) * 100;

  const DUMMY_BLOCKS = data.blocks;
  const DUMMY_TRANSACTIONS = data.transactions;

  const [selectedBlock, setSelectedBlock] = useState<Block | null>(null);
  const [searchQuery, setSearchQuery] = useState(() => sessionStorage.getItem(EXPLORER_SEARCH_KEY) || '');

  useEffect(() => {
    sessionStorage.setItem(EXPLORER_SEARCH_KEY, searchQuery);
  }, [searchQuery]);

  const normalizedSearch = searchQuery.trim().toLowerCase();

  const filteredBlocks = useMemo(() => {
    if (!normalizedSearch) {
      return DUMMY_BLOCKS;
    }

    return DUMMY_BLOCKS.filter((block) => {
      return (
        block.hash.toLowerCase().includes(normalizedSearch) ||
        block.miner.toLowerCase().includes(normalizedSearch) ||
        block.index.toString().includes(normalizedSearch)
      );
    });
  }, [DUMMY_BLOCKS, normalizedSearch]);

  const filteredTransactions = useMemo(() => {
    if (!normalizedSearch) {
      return DUMMY_TRANSACTIONS;
    }

    return DUMMY_TRANSACTIONS.filter((tx) => {
      return (
        tx.txid.toLowerCase().includes(normalizedSearch) ||
        tx.from.toLowerCase().includes(normalizedSearch) ||
        tx.to.toLowerCase().includes(normalizedSearch) ||
        tx.type.toLowerCase().includes(normalizedSearch)
      );
    });
  }, [DUMMY_TRANSACTIONS, normalizedSearch]);

  const avgTxPerBlock = useMemo(() => {
    if (DUMMY_BLOCKS.length === 0) return 0;
    const sample = DUMMY_BLOCKS.slice(0, 8);
    const sum = sample.reduce((acc, b) => acc + Number(b.transactions || 0), 0);
    return sum / sample.length;
  }, [DUMMY_BLOCKS]);

  const blocks24h = useMemo(
    () => DUMMY_BLOCKS.filter((b) => Date.now() - b.timestamp <= 24 * 60 * 60 * 1000).length,
    [DUMMY_BLOCKS],
  );

  const totalWalletBalance = useMemo(
    () => data.wallets.reduce((sum, w) => sum + Number(w.balanceNVC || 0), 0),
    [data.wallets],
  );

  const latestBlockTs = DUMMY_BLOCKS[0]?.timestamp ?? 0;
  const nextBlockTs = latestBlockTs > 0 ? latestBlockTs + 5 * 60 * 1000 : 0;
  const secToNext = nextBlockTs > 0 ? Math.max(0, Math.floor((nextBlockTs - Date.now()) / 1000)) : 0;
  const countdownText = nextBlockTs > 0
    ? `${Math.floor(secToNext / 60).toString().padStart(2, '0')}:${(secToNext % 60).toString().padStart(2, '0')}`
    : '--:--';
  const countdownPct = nextBlockTs > 0 ? Math.min(100, Math.max(0, Math.round(((300 - secToNext) / 300) * 100))) : 0;

  const pendingCount = DUMMY_TRANSACTIONS.filter((t) => t.status === 'PENDING').length;
  const healthyPeers = p2p.peer_health?.filter((entry) => entry.healthy).length ?? 0;
  const peerBase = p2p.peer_count || 0;
  const healthyPeerPct = peerBase > 0 ? Math.round((healthyPeers / peerBase) * 100) : 0;

  const scrollToTransactions = () => {
    const el = document.getElementById('explorer-transactions-table');
    el?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  };

  return (
    <div className="animate-in">
      {/* Header with Search */}
      <div className="flex items-center justify-between gap-6" style={{ marginBottom: 'var(--space-8)', flexWrap: 'wrap' }}>
        <div style={{ minWidth: '300px' }}>
          <h1 className="text-cyan">Blockchain Explorer</h1>
          <p className="text-muted text-sm mt-1">Audit blocks, transactions, and network health on-chain.</p>
        </div>
        <div className="nfm-search" style={{ maxWidth: '460px', width: '100%', flex: '1 1 300px' }}>
          <Search className="nfm-search__icon" size={18} />
          <input 
            type="text" 
            className="nfm-search__input" 
            placeholder="Search blocks, txids, addresses..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
      </div>

      {/* Primary Network Stats Grid */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-4)'
      }}>
        {[
          { label: 'Recent Avg Tx/Block', value: avgTxPerBlock.toFixed(2), icon: <Cpu size={16}/>, color: 'cyan', trend: `24h +${blocks24h} blocks` },
          { label: 'Active Nodes', value: p2p.peer_count.toLocaleString(), icon: <Globe size={16}/>, color: 'purple', trend: p2p.status.toUpperCase() },
          { label: 'Pending Transactions', value: pendingCount.toLocaleString(), icon: <Zap size={16}/>, color: 'success', trend: pendingCount > 0 ? 'Queue Open' : 'Queue Empty' },
          { label: 'Network Level', value: p2p.gossip_enabled ? 'Gossip ON' : 'Gossip OFF', icon: <ShieldCheck size={16}/>, color: 'cyan', trend: `Peers ${p2p.peer_count}` },
        ].map((stat, idx) => (
          <div key={idx} className="nfm-glass-card" style={{ padding: 'var(--space-5)', marginBottom: 0 }}>
            <div className="flex items-center gap-3 mb-3">
              <div className={`p-2 rounded-lg bg-surface-lowest text-${stat.color}`}>
                {stat.icon}
              </div>
              <span className="text-xs text-muted uppercase tracking-wider">{stat.label}</span>
            </div>
            <div className="flex justify-between items-end">
              <div className="text-xl font-display text-primary">{stat.value}</div>
              <div className={`text-xs text-${stat.color === 'success' ? 'success' : 'cyan'}`}>{stat.trend}</div>
            </div>
          </div>
        ))}
      </div>

      {/* Economic & Monetary Policy Row */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(4, 1fr)', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-8)' 
      }}>
        {/* Tokenomics Summary */}
        <div className="nfm-glass-card" style={{ marginBottom: 0, padding: 'var(--space-5)', gridColumn: 'span 2' }}>
          <div className="flex justify-between items-start mb-4">
            <h3 className="text-xs font-bold text-muted uppercase tracking-widest flex items-center gap-2">
              <Zap size={14} className="text-cyan" /> NVC Supply Dynamics
            </h3>
            <div className="nfm-badge nfm-badge--cyan font-mono text-[10px]">Deflationary</div>
          </div>
          <div className="grid gap-x-8 gap-y-6" style={{ 
            display: 'grid', 
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))' 
          }}>
            <div>
              <div className="text-[10px] text-muted mb-1 uppercase tracking-tighter">Mined Blocks</div>
              <div className="text-xl font-display text-primary">{data.status.blocks.toLocaleString()} <span className="text-xs text-muted">blocks</span></div>
              <div className="nfm-progress mt-2" style={{ height: '4px' }}>
                <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: `${Math.min(100, (data.status.blocks / 100) * 100)}%` }}></div>
              </div>
            </div>
            <div>
              <div className="text-[10px] text-muted mb-1 uppercase tracking-tighter">Circulating Supply</div>
              <div className="text-xl font-display text-purple">
                {(data.status.circulating_supply || totalWalletBalance).toLocaleString(undefined, { maximumFractionDigits: 2 })} 
                <span className="text-xs text-muted ml-1">NVC</span>
              </div>
              <div className="flex items-center gap-1 text-[10px] text-muted mt-1 italic">
                Active liquidity across {data.wallets.length} nodes
              </div>
            </div>
            <div>
              <div className="text-[10px] text-muted mb-1 uppercase tracking-tighter">Total Supply (Max)</div>
              <div className="text-xl font-display text-white">
                {(data.status.total_supply || 100000000).toLocaleString()}
                <span className="text-xs text-muted ml-1">NVC</span>
              </div>
              <div className="nfm-progress mt-2" style={{ height: '4px', background: 'rgba(255,255,255,0.05)' }}>
                <div className="nfm-progress__fill" style={{ width: `${((data.status.circulating_supply || totalWalletBalance) / (data.status.total_supply || 100000000) * 100).toFixed(4)}%`, background: 'var(--primary)' }}></div>
              </div>
            </div>
            <div>
              <div className="text-[10px] text-muted mb-1 uppercase tracking-tighter">Market Cap (Vault Value)</div>
              <div className="text-xl font-display text-cyan">
                ${((data.status.circulating_supply || totalWalletBalance) * 0.42).toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </div>
              <div className="text-[10px] text-muted mt-1">@ $0.42 / NVC (Simulated)</div>
            </div>
          </div>
        </div>

        {/* Burn Metrics */}
        <div className="nfm-glass-card--glow-pink" style={{ marginBottom: 0, padding: 'var(--space-5)', borderLeft: '3px solid var(--hyper-pink)', background: 'linear-gradient(135deg, rgba(255, 20, 147, 0.05), transparent)' }}>
          <div className="flex items-center gap-2 mb-4">
             <Flame size={16} className="text-pink" />
             <span className="text-xs text-muted uppercase tracking-wider">Total Burned</span>
          </div>
          <div className="text-xl font-display text-primary mb-1">{data.status.total_burned.toLocaleString(undefined, { maximumFractionDigits: 2 })} <span className="text-xs text-pink">NVC</span></div>
          <div className="flex items-center gap-1 text-[10px] text-success">
            <TrendingDown size={12} /> Burn tracked on-chain
          </div>
        </div>

        {/* Reward Pool */}
        <div className="nfm-glass-card--glow-cyan" style={{ marginBottom: 0, padding: 'var(--space-5)', borderLeft: '3px solid var(--neon-cyan)', background: 'linear-gradient(135deg, rgba(0, 245, 255, 0.05), transparent)' }}>
          <div className="flex items-center gap-2 mb-4">
             <Trophy size={16} className="text-cyan" />
             <span className="text-xs text-muted uppercase tracking-wider">Reward Pool</span>
          </div>
          <div className="text-xl font-display text-primary mb-1">{(data.status.reward_pool || 0).toLocaleString(undefined, { maximumFractionDigits: 2 })} <span className="text-xs text-cyan">NVC</span></div>
          <div className="flex justify-between items-center text-[10px] text-muted mt-1">
            <span>Dynamic Distribution</span>
            <span className="text-success">Next Batch Distribution</span>
          </div>
        </div>
      </div>

      {/* Network Life-cycle & Countdowns */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: '1fr 1fr', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-8)' 
      }}>
        {/* Next Block Countdown */}
        <div className="nfm-glass-card" style={{ marginBottom: 0, padding: 'var(--space-6)', borderLeft: '3px solid var(--neon-cyan)' }}>
          <h3 className="text-sm font-bold text-muted uppercase tracking-widest mb-6 flex items-center gap-2">
            <Hourglass size={14} className="text-cyan" /> Next Block Arrival
          </h3>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-4xl font-display text-primary">{countdownText}</div>
              <div className="text-xs text-muted">Estimated Confirmation Time</div>
            </div>
            <div className="relative" style={{ width: '60px', height: '60px' }}>
               <svg viewBox="0 0 36 36" style={{ width: '100%', height: '100%', transform: 'rotate(-90deg)' }}>
                 <circle cx="18" cy="18" r="16" fill="none" stroke="rgba(255,255,255,0.05)" strokeWidth="3" />
                 <circle cx="18" cy="18" r="16" fill="none" stroke="var(--neon-cyan)" strokeWidth="3" strokeDasharray="100" strokeDashoffset={100 - countdownPct} strokeLinecap="round" />
               </svg>
               <div className="absolute inset-0 flex items-center justify-center text-[10px] font-mono text-cyan">{countdownPct}%</div>
            </div>
          </div>
        </div>

        {/* Halving Countdown */}
        <div className="nfm-glass-card" style={{ marginBottom: 0, padding: 'var(--space-6)', borderLeft: '3px solid var(--sovereign-purple)' }}>
          <h3 className="text-sm font-bold text-muted uppercase tracking-widest mb-6 flex items-center gap-2">
            <Activity size={14} className="text-purple" /> Halving Milestone (Epoch Release)
          </h3>
          <div>
            <div className="flex justify-between items-end mb-2">
              <div className="text-2xl font-display text-primary">{blocksToHalving.toLocaleString()}</div>
              <div className="text-xs text-purple">Blocks to Reward Reduction</div>
            </div>
            <div className="nfm-progress" style={{ height: '6px' }}>
              <div className="nfm-progress__fill nfm-progress__fill--purple" style={{ width: `${halvingProgressPct}%` }}></div>
            </div>
            <div className="flex justify-between text-[10px] text-muted mt-2">
              <span>Next Halving Progress</span>
              <span>{halvingProgressPct.toFixed(2)}%</span>
            </div>
          </div>
        </div>
      </div>

      <div className="flex gap-8 wrap" style={{ flexWrap: 'wrap' }}>
        {/* Main Content: Blocks */}
        <div className="nfm-glass-card nfm-glass-card--glow-cyan" style={{ flex: '2 1 600px', marginBottom: 0 }}>
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-lg text-primary flex items-center gap-2">
              <Activity className="text-cyan" /> Latest Blocks
            </h2>
            <button className="nfm-btn nfm-btn--ghost nfm-btn--sm" onClick={() => setSearchQuery('')}>Clear Filter</button>
          </div>
          
          
          <div className="nfm-table-wrapper">
            <table className="nfm-table nfm-table--fixed">
              <thead>
                <tr>
                  <th style={{ width: '80px' }}>Height</th>
                  <th>Hash</th>
                  <th style={{ width: '60px' }}>Txs</th>
                  <th style={{ width: '180px' }}>Validator</th>
                  <th style={{ width: '100px' }}>Time</th>
                </tr>
              </thead>
              <tbody>
                {filteredBlocks.map(b => (
                  <tr key={b.index} className="nfm-glass-card--interactive" style={{cursor: 'pointer'}} onClick={() => setSelectedBlock(b)}>
                    <td className="font-mono text-cyan">#{b.index}</td>
                    <td className="font-mono text-muted text-xs">
                      <div className="text-truncate" title={b.hash}>{truncateHash(b.hash, 12)}</div>
                    </td>
                    <td className="text-center">{b.transactions}</td>
                    <td className="font-mono text-xs">
                      <div className="text-truncate text-cyan" title={b.miner}>{truncateHash(b.miner, 10)}</div>
                    </td>
                    <td className="text-muted text-xs whitespace-nowrap text-right">{formatAgo(b.timestamp)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          
          <button className="nfm-btn-more" onClick={() => setSearchQuery('')}>
            <ArrowRight size={14} /> View All Blocks
          </button>
        </div>

        {/* Sidebar: Chain Health */}
        <div className="flex-col gap-6" style={{ flex: '1 1 300px' }}>
          <div className="nfm-glass-card" style={{ marginBottom: 0 }}>
            <h3 className="text-md text-primary mb-4 flex items-center gap-2">
              <Server size={18} className="text-purple" /> Chain Health
            </h3>
            <div className="flex-col gap-4">
              <div className="flex-col gap-2">
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-muted">Network Load</span>
                  <span className="text-cyan">{Math.min(100, pendingCount * 8)}%</span>
                </div>
                <div className="nfm-progress">
                  <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: `${Math.min(100, pendingCount * 8)}%` }}></div>
                </div>
              </div>
              <div className="flex-col gap-2">
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-muted">Healthy Peers</span>
                  <span className="text-purple">{healthyPeerPct}%</span>
                </div>
                <div className="nfm-progress">
                  <div className="nfm-progress__fill nfm-progress__fill--purple" style={{ width: `${healthyPeerPct}%` }}></div>
                </div>
              </div>
            </div>
          </div>

          <div className="nfm-glass-card" style={{ marginBottom: 0 }}>
             <h3 className="text-md text-primary mb-4 flex items-center gap-2">
               <Hourglass size={18} className="text-cyan" /> Pending Txs
             </h3>
             <div className="flex-col gap-3">
               {DUMMY_TRANSACTIONS.filter(t => t.status === 'PENDING').map((t, idx) => (
                 <div key={idx} className="flex justify-between items-center py-2 border-b border-white-05" style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                   <div className="flex flex-col">
                     <span className="text-xs font-mono text-cyan truncate w-32">{t.txid}</span>
                     <span className="text-[10px] text-muted">{formatAgo(t.timestamp)}</span>
                   </div>
                   <span className="text-xs text-primary">{t.amount} NVC</span>
                 </div>
               ))}
             </div>
             <button className="nfm-btn-more" style={{ marginTop: 'var(--space-4)', padding: 'var(--space-2)' }} onClick={scrollToTransactions}>
               View Mempool
             </button>
          </div>
        </div>
      </div>

      {/* Full Transaction History Table */}
      <div id="explorer-transactions-table" className="nfm-glass-card mt-8" style={{ marginBottom: 0 }}>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-lg text-primary flex items-center gap-2">
            <Zap className="text-purple" /> Latest Transactions
          </h2>
          <div className="flex gap-2">
             <div className="nfm-badge nfm-badge--muted">Pending: {pendingCount}</div>
          </div>
        </div>
        
        <div className="nfm-table-wrapper">
          <table className="nfm-table nfm-table--fixed">
            <thead>
              <tr>
                <th style={{ width: '140px' }}>TXID</th>
                <th style={{ width: '100px' }}>Type</th>
                <th>From</th>
                <th>To</th>
                <th style={{ width: '110px' }}>Amount</th>
                <th style={{ width: '80px' }}>Fee</th>
                <th style={{ width: '100px' }}>Status</th>
              </tr>
            </thead>
            <tbody>
              {filteredTransactions.map((tx, idx) => (
                <tr key={idx} id={`tx-${tx.txid}`}>
                  <td className="font-mono text-cyan text-xs">
                    <div className="text-truncate" title={tx.txid}>{truncateHash(tx.txid, 8)}</div>
                  </td>
                  <td>
                    <span className={`nfm-badge nfm-badge--${tx.type === 'TRANSFER' ? 'cyan' : tx.type === 'SMART_CONTRACT' ? 'purple' : 'gold'}`} style={{ fontSize: '10px' }}>
                      {tx.type}
                    </span>
                  </td>
                  <td className="font-mono text-[10px]">
                    <div className="text-truncate text-muted" title={tx.from}>{truncateHash(tx.from, 10)}</div>
                  </td>
                  <td className="font-mono text-[10px]">
                    <div className="text-truncate text-muted" title={tx.to}>{truncateHash(tx.to, 10)}</div>
                  </td>
                  <td className="text-primary font-medium text-xs text-right">{tx.amount} NVC</td>
                  <td className="text-muted text-[10px] text-right">{tx.fee}</td>
                  <td>
                    <div className="flex items-center gap-2">
                      <div className={`nfm-status-dot nfm-status-dot--${tx.status === 'CONFIRMED' ? 'online' : 'syncing'}`}></div>
                      <span className="text-[10px]">{tx.status}</span>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        
        <button className="nfm-btn-more" onClick={() => setSearchQuery('')}>
          <ArrowRight size={14} /> View All Transactions
        </button>
      </div>

      {/* Block Detail Modal */}
      {selectedBlock && (
        <div className="nfm-modal-overlay" onClick={() => setSelectedBlock(null)}>
          <div className="nfm-modal animate-in" onClick={e => e.stopPropagation()}>
            <div className="nfm-modal-close" onClick={() => setSelectedBlock(null)}>
              <X size={20} />
            </div>
            
            <div className="flex items-center gap-4 mb-8">
              <div className="p-4 rounded-xl bg-surface-lowest text-cyan shadow-xl">
                <Activity size={32} />
              </div>
              <div>
                <h2 className="text-2xl font-display">Block Detail #{selectedBlock.index}</h2>
                <p className="text-muted font-mono text-xs">{selectedBlock.hash}</p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-6 mb-10">
              <div className="p-6 rounded-xl bg-surface-lowest border border-white-05">
                <label className="text-xs text-muted uppercase tracking-widest block mb-2">Previous Hash</label>
                <div className="text-sm font-mono text-secondary truncate">{selectedBlock.previous_hash}</div>
              </div>
              <div className="p-6 rounded-xl bg-surface-lowest border border-white-05">
                <label className="text-xs text-muted uppercase tracking-widest block mb-2">Validator (Node)</label>
                <div className="text-sm text-cyan" style={{ wordBreak: 'break-all', minHeight: '2.5rem' }}>{selectedBlock.miner}</div>
              </div>
              <div className="p-6 rounded-xl bg-surface-lowest border border-white-05">
                <label className="text-xs text-muted uppercase tracking-widest block mb-2">Block Size</label>
                <div className="text-sm text-primary">{selectedBlock.size}</div>
              </div>
              <div className="p-6 rounded-xl bg-surface-lowest border border-white-05">
                <label className="text-xs text-muted uppercase tracking-widest block mb-2">Block Rewards</label>
                <div className="text-sm text-success font-bold">{selectedBlock.rewards} NVC</div>
              </div>
            </div>

            <h3 className="text-lg mb-4">Transactions in this Block ({selectedBlock.transactions})</h3>
            <div className="space-y-3" style={{ maxHeight: '250px', overflowY: 'auto', paddingRight: 'var(--space-2)' }}>
               {selectedBlock.tx_hashes && selectedBlock.tx_hashes.length > 0 ? (
                 selectedBlock.tx_hashes.map((hash, i) => {
                   // Look up transaction details from global state
                   const txDetail = data.transactions.find(t => t.txid === hash);
                   
                   const info = {
                     type: txDetail?.type || 'TX',
                     amount: txDetail?.amount ? `${txDetail.amount} NVC` : '',
                     target: txDetail?.to ? `→ ${txDetail.to.slice(0, 8)}...` : ''
                   };

                   return (
                     <div 
                        key={i} 
                        className="p-3 rounded-lg bg-surface-lowest border border-white-05 flex justify-between items-center group hover:border-cyan/30 transition-all cursor-default"
                     >
                       <div className="flex items-center gap-3 overflow-hidden">
                         <div className={`w-8 h-8 rounded flex items-center justify-center text-[10px] font-bold flex-shrink-0 ${
                           info.type === 'TRANSFER' ? 'bg-cyan/10 text-cyan' : 
                           info.type === 'SMART_CONTRACT' ? 'bg-purple/10 text-purple' : 'bg-gold/10 text-gold'
                         }`}>
                           {info.type.slice(0, 2)}
                         </div>
                         <div className="flex flex-col min-w-0">
                           <span className="text-[11px] font-mono text-cyan truncate" title={hash}>
                             {hash.length > 20 ? `${hash.slice(0, 10)}...${hash.slice(-10)}` : hash}
                           </span>
                           {info.amount ? (
                             <span className="text-[10px] text-muted truncate">
                               {info.amount} {info.target}
                             </span>
                           ) : (
                             <span className="text-[10px] text-muted italic">Confirmed on-chain</span>
                           )}
                         </div>
                       </div>
                       <button
                          className="btn-ghost-mini"
                          style={{ marginLeft: 'var(--space-3)' }}
                          onClick={() => {
                            const txElement = document.getElementById(`tx-${hash}`);
                            if (txElement) {
                              setSelectedBlock(null);
                              txElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
                              // Highlight effect
                              txElement.style.background = 'rgba(0, 240, 255, 0.1)';
                              setTimeout(() => {
                                txElement.style.background = '';
                              }, 2000);
                            }
                          }}
                       >
                          View in Table
                       </button>
                     </div>
                   );
                 })
               ) : (
                 <div className="p-4 rounded-lg bg-surface-lowest text-sm text-muted text-center border border-dashed border-white-10">
                   No transaction data available in this block.
                 </div>
               )}
            </div>
            <button
               className="nfm-btn nfm-btn--ghost nfm-btn--sm mt-6"
               style={{ width: '100%' }}
               onClick={() => {
                 setSelectedBlock(null);
                 const el = document.querySelector('.nfm-table');
                 if (el) el.scrollIntoView({ behavior: 'smooth' });
               }}
            >
               Scroll to Latest Transactions
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default Explorer;
