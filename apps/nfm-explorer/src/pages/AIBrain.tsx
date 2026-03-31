import { Brain, Bot, Terminal, Send, RefreshCw, Layers } from 'lucide-react';
import { useAppData } from '../context/AppDataContext';

const AIBrain = () => {
  const { data } = useAppData();
  const DUMMY_AI_TASKS = data.ai_tasks;

  return (
    <div className="animate-in">
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <h1 className="text-cyan flex items-center gap-2"><Brain /> AI Brain & Autopilot</h1>
        <div className="nfm-badge nfm-badge--cyan">
          <div className="nfm-badge__dot"></div> Model: NFM-Orchestrator-70B
        </div>
      </div>

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        
        {/* Chat Interface */}
        <div className="nfm-glass-card" style={{ flex: '2 1 500px', display: 'flex', flexDirection: 'column', height: '600px' }}>
          <div className="flex items-center gap-2 mb-4 pb-4" style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
            <Bot className="text-purple" />
            <h2 className="text-lg">Command Interface</h2>
          </div>

          <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 'var(--space-4)', paddingRight: 'var(--space-2)' }}>
            {/* User Message */}
            <div className="flex gap-3" style={{ alignSelf: 'flex-end', maxWidth: '85%' }}>
              <div style={{ background: 'var(--surface-high)', padding: '12px 16px', borderRadius: '16px 16px 0 16px', border: '1px solid rgba(255,255,255,0.05)' }}>
                Please analyze the recent network traffic for any anomalies and summarize the findings.
              </div>
            </div>

            {/* AI Message */}
            <div className="flex gap-3" style={{ alignSelf: 'flex-start', maxWidth: '85%' }}>
              <div className="nfm-avatar" style={{ width: 32, height: 32, fontSize: '10px' }}>AI</div>
              <div style={{ background: 'rgba(0, 245, 255, 0.05)', border: '1px solid rgba(0, 245, 255, 0.15)', padding: '12px 16px', borderRadius: '16px 16px 16px 0' }}>
                <p className="mb-2">I have analyzed the network traffic for the past 24 hours. Here are the findings:</p>
                <ul style={{ paddingLeft: '20px', color: 'var(--text-secondary)' }}>
                  <li>Traffic volume is within normal parameters.</li>
                  <li>Detected 3 minor anomalies originating from unknown IPs. These were automatically blocked by the firewall.</li>
                  <li>Node synchronization efficiency has improved by 4% since the last epoch.</li>
                </ul>
              </div>
            </div>
            
            {/* AI Action Execution */}
            <div className="flex gap-3" style={{ alignSelf: 'flex-start', maxWidth: '85%', marginTop: '-8px' }}>
              <div style={{ width: 32 }} /> {/* Padding to align with avatar */}
              <div className="nfm-badge nfm-badge--purple" style={{ textTransform: 'none', background: 'transparent', border: '1px solid rgba(138, 43, 226, 0.3)' }}>
                <Terminal size={12} className="mr-1" /> Executed `net_analyzer.py --time 24h`
              </div>
            </div>
          </div>

          <div className="mt-4 pt-4" style={{ borderTop: '1px solid rgba(255,255,255,0.05)' }}>
            <div className="nfm-search">
              <input type="text" className="nfm-search__input" placeholder="Give the AI Brain a task..." style={{ paddingLeft: '16px', paddingRight: '48px' }} />
              <button className="nfm-btn nfm-btn--primary" style={{ position: 'absolute', right: '4px', top: '4px', bottom: '4px', padding: '0 12px' }}>
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
              <RefreshCw size={16} className="text-muted" style={{cursor: 'pointer'}} />
            </h2>
            
            <div className="flex-col gap-3">
              {DUMMY_AI_TASKS.map(task => (
                <div key={task.id} className="p-3" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-sm)', border: '1px solid rgba(255,255,255,0.03)' }}>
                  <div className="flex justify-between items-start mb-2">
                    <span className="font-bold text-sm" style={{maxWidth: '70%'}}>{task.name}</span>
                    <span className={`nfm-badge nfm-badge--${task.status === 'RUNNING' ? 'cyan' : task.status === 'COMPLETED' ? 'success' : task.status === 'FAILED' ? 'error' : 'muted'}`} style={{fontSize: '10px'}}>
                      {task.status}
                    </span>
                  </div>
                  
                  <div className="nfm-progress mb-2" style={{height: '4px'}}>
                     <div className={`nfm-progress__fill nfm-progress__fill--${task.status === 'RUNNING' ? 'cyan' : task.status === 'COMPLETED' ? 'success' : 'muted'}`} style={{ width: `${task.progress}%` }}></div>
                  </div>
                  
                  <div className="flex justify-between text-xs text-muted">
                    <span className="font-mono">{task.model}</span>
                    <span className="text-gold">{task.cost.toFixed(2)} NVC</span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="nfm-glass-card">
            <h3 className="text-sm uppercase text-muted mb-4 font-bold tracking-wider">Active Modules</h3>
            <div className="flex-col gap-2">
              <div className="flex items-center justify-between">
                <span className="text-sm">Web Search & Scrape</span>
                <span className="nfm-status-dot nfm-status-dot--online"></span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Code Execution Env</span>
                <span className="nfm-status-dot nfm-status-dot--online"></span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Blockchain TX Signer</span>
                <span className="nfm-status-dot nfm-status-dot--offline"></span>
              </div>
            </div>
          </div>

        </div>

      </div>
    </div>
  );
};

export default AIBrain;
