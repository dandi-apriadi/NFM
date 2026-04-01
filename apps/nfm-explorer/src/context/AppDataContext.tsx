import { createContext, useContext, useEffect, useMemo, useState } from 'react';
import type { Block, NFMStatus, NodeStats, P2PStatus, Transaction, UserProfile } from '../types';
import { appUpdateSettings } from '../api/client';

export interface AITask {
  id: string;
  name: string;
  status: 'RUNNING' | 'COMPLETED' | 'QUEUED' | 'FAILED';
  progress: number;
  model: string;
  cost: number;
}

export interface DriveFile {
  id: string;
  name: string;
  size: string;
  type: string;
  fragments: number;
  health: number;
  uploadedAt: number;
}

export interface KGConcept {
  id: string;
  name: string;
  connections: number;
  category: 'CODE' | 'DOCUMENT' | 'ENTITY';
}

export interface MarketItem {
  id: string;
  name?: string;
  title?: string;
  creator?: string;
  seller?: string;
  price: number;
  type?: 'AI_SKILL' | 'FRAGMENT' | 'NODE_LICENSE';
  sales?: number;
  rating?: number;
  auction_id?: number;
  starting_price?: number;
  highest_bid?: number;
  highest_bidder?: string;
  rarity?: string;
  power_multiplier?: number;
  status?: string;
  end_time?: number;
}

export interface Quest {
  id: string;
  title: string;
  description: string;
  rewardNVC: number;
  progress: number;
  total: number;
  status: 'ACTIVE' | 'CLAIMABLE' | 'COMPLETED';
}

export interface BoxOpenHistory {
  id: string;
  timestamp: number;
  rarity: 'COMMON' | 'RARE' | 'EPIC' | 'LEGENDARY';
  rewardInfo: string;
}

export interface RewardItem {
  id: string;
  name: string;
  description: string;
  rarity: 'COMMON' | 'RARE' | 'EPIC' | 'LEGENDARY';
  type: 'NVC' | 'FRAGMENT' | 'SKILL' | 'LICENSE';
}

export interface MysteryNews {
  id: string;
  type: 'MISSION' | 'BURN' | 'RARE_FIND' | 'SYSTEM';
  content: string;
  timestamp: number;
  user?: string;
}

export interface Proposal {
  id: string;
  title: string;
  creator: string;
  status: 'ACTIVE' | 'PASSED' | 'REJECTED';
  forVotes: number;
  againstVotes: number;
  endTime: number;
}

export interface DevAPIEndpoint {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  path: string;
  description: string;
  authRequired: boolean;
}

export interface WalletSummary {
  name: string;
  address: string;
  balanceNVC: number;
  balanceETH: number;
  isActive: boolean;
}

export interface AppState {
  status: NFMStatus;
  blocks: Block[];
  transactions: Transaction[];
  user_profile: UserProfile;
  wallets: WalletSummary[];
  node_stats: NodeStats;
  ai_tasks: AITask[];
  drive_files: DriveFile[];
  kg_concepts: KGConcept[];
  market_items: MarketItem[];
  quests: Quest[];
  box_history: BoxOpenHistory[];
  reward_catalog: RewardItem[];
  mystery_news: MysteryNews[];
  proposals: Proposal[];
  api_docs: DevAPIEndpoint[];
}

export interface AppToast {
  type: 'success' | 'error';
  message: string;
}

interface PromptModalOptions {
  title: string;
  message: string;
  placeholder?: string;
  defaultValue?: string;
  confirmText?: string;
  cancelText?: string;
}

interface ConfirmModalOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
}

type ModalState =
  | {
      kind: 'prompt';
      options: PromptModalOptions;
      resolve: (value: string | null) => void;
    }
  | {
      kind: 'confirm';
      options: ConfirmModalOptions;
      resolve: (value: boolean) => void;
    };

