const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL as string | undefined)?.trim() || 'http://127.0.0.1:3000';

type Json = Record<string, unknown>;

export interface ApiError extends Error {
  code: string;
  status: number;
  details?: unknown;
}

function createApiError(status: number, payload: unknown): ApiError {
  const body = (typeof payload === 'object' && payload !== null ? payload : {}) as Record<string, unknown>;
  const message = typeof body.error === 'string'
    ? body.error
    : typeof body.message === 'string'
      ? body.message
      : `HTTP ${status}`;

  const err = new Error(message) as ApiError;
  err.name = 'ApiError';
  err.status = status;
  err.code = typeof body.code === 'string' ? body.code : `HTTP_${status}`;
  err.details = body;
  return err;
}

async function request(path: string, method: 'GET' | 'POST', body?: Json) {
  const res = await fetch(`${API_BASE_URL}${path}`, {
    method,
    headers: {
      'Content-Type': 'application/json',
    },
    body: body ? JSON.stringify(body) : undefined,
  });

  const payload = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw createApiError(res.status, payload);
  }

  return payload;
}

export async function appTransfer(to: string, amount: number, from?: string) {
  return request('/api/app/wallet/transfer', 'POST', { to, amount, from });
}

export async function appCreateWallet() {
  return request('/api/app/wallet/create', 'POST');
}

export async function appCreateProposal(title: string, description: string, address?: string) {
  return request('/api/app/governance/proposal', 'POST', { title, description, address });
}

export async function appVoteProposal(proposalId: string, approve: boolean, address?: string) {
  return request('/api/app/governance/vote', 'POST', {
    proposal_id: proposalId,
    approve,
    address,
  });
}

export async function appClaimQuest(questId: string, address?: string) {
  return request('/api/app/quest/claim', 'POST', {
    quest_id: questId,
    address,
  });
}

export async function appExtractMystery(address?: string) {
  return request('/api/app/mystery/extract', 'POST', { address });
}

export async function appPurchaseMarketItem(itemId: string, price: number, address?: string) {
  return request('/api/app/market/purchase', 'POST', {
    item_id: itemId,
    price,
    address,
  });
}

export async function appDriveUpload(name: string, content: string, address?: string, type: string = 'TEXT') {
  return request('/api/drive/upload', 'POST', {
    name,
    content,
    address,
    type,
    fragments: 1,
  });
}

export async function appDriveFiles() {
  return request('/api/drive/files', 'GET');
}

export async function appDriveDownload(fileId: string, address?: string) {
  return request('/api/drive/download', 'POST', {
    file_id: fileId,
    address,
  });
}

export async function appUpdateSettings(settings: Json) {
  return request('/api/app/settings', 'POST', { settings });
}

export async function p2pSetSeeds(seeds: string[]) {
  return request('/api/p2p/seeds', 'POST', { seeds });
}

export async function p2pBootstrap() {
  return request('/api/p2p/bootstrap', 'POST');
}

export async function p2pSync() {
  return request('/api/p2p/sync', 'POST');
}

export async function p2pBanlist() {
  return request('/api/p2p/banlist', 'GET');
}

export async function p2pBan(endpoint: string) {
  return request('/api/p2p/ban', 'POST', { endpoint });
}

export async function p2pUnban(endpoint: string) {
  return request('/api/p2p/unban', 'POST', { endpoint });
}

export async function p2pBulkBan(endpoints: string[]) {
  return request('/api/p2p/ban/bulk', 'POST', { endpoints });
}

export async function p2pBulkUnban(endpoints: string[]) {
  return request('/api/p2p/unban/bulk', 'POST', { endpoints });
}

export async function appAuctionList() {
  return request('/api/auction/list', 'GET');
}

export async function appAuctionCreate(seller: string, name: string, rarity: string, power: number, startPrice: number, durationHours: number) {
  return request('/api/auction/create', 'POST', {
    seller,
    item_name: name,
    item_rarity: rarity,
    item_power_multiplier: power,
    starting_price: startPrice,
    duration_hours: durationHours,
  });
}

export async function appAuctionBid(auctionId: number, bidder: string, bidAmount: number) {
  return request('/api/auction/bid', 'POST', {
    auction_id: auctionId,
    bidder,
    bid_amount: bidAmount,
  });
}

export async function appAuctionSettle(auctionId: number) {
  return request('/api/auction/settle', 'POST', {
    auction_id: auctionId,
  });
}

export async function appAuctionCancel(auctionId: number, requester: string) {
  return request('/api/auction/cancel', 'POST', {
    auction_id: auctionId,
    requester,
  });
}

export async function appGetIdentity(address: string) {
  return request(`/api/identity/${address}`, 'GET');
}

export async function appNlcPreview(input: string, address?: string) {
  return request('/api/nlc/preview', 'POST', {
    input,
    address,
  });
}

export async function appBrainCurriculumPropose(payload: {
  address?: string;
  epoch?: number;
  start_block?: number;
  end_block?: number;
  model_version?: string;
  intent?: string;
  requires_quorum?: boolean;
}) {
  return request('/api/brain/curriculum/propose', 'POST', payload as Json);
}

export async function appBrainCurriculumActive() {
  return request('/api/brain/curriculum/active', 'GET');
}

export async function appBrainCurriculumVote(voteId: number, approve: boolean, address?: string, executeNow: boolean = true) {
  return request('/api/brain/curriculum/vote', 'POST', {
    vote_id: voteId,
    approve,
    address,
    execute_now: executeNow,
  });
}

export async function appBrainReputationLeaderboard() {
  return request('/api/brain/reputation/leaderboard', 'GET');
}

export async function appGovernanceIndicators() {
  return request('/api/governance/indicators', 'GET');
}

export async function appKgSemantic() {
  return request('/api/kg/semantic', 'GET');
}

// ========== DEVELOPMENT-ONLY: BLOCKCHAIN RESET ==========
// This function resets the entire blockchain to genesis state.
// Will be removed before production deployment.
export async function appAdminResetBlockchain(adminSecret: string) {
  return request('/api/admin/reset', 'POST', { secret: adminSecret });
}
