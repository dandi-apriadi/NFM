import { useState } from 'react';
import { 
  Shield, 
  Key, 
  Globe, 
  Eye, 
  EyeOff, 
  Copy, 
  Download, 
  Zap, 
  Bell, 
  Moon, 
  Sun, 
  Database, 
  CheckCircle,
  AlertCircle,
  ChevronRight,
  ShieldAlert,
  Server,
  Activity,
  RefreshCw
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';

// Sub-component: Neural Toggle Switch
const NeuralSwitch = ({ active, label, description }: { active: boolean, label: string, description: string }) => (
  <div className="flex justify-between items-center p-5 bg-surface-lowest/40 rounded-2xl border border-white/05 group hover:border-cyan/20 transition-all duration-300">
    <div className="flex flex-col gap-1 pr-4">
      <div className="text-sm font-bold text-primary group-hover:text-cyan transition-colors">{label}</div>
      <div className="text-[10px] text-muted leading-relaxed opacity-70 group-hover:opacity-100">{description}</div>
    </div>
    
    <div className={`w-14 h-7 rounded-full border border-white/10 relative p-1 cursor-pointer transition-all duration-500 overflow-hidden ${active ? 'bg-cyan/10' : 'bg-black-40'}`}>
       {/* Background Glow */}
       {active && <div className="absolute inset-0 bg-cyan opacity-20 blur-md animate-pulse"></div>}
       {/* Knob */}
       <div className={`absolute top-1 w-5 h-5 rounded-full shadow-[0_0_10px_rgba(0,245,255,0.5)] transition-all duration-500 z-10 ${active ? 'right-1 bg-cyan' : 'right-8 bg-zinc-700'}`}></div>
       {/* Inner Label */}
       <div className={`absolute inset-0 flex items-center justify-between px-2 text-[6px] font-bold uppercase tracking-tighter ${active ? 'text-cyan-500' : 'text-zinc-500'}`}>
         <span className="ml-1">ON</span>
         <span className="mr-1">OFF</span>
       </div>
    </div>
  </div>
);

const Settings = () => {
  const navigate = useNavigate();
  const { data } = useAppData();
  const DUMMY_USER = data.user_profile;
  const [activeTab, setActiveTab] = useState<'security' | 'network' | 'prefs'>('security');
  const [showSeed, setShowSeed] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleExportConfig = () => {
    const config = JSON.stringify({
      user: DUMMY_USER.username,
      address: DUMMY_USER.nfmAddress,
      settings: DUMMY_USER.settings
    }, null, 2);
    const blob = new Blob([config], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `nfm_config_${DUMMY_USER.username.replace('@', '')}.json`;
    a.click();
  };

  return (
    <div className="animate-in max-w-7xl mx-auto pb-24">
      {/* Immersive Header Section */}
      <div className="relative mb-16 pt-6">
        {/* Background Mesh Flare */}
        <div className="absolute -top-12 left-1/2 -translate-x-1/2 w-[800px] h-[300px] bg-gradient-to-b from-cyan/05 via-purple/05 to-transparent blur-[100px] pointer-events-none"></div>
        
        <div className="relative z-10 flex items-center justify-between">
          <div className="flex items-center gap-8">
            <button 
              className="nfm-btn nfm-btn--ghost w-12 h-12 p-0 flex items-center justify-center rounded-2xl border-white/05 hover:border-cyan/40 hover:bg-cyan/05 transition-all shadow-glow-cyan--soft" 
              onClick={() => navigate(-1)}
            >
              <ChevronRight size={22} style={{ transform: 'rotate(180deg)' }} />
            </button>
            <div className="h-14 w-[1px] bg-white/05"></div>
            <div>
              <div className="flex items-center gap-3">
                 <Zap size={24} className="text-cyan drop-shadow-cyan" />
                 <h1 className="text-3xl font-display font-bold text-primary tracking-tight">System Core</h1>
              </div>
              <p className="text-[10px] text-muted font-bold uppercase tracking-[0.25em] mt-2 opacity-60">
                 NODE_ENCLAVE_v1.2 // MESH_PROTOCOL_UNSET
              </p>
            </div>
          </div>
          
          <div className="flex gap-3">
            <div className="flex items-center gap-6 px-6 h-12 bg-white/02 border border-white/05 rounded-2xl">
               <div className="flex flex-col items-end">
                  <span className="text-[9px] text-muted uppercase font-bold tracking-widest leading-none">Latency Status</span>
                  <span className="text-xs font-mono font-bold text-success mt-1">4.2ms PERF_OK</span>
               </div>
               <div className="flex flex-col items-end">
                  <span className="text-[9px] text-muted uppercase font-bold tracking-widest leading-none">Shard ID</span>
                  <span className="text-xs font-mono font-bold text-cyan mt-1">AX-712-B</span>
               </div>
            </div>
            <button className="nfm-btn nfm-btn--primary h-12 px-6 gap-2 font-display font-bold tracking-wide" onClick={handleExportConfig}>
              <Download size={18} /> BACKUP ALL
            </button>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-12 gap-8 items-start">
        
        {/* Navigation Sidebar (3 cols) */}
        <div className="col-span-12 lg:col-span-3 flex flex-col gap-4">
          <div className="text-[10px] font-bold text-muted uppercase tracking-[0.25em] ml-4 mb-2">Protocol Groups</div>
          {[
            { id: 'security', label: 'Security Hub', icon: <Shield size={18} />, status: 'SECURED' },
            { id: 'network', label: 'Network & Shard', icon: <Globe size={18} />, status: 'SYNCED' },
            { id: 'prefs', label: 'Preferences', icon: <Bell size={18} />, status: 'READY' },
          ].map((tab) => (
            <button 
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`group flex items-center gap-5 p-5 rounded-[1.25rem] transition-all duration-500 text-left border overflow-hidden relative ${
                activeTab === tab.id 
                  ? 'bg-cyan/10 border-cyan/30 shadow-[0_4px_30px_rgba(0,245,255,0.1)]' 
                  : 'bg-white/01 border-transparent hover:bg-white/03 hover:border-white/10'
              }`}
            >
              <div className={`relative z-10 w-10 h-10 flex items-center justify-center rounded-xl transition-all duration-500 ${
                activeTab === tab.id ? 'bg-cyan text-black shadow-glow-cyan--soft' : 'bg-white/05 text-muted group-hover:text-primary'
              }`}>
                {tab.icon}
              </div>
              <div className="relative z-10">
                <div className={`text-sm font-bold transition-colors ${activeTab === tab.id ? 'text-primary' : 'text-muted'}`}>{tab.label}</div>
                <div className="flex items-center gap-2 mt-1">
                  <div className={`w-1 h-1 rounded-full ${activeTab === tab.id ? 'bg-cyan animate-pulse' : 'bg-white/20'}`}></div>
                  <div className="text-[9px] text-muted opacity-40 uppercase font-bold tracking-widest">{tab.status}</div>
                </div>
              </div>
              {activeTab === tab.id && (
                <div className="absolute right-0 top-0 bottom-0 w-1 bg-cyan shadow-glow-cyan"></div>
              )}
            </button>
          ))}

          <div className="mt-8 nfm-glass-card border border-white/05 bg-surface-lowest/20 p-6">
             <div className="flex items-center gap-3 mb-4">
                <div className="w-8 h-8 rounded-lg bg-success/10 flex items-center justify-center">
                   <ShieldAlert size={16} className="text-success" />
                </div>
                <span className="text-[10px] text-primary font-bold uppercase tracking-widest">Enclave Status</span>
             </div>
             <div className="flex flex-col gap-4">
                <p className="text-[10px] text-muted leading-relaxed">
                   Advanced End-to-End Encryption is active. All node-specific metadata is stored in a sandboxed hardware module.
                </p>
                <div className="flex items-center justify-between pt-4 border-t border-white/05">
                   <span className="text-[9px] text-muted uppercase font-bold tracking-tighter">Key Origin</span>
                   <span className="text-[9px] text-white/40 font-mono">LOCAL_HARDWARE</span>
                </div>
                <div className="flex items-center justify-between">
                   <span className="text-[9px] text-muted uppercase font-bold tracking-tighter">Cipher</span>
                   <span className="text-[9px] text-white/40 font-mono">AES-256-GCM</span>
                </div>
             </div>
          </div>
        </div>

        {/* Content Area (9 cols) */}
        <div className="col-span-12 lg:col-span-9 flex flex-col gap-6">
          
          {activeTab === 'security' && (
            <div className="animate-in-slide flex flex-col gap-8">
              <div className="nfm-glass-card shadow-glow-cyan border overflow-hidden p-0 relative">
                {/* Visual Texture Background */}
                <div className="absolute inset-0 opacity-[0.03] pointer-events-none" 
                     style={{ backgroundImage: 'radial-gradient(var(--neon-cyan) 0.5px, transparent 0.5px)', backgroundSize: '20px 20px' }}></div>
                
                <div className="p-10 relative z-10">
                  <div className="flex items-center justify-between mb-8">
                    <div className="flex items-start gap-5">
                       <div className="w-16 h-16 rounded-[1.25rem] bg-cyan/10 flex items-center justify-center border border-cyan/20">
                          <Key className="text-cyan drop-shadow-cyan" size={32} />
                       </div>
                       <div>
                          <h2 className="text-2xl font-display font-bold text-primary mb-1">Validator Discovery Mnemonic</h2>
                          <div className="flex items-center gap-2">
                             <div className="text-[10px] text-muted uppercase font-bold tracking-[0.15em]">Secure Enclave Vault</div>
                             <div className="w-[1px] h-3 bg-white/10"></div>
                             <div className="text-[10px] text-warning font-bold uppercase tracking-[0.15em] flex items-center gap-1.5">
                                <AlertCircle size={10} /> Read Carefully
                             </div>
                          </div>
                       </div>
                    </div>
                  </div>

                  <p className="text-sm text-muted mb-10 max-w-2xl leading-relaxed">
                    The 12-word mnemonic fragment below is the master decryption key for your Neural Mesh identity. 
                    Anyone with these words can permanently claim your node rewards and staked assets. 
                    <span className="text-white/60 block mt-2">Store this only in a trusted physical format or air-gapped hardware vault.</span>
                  </p>

                  <div className="relative group bg-surface-lowest/60 rounded-[2.5rem] border border-white/10 p-2 flex flex-col overflow-hidden shadow-2xl">
                    {/* Security Visual Grid */}
                    <div className="absolute inset-0 opacity-[0.07]" 
                         style={{ backgroundImage: 'linear-gradient(var(--text-muted) 1px, transparent 1px), linear-gradient(90deg, var(--text-muted) 1px, transparent 1px)', backgroundSize: '12% 25%' }}></div>
                    
                    <div className={`relative p-12 py-16 font-mono text-lg leading-relaxed text-center tracking-[0.2em] transition-all duration-700 rounded-[2rem] overflow-hidden ${!showSeed ? 'blur-2xl select-none grayscale scale-[0.98] bg-black/60' : 'bg-surface-lowest shadow-inner'}`}>
                      {showSeed ? (
                        <div className="flex flex-wrap items-center justify-center gap-4">
                           {DUMMY_USER.seedPhrase?.split(' ').map((word, i) => (
                             <div key={i} className="flex flex-col items-center gap-1 min-w-[100px] py-3 bg-white/03 rounded-xl border border-white/05 group/word hover:border-cyan/40 hover:bg-cyan/10 transition-all cursor-default">
                                <span className="text-[8px] text-muted/60 opacity-40 font-mono tracking-tighter">{i + 1}</span>
                                <span className="text-primary group-hover/word:text-cyan transition-colors">{word}</span>
                             </div>
                           ))}
                        </div>
                      ) : (
                        '•••• •••• •••• •••• •••• •••• •••• •••• •••• •••• •••• ••••'
                      )}
                      
                      {/* Scanline Effect */}
                      {showSeed && (
                        <div className="absolute top-0 left-0 w-full h-[2px] bg-cyan/40 shadow-[0_0_15px_rgba(0,245,255,0.5)] animate-scanline pointer-events-none"></div>
                      )}
                    </div>
                    
                    {!showSeed && (
                      <div className="absolute inset-0 flex items-center justify-center">
                        <button 
                          className="nfm-btn nfm-btn--primary px-12 py-4 h-16 rounded-2xl shadow-glow-cyan--soft animate-float"
                          onClick={() => setShowSeed(true)}
                        >
                          <div className="flex items-center gap-4">
                             <div className="w-8 h-8 rounded-lg bg-black/20 flex items-center justify-center">
                                <Eye size={18} />
                             </div>
                             <span className="font-display font-bold tracking-widest text-lg uppercase">Decrypt Fragment</span>
                          </div>
                        </button>
                      </div>
                    )}
                    
                    {showSeed && (
                      <div className="flex items-center justify-between p-4 px-8 bg-surface-lowest border-t border-white/10 mt-1 rounded-b-[2rem]">
                        <div className="flex items-center gap-3">
                           <div className="w-2 h-2 rounded-full bg-warning animate-pulse shadow-[0_0_8px_rgba(255,184,0,0.8)]"></div>
                           <span className="text-[10px] text-warning font-bold uppercase tracking-[0.2em]">Live Decryption Active</span>
                        </div>
                        <div className="flex gap-3">
                          <button 
                            className="flex items-center gap-3 px-6 h-11 bg-white/05 hover:bg-white/10 border border-white/10 rounded-xl text-xs font-bold transition-all group/copy"
                            onClick={() => handleCopy(DUMMY_USER.seedPhrase || '')}
                          >
                            {copied ? (
                              <><CheckCircle size={16} className="text-success" /> Copied</>
                            ) : (
                              <><Copy size={16} className="text-muted group-hover/copy:text-primary transition-colors" /> Copy phrase</>
                            )}
                          </button>
                          <button 
                            className="flex items-center gap-2 px-6 h-11 bg-white/05 hover:bg-white/10 border border-white/10 rounded-xl text-xs font-bold text-muted hover:text-primary transition-all"
                            onClick={() => setShowSeed(false)}
                          >
                            <EyeOff size={18} /> Hide
                          </button>
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="nfm-glass-card group hover:border-purple/40 hover:bg-purple/02 transition-all duration-500 overflow-hidden">
                  <div className="absolute top-0 right-0 p-6 opacity-05 group-hover:opacity-10 transition-opacity">
                     <Database size={60} />
                  </div>
                  <h3 className="text-sm font-bold text-primary mb-2 flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-surface-lowest flex items-center justify-center text-purple">
                       <Download size={14} />
                    </div>
                    Encrypted Vault Backup
                  </h3>
                  <p className="text-[11px] text-muted leading-relaxed mb-6">
                    Download an AES-GCM encrypted container containing your keys, node logic, and localized shard cache.
                  </p>
                  <button className="nfm-btn nfm-btn--ghost w-full py-4 text-[10px] font-bold uppercase tracking-widest border-white/05 group-hover:border-purple/30 group-hover:bg-purple/10">
                    Export Secure Archive (.nfm)
                  </button>
                </div>

                <div className="nfm-glass-card group hover:border-error/40 hover:bg-error/02 transition-all duration-500 overflow-hidden">
                  <div className="absolute top-0 right-0 p-6 opacity-05 group-hover:opacity-10 transition-opacity">
                     <ShieldAlert size={60} />
                  </div>
                  <h3 className="text-sm font-bold text-error mb-2 flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-error/10 flex items-center justify-center">
                       <RefreshCw size={14} />
                    </div>
                    Protocol Reset
                  </h3>
                  <p className="text-[11px] text-muted leading-relaxed mb-6 italic opacity-70">
                    Purges all local identity traces. This does not affect rewards stored on the blockchain, but requires seed re-import.
                  </p>
                  <button className="nfm-btn nfm-btn--danger w-full py-4 text-[10px] font-bold uppercase tracking-widest">
                    Factory Identity Wipe
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'network' && (
            <div className="animate-in-slide flex flex-col gap-8">
              <div className="nfm-glass-card relative overflow-hidden">
                 {/* Background Accent */}
                <div className="absolute top-0 right-0 w-64 h-64 bg-purple/05 rounded-full blur-[100px] -mr-32 -mt-32"></div>
                
                <h2 className="text-xl font-bold flex items-center gap-3 text-primary mb-12">
                  <Globe className="text-purple drop-shadow-purple" /> Gateway Interface
                </h2>
                
                <div className="flex flex-col gap-10">
                  <div className="p-8 bg-surface-lowest/40 rounded-3xl border border-white/05 relative group">
                    <label className="text-[10px] font-bold uppercase tracking-[0.25em] text-muted mb-4 block">Mesh Discovery Entry Point</label>
                    <div className="flex gap-4">
                      <div className="relative group grow">
                        <div className="absolute left-4 top-1/2 -translate-y-1/2 flex items-center gap-2 text-muted">
                           <Server size={18} />
                           <span className="text-[10px] font-mono opacity-50 px-2 py-0.5 bg-white/05 rounded">PROMPT_</span>
                        </div>
                        <input 
                          type="text" 
                          className="nfm-input pl-32 font-mono text-sm bg-black/40 h-14 border-white/10 focus:border-purple/60 focus:bg-white/03 shadow-inner"
                          defaultValue={DUMMY_USER.settings?.rpc}
                        />
                      </div>
                      <button className="nfm-btn nfm-btn--primary px-10 h-14 font-bold tracking-widest uppercase text-xs">
                        Relink Node
                      </button>
                    </div>
                    <div className="flex items-center gap-2 mt-4 ml-2">
                       <div className="w-1 h-1 rounded-full bg-success"></div>
                       <span className="text-[9px] text-muted uppercase font-bold tracking-widest">Shard Sync: 100% Operational</span>
                    </div>
                  </div>

                  <div>
                    <h3 className="text-xs font-bold uppercase tracking-[0.25em] text-muted mb-6 flex items-center gap-3 ml-2">
                       Shard Proximity Map
                       <div className="h-[1px] grow bg-white/05"></div>
                    </h3>
                    <div className="grid grid-cols-1 sm:grid-cols-3 gap-5">
                      {[
                        { id: 'local', name: 'Shard_01_Local', loc: 'Current Node', lat: '4ms', color: 'cyan', active: true },
                        { id: 'dev', name: 'Mesh_Alpha_Net', loc: 'Berlin, DE', lat: '120ms', color: 'purple', active: false },
                        { id: 'main', name: 'Validator_Z_Core', loc: 'Global Cluster', lat: '45ms', color: 'gold', active: false },
                      ].map(shard => (
                        <div key={shard.id} className={`p-6 rounded-[1.5rem] border transition-all duration-300 cursor-pointer relative group flex flex-col gap-4 ${
                          shard.active ? 'border-cyan bg-cyan/05 shadow-[0_4px_30px_rgba(0,245,255,0.05)]' : 'border-white/05 bg-white/02 hover:bg-white/04 hover:border-white/10'}`}>
                           
                           <div className="flex justify-between items-start">
                              <div className="flex-col">
                                 <div className={`text-xs font-bold uppercase tracking-tight ${shard.active ? 'text-primary' : 'text-muted'}`}>{shard.id}</div>
                                 <div className={`text-[10px] font-bold mt-1 ${shard.active ? 'text-cyan' : 'text-white/40'}`}>{shard.name}</div>
                              </div>
                              {shard.active ? (
                                <div className="w-6 h-6 rounded-lg bg-cyan text-black flex items-center justify-center">
                                   <CheckCircle size={14} />
                                </div>
                              ) : (
                                <div className="w-6 h-6 rounded-lg border border-white/05 flex items-center justify-center opacity-20">
                                   <Activity size={12} />
                                </div>
                              )}
                           </div>

                           <div className="mt-4 flex items-end justify-between">
                              <div className="flex flex-col gap-1">
                                 <div className="text-[8px] text-muted/60 uppercase font-bold tracking-widest leading-none">Region</div>
                                 <div className="text-[10px] text-white/50">{shard.loc}</div>
                              </div>
                              <div className="flex flex-col items-end gap-1">
                                 <div className="text-[8px] text-muted/60 uppercase font-bold tracking-widest leading-none">Latency</div>
                                 <div className={`text-[11px] font-mono font-bold ${shard.active ? 'text-cyan' : 'text-white/60'}`}>{shard.lat}</div>
                              </div>
                           </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'prefs' && (
            <div className="animate-in-slide flex flex-col gap-8">
              <div className="nfm-glass-card shadow-glow-pink">
                <h2 className="text-xl font-bold mb-12 flex items-center gap-3 text-primary">
                  <Bell className="text-pink drop-shadow-pink" /> Neural Interface Tuning
                </h2>
                
                <div className="flex flex-col gap-12">
                  <section>
                    <div className="flex items-center gap-4 mb-8">
                       <h3 className="text-xs font-bold uppercase tracking-[0.25em] text-muted">Aesthetic Synchronization</h3>
                       <div className="h-[1px] grow bg-white/05"></div>
                    </div>
                    <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
                      {[
                        { id: 'mesh', name: 'Dark Mesh', desc: 'Protocol Default', active: true, color: 'cyan', icon: <Moon size={24} className="text-cyan"/> },
                        { id: 'dark', name: 'Titanium', desc: 'Sober Matrix', active: false, color: 'muted', icon: <Database size={24} className="text-muted"/> },
                        { id: 'light', name: 'Stellar', desc: 'Light Shard', active: false, color: 'gold', icon: <Sun size={24} className="text-gold"/> },
                      ].map(theme => (
                        <div key={theme.id} className={`p-8 rounded-[2rem] border flex flex-col items-center gap-4 cursor-pointer transition-all duration-500 relative group overflow-hidden ${
                          theme.active ? 'border-cyan bg-cyan/05 shadow-[0_8px_40px_rgba(0,245,255,0.1)] scale-[1.02]' : 'border-white/05 bg-white/01 hover:bg-white/03'}`}>
                           
                           {/* Background Glow Overlay */}
                           {theme.active && <div className="absolute inset-0 bg-gradient-to-b from-cyan/10 to-transparent opacity-50"></div>}
                           
                           <div className={`p-5 rounded-3xl transition-all duration-500 relative z-10 ${theme.active ? 'bg-cyan/10 ring-4 ring-cyan/05 rotate-[-5deg]' : 'bg-white/05 group-hover:bg-white/10'}`}>
                              {theme.icon}
                           </div>
                           <div className="relative z-10 text-center">
                              <div className={`text-xs font-bold tracking-tight ${theme.active ? 'text-primary' : 'text-muted'}`}>{theme.name}</div>
                              <div className="text-[9px] text-muted opacity-40 uppercase font-bold tracking-widest mt-1">{theme.desc}</div>
                           </div>
                           
                           {theme.active && (
                             <div className="mt-2 text-[7px] bg-cyan text-black px-3 py-1 rounded-full font-bold uppercase tracking-widest relative z-10 shadow-glow-cyan">Active Protocol</div>
                           )}
                        </div>
                      ))}
                    </div>
                  </section>

                  <section className="pt-12 border-t border-white/05">
                     <div className="flex items-center gap-4 mb-10">
                        <h3 className="text-xs font-bold uppercase tracking-[0.25em] text-muted">Priority Telemetry Alerts</h3>
                        <div className="h-[1px] grow bg-white/05"></div>
                     </div>
                    <div className="flex flex-col gap-4">
                      {[
                         { id: 'rewards', label: 'Inbound Reward Flow', desc: 'Acoustic and visual pings when NVC fragments arrive at your local validator address.' },
                         { id: 'network', label: 'Peer Sync Latency Filter', desc: 'Monitors the health of your connected mesh nodes. Alert on shard disconnect.' },
                         { id: 'security', label: 'Neural Watchdog Protocol', desc: 'Automated defense alerts for unauthorized transaction-ready state attempts.' }
                      ].map(ntf => (
                        <NeuralSwitch 
                          key={ntf.id} 
                          active={true} 
                          label={ntf.label} 
                          description={ntf.desc} 
                        />
                      ))}
                    </div>
                  </section>
                </div>
              </div>
            </div>
          )}

        </div>
      </div>
    </div>
  );
};

export default Settings;
