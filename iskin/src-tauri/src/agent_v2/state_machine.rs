// Agent State Machine with 9 phases of task lifecycle
// ReceiveTask → Decompose → ImpactAssessment → DryRun → Execute → Verify → ArtifactSync → Commit → QueueNext

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Agent phase enum with strict transition rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPhase {
    /// Initial state: receiving and parsing the task
    ReceiveTask,
    /// Breaking down the task into atomic steps (≤3 steps per cycle)
    Decompose,
    /// Analyzing impact before execution (required before Execute)
    ImpactAssessment,
    /// Running actions in sandbox first for risky operations
    DryRun,
    /// Executing the actual actions
    Execute,
    /// Verifying results (auto-rollback on failure)
    Verify,
    /// Syncing documentation and artifacts (mandatory after Execute)
    ArtifactSync,
    /// Committing changes
    Commit,
    /// Queuing next task from decomposition
    QueueNext,
}

impl fmt::Display for AgentPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReceiveTask => write!(f, "ReceiveTask"),
            Self::Decompose => write!(f, "Decompose"),
            Self::ImpactAssessment => write!(f, "ImpactAssessment"),
            Self::DryRun => write!(f, "DryRun"),
            Self::Execute => write!(f, "Execute"),
            Self::Verify => write!(f, "Verify"),
            Self::ArtifactSync => write!(f, "ArtifactSync"),
            Self::Commit => write!(f, "Commit"),
            Self::QueueNext => write!(f, "QueueNext"),
        }
    }
}

/// State transition errors
#[derive(Error, Debug, Clone)]
pub enum StateTransitionError {
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: AgentPhase, to: AgentPhase },
    #[error("Transition blocked: {reason}")]
    Blocked { reason: String },
}

/// Agent state containing current phase, task queue, history, and project state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub current_phase: AgentPhase,
    pub task_queue: Vec<String>,
    pub conversation_history: Vec<ConversationMessage>,
    pub project_state: ProjectState,
    pub retry_count: u32,
    pub max_retry: u32,
    pub current_task_id: Option<String>,
    pub impact_report: Option<ImpactReport>,
    pub context_budget_used: usize,
    pub context_budget_limit: usize,
}

/// Conversation message for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Project state snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectState {
    pub modified_files: Vec<String>,
    pub created_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub tests_to_run: Vec<String>,
    pub doc_sync_needed: bool,
}

/// Impact report generated during ImpactAssessment phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub affected_files: Vec<String>,
    pub doc_sync_needed: bool,
    pub tests_to_run: Vec<String>,
    pub rollback_plan: RollbackPlan,
    pub risk_level: RiskLevel,
}

/// Rollback plan for recovery
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub files_to_restore: Vec<String>,
    pub commands_to_run: Vec<String>,
    pub git_reset_point: Option<String>,
}

/// Risk level for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Medium,
    High,
    Dangerous,
}

impl AgentState {
    /// Create a new agent state with initial configuration
    pub fn new(max_retry: u32, context_budget_limit: usize) -> Self {
        Self {
            current_phase: AgentPhase::ReceiveTask,
            task_queue: Vec::new(),
            conversation_history: Vec::new(),
            project_state: ProjectState::default(),
            retry_count: 0,
            max_retry,
            current_task_id: None,
            impact_report: None,
            context_budget_used: 0,
            context_budget_limit,
        }
    }

    /// Transition to a new phase with validation
    pub fn transition_to(&mut self, new_phase: AgentPhase) -> Result<(), StateTransitionError> {
        let old_phase = self.current_phase;
        
        // Validate transition
        if !Self::is_valid_transition(old_phase, new_phase) {
            return Err(StateTransitionError::InvalidTransition {
                from: old_phase,
                to: new_phase,
            });
        }

        // Check mandatory rules
        match (old_phase, new_phase) {
            // IMPACT_BEFORE_ACTION: must have impact report before Execute
            (_, AgentPhase::Execute) => {
                if self.impact_report.is_none() {
                    return Err(StateTransitionError::Blocked {
                        reason: "IMPACT_BEFORE_ACTION: ImpactReport required before Execute".to_string(),
                    });
                }
            }
            // VERIFY_OR_ROLLBACK: if Verify fails, must rollback
            (AgentPhase::Verify, AgentPhase::Execute) => {
                // Allow re-execution only after rollback
            }
            _ => {}
        }

        self.current_phase = new_phase;
        Ok(())
    }

