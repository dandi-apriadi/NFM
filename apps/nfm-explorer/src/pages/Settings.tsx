import { useEffect, useState } from 'react';
import { 
  Shield, 
  Key, 
  Globe, 
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
  Activity,
  RefreshCw,
  Lock,
  Wifi,
  AlertTriangle
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';
import * as client from '../api/client';

const SETTINGS_TAB_KEY = 'nfm.settings.activeTab';
const SETTINGS_TABS = ['security', 'network', 'prefs'] as const;

// Sub-component: Neural Toggle Switch (Standardized)
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
    className={`nfm-glass-card p-6 flex justify-between items-center transition-all duration-500 group cursor-pointer ${
      active ? 'border-cyan-40 bg-cyan-05 shadow-glow-cyan--soft' : 'hover:border-white-10'
    }`}
    onClick={onToggle}
    style={{ marginBottom: 'var(--space-4)' }}
  >
    <div className="flex flex-col gap-1.5 pr-4">
      <div className={`text-sm font-bold tracking-tight transition-colors ${active ? 'text-primary' : 'text-muted'}`}>{label}</div>
      <div className="text-10px text-muted leading-relaxed opacity-60 font-medium">{description}</div>
    </div>
    
    <div className={`w-12 h-6 rounded-full border relative p-1 transition-all duration-500 ${
      active ? 'bg-cyan-10 border-cyan-40' : 'bg-black-40 border-white-10'}`}>
       <div className={`absolute top-1 w-4 h-4 rounded-full transition-all duration-500 ${
         active ? 'right-1 bg-cyan shadow-glow-cyan' : 'right-7 bg-zinc-600'}`}></div>
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
    notifySuccess('Configuration backup exported');
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

  const handleBlockchainReset = async () => {
    const secret = await requestPrompt({
      title: '[DEV] Reset Blockchain to Genesis',
      message: 'Enter API secret to reset all blockchain state to genesis (IRREVERSIBLE):',
      placeholder: 'API Secret',
      confirmText: 'Reset',
    });
    if (secret) {
      try {
        notifySuccess('Resetting blockchain to genesis...');
        await client.appAdminResetBlockchain(secret);
        notifySuccess('✅ Blockchain reset complete! All state cleared to genesis.');
        setTimeout(() => window.location.reload(), 2000);
      } catch (err) {
        notifyError(`⚠️ Reset failed: ${err instanceof Error ? err.message : String(err)}`);
      }
    }
  };

  return (
    <div className="animate-in max-w-7xl mx-auto pb-24">
      {/* Immersive Header Section */}
      <div className="relative mb-12 pt-4">
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
              <div className="flex items-center gap-3 mt-1.5 font-mono text-[9px] font-bold tracking-[0.2em] text-muted opacity-60">
                 <span>PROTOCOL: ENCLAVE_V1.2</span>
                 <span className="text-white-10">|</span>
                 <span className="text-cyan-60">MESH_DISCOVERY: ACTIVE</span>
              </div>
            </div>
          </div>
          
          <div className="flex items-center gap-3">
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
          <div className="text-9px font-bold text-muted uppercase tracking-widest ml-2 mb-2 opacity-50">Discovery Command Groups</div>
          {[
            { id: 'security', label: 'Security Hub', icon: <Shield size={18} />, status: 'SECURED', color: 'cyan' },
            { id: 'network', label: 'Network & Shard', icon: <Globe size={18} />, status: 'SYNCED', color: 'purple' },
            { id: 'prefs', label: 'Preferences', icon: <Bell size={18} />, status: 'READY', color: 'pink' },
          ].map((tab) => (
            <button 
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`group flex items-center gap-4 p-4 rounded-2xl transition-all duration-500 text-left border relative overflow-hidden ${
                activeTab === tab.id 
                  ? `bg-surface-lowest border-${tab.color}-40 shadow-glow-${tab.color}--soft` 
                  : 'bg-surface-lowest border-transparent hover:bg-white-03 hover:border-white-10'
              }`}
            >
              <div className={`relative z-10 w-9 h-9 flex items-center justify-center rounded-xl transition-all duration-500 ${
                activeTab === tab.id ? `bg-${tab.color} text-black` : 'bg-surface-highest text-muted group-hover:text-primary'
              }`}>
                {tab.icon}
              </div>
              <div className="relative z-10">
                <div className={`text-xs font-bold transition-colors ${activeTab === tab.id ? 'text-primary' : 'text-muted'}`}>{tab.label}</div>
                <div className="text-9px text-muted opacity-40 uppercase font-bold tracking-widest mt-0.5">{tab.status}</div>
              </div>
              
              {activeTab === tab.id && (
                <div className={`absolute left-0 top-2 bottom-2 w-1 rounded-full bg-${tab.color} shadow-glow-${tab.color}`}></div>
              )}
            </button>
          ))}

          <div className="mt-8 nfm-glass-card border-white-05 bg-surface-lowest-20 p-6">
             <div className="relative flex items-center justify-between mb-4">
                <div className="w-8 h-8 rounded-lg bg-success-10 flex items-center justify-center">
                   <ShieldAlert size={16} className="text-success" />
                </div>
                <span className="text-10px text-primary font-bold uppercase tracking-widest">Enclave Status</span>
             </div>
             <div className="flex flex-col gap-4">
                <p className="text-[10px] text-muted leading-relaxed">
                   Advanced End-to-End Encryption is active. All node-specific metadata is stored in a sandboxed hardware module.
                </p>
                <div className="flex flex-col gap-1 pt-4 border-t border-white-05">
                   <div className="flex items-center justify-between">
                     <span className="text-[9px] text-white-40 font-mono uppercase tracking-widest">Enc_Standard</span>
                     <span className="text-[9px] text-primary font-mono">AES-256-GCM</span>
                   </div>
                   <div className="flex items-center justify-between">
                     <span className="text-[9px] text-white-40 font-mono uppercase tracking-widest">Hardware_Acc</span>
                     <span className="text-[9px] text-success font-mono">ENABLED</span>
                   </div>
                </div>
             </div>
          </div>
        </div>

        {/* Content Area (9 cols) */}
        <div className="col-span-12 lg:col-span-9 flex flex-col gap-6">
          
          {activeTab === 'security' && (
            <div className="animate-in-slide flex flex-col gap-8">
              <div className="nfm-glass-card nfm-glass-card--glow-cyan shadow-glow-cyan border overflow-hidden p-0 relative">
                <div className="absolute inset-0 opacity-[0.03] pointer-events-none" 
                     style={{ backgroundImage: 'radial-gradient(var(--neon-cyan) 0.5px, transparent 0.5px)', backgroundSize: '20px 20px' }}></div>
                
                <div className="p-10 relative z-10">
                  <div className="flex items-center justify-between mb-8">
                    <div className="flex items-start gap-4">
                       <div className="w-14 h-14 rounded-2xl bg-cyan-10 flex items-center justify-center border border-cyan-20">
                          <Key className="text-cyan" size={28} style={{ filter: 'drop-shadow(0 0 15px rgba(0,229,255,0.5))' }} />
                       </div>
                       <div>
                          <h2 className="text-xl font-display font-bold text-primary mb-1">Mnemonic Identity Fragment</h2>
                          <div className="flex items-center gap-3">
                             <div className="px-2 py-0.5 rounded bg-warning-10 text-[9px] text-warning font-bold uppercase tracking-widest border border-warning-20">
                                Critically Sensitive Data
                             </div>
                          </div>
                       </div>
                    </div>
                  </div>

                  <p className="text-sm text-muted mb-10 max-w-2xl leading-relaxed">
                    The mnemonic fragment below is the master decryption key for your Neural Mesh identity. 
                    Anyone with these words can permanently claim your node rewards and staked assets. 
                    <span className="text-white-60 block mt-2 font-bold italic opacity-80">!! DO NOT SHARE THIS WITH ANYONE !!</span>
                  </p>

                  <div className="relative group bg-surface-lowest-60 rounded-[2.5rem] border border-white-10 p-2 flex flex-col overflow-hidden shadow-2xl">
                    <div className={`relative p-12 py-16 font-mono text-lg leading-relaxed text-center tracking-[0.2em] transition-all duration-700 rounded-[2rem] overflow-hidden ${!showSeed ? 'blur-2xl select-none grayscale scale-[0.98] bg-black-60' : 'bg-surface-lowest shadow-inner'}`}>
                      {showSeed ? (
                        <div className="flex flex-wrap items-center justify-center gap-4">
                           {activeSeed?.split(' ').map((word: string, i: number) => (
                             <div key={i} className="flex flex-col items-center gap-1 min-w-[100px] py-4 bg-white-03 rounded-2xl border border-white-05 group/word hover:border-cyan-40 hover:bg-cyan-10 transition-all cursor-default">
                                <span className="text-[8px] text-white-40 font-mono tracking-tight uppercase">{i + 1}</span>
                                <span className="text-primary group-hover/word:text-cyan transition-colors text-base font-bold">{word}</span>
                             </div>
                           ))}
                        </div>
                      ) : (
                        '•••• •••• •••• •••• •••• •••• •••• •••• •••• •••• •••• ••••'
                      )}
                      
                      {showSeed && (
                        <div className="absolute top-0 left-0 w-full h-1 bg-cyan-40 animate-scanline pointer-events-none shadow-glow-cyan"></div>
                      )}
                    </div>
                    
                    {!showSeed && (
                      <div className="absolute inset-0 flex flex-col items-center justify-center gap-4">
                        <button 
                          className="nfm-btn nfm-btn--primary px-12 py-4 h-16 rounded-2xl shadow-glow-cyan--soft animate-float"
                          onClick={() => setShowSeed(true)}
                        >
                          <div className="flex items-center gap-4">
                             <Lock size={20} />
                             <span className="font-display font-bold tracking-widest text-lg uppercase">Decrypt Vault</span>
                          </div>
                        </button>
                        <button 
                          className="nfm-btn nfm-btn--ghost text-[10px] tracking-widest uppercase mt-4 border-none opacity-50 hover:opacity-100 transition-all"
                          onClick={handleImportSeed}
                        >
                          Import External Identity
                        </button>
                      </div>
                    )}
                    
                    {showSeed && (
                      <div className="flex items-center justify-between p-4 px-8 bg-surface-lowest border-t border-white-10 mt-1 rounded-b-[2rem]">
                        <div className="flex items-center gap-3">
                           <div className="w-2 h-2 rounded-full bg-warning animate-pulse shadow-glow-warning"></div>
                           <span className="text-[10px] text-warning font-bold uppercase tracking-widest">Active Decryption Mode</span>
                        </div>
                        <div className="flex gap-3">
                          <button 
                            className="nfm-btn nfm-btn--ghost px-6 h-11 border-white-10 rounded-xl text-xs font-bold transition-all group/copy"
                            onClick={() => handleCopy(activeSeed || '')}
                          >
                            {copied ? (
                              <><CheckCircle size={16} className="text-success" /> Copied</>
                            ) : (
                              <><Copy size={16} className="text-muted group-hover/copy:text-primary transition-colors" /> Copy</>
                            )}
                          </button>
                          <button 
                            className="nfm-btn nfm-btn--ghost px-6 h-11 border-white-10 rounded-xl text-xs font-bold text-muted hover:text-primary"
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
                <div className="nfm-glass-card group hover:border-cyan-40 hover:bg-cyan-05 transition-all duration-500 overflow-hidden relative">
                  <div className="absolute top-0 right-0 p-8 opacity-[0.03] group-hover:opacity-10 transition-opacity">
                     <Database size={80} />
                  </div>
                  <div className="relative z-10">
                    <h3 className="text-sm font-bold text-primary mb-3 flex items-center gap-3">
                       <div className="w-8 h-8 rounded-lg bg-cyan-10 flex items-center justify-center text-cyan">
                          <Download size={14} />
                       </div>
                       Protocol Backup
                    </h3>
                    <p className="text-11px text-muted leading-relaxed mb-8">
                      Download an AES-GCM encrypted container containing your local node configuration and encrypted shards.
                    </p>
                    <button className="nfm-btn nfm-btn--ghost w-full py-4 text-[10px] font-bold uppercase tracking-widest border-white-10 group-hover:border-cyan-30 group-hover:bg-cyan-10" onClick={handleExportConfig}>
                      Export Ident_Archive (.nfm)
                    </button>
                  </div>
                </div>

                <div className="nfm-glass-card group hover:border-error-40 hover:bg-error-02 transition-all duration-500 overflow-hidden relative">
                  <div className="absolute top-0 right-0 p-8 opacity-[0.03] group-hover:opacity-10 transition-opacity">
                     <ShieldAlert size={80} />
                  </div>
                  <div className="relative z-10">
                    <h3 className="text-sm font-bold text-error mb-3 flex items-center gap-3">
                       <div className="w-8 h-8 rounded-lg bg-error-10 flex items-center justify-center text-error">
                          <RefreshCw size={14} />
                       </div>
                       Security Purge
                    </h3>
                    <p className="text-11px text-muted leading-relaxed mb-8 italic opacity-70">
                      Instantly purge all local identity traces. This does not affect rewards stored on-chain.
                    </p>
                    <button 
                      className="nfm-btn nfm-btn--danger w-full py-4 text-[10px] font-bold uppercase tracking-widest"
                      onClick={handleFactoryWipe}
                    >
                      Factory Identity Wipe
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'network' && (
            <div className="animate-in-slide flex flex-col gap-8">
              <div className="nfm-glass-card nfm-glass-card--glow-purple relative overflow-hidden">
                <div className="absolute top-0 right-0 w-64 h-64 bg-purple-05 rounded-full blur-xl -mr-32 -mt-32"></div>
                
                <h2 className="text-xl font-bold flex items-center gap-3 text-primary mb-12">
                   <Globe className="text-purple drop-shadow-purple" /> Network Interface
                </h2>
                
                <div className="flex flex-col gap-10">
                  <div className="p-8 bg-surface-lowest-40 rounded-3xl border border-white-05 relative group">
                    <label className="text-[10px] font-bold uppercase tracking-[0.2em] text-muted mb-4 block">Gateway Discovery Entry</label>
                    <div className="flex gap-4">
                      <div className="relative group grow">
                        <div className="absolute left-4 top-1/2 -translate-y-1/2 flex items-center gap-2 text-muted">
                           <Wifi size={18} className="text-purple opacity-50" />
                        </div>
                        <input 
                          type="text" 
                          className="nfm-input pl-12 font-mono text-sm bg-black-40 h-14 border-white-10 focus:border-purple-60 focus:bg-white-03"
                          value={localRpc}
                          onChange={(e) => setLocalRpc(e.target.value)}
                        />
                      </div>
                      <button 
                        className="nfm-btn nfm-btn--primary px-10 h-14 font-bold tracking-widest uppercase text-xs shadow-glow-purple--soft"
                        onClick={() => {
                          void updateSettings({ rpc: localRpc });
                        }}
                      >
                        Active Relink
                      </button>
                    </div>
                  </div>

                  <div>
                    <h3 className="text-[10px] font-bold uppercase tracking-[0.3em] text-muted mb-6 flex items-center gap-4 ml-2">
                       Shard Connectivity Map
                       <div className="h-px grow bg-white-05"></div>
                    </h3>
                    <div className="grid grid-cols-1 sm:grid-cols-3 gap-5">
                      {[
                        { id: 'shard_01', name: 'Neural_Core_DE', loc: 'Current Node', lat: '4ms', status: 'ACTIVE', color: 'cyan' },
                        { id: 'shard_02', name: 'Mesh_Global_02', loc: 'North America', lat: '120ms', status: 'STANDBY', color: 'purple' },
                        { id: 'shard_03', name: 'Elite_Validator', loc: 'Asian Shard', lat: '45ms', status: 'STANDBY', color: 'purple' },
                      ].map(shard => (
                        <div key={shard.id} className={`p-6 rounded-3xl border transition-all duration-500 group flex flex-col gap-4 ${
                          shard.status === 'ACTIVE' ? 'border-cyan-40 bg-cyan-05' : 'border-white-05 bg-surface-lowest hover:border-white-20'}`}>
                           
                           <div className="flex justify-between items-start">
                              <div className="flex flex-col">
                                 <div className={`text-[10px] font-bold tracking-widest ${shard.status === 'ACTIVE' ? 'text-primary' : 'text-muted'}`}>{shard.id.toUpperCase()}</div>
                                 <div className={`text-xs font-bold mt-1 ${shard.status === 'ACTIVE' ? 'text-cyan' : 'text-white-40'}`}>{shard.name}</div>
                              </div>
                              <div className={`w-6 h-6 rounded-lg flex items-center justify-center ${shard.status === 'ACTIVE' ? 'bg-cyan text-black' : 'border border-white-10 text-white-10'}`}>
                                 {shard.status === 'ACTIVE' ? <CheckCircle size={14} /> : <Activity size={12} />}
                              </div>
                           </div>

                           <div className="mt-4 flex items-end justify-between border-t border-white-05 pt-4">
                              <div className="flex flex-col gap-1">
                                 <div className="text-[8px] text-white-40 uppercase font-black tracking-widest">Region</div>
                                 <div className="text-[10px] text-primary">{shard.loc}</div>
                              </div>
                              <div className="flex flex-col items-end gap-1">
                                 <div className="text-[8px] text-white-40 uppercase font-black tracking-widest">Lat</div>
                                 <div className={`text-[11px] font-mono font-bold ${shard.status === 'ACTIVE' ? 'text-cyan' : 'text-muted'}`}>{shard.lat}</div>
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
              <div className="nfm-glass-card nfm-glass-card--glow-purple">
                <h2 className="text-xl font-bold mb-12 flex items-center gap-3 text-primary">
                   <Bell className="text-pink drop-shadow-pink" /> Interface Parameters
                </h2>
                
                <div className="flex flex-col gap-12">
                  <section>
                    <div className="flex items-center gap-4 mb-8">
                       <h3 className="text-[10px] font-bold uppercase tracking-[0.25em] text-muted">Aesthetic Tuning</h3>
                       <div className="h-px grow bg-white-05"></div>
                    </div>
                    <div className="grid grid-cols-3 gap-5">
                      {[
                        { id: 'mesh', name: 'Dark Mesh', icon: <Moon size={22} />, desc: 'Standard' },
                        { id: 'dark', name: 'Titanium', icon: <Database size={22} />, desc: 'High Contrast' },
                        { id: 'light', name: 'Stellar', icon: <Sun size={22} />, desc: 'Light' },
                      ].map(themeItem => {
                        const active = localTheme === themeItem.id;
                        return (
                        <button 
                          key={themeItem.id} 
                          className={`p-6 py-10 rounded-3xl border flex flex-col items-center gap-6 transition-all duration-500 relative group ${
                            active ? 'border-cyan-40 bg-cyan-05 shadow-glow-cyan--soft' : 'border-white-05 bg-surface-lowest hover:border-white-10'
                          }`}
                          onClick={() => {
                            setLocalTheme(themeItem.id);
                            void updateSettings({ theme: themeItem.id as any });
                          }}
                        >
                           <div className={`p-4 rounded-2xl transition-all duration-500 ${active ? 'bg-cyan text-black' : 'bg-white-02 text-muted'}`}>
                              {themeItem.icon}
                           </div>
                           <div className="text-center">
                              <div className={`text-xs font-bold tracking-tight mb-1 ${active ? 'text-primary' : 'text-muted'}`}>{themeItem.name}</div>
                              <div className="text-[9px] text-muted opacity-40 uppercase font-black tracking-widest">{themeItem.desc}</div>
                           </div>
                        </button>
                      )})}
                    </div>
                  </section>

                  <section className="pt-12 border-t border-white-05">
                     <div className="flex items-center gap-4 mb-10">
                        <h3 className="text-[10px] font-bold uppercase tracking-[0.25em] text-muted">Notification Sub-Sys</h3>
                        <div className="h-px grow bg-white-05"></div>
                     </div>
                    <div className="flex flex-col gap-4">
                      {[
                         { id: 'rewards', label: 'Reward Distribution Alerts', desc: 'Notify when NVC mining rewards or mission claims are confirmed on-chain.' },
                         { id: 'network', label: 'Shard Connectivity Events', desc: 'Real-time alerts for peer discovery, syncing, and mesh health status changes.' },
                         { id: 'security', label: 'Neural Firewall Triggers', desc: 'Security alerts for unauthorized signature attempts or remote login attempts.' }
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

                  <section className="pt-12 border-t border-warning-20 bg-warning-05 rounded-2xl p-8">
                     <div className="flex items-center gap-4 mb-10">
                        <AlertTriangle size={20} className="text-warning" />
                        <h3 className="text-[10px] font-bold uppercase tracking-[0.25em] text-warning">Development Tools (Temporary)</h3>
                        <div className="h-px grow bg-white-05"></div>
                     </div>
                     <p className="text-[10px] text-muted mb-8 italic opacity-60">
                        These tools are for development/testing only and will be removed before production. Use with caution!
                     </p>
                    <div className="flex gap-4">
                      <button 
                        className="flex-1 nfm-btn nfm-btn--danger py-4 px-6 text-xs font-bold uppercase tracking-widest flex items-center justify-center gap-3 rounded-2xl transition-all duration-300"
                        onClick={handleBlockchainReset}
                      >
                        <RefreshCw size={16} />
                        Reset Blockchain to Genesis
                      </button>
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
