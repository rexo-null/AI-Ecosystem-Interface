import { useState } from 'react';
import { i18n } from './i18n';
import FileTree from './components/FileTree';
import KnowledgeBase from './components/KnowledgeBase';
import EditorArea from './components/EditorArea';
import ChatPanel from './components/ChatPanel';
import TerminalPanel from './components/TerminalPanel';

function App() {
  const [sidebarTab, setSidebarTab] = useState<'files' | 'knowledge'>('files');

  return (
    <div className="app-container">
      {/* Header */}
      <header className="header">
        <h1>{i18n.t('header.title')}</h1>
        <div style={{ flex: 1 }} />
        <button className="btn">{i18n.t('header.runAgent')}</button>
        <button className="btn btn-success">{i18n.t('header.sandbox')}</button>
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
        </div>
        
        {sidebarTab === 'files' ? <FileTree /> : <KnowledgeBase />}
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
    </div>
  );
}

export default App;
