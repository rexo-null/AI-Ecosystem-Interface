import React, { useState } from 'react';
import { useEditorStore } from '../store';
import './TaskPanel.css';

export const TaskPanel: React.FC = () => {
  const { agentTasks, currentTaskId, updateTaskStatus, addAgentTask } = useEditorStore();
  const [newTaskDesc, setNewTaskDesc] = useState('');
  
  const handleAddTask = () => {
    if (newTaskDesc.trim()) {
      addAgentTask({
        description: newTaskDesc.trim(),
        status: 'pending',
        subtasks: [],
      });
      setNewTaskDesc('');
    }
  };
  
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleAddTask();
    }
  };
  
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed': return '✅';
      case 'in_progress': return '🔄';
      case 'failed': return '❌';
      default: return '⏳';
    }
  };
  
  return (
    <div className="task-panel">
      <div className="panel-header">
        <h3>Tasks</h3>
        <span className="task-count">{agentTasks.length}</span>
      </div>
      
      <div className="task-input-container">
        <input
          type="text"
          className="task-input"
          placeholder="Add new task..."
          value={newTaskDesc}
          onChange={(e) => setNewTaskDesc(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        <button 
          className="add-task-btn"
          onClick={handleAddTask}
          disabled={!newTaskDesc.trim()}
        >
          +
        </button>
      </div>
      
      <div className="task-list">
        {agentTasks.length === 0 ? (
          <div className="empty-state">No tasks yet</div>
        ) : (
          agentTasks.map((task) => (
            <div 
              key={task.id}
              className={`task-item ${currentTaskId === task.id ? 'current' : ''} status-${task.status}`}
              onClick={() => updateTaskStatus(task.id, task.status === 'completed' ? 'pending' : 'in_progress')}
            >
              <span className="task-status-icon">{getStatusIcon(task.status)}</span>
              <span className="task-description">{task.description}</span>
              <span className="task-subtasks-count">
                {task.subtasks.length > 0 && `${task.subtasks.length} subtasks`}
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default TaskPanel;