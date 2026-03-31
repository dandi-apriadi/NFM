import { useState } from 'react';
import { Search, Activity, Server, Cpu, Globe, Zap, ShieldCheck, X, ArrowRight, Hourglass, Flame, Trophy, TrendingDown, Wallet } from 'lucide-react';
import type { Block } from '../types';
import { useAppData } from '../context/AppDataContext';

const Explorer = () => {
  const { data } = useAppData();
  const DUMMY_BLOCKS = data.blocks;
  const DUMMY_TRANSACTIONS = data.transactions;

  const [selectedBlock, setSelectedBlock] = useState<Block | null>(null);

  return (
    <div className="animate-in">
      {/* Header with Search */}
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <div>
          <h1 className="text-cyan">Blockchain Explorer</h1>
          <p className="text-muted text-sm mt-1">Audit blocks, transactions, and network health on-chain.</p>
        </div>
        <div className="nfm-search" style={{ width: '460px' }}>
          <Search className="nfm-search__icon" size={18} />
          <input 
            type="text" 
            className="nfm-search__input" 
            placeholder="Search blocks, txids, addresses..." 
          />
        </div>
      </div>

      {/* Primary Network Stats Grid */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(4, 1fr)', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-4)'
      }}>
        {[
          { label: 'Global Hashrate', value: '8.42 EH/s', icon: <Cpu size={16}/>, color: 'cyan', trend: '+2.1%' },
          { label: 'Active Nodes', value: '1,402', icon: <Globe size={16}/>, color: 'purple', trend: 'Online' },
          { label: 'Network Fee', value: '0.05 NVC', icon: <Zap size={16}/>, color: 'success', trend: 'Optimal' },
          { label: 'Network Level', value: 'Lvl 4', icon: <ShieldCheck size={16}/>, color: 'cyan', trend: 'Secure' },
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
          <div className="grid grid-cols-2 gap-8">
            <div>
              <div className="text-[10px] text-muted mb-1">Mined Supply</div>
              <div className="text-xl font-display text-primary">21,425,780 <span className="text-xs text-muted">/ 100M</span></div>
              <div className="nfm-progress mt-2" style={{ height: '4px' }}>
                <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: '21.4%' }}></div>
              </div>
            </div>
            <div>
              <div className="text-[10px] text-muted mb-1">Ecosystem Treasury</div>
              <div className="text-xl font-display text-purple">8,420,000 <span className="text-xs text-muted">NVC</span></div>
              <div className="flex items-center gap-1 text-[10px] text-muted mt-1">
                <Wallet size={10} /> 40% Epoch Allocation
              </div>
            </div>
          </div>
        </div>

        {/* Burn Metrics */}
        <div className="nfm-glass-card--glow-pink" style={{ marginBottom: 0, padding: 'var(--space-5)', borderLeft: '3px solid var(--hyper-pink)', background: 'linear-gradient(135deg, rgba(255, 20, 147, 0.05), transparent)' }}>
          <div className="flex items-center gap-2 mb-4">
             <Flame size={16} className="text-pink" />
             <span className="text-xs text-muted uppercase tracking-wider">Total Burned</span>
          </div>
          <div className="text-xl font-display text-primary mb-1">842,500 <span className="text-xs text-pink">NVC</span></div>
          <div className="flex items-center gap-1 text-[10px] text-success">
            <TrendingDown size={12} /> 50% Gas Fee Burned
          </div>
        </div>

        {/* Reward Pool */}
        <div className="nfm-glass-card--glow-cyan" style={{ marginBottom: 0, padding: 'var(--space-5)', borderLeft: '3px solid var(--neon-cyan)', background: 'linear-gradient(135deg, rgba(0, 245, 255, 0.05), transparent)' }}>
          <div className="flex items-center gap-2 mb-4">
             <Trophy size={16} className="text-cyan" />
             <span className="text-xs text-muted uppercase tracking-wider">Reward Pool</span>
          </div>
          <div className="text-xl font-display text-primary mb-1">12,450,000 <span className="text-xs text-cyan">NVC</span></div>
          <div className="flex justify-between items-center text-[10px] text-muted mt-1">
            <span>500 NVC / 5 Min</span>
            <span className="text-success">Refilling</span>
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
              <div className="text-4xl font-display text-primary">02:45</div>
              <div className="text-xs text-muted">Estimated Confirmation Time</div>
            </div>
            <div className="relative" style={{ width: '60px', height: '60px' }}>
               <svg viewBox="0 0 36 36" style={{ width: '100%', height: '100%', transform: 'rotate(-90deg)' }}>
                 <circle cx="18" cy="18" r="16" fill="none" stroke="rgba(255,255,255,0.05)" strokeWidth="3" />
                 <circle cx="18" cy="18" r="16" fill="none" stroke="var(--neon-cyan)" strokeWidth="3" strokeDasharray="100" strokeDashoffset="35" strokeLinecap="round" />
               </svg>
               <div className="absolute inset-0 flex items-center justify-center text-[10px] font-mono text-cyan">65%</div>
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
               <div className="text-2xl font-display text-primary">82,450</div>
               <div className="text-xs text-purple">Blocks to Reward Reduction</div>
            </div>
            <div className="nfm-progress" style={{ height: '6px' }}>
               <div className="nfm-progress__fill nfm-progress__fill--purple" style={{ width: '78%' }}></div>
            </div>
            <div className="flex justify-between text-[10px] text-muted mt-2">
               <span>Last: Aug 2024 (500 NVC)</span>
               <span>Next: Jan 2026 (250 NVC)</span>
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
            <button className="nfm-btn nfm-btn--ghost nfm-btn--sm">Filter Type</button>
          </div>
          
          <table className="nfm-table">
            <thead>
              <tr>
                <th>Height</th>
                <th>Hash</th>
                <th>Txs</th>
                <th>Validator</th>
                <th>Time</th>
              </tr>
            </thead>
            <tbody>
              {DUMMY_BLOCKS.map(b => (
                <tr key={b.index} className="nfm-glass-card--interactive" style={{cursor: 'pointer'}} onClick={() => setSelectedBlock(b)}>
                  <td className="font-mono text-cyan">#{b.index}</td>
                  <td className="font-mono text-muted">{b.hash.substring(0, 16)}...</td>
                  <td>{b.transactions}</td>
                  <td className="font-mono text-sm">{b.miner}</td>
                  <td className="text-muted">{Math.floor((Date.now() - b.timestamp) / 1000)}s ago</td>
                </tr>
              ))}
            </tbody>
          </table>
          <button className="nfm-btn-more">
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
                  <span className="text-cyan">42%</span>
                </div>
                <div className="nfm-progress">
                  <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: '42%' }}></div>
                </div>
              </div>
              <div className="flex-col gap-2">
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-muted">Staking Ratio</span>
                  <span className="text-purple">68.5%</span>
                </div>
                <div className="nfm-progress">
                  <div className="nfm-progress__fill nfm-progress__fill--purple" style={{ width: '68.5%' }}></div>
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
                     <span className="text-[10px] text-muted">Just now</span>
                   </div>
                   <span className="text-xs text-primary">{t.amount} NVC</span>
                 </div>
               ))}
             </div>
             <button className="nfm-btn-more" style={{ marginTop: 'var(--space-4)', padding: 'var(--space-2)' }}>
               View Mempool
             </button>
          </div>
        </div>
      </div>

      {/* Full Transaction History Table */}
      <div className="nfm-glass-card mt-8" style={{ marginBottom: 0 }}>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-lg text-primary flex items-center gap-2">
            <Zap className="text-purple" /> Latest Transactions
          </h2>
          <div className="flex gap-2">
             <div className="nfm-badge nfm-badge--muted">Total TPS: 1,240</div>
          </div>
        </div>
        
        <table className="nfm-table">
          <thead>
            <tr>
              <th>TXID</th>
              <th>Type</th>
              <th>From</th>
              <th>To</th>
              <th>Amount</th>
              <th>Fee</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {DUMMY_TRANSACTIONS.map((tx, idx) => (
              <tr key={idx}>
                <td className="font-mono text-cyan">{tx.txid}</td>
                <td>
                  <span className={`nfm-badge nfm-badge--${tx.type === 'TRANSFER' ? 'cyan' : tx.type === 'SMART_CONTRACT' ? 'purple' : 'gold'}`}>
                    {tx.type}
                  </span>
                </td>
                <td className="font-mono text-xs">{tx.from}</td>
                <td className="font-mono text-xs">{tx.to}</td>
                <td className="text-primary font-medium">{tx.amount} NVC</td>
                <td className="text-muted text-xs">{tx.fee}</td>
                <td>
                  <div className="flex items-center gap-2">
                    <div className={`nfm-status-dot nfm-status-dot--${tx.status === 'CONFIRMED' ? 'online' : 'syncing'}`}></div>
                    <span className="text-xs">{tx.status}</span>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        
        <button className="nfm-btn-more">
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
                <div className="text-sm text-cyan">{selectedBlock.miner}</div>
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
            <div className="space-y-3">
               {[1, 2, 3].map(i => (
                 <div key={i} className="flex justify-between items-center p-4 rounded-lg bg-surface-lowest">
                   <div className="flex items-center gap-3">
                     <div className="w-8 h-8 rounded bg-surface-container flex items-center justify-center text-xs text-muted">
                       {i}
                     </div>
                     <div className="flex flex-col">
                       <span className="text-xs font-mono text-cyan">0x{Math.random().toString(16).slice(2, 12)}...</span>
                       <span className="text-[10px] text-muted">Standard Transfer</span>
                     </div>
                   </div>
                   <div className="text-right">
                     <div className="text-sm text-primary">120.50 NVC</div>
                     <div className="text-[10px] text-muted">0.02 Fee</div>
                   </div>
                 </div>
               ))}
               <button className="nfm-btn nfm-btn--ghost nfm-btn--sm mt-2" style={{ width: '100%' }}>View All Block Transactions</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Explorer;
