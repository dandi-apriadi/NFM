import React from 'react';

interface NetworkChartProps {
  data: number[];
  color?: string;
  height?: number;
}

const NetworkChart: React.FC<NetworkChartProps> = ({ 
  data, 
  color = 'var(--neon-cyan)', 
  height = 120 
}) => {
  const safeData = data.length === 0 ? [0, 0] : data.length === 1 ? [data[0], data[0]] : data;
  const max = Math.max(...safeData, 1);
  const min = Math.min(...safeData);
  const range = Math.max(1, max - min);
  
  const pointsData = safeData.map((val, i) => {
    const x = (i / (safeData.length - 1)) * 100;
    const y = 100 - ((val - min) / range) * 70 - 15; // 15% padding top/bottom
    return { x, y, val };
  });

  const polyPoints = pointsData.map(p => `${p.x},${p.y}`).join(' ');

  return (
    <div className="nfm-network-chart" style={{ width: '100%', height: `${height}px`, position: 'relative', marginTop: 'var(--space-4)' }}>
      <svg
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
        style={{ width: '100%', height: '100%', overflow: 'visible' }}
      >
        <defs>
          <linearGradient id="chartGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor={color} stopOpacity="0.2" />
            <stop offset="100%" stopColor={color} stopOpacity="0" />
          </linearGradient>
          <filter id="pointGlow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="1.5" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
          </filter>
        </defs>
        
        {/* Grid Lines (Subtle) */}
        {[0, 25, 50, 75, 100].map(p => (
          <line 
            key={p} 
            x1="0" y1={p} x2="100" y2={p} 
            stroke="rgba(255,255,255,0.03)" 
            strokeWidth="0.2" 
          />
        ))}

        {/* Area Fill */}
        <polyline
          fill="url(#chartGradient)"
          stroke="none"
          points={`0,100 ${polyPoints} 100,100`}
        />
        
        {/* Main Line */}
        <polyline
          fill="none"
          stroke={color}
          strokeWidth="1"
          strokeLinecap="round"
          strokeLinejoin="round"
          points={polyPoints}
          style={{ opacity: 0.6 }}
        />
        
        {/* Dot Data Points */}
        {pointsData.map((p, i) => (
          <g key={i}>
            {/* Outer Glow Circle */}
            <circle 
              cx={p.x} 
              cy={p.y} 
              r="1.2" 
              fill={color} 
              style={{ filter: 'blur(1px)', opacity: 0.5 }}
            />
            {/* Inner Point Circle (The 'Dot') */}
            <circle 
              cx={p.x} 
              cy={p.y} 
              r="0.8" 
              fill="white" 
              style={{ stroke: color, strokeWidth: 0.3 }}
            />
          </g>
        ))}
      </svg>
    </div>
  );
};

export default NetworkChart;
