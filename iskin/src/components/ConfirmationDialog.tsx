import React, { useState } from 'react';
import './ConfirmationDialog.css';

interface ConfirmationDialogProps {
  action: string;
  risk: 'low' | 'medium' | 'high' | 'critical';
  onConfirm: () => void;
  onCancel: () => void;
}

const riskLabels: Record<string, string> = {
  low: 'Low Risk',
  medium: 'Medium Risk',
  high: 'High Risk',
  critical: 'Critical Risk',
};

const riskIcons: Record<string, string> = {
  low: '✅',
  medium: '⚠️',
  high: '🔴',
  critical: '💀',
};

export const ConfirmationDialog: React.FC<ConfirmationDialogProps> = ({
  action,
  risk,
  onConfirm,
  onCancel,
}) => {
  const [confirmed, setConfirmed] = useState(false);
  
  const isHighRisk = risk === 'high' || risk === 'critical';
  
  const handleConfirm = () => {
    if (isHighRisk && !confirmed) {
      setConfirmed(true);
    } else {
      onConfirm();
    }
  };
  
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey || !isHighRisk || confirmed)) {
      handleConfirm();
    }
  };
  
  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div 
        className={`dialog-container risk-${risk}`}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <div className="dialog-header">
          <span className="risk-icon">{riskIcons[risk]}</span>
          <h3>Confirm Action</h3>
        </div>
        
        <div className="dialog-content">
          <p className="action-description">{action}</p>
          
          <div className={`risk-badge risk-${risk}`}>
            {riskLabels[risk]}
          </div>
          
          {isHighRisk && !confirmed && (
            <div className="warning-box">
              <p><strong>Warning:</strong> This action involves {risk} risk.</p>
              <p>Please type <code>confirm</code> to proceed:</p>
              <input
                type="text"
                className="confirm-input"
                placeholder="Type confirm..."
                onChange={(e) => {
                  if (e.target.value === 'confirm') {
                    setConfirmed(true);
                  }
                }}
                autoFocus
              />
            </div>
          )}
          
          {isHighRisk && confirmed && (
            <div className="confirmed-box">
              ✓ Action confirmed
            </div>
          )}
        </div>
        
        <div className="dialog-actions">
          <button 
            className="btn btn-cancel"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button 
            className={`btn btn-confirm risk-${risk}`}
            onClick={handleConfirm}
            disabled={isHighRisk && !confirmed}
          >
            {isHighRisk ? 'Execute' : 'Confirm'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default ConfirmationDialog;