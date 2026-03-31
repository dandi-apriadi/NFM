import { useEffect, useState } from 'react';
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
  ChevronRight,
  ShieldAlert,
  Server,
  Activity,
  RefreshCw
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';

const SETTINGS_TAB_KEY = 'nfm.settings.activeTab';
const SETTINGS_TABS = ['security', 'network', 'prefs'] as const;

// Sub-component: Neural Toggle Switch
const NeuralSwitch = ({ 
  active, 
  label, 
  description, 
  onToggle 
}: { 
  active: boolean, 
  label: string, 
  description: string,
  onToggle?: () => void 
}) => (
  <div 
    className={`flex justify-between items-center p-6 bg-surface-lowest-30 rounded-2xl border transition-all duration-500 group cursor-pointer ${
      active ? 'border-cyan-20 shadow-[inset_0_0_20px_rgba(0,229,255,0.03)]' : 'border-white-05 hover:border-white-10'
    }`}
    onClick={onToggle}
  >
    <div className="flex flex-col gap-1.5 pr-4">
      <div className={`text-sm font-bold tracking-tight transition-colors ${active ? 'text-primary' : 'text-muted'}`}>{label}</div>
      <div className="text-10px text-muted leading-relaxed opacity-60 font-medium">{description}</div>
    </div>
    
    <div className={`w-12 h-6 rounded-full border relative p-1 transition-all duration-500 ${
      active ? 'bg-cyan-10 border-cyan-40 shadow-glow-cyan--soft' : 'bg-black-40 border-white-10'}`}>
       {/* Knob */}
       <div className={`absolute top-1 w-4 h-4 rounded-full transition-all duration-500 shadow-xl ${
         active ? 'right-1 bg-cyan shadow-[0_0_10px_rgba(0,229,255,1)]' : 'right-7 bg-zinc-600'}`}></div>
    </div>
  </div>
);

