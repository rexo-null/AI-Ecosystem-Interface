import { useState } from 'react';

export default function EditorArea() {
  const [tabs, setTabs] = useState([{ id: '1', name: 'main.rs', path: '/src-tauri/src/main.rs' }]);
  const [activeTab, setActiveTab] = useState('1');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div className="editor-tabs">
        {tabs.map(tab => (
          <button
            key={tab.id}
            className={`editor-tab ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.name}
          </button>
        ))}
      </div>
      
      <div className="editor-container" style={{ 
        flex: 1, 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'center',
        color: 'var(--text-secondary)',
        fontSize: '14px',
      }}>
        <div>
          <p>Monaco Editor will be integrated here</p>
          <p style={{ marginTop: '8px', fontSize: '12px' }}>Active file: {tabs.find(t => t.id === activeTab)?.path}</p>
        </div>
      </div>
    </div>
  );
}