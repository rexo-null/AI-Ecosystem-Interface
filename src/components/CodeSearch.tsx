import { useState } from 'react';
import { useEditorStore } from '../store';

const LANGUAGES = [
  { value: '', label: 'Все языки' },
  { value: 'rust', label: 'Rust' },
  { value: 'typescript', label: 'TypeScript' },
  { value: 'python', label: 'Python' },
];

const SYMBOL_COLORS: Record<string, string> = {
  Function: '#dcdcaa',
  Struct: '#4ec9b0',
  Enum: '#4ec9b0',
  Class: '#4ec9b0',
  Interface: '#4ec9b0',
  Trait: '#b5cea8',
  Impl: '#c586c0',
  Type: '#4ec9b0',
  Const: '#4fc1ff',
  Static: '#4fc1ff',
  Variable: '#9cdcfe',
  Import: '#c586c0',
  Module: '#ce9178',
};

export default function CodeSearch() {
  const [query, setQuery] = useState('');
  const [language, setLanguage] = useState('');
  const [projectPath, setProjectPath] = useState('');
  const [expandedFile, setExpandedFile] = useState<string | null>(null);

  const searchResults = useEditorStore(state => state.searchResults);
  const indexStats = useEditorStore(state => state.indexStats);
  const codeSearchLoading = useEditorStore(state => state.codeSearchLoading);
  const searchCode = useEditorStore(state => state.searchCode);
  const indexProject = useEditorStore(state => state.indexProject);
  const openFile = useEditorStore(state => state.openFile);

  const handleSearch = () => {
    if (!query.trim()) return;
    searchCode(query, language || undefined, 20);
  };

  const handleIndex = () => {
    if (!projectPath.trim()) return;
    indexProject(projectPath);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleSearch();
  };

  const handleFileClick = (filePath: string, preview: string) => {
    openFile({
      path: filePath,
      content: preview,
      language: filePath.split('.').pop() || 'text',
    });
  };

  const inputStyle: React.CSSProperties = {
    width: '100%',
    padding: '6px',
    borderRadius: '4px',
    border: '1px solid var(--border)',
    background: 'var(--bg-secondary)',
    color: 'var(--text-primary)',
    fontSize: '11px',
    boxSizing: 'border-box',
  };

  return (
    <div style={{ padding: '8px', display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Index section */}
      <div style={{
        marginBottom: '8px',
        padding: '8px',
        background: 'var(--bg-tertiary)',
        borderRadius: '4px',
        border: '1px solid var(--border)',
      }}>
        <div style={{ fontSize: '11px', fontWeight: 'bold', marginBottom: '6px', color: 'var(--text-primary)' }}>
          Индексация проекта
        </div>
        <div style={{ display: 'flex', gap: '4px' }}>
          <input
            type="text"
            placeholder="Путь к проекту..."
            value={projectPath}
            onChange={(e) => setProjectPath(e.target.value)}
            style={{ ...inputStyle, flex: 1 }}
          />
          <button
            onClick={handleIndex}
            disabled={!projectPath.trim() || codeSearchLoading}
            style={{
              padding: '6px 10px',
              fontSize: '10px',
              background: projectPath.trim() ? 'var(--accent)' : 'var(--bg-secondary)',
              border: 'none',
              borderRadius: '4px',
              color: 'white',
              cursor: projectPath.trim() ? 'pointer' : 'not-allowed',
              whiteSpace: 'nowrap',
            }}
          >
            {codeSearchLoading ? '...' : 'Index'}
          </button>
        </div>

        {/* Index stats */}
        {indexStats && (
          <div style={{ marginTop: '6px', fontSize: '9px', color: 'var(--text-secondary)', display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            <span>Файлов: {indexStats.total_files}</span>
            <span>Символов: {indexStats.total_symbols}</span>
            <span>Строк: {indexStats.total_lines}</span>
            {Object.entries(indexStats.languages).map(([lang, count]) => (
              <span key={lang} style={{ color: '#9cdcfe' }}>{lang}: {count}</span>
            ))}
          </div>
        )}
      </div>

      {/* Search section */}
      <div style={{ marginBottom: '8px' }}>
        <div style={{ display: 'flex', gap: '4px', marginBottom: '4px' }}>
          <input
            type="text"
            placeholder="Поиск по коду..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            style={{ ...inputStyle, flex: 1 }}
          />
          <button
            onClick={handleSearch}
            disabled={!query.trim() || codeSearchLoading}
            style={{
              padding: '6px 10px',
              fontSize: '10px',
              background: query.trim() ? 'var(--accent)' : 'var(--bg-secondary)',
              border: 'none',
              borderRadius: '4px',
              color: 'white',
              cursor: query.trim() ? 'pointer' : 'not-allowed',
            }}
          >
            Найти
          </button>
        </div>
        <select
          value={language}
          onChange={(e) => setLanguage(e.target.value)}
          style={{ ...inputStyle }}
        >
          {LANGUAGES.map(l => (
            <option key={l.value} value={l.value}>{l.label}</option>
          ))}
        </select>
      </div>

      {/* Loading */}
      {codeSearchLoading && (
        <div style={{ padding: '8px', textAlign: 'center', color: 'var(--text-secondary)', fontSize: '11px' }}>
          Поиск...
        </div>
      )}

      {/* Results */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '4px', overflow: 'auto', flex: 1 }}>
        {searchResults.length === 0 && !codeSearchLoading && query.trim() && (
          <div style={{ padding: '16px', textAlign: 'center', color: 'var(--text-secondary)', fontSize: '11px' }}>
            Ничего не найдено
          </div>
        )}

        {searchResults.map((result, idx) => {
          const isExpanded = expandedFile === result.file_path;
          const fileName = result.file_path.split('/').pop() || result.file_path;

          return (
            <div
              key={`${result.file_path}-${idx}`}
              style={{
                background: 'var(--bg-tertiary)',
                borderRadius: '4px',
                overflow: 'hidden',
                border: '1px solid var(--border)',
              }}
            >
              {/* File header */}
              <div
                style={{
                  padding: '6px 8px',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  cursor: 'pointer',
                }}
                onClick={() => setExpandedFile(isExpanded ? null : result.file_path)}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flex: 1, overflow: 'hidden' }}>
                  <span style={{ fontSize: '11px', color: '#dcdcaa', fontWeight: 'bold' }}>{fileName}</span>
                  <span style={{ fontSize: '9px', color: 'var(--text-secondary)' }}>{result.language}</span>
                </div>
                <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
                  <span style={{ fontSize: '9px', color: 'var(--text-secondary)' }}>
                    {result.symbols.length} sym / {result.line_count} lines
                  </span>
                  <button
                    onClick={(e) => { e.stopPropagation(); handleFileClick(result.file_path, result.preview); }}
                    style={{
                      padding: '2px 6px',
                      fontSize: '9px',
                      background: 'var(--accent)',
                      border: 'none',
                      borderRadius: '3px',
                      color: 'white',
                      cursor: 'pointer',
                    }}
                  >
                    Open
                  </button>
                </div>
              </div>

              {/* Preview */}
              {isExpanded && (
                <div style={{ borderTop: '1px solid var(--border)' }}>
                  {/* Code preview */}
                  <pre style={{
                    padding: '6px 8px',
                    margin: 0,
                    fontSize: '10px',
                    color: 'var(--text-primary)',
                    background: 'var(--bg-secondary)',
                    overflow: 'auto',
                    maxHeight: '120px',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-all',
                  }}>
                    {result.preview}
                  </pre>

                  {/* Symbols list */}
                  {result.symbols.length > 0 && (
                    <div style={{ padding: '6px 8px', borderTop: '1px solid var(--border)' }}>
                      <div style={{ fontSize: '9px', color: 'var(--text-secondary)', marginBottom: '4px' }}>
                        Символы:
                      </div>
                      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '3px' }}>
                        {result.symbols.slice(0, 20).map((sym, si) => (
                          <span
                            key={`${sym.name}-${si}`}
                            style={{
                              fontSize: '9px',
                              padding: '1px 4px',
                              background: 'var(--bg-tertiary)',
                              borderRadius: '2px',
                              color: SYMBOL_COLORS[sym.kind] || '#cccccc',
                              border: '1px solid var(--border)',
                            }}
                            title={`${sym.kind} at line ${sym.start_line}`}
                          >
                            {sym.kind === 'Function' ? 'fn ' : ''}{sym.name}
                          </span>
                        ))}
                        {result.symbols.length > 20 && (
                          <span style={{ fontSize: '9px', color: 'var(--text-secondary)' }}>
                            +{result.symbols.length - 20} more
                          </span>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
