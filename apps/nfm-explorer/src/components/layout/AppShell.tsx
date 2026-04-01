import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import TopBar from './TopBar';
import InteractiveGrid from './InteractiveGrid';

const AppShell = () => {
  return (
    <div className="nfm-app" style={{ position: 'relative' }}>
      <InteractiveGrid />
      
      <Sidebar />
      <div className="nfm-main-container" style={{ zIndex: 2, position: 'relative' }}>
        <TopBar />
        <main className="nfm-content">
          <Outlet />
        </main>
      </div>
    </div>
  );
};

export default AppShell;
