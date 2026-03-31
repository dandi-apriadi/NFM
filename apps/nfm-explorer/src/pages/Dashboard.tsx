import { Box, Flame, Zap, Database, TrendingUp, ArrowRight } from 'lucide-react';
import NetworkChart from '../components/ui/NetworkChart';
import { useAppData } from '../context/AppDataContext';

const Dashboard = () => {
  const { data } = useAppData();
  const DUMMY_STATUS = data.status;
  const DUMMY_BLOCKS = data.blocks;

  const chartData = [45, 52, 48, 70, 85, 74, 90, 82, 95];

  return (
    <div className="animate-in">
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <div>
          <h1 className="text-cyan">Network Dashboard</h1>
          <p className="text-muted text-sm mt-1">Real-time status of the NFM decentralized mesh network.</p>
        </div>
        <div className="nfm-badge nfm-badge--success">
          <div className="nfm-badge__dot"></div>
          Epoch 442 Active
        </div>
      </div>

      <div className="dashboard-grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(4, 1fr)', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-8)' 
      }}>
        <div className="nfm-glass-card" style={{ marginBottom: 0 }}>
          <div className="nfm-stat-tile">
            <div className="nfm-stat-tile__icon nfm-stat-tile__icon--cyan"><Box /></div>
            <div className="nfm-stat-tile__value">{DUMMY_STATUS.blocks.toLocaleString()}</div>
            <div className="nfm-stat-tile__label">Total Blocks</div>
            <div className="nfm-stat-tile__trend nfm-stat-tile__trend--up">
              <TrendingUp size={12} /> +12.4%
            </div>
          </div>
        </div>

        <div className="nfm-glass-card nfm-glass-card--interactive" style={{ marginBottom: 0 }}>
          <div className="nfm-stat-tile">
            <div className="nfm-stat-tile__icon nfm-stat-tile__icon--purple"><Zap /></div>
            <div className="nfm-stat-tile__value">24</div>
            <div className="nfm-stat-tile__label">Pending Transactions</div>
            <div className="nfm-stat-tile__trend nfm-stat-tile__trend--up">
              High Priority
            </div>
          </div>
        </div>

        <div className="nfm-glass-card" style={{ marginBottom: 0 }}>
          <div className="nfm-stat-tile">
            <div className="nfm-stat-tile__icon nfm-stat-tile__icon--cyan"><Database /></div>
            <div className="nfm-stat-tile__value">2.4 TB</div>
            <div className="nfm-stat-tile__label">Storage Occupied</div>
            <div className="nfm-stat-tile__trend nfm-stat-tile__trend--up">
               3,420 Active Nodes
            </div>
          </div>
        </div>

        <div className="nfm-glass-card" style={{ marginBottom: 0 }}>
          <div className="nfm-stat-tile">
            <div className="nfm-stat-tile__icon nfm-stat-tile__icon--pink"><Flame /></div>
            <div className="nfm-stat-tile__value">{DUMMY_STATUS.total_burned.toLocaleString()}</div>
            <div className="nfm-stat-tile__label">Total Burned (NVC)</div>
            <div className="nfm-stat-tile__trend nfm-stat-tile__trend--down">
               Deflation Rate: 0.12%
            </div>
          </div>
        </div>
      </div>

      <div className="flex gap-8" style={{ marginBottom: 'var(--space-8)' }}>
        <div className="nfm-glass-card" style={{ flex: 2, marginBottom: 0 }}>
          <div className="flex justify-between items-start mb-6">
            <div>
              <h2 className="text-cyan text-lg">Hashrate Performance</h2>
              <p className="text-xs text-muted">Network computational power over the last 24h.</p>
            </div>
            <div className="text-right">
              <div className="text-2xl font-display text-primary">8.42 EH/s</div>
              <div className="text-xs text-success">+5.2%</div>
            </div>
          </div>
          <NetworkChart data={chartData} color="var(--neon-cyan)" />
        </div>

        <div className="nfm-glass-card" style={{ flex: 1, marginBottom: 0 }}>
          <h2 className="text-purple text-lg mb-6">Next Epoch Countdown</h2>
          <div className="flex-col items-center justify-center p-8 gap-4" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-lg)', textAlign: 'center' }}>
            <div className="text-4xl font-display text-cyan">02:45:12</div>
            <div className="text-xs text-muted uppercase tracking-widest">Until Epoch Switch</div>
            <div className="nfm-progress mt-4">
              <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: '65%' }}></div>
            </div>
          </div>
        </div>
      </div>

      <div className="flex gap-8">
        <div className="nfm-glass-card" style={{ flex: 2, marginBottom: 0 }}>
          <h2 className="text-primary" style={{ marginBottom: 'var(--space-6)', fontSize: 'var(--text-lg)' }}>Recent Network Activity</h2>
          
          <table className="nfm-table">
            <thead>
              <tr>
                <th>Block Height</th>
                <th>Hash</th>
                <th>Txs</th>
                <th>Age</th>
              </tr>
            </thead>
            <tbody>
              {DUMMY_BLOCKS.slice(0, 5).map(block => (
                <tr key={block.index}>
                  <td className="font-mono text-cyan">#{block.index}</td>
                  <td className="font-mono">{block.hash.substring(0, 16)}...</td>
                  <td>{block.transactions}</td>
                  <td className="text-muted">{Math.floor((Date.now() - block.timestamp) / 1000)}s ago</td>
                </tr>
              ))}
            </tbody>
          </table>
          <button className="nfm-btn-more">
            <ArrowRight size={14} /> View Network History
          </button>
        </div>

        <div className="nfm-glass-card" style={{ flex: 1, marginBottom: 0 }}>
          <h2 className="text-cyan" style={{ marginBottom: 'var(--space-6)', fontSize: 'var(--text-lg)' }}>Node Connectivity</h2>
          <div className="flex-col gap-4">
            {[1, 2, 3].map(i => (
              <div key={i} className="flex justify-between items-center p-4" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(255,255,255,0.02)' }}>
                <div className="flex items-center gap-3">
                  <div className={`nfm-status-dot nfm-status-dot--${i === 3 ? 'syncing' : 'online'}`}></div>
                  <span className="font-mono text-sm">nfm-peer-00{i}</span>
                </div>
                <span className="text-xs text-muted">98ms</span>
              </div>
            ))}
            <button className="nfm-btn-more" style={{ marginTop: 'var(--space-2)' }}>
              Explore Peer Mesh
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
