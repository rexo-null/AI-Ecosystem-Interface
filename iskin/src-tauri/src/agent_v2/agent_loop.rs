// Agent Loop: Main execution loop with retry logic, fallback, and phase transitions

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use thiserror::Error;
use std::time::Duration;

use super::state_machine::{AgentState, AgentPhase, StateTransitionError, ImpactReport, RiskLevel, RollbackPlan};
use super::tool_protocol::{ToolUseProtocol, ToolCall, ToolResult, ToolValidationError};
use super::context_compressor::ContextCompressor;

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_retry: u32,
    pub context_budget_limit: usize,
    pub tool_timeout_ms: u64,
    pub dry_run_for_risk_above: RiskLevel,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_retry: 3,
            context_budget_limit: 6000,
            tool_timeout_ms: 30000,
            dry_run_for_risk_above: RiskLevel::Medium,
        }
    }
}

/// Agent events for streaming to UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    PhaseChanged { old_phase: String, new_phase: String },
    ToolCalled { tool_name: String, arguments: String },
    ToolResult { tool_name: String, success: bool, output: Option<String> },
    Error { message: String, recoverable: bool },
    Retry { attempt: u32, max_retry: u32 },
    Compression { original_tokens: usize, compressed_tokens: usize },
    ImpactReport { affected_files: Vec<String>, risk_level: String },
    TaskComplete { task_id: String, success: bool },
}