const EMPTY_STATE: AppState = {
  status: {
    node: 'nfm-node',
    version: 'NFM Vault v1.2',
    status: 'SYNCING',
    blocks: 0,
    total_burned: 0,
    reward_pool: 0,
    circulating_supply: 0,
    total_supply: 100000000,
    peers: 0,
  },
  blocks: [],
  transactions: [],
  user_profile: {
    username: '@operator',
    nfmAddress: 'nfm_unknown',
    balance: 0,
    reputation: 0,
    joinedAt: Date.now(),
    feedbackCount: 0,
    settings: {
      rpc: 'http://127.0.0.1:3000',
      theme: 'mesh',
      notifications: {
        rewards: true,
        network: true,
        security: true,
      },
    },
  },
  wallets: [],
  node_stats: {
    uptime: '0m',
    cpu: 0,
    memory: '0 GB / 8 GB',
    bandwidth: '0 rec/s',
  },
  ai_tasks: [],
  drive_files: [],
  kg_concepts: [],
  market_items: [],
  quests: [],
  box_history: [],
  reward_catalog: [],
  mystery_news: [],
  proposals: [],
  api_docs: [],
};

interface AppDataContextValue {
  data: AppState;
  p2p: P2PStatus;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  refreshPaused: boolean;
  setRefreshPaused: (paused: boolean) => void;
  toast: AppToast | null;
  notifyToast: (toast: AppToast) => void;
  notifySuccess: (message: string) => void;
  notifyError: (message: string) => void;
  clearToast: () => void;
  requestPrompt: (options: PromptModalOptions) => Promise<string | null>;
  requestConfirm: (options: ConfirmModalOptions) => Promise<boolean>;
  updateSettings: (settings: Partial<NonNullable<UserProfile['settings']>>) => Promise<void>;
}

const AppDataContext = createContext<AppDataContextValue | null>(null);

const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL as string | undefined)?.trim() || 'http://127.0.0.1:3000';
const EMPTY_P2P: P2PStatus = {
  gossip_enabled: false,
  listening_port: 0,
  peer_count: 0,
  known_peers: [],
  seed_count: 0,
  last_sync_unix: 0,
  chain_blocks: 0,
  status: 'inactive',
};

