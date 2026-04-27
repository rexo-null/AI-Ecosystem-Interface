use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};
use chrono::{DateTime, Utc};

/// Agent task status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Agent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: u8,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub steps: Vec<TaskStep>,
}

/// Task execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub executed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub can_execute_code: bool,
    pub can_access_filesystem: bool,
    pub can_run_commands: bool,
    pub can_use_network: bool,
    pub max_concurrent_tasks: usize,
}

/// Agent Module - autonomous task execution and self-improvement
pub struct AgentModule {
    tasks: Arc<RwLock<HashMap<String, AgentTask>>>,
    capabilities: Arc<RwLock<AgentCapabilities>>,
    module_id: String,
    active_tasks: Arc<RwLock<usize>>,
}

impl AgentModule {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(AgentCapabilities {
                can_execute_code: true,
                can_access_filesystem: true,
                can_run_commands: false, // Disabled for security
                can_use_network: false,  // Disabled for security
                max_concurrent_tasks: 3,
            })),
            module_id: "agent_module".to_string(),
            active_tasks: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new task
    pub async fn create_task(&self, description: String, priority: u8) -> anyhow::Result<String> {
        let task_id = format!("task_{}", chrono::Utc::now().timestamp_millis());
        let task = AgentTask {
            id: task_id.clone(),
            description,
            status: TaskStatus::Pending,
            priority,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            steps: Vec::new(),
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);

        info!("Created task: {}", task_id);
        Ok(task_id)
    }

    /// Start task execution
    pub async fn start_task(&self, task_id: &str) -> anyhow::Result<()> {
        let capabilities = self.capabilities.read().await;
        let active_count = *self.active_tasks.read().await;

        if active_count >= capabilities.max_concurrent_tasks {
            return Err(anyhow::anyhow!("Maximum concurrent tasks reached"));
        }

        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if matches!(task.status, TaskStatus::Pending) {
                task.status = TaskStatus::Running;
                task.started_at = Some(Utc::now());
                *self.active_tasks.write().await += 1;

                // Start task execution in background
                let task_id_clone = task_id.to_string();
                let tasks_clone = self.tasks.clone();
                let active_tasks_clone = self.active_tasks.clone();

                tokio::spawn(async move {
                    Self::execute_task_background(task_id_clone, tasks_clone, active_tasks_clone).await;
                });

                info!("Started task: {}", task_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Task is not in pending state"))
            }
        } else {
            Err(anyhow::anyhow!("Task not found: {}", task_id))
        }
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if matches!(task.status, TaskStatus::Running | TaskStatus::Pending) {
                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(Utc::now());
                *self.active_tasks.write().await = self.active_tasks.read().await.saturating_sub(1);

                info!("Cancelled task: {}", task_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Task cannot be cancelled"))
            }
        } else {
            Err(anyhow::anyhow!("Task not found: {}", task_id))
        }
    }

    /// Get task status
    pub async fn get_task(&self, task_id: &str) -> Option<AgentTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// Get all tasks
    pub async fn get_all_tasks(&self) -> Vec<AgentTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// Get active tasks
    pub async fn get_active_tasks(&self) -> Vec<AgentTask> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|t| matches!(t.status, TaskStatus::Running))
            .cloned()
            .collect()
    }

    /// Update agent capabilities
    pub async fn update_capabilities(&self, capabilities: AgentCapabilities) -> anyhow::Result<()> {
        let mut current = self.capabilities.write().await;
        *current = capabilities;
        info!("Updated agent capabilities");
        Ok(())
    }

    /// Get agent capabilities
    pub async fn get_capabilities(&self) -> AgentCapabilities {
        self.capabilities.read().await.clone()
    }

    /// Background task execution
    async fn execute_task_background(
        task_id: String,
        tasks: Arc<RwLock<HashMap<String, AgentTask>>>,
        active_tasks: Arc<RwLock<usize>>,
    ) {
        info!("Executing task in background: {}", task_id);

        // Simulate task execution with steps
        let steps = vec![
            "Analyzing task requirements",
            "Planning execution steps",
            "Executing planned actions",
            "Validating results",
            "Finalizing task",
        ];

        for (i, step_desc) in steps.iter().enumerate() {
            // Check if task was cancelled
            {
                let tasks_read = tasks.read().await;
                if let Some(task) = tasks_read.get(&task_id) {
                    if matches!(task.status, TaskStatus::Cancelled) {
                        info!("Task {} was cancelled during execution", task_id);
                        return;
                    }
                }
            }

            // Add step
            {
                let mut tasks_write = tasks.write().await;
                if let Some(task) = tasks_write.get_mut(&task_id) {
                    task.steps.push(TaskStep {
                        id: format!("step_{}", i + 1),
                        description: step_desc.to_string(),
                        status: TaskStatus::Running,
                        executed_at: Some(Utc::now()),
                        output: None,
                        error: None,
                    });
                }
            }

            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Complete step
            {
                let mut tasks_write = tasks.write().await;
                if let Some(task) = tasks_write.get_mut(&task_id) {
                    if let Some(step) = task.steps.last_mut() {
                        step.status = TaskStatus::Completed;
                        step.output = Some(format!("Completed: {}", step_desc));
                    }
                }
            }
        }

        // Complete task
        {
            let mut tasks_write = tasks.write().await;
            if let Some(task) = tasks_write.get_mut(&task_id) {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(Utc::now());
                task.result = Some("Task completed successfully".to_string());
            }
        }

        *active_tasks.write().await = active_tasks.read().await.saturating_sub(1);
        info!("Task completed: {}", task_id);
    }

    /// Self-improvement: analyze performance and suggest improvements
    pub async fn analyze_performance(&self) -> anyhow::Result<PerformanceReport> {
        let tasks = self.tasks.read().await;
        let total_tasks = tasks.len();
        let completed_tasks = tasks.values()
            .filter(|t| matches!(t.status, TaskStatus::Completed))
            .count();
        let failed_tasks = tasks.values()
            .filter(|t| matches!(t.status, TaskStatus::Failed))
            .count();

        let avg_execution_time = tasks.values()
            .filter(|t| t.completed_at.is_some() && t.started_at.is_some())
            .map(|t| {
                (t.completed_at.unwrap() - t.started_at.unwrap()).num_seconds() as f64
            })
            .sum::<f64>() / total_tasks as f64;

        let report = PerformanceReport {
            total_tasks,
            completed_tasks,
            failed_tasks,
            success_rate: if total_tasks > 0 { completed_tasks as f64 / total_tasks as f64 } else { 0.0 },
            avg_execution_time_seconds: avg_execution_time,
            recommendations: self.generate_recommendations(completed_tasks, failed_tasks, avg_execution_time).await,
        };

        Ok(report)
    }

    async fn generate_recommendations(&self, completed: usize, failed: usize, avg_time: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if failed > completed {
            recommendations.push("High failure rate detected. Consider improving error handling.".to_string());
        }

        if avg_time > 60.0 {
            recommendations.push("Tasks are taking too long. Consider optimizing execution pipeline.".to_string());
        }

        if completed == 0 {
            recommendations.push("No completed tasks yet. Focus on basic task execution first.".to_string());
        } else {
            recommendations.push("Performance looks good. Consider adding more complex task types.".to_string());
        }

        recommendations
    }
}

