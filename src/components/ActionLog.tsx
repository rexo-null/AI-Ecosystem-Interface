import React from 'react';
import { useEditorStore } from '../store';
import './ActionLog.css';

export const ActionLog: React.FC = () => {
  const { agentActions } = useEditorStore();
  
  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', { 
      hour12: false, 
      hour: '2-digit', 
      minute: '2-digit',
      second: '2-digit' 
    });
  };
  
  const formatDuration = (ms?: number) => {
    if (!ms) return '';
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };
  
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed': return '✅';
      case 'running': return '🔄';
      case 'failed': return '❌';
      default: return '⏳';
    }
  };
  
  return (
    <div className="action-log">
      <div className="panel-header">
        <h3>Action Log</h3>
        <span className="action-count">{agentActions.length}</span>
      </div>
      
      <div className="log-list">
        {agentActions.length === 0 ? (
          <div className="empty-state">No actions yet</div>
        ) : (
          agentActions.map((action) => (
            <div key={action.id} className={`log-item status-${action.status}`}>
              <div className="log-row">
                <span className="status-icon">{getStatusIcon(action.status)}</span>
                <span className="phase-tag">{action.phase}</span>
                <span className="timestamp">{formatTime(action.timestamp)}</span>
              </div>
              <div className="log-description">{action.description}</div>
              {action.duration_ms !== undefined && (
                <div className="log-duration">
                  Duration: {formatDuration(action.duration_ms)}
                </div>
              )}
              {action.error && (
                <div className="log-error">{action.error}</div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default ActionLog;