export const AppDataProvider = ({ children }: { children: React.ReactNode }) => {
  const [data, setData] = useState<AppState>(EMPTY_STATE);
  const [p2p, setP2p] = useState<P2PStatus>(EMPTY_P2P);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshPaused, setRefreshPaused] = useState<boolean>(() => localStorage.getItem('nfm.app.refreshPaused') === 'true');
  const [toast, setToast] = useState<AppToast | null>(null);
  const [modalState, setModalState] = useState<ModalState | null>(null);
  const [modalInputValue, setModalInputValue] = useState('');

  const notifyToast = (nextToast: AppToast) => {
    setToast(nextToast);
  };

  const notifySuccess = (message: string) => {
    setToast({ type: 'success', message });
  };

  const notifyError = (message: string) => {
    setToast({ type: 'error', message });
  };

  const clearToast = () => {
    setToast(null);
  };

  const requestPrompt = (options: PromptModalOptions): Promise<string | null> => new Promise((resolve) => {
    setModalInputValue(options.defaultValue ?? '');
    setModalState({
      kind: 'prompt',
      options,
      resolve,
    });
  });

  const requestConfirm = (options: ConfirmModalOptions): Promise<boolean> => new Promise((resolve) => {
    setModalState({
      kind: 'confirm',
      options,
      resolve,
    });
  });

  const closeModalAsCancelled = () => {
    if (!modalState) {
      return;
    }
    if (modalState.kind === 'confirm') {
      modalState.resolve(false);
    } else {
      modalState.resolve(null);
    }
    setModalState(null);
  };

  const submitModal = () => {
    if (!modalState) {
      return;
    }
    if (modalState.kind === 'confirm') {
      modalState.resolve(true);
    } else {
      modalState.resolve(modalInputValue.trim());
    }
    setModalState(null);
  };

  const updateSettings = async (next: Partial<NonNullable<UserProfile['settings']>>) => {
    const currentSettings = data.user_profile.settings || EMPTY_STATE.user_profile.settings!;
    const newSettings: Required<Exclude<UserProfile['settings'], undefined>> = {
      rpc: next.rpc ?? currentSettings.rpc,
      theme: next.theme ?? (currentSettings.theme as any),
      notifications: {
        ...currentSettings.notifications,
        ...(next.notifications || {}),
      },
    };

    // Optimistic UI update
    setData((prev) => ({
      ...prev,
      user_profile: {
        ...prev.user_profile,
        settings: newSettings,
      },
    }));

    try {
      if (next.rpc) {
        localStorage.setItem('nfm.settings.rpc', next.rpc);
      }
      if (next.theme) {
        localStorage.setItem('nfm.settings.theme', next.theme);
      }
      
      await appUpdateSettings(newSettings as any);
      notifySuccess('Settings synchronized');
    } catch (err) {
      console.error('Failed to sync settings:', err);
      // We keep the optimistic update locally but notify the user
      notifyError('Failed to sync settings with node');
    }
  };

  const refresh = async () => {
    try {
      const token = (import.meta.env.VITE_BRAIN_BEARER_TOKEN as string | undefined)?.trim();
      const appReq = fetch(`${API_BASE_URL}/api/app/state`, {
        headers: token ? { Authorization: `Bearer ${token}` } : undefined,
      });

      const p2pReq = fetch(`${API_BASE_URL}/api/p2p/status`);

      const [appRes, p2pRes] = await Promise.all([appReq, p2pReq]);

      if (!appRes.ok) {
        throw new Error(`HTTP ${appRes.status}`);
      }

      const payload = (await appRes.json()) as AppState;
      setData(payload);

      if (p2pRes.ok) {
        const p2pPayload = (await p2pRes.json()) as P2PStatus;
        setP2p(p2pPayload);
      } else {
        setP2p(EMPTY_P2P);
      }

      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load app state');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
    const timer = window.setInterval(() => {
      if (!refreshPaused) {
        refresh();
      }
    }, 5000);
    return () => window.clearInterval(timer);
  }, [refreshPaused]);

  useEffect(() => {
    localStorage.setItem('nfm.app.refreshPaused', refreshPaused ? 'true' : 'false');
  }, [refreshPaused]);

  useEffect(() => {
    if (!toast) {
      return;
    }

    const timer = window.setTimeout(() => setToast(null), 2200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    if (!modalState) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        closeModalAsCancelled();
      }
      if (event.key === 'Enter') {
        event.preventDefault();
        submitModal();
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [modalState, modalInputValue]);

  const value = useMemo(
    () => ({
      data,
      p2p,
      loading,
      error,
      refresh,
      refreshPaused,
      setRefreshPaused,
      toast,
      notifyToast,
      notifySuccess,
      notifyError,
      clearToast,
      requestPrompt,
      requestConfirm,
      updateSettings,
    }),
    [data, p2p, loading, error, refreshPaused, toast],
  );

  return (
    <>
      <AppDataContext.Provider value={value}>{children}</AppDataContext.Provider>
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
      {modalState && (
        <div className="nfm-modal-overlay" onClick={closeModalAsCancelled}>
          <div className="nfm-modal animate-in" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '560px' }}>
            <div className="nfm-modal__header">
              <h3 className="nfm-modal__title">{modalState.options.title}</h3>
              <button className="nfm-modal-close" onClick={closeModalAsCancelled} aria-label="Close modal">
                ✕
              </button>
            </div>
            <p className="text-muted" style={{ marginBottom: 'var(--space-6)' }}>{modalState.options.message}</p>

            {modalState.kind === 'prompt' && (
              <input
                className="nfm-input"
                autoFocus
                placeholder={modalState.options.placeholder}
                value={modalInputValue}
                onChange={(e) => setModalInputValue(e.target.value)}
                style={{ marginBottom: 'var(--space-6)' }}
              />
            )}

            <div className="flex gap-3" style={{ justifyContent: 'flex-end' }}>
              <button className="nfm-btn nfm-btn--ghost" onClick={closeModalAsCancelled}>
                {modalState.options.cancelText ?? 'Cancel'}
              </button>
              <button className="nfm-btn nfm-btn--primary" onClick={submitModal}>
                {modalState.options.confirmText ?? 'Confirm'}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export const useAppData = () => {
  const ctx = useContext(AppDataContext);
  if (!ctx) {
    throw new Error('useAppData must be used inside AppDataProvider');
  }
  return ctx;
};
