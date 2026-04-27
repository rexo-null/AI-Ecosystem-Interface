import { useState } from 'react';
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
        <h1>ISKIN - AI-Powered IDE</h1>
        <div style={{ flex: 1 }} />
        <button className="btn">Run Agent</button>
        <button className="btn btn-success">Sandbox</button>
      </header>

      {/* Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-tabs">
          <button 
            className={`sidebar-tab ${sidebarTab === 'files' ? 'active' : ''}`}
            onClick={() => setSidebarTab('files')}
          >
            Files
          </button>
          <button 
            className={`sidebar-tab ${sidebarTab === 'knowledge' ? 'active' : ''}`}
            onClick={() => setSidebarTab('knowledge')}
          >
            Knowledge
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
