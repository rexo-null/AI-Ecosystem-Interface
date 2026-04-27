import { i18n } from '../i18n';

export default function TerminalPanel() {
  return (
    <div style={{ 
      height: '100%', 
      fontFamily: 'Consolas, Monaco, monospace', 
      fontSize: '13px',
      color: '#cccccc',
      overflow: 'auto',
      padding: '8px',
    }}>
      <div style={{ marginBottom: '8px', display: 'flex', gap: '8px' }}>
        <button className="btn" style={{ padding: '4px 8px', fontSize: '11px' }}>Терминал</button>
        <button className="btn" style={{ padding: '4px 8px', fontSize: '11px' }}>Вывод</button>
        <button className="btn" style={{ padding: '4px 8px', fontSize: '11px' }}>Проблемы</button>
      </div>
      <div style={{ color: 'var(--text-secondary)' }}>
        <p>ISKIN Терминал v0.1.0</p>
        <p>Вводите команды для взаимодействия с системой...</p>
        <br />
        <p><span style={{ color: 'var(--success)' }}>➜</span> <span style={{ color: 'var(--text-primary)' }}>~</span> готов</p>
      </div>
    </div>
  );
}