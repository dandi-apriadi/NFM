import { NavLink } from 'react-router-dom';
import { 
  Box, 
  Activity, 
  Wallet, 
  Cpu,
  Brain,
  HardDrive,
  Share2,
  ShoppingCart,
  Target,
  Gift,
  AlignLeft,
  Settings as SettingsIcon,
  Code
} from 'lucide-react';
import { useAppData } from '../../context/AppDataContext';

const Sidebar = () => {
  const { data } = useAppData();
  const DUMMY_USER = data.user_profile;

  return (
    <aside className="nfm-sidebar">
      <div className="nfm-sidebar__logo">
        <Activity size={28} className="text-cyan animate-pulse" />
        <span className="font-display">NFM Vault</span>
      </div>

      <nav className="nfm-sidebar__nav">
        <div className="nav-section">Core Infrastructure</div>
        <NavLink to="/" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`} end>
          <Box size={20} /> Dashboard
        </NavLink>
        <NavLink to="/explorer" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Activity size={20} /> Blockchain Explorer
        </NavLink>
        <NavLink to="/wallet" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Wallet size={20} /> Wallet Management
        </NavLink>
        <NavLink to="/node" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Cpu size={20} /> Node Runner
        </NavLink>

        <div className="nav-section mt-4">Super-App & Features</div>
        <NavLink to="/ai" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Brain size={20} /> AI Autopilot
        </NavLink>
        <NavLink to="/drive" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <HardDrive size={20} /> NFM Drive
        </NavLink>
        <NavLink to="/kg" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Share2 size={20} /> Knowledge Graph
        </NavLink>
        <NavLink to="/market" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <ShoppingCart size={20} /> Marketplace
        </NavLink>
        <NavLink to="/quests" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Target size={20} /> Quest Center
        </NavLink>
        <NavLink to="/mystery" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Gift size={20} /> Mystery Box
        </NavLink>

        <div className="nav-section mt-4">Governance & Dev</div>
        <NavLink to="/governance" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <AlignLeft size={20} /> Governance
        </NavLink>
        <NavLink to="/dev" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <Code size={20} /> The Forge
        </NavLink>
        <NavLink to="/settings" className={({isActive}) => `nav-item ${isActive ? 'active' : ''}`}>
          <SettingsIcon size={20} /> Settings
        </NavLink>
      </nav>

      <div className="nfm-sidebar__footer">
        <div className="user-profile">
          <div className="nfm-avatar">FO</div>
          <div className="user-info">
            <span className="username">{DUMMY_USER.username}</span>
            <span className="balance">{DUMMY_USER.balance.toLocaleString('en-US')} NVC</span>
          </div>
        </div>
      </div>
{/* Base styles for sidebar added via inline or separate css. Since we use app shell CSS, we'll put it there */}      
    </aside>
  );
};

export default Sidebar;
