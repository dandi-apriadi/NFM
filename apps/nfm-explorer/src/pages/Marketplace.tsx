import { useEffect, useMemo, useState } from 'react';
import { ShoppingCart, Search, Filter, Star, TrendingUp, ArrowRight } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';

const MARKET_SEARCH_KEY = 'nfm.marketplace.searchQuery';
type MarketFilter = 'ALL' | 'AI_SKILL' | 'NODE_LICENSE' | 'DATASET';
const MARKET_FILTER_ORDER: MarketFilter[] = ['ALL', 'AI_SKILL', 'NODE_LICENSE', 'DATASET'];

const Marketplace = () => {
  const navigate = useNavigate();
  const { data } = useAppData();
  const DUMMY_MARKET_ITEMS = data.market_items;
  const [searchQuery, setSearchQuery] = useState(() => sessionStorage.getItem(MARKET_SEARCH_KEY) || '');
  const [activeFilter, setActiveFilter] = useState<MarketFilter>('ALL');

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

      return (
        item.name.toLowerCase().includes(q) ||
        item.creator.toLowerCase().includes(q) ||
        item.type.toLowerCase().includes(q)
      );
    });
  }, [DUMMY_MARKET_ITEMS, searchQuery, activeFilter]);

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
        <h1 className="text-cyan flex items-center gap-2"><ShoppingCart /> NFM Marketplace</h1>
        <div className="nfm-badge nfm-badge--pink">
          <div className="nfm-badge__dot"></div> {filteredItems.length.toLocaleString()} Active Listings
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
        {filteredItems.map(item => (
          <div key={item.id} className="nfm-glass-card nfm-glass-card--interactive" onClick={() => navigate(`/market/${item.id}`)} style={{ display: 'flex', flexDirection: 'column' }}>
            <div className={`nfm-badge nfm-badge--${item.type === 'AI_SKILL' ? 'cyan' : item.type === 'NODE_LICENSE' ? 'purple' : 'pink'} mb-4`} style={{ alignSelf: 'flex-start' }}>
              {item.type.replace('_', ' ')}
            </div>
            
            <h4 className="text-md text-primary mb-1">{item.name}</h4>
            <div className="text-xs text-muted mb-4">{item.creator}</div>
            
            <div className="mt-auto flex justify-between items-end border-t pt-3" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>
              <div>
                <div className="text-xs text-secondary flex items-center gap-1 mb-1">
                  <Star size={12} className="text-gold" /> {item.rating} ({item.sales} sold)
                </div>
                <div className="font-mono text-cyan">{item.price} NVC</div>
              </div>
              <button
                className="nfm-btn nfm-btn--ghost nfm-btn--sm"
                onClick={(e) => {
                  e.stopPropagation();
                  navigate(`/market/${item.id}`);
                }}
              >
                View
              </button>
            </div>
          </div>
        ))}
      </div>
      
      <button className="nfm-btn-more" onClick={clearMarketplaceFilters}>
        <ArrowRight size={14} /> Reset Listing Filters
      </button>
    </div>
  );
};

export default Marketplace;
