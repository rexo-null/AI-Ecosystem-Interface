import { useState } from 'react'

const getFileIcon = (fileName) => {
  const ext = fileName.split('.').pop().toLowerCase()
  const icons = {
    js: '📄',
    jsx: '⚛️',
    ts: '📘',
    tsx: '⚛️',
    py: '🐍',
    html: '🌐',
    css: '🎨',
    json: '📋',
    md: '📝',
    gitignore: '🙈',
    lock: '🔒',
    yml: '⚙️',
    yaml: '⚙️',
    env: '🔐',
  }
  return icons[ext] || '📄'
}

function FileTreeItem({ item, depth = 0, onFileSelect, activeFile }) {
  const [isExpanded, setIsExpanded] = useState(true)

  if (item.type === 'directory') {
    return (
      <div>
        <div 
          className="file-tree-item"
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
          onClick={() => setIsExpanded(!isExpanded)}
        >
          <span className="file-icon">{isExpanded ? '📂' : '📁'}</span>
          <span>{item.name}</span>
        </div>
        {isExpanded && item.children && (
          <div className="folder-content">
            {item.children.map((child, index) => (
              <FileTreeItem
                key={index}
                item={child}
                depth={depth + 1}
                onFileSelect={onFileSelect}
                activeFile={activeFile}
              />
            ))}
          </div>
        )}
      </div>
    )
  }

  return (
    <div
      className={`file-tree-item ${activeFile === item.path ? 'active' : ''}`}
      style={{ paddingLeft: `${depth * 16 + 8}px` }}
      onClick={() => onFileSelect(item.path)}
    >
      <span className="file-icon">{getFileIcon(item.name)}</span>
      <span>{item.name}</span>
    </div>
  )
}

function FileTree({ files, onFileSelect, activeFile }) {
  // Преобразуем плоский список файлов в дерево
  const buildTree = () => {
    const root = { name: 'root', type: 'directory', children: [] }
    
    Object.values(files).forEach(file => {
      const parts = file.path.split('/').filter(p => p)
      let current = root
      
      for (let i = 0; i < parts.length - 1; i++) {
        const part = parts[i]
        let child = current.children.find(c => c.name === part && c.type === 'directory')
        if (!child) {
          child = { name: part, type: 'directory', children: [], path: parts.slice(0, i + 1).join('/') }
          current.children.push(child)
        }
        current = child
      }
      
      const fileName = parts[parts.length - 1]
      if (!current.children.some(c => c.name === fileName)) {
        current.children.push({
          name: fileName,
          type: 'file',
          path: file.path
        })
      }
    })
    
    return root.children
  }

  const treeItems = buildTree()

  return (
    <div>
      {treeItems.map((item, index) => (
        <FileTreeItem
          key={index}
          item={item}
          onFileSelect={onFileSelect}
          activeFile={activeFile}
        />
      ))}
      {Object.keys(files).length === 0 && (
        <div style={{ padding: '16px', color: '#888', fontSize: '13px' }}>
          Нет открытых файлов
        </div>
      )}
    </div>
  )
}

export default FileTree
