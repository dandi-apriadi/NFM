import { Code, Terminal, Key, Database } from 'lucide-react';
import { useState } from 'react';
import { useAppData } from '../context/AppDataContext';
import { appNlcPreview } from '../api/client';

const DevPortal = () => {
  const { data, refresh, requestConfirm, notifySuccess, notifyError } = useAppData();
  const DUMMY_API_DOCS = data.api_docs;
  const [commandInput, setCommandInput] = useState('');
  const [nlcInput, setNlcInput] = useState('');
  const [nlcPreview, setNlcPreview] = useState<{ action?: string; amount?: number; target?: string; resolved_target?: string; executable?: boolean; reason?: string } | null>(null);
  const [secureDraft, setSecureDraft] = useState<{ body: string; curl: string; pwsh: string } | null>(null);
  const [nlcHistory, setNlcHistory] = useState<string[]>(() => {
    try {
      const raw = localStorage.getItem('nfm.dev.nlcHistory');
      const parsed = raw ? JSON.parse(raw) as string[] : [];
      return Array.isArray(parsed) ? parsed.slice(0, 10) : [];
    } catch {
      return [];
    }
  });
  const [consoleLines, setConsoleLines] = useState<string[]>([
    '$ nfm-cli query mempool --limit 5',
    '[]',
    '$ nfm-cli auth bio-zkp request --id user_01',
    'Challenge generated. Awaiting smartphone signature...',
  ]);

  const appendConsole = (line: string) => {
    setConsoleLines((prev) => [...prev, line].slice(-40));
  };

  const handleExecuteCommand = async () => {
    const cmd = commandInput.trim();
    if (!cmd) {
      notifyError('Command is empty');
      return;
    }

    appendConsole(`$ ${cmd}`);
    if (cmd === 'refresh' || cmd === 'sync') {
      await refresh();
      appendConsole('State refreshed from backend');
      notifySuccess('Command executed successfully');
    } else if (cmd === 'help') {
      appendConsole('Available commands: help, refresh, sync, status');
      notifySuccess('Help rendered in console');
    } else if (cmd === 'status') {
      appendConsole(`Blocks=${data.status.blocks}, Peers=${data.status.peers}, Burned=${data.status.total_burned}`);
      notifySuccess('Status snapshot generated');
    } else {
      appendConsole('Command recognized by UI shell only. Backend RPC bridge not available here yet.');
      notifyError('Unknown command for current UI shell');
    }

    setCommandInput('');
  };

  const handleResetChainState = async () => {
    const confirmed = await requestConfirm({
      title: 'Reset Chain State',
      message: 'This UI action is currently disabled. Use signed admin endpoint from secure console. Continue?',
      confirmText: 'Acknowledge',
      cancelText: 'Cancel',
    });

    if (!confirmed) {
      return;
    }
    notifyError('Reset chain from Dev Portal UI is disabled for safety');
  };

  const handleGenerateKey = () => {
    const generated = `nfm_dev_${Math.random().toString(36).slice(2, 12)}`;
    appendConsole(`Generated local API key preview: ${generated}`);
    notifySuccess('Local API key preview generated');
  };

  const pushNlcHistory = (intent: string) => {
    const next = [intent, ...nlcHistory.filter((x) => x !== intent)].slice(0, 10);
    setNlcHistory(next);
    localStorage.setItem('nfm.dev.nlcHistory', JSON.stringify(next));
  };

  const handlePreviewNlc = async () => {
    const input = nlcInput.trim();
    if (!input) {
      notifyError('NLC intent is empty');
      return;
    }

    try {
      const res = await appNlcPreview(input, data.user_profile.nfmAddress) as {
        preview?: { action?: string; amount?: number; target?: string; resolved_target?: string; executable?: boolean; reason?: string };
      };
      setNlcPreview(res.preview ?? null);
      pushNlcHistory(input);
      appendConsole(`NLC Preview => action=${res.preview?.action ?? 'UNKNOWN'} executable=${res.preview?.executable ? 'true' : 'false'}`);
      notifySuccess('NLC preview generated');
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Failed to preview NLC intent');
    }
  };

  const handleBuildSignedRequest = async () => {
    const input = nlcInput.trim();
    if (!input) {
      notifyError('NLC intent is empty');
      return;
    }

    const bodyObj = {
      input,
      address: data.user_profile.nfmAddress,
      ts: Date.now(),
      nonce: `nlc-${Math.random().toString(36).slice(2, 10)}`,
    };
    const body = JSON.stringify(bodyObj);
    const escapedBody = body.replace(/"/g, '\\"');
    const curl = [
      'curl -X POST "http://127.0.0.1:3000/api/nlc"',
      '  -H "Content-Type: application/json"',
      '  -H "x-signature: <sha256(secret:url:body)>"',
      `  -d "${escapedBody}"`,
    ].join(' \\\n');
    const pwsh = [
      '$body = @\'' ,
      body,
      '\'@',
      '$sig = "<sha256(secret:/api/nlc:$body)>"',
      'Invoke-WebRequest -Uri "http://127.0.0.1:3000/api/nlc" -Method POST -ContentType "application/json" -Headers @{ "x-signature" = $sig } -Body $body',
    ].join('\n');

    setSecureDraft({ body: JSON.stringify(bodyObj, null, 2), curl, pwsh });
    notifySuccess('Signed request draft generated');
    appendConsole('NLC secure draft generated (signature placeholder only)');
  };

  const handleCopyDraft = async (value: string, label: string) => {
    try {
      await navigator.clipboard.writeText(value);
      notifySuccess(`${label} copied`);
    } catch {
      notifyError(`Failed to copy ${label.toLowerCase()}`);
    }
  };

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
              {consoleLines.map((line, idx) => (
                <div key={idx} className={`${line.startsWith('$') ? 'text-muted' : 'text-cyan'} mt-1`}>{line}</div>
              ))}
              <div className="flex items-center text-muted">
                 $ <span className="ml-2 w-2 h-4 bg-cyan animate-pulse inline-block"></span>
              </div>
            </div>

            <div className="flex gap-4">
              <input
                type="text"
                className="nfm-search__input flex-1"
                placeholder="Enter command..."
                value={commandInput}
                onChange={(e) => setCommandInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault();
                    void handleExecuteCommand();
                  }
                }}
              />
              <button className="nfm-btn nfm-btn--primary" onClick={() => void handleExecuteCommand()}>Execute</button>
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
            <h3 className="text-lg text-primary mb-4">NLC Intent Builder</h3>
            <div className="text-xs text-muted mb-3">Preview intent parsing safely before signed execution on protected NLC endpoint.</div>
            <input
              type="text"
              className="nfm-search__input mb-3"
              placeholder="e.g. send 12.5 @alice"
              value={nlcInput}
              onChange={(e) => setNlcInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  void handlePreviewNlc();
                }
              }}
            />
            <button className="nfm-btn nfm-btn--primary w-full" onClick={() => void handlePreviewNlc()}>Preview Intent</button>
            <button className="nfm-btn nfm-btn--ghost w-full mt-2" onClick={() => void handleBuildSignedRequest()}>Build Signed Request Draft</button>

            {nlcPreview && (
              <div className="mt-4 p-3" style={{ border: '1px solid rgba(255,255,255,0.08)', borderRadius: '8px' }}>
                <div className="text-xs mb-1"><span className="text-muted">Action:</span> <span className="text-cyan">{nlcPreview.action ?? 'UNKNOWN'}</span></div>
                <div className="text-xs mb-1"><span className="text-muted">Executable:</span> {nlcPreview.executable ? 'true' : 'false'}</div>
                {typeof nlcPreview.amount === 'number' && <div className="text-xs mb-1"><span className="text-muted">Amount:</span> {nlcPreview.amount}</div>}
                {nlcPreview.target && <div className="text-xs mb-1"><span className="text-muted">Target:</span> {nlcPreview.target}</div>}
                {nlcPreview.resolved_target && <div className="text-xs mb-1"><span className="text-muted">Resolved:</span> {nlcPreview.resolved_target}</div>}
                {nlcPreview.reason && <div className="text-xs"><span className="text-muted">Reason:</span> {nlcPreview.reason}</div>}
              </div>
            )}

            {nlcHistory.length > 0 && (
              <div className="mt-4">
                <div className="text-xs text-muted mb-2">Recent Intents</div>
                {nlcHistory.slice(0, 5).map((h) => (
                  <button
                    key={h}
                    className="nfm-btn nfm-btn--ghost nfm-btn--sm w-full mb-1"
                    onClick={() => setNlcInput(h)}
                    style={{ justifyContent: 'flex-start' }}
                  >
                    {h}
                  </button>
                ))}
              </div>
            )}

            {secureDraft && (
              <div className="mt-4 p-3" style={{ border: '1px solid rgba(255,255,255,0.08)', borderRadius: '8px' }}>
                <div className="text-xs text-muted mb-2">Secure Execution Draft (signature placeholder only)</div>
                <div className="text-[11px] mb-2" style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>{secureDraft.body}</div>
                <div className="flex gap-2 mb-2">
                  <button className="nfm-btn nfm-btn--ghost nfm-btn--sm" onClick={() => void handleCopyDraft(secureDraft.body, 'Payload')}>Copy Payload</button>
                  <button className="nfm-btn nfm-btn--ghost nfm-btn--sm" onClick={() => void handleCopyDraft(secureDraft.curl, 'cURL Draft')}>Copy cURL</button>
                  <button className="nfm-btn nfm-btn--ghost nfm-btn--sm" onClick={() => void handleCopyDraft(secureDraft.pwsh, 'PowerShell Draft')}>Copy PS</button>
                </div>
                <div className="text-[10px] text-muted mb-2">Generate real signature only from trusted secure console with API secret.</div>
                <div className="text-[10px] text-cyan p-2" style={{ background: 'rgba(0, 245, 255, 0.05)', borderRadius: '4px' }}>
                  <strong>→ Use Secure Executor:</strong> Run <code style={{ fontSize: '9px' }}>./scripts/nlc_secure_executor.ps1 -Interactive</code> to compute the real signature and execute the intent with your API secret securely.
                </div>
              </div>
            )}
          </div>
          
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

            <button className="nfm-btn nfm-btn--ghost w-full mt-6 text-error hover:bg-error hover:text-white" style={{ borderColor: 'var(--error)' }} onClick={() => void handleResetChainState()}>
              Reset Chain State
            </button>
          </div>

          <div className="nfm-glass-card p-6 border text-center" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>
             <div className="w-16 h-16 rounded-full mx-auto mb-4 flex items-center justify-center" style={{ background: 'rgba(0,245,255,0.1)' }}>
               <Key size={32} className="text-cyan" />
             </div>
             <h3 className="text-md text-primary mb-2 font-bold tracking-wider">API Keys</h3>
             <p className="text-xs text-muted mb-4">Manage your OAuth & programmatic credentials.</p>
             <button className="nfm-btn nfm-btn--secondary w-full" onClick={handleGenerateKey}>Generate New Key</button>
          </div>

        </div>

      </div>
    </div>
  );
};

export default DevPortal;