    /// Check if transition is valid according to state machine rules
    fn is_valid_transition(from: AgentPhase, to: AgentPhase) -> bool {
        match from {
            AgentPhase::ReceiveTask => matches!(to, AgentPhase::Decompose),
            AgentPhase::Decompose => matches!(to, AgentPhase::ImpactAssessment | AgentPhase::QueueNext),
            AgentPhase::ImpactAssessment => matches!(to, AgentPhase::DryRun | AgentPhase::Execute),
            AgentPhase::DryRun => matches!(to, AgentPhase::Execute | AgentPhase::ImpactAssessment),
            AgentPhase::Execute => matches!(to, AgentPhase::Verify | AgentPhase::ArtifactSync),
            AgentPhase::Verify => matches!(to, AgentPhase::ArtifactSync | AgentPhase::ReceiveTask), // rollback
            AgentPhase::ArtifactSync => matches!(to, AgentPhase::Commit),
            AgentPhase::Commit => matches!(to, AgentPhase::QueueNext | AgentPhase::ReceiveTask),
            AgentPhase::QueueNext => matches!(to, AgentPhase::ReceiveTask | AgentPhase::Decompose),
        }
    }

    /// Reset retry count on success
    pub fn reset_retry(&mut self) {
        self.retry_count = 0;
    }

    /// Increment retry count, return true if exceeded
    pub fn increment_retry(&mut self) -> bool {
        self.retry_count += 1;
        self.retry_count > self.max_retry
    }

    /// Add a message to conversation history
    pub fn add_message(&mut self, role: String, content: String) {
        self.conversation_history.push(ConversationMessage {
            role,
            content,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Check if context budget is exceeded
    pub fn is_context_budget_exceeded(&self) -> bool {
        self.context_budget_used > self.context_budget_limit
    }

    /// Set impact report
    pub fn set_impact_report(&mut self, report: ImpactReport) {
        self.impact_report = Some(report);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_creation() {
        let state = AgentState::new(3, 6000);
        assert_eq!(state.current_phase, AgentPhase::ReceiveTask);
        assert_eq!(state.max_retry, 3);
        assert_eq!(state.context_budget_limit, 6000);
    }

    #[test]
    fn test_valid_transitions() {
        let mut state = AgentState::new(3, 6000);
        
        // ReceiveTask → Decompose
        assert!(state.transition_to(AgentPhase::Decompose).is_ok());
        
        // Decompose → ImpactAssessment
        assert!(state.transition_to(AgentPhase::ImpactAssessment).is_ok());
        
        // Set impact report before Execute
        state.set_impact_report(ImpactReport {
            affected_files: vec!["test.rs".to_string()],
            doc_sync_needed: false,
            tests_to_run: vec![],
            rollback_plan: RollbackPlan::default(),
            risk_level: RiskLevel::Safe,
        });
        
        // ImpactAssessment → DryRun
        assert!(state.transition_to(AgentPhase::DryRun).is_ok());
    }

    #[test]
    fn test_invalid_transition_without_impact() {
        let mut state = AgentState::new(3, 6000);
        state.transition_to(AgentPhase::Decompose).unwrap();
        state.transition_to(AgentPhase::ImpactAssessment).unwrap();
        
        // Try to execute without impact report
        let result = state.transition_to(AgentPhase::Execute);
        assert!(result.is_err());
        
        // Now set impact report and try again
        state.set_impact_report(ImpactReport {
            affected_files: vec![],
            doc_sync_needed: false,
            tests_to_run: vec![],
            rollback_plan: RollbackPlan::default(),
            risk_level: RiskLevel::Safe,
        });
        
        assert!(state.transition_to(AgentPhase::Execute).is_ok());
    }

    #[test]
    fn test_retry_count() {
        let mut state = AgentState::new(2, 6000);
        
        assert!(!state.increment_retry()); // 1
        assert!(!state.increment_retry()); // 2
        assert!(state.increment_retry());  // 3 > max_retry
        
        state.reset_retry();
        assert_eq!(state.retry_count, 0);
    }

    #[test]
    fn test_context_budget() {
        let mut state = AgentState::new(3, 6000);
        assert!(!state.is_context_budget_exceeded());
        
        state.context_budget_used = 7000;
        assert!(state.is_context_budget_exceeded());
    }

    #[test]
    fn test_add_message() {
        let mut state = AgentState::new(3, 6000);
        state.add_message("user".to_string(), "test task".to_string());
        assert_eq!(state.conversation_history.len(), 1);
        assert_eq!(state.conversation_history[0].role, "user");
    }
}
