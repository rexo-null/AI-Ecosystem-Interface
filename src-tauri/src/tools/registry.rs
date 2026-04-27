use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

/// Schema for a tool parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

/// Registered tool entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredTool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub handler_name: String,
    pub is_enabled: bool,
}

/// Tool Registry - manages available tools for the agent
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, RegisteredTool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new tool
    pub async fn register(&self, tool: RegisteredTool) -> Result<()> {
        info!("Registering tool: {} ({})", tool.name, tool.id);
        self.tools.write().await.insert(tool.id.clone(), tool);
        Ok(())
    }

    /// Unregister a tool
    pub async fn unregister(&self, tool_id: &str) -> Result<()> {
        self.tools
            .write()
            .await
            .remove(tool_id)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_id))?;
        info!("Unregistered tool: {}", tool_id);
        Ok(())
    }

    /// Get a tool by ID
    pub async fn get(&self, tool_id: &str) -> Option<RegisteredTool> {
        self.tools.read().await.get(tool_id).cloned()
    }

    /// List all registered tools
    pub async fn list_all(&self) -> Vec<RegisteredTool> {
        self.tools.read().await.values().cloned().collect()
    }

    /// List only enabled tools
    pub async fn list_enabled(&self) -> Vec<RegisteredTool> {
        self.tools
            .read()
            .await
            .values()
            .filter(|t| t.is_enabled)
            .cloned()
            .collect()
    }
}
