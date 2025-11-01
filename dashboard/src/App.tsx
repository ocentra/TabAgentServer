
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import ThemeProvider from '@/components/providers/ThemeProvider';
import QueryProvider from '@/components/providers/QueryProvider';
import Layout from '@/components/layout/Layout';
import Dashboard from '@/pages/Dashboard';
import Logs from '@/pages/Logs';
import Models from '@/pages/Models';
import Database from '@/pages/Database';
import Settings from '@/pages/Settings';
import API from '@/pages/API';
import WebRTC from '@/pages/WebRTC';
import Chat from '@/pages/Chat';
import Knowledge from '@/pages/Knowledge';
import KnowledgeGraph from '@/pages/KnowledgeGraph';
import NotFound from '@/pages/NotFound';

function App() {
  return (
    <QueryProvider>
      <ThemeProvider>
        <Router>
          <div className="min-h-screen bg-background text-foreground">
            <Routes>
              <Route path="/" element={<Layout />}>
                <Route index element={<Dashboard />} />
                <Route path="logs" element={<Logs />} />
                <Route path="models" element={<Models />} />
                <Route path="database" element={<Database />} />
                <Route path="settings" element={<Settings />} />
                <Route path="api" element={<API />} />
                <Route path="webrtc" element={<WebRTC />} />
                <Route path="chat" element={<Chat />} />
                <Route path="knowledge" element={<Knowledge />} />
                <Route path="knowledge-graph" element={<KnowledgeGraph />} />
              </Route>
              <Route path="*" element={<NotFound />} />
            </Routes>
          </div>
        </Router>
      </ThemeProvider>
    </QueryProvider>
  );
}

export default App;
