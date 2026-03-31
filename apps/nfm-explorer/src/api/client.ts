const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL as string | undefined)?.trim() || 'http://127.0.0.1:3000';

type Json = Record<string, unknown>;

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
    const errorMessage = typeof payload?.error === 'string' ? payload.error : `HTTP ${res.status}`;
    throw new Error(errorMessage);
  }

  return payload;
}

export async function appTransfer(to: string, amount: number, from?: string) {
  return request('/api/app/wallet/transfer', 'POST', { to, amount, from });
}

export async function appCreateProposal(title: string, description: string, proposer?: string) {
  return request('/api/app/governance/proposal', 'POST', { title, description, proposer });
}

export async function appVoteProposal(proposalId: string, approve: boolean, voter?: string) {
  return request('/api/app/governance/vote', 'POST', {
    proposal_id: proposalId,
    approve,
    voter,
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
