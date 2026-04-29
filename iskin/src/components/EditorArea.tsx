import { useEffect, useRef } from 'react';
import { i18n } from '../i18n';
import { useEditorStore } from '../store';
import * as monaco from 'monaco-editor';

export default function EditorArea() {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const activeFile = useEditorStore(state => state.activeFile);
  const openTabs = useEditorStore(state => state.openTabs);
  const setActiveFile = useEditorStore(state => state.setActiveFile);
  const closeTab = useEditorStore(state => state.closeTab);

  useEffect(() => {
    if (!containerRef.current) return;

    // Инициализация Monaco Editor
    if (!editorRef.current) {
      editorRef.current = monaco.editor.create(containerRef.current, {
        value: activeFile?.content || '',
        language: activeFile?.language || 'plaintext',
        theme: 'vs-dark',
        readOnly: true,
        automaticLayout: true,
      });
    } else if (activeFile) {
      editorRef.current.setValue(activeFile.content);
      monaco.editor.setModelLanguage(
        editorRef.current.getModel()!,
        activeFile.language || 'plaintext'
      );
    }

    return () => {
      // Cleanup при необходимости
    };
  }, [activeFile]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div className="editor-tabs" style={{ 
        display: 'flex', 
        gap: '4px',
        borderBottom: '1px solid var(--border)',
        overflow: 'auto',
        padding: '4px',
        minHeight: '36px',
      }}>
        {openTabs.length === 0 ? (
          <div style={{ 
            padding: '8px 12px',
            color: 'var(--text-secondary)',
            fontSize: '12px',
            display: 'flex',
            alignItems: 'center',
          }}>
            {i18n.t('editor.placeholder')}
          </div>
        ) : (
          openTabs.map(tab => (
            <button
              key={tab.path}
              className={`editor-tab ${activeFile?.path === tab.path ? 'active' : ''}`}
              onClick={() => setActiveFile(tab)}
              style={{
                padding: '6px 12px',
                background: activeFile?.path === tab.path ? 'var(--accent)' : 'var(--bg-tertiary)',
                border: '1px solid var(--border)',
                borderRadius: '4px 4px 0 0',
                cursor: 'pointer',
                color: 'var(--text-primary)',
                fontSize: '12px',
                whiteSpace: 'nowrap',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
              }}
            >
              {tab.path.split('/').pop()}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  closeTab(tab.path);
                }}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'inherit',
                  cursor: 'pointer',
                  fontSize: '14px',
                  padding: '0',
                }}
              >
                ×
              </button>
            </button>
          ))
        )}
      </div>
      
      <div 
        className="editor-container"
        ref={containerRef}
        style={{ 
          flex: 1,
          position: 'relative',
        }}
      >
        {!activeFile && (
          <div style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            textAlign: 'center',
            color: 'var(--text-secondary)',
            fontSize: '14px',
            pointerEvents: 'none',
          }}>
            {i18n.t('editor.placeholder')}
          </div>
        )}
      </div>
    </div>
  );
}