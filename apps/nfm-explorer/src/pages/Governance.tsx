import { AlignLeft, CheckCircle2, XCircle, Clock, ArrowRight, Vote, Plus } from 'lucide-react';
import { useAppData } from '../context/AppDataContext';
import { appCreateProposal, appVoteProposal } from '../api/client';

const Governance = () => {
  const { data, refresh } = useAppData();
  const DUMMY_PROPOSALS = data.proposals;
  const DUMMY_USER = data.user_profile;

  const handleCreateProposal = async () => {
    const title = window.prompt('Proposal title');
    if (!title) return;
    const description = window.prompt('Proposal description') || 'Created from NFM Explorer';

    try {
      await appCreateProposal(title, description, DUMMY_USER.nfmAddress);
      await refresh();
      window.alert('Proposal created');
    } catch (e) {
      window.alert(e instanceof Error ? e.message : 'Create proposal failed');
    }
  };

  const handleVote = async (proposalId: string, approve: boolean) => {
    try {
      await appVoteProposal(proposalId, approve, DUMMY_USER.nfmAddress);
      await refresh();
      window.alert('Vote submitted');
    } catch (e) {
      window.alert(e instanceof Error ? e.message : 'Vote failed');
    }
  };

  return (
    <div className="animate-in">
      <div className="flex items-center justify-between mb-8">
        <div className="flex-col">
          <h1 className="text-purple flex items-center gap-2 mb-1"><AlignLeft /> Protocol Governance</h1>
          <p className="text-muted text-xs uppercase tracking-widest font-semibold ml-8 opacity-70">DAO Decision Matrix</p>
        </div>
        <div className="flex items-center gap-6">
          <div className="text-right hide-mobile">
            <div className="text-[10px] text-muted uppercase tracking-wider mb-1">Your Voting Power</div>
            <div className="font-mono text-cyan font-bold">{Math.floor(DUMMY_USER.balance).toLocaleString()} VP</div>
          </div>
          <button className="nfm-btn nfm-btn--primary" onClick={handleCreateProposal}>
            <Plus size={16} /> Create Proposal
          </button>
        </div>
      </div>

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        
        <div className="flex-col gap-6" style={{ flex: '2 1 600px' }}>
          
          <div className="nfm-glass-card nfm-glass-card--glow-purple">
            <div className="flex items-center gap-2 mb-8">
              <Vote className="text-purple" size={20} />
              <h2 className="text-lg">Active & Recent Proposals</h2>
            </div>

            <div className="flex-col gap-5">
              {DUMMY_PROPOSALS.map(prop => {
                const totalVotes = prop.forVotes + prop.againstVotes;
                const forPercentage = totalVotes > 0 ? (prop.forVotes / totalVotes) * 100 : 0;
                
                return (
                  <div key={prop.id} className="nfm-proposal-card">
                    <div className="flex justify-between items-start mb-3">
                       <div>
                         <h3 className="text-base font-bold text-primary mb-1">{prop.title}</h3>
                         <div className="flex items-center gap-3 text-[10px] text-muted uppercase tracking-wider">
                           <span>Author: <span className="text-secondary">{prop.creator}</span></span>
                           <span className="opacity-40">|</span>
                           <span>ID: <span className="font-mono text-secondary">{prop.id}</span></span>
                         </div>
                       </div>
                       <span className={`nfm-badge nfm-badge--${prop.status === 'ACTIVE' ? 'cyan' : prop.status === 'PASSED' ? 'success' : 'error'}`}>
                         {prop.status}
                       </span>
                    </div>

                    <div className="flex-col gap-4">
                       <div className="flex justify-between text-xs font-mono mb-1">
                         <div className="flex items-center gap-2 text-success">
                           <CheckCircle2 size={12}/> FOR ({(prop.forVotes / 1000).toFixed(0)}k)
                         </div>
                         <div className="flex items-center gap-2 text-pink">
                           AGAINST ({(prop.againstVotes / 1000).toFixed(0)}k) <XCircle size={12}/>
                         </div>
                       </div>

                       <div className="nfm-vote-bar">
                         <div className="nfm-vote-bar__fill--for" style={{ width: `${forPercentage}%` }}></div>
                         <div className="nfm-vote-bar__fill--against" style={{ width: `${100 - forPercentage}%` }}></div>
                       </div>

                       <div className="flex justify-between items-center mt-2">
                         {prop.status === 'ACTIVE' ? (
                           <div className="flex items-center gap-1.5 text-cyan text-[10px] uppercase font-bold tracking-widest">
                             <Clock size={12} className="animate-pulse" /> {Math.ceil((prop.endTime - Date.now()) / 86400000)} Days Remaining
                           </div>
                         ) : (
                           <div className="text-[10px] text-muted uppercase tracking-widest font-bold">Voting Cycle Concluded</div>
                         )}
                         
                         {prop.status === 'ACTIVE' ? (
                           <div className="flex gap-2">
                             <button className="nfm-btn nfm-btn--ghost nfm-btn--sm" style={{borderColor: 'var(--success)', color: 'var(--success)'}} onClick={() => handleVote(prop.id, true)}>Vote For</button>
                             <button className="nfm-btn nfm-btn--ghost nfm-btn--sm" style={{borderColor: 'var(--hyper-pink)', color: 'var(--hyper-pink)'}} onClick={() => handleVote(prop.id, false)}>Vote Against</button>
                           </div>
                         ) : (
                           <button className="nfm-btn nfm-btn--ghost nfm-btn--sm border-white/10 text-muted">View Details</button>
                         )}
                       </div>
                    </div>
                  </div>
                );
              })}
            </div>
            
            <button className="nfm-btn-more">
              <ArrowRight size={14} /> Full Proposal Archive
            </button>
          </div>
        </div>

        <div className="flex-col gap-6" style={{ flex: '1 1 320px' }}>
          
          <div className="nfm-glass-card">
             <h3 className="text-lg text-primary mb-6">Governance Rules</h3>
             <ul className="nfm-rules-list">
               <li>1 NVC locked in staking equals 1 Voting Power (VP).</li>
               <li>Proposals require a minimum quorum of 10M VP to be valid.</li>
               <li>Active proposals run for exactly 7 epochs (approx. 3 days).</li>
               <li>Malicious proposals will result in the slashing of the creator's stake.</li>
             </ul>
             <button className="nfm-btn nfm-btn--ghost w-full mt-4 text-xs">Download Charter</button>
          </div>

          <div className="nfm-treasury-card">
             <div className="text-[10px] text-cyan font-bold uppercase tracking-[0.2em] mb-4">DAO Treasury Pool</div>
             <div className="font-display text-5xl font-bold text-primary mb-2">12.5M</div>
             <div className="text-xs font-mono text-muted uppercase tracking-widest mb-8">NVC Contained</div>
             <button className="nfm-btn nfm-btn--primary nfm-btn--sm w-full">View Allocation</button>
             <div className="mt-4 text-[10px] text-muted italic">Managed by Neural Vault Protocol Smart Contract</div>
          </div>

        </div>

      </div>
    </div>
  );
};

export default Governance;
