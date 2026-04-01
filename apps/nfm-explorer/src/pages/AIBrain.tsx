import { Brain, Bot, Terminal, Send, RefreshCw, Layers } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useAppData } from '../context/AppDataContext';
import {
  appBrainCurriculumActive,
  appBrainCurriculumPropose,
  appBrainCurriculumVote,
  appBrainReputationLeaderboard,
} from '../api/client';

const AIBrain = () => {
  const { data, refresh, notifySuccess, notifyError, requestPrompt } = useAppData();
  const DUMMY_AI_TASKS = data.ai_tasks;
  const [taskInput, setTaskInput] = useState('');
  const [activeWindows, setActiveWindows] = useState<Array<{ id: number; epoch: number; model_version: string; participants: string[] }>>([]);
  const [leaderboard, setLeaderboard] = useState<Array<{ rank: number; address: string; reputation_score: number }>>([]);
  const [conversation, setConversation] = useState<Array<{ role: 'user' | 'ai'; text: string }>>([
    {
      role: 'user',
      text: 'Please analyze the recent network traffic for any anomalies and summarize the findings.',
    },
    {
      role: 'ai',
      text: 'Traffic volume is within normal parameters. 3 minor anomaly events were blocked, and sync efficiency improved this epoch.',
    },
  ]);

  const queueSummary = useMemo(() => {
    const running = DUMMY_AI_TASKS.filter((task) => task.status === 'RUNNING').length;
    const queued = DUMMY_AI_TASKS.filter((task) => task.status === 'QUEUED').length;
    return { running, queued };
  }, [DUMMY_AI_TASKS]);

  useEffect(() => {
    const loadBrainGovernance = async () => {
      try {
        const active = await appBrainCurriculumActive() as {
          windows?: Array<{ id?: number; epoch?: number; model_version?: string; participants?: string[] }>;
        };
        setActiveWindows((active.windows ?? []).map((w) => ({
          id: Number(w.id ?? 0),
          epoch: Number(w.epoch ?? 0),
          model_version: w.model_version ?? 'nfm-brain-v1',
          participants: Array.isArray(w.participants) ? w.participants : [],
        })));
      } catch {
        setActiveWindows([]);
      }

      try {
        const board = await appBrainReputationLeaderboard() as {
          leaderboard?: Array<{ rank?: number; address?: string; reputation_score?: number }>;
        };
        setLeaderboard((board.leaderboard ?? []).map((x, idx) => ({
          rank: Number(x.rank ?? idx + 1),
          address: x.address ?? 'unknown',
          reputation_score: Number(x.reputation_score ?? 0),
        })));
      } catch {
        setLeaderboard([]);
      }
    };

    void loadBrainGovernance();
  }, [data.user_profile.nfmAddress]);

  const handleSubmitTask = () => {
    const text = taskInput.trim();
    if (!text) {
      notifyError('Task command is empty');
      return;
    }

    setConversation((prev) => [
      ...prev,
      { role: 'user', text },
      { role: 'ai', text: `Task queued: "${text}". Monitor progress in Autopilot Queue.` },
    ]);
    setTaskInput('');
    notifySuccess('Task submitted to AI queue');
  };

  const handleRefreshQueue = async () => {
    await refresh();
    notifySuccess('Autopilot queue refreshed');
  };

  const handleProposeCurriculum = async () => {
    const modelVersion = await requestPrompt({
      title: 'Propose Curriculum Window',
      message: 'Model version to train (e.g. nfm-brain-v2)',
      placeholder: 'nfm-brain-v2',
      confirmText: 'Propose',
    });
    if (!modelVersion || !modelVersion.trim()) {
      return;
    }

    try {
      const res = await appBrainCurriculumPropose({
        address: data.user_profile.nfmAddress,
        model_version: modelVersion.trim(),
        intent: 'start_learning_window',
        requires_quorum: true,
      }) as { window_id?: number; intent_vote_id?: number };

      notifySuccess(`Curriculum proposed (window #${res.window_id ?? '-'}, vote #${res.intent_vote_id ?? '-'})`);
      setConversation((prev) => [
        ...prev,
        { role: 'user', text: `Propose curriculum: ${modelVersion.trim()}` },
        { role: 'ai', text: `Curriculum window proposed. Intent vote #${res.intent_vote_id ?? 'N/A'} is ready.` },
      ]);
      await refresh();
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Failed to propose curriculum');
    }
  };

  const handleVoteCurriculum = async () => {
    const voteRaw = await requestPrompt({
      title: 'Vote Curriculum Intent',
      message: 'Enter intent vote id',
      placeholder: '1',
      confirmText: 'Vote',
    });
    if (!voteRaw) {
      return;
    }

    const voteId = Number(voteRaw);
    if (!Number.isFinite(voteId) || voteId <= 0) {
      notifyError('Invalid vote id');
      return;
    }

    try {
      const res = await appBrainCurriculumVote(voteId, true, data.user_profile.nfmAddress, true) as {
        execution?: { approved?: boolean; error?: string };
      };
      if (res.execution?.error) {
        notifyError(`Vote submitted, execution pending: ${res.execution.error}`);
      } else {
        notifySuccess(`Vote submitted and executed (approved=${res.execution?.approved ? 'true' : 'false'})`);
      }
      await refresh();
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Failed to vote curriculum intent');
    }
  };

  return (
    <div className="animate-in pb-12">
      <div className="flex items-center justify-between wrap gap-4" style={{ marginBottom: 'var(--space-10)' }}>
        <h1 className="text-cyan flex items-center gap-2"><Brain /> AI Brain & Autopilot</h1>
        <div className="nfm-badge nfm-badge--cyan">
          <div className="nfm-badge__dot"></div> Model: NFM-Orchestrator-70B
        </div>
      </div>

      <div className="flex gap-10 wrap" style={{ flexWrap: 'wrap' }}>
        
        {/* Chat Interface */}
        <div className="nfm-glass-card" style={{ flex: '2 1 500px', display: 'flex', flexDirection: 'column', height: '600px' }}>
          <div className="flex items-center gap-2 mb-4 pb-4" style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
            <Bot className="text-purple" />
            <h2 className="text-lg">Command Interface</h2>
          </div>

          <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 'var(--space-4)', paddingRight: 'var(--space-2)' }}>
            {conversation.map((msg, idx) => (
              msg.role === 'user' ? (
                <div key={idx} className="flex gap-3" style={{ alignSelf: 'flex-end', maxWidth: '85%' }}>
                  <div style={{ background: 'var(--surface-high)', padding: '14px 18px', borderRadius: '18px 18px 0 18px', border: '1px solid rgba(255,255,255,0.08)', boxShadow: 'var(--shadow-lg)' }}>
                    {msg.text}
                  </div>
                </div>
              ) : (
                <div key={idx} className="flex gap-3" style={{ alignSelf: 'flex-start', maxWidth: '85%' }}>
                  <div className="nfm-avatar" style={{ width: 36, height: 36, fontSize: '11px', background: 'var(--neon-cyan)', color: 'var(--surface-lowest)' }}>AI</div>
                  <div style={{ background: 'rgba(0, 245, 255, 0.08)', border: '1px solid rgba(0, 245, 255, 0.2)', padding: '14px 18px', borderRadius: '18px 18px 18px 0', color: 'var(--primary)', lineHeight: '1.5' }}>
                    {msg.text}
                  </div>
                </div>
              )
            ))}
            
            {/* AI Action Execution */}
            <div className="flex gap-3" style={{ alignSelf: 'flex-start', maxWidth: '85%', marginTop: '-4px', marginLeft: '48px' }}>
              <div className="nfm-badge nfm-badge--purple" style={{ textTransform: 'none', background: 'transparent', border: '1px solid rgba(138, 43, 226, 0.4)', borderRadius: 'var(--radius-full)', padding: '2px 10px' }}>
                <Terminal size={12} className="mr-1.5" /> Executed `net_analyzer.py --time 24h`
              </div>
            </div>
          </div>

          <div className="mt-4 pt-4" style={{ borderTop: '1px solid rgba(255,255,255,0.05)' }}>
            <div className="nfm-search">
              <input
                type="text"
                className="nfm-search__input"
                placeholder="Give the AI Brain a task..."
                style={{ paddingLeft: '16px', paddingRight: '48px' }}
                value={taskInput}
                onChange={(e) => setTaskInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault();
                    handleSubmitTask();
                  }
                }}
              />
              <button className="nfm-btn nfm-btn--primary" style={{ position: 'absolute', right: '4px', top: '4px', bottom: '4px', padding: '0 12px' }} onClick={handleSubmitTask}>
                <Send size={16} />
              </button>
            </div>
          </div>
        </div>

        {/* Task Queue Layout */}
        <div className="flex-col gap-6" style={{ flex: '1 1 300px' }}>
          
          <div className="nfm-glass-card nfm-glass-card--glow-cyan">
            <h2 className="text-lg text-primary mb-4 flex items-center justify-between">
              <div className="flex items-center gap-2"><Layers size={20}/> Autopilot Queue</div>
              <div className="flex items-center gap-2">
                <span className="text-10px text-muted uppercase tracking-wider">{queueSummary.running} running / {queueSummary.queued} queued</span>
                <RefreshCw size={16} className="text-muted" style={{cursor: 'pointer'}} onClick={() => void handleRefreshQueue()} />
              </div>
            </h2>
            
            <div className="flex flex-col gap-4">
              {DUMMY_AI_TASKS.map(task => (
                <div key={task.id} className="p-4" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-lg)', border: '1px solid var(--white-05)' }}>
                  <div className="flex justify-between items-start mb-3">
                    <span className="font-bold text-sm text-primary" style={{maxWidth: '75%'}}>{task.name}</span>
                    <span className={`nfm-badge nfm-badge--${task.status === 'RUNNING' ? 'cyan' : task.status === 'COMPLETED' ? 'success' : task.status === 'FAILED' ? 'error' : 'secondary'}`} style={{fontSize: '9px'}}>
                      {task.status}
                    </span>
                  </div>
                  
                  <div className="nfm-progress mb-3" style={{height: '6px', background: 'rgba(255,255,255,0.03)'}}>
                     <div className={`nfm-progress__fill nfm-progress__fill--${task.status === 'RUNNING' ? 'cyan' : task.status === 'COMPLETED' ? 'success' : 'secondary'}`} style={{ width: `${task.progress}%` }}></div>
                  </div>
                  
                  <div className="flex justify-between text-10px font-mono text-muted">
                    <span>{task.model}</span>
                    <span className="text-gold">{task.cost.toFixed(2)} NVC</span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="nfm-glass-card p-6">
            <h3 className="text-xs uppercase text-muted mb-6 font-bold tracking-widest flex items-center gap-2">
              <RefreshCw size={14} className="text-cyan" /> Active Modules
            </h3>
            <div className="flex flex-col gap-4">
              {[
                { name: 'Web Search & Scrape', status: 'ONLINE', icon: <Bot size={14} /> },
                { name: 'Code Execution Env', status: 'ONLINE', icon: <Terminal size={14} /> },
                { name: 'Blockchain TX Signer', status: 'OFFLINE', icon: <Layers size={14} /> }
              ].map(mod => (
                <div key={mod.name} className="flex items-center justify-between p-3 bg-black-20 rounded-xl border border-white-05">
                  <div className="flex items-center gap-3">
                    <div className={`p-2 rounded-lg ${mod.status === 'ONLINE' ? 'bg-cyan-10 text-cyan' : 'bg-secondary-10 text-muted'}`}>
                      {mod.icon}
                    </div>
                    <span className="text-sm font-medium text-primary">{mod.name}</span>
                  </div>
                  <div className={`nfm-badge nfm-badge--${mod.status === 'ONLINE' ? 'cyan' : 'secondary'} text-10px px-2 h-auto`}>{mod.status}</div>
                </div>
              ))}
            </div>
          </div>

          <div className="nfm-glass-card nfm-glass-card--glow-purple p-6">
            <h3 className="text-xs uppercase text-muted mb-6 font-bold tracking-widest flex items-center gap-2">
              <Layers size={14} className="text-purple" /> Brain Curriculum Governance
            </h3>
            <div className="flex gap-2 mb-6">
              <button className="nfm-btn nfm-btn--primary nfm-btn--sm flex-1 font-bold uppercase tracking-wider" onClick={() => void handleProposeCurriculum()}>Propose</button>
              <button className="nfm-btn nfm-btn--ghost nfm-btn--sm flex-1 font-bold uppercase tracking-wider" onClick={() => void handleVoteCurriculum()}>Vote</button>
            </div>
            
            <div className="space-y-4">
              <div>
                <div className="text-10px text-muted uppercase tracking-tighter mb-2">Active Windows ({activeWindows.length})</div>
                {activeWindows.length === 0 ? (
                   <div className="text-xs text-muted font-italic bg-white-05 p-2 rounded text-center">No active windows</div>
                ) : (
                  <div className="flex flex-col gap-2">
                    {activeWindows.slice(0, 2).map((w) => (
                      <div key={w.id} className="text-11px p-2 bg-black-20 rounded-lg border border-white-05">
                        <div className="flex justify-between items-center mb-1">
                          <span className="text-cyan font-bold">Window #{w.id}</span>
                          <span className="text-muted font-mono">{w.model_version}</span>
                        </div>
                        <div className="text-secondary opacity-70">Epoch {w.epoch} • {w.participants.length} Active Participants</div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              <div>
                <div className="text-10px text-muted uppercase tracking-tighter mb-2">Reputation Leaderboard</div>
                <div className="flex flex-col gap-1.5">
                  {leaderboard.slice(0, 3).map((r) => (
                    <div key={`${r.rank}-${r.address}`} className="flex justify-between items-center p-2 bg-white-05 rounded-lg text-11px">
                      <div className="flex items-center gap-3">
                        <span className={`font-display w-4 ${r.rank === 1 ? 'text-gold' : r.rank === 2 ? 'text-cyan' : 'text-muted'}`}>#{r.rank}</span>
                        <span className="text-primary font-mono">{r.address.slice(0, 8)}...{r.address.slice(-6)}</span>
                      </div>
                      <span className="text-gold font-bold">{r.reputation_score}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

        </div>

      </div>
    </div>
  );
};

export default AIBrain;
