import { useState, useEffect } from 'react';
import { i18n } from '../i18n';
import { useEditorStore } from '../store';

export default function KnowledgeBase() {
  const [filter, setFilter] = useState<string>('all');
  const [showForm, setShowForm] = useState(false);
  const [newEntry, setNewEntry] = useState({
    title: '',
    type: 'Pattern' as const,
    tags: '',
    priority: 5,
  });

  const memoryEntries = useEditorStore(state => state.memoryEntries);
  const addMemoryEntry = useEditorStore(state => state.addMemoryEntry);
  const deleteMemoryEntry = useEditorStore(state => state.deleteMemoryEntry);

  // Default memory entries при первом запуске
  useEffect(() => {
    if (memoryEntries.length === 0) {
      addMemoryEntry({
        title: 'Принцип безопасности',
        type: 'Constitution',
        tags: ['безопасность', 'основы'],
        priority: 10,
      });
      addMemoryEntry({
        title: 'Протокол проверки кода',
        type: 'Protocol',
        tags: ['проверка', 'качество'],
        priority: 8,
      });
      addMemoryEntry({
        title: 'Обработка ошибок в Rust',
        type: 'Pattern',
        tags: ['rust', 'ошибки'],
        priority: 7,
      });
    }
  }, []);

  const filtered = filter === 'all'
    ? memoryEntries
    : memoryEntries.filter(e => e.type === filter);

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'Constitution': return '#f44747';
      case 'Protocol': return '#ce9178';
      case 'Pattern': return '#4ec9b0';
      case 'Tool': return '#569cd6';
      case 'UserRule': return '#9cdcfe';
      default: return '#cccccc';
    }
  };

  const handleAddEntry = () => {
    if (!newEntry.title.trim()) return;

    addMemoryEntry({
      title: newEntry.title,
      type: newEntry.type,
      tags: newEntry.tags.split(',').map(t => t.trim()).filter(t => t),
      priority: newEntry.priority,
    });

    setNewEntry({
      title: '',
      type: 'Pattern',
      tags: '',
      priority: 5,
    });
    setShowForm(false);
  };

  return (
    <div style={{ padding: '8px', display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      <div style={{ marginBottom: '12px', display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
        {['all', 'Constitution', 'Protocol', 'Pattern', 'Tool', 'UserRule'].map(type => (
          <button
            key={type}
            onClick={() => setFilter(type)}
            style={{
              padding: '4px 8px',
              fontSize: '11px',
              background: filter === type ? 'var(--accent)' : 'var(--bg-tertiary)',
              border: 'none',
              borderRadius: '4px',
              color: 'var(--text-primary)',
              cursor: 'pointer',
            }}
          >
            {type === 'all' ? i18n.t('knowledge.all') : i18n.t(`knowledge.${type.toLowerCase()}`)}
          </button>
        ))}
      </div>

      <button
        onClick={() => setShowForm(!showForm)}
        style={{
          padding: '6px 12px',
          fontSize: '12px',
          background: 'var(--accent)',
          border: 'none',
          borderRadius: '4px',
          color: 'white',
          cursor: 'pointer',
          marginBottom: '12px',
        }}
      >
        + {i18n.t('knowledge.addNew')}
      </button>

      {showForm && (
        <div style={{
          padding: '12px',
          background: 'var(--bg-tertiary)',
          borderRadius: '4px',
          marginBottom: '12px',
          border: '1px solid var(--border)',
        }}>
          <input
            type="text"
            placeholder={i18n.t('knowledge.title')}
            value={newEntry.title}
            onChange={(e) => setNewEntry({...newEntry, title: e.target.value})}
            style={{
              width: '100%',
              padding: '6px',
              borderRadius: '4px',
              border: '1px solid var(--border)',
              background: 'var(--bg-secondary)',
              color: 'var(--text-primary)',
              marginBottom: '8px',
              fontSize: '11px',
            }}
          />
          <select
            value={newEntry.type}
            onChange={(e) => setNewEntry({...newEntry, type: e.target.value as any})}
            style={{
              width: '100%',
              padding: '6px',
              borderRadius: '4px',
              border: '1px solid var(--border)',
              background: 'var(--bg-secondary)',
              color: 'var(--text-primary)',
              marginBottom: '8px',
              fontSize: '11px',
            }}
          >
            <option>Constitution</option>
            <option>Protocol</option>
            <option>Pattern</option>
            <option>Tool</option>
            <option>UserRule</option>
          </select>
          <input
            type="text"
            placeholder={i18n.t('knowledge.tags')}
            value={newEntry.tags}
            onChange={(e) => setNewEntry({...newEntry, tags: e.target.value})}
            style={{
              width: '100%',
              padding: '6px',
              borderRadius: '4px',
              border: '1px solid var(--border)',
              background: 'var(--bg-secondary)',
              color: 'var(--text-primary)',
              marginBottom: '8px',
              fontSize: '11px',
            }}
          />
          <input
            type="number"
            min="1"
            max="10"
            value={newEntry.priority}
            onChange={(e) => setNewEntry({...newEntry, priority: parseInt(e.target.value)})}
            style={{
              width: '100%',
              padding: '6px',
              borderRadius: '4px',
              border: '1px solid var(--border)',
              background: 'var(--bg-secondary)',
              color: 'var(--text-primary)',
              marginBottom: '8px',
              fontSize: '11px',
            }}
          />
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={handleAddEntry}
              style={{
                flex: 1,
                padding: '6px',
                background: 'var(--accent)',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '11px',
              }}
            >
              Добавить
            </button>
            <button
              onClick={() => setShowForm(false)}
              style={{
                flex: 1,
                padding: '6px',
                background: 'var(--bg-secondary)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border)',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '11px',
              }}
            >
              Отмена
            </button>
          </div>
        </div>
      )}

      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', overflow: 'auto', flex: 1 }}>
        {filtered.length === 0 ? (
          <div style={{
            padding: '16px',
            textAlign: 'center',
            color: 'var(--text-secondary)',
            fontSize: '12px',
          }}>
            Нет записей в этой категории
          </div>
        ) : (
          filtered.map(entry => (
            <div
              key={entry.id}
              style={{
                padding: '12px',
                background: 'var(--bg-tertiary)',
                borderRadius: '4px',
                border: `2px solid ${getTypeColor(entry.type)}`,
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '8px' }}>
                <strong style={{ fontSize: '13px', flex: 1 }}>{entry.title}</strong>
                <div style={{ display: 'flex', gap: '4px' }}>
                  <span style={{
                    fontSize: '10px',
                    padding: '2px 6px',
                    background: getTypeColor(entry.type),
                    borderRadius: '4px',
                    color: '#1e1e1e',
                    fontWeight: 'bold',
                  }}>
                    {entry.type}
                  </span>
                  <button
                    onClick={() => deleteMemoryEntry(entry.id)}
                    style={{
                      background: '#f44747',
                      border: 'none',
                      borderRadius: '4px',
                      color: 'white',
                      padding: '2px 6px',
                      cursor: 'pointer',
                      fontSize: '10px',
                    }}
                  >
                    ✕
                  </button>
                </div>
              </div>
              {entry.tags && entry.tags.length > 0 && (
                <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap', marginBottom: '6px' }}>
                  {entry.tags.map(tag => (
                    <span
                      key={tag}
                      style={{
                        fontSize: '10px',
                        padding: '2px 6px',
                        background: 'var(--bg-secondary)',
                        borderRadius: '3px',
                        color: '#9cdcfe',
                      }}
                    >
                      #{tag}
                    </span>
                  ))}
                </div>
              )}
              <div style={{ fontSize: '10px', color: 'var(--text-secondary)' }}>
                Приоритет: {'★'.repeat(entry.priority)}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}