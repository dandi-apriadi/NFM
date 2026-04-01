import { useState, useEffect } from 'react';
import { Cpu, Power, Zap, Activity, Leaf, Globe, Terminal, RefreshCw, Key, Database, Download, CheckCircle, Info, ArrowRight } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';
import { p2pBan, p2pBootstrap, p2pSetSeeds, p2pSync, p2pUnban } from '../api/client';

const NodeRunner = () => {
   const navigate = useNavigate();
   const { data, p2p, refresh, refreshPaused, notifySuccess, notifyError, requestPrompt } = useAppData();
   const DUMMY_NODE_STATS = data.node_stats;
   const DUMMY_STATUS = data.status;
   const DUMMY_API_DOCS = data.api_docs;
   const peerRows = p2p.known_peers.slice(0, 8);
   const p2pHealthLabel = p2p.gossip_enabled ? (p2p.status || 'online').toUpperCase() : 'DISABLED';
   const peerHealthMap = new Map((p2p.peer_health ?? []).map((entry) => [entry.endpoint, entry]));
   const nowUnix = Math.floor(Date.now() / 1000);
   const reconnectCountdown = p2p.next_reconnect_unix && p2p.next_reconnect_unix > nowUnix
      ? p2p.next_reconnect_unix - nowUnix
      : 0;

    const nodeHealthPct = (() => {
         const peerBase = p2p.peer_count || 0;
         const healthy = p2p.healthy_peers ?? peerBase;
         if (peerBase > 0) {
             return Math.min(100, Math.max(0, (healthy / peerBase) * 100));
         }
         return p2p.gossip_enabled ? 100 : 0;
    })();

    const memoryUtilPct = (() => {
         const match = /^\s*([\d.]+)\s*GB\s*\/\s*([\d.]+)\s*GB\s*$/i.exec(DUMMY_NODE_STATS.memory || '');
         if (!match) {
             return 0;
         }
         const used = Number(match[1]);
         const total = Number(match[2]);
         if (!Number.isFinite(used) || !Number.isFinite(total) || total <= 0) {
             return 0;
         }
         return Math.min(100, Math.max(0, (used / total) * 100));
    })();

   const [terminalLogs, setTerminalLogs] = useState([
      { time: new Date().toLocaleTimeString(), level: 'INFO', module: 'mesh_node', msg: `Connected peers: ${p2p.peer_count}` },
      { time: new Date().toLocaleTimeString(), level: 'INFO', module: 'chain_sync', msg: `Latest synced block: #${DUMMY_STATUS.blocks}` },
      { time: new Date().toLocaleTimeString(), level: 'OK', module: 'consensus', msg: `P2P status: ${p2pHealthLabel}` },
   ]);
   const [cmdInput, setCmdInput] = useState('');

   const pushTerminalLog = (level: 'INFO' | 'OK' | 'WORK', module: string, msg: string) => {
      setTerminalLogs((prev) => [
         ...prev,
         { time: new Date().toLocaleTimeString(), level, module, msg },
      ].slice(-24));
   };

   // Dynamic log pushing
   useEffect(() => {
     if (refreshPaused) return;
     pushTerminalLog('INFO', 'mesh_node', `Mesh sync heartbeat: ${p2p.peer_count} peers connected`);
   }, [p2p.peer_count, refreshPaused]);

   useEffect(() => {
     if (refreshPaused || DUMMY_STATUS.blocks === 0) return;
     pushTerminalLog('OK', 'chain_sync', `New block discovered: #${DUMMY_STATUS.blocks}`);
   }, [DUMMY_STATUS.blocks, refreshPaused]);

  const formatTime = (s: number) => {
    const m = Math.floor(s / 60);
    const rs = s % 60;
    return `${m.toString().padStart(2, '0')}:${rs.toString().padStart(2, '0')}`;
  };

   const signalStrengthBars = (latencyMs?: number) => {
      if (latencyMs === undefined || latencyMs <= 0) {
         return 0;
      }
      if (latencyMs <= 100) {
         return 4;
      }
      if (latencyMs <= 220) {
         return 3;
      }
      if (latencyMs <= 400) {
         return 2;
      }
      if (latencyMs <= 700) {
         return 1;
      }
      return 0;
   };

   const handleP2pSync = async () => {
      try {
         await p2pSync();
         await refresh();
         pushTerminalLog('OK', 'chain_sync', 'Manual P2P sync command accepted');
         notifySuccess('P2P sync command accepted');
      } catch (e) {
         pushTerminalLog('INFO', 'chain_sync', 'Manual P2P sync failed');
         notifyError(e instanceof Error ? e.message : 'Failed to trigger P2P sync');
      }
   };

   const handleP2pBootstrap = async () => {
      const defaultSeeds = p2p.known_peers.slice(0, 4).join(',');
      const seedInput = await requestPrompt({
         title: 'Bootstrap From Seeds',
         message: 'Seed peers CSV (host:port,host:port)',
         defaultValue: defaultSeeds,
         placeholder: '127.0.0.1:9000,127.0.0.1:9001',
         confirmText: 'Bootstrap',
      });
      if (seedInput === null) {
         return;
      }

      const seeds = seedInput
         .split(',')
         .map((s) => s.trim())
         .filter((s) => s.length > 0);

      try {
         await p2pSetSeeds(seeds);
         await p2pBootstrap();
         await refresh();
         pushTerminalLog('OK', 'mesh_node', `Bootstrap accepted with ${seeds.length} seed(s)`);
         notifySuccess(`P2P bootstrap command accepted (${seeds.length} seeds)`);
      } catch (e) {
         pushTerminalLog('INFO', 'mesh_node', 'Bootstrap command failed');
         notifyError(e instanceof Error ? e.message : 'Failed to bootstrap P2P mesh');
      }
   };

   const handleBanPeer = async () => {
      const defaultPeer = (p2p.peer_health ?? []).find((entry) => !entry.healthy)?.endpoint || peerRows[0] || '';
      const endpoint = await requestPrompt({
         title: 'Ban Peer Endpoint',
         message: 'Peer endpoint to ban (host:port)',
         defaultValue: defaultPeer,
         placeholder: '127.0.0.1:9000',
         confirmText: 'Ban',
      });
      if (endpoint === null) {
         return;
      }

      const trimmed = endpoint.trim();
      if (!trimmed || !trimmed.includes(':')) {
         notifyError('Invalid endpoint format. Use host:port');
         return;
      }

      try {
         await p2pBan(trimmed);
         await refresh();
         pushTerminalLog('OK', 'security', `Peer banned: ${trimmed}`);
         notifySuccess(`Ban command accepted for ${trimmed}`);
      } catch (e) {
         pushTerminalLog('INFO', 'security', `Ban failed: ${trimmed}`);
         notifyError(e instanceof Error ? e.message : 'Failed to ban peer');
      }
   };

   const handleUnbanPeer = async () => {
      const defaultPeer = p2p.banned_peers?.[0] || '';
      const endpoint = await requestPrompt({
         title: 'Unban Peer Endpoint',
         message: 'Peer endpoint to unban (host:port)',
         defaultValue: defaultPeer,
         placeholder: '127.0.0.1:9000',
         confirmText: 'Unban',
      });
      if (endpoint === null) {
         return;
      }

      const trimmed = endpoint.trim();
      if (!trimmed || !trimmed.includes(':')) {
         notifyError('Invalid endpoint format. Use host:port');
         return;
      }

      try {
         await p2pUnban(trimmed);
         await refresh();
         pushTerminalLog('OK', 'security', `Peer unbanned: ${trimmed}`);
         notifySuccess(`Unban command accepted for ${trimmed}`);
      } catch (e) {
         pushTerminalLog('INFO', 'security', `Unban failed: ${trimmed}`);
         notifyError(e instanceof Error ? e.message : 'Failed to unban peer');
      }
   };

   const handleRunCommand = async () => {
      const cmd = cmdInput.trim().toLowerCase();
      if (!cmd) {
         return;
      }

      if (cmd === 'sync') {
         await handleP2pSync();
      } else if (cmd === 'bootstrap') {
         await handleP2pBootstrap();
      } else if (cmd === 'status') {
         await refresh();
         pushTerminalLog('INFO', 'mesh_node', `Status refreshed. Peers=${p2p.peer_count}, blocks=${DUMMY_STATUS.blocks}`);
         notifySuccess('Node status refreshed');
      } else {
         pushTerminalLog('INFO', 'shell', `Unknown command: ${cmd}`);
         notifyError(`Unknown command: ${cmd}`);
      }

      setCmdInput('');
   };

   const handleRotateKeys = () => {
      pushTerminalLog('INFO', 'security', 'Rotate key requested but endpoint is not available yet');
      notifyError('Rotate key endpoint is not available on backend yet');
   };

   const handleFlushCache = async () => {
      await refresh();
      pushTerminalLog('OK', 'cache', 'Client cache refreshed from API');
      notifySuccess('Client cache refreshed from backend');
   };

   const handleExportConfig = () => {
      const config = {
         node: DUMMY_STATUS,
         p2p,
         generatedAt: new Date().toISOString(),
      };
      const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `nfm-node-config-${Date.now()}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      pushTerminalLog('OK', 'config', 'Node config exported');
      notifySuccess('Node config exported');
   };

  return (
    <div className="animate-in pb-12">
      {/* Header & Health Status */}
      <div className="flex items-center justify-between wrap gap-4" style={{ marginBottom: 'var(--space-10)' }}>
        <div>
           <h1 className="text-cyan flex items-center gap-2"><Terminal size={24} /> Node Operator Console</h1>
           <div className="flex items-center gap-4 mt-2">
              <div className="flex items-center gap-1.5 text-xs text-muted">
                 <div className="nfm-status-dot nfm-status-dot--online"></div> {DUMMY_STATUS.node}
              </div>
              <div className="text-10px bg-white-05 px-2 py-0.5 rounded font-mono text-muted uppercase">{DUMMY_STATUS.version}</div>
           </div>
        </div>
        <div className="flex items-center gap-4">
           <div className="text-right">
              <div className="text-10px text-muted uppercase tracking-tight">Node Health</div>
              <div className="text-xl font-display text-success">{nodeHealthPct.toFixed(1)}% <span className="text-xs opacity-60">{p2pHealthLabel}</span></div>
           </div>
           <button className="nfm-btn nfm-btn--danger nfm-btn--sm h-10 px-4" onClick={() => notifyError('Kill process is disabled from UI for safety. Use terminal if needed.')}> 
              <Power size={14} /> Kill Process
           </button>
        </div>
      </div>

      {/* Advanced Performance HUD */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(4, 1fr)', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-10)' 
      }}>
        {[
          { label: 'CPU LOAD', value: `${DUMMY_NODE_STATS.cpu}%`, icon: <Cpu size={16}/>, color: 'cyan', bar: DUMMY_NODE_STATS.cpu },
               { label: 'MEMORY', value: DUMMY_NODE_STATS.memory.split('/')[0], icon: <Database size={16}/>, color: 'purple', bar: memoryUtilPct },
               { label: 'CONNECTED PEERS', value: p2p.peer_count, icon: <Globe size={16}/>, color: 'pink', bar: Math.min(100, p2p.peer_count * 20) },
               { label: 'RETRY COUNTDOWN', value: reconnectCountdown > 0 ? formatTime(reconnectCountdown) : '00:00', icon: <RefreshCw size={16}/>, color: 'gold', bar: reconnectCountdown > 0 ? Math.min(100, (reconnectCountdown / 60) * 100) : 0 },
        ].map((stat, idx) => (
          <div key={idx} className="nfm-glass-card" style={{ padding: 'var(--space-6)', marginBottom: 0 }}>
             <div className="flex items-center justify-between mb-4">
                <span className="text-10px text-muted uppercase tracking-widest">{stat.label}</span>
                <div className={`text-${stat.color}`}>{stat.icon}</div>
             </div>
             <div className="text-2xl font-display text-primary mb-4">{stat.value}</div>
             <div className="nfm-progress" style={{ height: '3px', background: 'rgba(255,255,255,0.05)' }}>
                <div className={`nfm-progress__fill nfm-progress__fill--${stat.color}`} style={{ width: `${stat.bar}%` }}></div>
             </div>
          </div>
        ))}
      </div>

      <div className="flex gap-10 wrap" style={{ flexWrap: 'wrap' }}>
        
        {/* Left Column: Logs & Peers */}
        <div className="flex flex-col gap-8" style={{ flex: '2 1 600px' }}>
          
          {/* Main Terminal Stream */}
          <div className="nfm-glass-card nfm-glass-card--glow-cyan p-0 overflow-hidden" style={{ border: '1px solid rgba(0, 245, 255, 0.1)' }}>
            <div className="flex justify-between items-center px-6 py-4 border-b border-white-05 bg-black-20">
               <h3 className="text-xs font-mono text-cyan flex items-center gap-2">
                  <Activity size={14} /> LIVE_TELEMETRY_STREAM
               </h3>
               <div className="flex gap-4">
                  <div className="text-10px text-muted font-mono"><span className="text-success">SYNCED</span> @ #{DUMMY_STATUS.blocks}</div>
                  <div className="flex gap-1.5 item-center">
                     <div className="w-2 h-2 rounded-full bg-red-500"></div>
                     <div className="w-2 h-2 rounded-full bg-yellow-500"></div>
                     <div className="w-2 h-2 rounded-full bg-green-500"></div>
                  </div>
               </div>
            </div>
            
            <div className="font-mono text-11px p-6 overflow-y-auto bg-black-40" style={{ height: '320px' }}>
              {terminalLogs.map((log, i) => (
                <div key={i} className="mb-2.5 flex gap-3 whitespace-nowrap">
                   <span className="text-muted opacity-40">[{log.time}]</span>
                   <span className={`font-bold w-12 ${log.level === 'INFO' ? 'text-cyan-60' : log.level === 'OK' ? 'text-success' : 'text-purple-60'}`}>
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
                           className="bg-transparent border-none outline-none text-primary w-full font-mono text-11px"
                           value={cmdInput}
                           onChange={(e) => setCmdInput(e.target.value)}
                           onKeyDown={(e) => {
                               if (e.key === 'Enter') {
                                    e.preventDefault();
                                    void handleRunCommand();
                               }
                           }}
                />
                        <button className="nfm-btn nfm-btn--secondary nfm-btn--sm" onClick={() => void handleRunCommand()}>
                           <ArrowRight size={12} />
                        </button>
              </div>
            </div>
          </div>

          {/* Connected Peers Topology */}
          <div className="nfm-glass-card p-6">
             <h3 className="text-xs text-muted font-bold uppercase tracking-widest mb-6 flex items-center gap-2">
                <Globe size={14} className="text-pink" /> P2P Network Topology
             </h3>
             <div className="overflow-x-auto">
                <table className="w-full text-left font-mono text-11px">
                   <thead>
                      <tr className="text-muted opacity-60 border-b border-white-05">
                         <th className="pb-3 font-normal">PEER IDENTITY</th>
                         <th className="pb-3 font-normal">LATENCY</th>
                         <th className="pb-3 font-normal">PROTOCOL</th>
                         <th className="pb-3 font-normal text-right">STATUS</th>
                      </tr>
                   </thead>
                   <tbody className="text-secondary">
                      {peerRows.length > 0 ? peerRows.map((peer, idx) => {
                         const telemetry = peerHealthMap.get(peer);
                         const latencyText = telemetry ? `${telemetry.latency_ms} ms` : 'n/a';
                         const bars = signalStrengthBars(telemetry?.latency_ms);
                         const quality = telemetry?.quality ?? (p2p.status === 'online' ? 'good' : 'degraded');
                         const score = telemetry?.score ?? (p2p.status === 'online' ? 70 : 30);
                         const statusClass = quality === 'excellent' || quality === 'good'
                            ? 'text-success'
                            : quality === 'critical' || quality === 'poor'
                               ? 'text-danger'
                               : 'text-warning';

                         return (
                            <tr key={idx} className="border-b border-white-02">
                               <td className="py-3 text-cyan">{peer}</td>
                               <td className="py-3 text-muted">
                                  <div className="flex items-center gap-2">
                                     <span>{latencyText}</span>
                                     <span className="flex items-end gap-0.5" title={`Signal ${bars}/4`}>
                                        {[1, 2, 3, 4].map((bar) => (
                                           <span
                                              key={`${peer}-bar-${bar}`}
                                              style={{
                                                 width: '3px',
                                                 height: `${4 + bar * 2}px`,
                                                 borderRadius: '2px',
                                                 background: bar <= bars ? 'var(--neon-cyan)' : 'rgba(255,255,255,0.18)',
                                                 display: 'inline-block',
                                              }}
                                           />
                                        ))}
                                     </span>
                                  </div>
                               </td>
                               <td className="py-3 text-muted">p2p_v3</td>
                               <td className={`py-3 text-right font-bold ${statusClass}`}>
                                  {quality.toUpperCase()} ({score})
                               </td>
                            </tr>
                         );
                      }) : (
                        <tr>
                           <td className="py-3 text-muted" colSpan={4}>No peer discovered yet. Configure NFM_P2P_SEEDS and restart node.</td>
                        </tr>
                      )}
                   </tbody>
                </table>
             </div>
             <div className="text-10px text-muted mt-4">
                P2P status: {p2p.status} | Port: {p2p.listening_port} | Seeds: {p2p.seed_count} | Banned: {p2p.ban_count ?? p2p.banned_peers?.length ?? 0} | Healthy: {p2p.healthy_peers ?? p2p.peer_count} | Unhealthy: {p2p.unhealthy_peers ?? 0} | Reconnect attempts: {p2p.reconnect_attempts ?? 0} | Backoff: {p2p.reconnect_backoff_secs ?? 1}s{reconnectCountdown > 0 ? ` (${reconnectCountdown}s to retry)` : ''}
             </div>
          </div>
        </div>

        {/* Right Column: Actions & Docs */}
        <div className="flex flex-col gap-8" style={{ flex: '1 1 350px' }}>
          
          {/* Developer Action Hub */}
          <div className="nfm-glass-card p-6" style={{ marginBottom: 0 }}>
             <h3 className="text-xs text-muted uppercase tracking-widest mb-6">Operator Toolkit</h3>
             <div className="flex flex-col gap-4">
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-11px gap-3 h-11 border-purple-20 hover:border-purple" onClick={handleRotateKeys}>
                   <Key size={14} className="text-purple" /> Rotate Validator Keys
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-11px gap-3 h-11 border-cyan-20 hover:border-cyan" onClick={handleP2pSync}>
                   <RefreshCw size={14} className="text-cyan" /> Force Chain Resync
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-11px gap-3 h-11 border-cyan-20 hover:border-cyan" onClick={handleP2pBootstrap}>
                   <Globe size={14} className="text-cyan" /> Bootstrap from Seeds
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-11px gap-3 h-11 border-error-20 hover:border-error" onClick={handleBanPeer}>
                   <Power size={14} className="text-danger" /> Ban Peer Endpoint
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-11px gap-3 h-11 border-success-20 hover:border-success" onClick={handleUnbanPeer}>
                   <CheckCircle size={14} className="text-success" /> Unban Peer Endpoint
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-11px gap-3 h-11 border-pink-20 hover:border-pink" onClick={() => void handleFlushCache()}>
                   <Database size={14} className="text-pink" /> Flush Local Cache
                </button>
                <button className="nfm-btn nfm-btn--secondary w-full justify-start text-11px gap-3 h-11" onClick={handleExportConfig}>
                   <Download size={14} className="text-muted" /> Export config.toml
                </button>
             </div>
          </div>

          {/* Performance Strategies (Renamed) */}
          <div className="nfm-glass-card--glow-cyan p-6" style={{ marginBottom: 0 }}>
             <h3 className="text-xs text-muted uppercase tracking-widest mb-6">Operational Strategy</h3>
             <div className="flex flex-col gap-4">
                <div className="nfm-performance-card nfm-performance-card--selected p-4 bg-cyan-10 border-cyan-30 rounded-xl border">
                   <div className="flex justify-between items-center mb-1">
                      <div className="text-11px font-bold text-primary flex items-center gap-2">
                         <Zap size={12} className="text-cyan" /> High Performance
                      </div>
                      <CheckCircle size={12} className="text-cyan" />
                   </div>
                   <p className="text-[10px] text-muted leading-relaxed">Prioritizes block proposing and AI task execution for max rewards.</p>
                </div>
                <div className="nfm-performance-card p-5 bg-white-02 border-white-05 rounded-xl border opacity-50">
                   <div className="text-11px font-bold text-primary flex items-center gap-2 mb-2">
                      <Leaf size={12} className="text-success" /> Eco / Background
                   </div>
                   <p className="text-10px text-muted leading-relaxed">Limits AI workload and reduces CPU frequency to save energy.</p>
                </div>
             </div>
          </div>

          {/* Quick API Reference */}
          <div className="nfm-glass-card p-6" style={{ marginBottom: 0 }}>
             <h3 className="text-xs text-muted uppercase tracking-widest mb-6 flex items-center gap-2">
                <Info size={14} className="text-gold" /> API Endpoint Preview
             </h3>
             <div className="flex flex-col gap-4">
                {DUMMY_API_DOCS.slice(0, 3).map((api, idx) => (
                   <div key={idx} className="p-3 bg-surface-lowest rounded-lg border border-white-05">
                      <div className="flex items-center gap-2 mb-1">
                         <span className={`text-9px font-bold p-0.5 rounded leading-none ${api.method === 'POST' ? 'bg-purple text-white' : 'bg-green-600 text-white'}`}>
                            {api.method}
                         </span>
                         <span className="text-11px font-mono text-primary">{api.path}</span>
                      </div>
                      <p className="text-10px text-muted leading-tight">{api.description}</p>
                   </div>
                ))}
             </div>
             <button className="nfm-btn-more mt-4" onClick={() => navigate('/dev')}>
                <ArrowRight size={14} /> Full API Documentation
             </button>
          </div>

        </div>

      </div>
    </div>
  );
};

export default NodeRunner;
