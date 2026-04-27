import { useState } from 'react';

interface FileNode {
  id: string;
  name: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  path: string;
}

const mockFiles: FileNode[] = [
  {
    id: '1',
    name: 'src-tauri',
    type: 'directory',
    path: '/src-tauri',
    children: [
      { id: '2', name: 'Cargo.toml', type: 'file', path: '/src-tauri/Cargo.toml' },
      { 
        id: '3', 
        name: 'src', 
        type: 'directory', 
        path: '/src-tauri/src',
        children: [
          { id: '4', name: 'main.rs', type: 'file', path: '/src-tauri/src/main.rs' },
        ]
      },
    ]
  },
  {
    id: '5',
    name: 'src',
    type: 'directory',
    path: '/src',
    children: [
      { id: '6', name: 'App.tsx', type: 'file', path: '/src/App.tsx' },
      { id: '7', name: 'main.tsx', type: 'file', path: '/src/main.tsx' },
    ]
  },
  { id: '8', name: 'README.md', type: 'file', path: '/README.md' },
];

export default function FileTree() {
  const [expanded, setExpanded] = useState<Set<string>>(new Set(['1', '5']));

  const toggleExpand = (id: string) => {
    const newExpanded = new Set(expanded);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpanded(newExpanded);
  };

  const renderNode = (node: FileNode, depth: number = 0) => {
    const isExpanded = expanded.has(node.id);
    const hasChildren = node.children && node.children.length > 0;

    return (
      <div key={node.id}>
        <div 
          style={{ 
            paddingLeft: `${depth * 16}px`,
            padding: '4px 8px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            fontSize: '13px',
          }}
          onClick={() => hasChildren && toggleExpand(node.id)}
        >
          <span>{hasChildren ? (isExpanded ? '📂' : '📁') : '📄'}</span>
          <span>{node.name}</span>
        </div>
        {hasChildren && isExpanded && (
          <div>
            {node.children!.map(child => renderNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div style={{ padding: '8px 0' }}>
      {mockFiles.map(node => renderNode(node))}
    </div>
  );
}