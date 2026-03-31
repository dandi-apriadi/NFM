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
  const max = Math.max(...data);
  const min = Math.min(...data);
  const range = max - min;
  
  const points = data.map((val, i) => {
    const x = (i / (data.length - 1)) * 100;
    const y = 100 - ((val - min) / range) * 80 - 10; // 10% padding
    return `${x},${y}`;
  }).join(' ');

  return (
    <div style={{ width: '100%', height: `${height}px`, position: 'relative', marginTop: 'var(--space-4)' }}>
      <svg
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
        style={{ width: '100%', height: '100%', overflow: 'visible' }}
      >
        <defs>
          <linearGradient id="chartGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor={color} stopOpacity="0.3" />
            <stop offset="100%" stopColor={color} stopOpacity="0" />
          </linearGradient>
        </defs>
        
        {/* Area */}
        <polyline
          fill="url(#chartGradient)"
          stroke="none"
          points={`0,100 ${points} 100,100`}
        />
        
        {/* Line */}
        <polyline
          fill="none"
          stroke={color}
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          points={points}
          style={{ filter: `drop-shadow(0 0 4px ${color})` }}
        />
        
        {/* Grid Lines (Subtle) */}
        {[0, 25, 50, 75, 100].map(p => (
          <line 
            key={p} 
            x1="0" y1={p} x2="100" y2={p} 
            stroke="rgba(255,255,255,0.03)" 
            strokeWidth="0.5" 
          />
        ))}
      </svg>
    </div>
  );
};

export default NetworkChart;
