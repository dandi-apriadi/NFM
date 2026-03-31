import { Box, Flame, Zap, Database, TrendingUp, ArrowRight } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { p2pBan, p2pBulkBan, p2pBulkUnban, p2pUnban } from '../api/client';
import NetworkChart from '../components/ui/NetworkChart';
import { useAppData } from '../context/AppDataContext';

const Dashboard = () => {
  const { data, p2p, refresh, refreshPaused, setRefreshPaused } = useAppData();
  const [sortMode, setSortMode] = useState<'score-desc' | 'score-asc'>(() => {
    const saved = localStorage.getItem('nfm.dashboard.peerSortMode');
    return saved === 'score-asc' ? 'score-asc' : 'score-desc';
  });
  const [qualityFilter, setQualityFilter] = useState<'all' | 'risk' | 'banned'>(() => {
    const saved = localStorage.getItem('nfm.dashboard.peerQualityFilter');
    if (saved === 'risk' || saved === 'banned') {
      return saved;
    }
    return 'all';
  });
  const [batchScoreThreshold, setBatchScoreThreshold] = useState<number>(() => {
    const saved = Number(localStorage.getItem('nfm.dashboard.batchScoreThreshold'));
    if (Number.isFinite(saved) && saved >= 0 && saved <= 100) {
      return saved;
    }
    return 40;
  });
  const [pendingEndpoints, setPendingEndpoints] = useState<string[]>([]);
  const [batchPending, setBatchPending] = useState(false);
  const [toast, setToast] = useState<{ type: 'success' | 'error'; message: string } | null>(null);
  const [lastBatchAction, setLastBatchAction] = useState<{
    ts: number;
    type: 'ban' | 'unban';
    endpoints: string[];
    reason?: string;
  } | null>(() => {
    const raw = localStorage.getItem('nfm.dashboard.lastBatchAction');
    if (!raw) {
      return null;
    }
    try {
      const parsed = JSON.parse(raw) as {
        ts: number;
        type: 'ban' | 'unban';
        endpoints: string[];
        reason?: string;
      };
      if (!Array.isArray(parsed.endpoints) || (parsed.type !== 'ban' && parsed.type !== 'unban')) {
        return null;
      }
      return parsed;
    } catch {
      return null;
    }
  });
  const [pauseStartedAt, setPauseStartedAt] = useState<number | null>(() => {
    const raw = localStorage.getItem('nfm.dashboard.pauseStartedAt');
    const parsed = Number(raw);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
  });
  const [lastPauseDurationSec, setLastPauseDurationSec] = useState<number>(() => {
    const raw = Number(localStorage.getItem('nfm.dashboard.lastPauseDurationSec'));
    return Number.isFinite(raw) && raw >= 0 ? raw : 0;
  });
  const [operatorLog, setOperatorLog] = useState<Array<{ ts: number; action: string; detail: string }>>(() => {
    const raw = localStorage.getItem('nfm.dashboard.operatorLog');
    if (!raw) {
      return [];
    }
    try {
      const parsed = JSON.parse(raw) as Array<{ ts: number; action: string; detail: string }>;
      return parsed.slice(0, 15);
    } catch {
      return [];
    }
  });
  const DUMMY_STATUS = data.status;
  const DUMMY_BLOCKS = data.blocks;
  const p2pOnline = p2p.gossip_enabled && p2p.status === 'online';
  const bannedSet = new Set(p2p.banned_peers ?? []);
  const allPeerRows = useMemo(() => {
    const telemetryMap = new Map((p2p.peer_health ?? []).map((entry) => [entry.endpoint, entry]));
    const knownRows = p2p.known_peers.map((endpoint) => {
      const telemetry = telemetryMap.get(endpoint);
      const quality = (telemetry?.quality || 'critical').toLowerCase();
      return {
        endpoint,
        healthy: telemetry?.healthy ?? p2pOnline,
        score: telemetry?.score ?? (p2pOnline ? 70 : 20),
        quality,
        latencyMs: telemetry?.latency_ms,
      };
    });

    const knownSet = new Set(knownRows.map((r) => r.endpoint));
    const bannedOnlyRows = (p2p.banned_peers ?? [])
      .filter((endpoint) => !knownSet.has(endpoint))
      .map((endpoint) => ({
        endpoint,
        healthy: false,
        score: 0,
        quality: 'critical',
        latencyMs: undefined as number | undefined,
      }));

    return [...knownRows, ...bannedOnlyRows];
  }, [p2p.known_peers, p2p.peer_health, p2p.banned_peers, p2pOnline]);

  const visiblePeerRows = useMemo(() => {
    const riskSet = new Set(['degraded', 'poor', 'critical']);
    let rows = allPeerRows;
    if (qualityFilter === 'risk') {
      rows = rows.filter((row) => riskSet.has(row.quality));
    } else if (qualityFilter === 'banned') {
      rows = rows.filter((row) => bannedSet.has(row.endpoint));
    }

    const sorted = [...rows].sort((a, b) => {
      if (sortMode === 'score-asc') {
        return a.score - b.score;
      }
      return b.score - a.score;
    });

    return sorted.slice(0, 8);
  }, [allPeerRows, qualityFilter, sortMode, p2p.banned_peers]);

  const qualityStats = (p2p.peer_health ?? []).reduce(
    (acc, peer) => {
      const key = (peer.quality || 'critical').toLowerCase();
      if (key === 'excellent' || key === 'good' || key === 'degraded' || key === 'poor' || key === 'critical') {
        acc[key] += 1;
      } else {
        acc.critical += 1;
      }
      return acc;
    },
    { excellent: 0, good: 0, degraded: 0, poor: 0, critical: 0 },
  );
  const atRiskCount = qualityStats.degraded + qualityStats.poor + qualityStats.critical;
  const atRiskBase = p2p.peer_health?.length || p2p.peer_count || 0;
  const atRiskRatioPct = atRiskBase > 0 ? Math.round((atRiskCount / atRiskBase) * 100) : 0;
  const riskCandidates = useMemo(
    () =>
      allPeerRows
        .filter(
          (row) =>
            (row.quality === 'degraded' || row.quality === 'poor' || row.quality === 'critical')
            && row.score <= batchScoreThreshold
            && !bannedSet.has(row.endpoint),
        )
        .map((row) => row.endpoint),
    [allPeerRows, p2p.banned_peers, batchScoreThreshold],
  );

  useEffect(() => {
    localStorage.setItem('nfm.dashboard.peerSortMode', sortMode);
  }, [sortMode]);

  useEffect(() => {
    localStorage.setItem('nfm.dashboard.peerQualityFilter', qualityFilter);
  }, [qualityFilter]);

  useEffect(() => {
    localStorage.setItem('nfm.dashboard.batchScoreThreshold', String(batchScoreThreshold));
  }, [batchScoreThreshold]);

  useEffect(() => {
    localStorage.setItem('nfm.dashboard.operatorLog', JSON.stringify(operatorLog.slice(0, 15)));
  }, [operatorLog]);

  useEffect(() => {
    if (!lastBatchAction) {
      localStorage.removeItem('nfm.dashboard.lastBatchAction');
    } else {
      localStorage.setItem('nfm.dashboard.lastBatchAction', JSON.stringify(lastBatchAction));
    }
  }, [lastBatchAction]);

  useEffect(() => {
    if (pauseStartedAt === null) {
      localStorage.removeItem('nfm.dashboard.pauseStartedAt');
    } else {
      localStorage.setItem('nfm.dashboard.pauseStartedAt', String(pauseStartedAt));
    }
  }, [pauseStartedAt]);

  useEffect(() => {
    localStorage.setItem('nfm.dashboard.lastPauseDurationSec', String(lastPauseDurationSec));
  }, [lastPauseDurationSec]);

  useEffect(() => {
    if (!toast) {
      return;
    }
    const timer = window.setTimeout(() => setToast(null), 2200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  const isEndpointPending = (endpoint: string) => pendingEndpoints.includes(endpoint);

  const pushOperatorLog = (action: string, detail: string) => {
    setOperatorLog((prev) => [{ ts: Date.now(), action, detail }, ...prev].slice(0, 15));
  };

  const setEndpointPending = (endpoint: string, pending: boolean) => {
    setPendingEndpoints((prev) => {
      if (pending) {
        return prev.includes(endpoint) ? prev : [...prev, endpoint];
      }
      return prev.filter((item) => item !== endpoint);
    });
  };

  const inFlight = batchPending || pendingEndpoints.length > 0;

  const handleQuickBan = async (endpoint: string) => {
    if (bannedSet.has(endpoint) || inFlight) {
      return;
    }
    if (!window.confirm(`Ban peer ${endpoint}?`)) {
      return;
    }

    try {
      setEndpointPending(endpoint, true);
      await p2pBan(endpoint);
      await refresh();
      setToast({ type: 'success', message: `Ban accepted: ${endpoint}` });
      pushOperatorLog('BAN', endpoint);
    } catch (e) {
      setToast({ type: 'error', message: e instanceof Error ? e.message : 'Failed to ban peer' });
      pushOperatorLog('BAN_FAIL', endpoint);
    } finally {
      setEndpointPending(endpoint, false);
    }
  };

  const handleQuickUnban = async (endpoint: string) => {
    if (inFlight) {
      return;
    }
    if (!window.confirm(`Unban peer ${endpoint}?`)) {
      return;
    }

    try {
      setEndpointPending(endpoint, true);
      await p2pUnban(endpoint);
      await refresh();
      setToast({ type: 'success', message: `Unban accepted: ${endpoint}` });
      pushOperatorLog('UNBAN', endpoint);
    } catch (e) {
      setToast({ type: 'error', message: e instanceof Error ? e.message : 'Failed to unban peer' });
      pushOperatorLog('UNBAN_FAIL', endpoint);
    } finally {
      setEndpointPending(endpoint, false);
    }
  };

  const handleExportRiskList = async () => {
    const payload = riskCandidates.join('\n');
    if (!payload) {
      setToast({ type: 'success', message: 'No risk peers to export' });
      return;
    }

    try {
      await navigator.clipboard.writeText(payload);
      setToast({ type: 'success', message: `Risk list copied (${riskCandidates.length})` });
      pushOperatorLog('EXPORT_RISK_LIST', `${riskCandidates.length} endpoints`);
    } catch {
      setToast({ type: 'error', message: 'Clipboard write failed' });
      pushOperatorLog('EXPORT_RISK_LIST_FAIL', `${riskCandidates.length} endpoints`);
    }
  };

  const handleImportBanList = async () => {
    if (inFlight) {
      return;
    }

    const sample = (p2p.banned_peers ?? []).slice(0, 5).join('\n');
    const raw = window.prompt(
      'Paste endpoints to ban (newline or comma separated, format host:port)',
      sample,
    );
    if (raw === null) {
      return;
    }

    const candidates = Array.from(
      new Set(
        raw
          .split(/\n|,/)
          .map((item) => item.trim())
          .filter((item) => item.length > 0 && item.includes(':') && !bannedSet.has(item)),
      ),
    );

    if (candidates.length === 0) {
      setToast({ type: 'success', message: 'No valid new endpoints to import' });
      return;
    }

    const reasonInput = window.prompt('Optional reason for import batch action', 'imported list');
    if (reasonInput === null) {
      return;
    }
    const reasonSuffix = reasonInput.trim() ? ` | reason: ${reasonInput.trim()}` : '';

    if (!window.confirm(`Import and ban ${candidates.length} endpoint(s)?`)) {
      return;
    }

    let success = 0;
    setBatchPending(true);
    try {
      setPendingEndpoints(candidates);
      const response = (await p2pBulkBan(candidates)) as { accepted_count?: number; endpoints?: string[] };
      success = response.accepted_count ?? 0;
      const acceptedEndpoints = response.endpoints ?? [];
      await refresh();
      setToast({ type: 'success', message: `Import ban done: ${success}/${candidates.length}` });
      pushOperatorLog('IMPORT_BAN_LIST', `${success}/${candidates.length}${reasonSuffix}`);
      if (acceptedEndpoints.length > 0) {
        setLastBatchAction({
          ts: Date.now(),
          type: 'ban',
          endpoints: acceptedEndpoints,
          reason: reasonInput.trim() || undefined,
        });
      }
    } finally {
      setPendingEndpoints([]);
      setBatchPending(false);
    }
  };

  const handleUndoLastBatch = async () => {
    if (inFlight || !lastBatchAction || lastBatchAction.endpoints.length === 0) {
      return;
    }

    const actionLabel = lastBatchAction.type === 'ban' ? 'unban' : 'ban';
    const reason = lastBatchAction.reason ? `\nOriginal reason: ${lastBatchAction.reason}` : '';
    const ok = window.confirm(
      `Undo last batch by applying ${actionLabel} to ${lastBatchAction.endpoints.length} endpoint(s)?${reason}`,
    );
    if (!ok) {
      return;
    }

    let success = 0;
    setBatchPending(true);
    setPendingEndpoints(lastBatchAction.endpoints);
    try {
      if (lastBatchAction.type === 'ban') {
        const response = (await p2pBulkUnban(lastBatchAction.endpoints)) as { accepted_count?: number };
        success = response.accepted_count ?? 0;
      } else {
        const response = (await p2pBulkBan(lastBatchAction.endpoints)) as { accepted_count?: number };
        success = response.accepted_count ?? 0;
      }
      await refresh();
      setToast({ type: 'success', message: `Undo batch done: ${success}/${lastBatchAction.endpoints.length}` });
      pushOperatorLog('UNDO_BATCH', `${success}/${lastBatchAction.endpoints.length} (${lastBatchAction.type})`);
      setLastBatchAction(null);
    } catch (e) {
      setToast({ type: 'error', message: e instanceof Error ? e.message : 'Undo batch failed' });
      pushOperatorLog('UNDO_BATCH_FAIL', lastBatchAction.type);
    } finally {
      setPendingEndpoints([]);
      setBatchPending(false);
    }
  };

  const handleExportOperatorLog = () => {
    if (operatorLog.length === 0) {
      setToast({ type: 'success', message: 'No operator log to export' });
      return;
    }

    const lines = operatorLog.map((entry) => {
      const time = new Date(entry.ts).toISOString();
      return `${time} | ${entry.action} | ${entry.detail}`;
    });
    const content = lines.join('\n');
    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `nfm-operator-log-${Date.now()}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    setToast({ type: 'success', message: `Operator log exported (${operatorLog.length})` });
    pushOperatorLog('EXPORT_OPERATOR_LOG', `${operatorLog.length} entries`);
  };

  const handleClearOperatorLog = () => {
    if (operatorLog.length === 0) {
      setToast({ type: 'success', message: 'Operator log already empty' });
      return;
    }

    if (!window.confirm(`Clear ${operatorLog.length} operator log entries?`)) {
      return;
    }

    setOperatorLog([]);
    localStorage.removeItem('nfm.dashboard.operatorLog');
    setToast({ type: 'success', message: 'Operator log cleared' });
  };

  const handleToggleRefreshPause = (nextPaused: boolean) => {
    if (nextPaused) {
      const now = Date.now();
      setPauseStartedAt(now);
      setRefreshPaused(true);
      pushOperatorLog('PAUSE_REFRESH', 'manual toggle');
      setToast({ type: 'success', message: 'Auto-refresh paused' });
      return;
    }

    const now = Date.now();
    const started = pauseStartedAt;
    let durationSec = 0;
    if (started && started <= now) {
      durationSec = Math.round((now - started) / 1000);
      setLastPauseDurationSec(durationSec);
    }
    setPauseStartedAt(null);
    setRefreshPaused(false);
    pushOperatorLog('RESUME_REFRESH', durationSec > 0 ? `${durationSec}s paused` : 'manual toggle');
    setToast({ type: 'success', message: 'Auto-refresh resumed' });
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const tag = (event.target as HTMLElement | null)?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea') {
        return;
      }

      if ((event.key === 'p' || event.key === 'P') && !refreshPaused && !inFlight) {
        event.preventDefault();
        handleToggleRefreshPause(true);
      }
      if ((event.key === 'r' || event.key === 'R') && refreshPaused && !inFlight) {
        event.preventDefault();
        handleToggleRefreshPause(false);
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [refreshPaused, inFlight, pauseStartedAt]);

  const handleBanAllRisk = async () => {
    if (inFlight) {
      return;
    }

    if (riskCandidates.length === 0) {
      setToast({ type: 'success', message: 'No risk peers to ban' });
      return;
    }

    const preview = riskCandidates.slice(0, 8).join('\n- ');
    const moreSuffix = riskCandidates.length > 8 ? `\n...and ${riskCandidates.length - 8} more` : '';
    const confirmMessage = `Ban ${riskCandidates.length} risk peers now?\n\nTargets:\n- ${preview}${moreSuffix}`;

    if (!window.confirm(confirmMessage)) {
      return;
    }

    const reasonInput = window.prompt('Optional reason for this batch action (saved to operator log)', '');
    if (reasonInput === null) {
      return;
    }
    const reasonSuffix = reasonInput.trim() ? ` | reason: ${reasonInput.trim()}` : '';

    let success = 0;
    setBatchPending(true);
    try {
      setPendingEndpoints(riskCandidates);
      const response = (await p2pBulkBan(riskCandidates)) as { accepted_count?: number; endpoints?: string[] };
      success = response.accepted_count ?? 0;
      const acceptedEndpoints = response.endpoints ?? [];
      await refresh();
      setToast({ type: 'success', message: `Batch ban done: ${success}/${riskCandidates.length}` });
      pushOperatorLog('BATCH_BAN_RISK', `${success}/${riskCandidates.length}${reasonSuffix}`);
      if (acceptedEndpoints.length > 0) {
        setLastBatchAction({
          ts: Date.now(),
          type: 'ban',
          endpoints: acceptedEndpoints,
          reason: reasonInput.trim() || undefined,
        });
      }
    } finally {
      setPendingEndpoints([]);
      setBatchPending(false);
    }
  };

  const handleUnbanAll = async () => {
    if (inFlight) {
      return;
    }

    const candidates = p2p.banned_peers ?? [];
    if (candidates.length === 0) {
      setToast({ type: 'success', message: 'No banned peers to unban' });
      return;
    }

    if (!window.confirm(`Unban all ${candidates.length} peers?`)) {
      return;
    }

    const reasonInput = window.prompt('Optional reason for this batch action (saved to operator log)', '');
    if (reasonInput === null) {
      return;
    }
    const reasonSuffix = reasonInput.trim() ? ` | reason: ${reasonInput.trim()}` : '';

    let success = 0;
    setBatchPending(true);
    try {
      setPendingEndpoints(candidates);
      const response = (await p2pBulkUnban(candidates)) as { accepted_count?: number; endpoints?: string[] };
      success = response.accepted_count ?? 0;
      const acceptedEndpoints = response.endpoints ?? [];
      await refresh();
      setToast({ type: 'success', message: `Batch unban done: ${success}/${candidates.length}` });
      pushOperatorLog('BATCH_UNBAN_ALL', `${success}/${candidates.length}${reasonSuffix}`);
      if (acceptedEndpoints.length > 0) {
        setLastBatchAction({
          ts: Date.now(),
          type: 'unban',
          endpoints: acceptedEndpoints,
          reason: reasonInput.trim() || undefined,
        });
      }
    } finally {
      setPendingEndpoints([]);
      setBatchPending(false);
    }
  };

  const chartData = [45, 52, 48, 70, 85, 74, 90, 82, 95];

  return (
    <div className="animate-in">
      {toast && (
        <div
          style={{
            position: 'fixed',
            top: '18px',
            right: '18px',
            zIndex: 60,
            borderRadius: '12px',
            padding: '10px 12px',
            border: '1px solid rgba(255,255,255,0.12)',
            background: toast.type === 'success' ? 'rgba(13, 185, 122, 0.16)' : 'rgba(220, 38, 38, 0.16)',
            color: 'var(--text-primary)',
            fontSize: '12px',
            backdropFilter: 'blur(6px)',
          }}
        >
          {toast.message}
        </div>
      )}
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <div>
          <h1 className="text-cyan">Network Dashboard</h1>
          <p className="text-muted text-sm mt-1">Real-time status of the NFM decentralized mesh network.</p>
        </div>
        <div className={`nfm-badge ${refreshPaused ? '' : 'nfm-badge--success'}`} style={refreshPaused ? { borderColor: 'rgba(245, 158, 11, 0.45)', color: '#f59e0b' } : undefined}>
          <div className="nfm-badge__dot"></div>
          {refreshPaused ? 'PAUSED' : 'LIVE'}
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
            <div className="nfm-stat-tile__value">{p2p.peer_count.toLocaleString()}</div>
            <div className="nfm-stat-tile__label">Connected Peers</div>
            <div className="nfm-stat-tile__trend nfm-stat-tile__trend--up">
              At-risk: {atRiskCount}/{atRiskBase || p2p.peer_count} ({atRiskRatioPct}%) | Seeds: {p2p.seed_count}
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
            <div className="flex gap-2" style={{ marginBottom: 'var(--space-2)' }}>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '30px', fontSize: '11px', padding: '0 10px' }}
                disabled={inFlight}
                onClick={() => setSortMode(sortMode === 'score-desc' ? 'score-asc' : 'score-desc')}
              >
                Sort: {sortMode === 'score-desc' ? 'High Score' : 'Low Score'}
              </button>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '30px', fontSize: '11px', padding: '0 10px' }}
                disabled={inFlight}
                onClick={() => {
                  setQualityFilter(
                    qualityFilter === 'all'
                      ? 'risk'
                      : qualityFilter === 'risk'
                        ? 'banned'
                        : 'all',
                  );
                }}
              >
                Filter: {qualityFilter === 'all' ? 'All' : qualityFilter === 'risk' ? 'Risk Only' : 'Banned'}
              </button>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '30px', fontSize: '11px', padding: '0 10px' }}
                disabled={inFlight}
                onClick={() => handleToggleRefreshPause(!refreshPaused)}
              >
                {refreshPaused ? 'Resume Refresh' : 'Pause Refresh'}
              </button>
            </div>
            <div className="flex gap-2" style={{ marginBottom: 'var(--space-2)' }}>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '28px', fontSize: '10px', padding: '0 8px' }}
                disabled={inFlight}
                onClick={handleBanAllRisk}
              >
                {batchPending ? 'Batch...' : `Ban All Risk (${riskCandidates.length})`}
              </button>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '28px', fontSize: '10px', padding: '0 8px' }}
                disabled={inFlight}
                onClick={handleUnbanAll}
              >
                {batchPending ? 'Batch...' : 'Unban All'}
              </button>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '28px', fontSize: '10px', padding: '0 8px' }}
                disabled={inFlight || riskCandidates.length === 0}
                onClick={handleExportRiskList}
              >
                Export Risk List
              </button>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '28px', fontSize: '10px', padding: '0 8px' }}
                disabled={inFlight || !lastBatchAction || lastBatchAction.endpoints.length === 0}
                onClick={handleUndoLastBatch}
              >
                Undo Last Batch
              </button>
              <button
                className="nfm-btn nfm-btn--secondary"
                style={{ height: '28px', fontSize: '10px', padding: '0 8px' }}
                disabled={inFlight}
                onClick={handleImportBanList}
              >
                Import Ban List
              </button>
            </div>
            <div className="p-3 text-xs" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(255,255,255,0.02)', marginBottom: 'var(--space-2)' }}>
              <div className="text-muted" style={{ marginBottom: 'var(--space-1)' }}>
                Dry Run Preview: {riskCandidates.length} risk peer(s) eligible for batch ban (score &lt;= {batchScoreThreshold})
              </div>
              <div className="flex items-center gap-2" style={{ marginBottom: 'var(--space-1)' }}>
                <span className="text-muted">Threshold</span>
                <input
                  type="number"
                  min={0}
                  max={100}
                  value={batchScoreThreshold}
                  disabled={inFlight}
                  onChange={(e) => {
                    const next = Number(e.target.value);
                    if (Number.isFinite(next)) {
                      setBatchScoreThreshold(Math.min(100, Math.max(0, next)));
                    }
                  }}
                  style={{
                    width: '70px',
                    background: 'rgba(255,255,255,0.04)',
                    border: '1px solid rgba(255,255,255,0.12)',
                    borderRadius: '8px',
                    color: 'var(--text-primary)',
                    padding: '4px 6px',
                    fontSize: '11px',
                  }}
                />
                <span className="text-muted">(0-100)</span>
              </div>
              <div className="flex gap-2" style={{ marginBottom: 'var(--space-1)' }}>
                {[10, 25, 40, 60].map((preset) => (
                  <button
                    key={preset}
                    className="nfm-btn nfm-btn--secondary"
                    style={{ height: '24px', fontSize: '10px', padding: '0 6px' }}
                    disabled={inFlight}
                    onClick={() => setBatchScoreThreshold(preset)}
                  >
                    {preset}
                  </button>
                ))}
              </div>
              {riskCandidates.length > 0 ? (
                <div className="font-mono" style={{ maxHeight: '88px', overflowY: 'auto', lineHeight: 1.5 }}>
                  {riskCandidates.slice(0, 8).map((endpoint) => (
                    <div key={endpoint}>- {endpoint}</div>
                  ))}
                  {riskCandidates.length > 8 && <div>- ...and {riskCandidates.length - 8} more</div>}
                </div>
              ) : (
                <div className="text-muted">No risk peer currently eligible.</div>
              )}
            </div>
            <div className="p-3 text-xs" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(255,255,255,0.02)', marginBottom: 'var(--space-2)' }}>
              <div className="text-muted" style={{ marginBottom: 'var(--space-1)' }}>Operator Activity (latest 6)</div>
              <div className="text-muted" style={{ marginBottom: 'var(--space-1)' }}>
                Last pause duration: {lastPauseDurationSec}s | Shortcuts: P (pause), R (resume)
              </div>
              <div className="flex gap-2" style={{ marginBottom: 'var(--space-1)' }}>
                <button
                  className="nfm-btn nfm-btn--secondary"
                  style={{ height: '24px', fontSize: '10px', padding: '0 6px' }}
                  onClick={handleExportOperatorLog}
                >
                  Export Log
                </button>
                <button
                  className="nfm-btn nfm-btn--secondary"
                  style={{ height: '24px', fontSize: '10px', padding: '0 6px' }}
                  onClick={handleClearOperatorLog}
                >
                  Clear Log
                </button>
              </div>
              {operatorLog.length > 0 ? (
                <div style={{ maxHeight: '96px', overflowY: 'auto' }}>
                  {operatorLog.slice(0, 6).map((entry) => (
                    <div key={`${entry.ts}-${entry.action}-${entry.detail}`} className="flex items-center justify-between" style={{ marginBottom: '4px' }}>
                      <span className="font-mono text-muted">{new Date(entry.ts).toLocaleTimeString()}</span>
                      <span className="text-cyan" style={{ marginLeft: '8px', marginRight: '8px' }}>{entry.action}</span>
                      <span className="font-mono text-muted" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '46%' }}>{entry.detail}</span>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-muted">No operator action recorded yet.</div>
              )}
            </div>
            {visiblePeerRows.length > 0 ? visiblePeerRows.map((peer) => (
              <div key={peer.endpoint} className="flex justify-between items-center p-4" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(255,255,255,0.02)' }}>
                <div className="flex items-center gap-3">
                  <div className={`nfm-status-dot nfm-status-dot--${peer.healthy ? 'online' : 'syncing'}`}></div>
                  <div>
                    <span className="font-mono text-sm">{peer.endpoint}</span>
                    <div className="text-xs text-muted">{peer.latencyMs !== undefined ? `${peer.latencyMs} ms` : 'n/a latency'} | score {peer.score}</div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    className="nfm-btn nfm-btn--secondary"
                    style={{ height: '28px', fontSize: '10px', padding: '0 8px' }}
                    disabled={bannedSet.has(peer.endpoint) || inFlight}
                    onClick={() => handleQuickBan(peer.endpoint)}
                  >
                    {isEndpointPending(peer.endpoint) ? '...' : bannedSet.has(peer.endpoint) ? 'Banned' : 'Ban'}
                  </button>
                  <span
                    className={`text-xs ${peer.quality === 'excellent' || peer.quality === 'good' ? 'text-success' : peer.quality === 'degraded' ? 'text-warning' : 'text-danger'}`}
                    style={{
                      textTransform: 'uppercase',
                      letterSpacing: '0.06em',
                      border: '1px solid rgba(255,255,255,0.08)',
                      borderRadius: '999px',
                      padding: '4px 8px',
                    }}
                  >
                    {peer.quality}
                  </span>
                </div>
              </div>
            )) : (
              <div className="p-4 text-xs text-muted" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(255,255,255,0.02)' }}>
                {allPeerRows.length === 0
                  ? 'No peers discovered yet. Add NFM_P2P_SEEDS to bootstrap mesh discovery.'
                  : 'No peer matched current filter. Switch back to All filter.'}
              </div>
            )}
            <div className="text-xs text-muted" style={{ paddingLeft: 'var(--space-1)' }}>
              P2P status: {p2p.status} | Port: {p2p.listening_port} | Last sync: {p2p.last_sync_unix > 0 ? new Date(p2p.last_sync_unix * 1000).toLocaleTimeString() : '-'}
            </div>
            <div className="p-4 text-xs" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(255,255,255,0.02)' }}>
              <div className="text-muted" style={{ marginBottom: 'var(--space-2)' }}>Banned Peers</div>
              {p2p.banned_peers && p2p.banned_peers.length > 0 ? (
                <div className="flex-col gap-2">
                  {p2p.banned_peers.slice(0, 5).map((endpoint) => (
                    <div key={endpoint} className="flex items-center justify-between" style={{ marginBottom: 'var(--space-1)' }}>
                      <span className="font-mono text-muted">{endpoint}</span>
                      <button
                        className="nfm-btn nfm-btn--secondary"
                        style={{ height: '26px', fontSize: '10px', padding: '0 8px' }}
                        disabled={inFlight}
                        onClick={() => handleQuickUnban(endpoint)}
                      >
                        {isEndpointPending(endpoint) ? '...' : 'Unban'}
                      </button>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-muted">No banned peers.</div>
              )}
            </div>
            <div className="p-4 text-xs" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(255,255,255,0.02)' }}>
              <div className="text-muted" style={{ marginBottom: 'var(--space-2)' }}>Peer Quality Distribution</div>
              <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-1)' }}>
                <span className="text-success">Excellent</span>
                <span className="font-mono">{qualityStats.excellent}</span>
              </div>
              <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-1)' }}>
                <span className="text-cyan">Good</span>
                <span className="font-mono">{qualityStats.good}</span>
              </div>
              <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-1)' }}>
                <span className="text-warning">Degraded</span>
                <span className="font-mono">{qualityStats.degraded}</span>
              </div>
              <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-1)' }}>
                <span className="text-orange-300">Poor</span>
                <span className="font-mono">{qualityStats.poor}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-danger">Critical</span>
                <span className="font-mono">{qualityStats.critical}</span>
              </div>
            </div>
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
