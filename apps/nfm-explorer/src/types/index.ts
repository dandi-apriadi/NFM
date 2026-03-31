// types/index.ts

export interface NFMStatus {
  node: string;
  version: string;
  status: 'ONLINE' | 'SYNCING' | 'OFFLINE';
  blocks: number;
  total_burned: number;
  peers: number;
}

export interface Block {
  index: number;
  hash: string;
  previous_hash: string;
  timestamp: number;
  transactions: number;
  size: string;
  miner: string;
  rewards: number;
}

export interface Transaction {
  txid: string;
  type: 'TRANSFER' | 'SMART_CONTRACT' | 'NODE_REWARD' | 'BURN';
  from: string;
  to: string;
  amount: number;
  timestamp: number;
  fee: number;
  status: 'CONFIRMED' | 'PENDING' | 'FAILED';
}

export interface UserProfile {
  username: string;
  nfmAddress: string;
  balance: number;
  reputation?: number;
  joinedAt: number;
  feedbackCount: number;
  avatarUrl?: string;
  seedPhrase?: string;
  settings?: {
    rpc: string;
    theme: 'dark' | 'light' | 'mesh';
    notifications: {
      rewards: boolean;
      network: boolean;
      security: boolean;
    };
  };
}

export interface NodeStats {
  uptime: string;
  cpu: number;
  memory: string;
  bandwidth: string;
}

export interface P2PStatus {
  gossip_enabled: boolean;
  listening_port: number;
  peer_count: number;
  healthy_peers?: number;
  unhealthy_peers?: number;
  known_peers: string[];
  peer_health?: Array<{
    endpoint: string;
    healthy: boolean;
    latency_ms: number;
    score?: number;
    quality?: 'excellent' | 'good' | 'degraded' | 'poor' | 'critical' | string;
    error?: string | null;
  }>;
  seed_count: number;
  ban_count?: number;
  banned_peers?: string[];
  reconnect_attempts?: number;
  reconnect_backoff_secs?: number;
  next_reconnect_unix?: number;
  last_reconnect_unix?: number;
  last_sync_unix: number;
  chain_blocks: number;
  status: string;
}
