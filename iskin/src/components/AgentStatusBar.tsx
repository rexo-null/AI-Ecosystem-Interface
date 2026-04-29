import React from 'react';
import { useEditorStore } from '../store';
import './AgentStatusBar.css';

const phaseIcons: Record<string, string> = {
  ReceiveTask: '📥',
  Decompose: '🔀',
  ImpactAssessment: '📊',
  DryRun: '🧪',
  Execute: '⚡',
  Verify: '✅',
  ArtifactSync: '📝',
  Commit: '💾',
  QueueNext: '⏭',
};

const phaseLabels: Record<string, string> = {
  ReceiveTask: 'Receiving Task',
  Decompose: 'Decomposing',
  ImpactAssessment: 'Impact Assessment',
  DryRun: 'Dry Run',
  Execute: 'Executing',
  Verify: 'Verifying',
  ArtifactSync: 'Syncing Artifacts',
  Commit: 'Committing',
  QueueNext: 'Queue Next',
};

export const AgentStatusBar: React.FC = () => {
  const { agentPhase, agentActive } = useEditorStore();
  
  return (
    <div className="agent-status-bar">
      <div className={`status-indicator ${agentActive ? 'active' : 'idle'}`}>
        <span className="status-dot" />
        <span className="status-text">
          {agentActive ? 'Active' : 'Idle'}
        </span>
      </div>
      
      <div className="phase-display">
        <span className="phase-icon">{phaseIcons[agentPhase]}</span>
        <span className="phase-label">{phaseLabels[agentPhase]}</span>
      </div>
      
      {agentActive && (
        <div className="phase-progress">
          <div 
            className="progress-bar"
            style={{ 
              width: `${((Object.keys(phaseLabels).indexOf(agentPhase) + 1) / Object.keys(phaseLabels).length) * 100}%` 
            }} 
          />
        </div>
      )}
    </div>
  );
};

export default AgentStatusBar;