const Settings = () => {
  const navigate = useNavigate();
  const { data, updateSettings, requestPrompt, requestConfirm, notifySuccess, notifyError } = useAppData();
  const DUMMY_USER = data.user_profile;
  const [activeTab, setActiveTab] = useState<'security' | 'network' | 'prefs'>(() => {
    const saved = localStorage.getItem(SETTINGS_TAB_KEY);
    return SETTINGS_TABS.includes(saved as (typeof SETTINGS_TABS)[number])
      ? (saved as 'security' | 'network' | 'prefs')
      : 'security';
  });
  const [showSeed, setShowSeed] = useState(false);
  const [copied, setCopied] = useState(false);

  const [localSeed, setLocalSeed] = useState(() => localStorage.getItem('nfm.user.seedPhrase') || '');
  const activeSeed = localSeed || DUMMY_USER.seedPhrase;

  const [localRpc, setLocalRpc] = useState(() => localStorage.getItem('nfm.settings.rpc') || DUMMY_USER.settings?.rpc || 'http://127.0.0.1:3000');
  const [localTheme, setLocalTheme] = useState(() => localStorage.getItem('nfm.settings.theme') || 'mesh');

  useEffect(() => {
    localStorage.setItem(SETTINGS_TAB_KEY, activeTab);
  }, [activeTab]);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', localTheme);
  }, [localTheme]);

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

  const handleImportSeed = async () => {
    const nextSeed = await requestPrompt({
      title: 'Import Seed Phrase',
      message: 'Enter 12-word recovery mnemonic',
      placeholder: 'word word word ...',
      confirmText: 'Import',
    });
    if (nextSeed) {
      if (nextSeed.trim().split(' ').length < 12) {
        notifyError('Invalid seed phrase. Must be 12 words.');
        return;
      }
      localStorage.setItem('nfm.user.seedPhrase', nextSeed.trim());
      setLocalSeed(nextSeed.trim());
      notifySuccess('Mnemonic imported and secured locally');
    }
  };

  const handleFactoryWipe = async () => {
    const confirm = await requestConfirm({
      title: 'Factory Identity Wipe',
      message: 'This will purge all local keys and settings. Are you sure?',
      confirmText: 'Yes, Wipe Everything',
    });
    if (confirm) {
      localStorage.clear();
      setLocalSeed('');
      notifySuccess('Factory wipe complete. Local data purged.');
      setTimeout(() => window.location.reload(), 1500);
    }
  };

  return (
    <div className="animate-in max-w-7xl mx-auto pb-24">
      {/* Immersive Header Section */}
      <div className="relative mb-12 pt-4">
        {/* Ambient Glows */}
        <div className="absolute -top-24 left-1/4 w-2px bg-cyan-05 blur-xl pointer-events-none rounded-full" style={{ filter: 'blur(120px)' }}></div>
        <div className="absolute top-0 right-1/4 bg-purple-03 blur-lg pointer-events-none rounded-full" style={{ width: '400px', height: '200px', filter: 'blur(100px)' }}></div>
        
        <div className="relative z-10 flex items-end justify-between border-b border-white-05 pb-8">
          <div className="flex items-center gap-6">
            <button 
              className="nfm-btn nfm-btn--ghost w-11 h-11 p-0 flex items-center justify-center rounded-xl bg-white-02 border-white-05 hover:border-cyan-40 hover:bg-cyan-05 transition-all" 
              onClick={() => navigate(-1)}
            >
              <ChevronRight size={20} style={{ transform: 'rotate(180deg)' }} />
            </button>
            
            <div className="flex flex-col">
              <div className="flex items-center gap-3">
                 <Zap size={22} className="text-cyan animate-pulse" />
                 <h1 className="text-2xl font-display font-bold text-primary tracking-tight">System Core</h1>
              </div>
              <div className="flex items-center gap-3 mt-1.5 font-mono text-9px font-bold tracking-widest text-muted opacity-60">
                 <span>PROTOCOL: ENCLAVE_V1.2</span>
                 <span className="text-white-10">|</span>
                 <span className="text-cyan-60">MESH_DISCOVERY: ACTIVE</span>
              </div>
            </div>
          </div>
          
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-4 px-5 h-11 bg-surface-lowest-40 border border-white-05 rounded-xl">
               <div className="flex flex-col items-start pr-4 border-r border-white-05">
                  <span className="text-10px font-mono font-bold text-success">4.2ms</span>
               </div>
               <div className="flex flex-col items-start">
                  <span className="text-10px font-mono font-bold text-cyan">AX-712-B</span>
               </div>
            </div>
            
            <button 
              className="nfm-btn nfm-btn--primary h-11 px-6 gap-2 font-display font-bold tracking-wide shadow-glow-cyan--soft" 
              onClick={handleExportConfig}
            >
              <Download size={16} /> BACKUP ALL
            </button>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-12 gap-8 items-start">
        
        {/* Navigation Sidebar (3 cols) */}
        <div className="col-span-12 lg:col-span-3 flex flex-col gap-3">
          <div className="text-9px font-bold text-muted uppercase tracking-widest ml-2 mb-2 opacity-50">Protocol Command Groups</div>
          {[
            { id: 'security', label: 'Security Hub', icon: <Shield size={18} />, status: 'SECURED' },
            { id: 'network', label: 'Network & Shard', icon: <Globe size={18} />, status: 'SYNCED' },
            { id: 'prefs', label: 'Preferences', icon: <Bell size={18} />, status: 'READY' },
          ].map((tab) => (
            <button 
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`group flex items-center gap-4 p-4 rounded-2xl transition-all duration-500 text-left border relative overflow-hidden ${
                activeTab === tab.id 
                  ? 'bg-cyan-08 border-cyan-30 shadow-[0_4px_30px_rgba(0,229,255,0.08)]' 
                  : 'bg-surface-lowest border-transparent hover:bg-white-03 hover:border-white-10'
              }`}
            >
              <div className={`relative z-10 w-9 h-9 flex items-center justify-center rounded-xl transition-all duration-500 ${
                activeTab === tab.id ? 'bg-cyan text-black shadow-glow-cyan--soft' : 'bg-surface-highest text-muted group-hover:text-primary'
              }`}>
                {tab.icon}
              </div>
              <div className="relative z-10">
                <div className={`text-xs font-bold transition-colors ${activeTab === tab.id ? 'text-primary' : 'text-muted'}`}>{tab.label}</div>
                <div className="text-9px text-muted opacity-40 uppercase font-bold tracking-widest mt-0.5">{tab.status}</div>
              </div>
              
              {/* Scanline Side Indicator */}
              {activeTab === tab.id && (
                <div className="absolute left-0 top-2 bottom-2 w-2px bg-cyan" style={{ boxShadow: '0 0 10px rgba(0,245,255,1)' }}></div>
              )}
            </button>
          ))}

          <div className="mt-8 nfm-glass-card border border-white-05 bg-surface-lowest-20 p-6">
             <div className="relative flex items-center justify-between mb-4">
                <div className="w-8 h-8 rounded-lg bg-success-10 flex items-center justify-center">
                   <ShieldAlert size={16} className="text-success" />
                </div>
                <span className="text-10px text-primary font-bold uppercase tracking-widest">Enclave Status</span>
             </div>
             <div className="flex flex-col gap-4">
                <p className="text-10px text-muted leading-relaxed">
                   Advanced End-to-End Encryption is active. All node-specific metadata is stored in a sandboxed hardware module.
                </p>
                <div className="flex items-center justify-between pt-4 border-t border-white-05">
                   <div className="flex items-center gap-2">
                   <span className="text-[9px] text-white-40 font-mono">LOCAL_HARDWARE</span>
                   </div>
                </div>
                <div className="flex items-center justify-between">
                   <span className="text-9px text-white-40 font-mono">AES-256-GCM</span>
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
                    <div className="flex items-start gap-4">
                       <div className="w-14 h-14 rounded-2xl bg-cyan-10 flex items-center justify-center border border-cyan-20">
                          <Key className="text-cyan" size={28} style={{ filter: 'drop-shadow(0 0 15px rgba(0,229,255,0.5))' }} />
                       </div>
                       <div>
                          <h2 className="text-xl font-display font-bold text-primary mb-1">Discovery Mnemonic Fragment</h2>
                          <div className="flex items-center gap-3">
                             <div className="px-2 py-0.5 rounded bg-warning-10 text-9px text-warning font-bold uppercase tracking-widest border border-warning-20">
                                Critically Sensitive
                             </div>
                          </div>
                       </div>
                    </div>
                  </div>

                  <p className="text-sm text-muted mb-10 max-w-2xl leading-relaxed">
                    The 12-word mnemonic fragment below is the master decryption key for your Neural Mesh identity. 
                    Anyone with these words can permanently claim your node rewards and staked assets. 
                    <span className="text-white-60 block mt-2">Store this only in a trusted physical format or air-gapped hardware vault.</span>
                  </p>

                  <div className="relative group bg-surface-lowest-60 rounded-[2.5rem] border border-white-10 p-2 flex flex-col overflow-hidden shadow-2xl">
                    <div className="absolute top-0 left-0 w-full h-[600px] bg-cyan-08 blur-[200px] opacity-10 pointer-events-none"></div>
                    <div className="flex items-center justify-between px-10 py-6 relative z-10">
                      <div className="text-sm font-bold text-muted uppercase tracking-widest">Protocol Metadata</div>
                    </div>
                    
                    <div className={`relative p-12 py-16 font-mono text-lg leading-relaxed text-center tracking-[0.2em] transition-all duration-700 rounded-[2rem] overflow-hidden ${!showSeed ? 'blur-2xl select-none grayscale scale-[0.98] bg-black-60' : 'bg-surface-lowest shadow-inner'}`}>
                      {showSeed ? (
                        <div className="flex flex-wrap items-center justify-center gap-4">
                           {activeSeed?.split(' ').map((word: string, i: number) => (
                             <div key={i} className="flex flex-col items-center gap-1 min-w-[100px] py-3 bg-white-03 rounded-xl border border-white-05 group/word hover:border-cyan-40 hover:bg-cyan-10 transition-all cursor-default">
                                <span className="text-8px text-white-40 font-mono tracking-tight">{i + 1}</span>
                                <span className="text-primary group-hover/word:text-cyan transition-colors">{word}</span>
                             </div>
                           ))}
                        </div>
                      ) : (
                        '•••• •••• •••• •••• •••• •••• •••• •••• •••• •••• •••• ••••'
                      )}
                      
                      {/* Scanline Effect */}
                      {showSeed && (
                        <div className="absolute top-0 left-0 w-full h-2px bg-cyan-40 animate-scanline pointer-events-none" style={{ boxShadow: '0 0 15px rgba(0,245,255,0.5)' }}></div>
                      )}
                    </div>
                    
                    {!showSeed && (
                      <div className="absolute inset-0 flex flex-col items-center justify-center gap-4">
                        <button 
                          className="nfm-btn nfm-btn--primary px-12 py-4 h-16 rounded-2xl shadow-glow-cyan--soft animate-float"
                          onClick={() => setShowSeed(true)}
                        >
                          <div className="flex items-center gap-4">
                             <div className="w-8 h-8 rounded-lg bg-black-20 flex items-center justify-center">
                                <Eye size={18} />
                             </div>
                             <span className="font-display font-bold tracking-widest text-lg uppercase">Decrypt Fragment</span>
                          </div>
                        </button>
                        <button 
                          className="nfm-btn nfm-btn--ghost text-10px shadow-none tracking-widest uppercase mt-4 border-none opacity-50 hover:opacity-100 transition-all"
                          onClick={handleImportSeed}
                        >
                          Import Existing Mnemonic
                        </button>
                      </div>
                    )}
                    
                    {showSeed && (
                      <div className="flex items-center justify-between p-4 px-8 bg-surface-lowest border-t border-white-10 mt-1 rounded-b-[2rem]">
                        <div className="flex items-center gap-3">
                           <div className="w-2 h-2 rounded-full bg-warning animate-pulse" style={{ boxShadow: '0 0 8px rgba(255,184,0,0.8)' }}></div>
                           <span className="text-10px text-warning font-bold uppercase tracking-widest">Live Decryption Active</span>
                        </div>
                        <div className="flex gap-3">
                          <button 
                            className="flex items-center gap-3 px-6 h-11 bg-white-05 hover:bg-white-10 border border-white-10 rounded-xl text-xs font-bold transition-all group/copy"
                            onClick={() => handleCopy(activeSeed || '')}
                          >
                            {copied ? (
                              <><CheckCircle size={16} className="text-success" /> Copied</>
                            ) : (
                              <><Copy size={16} className="text-muted group-hover/copy:text-primary transition-colors" /> Copy phrase</>
                            )}
                          </button>
                          <button 
                            className="flex items-center gap-2 px-6 h-11 bg-white-05 hover:bg-white-10 border border-white-10 rounded-xl text-xs font-bold text-muted hover:text-primary transition-all"
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
                <div className="nfm-glass-card group hover:border-purple-40 hover:bg-purple-05 transition-all duration-500 overflow-hidden">
                  <div className="absolute top-0 right-0 p-6 opacity-05 group-hover:opacity-10 transition-opacity">
                     <Database size={60} />
                  </div>
                  <h3 className="text-sm font-bold text-primary mb-2 flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-surface-lowest flex items-center justify-center text-purple">
                       <Download size={14} />
                    </div>
                    Encrypted Vault Backup
                  </h3>
                  <p className="text-11px text-muted leading-relaxed mb-6">
                    Download an AES-GCM encrypted container containing your keys, node logic, and localized shard cache.
                  </p>
                  <button className="nfm-btn nfm-btn--ghost w-full py-4 text-10px font-bold uppercase tracking-widest border-white-05 group-hover:border-purple-30 group-hover:bg-purple-10" onClick={handleExportConfig}>
                    Export Secure Archive (.nfm)
                  </button>
                </div>

                <div className="nfm-glass-card group hover:border-error-40 hover:bg-error-02 transition-all duration-500 overflow-hidden">
                  <div className="p-6 pb-2">
                    <div className="w-8 h-8 rounded-lg bg-error-10 flex items-center justify-center">
                       <ShieldAlert size={60} />
                    </div>
                  </div>
                  <h3 className="text-sm font-bold text-error mb-2 flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-error-10 flex items-center justify-center">
                       <RefreshCw size={14} />
                    </div>
                    Protocol Reset
                  </h3>
                  <p className="text-11px text-muted leading-relaxed mb-6 italic opacity-70">
                    Purges all local identity traces. This does not affect rewards stored on the blockchain, but requires seed re-import.
                  </p>
                  <button 
                    className="nfm-btn nfm-btn--danger w-full py-4 text-10px font-bold uppercase tracking-widest"
                    onClick={handleFactoryWipe}
                  >
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
                <div className="absolute top-0 right-0 w-64 h-64 bg-purple-05 rounded-full blur-xl -mr-32 -mt-32"></div>
                
                <h2 className="text-xl font-bold flex items-center gap-3 text-primary mb-12">
                  <Globe className="text-purple drop-shadow-purple" /> Gateway Interface
                </h2>
                
                <div className="flex flex-col gap-10">
                  <div className="p-8 bg-surface-lowest-40 rounded-3xl border border-white-05 relative group">
                    <label className="text-10px font-bold uppercase tracking-widest text-muted mb-4 block">Mesh Discovery Entry Point</label>
                    <div className="flex gap-4">
                      <div className="relative group grow">
                        <div className="absolute left-4 top-1/2 -translate-y-1/2 flex items-center gap-2 text-muted">
                           <Server size={18} />
                           <span className="text-[10px] font-mono opacity-50 px-2 py-0.5 bg-white-05 rounded">PROMPT_</span>
                        </div>
                        <input 
                          type="text" 
                          className="nfm-input pl-32 font-mono text-sm bg-black-40 h-14 border-white-10 focus:border-purple-60 focus:bg-white-03 shadow-inner"
                          value={localRpc}
                          onChange={(e) => setLocalRpc(e.target.value)}
                        />
                      </div>
                        <button 
                          className="nfm-btn nfm-btn--primary px-10 h-14 font-bold tracking-widest uppercase text-xs"
                          onClick={() => {
                            void updateSettings({ rpc: localRpc });
                          }}
                        >
                          Relink Node
                        </button>
                    </div>
                    <div className="flex items-center gap-2 mt-4 ml-2">
                       <div className="w-1 h-1 rounded-full bg-success"></div>
                       <span className="text-9px text-muted uppercase font-bold tracking-widest">Shard Sync: 100% Operational</span>
                    </div>
                  </div>

                  <div>
                    <h3 className="text-xs font-bold uppercase tracking-[0.25em] text-muted mb-6 flex items-center gap-3 ml-2">
                       Shard Proximity Map
                       <div className="h-px grow bg-white-05"></div>
                    </h3>
                    <div className="grid grid-cols-1 sm:grid-cols-3 gap-5">
                      {[
                        { id: 'local', name: 'Shard_01_Local', loc: 'Current Node', lat: '4ms', color: 'cyan', active: true },
                        { id: 'dev', name: 'Mesh_Alpha_Net', loc: 'Berlin, DE', lat: '120ms', color: 'purple', active: false },
                        { id: 'main', name: 'Validator_Z_Core', loc: 'Global Cluster', lat: '45ms', color: 'gold', active: false },
                      ].map(shard => (
                        <div key={shard.id} className={`p-6 rounded-[1.5rem] border transition-all duration-300 cursor-pointer relative group flex flex-col gap-4 ${
                          shard.active ? 'border-cyan bg-cyan-05 shadow-[0_4px_30px_rgba(0,229,255,0.05)]' : 'border-white-05 bg-white-02 hover:bg-white-04 hover:border-white-10'}`}>
                           
                           <div className="flex justify-between items-start">
                              <div className="flex-col">
                                 <div className={`text-xs font-bold uppercase tracking-tight ${shard.active ? 'text-primary' : 'text-muted'}`}>{shard.id}</div>
                                 <div className={`text-10px font-bold mt-1 ${shard.active ? 'text-cyan' : 'text-white-40'}`}>{shard.name}</div>
                              </div>
                              {shard.active ? (
                                <div className="w-6 h-6 rounded-lg bg-cyan text-black flex items-center justify-center">
                                   <CheckCircle size={14} />
                                </div>
                              ) : (
                                <div className="w-6 h-6 rounded-lg border border-white-05 flex items-center justify-center opacity-20">
                                   <Activity size={12} />
                                </div>
                              )}
                           </div>

                           <div className="mt-4 flex items-end justify-between">
                              <div className="flex flex-col gap-1">
                                 <div className="text-8px text-muted-60 uppercase font-bold tracking-widest leading-none">Region</div>
                                 <div className="text-10px text-white-50">{shard.loc}</div>
                              </div>
                              <div className="flex flex-col items-end gap-1">
                                 <div className="text-8px text-muted-60 uppercase font-bold tracking-widest leading-none">Latency</div>
                                 <div className={`text-[11px] font-mono font-bold ${shard.active ? 'text-cyan' : 'text-white-60'}`}>{shard.lat}</div>
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
                       <div className="h-px grow bg-white-05"></div>
                    </div>
                    <div className="grid grid-cols-1 sm:grid-cols-3 gap-5">
                      {[
                        { id: 'mesh', name: 'Dark Mesh', desc: 'Protocol Standard' },
                        { id: 'dark', name: 'Titanium', desc: 'Sober Matrix' },
                        { id: 'light', name: 'Stellar', desc: 'Light Shard' },
                      ].map(themeItem => {
                        const theme = {
                          ...themeItem,
                          active: localTheme === themeItem.id,
                          icon: themeItem.id === 'mesh' ? <Moon size={22} className={localTheme === 'mesh' ? 'text-cyan' : 'text-muted'} /> :
                                themeItem.id === 'dark' ? <Database size={22} className={localTheme === 'dark' ? 'text-cyan' : 'text-muted'} /> :
                                <Sun size={22} className={localTheme === 'light' ? 'text-cyan' : 'text-muted'} />
                        };
                        return (
                        <button 
                          key={theme.id} 
                          className={`p-6 py-8 rounded-3xl border flex flex-col items-center gap-5 transition-all duration-300 relative group overflow-hidden ${
                            theme.active ? 'border-cyan-40 bg-cyan-05 shadow-[0_0_30px_rgba(0,229,255,0.05)] ring-1 ring-cyan-20' : 'border-white-05 bg-surface-lowest hover:border-white-10'
                          }`}
                          onClick={() => {
                            setLocalTheme(themeItem.id);
                            void updateSettings({ theme: themeItem.id as any });
                          }}
                        >
                           {/* Background Glow */}
                           {theme.active && <div className="absolute -top-12 -right-12 w-32 h-32 bg-cyan-10 blur-2xl opacity-60"></div>}
                           
                           <div className={`p-4 rounded-2xl transition-all duration-500 relative z-10 ${theme.active ? 'bg-cyan-10 border border-cyan-30' : 'bg-white-02 border border-transparent'}`} style={theme.active ? { boxShadow: '0 0 15px rgba(0,229,255,0.2)' } : undefined}>
                              {theme.icon}
                           </div>
                           <div className="relative z-10 text-center">
                              <div className={`text-xs font-bold tracking-tight mb-1 ${theme.active ? 'text-primary' : 'text-muted'}`}>{theme.name}</div>
                              <div className="text-8px text-muted opacity-40 uppercase font-bold tracking-widest">{theme.desc}</div>
                           </div>
                           
                           {theme.active && (
                             <div className="absolute top-3 right-3 text-9px text-cyan font-bold uppercase tracking-widest flex items-center gap-1 animate-in">
                                <div className="w-1 h-1 rounded-full bg-cyan animate-pulse"></div>
                                Active
                             </div>
                           )}
                        </button>
                      )})}
                    </div>
                  </section>

                  <section className="pt-12 border-t border-white-05">
                     <div className="flex items-center gap-4 mb-10">
                        <h3 className="text-xs font-bold uppercase tracking-[0.25em] text-muted">Priority Telemetry Alerts</h3>
                        <div className="h-px grow bg-white-05"></div>
                     </div>
                    <div className="flex flex-col gap-4">
                      {[
                         { id: 'rewards', label: 'Inbound Reward Flow', desc: 'Acoustic and visual pings when NVC fragments arrive at your local validator address.' },
                         { id: 'network', label: 'Peer Sync Latency Filter', desc: 'Monitors the health of your connected mesh nodes. Alert on shard disconnect.' },
                         { id: 'security', label: 'Neural Watchdog Protocol', desc: 'Automated defense alerts for unauthorized transaction-ready state attempts.' }
                      ].map(ntf => (
                        <NeuralSwitch 
                          key={ntf.id} 
                          active={!!DUMMY_USER.settings?.notifications?.[ntf.id as keyof typeof DUMMY_USER.settings.notifications]} 
                          label={ntf.label} 
                          description={ntf.desc} 
                          onToggle={() => {
                            const current = DUMMY_USER.settings?.notifications;
                            if (!current) return;
                            void updateSettings({
                              notifications: {
                                ...current,
                                [ntf.id]: !current[ntf.id as keyof typeof current]
                              }
                            });
                          }}
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
