import { useEffect, useState } from 'react';
import { Gift, Sparkles, ChevronRight, Info, Activity, Trophy, Globe, Zap, Terminal } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';
import { appExtractMystery } from '../api/client';

type ExtractionStage = 'CONNECTING_GATEWAY' | 'SCANNING_SHARD' | 'DECRYPTING_CARGO' | 'SUCCESS_FLASH';

const EXTRACTION_STAGES: ExtractionStage[] = [
   'CONNECTING_GATEWAY',
   'SCANNING_SHARD',
   'DECRYPTING_CARGO',
   'SUCCESS_FLASH',
];

const MYSTERY_TAB_KEY = 'nfm.mystery.activeTab';
const MYSTERY_TABS = ['Rewards', 'Feed', 'History'] as const;

const sleep = (ms: number) => new Promise((resolve) => window.setTimeout(resolve, ms));

const MysteryBox = () => {
  const navigate = useNavigate();
   const { data, refresh, notifySuccess, notifyError } = useAppData();
   const DUMMY_BOX_HISTORY = data.box_history;
   const DUMMY_USER = data.user_profile;
   const DUMMY_REWARD_CATALOG = data.reward_catalog;
   const DUMMY_MYSTERY_NEWS = data.mystery_news;

  const [isExtracting, setIsExtracting] = useState(false);
  const [extractionStage, setExtractionStage] = useState<ExtractionStage | null>(null);
   const [activeTab, setActiveTab] = useState<'Rewards' | 'Feed' | 'History'>(() => {
      const saved = sessionStorage.getItem(MYSTERY_TAB_KEY);
      return MYSTERY_TABS.includes(saved as (typeof MYSTERY_TABS)[number])
         ? (saved as 'Rewards' | 'Feed' | 'History')
         : 'Rewards';
   });

   useEffect(() => {
      sessionStorage.setItem(MYSTERY_TAB_KEY, activeTab);
   }, [activeTab]);

   const handleExtract = async () => {
    if (isExtracting) {
      return;
    }

    setIsExtracting(true);
    setExtractionStage('CONNECTING_GATEWAY');
      try {
         await sleep(450);
         setExtractionStage('SCANNING_SHARD');
         await sleep(600);
         setExtractionStage('DECRYPTING_CARGO');

         const result = await appExtractMystery(DUMMY_USER.nfmAddress);
         await refresh();

         setExtractionStage('SUCCESS_FLASH');
         await sleep(350);
         const reward = typeof result?.reward === 'string' ? result.reward : 'Unknown reward';
         notifySuccess(`Extraction success: ${reward}`);
      } catch (e) {
         notifyError(e instanceof Error ? e.message : 'Extraction failed');
      } finally {
         setExtractionStage(null);
         setIsExtracting(false);
      }
  };

  const nextBoxThreshold = 10;
  const progressPercent = (DUMMY_USER.feedbackCount / nextBoxThreshold) * 100;
   const stageIndex = extractionStage ? EXTRACTION_STAGES.indexOf(extractionStage) : -1;
   const stageProgressPct = stageIndex >= 0 ? ((stageIndex + 1) / EXTRACTION_STAGES.length) * 100 : 0;
   const pendingBoxes = Math.floor((DUMMY_USER.feedbackCount || 0) / nextBoxThreshold);
   const validationsRemaining = Math.max(0, nextBoxThreshold - ((DUMMY_USER.feedbackCount || 0) % nextBoxThreshold));

  return (
    <div className="animate-in pb-12">
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-6">
          <button className="nfm-btn nfm-btn--ghost nfm-btn--sm h-10 w-10 p-0 flex items-center justify-center rounded-xl" onClick={() => navigate(-1)}>
            <ChevronRight size={18} style={{ transform: 'rotate(180deg)' }} />
          </button>
          <div>
            <h1 className="text-cyan text-2xl font-bold tracking-tight flex items-center gap-3">
              <Gift className="text-pink drop-shadow-pink--soft" /> Mystery Extraction
            </h1>
            <p className="text-10px text-muted font-bold uppercase tracking-[0.2em] mt-1 opacity-60">Protocol_Beacon_v1.4_Gateway</p>
          </div>
        </div>
        <div className="nfm-badge nfm-badge--cyan px-4 py-2 flex items-center gap-3 bg-surface-lowest border-l-2 border-cyan-40">
           <Globe size={14} className="text-cyan" /> 
           <span className="text-9px uppercase font-bold tracking-[0.2em] text-primary">Mesh_Shard: <span className="text-cyan">AX-7_Optimal</span></span>
        </div>
      </div>

      <div className="flex flex-col gap-6 items-stretch">
        
        {/* Top Control Deck: Unified Extraction Console */}
        <div className="nfm-glass-card p-0 overflow-hidden flex flex-col relative bg-surface-lowest" style={{ borderLeft: '3px solid var(--sovereign-purple)', background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.05), transparent)' }}>
           
           {/* Integrated System Credits Header */}
           <div className="p-4 px-8 bg-black-40 border-b border-white-05 flex justify-between items-center z-20 relative">
              <div className="flex items-center gap-8">
                 <div className="flex flex-col">
                    <span className="text-9px text-muted uppercase tracking-widest font-bold mb-1 opacity-60">System Credits</span>
                    <div className="text-xl font-display font-medium text-primary tracking-tight">
                       {DUMMY_USER.balance.toLocaleString()} <span className="text-xs text-cyan opacity-40">NVC</span>
                    </div>
                 </div>
                 <div className="h-8 w-px bg-white-10"></div>
                 <div className="flex flex-col">
                    <span className="text-9px text-muted uppercase tracking-widest font-bold mb-1 opacity-60">Pending Boxes</span>
                    <div className="text-xl font-display font-medium text-cyan tracking-tight">
                       {pendingBoxes.toString().padStart(2, '0')} <span className="text-xs opacity-40">BOXES</span>
                    </div>
                 </div>
              </div>
              <div className="flex items-center gap-4">
                 <div className="nfm-badge nfm-badge--cyan text-9px font-mono px-3 py-1">AX-7_Gateway</div>
                 <div className="nfm-status-dot nfm-status-dot--online"></div>
              </div>
           </div>

           <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(139,92,246,0.1)_0%,transparent_70%)]"></div>
           <div className="absolute inset-0 nfm-scan-grid opacity-10"></div>
           <div className={`nfm-extraction-beam ${isExtracting ? 'nfm-extraction-beam--active' : ''}`}></div>

           {/* Console Body: 3-Col Horizontal Layout */}
           <div className="grid grid-cols-1 md:grid-cols-3 gap-8 p-8 py-10 relative z-10 items-center min-h-[320px]">
              
              {/* Left Column: Progress Info */}
              <div className="flex flex-col gap-4 h-full">
                 <h3 className="text-9px text-muted uppercase tracking-[0.25em] font-bold flex items-center gap-2">
                    <Activity size={12} className="text-purple" /> Neural Protocol
                 </h3>
                 <div className="flex-1 flex flex-col justify-center items-center gap-6 p-6 rounded-2xl bg-black-40 border border-white-05 relative overflow-hidden group">
                    <div className="absolute inset-0 nfm-scan-grid opacity-20"></div>
                    <div className="flex items-center gap-6 w-full relative z-10">
                       <div className="relative w-24 h-24 shrink-0">
                          <svg className="w-full h-full transform -rotate-90">
                             <circle cx="48" cy="48" r="40" stroke="rgba(255,255,255,0.03)" strokeWidth="6" fill="transparent" />
                             <circle 
                               cx="48" cy="48" r="40" stroke="var(--sovereign-purple)" strokeWidth="6" fill="transparent" 
                               strokeDasharray={251.2}
                               strokeDashoffset={251.2 - (251.2 * progressPercent) / 100}
                               strokeLinecap="round"
                               className="transition-all duration-1000 ease-out"
                               style={{ filter: 'drop-shadow(0 0 10px var(--sovereign-purple))' }}
                             />
                          </svg>
                          <div className="absolute inset-0 flex flex-col items-center justify-center">
                             <div className="text-xl font-bold text-primary tracking-tight">{Math.floor(progressPercent)}%</div>
                             <div className="text-[6px] text-muted uppercase font-bold tracking-[0.2em] mt-1 opacity-60">SYNCED</div>
                          </div>
                       </div>
                       <div className="flex flex-col gap-3 flex-1">
                          <p className="text-9px text-muted leading-relaxed uppercase tracking-wider opacity-60 line-clamp-2">
                             Estimated arrival: <br/>
                             <span className="text-purple font-bold">{validationsRemaining} MORE VALIDATIONS</span>
                          </p>
                          <div className="h-1 w-full bg-white-05 rounded-full overflow-hidden mt-2">
                             <div className="h-full bg-purple shadow-glow-purple" style={{ width: `${progressPercent}%` }}></div>
                          </div>
                       </div>
                    </div>
                 </div>
              </div>

              {/* Center Column: Extraction Altar Graphic */}
              <div className="flex flex-col items-center justify-center h-full relative">
                 <h2 className="text-2xl font-display text-primary mb-1 absolute -top-4 text-center">Extraction Altar</h2>
                 <div className="text-10px text-muted uppercase tracking-[0.4em] font-bold opacity-40 absolute top-4 text-center w-full">NODE_GATEWAY_V1.4</div>
                 
                 <div className="relative nfm-extraction-altar mt-12 scale-[0.85]">
                    <div className={`mx-auto flex items-center justify-center relative transition-all duration-1000 ${isExtracting ? 'scale-110' : 'animate-float'}`}>
                       <div className={`absolute w-36 h-36 bg-purple-10 rounded-full blur-xl animate-pulse transition-opacity duration-700 ${isExtracting ? 'opacity-100' : 'opacity-40'}`}></div>
                       <div className={`relative p-8 rounded-[2rem] bg-surface-lowest border border-white-10 shadow-2xl transition-all duration-700 ${isExtracting ? 'rotate-[-20deg] scale-90' : ''}`}>
                          <Gift size={70} className={`text-pink drop-shadow-pink ${isExtracting ? 'animate-bounce' : ''}`} />
                       </div>
                    </div>
                    <div className="mt-8 nfm-altar-pedestal translate-y-4"></div>
                 </div>
              </div>

              {/* Right Column: Pre-Flight Checks & Execute */}
              <div className="flex flex-col gap-4 h-full justify-between">
                 <div>
                    <h3 className="text-9px text-muted uppercase tracking-[0.25em] font-bold flex items-center gap-2 mb-4">
                       <Info size={12} className="text-cyan" /> Pre-Flight Data
                    </h3>

                    <div className="p-3 rounded-xl bg-black-40 border border-white-05 mb-4">
                       <div className="flex items-center justify-between mb-2">
                          <span className="text-[8px] text-muted uppercase tracking-widest">Extraction Stage</span>
                          <span className="text-9px font-mono text-cyan">
                             {extractionStage ?? 'IDLE'}
                          </span>
                       </div>
                       <div className="nfm-progress" style={{ height: '4px', marginBottom: '8px' }}>
                          <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: `${stageProgressPct}%` }}></div>
                       </div>
                       <div className="grid grid-cols-2 gap-1">
                          {EXTRACTION_STAGES.map((stageName, idx) => (
                             <div key={stageName} className="text-[8px] font-mono" style={{ color: idx <= stageIndex ? 'var(--neon-cyan)' : 'var(--text-muted)' }}>
                                {idx + 1}. {stageName}
                             </div>
                          ))}
                       </div>
                    </div>
                    
                    <div className="grid grid-cols-2 gap-3 mb-4">
                       <div className="p-3 rounded-xl bg-black-40 border border-white-05 flex flex-col justify-center">
                          <span className="text-[8px] text-muted uppercase tracking-widest mb-1">Fee Rate</span>
                          <span className="text-sm font-bold text-primary">5.0 <span className="text-cyan text-10px">NVC</span></span>
                       </div>
                       
                       <div className="p-3 rounded-xl bg-black-40 border border-white-05 flex flex-col justify-center">
                          <span className="text-[8px] text-muted uppercase tracking-widest mb-1">Stability</span>
                          <span className="text-sm font-bold text-success uppercase">Optimal</span>
                       </div>
                    </div>
                    
                    <div className="grid grid-cols-2 gap-2 opacity-80">
                       {[
                         { rarity: 'Legendary', chance: '1.2%', color: 'gold' },
                         { rarity: 'Epic', chance: '5.8%', color: 'pink' },
                         { rarity: 'Rare', chance: '18.0%', color: 'cyan' },
                         { rarity: 'Common', chance: '75.0%', color: 'muted' },
                       ].map((tier, idx) => (
                         <div key={idx} className="flex justify-between items-center p-2 rounded-lg bg-surface-lowest-40 border border-white-05">
                            <span className={`nfm-badge nfm-badge--${tier.color === 'gold' ? 'muted' : tier.color} font-mono text-[8px] px-1 py-0 rounded leading-none`}>
                               {tier.rarity}
                            </span>
                            <span className="font-mono text-[8px] text-white-60">{tier.chance}</span>
                         </div>
                       ))}
                    </div>
                 </div>

                 <button 
                   className={`nfm-btn nfm-btn--lg h-14 mt-auto rounded-xl ${isExtracting ? 'nfm-btn--ghost grayscale opacity-40 cursor-wait' : 'nfm-btn--primary shadow-purple'}`}
                   onClick={handleExtract}
                   disabled={isExtracting}
                 >
                   {isExtracting ? (
                     <div className="flex items-center justify-center gap-3">
                        <Activity size={18} className="animate-spin text-cyan" /> 
                                    <span className="text-10px font-bold tracking-[0.2em] uppercase">{extractionStage ?? 'Processing...'}</span>
                     </div>
                   ) : (
                     <div className="flex items-center justify-center gap-3">
                        <Sparkles size={18} className="text-black" />
                        <span className="text-[11px] font-bold tracking-[0.3em] uppercase text-black">INITIALISE MESH</span>
                     </div>
                   )}
                 </button>
              </div>

           </div>
        </div>

        {/* Bottom Data Deck: Discovery Terminal */}
        <div className="nfm-glass-card p-0 bg-surface-lowest-40 flex flex-col min-h-[460px] overflow-hidden" style={{ borderLeft: '3px solid var(--neon-cyan)' }}>
           <div className="p-1 px-4 bg-black-40 border-b border-white-05 flex items-center justify-between">
              <div className="nfm-terminal-tabs">
                 {[
                   { id: 'Rewards', icon: <Trophy size={11} />, label: 'Mesh_Shards' },
                   { id: 'Feed', icon: <Zap size={11} />, label: 'Protocol_Feed' },
                   { id: 'History', icon: <Terminal size={11} />, label: 'Terminal_logs' }
                 ].map(tab => (
                   <button 
                     key={tab.id}
                     className={`nfm-terminal-btn flex items-center justify-center gap-2 ${activeTab === tab.id ? 'nfm-terminal-btn--active' : ''}`}
                     onClick={() => setActiveTab(tab.id as any)}
                   >
                     {tab.icon} {tab.label}
                   </button>
                 ))}
              </div>
              <span className="text-[7px] font-mono text-muted opacity-40">CONSOLE_V1.4.2</span>
           </div>

           <div className="flex-1 overflow-y-auto p-6 scrollbar-hide">
              {activeTab === 'Rewards' && (
                 <table className="nfm-table">
                    <thead>
                       <tr className="text-9px uppercase tracking-widest text-muted border-b border-white-05">
                          <th className="pb-3 text-left">Shard Name</th>
                          <th className="pb-3 text-left">Rarity</th>
                          <th className="pb-3 text-left">Integrity</th>
                       </tr>
                    </thead>
                    <tbody className="divide-y divide-white-05">
                       {DUMMY_REWARD_CATALOG.map(reward => (
                         <tr key={reward.id} className="group hover:bg-white-03 transition-all cursor-default">
                            <td className="py-4">
                               <div className="flex items-center gap-3">
                                  <div className={`nfm-status-dot nfm-status-dot--${reward.rarity === 'COMMON' ? 'online' : 'syncing'}`}></div>
                                  <div className="flex flex-col">
                                     <span className="text-[11px] font-bold text-primary group-hover:text-cyan transition-colors">{reward.name}</span>
                                     <span className="text-9px text-muted opacity-80 line-clamp-1">{reward.description}</span>
                                  </div>
                               </div>
                            </td>
                            <td className="py-4">
                               <span className={`nfm-badge nfm-badge--${reward.rarity === 'COMMON' ? 'muted' : reward.rarity === 'LEGENDARY' ? 'gold' : reward.rarity === 'EPIC' ? 'pink' : 'cyan'} text-[8px] px-2 py-0.5 font-mono`}>
                                  {reward.rarity}
                               </span>
                            </td>
                            <td className="py-4 w-1/3">
                               <div className="flex items-center gap-3 max-w-[200px]">
                                  <div className="flex-1 h-1 bg-white-05 rounded-full overflow-hidden">
                                     <div className="h-full bg-success shadow-glow-success--soft opacity-60" style={{ width: '92%' }}></div>
                                  </div>
                                  <span className="text-9px font-mono text-success">92%</span>
                               </div>
                            </td>
                         </tr>
                       ))}
                    </tbody>
                 </table>
              )}

              {activeTab === 'Feed' && (
                 <div className="space-y-1">
                    <div className="divide-y divide-white-03">
                       {DUMMY_MYSTERY_NEWS.map(news => (
                         <div key={news.id} className="flex gap-4 p-4 hover:bg-white-02 transition-all group">
                            <div className="flex flex-col items-center gap-1">
                               <div className={`w-1 h-4 rounded-full mt-1 ${news.type === 'RARE_FIND' ? 'bg-gold' : news.type === 'BURN' ? 'bg-pink' : 'bg-cyan'} shadow-glow`}></div>
                            </div>
                            <div className="flex flex-col gap-1 flex-1">
                               <div className="flex items-center justify-between mb-1">
                                  <span className="text-9px font-bold uppercase tracking-[0.25em] text-muted opacity-40 group-hover:opacity-80 transition-opacity">_{news.type}_UNIT</span>
                                  <span className="text-[8px] text-muted opacity-30 font-mono italic">
                                     {Math.floor((Date.now() - news.timestamp) / 60000)}M_AGO
                                  </span>
                               </div>
                               <p className="text-[11px] text-white-60 leading-relaxed font-medium group-hover:text-primary transition-colors">{news.content}</p>
                            </div>
                         </div>
                       ))}
                    </div>
                 </div>
              )}

              {activeTab === 'History' && (
                 <table className="nfm-table">
                    <thead>
                       <tr className="text-9px uppercase tracking-widest text-muted border-b border-white-05">
                          <th className="pb-3 text-left">Record ID</th>
                          <th className="pb-3 text-left">Reward Info</th>
                          <th className="pb-3 text-right">Timestamp</th>
                       </tr>
                    </thead>
                    <tbody className="divide-y divide-white-05">
                       {DUMMY_BOX_HISTORY.map(log => (
                         <tr key={log.id} className="group hover:bg-white-03 transition-all">
                            <td className="py-4 font-mono text-10px text-cyan opacity-60">#{log.id.substring(0, 8)}</td>
                            <td className="py-4">
                               <div className="flex items-center gap-3">
                                  <div className={`w-1.5 h-1.5 rounded-full bg-${log.rarity.toLowerCase()} shadow-glow`}></div>
                                  <span className="text-[11px] font-bold text-primary group-hover:text-white transition-colors">{log.rewardInfo}</span>
                               </div>
                            </td>
                            <td className="py-4 text-right font-mono text-9px text-white-60">
                               {new Date(log.timestamp).toLocaleDateString()}
                            </td>
                         </tr>
                       ))}
                    </tbody>
                 </table>
              )}
           </div>
        </div>
      </div>
    </div>
  );
};

export default MysteryBox;
