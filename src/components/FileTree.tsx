import { useState, useEffect } from 'react';
import { i18n } from '../i18n';
import { fileService } from '../fileService';
import { useEditorStore } from '../store';

interface FileNode {
  id: string;
  name: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  path: string;
}

export default function FileTree() {
  const [expanded, setExpanded] = useState<Set<string>>(new Set(['1', '5']));
  const [files, setFiles] = useState<FileNode[]>([]);
  const [loading, setLoading] = useState(true);
  const openFile = useEditorStore(state => state.openFile);

  useEffect(() => {
    loadFiles();
  }, []);

  const loadFiles = async () => {
    setLoading(true);
    try {
      const fileList = await fileService.listFiles('/');
      setFiles(fileList);
    } catch (error) {
      console.error('Error loading files:', error);
    } finally {
      setLoading(false);
    }
  };

  const toggleExpand = (id: string) => {
    const newExpanded = new Set(expanded);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpanded(newExpanded);
  };

  const handleFileClick = async (node: FileNode) => {
    if (node.type === 'file') {
      try {
        const content = await fileService.readFile(node.path);
        openFile(content);
      } catch (error) {
        console.error('Error opening file:', error);
      }
    }
  };

  const renderNode = (node: FileNode, depth: number = 0) => {
    const isExpanded = expanded.has(node.id);
    const hasChildren = node.children && node.children.length > 0;

    return (
      <div key={node.id}>
        <div 
          style={{ 
            paddingLeft: `${depth * 16}px`,
            padding: '6px 8px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            fontSize: '13px',
            transition: 'background-color 0.2s',
          }}
          onClick={() => {
            if (hasChildren) {
              toggleExpand(node.id);
            } else {
              handleFileClick(node);
            }
          }}
          onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'var(--bg-tertiary)'}
          onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'transparent'}
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

  if (loading) {
    return (
      <div style={{ padding: '16px', textAlign: 'center', color: 'var(--text-secondary)' }}>
        {i18n.t('fileTree.loading')}
      </div>
    );
  }

  return (
    <div style={{ padding: '8px 0', overflow: 'auto', height: '100%' }}>
      {files.length > 0 ? (
        files.map(node => renderNode(node))
      ) : (
        <div style={{ padding: '16px', textAlign: 'center', color: 'var(--text-secondary)' }}>
          {i18n.t('fileTree.error')}
        </div>
      )}
    </div>
  );
}