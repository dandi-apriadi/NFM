import { useState, useEffect } from 'react';
import { Cpu, Power, Zap, Activity, Leaf, Globe, Terminal, RefreshCw, Key, Database, Download, CheckCircle, Info, ArrowRight } from 'lucide-react';
import { useAppData } from '../context/AppDataContext';

const NodeRunner = () => {
   const { data, p2p } = useAppData();
   const DUMMY_NODE_STATS = data.node_stats;
   const DUMMY_STATUS = data.status;
   const DUMMY_API_DOCS = data.api_docs;
   const peerRows = p2p.known_peers.slice(0, 8);
   const p2pHealthLabel = p2p.gossip_enabled ? (p2p.status || 'online').toUpperCase() : 'DISABLED';

  const [terminalLogs] = useState([
    { time: '14:32:01', level: 'INFO', module: 'mesh_node', msg: `Connected to ${p2p.peer_count} peers` },
    { time: '14:32:04', level: 'INFO', module: 'chain_sync', msg: 'Received block #128450' },
    { time: '14:32:04', level: 'OK', module: 'consensus', msg: 'Validated block #128450' },
    { time: '14:32:15', level: 'INFO', module: 'mempool', msg: 'Added 3 new intents' },
    { time: '14:33:00', level: 'WORK', module: 'ai_engine', msg: 'Processing task-001 (Market Scrape)' },
  ]);

  const [epochTime, setEpochTime] = useState(245); // 04:05 in seconds (5m cycle)

  useEffect(() => {
    const timer = setInterval(() => {
      setEpochTime(p => (p > 0 ? p - 1 : 300));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const formatTime = (s: number) => {
    const m = Math.floor(s / 60);
    const rs = s % 60;
    return `${m.toString().padStart(2, '0')}:${rs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="animate-in">
      {/* Header & Health Status */}
      <div className="flex items-center justify-between wrap gap-4" style={{ marginBottom: 'var(--space-8)' }}>
        <div>
           <h1 className="text-cyan flex items-center gap-2"><Terminal size={24} /> Node Operator Console</h1>
           <div className="flex items-center gap-4 mt-2">
              <div className="flex items-center gap-1.5 text-xs text-muted">
                 <div className="nfm-status-dot nfm-status-dot--online"></div> {DUMMY_STATUS.node}
              </div>
              <div className="text-[10px] bg-white-05 px-2 py-0.5 rounded font-mono text-muted uppercase">v1.2 Stable Build</div>
           </div>
        </div>
        <div className="flex items-center gap-4">
           <div className="text-right">
              <div className="text-[10px] text-muted uppercase tracking-tighter">Node Health</div>
              <div className="text-xl font-display text-success">98.4% <span className="text-xs opacity-60">{p2pHealthLabel}</span></div>
           </div>
           <button className="nfm-btn nfm-btn--danger nfm-btn--sm h-10 px-4">
              <Power size={14} /> Kill Process
           </button>
        </div>
      </div>

      {/* Advanced Performance HUD */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(4, 1fr)', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-8)' 
      }}>
        {[
          { label: 'CPU LOAD', value: `${DUMMY_NODE_STATS.cpu}%`, icon: <Cpu size={16}/>, color: 'cyan', bar: DUMMY_NODE_STATS.cpu },
          { label: 'MEMORY', value: DUMMY_NODE_STATS.memory.split('/')[0], icon: <Database size={16}/>, color: 'purple', bar: 45 },
               { label: 'CONNECTED PEERS', value: p2p.peer_count, icon: <Globe size={16}/>, color: 'pink', bar: p2p.peer_count > 0 ? 100 : 0 },
          { label: 'EPOCH CYCLE', value: formatTime(epochTime), icon: <RefreshCw size={16}/>, color: 'gold', bar: (epochTime / 300) * 100 },
        ].map((stat, idx) => (
          <div key={idx} className="nfm-glass-card" style={{ padding: 'var(--space-5)', marginBottom: 0 }}>
             <div className="flex items-center justify-between mb-3">
                <span className="text-[10px] text-muted uppercase tracking-widest">{stat.label}</span>
                <div className={`text-${stat.color}`}>{stat.icon}</div>
             </div>
             <div className="text-2xl font-display text-primary mb-3">{stat.value}</div>
             <div className="nfm-progress" style={{ height: '3px', background: 'rgba(255,255,255,0.05)' }}>
                <div className={`nfm-progress__fill nfm-progress__fill--${stat.color}`} style={{ width: `${stat.bar}%` }}></div>
             </div>
          </div>
        ))}
      </div>

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        
        {/* Left Column: Logs & Peers */}
        <div className="flex-col gap-6" style={{ flex: '2 1 600px' }}>
          
          {/* Main Terminal Stream */}
          <div className="nfm-glass-card nfm-glass-card--glow-cyan p-0 overflow-hidden" style={{ border: '1px solid rgba(0, 245, 255, 0.1)' }}>
            <div className="flex justify-between items-center px-6 py-4 border-b border-white-05 bg-black-20">
               <h3 className="text-xs font-mono text-cyan flex items-center gap-2">
                  <Activity size={14} /> LIVE_TELEMETRY_STREAM
               </h3>
               <div className="flex gap-4">
                  <div className="text-[10px] text-muted font-mono"><span className="text-success">SYNCED</span> @ #{DUMMY_STATUS.blocks}</div>
                  <div className="flex gap-1.5 item-center">
                     <div className="w-2 h-2 rounded-full bg-red-500"></div>
                     <div className="w-2 h-2 rounded-full bg-yellow-500"></div>
                     <div className="w-2 h-2 rounded-full bg-green-500"></div>
                  </div>
               </div>
            </div>
            
            <div className="font-mono text-[11px] p-6 h-[280px] overflow-y-auto bg-black-40">
              {terminalLogs.map((log, i) => (
                <div key={i} className="mb-1.5 flex gap-3 whitespace-nowrap">
                   <span className="text-muted opacity-40">[{log.time}]</span>
                   <span className={`font-bold w-12 ${log.level === 'INFO' ? 'text-blue-400' : log.level === 'OK' ? 'text-success' : 'text-purple'}`}>
                      {log.level}
                   </span>
                   <span className="text-muted w-20">[{log.module}]</span>
                   <span className="text-secondary whitespace-normal">{log.msg}</span>
                </div>
              ))}
              <div className="mt-4 flex gap-2 items-center">
                <span className="text-success">root@nfm-alpha:~$</span>
                <input 
                  type="text" 
                  placeholder="enter node command..." 
                  className="bg-transparent border-none outline-none text-primary w-full font-mono text-[11px]" 
                />
              </div>
            </div>
          </div>

          {/* Connected Peers Topology */}
          <div className="nfm-glass-card p-6">
             <h3 className="text-xs text-muted font-bold uppercase tracking-widest mb-6 flex items-center gap-2">
                <Globe size={14} className="text-pink" /> P2P Network Topology
             </h3>
             <div className="overflow-x-auto">
                <table className="w-full text-left font-mono text-[11px]">
                   <thead>
                      <tr className="text-muted opacity-60 border-b border-white-05">
                         <th className="pb-3 font-normal">PEER IDENTITY</th>
                         <th className="pb-3 font-normal">LATENCY</th>
                         <th className="pb-3 font-normal">PROTOCOL</th>
                         <th className="pb-3 font-normal text-right">STATUS</th>
                      </tr>
                   </thead>
                   <tbody className="text-secondary">
                      {peerRows.length > 0 ? peerRows.map((peer, idx) => (
                        <tr key={idx} className="border-b border-white-02">
                           <td className="py-3 text-cyan">{peer}</td>
                           <td className="py-3 text-muted">n/a</td>
                           <td className="py-3 text-muted">p2p_v3</td>
                           <td className={`py-3 text-right font-bold ${p2p.status === 'online' ? 'text-success' : 'text-warning'}`}>
                              {p2p.status === 'online' ? 'ACTIVE' : 'SYNCING'}
                           </td>
                        </tr>
                      )) : (
                        <tr>
                           <td className="py-3 text-muted" colSpan={4}>No peer discovered yet. Configure NFM_P2P_SEEDS and restart node.</td>
                        </tr>
                      )}
                   </tbody>
                </table>
             </div>
             <div className="text-[10px] text-muted mt-4">
                P2P status: {p2p.status} | Port: {p2p.listening_port} | Seeds: {p2p.seed_count}
             </div>
          </div>
        </div>

        {/* Right Column: Actions & Docs */}
        <div className="flex-col gap-6" style={{ flex: '1 1 350px' }}>
          
          {/* Developer Action Hub */}
          <div className="nfm-glass-card p-6">
             <h3 className="text-xs text-muted uppercase tracking-widest mb-6">Operator Toolkit</h3>
             <div className="flex-col gap-3">
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-[11px] gap-3 h-11 border-purple-20 hover:border-purple">
                   <Key size={14} className="text-purple" /> Rotate Validator Keys
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-[11px] gap-3 h-11 border-cyan-20 hover:border-cyan">
                   <RefreshCw size={14} className="text-cyan" /> Force Chain Resync
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-[11px] gap-3 h-11 border-pink-20 hover:border-pink">
                   <Database size={14} className="text-pink" /> Flush Local Cache
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-[11px] gap-3 h-11">
                   <Download size={14} className="text-muted" /> Export config.toml
                </button>
             </div>
          </div>

          {/* Performance Strategies (Renamed) */}
          <div className="nfm-glass-card--glow-cyan p-6">
             <h3 className="text-xs text-muted uppercase tracking-widest mb-4">Operational Strategy</h3>
             <div className="flex-col gap-3">
                <div className="nfm-performance-card nfm-performance-card--selected p-4 bg-cyan-10 border-cyan-30 rounded-xl border">
                   <div className="flex justify-between items-center mb-1">
                      <div className="text-[11px] font-bold text-primary flex items-center gap-2">
                         <Zap size={12} className="text-cyan" /> High Performance
                      </div>
                      <CheckCircle size={12} className="text-cyan" />
                   </div>
                   <p className="text-[10px] text-muted">Prioritizes block proposing and AI task execution for max rewards.</p>
                </div>
                <div className="nfm-performance-card p-4 bg-white-02 border-white-05 rounded-xl border opacity-50">
                   <div className="text-[11px] font-bold text-primary flex items-center gap-2 mb-1">
                      <Leaf size={12} className="text-success" /> Eco / Background
                   </div>
                   <p className="text-[10px] text-muted">Limits AI workload and reduces CPU frequency to save energy.</p>
                </div>
             </div>
          </div>

          {/* Quick API Reference */}
          <div className="nfm-glass-card p-6">
             <h3 className="text-xs text-muted uppercase tracking-widest mb-4 flex items-center gap-2">
                <Info size={14} className="text-gold" /> API Endpoint Preview
             </h3>
             <div className="flex-col gap-3">
                {DUMMY_API_DOCS.slice(0, 3).map((api, idx) => (
                   <div key={idx} className="p-3 bg-surface-lowest rounded-lg border border-white-05">
                      <div className="flex items-center gap-2 mb-1">
                         <span className={`text-[9px] font-bold p-0.5 rounded leading-none ${api.method === 'POST' ? 'bg-purple text-white' : 'bg-green-600 text-white'}`}>
                            {api.method}
                         </span>
                         <span className="text-[11px] font-mono text-primary">{api.path}</span>
                      </div>
                      <p className="text-[10px] text-muted leading-tight">{api.description}</p>
                   </div>
                ))}
             </div>
             <button className="nfm-btn-more mt-4">
                <ArrowRight size={14} /> Full API Documentation
             </button>
          </div>

        </div>

      </div>
    </div>
  );
};

export default NodeRunner;
