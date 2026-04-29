use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

/// Tool definition for the tool registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub handler: String, // Function name to call
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

/// Tool Module - manages external tools and integrations
pub struct ToolModule {
    tools: Arc<RwLock<HashMap<String, ToolDefinition>>>,
    module_id: String,
}

impl ToolModule {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            module_id: "tool_module".to_string(),
        }
    }

    /// Register a new tool
    pub async fn register_tool(&self, tool: ToolDefinition) -> anyhow::Result<()> {
        let tool_name = tool.name.clone();
        let mut tools = self.tools.write().await;
        tools.insert(tool.id.clone(), tool);
        info!("Registered tool: {}", tool_name);
        Ok(())
    }

    /// Execute a tool
    pub async fn execute_tool(&self, tool_id: &str, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let tools = self.tools.read().await;
        let tool = tools.get(tool_id)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_id))?;

        info!("Executing tool: {} with args: {:?}", tool.name, args);

        // This is a placeholder - in real implementation, this would call
        // the actual tool handler based on tool.handler
        match tool.handler.as_str() {
            "file_read" => self.handle_file_read(args).await,
            "file_write" => self.handle_file_write(args).await,
            "run_command" => self.handle_run_command(args).await,
            _ => Err(anyhow::anyhow!("Unknown tool handler: {}", tool.handler)),
        }
    }

    /// Get all registered tools
    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    // Tool handlers
    async fn handle_file_read(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        match std::fs::read_to_string(path) {
            Ok(content) => Ok(ToolResult {
                success: true,
                output: serde_json::json!({ "content": content }),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn handle_file_write(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

        match std::fs::write(path, content) {
            Ok(_) => Ok(ToolResult {
                success: true,
                output: serde_json::json!({ "written": true }),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn handle_run_command(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

        // Security check - only allow safe commands
        if command.contains("rm") || command.contains("del") || command.contains("format") {
            return Ok(ToolResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some("Dangerous command blocked".to_string()),
            });
        }

        match std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                Ok(ToolResult {
                    success: output.status.success(),
                    output: serde_json::json!({
                        "stdout": stdout,
                        "stderr": stderr,
                        "exit_code": output.status.code()
                    }),
                    error: None,
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }
}

#[async_trait::async_trait]
impl super::ISKINModule for ToolModule {
    fn id(&self) -> &str {
        &self.module_id
    }

    fn name(&self) -> &str {
        "Tool Registry"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing Tool Module");

        // Register default tools
        let file_read_tool = ToolDefinition {
            id: "file_read".to_string(),
            name: "File Read".to_string(),
            description: "Read content from a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to read" }
                },
                "required": ["path"]
            }),
            handler: "file_read".to_string(),
        };

        let file_write_tool = ToolDefinition {
            id: "file_write".to_string(),
            name: "File Write".to_string(),
            description: "Write content to a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to write" },
                    "content": { "type": "string", "description": "Content to write" }
                },
                "required": ["path", "content"]
            }),
            handler: "file_write".to_string(),
        };

        let run_command_tool = ToolDefinition {
            id: "run_command".to_string(),
            name: "Run Command".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Command to execute" }
                },
                "required": ["command"]
            }),
            handler: "run_command".to_string(),
        };

        self.register_tool(file_read_tool).await?;
        self.register_tool(file_write_tool).await?;
        self.register_tool(run_command_tool).await?;

        info!("Tool Module initialized with {} tools", self.tools.read().await.len());
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        info!("Shutting down Tool Module");
        Ok(())
    }

    async fn execute(&self, command: &str, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match command {
            "list_tools" => {
                let tools = self.list_tools().await;
                Ok(serde_json::to_value(tools)?)
            }
            "execute_tool" => {
                let tool_id = args.get("tool_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing tool_id"))?;

                let tool_args = args.get("args")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                let result = self.execute_tool(tool_id, tool_args).await?;
                Ok(serde_json::to_value(result)?)
            }
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }
}