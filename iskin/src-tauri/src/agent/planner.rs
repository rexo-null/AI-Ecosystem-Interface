use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

/// Unique task identifier
pub type TaskId = Uuid;

/// Planning error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlannerError {
    DecompositionFailed(String),
    InvalidTaskFormat(String),
    MaxSubtasksExceeded,
    ParseError(String),
}

impl std::fmt::Display for PlannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecompositionFailed(msg) => write!(f, "Decomposition failed: {}", msg),
            Self::InvalidTaskFormat(msg) => write!(f, "Invalid task format: {}", msg),
            Self::MaxSubtasksExceeded => write!(f, "Maximum number of subtasks exceeded"),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for PlannerError {}

/// Language types for validation rules
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    Rust,
    Toml,
    Json,
    Yaml,
    Markdown,
    Shell,
    Python,
    TypeScript,
}

/// Output format for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
    CodeBlock(String), // language hint
}

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    SyntaxCheck(Language),
    CompilationCheck,
    TestPass,
    FileExists,
    RegexMatch(String),
    Custom(String),
}

/// Atomic task step types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStep {
    /// Generate or edit code in a file
    GenerateCode {
        id: TaskId,
        file_path: String,
        description: String,
        start_line: Option<usize>,
        end_line: Option<usize>,
    },
    /// Run a command in terminal
    RunCommand {
        id: TaskId,
        command: String,
        cwd: Option<String>,
        timeout_sec: u64,
        expected_exit_code: Option<i32>,
    },
    /// Validate an artifact
    Validate {
        id: TaskId,
        artifact_path: String,
        rules: Vec<ValidationRule>,
    },
    /// Fix an error (recursive subtask)
    FixError {
        id: TaskId,
        error_log: String,
        original_step_id: TaskId,
        suggested_fix: Option<String>,
    },
    /// Aggregate results
    Aggregate {
        id: TaskId,
        output_format: OutputFormat,
        include_steps: Vec<TaskId>,
    },
}

impl TaskStep {
    pub fn id(&self) -> TaskId {
        match self {
            Self::GenerateCode { id, .. } => *id,
            Self::RunCommand { id, .. } => *id,
            Self::Validate { id, .. } => *id,
            Self::FixError { id, .. } => *id,
            Self::Aggregate { id, .. } => *id,
        }
    }
}

/// Planner configuration
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub max_subtasks: usize,
    pub prefer_atomic: bool,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_subtasks: 20,
            prefer_atomic: true,
        }
    }
}

/// Task decomposition and planning
pub struct Planner {
    config: PlannerConfig,
}

impl Planner {
    /// Create a new planner
    pub fn new(config: PlannerConfig) -> Self {
        Self { config }
    }

    /// Decompose a task into atomic steps
    /// 
    /// Algorithm:
    /// 1. If task is simple (single action) → return one step
    /// 2. If complex → apply heuristic decomposition
    /// 3. Validate structure and return steps
    pub fn decompose(&self, task: &str) -> Result<Vec<TaskStep>, PlannerError> {
        // Simple heuristic-based decomposition (Шаг 1 baseline)
        // In production, this would call LLM for smart decomposition
        
        let steps = self.decompose_heuristic(task)?;
        
        if steps.len() > self.config.max_subtasks {
            return Err(PlannerError::MaxSubtasksExceeded);
        }
        
        Ok(steps)
    }

    /// Heuristic decomposition (temporary, for testing)
    fn decompose_heuristic(&self, task: &str) -> Result<Vec<TaskStep>, PlannerError> {
        let mut steps = Vec::new();
        
        // Detect common patterns and create steps accordingly
        let task_lower = task.to_lowercase();
        
        // Pattern: "create file"
        if task_lower.contains("create") && task_lower.contains("file") {
            let file_path = self.extract_file_path(task)
                .unwrap_or_else(|| "output.txt".to_string());
            
            steps.push(TaskStep::GenerateCode {
                id: Uuid::new_v4(),
                file_path: file_path.clone(),
                description: task.to_string(),
                start_line: None,
                end_line: None,
            });
            
            // Add validation step
            steps.push(TaskStep::Validate {
                id: Uuid::new_v4(),
                artifact_path: file_path,
                rules: vec![ValidationRule::FileExists],
            });
        } else {
            // Default: single GenerateCode step
            steps.push(TaskStep::GenerateCode {
                id: Uuid::new_v4(),
                file_path: "output.txt".to_string(),
                description: task.to_string(),
                start_line: None,
                end_line: None,
            });
        }
        
        Ok(steps)
    }

    /// Extract file path from task description (heuristic)
    fn extract_file_path(&self, task: &str) -> Option<String> {
        // Look for patterns like "file:path/to/file" or quoted paths
        if let Some(start) = task.find("file:") {
            let rest = &task[start + 5..];
            if let Some(end) = rest.find(|c: char| c.is_whitespace()) {
                return Some(rest[..end].to_string());
            } else {
                return Some(rest.to_string());
            }
        }
        None
    }

    /// Create a fix step for a failed task
    pub fn create_fix_step(
        &self,
        error_msg: &str,
        original_step: &TaskStep,
    ) -> TaskStep {
        TaskStep::FixError {
            id: Uuid::new_v4(),
            error_log: error_msg.to_string(),
            original_step_id: original_step.id(),
            suggested_fix: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_planner() {
        let config = PlannerConfig::default();
        let planner = Planner::new(config);
        assert!(planner.config.max_subtasks > 0);
    }

    #[test]
    fn test_decompose_simple_task() {
        let planner = Planner::new(PlannerConfig::default());
        let task = "create a simple output";
        
        let steps = planner.decompose(task).expect("Decomposition failed");
        assert!(!steps.is_empty());
    }

    #[test]
    fn test_decompose_file_creation() {
        let planner = Planner::new(PlannerConfig::default());
        let task = "create file:src/test.rs with Rust code";
        
        let steps = planner.decompose(task).expect("Decomposition failed");
        assert!(steps.len() >= 2); // GenerateCode + Validate
        
        // Check first step is GenerateCode
        match &steps[0] {
            TaskStep::GenerateCode { file_path, .. } => {
                assert_eq!(file_path, "src/test.rs");
            }
            _ => panic!("Expected GenerateCode step"),
        }
    }

    #[test]
    fn test_create_fix_step() {
        let planner = Planner::new(PlannerConfig::default());
        let original = TaskStep::GenerateCode {
            id: Uuid::new_v4(),
            file_path: "test.rs".to_string(),
            description: "test".to_string(),
            start_line: None,
            end_line: None,
        };
        
        let fix_step = planner.create_fix_step("Compilation error", &original);
        
        match fix_step {
            TaskStep::FixError { error_log, original_step_id, .. } => {
                assert!(error_log.contains("Compilation error"));
                assert_eq!(original_step_id, original.id());
            }
            _ => panic!("Expected FixError step"),
        }
    }

    #[test]
    fn test_max_subtasks_limit() {
        let mut config = PlannerConfig::default();
        config.max_subtasks = 1;
        let planner = Planner::new(config);
        
        let task = "create file:a.rs; create file:b.rs; create file:c.rs";
        let result = planner.decompose(task);
        
        // Should succeed or fail gracefully
        match result {
            Ok(steps) => assert!(steps.len() <= 1),
            Err(PlannerError::MaxSubtasksExceeded) => {} // Expected
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}
