import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, CheckCircle2, ShoppingCart, User, Activity } from 'lucide-react';
import { useAppData } from '../context/AppDataContext';
import { appPurchaseMarketItem } from '../api/client';

const AssetDetail = () => {
  const { id } = useParams();
  const navigate = useNavigate();
  const { data, refresh, requestPrompt, notifySuccess, notifyError } = useAppData();
  const DUMMY_MARKET_ITEMS = data.market_items;

  const asset = DUMMY_MARKET_ITEMS.find(item => item.id === id);

  const handlePurchase = async () => {
    if (!asset) {
      notifyError('Asset not found in backend listing');
      return;
    }
    try {
      await appPurchaseMarketItem(asset.id, asset.price, data.user_profile.nfmAddress);
      await refresh();
      notifySuccess('Purchase successful');
    } catch (e) {
      notifyError(e instanceof Error ? e.message : 'Purchase failed');
    }
  };

  const handleOffer = async () => {
    if (!asset) {
      notifyError('Asset not found in backend listing');
      return;
    }

    const input = await requestPrompt({
      title: 'Submit Offer',
      message: `Enter your offer for ${asset.name} (NVC)`,
      placeholder: String(asset.price),
      confirmText: 'Submit Offer',
    });
    if (input === null) {
      return;
    }

    const offer = Number(input);
    if (!Number.isFinite(offer) || offer <= 0) {
      notifyError('Invalid offer amount');
      return;
    }

    notifySuccess(`Offer of ${offer.toLocaleString('en-US')} NVC queued for manual review`);
  };

  return (
    <div className="animate-in">
      <button className="nfm-btn nfm-btn--ghost mb-6" onClick={() => navigate(-1)}>
        <ArrowLeft size={18} /> Back to Market
      </button>

      {!asset && (
        <div className="nfm-glass-card" style={{ marginBottom: 'var(--space-6)' }}>
          <h2 className="text-primary" style={{ marginBottom: 'var(--space-2)' }}>Asset Not Found</h2>
          <p className="text-muted text-sm" style={{ marginBottom: 'var(--space-4)' }}>
            Item `{id}` tidak ada di listing backend saat ini.
          </p>
          <button className="nfm-btn nfm-btn--primary" onClick={() => navigate('/market')}>
            Kembali ke Marketplace
          </button>
        </div>
      )}

      {asset && <div className="flex gap-8 wrap" style={{ flexWrap: 'wrap' }}>
        {/* Visual / Metadata Area */}
        <div style={{ flex: '1 1 350px' }}>
          <div className="nfm-glass-card nfm-glass-card--glow-cyan mb-6" style={{ height: '350px', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'radial-gradient(circle at center, rgba(0,245,255,0.15), transparent)' }}>
             <Activity size={100} className="text-cyan opacity-80 animate-pulse" />
          </div>

          <div className="nfm-glass-card">
             <h3 className="text-lg text-primary mb-4 border-b pb-2" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>Asset Integrity Check</h3>
             <ul className="flex-col gap-3 text-sm">
               <li className="flex gap-2 items-center text-secondary"><CheckCircle2 size={16} className="text-success" /> Code Signature Verified</li>
               <li className="flex gap-2 items-center text-secondary"><CheckCircle2 size={16} className="text-success" /> Smart Contract Audited</li>
               <li className="flex gap-2 items-center text-secondary"><CheckCircle2 size={16} className="text-success" /> Dependency Tree Safe</li>
             </ul>
          </div>
        </div>

        {/* Purchase Area */}
        <div style={{ flex: '2 1 500px' }}>
          <div className="nfm-glass-card h-full">
            <div className={`nfm-badge nfm-badge--${asset.type === 'AI_SKILL' ? 'cyan' : asset.type === 'NODE_LICENSE' ? 'purple' : 'pink'} mb-4`}>
              {asset.type.replace('_', ' ')}
            </div>

            <h1 className="text-3xl text-primary mb-2 font-display">{asset.name}</h1>
            
            <div className="flex items-center gap-4 mb-6">
               <span className="flex items-center gap-2 text-muted text-sm cursor-pointer hover:text-cyan transition-colors">
                 <User size={16} /> {asset.creator}
               </span>
               <span className="text-gold text-sm font-bold">★ {asset.rating}</span>
               <span className="text-muted text-sm">{asset.sales} Sold</span>
            </div>

            <p className="text-secondary leading-relaxed mb-8">
              This asset is fully encrypted and stored on the NFM distributed drive. 
              Upon purchase, smart contracts will seamlessly transfer the decryption keys to your wallet. 
              You can integrate this directly into your local AI Brain or Node Runner.
            </p>

            <div className="p-6 mb-8" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(0,245,255,0.15)' }}>
              <div className="text-sm uppercase text-muted tracking-widest mb-2 font-bold">Acquisition Cost</div>
              <div className="font-display text-4xl text-cyan mb-4">{asset.price.toLocaleString('en-US')} NVC</div>
              
              <div className="flex gap-4">
                  <button className="nfm-btn nfm-btn--primary nfm-btn--lg" style={{ flex: 1 }} onClick={handlePurchase}>
                    <ShoppingCart size={20} /> Purchase Now
                 </button>
                    <button className="nfm-btn nfm-btn--secondary nfm-btn--lg" onClick={() => void handleOffer()}>
                    Offer
                 </button>
              </div>
            </div>
            
            <div className="text-xs text-muted">
               <p className="mb-1"><strong>Note:</strong> 60% of logical volume goes to {asset.creator}, 10% is burned, and 30% enters the Protocol Growth Fund.</p>
               <p>Purchase requires Bio-ZKP authentication via linked mobile app.</p>
            </div>
          </div>
        </div>

      </div>}
    </div>
  );
};

export default AssetDetail;