/// Agent execution errors
#[derive(Error, Debug, Clone)]
pub enum AgentError {
    #[error("State transition failed: {0}")]
    StateTransition(#[from] StateTransitionError),
    
    #[error("Tool validation failed: {0}")]
    ToolValidation(#[from] ToolValidationError),
    
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    
    #[error("Tool execution timeout")]
    ToolTimeout,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Main agent loop executor
pub struct AgentLoop {
    state: AgentState,
    config: AgentConfig,
    tool_protocol: ToolUseProtocol,
    context_compressor: ContextCompressor,
    event_tx: Option<mpsc::Sender<AgentEvent>>,
}

impl AgentLoop {
    /// Create a new agent loop
    pub fn new(config: AgentConfig, event_tx: Option<mpsc::Sender<AgentEvent>>) -> Self {
        let state = AgentState::new(config.max_retry, config.context_budget_limit);
        let context_compressor = ContextCompressor::new(
            config.context_budget_limit,
            "You are ISKIN, an autonomous code agent. Follow the rules strictly.".to_string(),
        );

        Self {
            state,
            config,
            tool_protocol: ToolUseProtocol::new(),
            context_compressor,
            event_tx,
        }
    }

    /// Receive a new task
    pub async fn receive_task(&mut self, task: String) -> Result<(), AgentError> {
        self.state.transition_to(AgentPhase::Decompose)?;
        self.emit_event(AgentEvent::PhaseChanged {
            old_phase: "ReceiveTask".to_string(),
            new_phase: "Decompose".to_string(),
        });

        self.state.add_message("user".to_string(), task.clone());
        self.state.current_task_id = Some(format!("task_{}", chrono::Utc::now().timestamp()));

        Ok(())
    }

    /// Decompose task into steps (≤3 steps per cycle)
    pub async fn decompose_task(&mut self) -> Result<Vec<String>, AgentError> {
        // In production, this would call LLM for smart decomposition
        // For now, return a simple heuristic decomposition
        
        let steps = vec![
            "Analyze requirements".to_string(),
            "Implement solution".to_string(),
            "Verify results".to_string(),
        ];

        for step in &steps {
            self.state.add_message("assistant".to_string(), format!("Step: {}", step));
        }

        // Transition to ImpactAssessment
        self.state.transition_to(AgentPhase::ImpactAssessment)?;
        self.emit_event(AgentEvent::PhaseChanged {
            old_phase: "Decompose".to_string(),
            new_phase: "ImpactAssessment".to_string(),
        });

        Ok(steps)
    }

    /// Generate impact report before execution
    pub async fn assess_impact(&mut self, steps: &[String]) -> Result<ImpactReport, AgentError> {
        // In production, analyze actual file changes
        let report = ImpactReport {
            affected_files: vec!["src/main.rs".to_string()],
            doc_sync_needed: true,
            tests_to_run: vec!["cargo test".to_string()],
            rollback_plan: RollbackPlan {
                files_to_restore: vec!["src/main.rs".to_string()],
                commands_to_run: vec!["git checkout HEAD -- src/main.rs".to_string()],
                git_reset_point: Some("HEAD".to_string()),
            },
            risk_level: RiskLevel::Medium,
        };

        self.state.set_impact_report(report.clone());
        
        self.emit_event(AgentEvent::ImpactReport {
            affected_files: report.affected_files.clone(),
            risk_level: format!("{:?}", report.risk_level),
        });

        // Decide whether to dry run
        if report.risk_level > self.config.dry_run_for_risk_above {
            self.state.transition_to(AgentPhase::DryRun)?;
            self.emit_event(AgentEvent::PhaseChanged {
                old_phase: "ImpactAssessment".to_string(),
                new_phase: "DryRun".to_string(),
            });
        } else {
            self.state.transition_to(AgentPhase::Execute)?;
            self.emit_event(AgentEvent::PhaseChanged {
                old_phase: "ImpactAssessment".to_string(),
                new_phase: "Execute".to_string(),
            });
        }

        Ok(report)
    }

    /// Execute a tool call with validation and policy check
    pub async fn execute_tool(&mut self, tool_call: ToolCall) -> Result<ToolResult, AgentError> {
        // Validate tool call against schema
        self.tool_protocol.validate_tool_call(&tool_call)?;

        self.emit_event(AgentEvent::ToolCalled {
            tool_name: tool_call.name.clone(),
            arguments: format!("{:?}", tool_call.arguments),
        });

        // In production, actually execute the tool
        // For now, simulate success
        let result = ToolResult {
            call_id: tool_call.call_id.clone(),
            success: true,
            output: Some("Tool executed successfully".to_string()),
            error: None,
        };

        self.emit_event(AgentEvent::ToolResult {
            tool_name: tool_call.name.clone(),
            success: result.success,
            output: result.output.clone(),
        });

        Ok(result)
    }

    /// Verify execution results
    pub async fn verify_results(&mut self) -> Result<bool, AgentError> {
        self.state.transition_to(AgentPhase::Verify)?;
        self.emit_event(AgentEvent::PhaseChanged {
            old_phase: "Execute".to_string(),
            new_phase: "Verify".to_string(),
        });

        // In production, run tests and validation
        // For now, assume success
        let success = true;

        if !success {
            // Auto-rollback on failure
            if self.state.increment_retry() {
                return Err(AgentError::MaxRetriesExceeded);
            }
            
            self.emit_event(AgentEvent::Retry {
                attempt: self.state.retry_count,
                max_retry: self.state.max_retry,
            });
            
            // Rollback and retry
            self.state.transition_to(AgentPhase::ReceiveTask)?;
        } else {
            self.state.reset_retry();
            self.state.transition_to(AgentPhase::ArtifactSync)?;
            self.emit_event(AgentEvent::PhaseChanged {
                old_phase: "Verify".to_string(),
                new_phase: "ArtifactSync".to_string(),
            });
        }

        Ok(success)
    }

    /// Sync artifacts (documentation, plans)
    pub async fn sync_artifacts(&mut self) -> Result<(), AgentError> {
        // In production, update documentation based on changes
        self.state.project_state.doc_sync_needed = false;

        self.state.transition_to(AgentPhase::Commit)?;
        self.emit_event(AgentEvent::PhaseChanged {
            old_phase: "ArtifactSync".to_string(),
            new_phase: "Commit".to_string(),
        });

        Ok(())
    }

    /// Commit changes
    pub async fn commit_changes(&mut self) -> Result<(), AgentError> {
        // In production, create git commit
        self.state.transition_to(AgentPhase::QueueNext)?;
        self.emit_event(AgentEvent::PhaseChanged {
            old_phase: "Commit".to_string(),
            new_phase: "QueueNext".to_string(),
        });

        Ok(())
    }

    /// Queue next task or finish
    pub async fn queue_next(&mut self) -> Result<Option<String>, AgentError> {
        if self.state.task_queue.is_empty() {
            self.state.transition_to(AgentPhase::ReceiveTask)?;
            self.emit_event(AgentEvent::PhaseChanged {
                old_phase: "QueueNext".to_string(),
                new_phase: "ReceiveTask".to_string(),
            });
            Ok(None)
        } else {
            let next_task = self.state.task_queue.remove(0);
            self.state.transition_to(AgentPhase::Decompose)?;
            Ok(Some(next_task))
        }
    }

    /// Check if context compression is needed and perform it
    pub fn manage_context(&mut self) -> Option<usize> {
        if self.context_compressor.should_compress() {
            let summary = self.context_compressor.compress();
            self.state.context_budget_used = summary.compressed_token_count;
            
            self.emit_event(AgentEvent::Compression {
                original_tokens: summary.original_token_count,
                compressed_tokens: summary.compressed_token_count,
            });
            
            Some(summary.compressed_token_count)
        } else {
            None
        }
    }

    /// Get current state
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Get current phase
    pub fn current_phase(&self) -> AgentPhase {
        self.state.current_phase
    }

    /// Emit event to UI
    fn emit_event(&self, event: AgentEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.try_send(event);
        }
    }

    /// Add critical keyword to context compressor
    pub fn add_critical_keyword(&mut self, keyword: String) {
        self.context_compressor.add_critical_keyword(keyword);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_agent_loop_creation() {
        let config = AgentConfig::default();
        let agent = AgentLoop::new(config, None);
        
        assert_eq!(agent.current_phase(), AgentPhase::ReceiveTask);
        assert_eq!(agent.state().max_retry, 3);
    }

    #[tokio::test]
    async fn test_receive_task() {
        let config = AgentConfig::default();
        let mut agent = AgentLoop::new(config, None);
        
        let result = agent.receive_task("Create a new file".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(agent.current_phase(), AgentPhase::Decompose);
    }

    #[tokio::test]
    async fn test_decompose_task() {
        let config = AgentConfig::default();
        let mut agent = AgentLoop::new(config, None);
        agent.receive_task("Test task".to_string()).await.unwrap();
        
        let steps = agent.decompose_task().await;
        assert!(steps.is_ok());
        assert_eq!(steps.unwrap().len(), 3);
        assert_eq!(agent.current_phase(), AgentPhase::ImpactAssessment);
    }

    #[tokio::test]
    async fn test_assess_impact() {
        let config = AgentConfig::default();
        let mut agent = AgentLoop::new(config, None);
        agent.receive_task("Test task".to_string()).await.unwrap();
        agent.decompose_task().await.unwrap();
        
        let report = agent.assess_impact(&["step1".to_string()]).await;
        assert!(report.is_ok());
        assert_eq!(report.unwrap().risk_level, RiskLevel::Medium);
    }

    #[tokio::test]
    async fn test_execute_tool() {
        let config = AgentConfig::default();
        let mut agent = AgentLoop::new(config, None);
        
        let tool_call = ToolCall {
            name: "file_read".to_string(),
            arguments: std::collections::HashMap::from([
                ("path".to_string(), serde_json::json!("test.txt")),
            ]),
            call_id: "call_1".to_string(),
        };
        
        let result = agent.execute_tool(tool_call).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_verify_results() {
        let config = AgentConfig::default();
        let mut agent = AgentLoop::new(config, None);
        agent.state.transition_to(AgentPhase::Execute).unwrap();
        agent.state.set_impact_report(ImpactReport {
            affected_files: vec![],
            doc_sync_needed: false,
            tests_to_run: vec![],
            rollback_plan: RollbackPlan::default(),
            risk_level: RiskLevel::Safe,
        });
        
        let result = agent.verify_results().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(agent.current_phase(), AgentPhase::ArtifactSync);
    }

    #[tokio::test]
    async fn test_full_cycle() {
        let config = AgentConfig::default();
        let mut agent = AgentLoop::new(config, None);
        
        // Receive and decompose
        agent.receive_task("Test task".to_string()).await.unwrap();
        agent.decompose_task().await.unwrap();
        
        // Assess impact
        agent.assess_impact(&["step1".to_string()]).await.unwrap();
        
        // Verify (skipping actual execute for this test)
        agent.state.transition_to(AgentPhase::Verify).unwrap();
        agent.verify_results().await.unwrap();
        
        // Sync and commit
        agent.sync_artifacts().await.unwrap();
        agent.commit_changes().await.unwrap();
        
        // Queue next
        let next = agent.queue_next().await.unwrap();
        assert!(next.is_none()); // No tasks in queue
        assert_eq!(agent.current_phase(), AgentPhase::ReceiveTask);
    }

    #[test]
    fn test_context_management() {
        let config = AgentConfig::default();
        let mut agent = AgentLoop::new(config, None);
        
        // Add many messages to trigger compression
        for i in 0..50 {
            agent.context_compressor.add_message(
                "user".to_string(),
                format!("Message {}", i),
            );
        }
        
        let compressed = agent.manage_context();
        assert!(compressed.is_some());
    }
}
