import { useState } from 'react';
import { i18n } from './i18n';
import { useEditorStore } from './store';
import FileTree from './components/FileTree';
import KnowledgeBase from './components/KnowledgeBase';
import CodeSearch from './components/CodeSearch';
import SandboxPanel from './components/SandboxPanel';
import EditorArea from './components/EditorArea';
import ChatPanel from './components/ChatPanel';
import TerminalPanel from './components/TerminalPanel';
import AgentStatusBar from './components/AgentStatusBar';
import TaskPanel from './components/TaskPanel';
import ImpactReport from './components/ImpactReport';
import ActionLog from './components/ActionLog';
import ConfirmationDialog from './components/ConfirmationDialog';

type SidebarTab = 'files' | 'knowledge' | 'search' | 'sandbox' | 'tasks' | 'impact' | 'actions';

function App() {
  const [sidebarTab, setSidebarTab] = useState<SidebarTab>('files');
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [confirmAction, setConfirmAction] = useState('');
  const [confirmRisk, setConfirmRisk] = useState<'low' | 'medium' | 'high' | 'critical'>('low');
  
  const { agentActive, setAgentActive, agentPhase, setAgentPhase, addAgentAction } = useEditorStore();
  
  const handleRunAgent = () => {
    if (!agentActive) {
      setAgentActive(true);
      setAgentPhase('ReceiveTask');
      addAgentAction({
        phase: agentPhase,
        description: 'Starting agent task execution',
        status: 'running',
      });
    }
  };
  
  const handleConfirmHighRisk = (action: string) => {
    setConfirmAction(action);
    setConfirmRisk('high');
    setShowConfirmDialog(true);
  };
  
  const handleConfirm = () => {
    setShowConfirmDialog(false);
    // Execute confirmed action
    addAgentAction({
      phase: agentPhase,
      description: confirmAction,
      status: 'completed',
      duration_ms: 0,
    });
  };
  
  const renderSidebarContent = () => {
    switch (sidebarTab) {
      case 'files': return <FileTree />;
      case 'knowledge': return <KnowledgeBase />;
      case 'search': return <CodeSearch />;
      case 'sandbox': return <SandboxPanel />;
      case 'tasks': return <TaskPanel />;
      case 'impact': return <ImpactReport />;
      case 'actions': return <ActionLog />;
    }
  };

  return (
    <div className="app-container">
      {/* Agent Status Bar */}
      <AgentStatusBar />
      
      {/* Header */}
      <header className="header">
        <h1>{i18n.t('header.title')}</h1>
        <div style={{ flex: 1 }} />
        <button className="btn" onClick={handleRunAgent}>
          {i18n.t('header.runAgent')}
        </button>
        <button 
          className="btn btn-success"
          onClick={() => handleConfirmHighRisk('Run sandbox tests')}
        >
          {i18n.t('header.sandbox')}
        </button>
      </header>

      {/* Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-tabs">
          <button 
            className={`sidebar-tab ${sidebarTab === 'files' ? 'active' : ''}`}
            onClick={() => setSidebarTab('files')}
          >
            {i18n.t('sidebar.files')}
          </button>
          <button 
            className={`sidebar-tab ${sidebarTab === 'knowledge' ? 'active' : ''}`}
            onClick={() => setSidebarTab('knowledge')}
          >
            {i18n.t('sidebar.knowledge')}
          </button>
          <button
            className={`sidebar-tab ${sidebarTab === 'search' ? 'active' : ''}`}
            onClick={() => setSidebarTab('search')}
          >
            {i18n.t('sidebar.search')}
          </button>
          <button
            className={`sidebar-tab ${sidebarTab === 'sandbox' ? 'active' : ''}`}
            onClick={() => setSidebarTab('sandbox')}
          >
            Sandbox
          </button>
          <button
            className={`sidebar-tab ${sidebarTab === 'tasks' ? 'active' : ''}`}
            onClick={() => setSidebarTab('tasks')}
          >
            Tasks
          </button>
          <button
            className={`sidebar-tab ${sidebarTab === 'impact' ? 'active' : ''}`}
            onClick={() => setSidebarTab('impact')}
          >
            Impact
          </button>
          <button
            className={`sidebar-tab ${sidebarTab === 'actions' ? 'active' : ''}`}
            onClick={() => setSidebarTab('actions')}
          >
            Actions
          </button>
        </div>
        
        {renderSidebarContent()}
      </aside>

      {/* Main Editor Area */}
      <main className="editor-area">
        <EditorArea />
      </main>

      {/* Chat Panel */}
      <aside className="chat-panel">
        <ChatPanel />
      </aside>

      {/* Terminal Panel */}
      <footer className="terminal-panel">
        <TerminalPanel />
      </footer>
      
      {/* Confirmation Dialog for high-risk actions */}
      {showConfirmDialog && (
        <ConfirmationDialog
          action={confirmAction}
          risk={confirmRisk}
          onConfirm={handleConfirm}
          onCancel={() => setShowConfirmDialog(false)}
        />
      )}
    </div>
  );
}

export default App;
