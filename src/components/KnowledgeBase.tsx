import { useState, useEffect } from 'react';

interface MemoryEntry {
  id: string;
  title: string;
  type: 'Constitution' | 'Protocol' | 'Pattern' | 'UserRule' | 'Tool';
  tags: string[];
  priority: number;
}

const mockKnowledge: MemoryEntry[] = [
  { id: '1', title: 'Core Safety Principle', type: 'Constitution', tags: ['safety', 'core'], priority: 10 },
  { id: '2', title: 'Code Review Protocol', type: 'Protocol', tags: ['review', 'quality'], priority: 8 },
  { id: '3', title: 'Rust Error Handling', type: 'Pattern', tags: ['rust', 'errors'], priority: 7 },
  { id: '4', title: 'Custom Tool: File Watcher', type: 'Tool', tags: ['tool', 'filesystem'], priority: 5 },
];

export default function KnowledgeBase() {
  const [filter, setFilter] = useState<string>('all');
  const [entries, setEntries] = useState<MemoryEntry[]>(mockKnowledge);

  const filtered = filter === 'all' 
    ? entries 
    : entries.filter(e => e.type === filter);

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'Constitution': return '#f44747';
      case 'Protocol': return '#ce9178';
      case 'Pattern': return '#4ec9b0';
      case 'Tool': return '#569cd6';
      default: return '#cccccc';
    }
  };

  return (
    <div style={{ padding: '8px' }}>
      <div style={{ marginBottom: '12px', display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
        {['all', 'Constitution', 'Protocol', 'Pattern', 'Tool'].map(type => (
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
            {type}
          </button>
        ))}
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {filtered.map(entry => (
          <div
            key={entry.id}
            style={{
              padding: '12px',
              background: 'var(--bg-tertiary)',
              borderRadius: '4px',
              border: `2px solid ${getTypeColor(entry.type)}`,
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
              <strong style={{ fontSize: '13px' }}>{entry.title}</strong>
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
            </div>
            <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
              {entry.tags.map(tag => (
                <span key={tag} style={{ fontSize: '10px', color: 'var(--text-secondary)' }}>#{tag}</span>
              ))}
            </div>
            <div style={{ marginTop: '8px', fontSize: '11px', color: 'var(--text-secondary)' }}>
              Priority: {entry.priority}/10
            </div>
          </div>
        ))}
      </div>

      <button 
        className="btn"
        style={{ marginTop: '16px', width: '100%' }}
      >
        + Add New Entry
      </button>
    </div>
  );
}