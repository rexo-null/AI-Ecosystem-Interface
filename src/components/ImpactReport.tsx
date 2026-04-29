import React from 'react';
import { useEditorStore } from '../store';
import './ImpactReport.css';

const riskColors: Record<string, string> = {
  low: '#4caf50',
  medium: '#ff9800',
  high: '#f44336',
  critical: '#9c27b0',
};

export const ImpactReport: React.FC = () => {
  const { impactReport } = useEditorStore();
  
  if (!impactReport) {
    return (
      <div className="impact-report empty">
        <div className="empty-state">
          <span className="empty-icon">📊</span>
          <p>No impact assessment yet</p>
          <p className="empty-hint">Run Impact Assessment in agent phase</p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="impact-report">
      <div className="report-header">
        <h3>Impact Assessment</h3>
        <span 
          className="risk-badge"
          style={{ backgroundColor: riskColors[impactReport.risk_level] }}
        >
          {impactReport.risk_level.toUpperCase()}
        </span>
      </div>
      
      <div className="report-section">
        <h4>Affected Files ({impactReport.affected_files.length})</h4>
        <div className="file-list">
          {impactReport.affected_files.map((file, idx) => (
            <div key={idx} className="file-item">
              <span className="file-icon">📄</span>
              <span className="file-path">{file}</span>
            </div>
          ))}
        </div>
      </div>
      
      {impactReport.tests_to_run.length > 0 && (
        <div className="report-section">
          <h4>Tests to Run ({impactReport.tests_to_run.length})</h4>
          <div className="test-list">
            {impactReport.tests_to_run.map((test, idx) => (
              <div key={idx} className="test-item">
                <span className="test-icon">🧪</span>
                <span className="test-path">{test}</span>
              </div>
            ))}
          </div>
        </div>
      )}
      
      <div className="report-section">
        <h4>Rollback Plan</h4>
        <div className="rollback-list">
          {impactReport.rollback_plan.steps.map((step, idx) => (
            <div key={idx} className="rollback-item">
              <span className="step-number">{idx + 1}</span>
              <span className="step-description">{step}</span>
            </div>
          ))}
        </div>
        <div className="rollback-time">
          Estimated time: {impactReport.rollback_plan.estimated_time}s
        </div>
      </div>
      
      {impactReport.doc_sync_needed && (
        <div className="doc-sync-notice">
          <span className="notice-icon">📝</span>
          Documentation sync required after execution
        </div>
      )}
    </div>
  );
};

export default ImpactReport;