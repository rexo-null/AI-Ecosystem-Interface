interface FileNode {
  id: string;
  name: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  path: string;
}

interface FileContent {
  path: string;
  content: string;
  language?: string;
}

// Функция для определения языка файла по расширению
const getLanguageFromExtension = (filename: string): string => {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  const languageMap: Record<string, string> = {
    ts: 'typescript',
    tsx: 'typescript',
    js: 'javascript',
    jsx: 'javascript',
    rs: 'rust',
    py: 'python',
    java: 'java',
    cpp: 'cpp',
    c: 'c',
    go: 'go',
    rb: 'ruby',
    php: 'php',
    json: 'json',
    xml: 'xml',
    html: 'html',
    css: 'css',
    scss: 'scss',
    sql: 'sql',
    md: 'markdown',
    txt: 'plaintext',
    toml: 'toml',
    yaml: 'yaml',
    yml: 'yaml',
  };
  return languageMap[ext] || 'plaintext';
};

// Simulated file system reader (для dev)
// В реальном приложении это будет использовать API к серверу
export const fileService = {
  // Получить список файлов из директории
  listFiles: async (dirPath: string = '.'): Promise<FileNode[]> => {
    try {
      const response = await fetch('/api/files/list', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: dirPath }),
      });
      
      if (!response.ok) {
        // Fallback для работы без backend
        return getMockFiles();
      }
      
      return await response.json();
    } catch (error) {
      console.warn('Failed to fetch real files, using mock data:', error);
      return getMockFiles();
    }
  },

  // Прочитать содержимое файла
  readFile: async (filePath: string): Promise<FileContent> => {
    try {
      const response = await fetch('/api/files/read', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: filePath }),
      });
      
      if (!response.ok) {
        throw new Error('Failed to read file');
      }
      
      const data = await response.json();
      return {
        path: filePath,
        content: data.content,
        language: getLanguageFromExtension(filePath),
      };
    } catch (error) {
      console.error('Error reading file:', error);
      return {
        path: filePath,
        content: '// Ошибка при загрузке файла',
        language: 'plaintext',
      };
    }
  },

  // Сохранить файл
  writeFile: async (filePath: string, content: string): Promise<boolean> => {
    try {
      const response = await fetch('/api/files/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: filePath, content }),
      });
      
      return response.ok;
    } catch (error) {
      console.error('Error writing file:', error);
      return false;
    }
  },
};

// Mock данные для разработки
function getMockFiles(): FileNode[] {
  return [
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
            {
              id: '3a',
              name: 'api',
              type: 'directory',
              path: '/src-tauri/src/api',
              children: [
                { id: '3b', name: 'commands.rs', type: 'file', path: '/src-tauri/src/api/commands.rs' },
                { id: '3c', name: 'mod.rs', type: 'file', path: '/src-tauri/src/api/mod.rs' },
              ],
            },
          ],
        },
      ],
    },
    {
      id: '5',
      name: 'src',
      type: 'directory',
      path: '/src',
      children: [
        { id: '6', name: 'App.tsx', type: 'file', path: '/src/App.tsx' },
        { id: '7', name: 'main.tsx', type: 'file', path: '/src/main.tsx' },
        { id: '8', name: 'i18n.ts', type: 'file', path: '/src/i18n.ts' },
        {
          id: '9',
          name: 'components',
          type: 'directory',
          path: '/src/components',
          children: [
            { id: '10', name: 'ChatPanel.tsx', type: 'file', path: '/src/components/ChatPanel.tsx' },
            { id: '11', name: 'EditorArea.tsx', type: 'file', path: '/src/components/EditorArea.tsx' },
            { id: '12', name: 'FileTree.tsx', type: 'file', path: '/src/components/FileTree.tsx' },
            { id: '13', name: 'KnowledgeBase.tsx', type: 'file', path: '/src/components/KnowledgeBase.tsx' },
            { id: '14', name: 'TerminalPanel.tsx', type: 'file', path: '/src/components/TerminalPanel.tsx' },
          ],
        },
      ],
    },
    { id: '15', name: 'package.json', type: 'file', path: '/package.json' },
    { id: '16', name: 'README.md', type: 'file', path: '/README.md' },
    { id: '17', name: 'vite.config.ts', type: 'file', path: '/vite.config.ts' },
  ];
}
