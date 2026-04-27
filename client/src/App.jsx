import { useState, useEffect } from 'react'
import Editor from '@monaco-editor/react'
import FileTree from './components/FileTree'
import ChatPanel from './components/ChatPanel'
import Tabs from './components/Tabs'
import { useFileSystem } from './hooks/useFileSystem'

function App() {
  const { files, openFiles, activeFile, openFile, closeFile, updateFileContent } = useFileSystem()
  const [editorValue, setEditorValue] = useState('')

  useEffect(() => {
    if (activeFile && files[activeFile]) {
      setEditorValue(files[activeFile].content)
    } else {
      setEditorValue('')
    }
  }, [activeFile, files])

  const handleEditorChange = (value) => {
    if (activeFile && value !== undefined) {
      updateFileContent(activeFile, value)
    }
  }

  return (
    <div className="app-container">
      {/* Sidebar - Project Tree */}
      <aside className="sidebar">
        <div className="sidebar-header">Explorer</div>
        <FileTree 
          files={files}
          onFileSelect={openFile}
          activeFile={activeFile}
        />
      </aside>

      {/* Main Editor Area */}
      <main className="editor-area">
        <Tabs 
          openFiles={openFiles}
          activeFile={activeFile}
          onTabClick={openFile}
          onTabClose={closeFile}
        />
        <div className="editor-container">
          <Editor
            height="100%"
            theme="vs-dark"
            language={activeFile?.split('.').pop()}
            value={editorValue}
            onChange={handleEditorChange}
            options={{
              minimap: { enabled: true },
              fontSize: 14,
              lineNumbers: 'on',
              automaticLayout: true,
              scrollBeyondLastLine: false,
            }}
          />
        </div>
      </main>

      {/* Chat Panel */}
      <ChatPanel />
    </div>
  )
}

export default App
