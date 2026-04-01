import { useEffect, useMemo, useState } from 'react';
import { ShoppingCart, Search, Filter, Star, TrendingUp, ArrowRight, X, Gavel, Shield } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';
import { appAuctionBid, appAuctionCreate, appAuctionSettle, appAuctionCancel, appGetIdentity } from '../api/client';
import type { UserIdentity } from '../types';

const MARKET_SEARCH_KEY = 'nfm.marketplace.searchQuery';
type MarketFilter = 'ALL' | 'AI_SKILL' | 'NODE_LICENSE' | 'DATASET';
const MARKET_FILTER_ORDER: MarketFilter[] = ['ALL', 'AI_SKILL', 'NODE_LICENSE', 'DATASET'];

const Marketplace = () => {
  const navigate = useNavigate();
  const { data, notifyError, notifySuccess } = useAppData();
  const DUMMY_MARKET_ITEMS = data.market_items;
  const [searchQuery, setSearchQuery] = useState(() => sessionStorage.getItem(MARKET_SEARCH_KEY) || '');
  const [activeFilter, setActiveFilter] = useState<MarketFilter>('ALL');
  const [showBidModal, setShowBidModal] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedAuction, setSelectedAuction] = useState<any>(null);
  const [bidAmount, setBidAmount] = useState('');
  const [createForm, setCreateForm] = useState({ name: '', rarity: 'COMMON', power: 1, startPrice: 100, hours: 24 });
  const [userIdentity, setUserIdentity] = useState<UserIdentity | null>(null);

  useEffect(() => {
    const loadIdentity = async () => {
      try {
        const identity = await appGetIdentity(data.user_profile.nfmAddress);
        setUserIdentity(identity as UserIdentity);
      } catch (e) {
        // Identity not found is ok - just means not an elite user yet
      }
    };
    loadIdentity();
  }, [data.user_profile.nfmAddress]);

  useEffect(() => {
    sessionStorage.setItem(MARKET_SEARCH_KEY, searchQuery);
  }, [searchQuery]);

  const filteredItems = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    return DUMMY_MARKET_ITEMS.filter((item) => {
      const matchesFilter = activeFilter === 'ALL' || item.type === activeFilter;
      if (!matchesFilter) {
        return false;
      }

      if (!q) {
        return true;
      }

      const name = item.title || item.name || '';
      const creator = item.seller || item.creator || '';
      return (
        name.toLowerCase().includes(q) ||
        creator.toLowerCase().includes(q)
      );
    });
  }, [DUMMY_MARKET_ITEMS, searchQuery, activeFilter]);

  const handleBid = async () => {
    if (!selectedAuction || !bidAmount) {
      notifyError('Please enter a bid amount');
      return;
    }
    try {
      await appAuctionBid(selectedAuction.auction_id, data.user_profile.nfmAddress, parseFloat(bidAmount));
      notifySuccess(`Bid placed for ${bidAmount} NVC`);
      setShowBidModal(false);
      setBidAmount('');
    } catch (e: any) {
      notifyError(e.message || 'Bid failed');
    }
  };

  const handleCreateAuction = async () => {
    if (!createForm.name) {
      notifyError('Please enter item name');
      return;
    }
    try {
      await appAuctionCreate(
        data.user_profile.nfmAddress,
        createForm.name,
        createForm.rarity,
        createForm.power,
        createForm.startPrice,
        createForm.hours
      );
      notifySuccess(`Auction created for "${createForm.name}"`);
      setShowCreateModal(false);
      setCreateForm({ name: '', rarity: 'COMMON', power: 1, startPrice: 100, hours: 24 });
    } catch (e: any) {
      notifyError(e.message || 'Auction creation failed');
    }
  };

  const handleSettle = async (auctionId: number) => {
    try {
      await appAuctionSettle(auctionId);
      notifySuccess('Auction settled successfully');
    } catch (e: any) {
      notifyError(e.message || 'Settlement failed');
    }
  };

  const handleCancel = async (auctionId: number) => {
    try {
      await appAuctionCancel(auctionId, data.user_profile.nfmAddress);
      notifySuccess('Auction cancelled successfully');
    } catch (e: any) {
      notifyError(e.message || 'Cancellation failed');
    }
  };

  const cycleFilter = () => {
    setActiveFilter((prev) => {
      const currentIdx = MARKET_FILTER_ORDER.indexOf(prev);
      return MARKET_FILTER_ORDER[(currentIdx + 1) % MARKET_FILTER_ORDER.length];
    });
  };

  const clearMarketplaceFilters = () => {
    setSearchQuery('');
    setActiveFilter('ALL');
  };

  return (
    <div className="animate-in">
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <div className="flex items-center gap-4">
          <h1 className="text-cyan flex items-center gap-2"><ShoppingCart /> NFM Marketplace</h1>
          {userIdentity?.elite_shield && (
            <div className="flex items-center gap-2 px-3 py-1 rounded-lg bg-gradient-to-r from-purple-900 to-pink-900 border border-purple-500">
              <Shield size={16} className="text-purple-300" />
              <span className="text-sm font-mono text-purple-200">ELITE VERIFIED</span>
            </div>
          )}
        </div>
        <div className="flex gap-3">
          {userIdentity?.elite_items && userIdentity.elite_items.length > 0 && (
            <div className="nfm-badge nfm-badge--purple" title={`Elite Items: ${userIdentity.elite_items.join(', ')}`}>
              🏅 {userIdentity.elite_items.length} Elite
            </div>
          )}
          <div className="nfm-badge nfm-badge--pink">
            <div className="nfm-badge__dot"></div> {filteredItems.length.toLocaleString()} Active Listings
          </div>
        </div>
      </div>

      {/* Top Controls */}
      <div className="flex gap-4 mb-6">
        <div className="nfm-search" style={{ flex: 1 }}>
          <Search className="nfm-search__icon" size={18} />
          <input 
            type="text" 
            className="nfm-search__input" 
            placeholder="Search AI Skills, Node Licenses, or Datasets..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <button className="nfm-btn nfm-btn--ghost" onClick={cycleFilter}>
          <Filter size={18} /> Filter: {activeFilter.replace('_', ' ')}
        </button>
        <button className="nfm-btn nfm-btn--primary" onClick={() => setShowCreateModal(true)}>
          <Gavel size={18} /> Create Auction
        </button>
      </div>

      {/* Featured Banner */}
      <div className="nfm-glass-card nfm-glass-card--glow-cyan mb-8" style={{ borderLeft: '4px solid var(--neon-cyan)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <div className="nfm-badge nfm-badge--cyan mb-2">Featured Item</div>
          <h2 className="text-2xl text-primary mb-1">Genesis Node Architect (AI Skill)</h2>
          <p className="text-muted mb-4 max-w-2xl text-sm">Automate the deployment and topologizing of your decentralized infrastructure with this premium AI skill trained on the core NFM architecture logs.</p>
          <div className="flex gap-4 items-center">
             <span className="font-display text-xl text-gold">1,250 NVC</span>
             <button className="nfm-btn nfm-btn--primary nfm-btn--sm" onClick={() => navigate('/market/m-featured')}>View Detail</button>
          </div>
        </div>
        <div className="hide-mobile" style={{ width: '150px', height: '150px', background: 'radial-gradient(circle, rgba(0,245,255,0.2) 0%, transparent 70%)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <TrendingUp size={48} className="text-cyan opacity-80" />
        </div>
      </div>

      {/* Grid */}
      <h3 className="text-lg text-primary mb-4 border-b pb-2" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>
        Trending Listings {activeFilter !== 'ALL' ? `(${activeFilter.replace('_', ' ')})` : ''}
      </h3>
      
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: 'var(--space-4)' }}>
        {filteredItems.map(item => {
          const isAuction = !!item.auction_id;
          const itemName = item.title || item.name || '';
          const itemCreator = item.seller || item.creator || '';
          const getRarityColor = (rarity: string) => {
            const colors: Record<string, string> = {
              'COMMON': 'pink',
              'RARE': 'cyan',
              'EPIC': 'purple',
              'LEGENDARY': 'gold',
              'MYTHIC': 'red'
            };
            return colors[rarity] || 'pink';
          };
          
          return (
            <div key={item.id} className="nfm-glass-card nfm-glass-card--interactive" style={{ display: 'flex', flexDirection: 'column' }}>
              <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
                {isAuction ? (
                  <div className={`nfm-badge nfm-badge--${getRarityColor(item.rarity || 'COMMON')}`} style={{ alignSelf: 'flex-start' }}>
                    {item.rarity || 'COMMON'}
                  </div>
                ) : (
                  <div className={`nfm-badge nfm-badge--${item.type === 'AI_SKILL' ? 'cyan' : item.type === 'NODE_LICENSE' ? 'purple' : 'pink'}`} style={{ alignSelf: 'flex-start' }}>
                    {item.type?.replace('_', ' ')}
                  </div>
                )}
                {isAuction && (
                  <div className={`nfm-badge nfm-badge--${item.status === 'ACTIVE' ? 'green' : item.status === 'SOLD' ? 'blue' : 'gray'}`} style={{ alignSelf: 'flex-start' }}>
                    {item.status}
                  </div>
                )}
              </div>
              
              <h4 className="text-md text-primary mb-1">{itemName}</h4>
              <div className="text-xs text-muted mb-4">{itemCreator}</div>
              
              {isAuction && (
                <div className="text-xs text-secondary mb-3 space-y-1">
                  <div>Starting: <span className="text-cyan">{item.starting_price} NVC</span></div>
                  {(item.highest_bid ?? 0) > 0 && <div>Current: <span className="text-gold">{item.highest_bid} NVC</span></div>}
                  {item.highest_bidder && <div>Bidder: <span className="text-cyan">{item.highest_bidder}</span></div>}
                </div>
              )}
              
              <div className="mt-auto flex justify-between items-end border-t pt-3" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>
                <div>
                  {isAuction ? (
                    <div className="font-mono text-gold">{item.price} NVC</div>
                  ) : (
                    <>
                      <div className="text-xs text-secondary flex items-center gap-1 mb-1">
                        <Star size={12} className="text-gold" /> {item.rating} ({item.sales} sold)
                      </div>
                      <div className="font-mono text-cyan">{item.price} NVC</div>
                    </>
                  )}
                </div>
                <div className="flex gap-2">
                  {isAuction && item.status === 'ACTIVE' && (
                    <button
                      className="nfm-btn nfm-btn--primary nfm-btn--sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        setSelectedAuction(item);
                        setShowBidModal(true);
                      }}
                    >
                      Bid
                    </button>
                  )}
                  {isAuction && item.auction_id !== undefined && (
                    <button
                      className="nfm-btn nfm-btn--ghost nfm-btn--sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        if (item.status === 'ACTIVE') {
                          handleSettle(item.auction_id!);
                        } else if (item.seller === data.user_profile.nfmAddress) {
                          handleCancel(item.auction_id!);
                        }
                      }}
                    >
                      {item.status === 'ACTIVE' ? 'Settle' : item.seller === data.user_profile.nfmAddress ? 'Cancel' : 'View'}
                    </button>
                  )}
                  {!isAuction && (
                    <button
                      className="nfm-btn nfm-btn--ghost nfm-btn--sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        navigate(`/market/${item.id}`);
                      }}
                    >
                      View
                    </button>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>
      
      <button className="nfm-btn-more" onClick={clearMarketplaceFilters}>
        <ArrowRight size={14} /> Reset Listing Filters
      </button>

      {/* Bid Modal */}
      {showBidModal && selectedAuction && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="nfm-glass-card max-w-md w-full p-6">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg text-primary">Place Bid</h3>
              <button onClick={() => setShowBidModal(false)} className="text-muted hover:text-primary">
                <X size={20} />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <div className="text-sm text-muted mb-2">Item: {selectedAuction.title || selectedAuction.name}</div>
                <div className="text-sm text-muted mb-2">Current Bid: {(selectedAuction.highest_bid ?? selectedAuction.starting_price ?? 0)} NVC</div>
              </div>
              <input
                type="number"
                placeholder="Bid amount (NVC)"
                value={bidAmount}
                onChange={(e) => setBidAmount(e.target.value)}
                className="nfm-input w-full"
                min={(selectedAuction.highest_bid ?? selectedAuction.starting_price ?? 0) + 1}
              />
              <div className="flex gap-2">
                <button className="nfm-btn nfm-btn--primary flex-1" onClick={handleBid}>
                  Place Bid
                </button>
                <button className="nfm-btn nfm-btn--ghost flex-1" onClick={() => setShowBidModal(false)}>
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create Auction Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="nfm-glass-card max-w-md w-full p-6">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg text-primary">Create Auction</h3>
              <button onClick={() => setShowCreateModal(false)} className="text-muted hover:text-primary">
                <X size={20} />
              </button>
            </div>
            <div className="space-y-4">
              <input
                type="text"
                placeholder="Item Name"
                value={createForm.name}
                onChange={(e) => setCreateForm({ ...createForm, name: e.target.value })}
                className="nfm-input w-full"
              />
              <select
                value={createForm.rarity}
                onChange={(e) => setCreateForm({ ...createForm, rarity: e.target.value })}
                className="nfm-input w-full"
              >
                <option>COMMON</option>
                <option>RARE</option>
                <option>EPIC</option>
                <option>LEGENDARY</option>
                <option>MYTHIC</option>
              </select>
              <input
                type="number"
                placeholder="Power Multiplier"
                value={createForm.power}
                onChange={(e) => setCreateForm({ ...createForm, power: parseFloat(e.target.value) })}
                className="nfm-input w-full"
                step="0.1"
              />
              <input
                type="number"
                placeholder="Starting Price (NVC)"
                value={createForm.startPrice}
                onChange={(e) => setCreateForm({ ...createForm, startPrice: parseFloat(e.target.value) })}
                className="nfm-input w-full"
              />
              <input
                type="number"
                placeholder="Duration (hours)"
                value={createForm.hours}
                onChange={(e) => setCreateForm({ ...createForm, hours: parseInt(e.target.value) })}
                className="nfm-input w-full"
              />
              <div className="flex gap-2">
                <button className="nfm-btn nfm-btn--primary flex-1" onClick={handleCreateAuction}>
                  Create Auction
                </button>
                <button className="nfm-btn nfm-btn--ghost flex-1" onClick={() => setShowCreateModal(false)}>
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Marketplace;
