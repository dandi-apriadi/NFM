import { useEffect, useRef } from 'react';

const InteractiveGrid = () => {
  const mousePos = useRef({ x: -1000, y: -1000 });
  const currentPos = useRef({ x: -1000, y: -1000 });

  useEffect(() => {
    let animationFrameId: number;

    const handleMouseMove = (e: MouseEvent) => {
      mousePos.current = { x: e.clientX, y: e.clientY };
    };

    const loop = () => {
      const lerpFactor = 0.12; // Lower is smoother/slower
      
      currentPos.current.x += (mousePos.current.x - currentPos.current.x) * lerpFactor;
      currentPos.current.y += (mousePos.current.y - currentPos.current.y) * lerpFactor;

      document.documentElement.style.setProperty('--mouse-x', `${currentPos.current.x}px`);
      document.documentElement.style.setProperty('--mouse-y', `${currentPos.current.y}px`);

      animationFrameId = requestAnimationFrame(loop);
    };

    window.addEventListener('mousemove', handleMouseMove);
    animationFrameId = requestAnimationFrame(loop);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      cancelAnimationFrame(animationFrameId);
    };
  }, []);

  return (
    <>
      <div className="nfm-interactive-grid" />
      <div className="nfm-interactive-grid--bubble" />
    </>
  );
};

export default InteractiveGrid;
