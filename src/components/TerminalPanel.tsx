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
        <button className="btn" style={{ padding: '4px 8px', fontSize: '11px' }}>Terminal</button>
        <button className="btn" style={{ padding: '4px 8px', fontSize: '11px' }}>Output</button>
        <button className="btn" style={{ padding: '4px 8px', fontSize: '11px' }}>Problems</button>
      </div>
      <div style={{ color: 'var(--text-secondary)' }}>
        <p>ISKIN Terminal v0.1.0</p>
        <p>Type commands to interact with the system...</p>
        <br />
        <p><span style={{ color: 'var(--success)' }}>➜</span> <span style={{ color: 'var(--text-primary)' }}>~</span> ready</p>
      </div>
    </div>
  );
}