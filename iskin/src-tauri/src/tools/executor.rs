#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use log::{info, warn};

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub tool_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Tool Executor - runs tools in a controlled environment
pub struct ToolExecutor;

impl ToolExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a tool by handler name with given arguments
    pub async fn execute(
        &self,
        tool_id: &str,
        handler_name: &str,
        args: serde_json::Value,
    ) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();
        info!("Executing tool: {} (handler: {})", tool_id, handler_name);

        let result = match handler_name {
            "file_read" => self.handle_file_read(&args).await,
            "file_write" => self.handle_file_write(&args).await,
            "shell_exec" => self.handle_shell_exec(&args).await,
            _ => {
                warn!("Unknown handler: {}", handler_name);
                Err(anyhow::anyhow!("Unknown handler: {}", handler_name))
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(output) => Ok(ExecutionResult {
                tool_id: tool_id.to_string(),
                success: true,
                output,
                error: None,
                duration_ms,
            }),
            Err(e) => Ok(ExecutionResult {
                tool_id: tool_id.to_string(),
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                duration_ms,
            }),
        }
    }

    async fn handle_file_read(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let content = tokio::fs::read_to_string(path).await?;
        Ok(serde_json::json!({ "content": content }))
    }

    async fn handle_file_write(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

        tokio::fs::write(path, content).await?;
        Ok(serde_json::json!({ "written": true }))
    }

    async fn handle_shell_exec(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await?;

        Ok(serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "exit_code": output.status.code(),
        }))
    }
}
