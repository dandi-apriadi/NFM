import { createContext, useContext, useEffect, useMemo, useState } from 'react';
import type { Block, NFMStatus, NodeStats, Transaction, UserProfile } from '../types';

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
  name: string;
  creator: string;
  price: number;
  type: 'AI_SKILL' | 'FRAGMENT' | 'NODE_LICENSE';
  sales: number;
  rating: number;
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

const EMPTY_STATE: AppState = {
  status: {
    node: 'nfm-node',
    version: 'NFM Vault v1.2',
    status: 'SYNCING',
    blocks: 0,
    total_burned: 0,
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
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

const AppDataContext = createContext<AppDataContextValue | null>(null);

const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL as string | undefined)?.trim() || 'http://127.0.0.1:3000';

export const AppDataProvider = ({ children }: { children: React.ReactNode }) => {
  const [data, setData] = useState<AppState>(EMPTY_STATE);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    try {
      const token = (import.meta.env.VITE_BRAIN_BEARER_TOKEN as string | undefined)?.trim();
      const res = await fetch(`${API_BASE_URL}/api/app/state`, {
        headers: token ? { Authorization: `Bearer ${token}` } : undefined,
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      const payload = (await res.json()) as AppState;
      setData(payload);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load app state');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
    const timer = window.setInterval(refresh, 5000);
    return () => window.clearInterval(timer);
  }, []);

  const value = useMemo(
    () => ({ data, loading, error, refresh }),
    [data, loading, error],
  );

  return <AppDataContext.Provider value={value}>{children}</AppDataContext.Provider>;
};

export const useAppData = () => {
  const ctx = useContext(AppDataContext);
  if (!ctx) {
    throw new Error('useAppData must be used inside AppDataProvider');
  }
  return ctx;
};
