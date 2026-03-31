import { Target, Gift, Zap, Shield, ChevronRight, ArrowRight, Trophy, Flame, Network, Cpu, ShieldCheck, Activity } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';

const QuestCenter = () => {
  const navigate = useNavigate();
   const { data } = useAppData();
   const DUMMY_QUESTS = data.quests;

  return (
    <div className="animate-in">
      {/* Header with Rank Badges */}
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <div>
          <h1 className="text-cyan flex items-center gap-2"><Target /> Core Quest Center</h1>
          <p className="text-muted text-sm mt-1">Contribute to the NFM mesh, train local AI models, and secure the network to earn NVC.</p>
        </div>
        <div className="flex gap-4">
          <div className="nfm-badge nfm-badge--purple">
            <div className="nfm-badge__dot"></div> Validator Agent (Lvl 12)
          </div>
          <div className="nfm-badge nfm-badge--cyan">
            <div className="nfm-badge__dot"></div> 7,200 XP
          </div>
        </div>
      </div>

      {/* Rewards & Performance Stats Grid */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(4, 1fr)', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-8)' 
      }}>
        {[
          { label: 'Total Rewards', value: '242.5 NVC', icon: <Trophy size={16}/>, color: 'gold' },
          { label: 'Missions Concluded', value: '42', icon: <Activity size={16}/>, color: 'cyan' },
          { label: 'Active Streak', value: '5 Days', icon: <Flame size={16}/>, color: 'pink' },
          { label: 'Global Standing', value: '#1,402', icon: <Shield size={16}/>, color: 'purple' },
        ].map((stat, idx) => (
          <div key={idx} className="nfm-glass-card" style={{ padding: 'var(--space-5)', marginBottom: 0 }}>
            <div className="flex items-center gap-3 mb-3">
              <div className={`p-2 rounded-lg bg-surface-lowest text-${stat.color}`}>
                {stat.icon}
              </div>
              <span className="text-[10px] text-muted uppercase tracking-widest">{stat.label}</span>
            </div>
            <div className="text-2xl font-display text-primary">{stat.value}</div>
          </div>
        ))}
      </div>

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        
        {/* Main Mission Hub */}
        <div className="flex-col gap-6" style={{ flex: '2 1 600px' }}>
          <div className="nfm-glass-card nfm-glass-card--glow-cyan p-8" style={{ border: '1px solid rgba(0, 245, 255, 0.1)' }}>
            <div className="flex justify-between items-center mb-6">
               <h2 className="text-xl text-primary flex items-center gap-3">
                  <Zap className="text-gold" fill="currentColor" /> Daily Missions
               </h2>
               <div className="text-[10px] text-muted font-mono bg-white-05 px-2 py-1 rounded">
                  Refreshes in: <span className="text-cyan">04:22:15</span>
               </div>
            </div>

            <div className="flex-col gap-4">
              {DUMMY_QUESTS.map(quest => (
                <div key={quest.id} className="nfm-glass-card--interactive p-4" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-lg)', borderLeft: `4px solid ${quest.status === 'CLAIMABLE' ? 'var(--neon-cyan)' : quest.status === 'COMPLETED' ? 'var(--success)' : 'rgba(255,255,255,0.05)'}` }}>
                  <div className="flex items-center justify-between wrap gap-4">
                    <div className="flex gap-4" style={{ flex: 1, minWidth: '300px' }}>
                       <div className={`p-3 rounded-xl bg-black-20 text-${quest.id.includes('sh') ? 'purple' : quest.id.includes('ai') ? 'pink' : 'cyan'}`}>
                          {quest.id.includes('sh') ? <ShieldCheck size={20} /> : quest.id.includes('ai') ? <Cpu size={20} /> : <Network size={20} />}
                       </div>
                       <div style={{ flex: 1 }}>
                          <div className="flex items-center gap-2 mb-1">
                             <span className="font-bold text-primary">{quest.title}</span>
                             {quest.status === 'CLAIMABLE' && <span className="nfm-badge nfm-badge--cyan text-[9px] uppercase font-bold py-0.5 px-1.5 h-auto">Ready</span>}
                             {quest.status === 'COMPLETED' && <span className="nfm-badge nfm-badge--success text-[9px] uppercase font-bold py-0.5 px-1.5 h-auto">Claimed</span>}
                          </div>
                          <p className="text-xs text-muted mb-4">{quest.description}</p>
                          <div className="flex items-center gap-4">
                             <div className="nfm-progress" style={{ flex: 1, maxWidth: '240px', height: '6px', background: 'rgba(255,255,255,0.05)' }}>
                                <div className={`nfm-progress__fill nfm-progress__fill--${quest.status === 'COMPLETED' ? 'success' : 'cyan'}`} style={{ width: `${(quest.progress / quest.total) * 100}%` }}></div>
                             </div>
                             <span className="text-[10px] font-mono text-muted">{quest.progress}/{quest.total}</span>
                          </div>
                       </div>
                    </div>
                    
                    <div className="flex-col items-end gap-2" style={{ minWidth: '140px' }}>
                       <div className="font-display text-lg text-gold">+{quest.rewardNVC.toFixed(1)} <span className="text-xs">NVC</span></div>
                       <button 
                         className={`nfm-btn nfm-btn--sm w-full font-bold uppercase tracking-widest text-[10px] ${quest.status === 'CLAIMABLE' ? 'nfm-btn--primary shadow-cyan' : 'nfm-btn--ghost opacity-50'}`}
                         disabled={quest.status !== 'CLAIMABLE'}
                       >
                         {quest.status === 'CLAIMABLE' ? 'Claim Reward' : quest.status === 'COMPLETED' ? 'Completed' : 'Processing'}
                       </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            <button className="nfm-btn-more mt-8">
              <ArrowRight size={14} /> Full Mission Archive
            </button>
          </div>
        </div>

        {/* Player Tier & Progression Sidebar */}
        <div className="flex-col gap-6" style={{ flex: '1 1 350px' }}>
          
          {/* Rank Card */}
          <div className="nfm-glass-card p-6 text-center">
            <h3 className="text-xs text-muted uppercase tracking-widest mb-6 flex items-center justify-center gap-2">
              <Shield size={14} className="text-purple" /> Agent Standing
            </h3>
            
            <div className="relative inline-block mb-4">
               <div className="w-24 h-24 rounded-full border-2 border-purple p-1 relative z-10 bg-surface-lowest">
                  <div className="w-full h-full rounded-full bg-gradient-to-tr from-purple-90 to-cyan-20 flex items-center justify-center">
                     <ShieldCheck size={40} className="text-purple" />
                  </div>
               </div>
               <div className="absolute inset-0 bg-glow-purple--soft rounded-full -z-10 animate-pulse"></div>
            </div>

            <div className="text-2xl font-display text-primary">Validator Agent</div>
            <div className="text-xs text-purple font-bold mb-6">Tier 2 Elite</div>

            <div className="bg-surface-lowest p-4 rounded-xl mb-6 text-left border border-white-05">
               <div className="flex justify-between text-xs mb-2">
                  <span className="text-muted">Stability Index</span>
                  <span className="text-cyan">94%</span>
               </div>
               <div className="nfm-progress" style={{ height: '4px' }}>
                  <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: '94%' }}></div>
               </div>
            </div>

            <div className="text-left">
               <div className="flex justify-between text-[10px] text-muted mb-2 uppercase tracking-tighter">
                  <span>Next Rank: Mesh Architect</span>
                  <span>72%</span>
               </div>
               <div className="nfm-progress" style={{ height: '8px' }}>
                  <div className="nfm-progress__fill nfm-progress__fill--purple" style={{ width: '72%' }}></div>
               </div>
               <div className="flex justify-between text-[10px] font-mono text-muted mt-2">
                  <span>7,200 XP</span>
                  <span>10,000 XP</span>
               </div>
            </div>
          </div>

          {/* Active Multiplier Card */}
          <div className="nfm-glass-card--glow-pink p-5 border border-white-05">
             <div className="flex items-center gap-3 mb-4">
                <div className="p-2 rounded-lg bg-pink-10 text-pink">
                   <Flame size={20} />
                </div>
                <div>
                   <div className="text-xs font-bold text-primary">5-Day Hot Streak</div>
                   <div className="text-[10px] text-pink">1.2x Multiplier Active</div>
                </div>
             </div>
             <p className="text-[10px] text-muted">Maintain your daily activity to reach Day 7 and unlock the <span className="text-pink">1.5x Multiplier</span>.</p>
          </div>

          <div className="nfm-glass-card--interactive p-6 border border-white-05" onClick={() => navigate('/mystery')}>
             <div className="flex justify-between items-start mb-2">
                <h3 className="text-md text-gold font-bold flex items-center gap-2">
                   <Gift size={18} /> extraction pool
                </h3>
                <ChevronRight size={16} className="text-muted" />
             </div>
             <p className="text-xs text-secondary mb-4">
                Redeem mystery boxes with your earned NVC to unlock rare AI Brain modules.
             </p>
             <div className="flex gap-1">
                {[1, 2, 3].map(i => (
                   <div key={i} className="h-1 flex-1 bg-gold opacity-30 rounded-full"></div>
                ))}
                <div className="h-1 flex-1 bg-surface-lowest rounded-full"></div>
             </div>
          </div>

        </div>

      </div>
    </div>
  );
};

export default QuestCenter;
