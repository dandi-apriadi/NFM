import { Fingerprint, Smartphone, ExternalLink } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

const QRLogin = () => {
  const navigate = useNavigate();

  return (
    <div className="flex items-center justify-center min-h-screen bg-black" style={{ backgroundImage: 'radial-gradient(circle at center, rgba(0,245,255,0.05) 0%, transparent 60%)' }}>
      
      <div className="nfm-glass-card p-12 max-w-lg w-full text-center" style={{ border: '1px solid rgba(0,245,255,0.2)', boxShadow: '0 0 50px rgba(0,245,255,0.05)' }}>
        
        <div className="mb-8">
          <Fingerprint size={64} className="text-cyan mx-auto mb-4 animate-pulse opacity-80" style={{ filter: 'drop-shadow(0 0 10px rgba(0,245,255,0.5))' }} />
          <h1 className="text-3xl font-display text-primary tracking-wide mb-2">NFM Identity</h1>
          <p className="text-secondary text-sm">Bio-ZKP Authentication Portal</p>
        </div>

        <div className="p-8 mb-8" style={{ background: '#fff', borderRadius: 'var(--radius-md)', display: 'inline-block' }}>
           {/* Mock QR Code */}
           <div style={{ width: '200px', height: '200px', background: `repeating-linear-gradient(45deg, #000 25%, transparent 25%, transparent 75%, #000 75%, #000), repeating-linear-gradient(45deg, #000 25%, #fff 25%, #fff 75%, #000 75%, #000)`, backgroundPosition: '0 0, 10px 10px', backgroundSize: '20px 20px', padding: '10px' }}>
              <div style={{ width: '100%', height: '100%', border: '10px solid #000', boxSizing: 'border-box', position: 'relative' }}>
                 <div style={{position: 'absolute', top: 0, left: 0, width: '40px', height: '40px', background: '#000'}}></div>
                 <div style={{position: 'absolute', top: 0, right: 0, width: '40px', height: '40px', background: '#000'}}></div>
                 <div style={{position: 'absolute', bottom: 0, left: 0, width: '40px', height: '40px', background: '#000'}}></div>
                 <div style={{position: 'absolute', bottom: '40px', right: '40px', width: '20px', height: '20px', background: '#000'}}></div>
              </div>
           </div>
        </div>

        <div className="flex-col gap-4 text-sm text-muted">
           <p className="flex items-center justify-center gap-2">
             <Smartphone size={16} /> Scan with your NFM Mobile App
           </p>
           <p className="flex items-center justify-center gap-2 cursor-pointer hover:text-cyan transition-colors" onClick={() => navigate('/')}>
               Bypass (Dev Environment) <ExternalLink size={14} />
           </p>
        </div>

      </div>
    </div>
  );
};

export default QRLogin;