/// Performance analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub success_rate: f64,
    pub avg_execution_time_seconds: f64,
    pub recommendations: Vec<String>,
}

#[async_trait::async_trait]
impl super::ISKINModule for AgentModule {
    fn id(&self) -> &str {
        &self.module_id
    }

    fn name(&self) -> &str {
        "Autonomous Agent"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing Agent Module");

        // Create a sample task
        self.create_task(
            "Initialize autonomous agent capabilities".to_string(),
            5
        ).await?;

        info!("Agent Module initialized");
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        info!("Shutting down Agent Module");

        // Cancel all running tasks
        let active_tasks = self.get_active_tasks().await;
        for task in active_tasks {
            let _ = self.cancel_task(&task.id).await;
        }

        Ok(())
    }

    async fn execute(&self, command: &str, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match command {
            "create_task" => {
                let description = args.get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing description"))?;
                let priority = args.get("priority")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as u8;

                let task_id = self.create_task(description.to_string(), priority).await?;
                Ok(serde_json::json!({ "task_id": task_id }))
            }
            "start_task" => {
                let task_id = args.get("task_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?;

                self.start_task(task_id).await?;
                Ok(serde_json::json!({ "success": true }))
            }
            "cancel_task" => {
                let task_id = args.get("task_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?;

                self.cancel_task(task_id).await?;
                Ok(serde_json::json!({ "success": true }))
            }
            "get_task" => {
                let task_id = args.get("task_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?;

                if let Some(task) = self.get_task(task_id).await {
                    Ok(serde_json::to_value(task)?)
                } else {
                    Err(anyhow::anyhow!("Task not found"))
                }
            }
            "get_all_tasks" => {
                let tasks = self.get_all_tasks().await;
                Ok(serde_json::to_value(tasks)?)
            }
            "get_capabilities" => {
                let caps = self.get_capabilities().await;
                Ok(serde_json::to_value(caps)?)
            }
            "analyze_performance" => {
                let report = self.analyze_performance().await?;
                Ok(serde_json::to_value(report)?)
            }
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }
}