import React, { useState, useEffect } from 'react';
import { 
  Activity, 
  Box, 
  Flame, 
  Search, 
  Wallet,
  Clock,
  Server
} from 'lucide-react';
import './index.css';

// Tipe Data sesuai Rust Backend
interface NodeStatus {
  node: string;
  balance: number;
  blocks: number;
  total_fees: number;
  total_burned: number;
  status: string;
  version: string;
  aliases?: Record<string, string>;
  mempool_count?: number;
  last_block_timestamp?: number;
  next_block_timestamp?: number;
  block_interval_secs?: number;
}

interface Block {
  index: number;
  timestamp: number;
  data: string;
  previous_hash: string;
  hash: string;
  nonce: number;
}


function App() {
  const [status, setStatus] = useState<NodeStatus | null>(null);
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [searchAddr, setSearchAddr] = useState('');
  const [walletBalance, setWalletBalance] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [selectedBlock, setSelectedBlock] = useState<Block | null>(null);
  const [now, setNow] = useState(Math.floor(Date.now() / 1000));
  const [view, setView] = useState<'dashboard' | 'mempool'>('dashboard');
  const [mempoolTxs, setMempoolTxs] = useState<string[]>([]);

  // Ticking timer untuk smooth UI countdown
  useEffect(() => {
    const timer = setInterval(() => {
      setNow(Math.floor(Date.now() / 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Auto-refresh setiap 3 detik
  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch status
        const statusRes = await fetch('http://127.0.0.1:3000/api/status');
        if (!statusRes.ok) throw new Error('API down');
        const statusData: NodeStatus = await statusRes.json();
        setStatus(statusData);

        // Fetch blocks
        const blocksRes = await fetch('http://127.0.0.1:3000/api/blocks');
        if (blocksRes.ok) {
          const blocksData = await blocksRes.json();
          // Tampilkan block terbaru duluan
          if (Array.isArray(blocksData)) {
            setBlocks(blocksData.reverse().slice(0, 10));
          }
        }
        
        // Fetch mempool
        const mempoolRes = await fetch('http://127.0.0.1:3000/api/mempool');
        if (mempoolRes.ok) {
          const memData = await mempoolRes.json();
          setMempoolTxs(memData);
        }

        setLoading(false);
        setError(false);
      } catch (err) {
        console.error("Failed to fetch NFM Node data:", err);
        setError(true);
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 3000);
    return () => clearInterval(interval);
  }, []);

  const searchWallet = async (e: React.FormEvent) => {
    e.preventDefault();
    const query = searchAddr.trim();
    if (!query) return;
    
    // Resolve alias if needed (e.g. @alice -> nfm_...)
    let address = query;
    if (status?.aliases && status.aliases[query]) {
      address = status.aliases[query];
    }
    
    try {
      const res = await fetch('http://127.0.0.1:3000/api/wallets');
      const wallets = await res.json();
      const balance = wallets[address] || 0;
      setWalletBalance(balance);
    } catch {
      setWalletBalance(0);
    }
  };

  const formatHash = (str: string | undefined | null) => {
    if (!str) return '---';
    if (str.length < 16) return str;
    return `${str.substring(0, 8)}...${str.substring(str.length - 8)}`;
  };

  const getBlockSize = (b: Block) => {
    const bytes = new Blob([JSON.stringify(b)]).size;
    if (bytes < 1024) return `${bytes} Bytes`;
    return `${(bytes / 1024).toFixed(2)} KB`;
  };

  const getTimeAgo = (timestampSecs: number) => {
    const diff = Math.max(0, now - timestampSecs);
    if (diff < 60) return `${diff} secs ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)} mins ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} hours ago`;
    return `${Math.floor(diff / 86400)} days ago`;
  };

  if (loading && !status) {
    return (
      <div className="app-container" style={{ alignItems: 'center', justifyContent: 'center' }}>
        <div className="loader"><Activity size={48} /></div>
        <p>Connecting to Neural Fragment Mesh Node...</p>
      </div>
    );
  }

  return (
    <div className="app-container">
      <header>
        <div className="brand">
          <Activity size={32} color="var(--accent-cyan)" />
          <h1>NFM Explorer</h1>
        </div>
        
        <div className="status-badge">
          <span className="dot" style={{ backgroundColor: error ? '#ff007f' : '#00f0ff'}}></span>
          {error ? 'Node Offline' : `Sync: ${status?.version || 'V1.0.0'}`}
        </div>
      </header>

      {error && !status ? (
        <div className="glass-card" style={{ borderColor: 'var(--accent-pink)', textAlign: 'center' }}>
          <h2>Connection Error</h2>
          <p style={{ color: 'var(--text-muted)', marginTop: '1rem' }}>
            Cannot connect to local NFM Node at http://127.0.0.1:3000.<br/>
            Please ensure you have started the node using <code>cargo run</code>.
          </p>
        </div>
      ) : (
        <>
          {/* Top Dashboard Stats */}
          <div className="dashboard-grid">
            <div className="glass-card">
              <div className="stat-icon cyan"><Box /></div>
              <div className="stat-value">{status?.blocks?.toLocaleString() || '0'}</div>
              <div className="stat-label">Total Blocks</div>
            </div>
            
            <div className="glass-card" style={{ cursor: 'pointer', transition: 'transform 0.2s', border: view === 'mempool' ? '1px solid var(--accent-purple)' : undefined }} onClick={() => setView('mempool')} onMouseOver={e => e.currentTarget.style.transform = 'translateY(-2px)'} onMouseOut={e => e.currentTarget.style.transform = 'translateY(0)'}>
              <div className="stat-icon purple"><Activity /></div>
              <div className="stat-value">{status?.mempool_count || '0'}</div>
              <div className="stat-label">Mempool (Click to View)</div>
            </div>
            
            <div className="glass-card">
              <div className="stat-icon cyan"><Clock /></div>
              <div className="stat-value">
                {status?.next_block_timestamp ? (() => {
                  const next = status.next_block_timestamp;
                  const diff = Math.max(0, next - now);
                  const m = Math.floor(diff / 60);
                  const s = diff % 60;
                  return `${m}:${s.toString().padStart(2, '0')}`;
                })() : '--:--'}
              </div>
              <div className="stat-label">Next Block In</div>
            </div>

            <div className="glass-card">
              <div className="stat-icon pink"><Flame /></div>
              <div className="stat-value">{status?.total_burned?.toFixed(2) || '0.00'}</div>
              <div className="stat-label">Total Burned (NVC)</div>
            </div>
          </div>

          {/* Main Layout */}
          <div className="main-grid">
            
            {/* Live Block Feed */}
            <div className="glass-card">
              <h2 className="section-title cyan"><Server /> Network Activity</h2>
              <div className="block-list">
                {blocks.map((b) => (
                  <div 
                    key={b.index} 
                    className="block-item" 
                    onClick={() => setSelectedBlock(b)}
                  >
                    <div className="block-info">
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <span className="block-index">Block #{b.index}</span>
                        <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)', background: 'rgba(255,255,255,0.05)', padding: '2px 8px', borderRadius: '10px' }}>
                          Size: {getBlockSize(b)}
                        </span>
                      </div>
                      <span className="block-hash">Hash: {formatHash(b.hash)}</span>
                      {(() => {
                        try {
                          const parsed = JSON.parse(b.data);
                          return (
                            <div style={{ fontSize: '0.8rem', marginTop: '6px', backgroundColor: 'rgba(0,0,0,0.2)', padding: '6px 10px', borderRadius: '8px', display: 'flex', gap: '12px' }}>
                              <span><span style={{color: 'var(--text-muted)'}}>TXs:</span> {parsed.transactions?.length || 0}</span>
                              <span><span style={{color: 'var(--text-muted)'}}>Rewards:</span> <span style={{color: 'var(--accent-cyan)'}}>{parsed.rewards?.length || 0}</span></span>
                              {parsed.economy?.fees_collected > 0 && <span><span style={{color: 'var(--text-muted)'}}>Fees:</span> <span style={{color: '#00ff88'}}>{parsed.economy.fees_collected.toFixed(1)}</span></span>}
                            </div>
                          );
                        } catch {
                          return <span className="block-data" style={{ marginTop: '4px' }}>{b.data}</span>;
                        }
                      })()}
                    </div>
                    <div className="block-time" style={{display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '10px', paddingTop: '10px', borderTop: '1px solid rgba(255,255,255,0.05)'}}>
                      <span style={{ fontSize: '0.8rem', color: 'var(--accent-purple)' }}>{getTimeAgo(b.timestamp)}</span>
                      <span style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '0.75rem' }}>
                        <Clock size={12} />
                        {new Date(b.timestamp * 1000).toLocaleTimeString()}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Wallet Utility */}
            <div className="glass-card">
              <h2 className="section-title purple"><Wallet /> Inspector</h2>
              <p style={{ color: 'var(--text-muted)', marginBottom: '1rem', fontSize: '0.9rem' }}>
                Look up an NFM Address to view its real-time balance.
              </p>
              
              <form onSubmit={searchWallet} className="search-box">
                <input 
                  type="text" 
                  className="search-input" 
                  placeholder="Enter NFM Address..." 
                  value={searchAddr}
                  onChange={(e) => setSearchAddr(e.target.value)}
                />
                <button type="submit" className="btn">
                  <Search size={18} />
                </button>
              </form>

              {walletBalance !== null && (
                <div className="wallet-result">
                  <div className="stat-label">
                    Balance for {searchAddr.startsWith('@') ? searchAddr : formatHash(searchAddr)}
                    {status?.aliases && Object.entries(status.aliases).find(([name, addr]) => addr === searchAddr || name === searchAddr) && (
                      <span style={{ color: 'var(--accent-cyan)', marginLeft: '8px' }}>
                        ({Object.entries(status.aliases).find(([name, addr]) => addr === searchAddr || name === searchAddr)?.[0]})
                      </span>
                    )}
                  </div>
                  <div className="wallet-balance">{walletBalance?.toFixed(2) || '0.00'} NVC</div>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    Synced with Block #{status?.blocks || '0'}
                  </div>
                </div>
              )}

              <div style={{ marginTop: '2rem', padding: '1rem', background: 'rgba(0,0,0,0.3)', borderRadius: '12px' }}>
                <div className="stat-label" style={{ marginBottom: '0.5rem' }}>Active Local Node</div>
                <div style={{ wordBreak: 'break-all', fontSize: '0.85rem', color: 'var(--accent-cyan)', fontFamily: 'monospace' }}>
                  {status?.node}
                </div>
              </div>
            </div>

          </div>

          {/* Modal Block Details */}
          {selectedBlock && (
            <div className="modal-overlay" onClick={() => setSelectedBlock(null)}>
              <div className="modal-content" onClick={(e) => e.stopPropagation()}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
                  <h2 className="section-title cyan" style={{ margin: 0, padding: 0, border: 'none' }}>Virtual Block Inspector</h2>
                  <button className="btn" style={{ padding: '0.4rem 1rem' }} onClick={() => setSelectedBlock(null)}>Close</button>
                </div>
                
                <div className="detail-row">
                  <span className="detail-label">Index / Height</span>
                  <span className="detail-value">#{selectedBlock.index}</span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">Block Size</span>
                  <span className="detail-value" style={{ color: 'var(--accent-cyan)' }}>{getBlockSize(selectedBlock)}</span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">Timestamp</span>
                  <span className="detail-value">
                    {new Date(selectedBlock.timestamp * 1000).toLocaleString()} 
                    <span style={{color: 'var(--text-muted)', fontSize: '0.8rem', marginLeft: '8px'}}>({selectedBlock.timestamp}) {getTimeAgo(selectedBlock.timestamp)}</span>
                  </span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">Cryptographic Hash (SHA-256)</span>
                  <span className="detail-value" style={{ fontFamily: 'monospace', wordBreak: 'break-all', color: 'var(--accent-cyan)' }}>
                    {selectedBlock.hash}
                  </span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">Previous Block Hash</span>
                  <span className="detail-value" style={{ fontFamily: 'monospace', wordBreak: 'break-all', color: 'var(--text-muted)' }}>
                    {selectedBlock.previous_hash}
                  </span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">PoW Nonce</span>
                  <span className="detail-value">{selectedBlock.nonce.toLocaleString()}</span>
                </div>
                
                <div style={{ marginTop: '2rem' }}>
                  {(() => {
                    try {
                      const parsed = JSON.parse(selectedBlock.data);
                      return (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                          
                          {/* Transactions */}
                          <div style={{ background: 'rgba(255,255,255,0.02)', padding: '15px', borderRadius: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                            <div className="detail-label" style={{ display: 'flex', justifyContent: 'space-between' }}>
                              <span>Transactions / Intents</span>
                              <span style={{ color: 'var(--text-light)' }}>{parsed.transactions?.length || 0}</span>
                            </div>
                            <div style={{ marginTop: '10px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
                              {parsed.transactions && parsed.transactions.length > 0 ? (
                                parsed.transactions.map((t: string, i: number) => (
                                  <div key={i} style={{ fontSize: '0.85rem', fontFamily: 'monospace', color: 'var(--text-muted)', padding: '8px', background: 'rgba(0,0,0,0.3)', borderRadius: '6px', wordBreak: 'break-all' }}>
                                    {t}
                                  </div>
                                ))
                              ) : (
                                <div style={{ color: 'var(--text-muted)' }}>No transactions in this block.</div>
                              )}
                            </div>
                          </div>

                          {/* Rewards */}
                          <div style={{ background: 'rgba(0,255,136,0.05)', padding: '15px', borderRadius: '12px', border: '1px solid rgba(0,255,136,0.2)' }}>
                            <div className="detail-label" style={{ color: '#00ff88', display: 'flex', justifyContent: 'space-between' }}>
                              <span>Distributed Rewards</span>
                              <span style={{ color: 'var(--text-light)' }}>{parsed.rewards?.length || 0} Nodes</span>
                            </div>
                            <div style={{ marginTop: '10px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
                              {parsed.rewards && parsed.rewards.length > 0 ? (
                                parsed.rewards.map((r: { address: string; amount: number; category?: string }, i: number) => (
                                  <div key={i} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', padding: '8px', background: 'rgba(0,0,0,0.3)', borderRadius: '6px' }}>
                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                      <span style={{ fontFamily: 'monospace' }}>{r.address}</span>
                                      {r.category && <span style={{ fontSize: '0.65rem', color: 'rgba(255,255,255,0.4)', textTransform: 'uppercase', letterSpacing: '0.5px' }}>➤ {r.category}</span>}
                                    </div>
                                    <span style={{ color: '#00ff88', fontWeight: 'bold', display: 'flex', alignItems: 'center' }}>+{r.amount.toFixed(4)} NVC</span>
                                  </div>
                                ))
                              ) : (
                                <div style={{ color: 'var(--text-muted)' }}>No rewards distributed.</div>
                              )}
                            </div>
                          </div>

                          {/* Economy Summary */}
                          <div style={{ background: 'rgba(0,243,255,0.05)', padding: '15px', borderRadius: '12px', border: '1px solid rgba(0,243,255,0.2)' }}>
                            <div className="detail-label" style={{ color: 'var(--accent-cyan)', marginBottom: '10px' }}>Economy Summary</div>
                            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px', fontSize: '0.9rem' }}>
                              <div>Epoch: <span style={{ color: 'var(--text-light)' }}>{parsed.economy?.epoch_number || 0}</span></div>
                              <div>Fees: <span style={{ color: '#00ff88' }}>{parsed.economy?.fees_collected?.toFixed(2) || '0.00'} NVC</span></div>
                              <div>Burned: <span style={{ color: '#ff007f' }}>{parsed.economy?.burned?.toFixed(2) || '0.00'} NVC</span></div>
                            </div>
                          </div>
                        </div>
                      );
                    } catch {
                      return (
                        <>
                          <div className="detail-label" style={{ marginBottom: '0.5rem' }}>On-Chain Transaction & Event Log</div>
                          <div className="block-data-box">
                            {selectedBlock.data || "Empty DPoS Heartbeat Block"}
                          </div>
                        </>
                      );
                    }
                  })()}
                </div>
              </div>
            </div>
          )}

          {/* Mempool View Modal Overlay */}
          {view === 'mempool' && (
            <div className="modal-overlay" onClick={() => setView('dashboard')}>
              <div className="modal-content glass-card" onClick={e => e.stopPropagation()} style={{ width: '80%', maxWidth: '800px', maxHeight: '80vh', overflowY: 'auto' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem', borderBottom: '1px solid rgba(255,255,255,0.1)', paddingBottom: '1rem' }}>
                  <h2 className="section-title purple" style={{ margin: 0 }}><Activity /> Mempool (Pending Transactions)</h2>
                  <button className="btn" onClick={() => setView('dashboard')}>Close</button>
                </div>
                
                <div style={{ color: 'var(--text-muted)', marginBottom: '1rem' }}>
                  Transactions in the mempool are waiting to be processed in the next block (up to 5 minutes).
                </div>

                {mempoolTxs.length === 0 ? (
                  <div style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-muted)', background: 'rgba(0,0,0,0.2)', borderRadius: '8px' }}>
                    Mempool is currently empty.
                  </div>
                ) : (
                  <div className="block-list">
                    {mempoolTxs.map((txStr, idx) => {
                      let displayTx = txStr;
                      try {
                        const parsed = JSON.parse(txStr);
                        displayTx = `${parsed.type}: ${parsed.address} ${parsed.target ? `-> ${parsed.target}` : ''} ${parsed.amount ? `(${parsed.amount} NVC)` : ''}`;
                      } catch {
                        displayTx = txStr;
                      }
                      
                      return (
                        <div key={idx} className="block-item" style={{ cursor: 'default' }}>
                          <span style={{ color: 'var(--text-main)', wordBreak: 'break-all' }}>{displayTx}</span>
                        </div>
                      )
                    })}
                  </div>
                )}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default App;
