import { useState } from 'react';
import { Share2, FileText, Database, GitMerge, Search, LayoutTemplate, Activity, Network, Layers, ZoomIn, ZoomOut, Maximize2, ArrowRight, Box } from 'lucide-react';
import { useAppData } from '../context/AppDataContext';

const KnowledgeGraph = () => {
  const { data } = useAppData();
  const DUMMY_KG_CONCEPTS = data.kg_concepts;

  const [viewMode, setViewMode] = useState<'2D' | '3D'>('2D');
  const [zoom, setZoom] = useState(1);

  return (
    <div className="animate-in">
      {/* Header with Badges */}
      <div className="flex items-center justify-between" style={{ marginBottom: 'var(--space-8)' }}>
        <div>
          <h1 className="text-purple flex items-center gap-2"><Share2 /> Decentralized Knowledge Graph</h1>
          <p className="text-muted text-sm mt-1">Fragmented AI memories and conceptual links synced across the NFM core.</p>
        </div>
        <div className="flex gap-4">
          <div className="nfm-badge nfm-badge--purple">
            <div className="nfm-badge__dot"></div> 12,450 Nodes Synced
          </div>
          <div className="nfm-badge nfm-badge--cyan">
            <div className="nfm-badge__dot"></div> 0.62 Graph Density
          </div>
        </div>
      </div>

      {/* Graph Analytics Stats */}
      <div className="grid" style={{ 
        display: 'grid', 
        gridTemplateColumns: 'repeat(4, 1fr)', 
        gap: 'var(--space-6)', 
        marginBottom: 'var(--space-8)' 
      }}>
        {[
          { label: 'Total Fragments', value: '12,450', icon: <Database size={16}/>, color: 'purple' },
          { label: 'Logic Chains', value: '842', icon: <GitMerge size={16}/>, color: 'cyan' },
          { label: 'Vector Clusters', value: '124', icon: <Layers size={16}/>, color: 'pink' },
          { label: 'Sync Stability', value: '98.4%', icon: <Activity size={16}/>, color: 'success' },
        ].map((stat, idx) => (
          <div key={idx} className="nfm-glass-card" style={{ padding: 'var(--space-5)', marginBottom: 0 }}>
            <div className="flex items-center gap-3 mb-3">
              <div className={`p-2 rounded-lg bg-surface-lowest text-${stat.color}`}>
                {stat.icon}
              </div>
              <span className="text-[10px] text-muted uppercase tracking-widest">{stat.label}</span>
            </div>
            <div className="text-2xl font-display text-primary">{stat.value}</div>
          </div>
        ))}
      </div>

      <div className="flex gap-6 wrap" style={{ flexWrap: 'wrap' }}>
        
        {/* Visualization Area */}
        <div className="flex-col gap-6" style={{ flex: '2 1 600px' }}>
          <div className="nfm-glass-card nfm-glass-card--glow-purple p-0" style={{ height: '560px', position: 'relative', overflow: 'hidden' }}>
            
            {/* Graph Visual Mock */}
            <div style={{ position: 'absolute', inset: 0, background: 'radial-gradient(circle at center, rgba(138,43,226,0.05) 0%, transparent 70%)' }}></div>
            
            {/* NEW: Compact Command HUD Toolbar */}
            <div className="absolute top-4 left-4 z-20 flex items-center gap-2 p-1 bg-surface-lowest-80 backdrop-blur rounded-lg border border-white-05 shadow-2xl">
              <div className="flex items-center gap-1 border-r border-white-05 pr-2 mr-1">
                <button 
                  className={`nfm-btn nfm-btn--sm p-2 ${viewMode === '2D' ? 'nfm-btn--primary' : 'nfm-btn--ghost'}`}
                  onClick={() => setViewMode('2D')}
                >
                  <Network size={14} /> <span className="text-[10px] uppercase font-bold tracking-tighter">2D Abstract</span>
                </button>
                <button 
                  className={`nfm-btn nfm-btn--sm p-2 ${viewMode === '3D' ? 'nfm-btn--primary' : 'nfm-btn--ghost'}`}
                  onClick={() => setViewMode('3D')}
                >
                  <Box size={14} /> <span className="text-[10px] uppercase font-bold tracking-tighter">3D Spatial</span>
                </button>
              </div>
              <div className="flex items-center gap-1">
                 <button className="nfm-btn nfm-btn--ghost nfm-btn--sm p-2" onClick={() => setZoom(z => Math.min(z + 0.2, 2))}><ZoomIn size={14} /></button>
                 <button className="nfm-btn nfm-btn--ghost nfm-btn--sm p-2" onClick={() => setZoom(1)}><Maximize2 size={14} /></button>
                 <button className="nfm-btn nfm-btn--ghost nfm-btn--sm p-2" onClick={() => setZoom(z => Math.max(z - 0.2, 0.5))}><ZoomOut size={14} /></button>
              </div>
            </div>

            {/* NEW: Compact Focus Card (Bottom-Right) */}
            <div className="absolute bottom-4 right-4 z-20 max-w-[240px] animate-in" style={{ animationDuration: '0.6s' }}>
              <div className="p-4 nfm-glass-card" style={{ background: 'rgba(5, 8, 12, 0.9)', border: '1px solid rgba(138,43,226,0.3)', marginBottom: 0, boxShadow: '0 8px 32px rgba(0,0,0,0.8)' }}>
                <div className="flex items-center gap-2 mb-2">
                   <div className="w-2 h-2 rounded-full bg-cyan animate-pulse"></div>
                   <span className="text-[10px] text-muted uppercase tracking-widest">Active Node</span>
                </div>
                <div className="text-xs font-bold text-primary mb-1 truncate">core-engine-topology-v2</div>
                <div className="text-[10px] text-secondary mb-3">Cluster ID: 0x992cf882...</div>
                <div className="flex justify-between items-center text-[10px]">
                   <span className="text-cyan font-mono">24 Links</span>
                   <button className="text-purple hover:underline">Inspect Node</button>
                </div>
              </div>
            </div>

            {/* SVG Visual Content with Toggle Logic */}
            <svg viewBox="0 0 800 600" style={{ width: '100%', height: '100%', cursor: 'crosshair' }}>
               <defs>
                 <radialGradient id="glowPurple" cx="50%" cy="50%" r="50%" fx="50%" fy="50%">
                   <stop offset="0%" style={{ stopColor: 'var(--sovereign-purple)', stopOpacity: 0.4 }} />
                   <stop offset="100%" style={{ stopColor: 'transparent', stopOpacity: 0 }} />
                 </radialGradient>
                 <filter id="nodeGlow">
                   <feGaussianBlur stdDeviation="4" result="blur" />
                   <feComposite in="SourceGraphic" in2="blur" operator="over" />
                 </filter>
               </defs>

               <g transform={`scale(${zoom})`} style={{ transition: 'transform 0.4s cubic-bezier(0.4, 0, 0.2, 1)', transformOrigin: 'center' }}>
                 {/* Connections */}
                 <g opacity={viewMode === '2D' ? '0.2' : '0.1'} stroke="var(--sovereign-purple)" strokeWidth="1" style={{ transition: 'all 0.5s ease' }}>
                   <line x1="400" y1="300" x2={viewMode === '2D' ? 300 : 250} y2={viewMode === '2D' ? 200 : 150} />
                   <line x1="400" y1="300" x2={viewMode === '2D' ? 500 : 650} y2={viewMode === '2D' ? 220 : 180} />
                   <line x1="400" y1="300" x2={viewMode === '2D' ? 480 : 550} y2={viewMode === '2D' ? 420 : 500} />
                   <line x1="400" y1="300" x2={viewMode === '2D' ? 320 : 150} y2={viewMode === '2D' ? 450 : 400} />
                   <line x1={viewMode === '2D' ? 300 : 250} y1={viewMode === '2D' ? 200 : 150} x2="250" y2="280" />
                   <line x1={viewMode === '2D' ? 320 : 150} y1={viewMode === '2D' ? 450 : 400} x2="250" y2="380" />
                 </g>

                 {/* Center Node Glow */}
                 <circle cx="400" cy="300" r={viewMode === '2D' ? 100 : 180} fill="url(#glowPurple)" style={{ transition: 'all 0.8s ease' }} />

                 {/* Nodes with ViewMode logic */}
                 <circle cx="400" cy="300" r={viewMode === '2D' ? 12 : 20} fill="var(--sovereign-purple)" filter="url(#nodeGlow)" style={{ transition: 'all 0.5s' }} />
                 
                 <circle cx={viewMode === '2D' ? 300 : 250} cy={viewMode === '2D' ? 200 : 150} r={viewMode === '2D' ? 8 : 12} fill="var(--neon-cyan)" opacity={viewMode === '3D' ? 0.4 : 0.8} />
                 <circle cx={viewMode === '2D' ? 500 : 650} cy={viewMode === '2D' ? 220 : 180} r={viewMode === '2D' ? 6 : 4} fill="var(--hyper-pink)" opacity={viewMode === '3D' ? 0.3 : 0.6} />
                 <circle cx={viewMode === '2D' ? 480 : 550} cy={viewMode === '2D' ? 420 : 500} r={viewMode === '2D' ? 10 : 16} fill="var(--neon-cyan)" opacity={viewMode === '3D' ? 0.9 : 0.8} />
                 <circle cx={viewMode === '2D' ? 320 : 150} cy={viewMode === '2D' ? 450 : 400} r={viewMode === '2D' ? 7 : 14} fill="var(--sovereign-purple)" opacity={viewMode === '3D' ? 1 : 0.7} />
               </g>
            </svg>

            <div className="absolute bottom-4 left-6 text-[10px] text-muted font-mono tracking-widest uppercase opacity-40">
               Engine: WebGL-Renderer-v2 | Mode: {viewMode}
            </div>
          </div>
        </div>

        {/* Fragment Index & Controller */}
        <div className="flex-col gap-6" style={{ flex: '1 1 350px' }}>
          <div className="nfm-glass-card h-full" style={{ marginBottom: 0 }}>
            <div className="flex items-center gap-2 mb-6">
              <LayoutTemplate className="text-pink" /> 
              <h2 className="text-lg">Fragment Index</h2>
            </div>

            <div className="nfm-search mb-6">
              <Search className="nfm-search__icon" size={16} />
              <input type="text" className="nfm-search__input" placeholder="Search conceptual nodes..." style={{ height: '36px', fontSize: '13px' }}/>
            </div>
            
            <div className="flex-col gap-3">
              {DUMMY_KG_CONCEPTS.map(concept => (
                <div key={concept.id} className="nfm-glass-card--interactive p-4" style={{ background: 'var(--surface-lowest)', borderRadius: 'var(--radius-md)', borderLeft: `3px solid ${concept.category === 'CODE' ? 'var(--neon-cyan)' : concept.category === 'DOCUMENT' ? 'var(--sovereign-purple)' : 'var(--hyper-pink)'}` }}>
                  <div className="flex justify-between items-start mb-2">
                    <span className="font-bold text-sm text-primary">{concept.name}</span>
                    <span className="text-[10px] text-muted font-mono bg-white-05 px-2 py-1 rounded">
                      ID: {concept.id.substring(0, 8)}
                    </span>
                  </div>
                  <div className="flex justify-between items-center text-xs text-secondary">
                    <div className="flex items-center gap-1">
                       <FileText size={12} className="opacity-60" /> {concept.category}
                    </div>
                    <div className="flex items-center gap-1 text-cyan">
                       <GitMerge size={12} /> {concept.connections} links
                    </div>
                  </div>
                </div>
              ))}
            </div>

            <button className="nfm-btn-more mt-6">
              <ArrowRight size={14} /> Full Knowledge Base
            </button>

            <div className="mt-auto pt-8">
               <button className="nfm-btn nfm-btn--secondary w-full">
                  <Activity size={16} /> Run Discovery Audit
               </button>
            </div>
          </div>
        </div>

      </div>
    </div>
  );
};

export default KnowledgeGraph;
