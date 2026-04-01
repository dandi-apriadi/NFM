import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';

/* --- Design System & Global Styles --- */
import './styles/design-tokens.css';
import './styles/global.css';
import './styles/components.css';
import './styles/layout.css';
import './styles/pages.css';

/* --- Layout & Pages --- */
import AppShell from './components/layout/AppShell';
import Dashboard from './pages/Dashboard';
import Explorer from './pages/Explorer';
import Wallet from './pages/Wallet';
import NodeRunner from './pages/NodeRunner';

import AIBrain from './pages/AIBrain';
import Drive from './pages/Drive';
import KnowledgeGraph from './pages/KnowledgeGraph';

import Marketplace from './pages/Marketplace';
import AssetDetail from './pages/AssetDetail';
import QuestCenter from './pages/QuestCenter';
import MysteryBox from './pages/MysteryBox';

import Governance from './pages/Governance';
import DevPortal from './pages/DevPortal';
import QRLogin from './pages/QRLogin';

import Settings from './pages/Settings';
import { AppDataProvider } from './context/AppDataContext';

const router = createBrowserRouter([
  {
    path: '/login',
    element: <QRLogin />
  },
  {
    path: '/',
    element: <AppShell />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: 'explorer', element: <Explorer /> },
      { path: 'wallet', element: <Wallet /> },
      { path: 'node', element: <NodeRunner /> },
      { path: 'ai', element: <AIBrain /> },
      { path: 'drive', element: <Drive /> },
      { path: 'kg', element: <KnowledgeGraph /> },
      { path: 'market', element: <Marketplace /> },
      { path: 'market/:id', element: <AssetDetail /> },
      { path: 'quests', element: <QuestCenter /> },
      { path: 'mystery', element: <MysteryBox /> },
      { path: 'governance', element: <Governance /> },
      { path: 'dev', element: <DevPortal /> },
      { path: 'settings', element: <Settings /> },
    ],
  },
]);

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <AppDataProvider>
      <RouterProvider router={router} />
    </AppDataProvider>
  </StrictMode>
);
