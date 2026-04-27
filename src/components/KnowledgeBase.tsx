import { useState, useEffect } from 'react';
import { i18n } from '../i18n';
import { useEditorStore } from '../store';

const MEMORY_TYPES = ['Constitution', 'Protocol', 'Pattern', 'ToolDefinition', 'UserRule', 'ProjectContext'] as const;

const TYPE_COLORS: Record<string, string> = {
  Constitution: '#f44747',
  Protocol: '#ce9178',
  Pattern: '#4ec9b0',
  ToolDefinition: '#569cd6',
  UserRule: '#9cdcfe',
  ProjectContext: '#dcdcaa',
};

interface NewEntryForm {
  title: string;
  content: string;
  memory_type: string;
  tags: string;
  priority: number;
}

const EMPTY_FORM: NewEntryForm = {
  title: '',
  content: '',
  memory_type: 'Pattern',
  tags: '',
  priority: 5,
};

export default function KnowledgeBase() {
  const [filter, setFilter] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [newEntry, setNewEntry] = useState<NewEntryForm>(EMPTY_FORM);

  const memoryEntries = useEditorStore(state => state.memoryEntries);
  const knowledgeLoading = useEditorStore(state => state.knowledgeLoading);
  const loadKnowledgeEntries = useEditorStore(state => state.loadKnowledgeEntries);
  const addMemoryEntry = useEditorStore(state => state.addMemoryEntry);
  const deleteMemoryEntry = useEditorStore(state => state.deleteMemoryEntry);
  const searchKnowledge = useEditorStore(state => state.searchKnowledge);

  useEffect(() => {
    loadKnowledgeEntries();
  }, []);

  // Debounced search
  useEffect(() => {
    if (!searchQuery.trim()) {
      loadKnowledgeEntries();
      return;
    }
    const timer = setTimeout(() => {
      searchKnowledge(searchQuery, filter === 'all' ? undefined : filter);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const filtered = filter === 'all'
    ? memoryEntries
    : memoryEntries.filter(e => e.memory_type === filter);

  const handleAddEntry = async () => {
    if (!newEntry.title.trim()) return;

    await addMemoryEntry({
      title: newEntry.title,
      content: newEntry.content,
      memory_type: newEntry.memory_type,
      tags: newEntry.tags.split(',').map(t => t.trim()).filter(t => t),
      priority: newEntry.priority,
    });

    setNewEntry(EMPTY_FORM);
    setShowForm(false);
  };

  const handleFilterChange = (type: string) => {
    setFilter(type);
    if (searchQuery.trim()) {
      searchKnowledge(searchQuery, type === 'all' ? undefined : type);
    } else {
      loadKnowledgeEntries(type === 'all' ? undefined : type);
    }
  };

  const formatDate = (ts: number) => {
    if (!ts) return '';
    const d = new Date(ts > 1e12 ? ts : ts * 1000);
    return d.toLocaleDateString('ru-RU', { day: '2-digit', month: '2-digit', year: '2-digit' });
  };

  const inputStyle: React.CSSProperties = {
    width: '100%',
    padding: '6px',
    borderRadius: '4px',
    border: '1px solid var(--border)',
    background: 'var(--bg-secondary)',
    color: 'var(--text-primary)',
    marginBottom: '8px',
    fontSize: '11px',
    boxSizing: 'border-box',
  };

  return (
    <div style={{ padding: '8px', display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Search bar */}
      <div style={{ marginBottom: '8px' }}>
        <input
          type="text"
          placeholder="Поиск по базе знаний..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          style={{
            ...inputStyle,
            marginBottom: '0',
            padding: '8px',
            fontSize: '12px',
          }}
        />
      </div>

      {/* Filter buttons */}
      <div style={{ marginBottom: '8px', display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
        {['all', ...MEMORY_TYPES].map(type => (
          <button
            key={type}
            onClick={() => handleFilterChange(type)}
            style={{
              padding: '3px 6px',
              fontSize: '10px',
              background: filter === type ? 'var(--accent)' : 'var(--bg-tertiary)',
              border: 'none',
              borderRadius: '3px',
              color: filter === type ? 'white' : 'var(--text-primary)',
              cursor: 'pointer',
            }}
          >
            {type === 'all' ? i18n.t('knowledge.all') : type}
          </button>
        ))}
      </div>

      {/* Add button */}
      <button
        onClick={() => setShowForm(!showForm)}
        style={{
          padding: '6px 12px',
          fontSize: '11px',
          background: showForm ? 'var(--bg-tertiary)' : 'var(--accent)',
          border: 'none',
          borderRadius: '4px',
          color: showForm ? 'var(--text-primary)' : 'white',
          cursor: 'pointer',
          marginBottom: '8px',
        }}
      >
        {showForm ? 'Отмена' : `+ ${i18n.t('knowledge.addNew')}`}
      </button>

      {/* Add form */}
      {showForm && (
        <div style={{
          padding: '10px',
          background: 'var(--bg-tertiary)',
          borderRadius: '4px',
          marginBottom: '8px',
          border: '1px solid var(--border)',
        }}>
          <input
            type="text"
            placeholder={i18n.t('knowledge.title')}
            value={newEntry.title}
            onChange={(e) => setNewEntry({...newEntry, title: e.target.value})}
            style={inputStyle}
          />
          <textarea
            placeholder="Содержимое записи..."
            value={newEntry.content}
            onChange={(e) => setNewEntry({...newEntry, content: e.target.value})}
            rows={3}
            style={{ ...inputStyle, resize: 'vertical' }}
          />
          <select
            value={newEntry.memory_type}
            onChange={(e) => setNewEntry({...newEntry, memory_type: e.target.value})}
            style={inputStyle}
          >
            {MEMORY_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
          </select>
          <input
            type="text"
            placeholder={`${i18n.t('knowledge.tags')} (через запятую)`}
            value={newEntry.tags}
            onChange={(e) => setNewEntry({...newEntry, tags: e.target.value})}
            style={inputStyle}
          />
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
            <label style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>
              {i18n.t('knowledge.priority')}:
            </label>
            <input
              type="range"
              min="1"
              max="10"
              value={newEntry.priority}
              onChange={(e) => setNewEntry({...newEntry, priority: parseInt(e.target.value)})}
              style={{ flex: 1 }}
            />
            <span style={{ fontSize: '11px', minWidth: '20px' }}>{newEntry.priority}</span>
          </div>
          <button
            onClick={handleAddEntry}
            disabled={!newEntry.title.trim()}
            style={{
              width: '100%',
              padding: '6px',
              background: newEntry.title.trim() ? 'var(--accent)' : 'var(--bg-secondary)',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: newEntry.title.trim() ? 'pointer' : 'not-allowed',
              fontSize: '11px',
            }}
          >
            Добавить
          </button>
        </div>
      )}

      {/* Loading indicator */}
      {knowledgeLoading && (
        <div style={{ padding: '8px', textAlign: 'center', color: 'var(--text-secondary)', fontSize: '11px' }}>
          Загрузка...
        </div>
      )}

      {/* Entry list */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', overflow: 'auto', flex: 1 }}>
        {filtered.length === 0 && !knowledgeLoading ? (
          <div style={{
            padding: '16px',
            textAlign: 'center',
            color: 'var(--text-secondary)',
            fontSize: '12px',
          }}>
            {searchQuery ? 'Ничего не найдено' : 'Нет записей в этой категории'}
          </div>
        ) : (
          filtered.map(entry => {
            const isExpanded = expandedId === entry.id;
            const typeColor = TYPE_COLORS[entry.memory_type] || '#cccccc';

            return (
              <div
                key={entry.id}
                style={{
                  padding: '10px',
                  background: 'var(--bg-tertiary)',
                  borderRadius: '4px',
                  borderLeft: `3px solid ${typeColor}`,
                  cursor: 'pointer',
                }}
                onClick={() => setExpandedId(isExpanded ? null : entry.id)}
              >
                {/* Header */}
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <strong style={{ fontSize: '12px', flex: 1 }}>{entry.title}</strong>
                  <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
                    <span style={{
                      fontSize: '9px',
                      padding: '1px 5px',
                      background: typeColor,
                      borderRadius: '3px',
                      color: '#1e1e1e',
                      fontWeight: 'bold',
                    }}>
                      {entry.memory_type}
                    </span>
                    <button
                      onClick={(e) => { e.stopPropagation(); deleteMemoryEntry(entry.id); }}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: '#f44747',
                        cursor: 'pointer',
                        fontSize: '12px',
                        padding: '0 2px',
                        lineHeight: 1,
                      }}
                    >
                      x
                    </button>
                  </div>
                </div>

                {/* Tags */}
                {entry.tags && entry.tags.length > 0 && (
                  <div style={{ display: 'flex', gap: '3px', flexWrap: 'wrap', marginTop: '4px' }}>
                    {entry.tags.map(tag => (
                      <span
                        key={tag}
                        style={{
                          fontSize: '9px',
                          padding: '1px 4px',
                          background: 'var(--bg-secondary)',
                          borderRadius: '2px',
                          color: '#9cdcfe',
                        }}
                      >
                        #{tag}
                      </span>
                    ))}
                  </div>
                )}

                {/* Meta */}
                <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px', fontSize: '9px', color: 'var(--text-secondary)' }}>
                  <span>{i18n.t('knowledge.priority')}: {'★'.repeat(Math.min(entry.priority, 10))}</span>
                  {entry.created_at ? <span>{formatDate(entry.created_at)}</span> : null}
                </div>

                {/* Expanded content */}
                {isExpanded && entry.content && (
                  <div style={{
                    marginTop: '8px',
                    padding: '8px',
                    background: 'var(--bg-secondary)',
                    borderRadius: '3px',
                    fontSize: '11px',
                    lineHeight: '1.5',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                    color: 'var(--text-primary)',
                    borderTop: '1px solid var(--border)',
                  }}>
                    {entry.content}
                  </div>
                )}

                {/* Access count */}
                {isExpanded && (
                  <div style={{ marginTop: '4px', fontSize: '9px', color: 'var(--text-secondary)' }}>
                    Обращений: {entry.access_count || 0}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
