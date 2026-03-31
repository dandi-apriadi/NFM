import { Code, Terminal, Key, Database } from 'lucide-react';
import { useAppData } from '../context/AppDataContext';

const DevPortal = () => {
  const { data } = useAppData();
  const DUMMY_API_DOCS = data.api_docs;

  return (
    <div className="animate-in">
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <h1 className="text-cyan flex items-center gap-2"><Code /> The Forge (Dev Portal)</h1>
        <div className="nfm-badge nfm-badge--cyan">
          <div className="nfm-badge__dot"></div> v0.8.4 Alpha API
        </div>
      </div>

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        
        {/* Core Dev Hub */}
        <div className="flex-col gap-6" style={{ flex: '2 1 600px' }}>
          
          <div className="nfm-glass-card nfm-glass-card--glow-cyan mb-8" style={{ display: 'flex', flexDirection: 'column' }}>
            <h2 className="text-xl text-primary mb-4 flex items-center gap-2">
               <Terminal className="text-cyan" /> Interactive RPC Console
            </h2>

            <div className="font-mono text-sm p-4 rounded-md mb-4" style={{ background: '#000', height: '200px', overflowY: 'auto', border: '1px solid rgba(0, 245, 255, 0.2)' }}>
              <div className="text-muted">$ nfm-cli query mempool --limit 5</div>
              <div className="text-cyan mt-1 mb-3">{JSON.stringify([
                { txid: "0x4f...a1", gas: 0.005, size: 245 },
                { txid: "0x8e...9b", gas: 0.001, size: 120 }
              ], null, 2)}</div>
              
              <div className="text-muted">$ nfm-cli auth bio-zkp request --id user_01</div>
              <div className="text-success mt-1 mb-3">Challenge generated. Awaiting smartphone signature...</div>
              
              <div className="flex items-center text-muted">
                 $ <span className="ml-2 w-2 h-4 bg-cyan animate-pulse inline-block"></span>
              </div>
            </div>

            <div className="flex gap-4">
              <input type="text" className="nfm-search__input flex-1" placeholder="Enter command..." />
              <button className="nfm-btn nfm-btn--primary">Execute</button>
            </div>
          </div>

          <div className="nfm-glass-card border" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>
             <h3 className="text-lg text-primary mb-4 border-b pb-2" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>API Endpoints</h3>
             <table className="nfm-table">
               <thead>
                 <tr>
                   <th>Method</th>
                   <th>Endpoint</th>
                   <th>Description</th>
                   <th>Auth</th>
                 </tr>
               </thead>
               <tbody>
                 {DUMMY_API_DOCS.map(doc => (
                   <tr key={doc.path} className="nfm-glass-card--interactive" style={{cursor: 'pointer'}}>
                     <td>
                       <span className={`nfm-badge nfm-badge--${doc.method === 'GET' ? 'cyan' : doc.method === 'POST' ? 'success' : doc.method === 'DELETE' ? 'error' : 'purple'} text-[10px]`}>
                         {doc.method}
                       </span>
                     </td>
                     <td className="font-mono text-sm">{doc.path}</td>
                     <td className="text-xs text-muted max-w-xs truncate">{doc.description}</td>
                     <td>
                        {doc.authRequired ? <Key size={14} className="text-gold" /> : <span className="text-xs text-secondary">None</span>}
                     </td>
                   </tr>
                 ))}
               </tbody>
             </table>
          </div>

        </div>

        {/* Integration Stats */}
        <div className="flex-col gap-6" style={{ flex: '1 1 300px' }}>
          
          <div className="nfm-glass-card" style={{ background: 'var(--surface-lowest)' }}>
            <h3 className="text-lg text-primary mb-4 flex items-center gap-2">
              <Database /> Local Dev Database
            </h3>
            <ul className="text-sm text-secondary flex-col gap-3">
              <li className="flex justify-between items-center">
                <span>RocksDB Size</span>
                <span className="font-mono text-cyan">14.2 GB</span>
              </li>
              <li className="flex justify-between items-center">
                <span>Mempool State</span>
                <span className="font-mono text-success">Clean (0 txs)</span>
              </li>
              <li className="flex justify-between items-center">
                <span>Vector Indexes</span>
                <span className="font-mono text-purple">125,430</span>
              </li>
            </ul>

            <button className="nfm-btn nfm-btn--ghost w-full mt-6 text-error hover:bg-error hover:text-white" style={{ borderColor: 'var(--error)' }}>
              Reset Chain State
            </button>
          </div>

          <div className="nfm-glass-card p-6 border text-center" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>
             <div className="w-16 h-16 rounded-full mx-auto mb-4 flex items-center justify-center" style={{ background: 'rgba(0,245,255,0.1)' }}>
               <Key size={32} className="text-cyan" />
             </div>
             <h3 className="text-md text-primary mb-2 font-bold tracking-wider">API Keys</h3>
             <p className="text-xs text-muted mb-4">Manage your OAuth & programmatic credentials.</p>
             <button className="nfm-btn nfm-btn--secondary w-full">Generate New Key</button>
          </div>

        </div>

      </div>
    </div>
  );
};

export default DevPortal;
