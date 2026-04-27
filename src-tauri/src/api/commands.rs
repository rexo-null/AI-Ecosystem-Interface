use crate::core::LifecycleManager;
use crate::memory::KnowledgeBase;
use crate::llm::{LLMEngine, ChatRequest, Message, LLMResponse};
use crate::modules::{ToolModule, MemoryModule, AgentModule};
use serde::{Deserialize, Serialize};

/// Simple ping command to test connectivity
#[tauri::command]
pub fn ping() -> String {
    "ISKIN is alive!".to_string()
}

/// List all active dynamic modules
#[tauri::command]
pub async fn list_modules(lifecycle: tauri::State<'_, LifecycleManager>) -> Vec<crate::core::lifecycle::DynamicModuleInfo> {
    lifecycle.list_modules().await
}

/// Get knowledge base entries
#[derive(Debug, Deserialize)]
pub struct GetKnowledgeParams {
    #[serde(default)]
    pub memory_type: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeEntryResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub memory_type: String,
    pub tags: Vec<String>,
    pub priority: u8,
}

#[tauri::command]
pub async fn get_knowledge_entries(
    kb: tauri::State<'_, KnowledgeBase>,
    params: Option<GetKnowledgeParams>,
) -> Result<Vec<KnowledgeEntryResponse>, String> {
    let memory_type = params.and_then(|p| p.memory_type).and_then(|t| {
        match t.as_str() {
            "Constitution" => Some(crate::memory::knowledge_base::MemoryType::Constitution),
            "Protocol" => Some(crate::memory::knowledge_base::MemoryType::Protocol),
            "Pattern" => Some(crate::memory::knowledge_base::MemoryType::Pattern),
            "UserRule" => Some(crate::memory::knowledge_base::MemoryType::UserRule),
            "ToolDefinition" => Some(crate::memory::knowledge_base::MemoryType::ToolDefinition),
            _ => None,
        }
    });
    
    let entries = kb.search_entries(memory_type, params.and_then(|p| p.tags)).await;
    
    Ok(entries.into_iter().map(|e| KnowledgeEntryResponse {
        id: e.id,
        title: e.title,
        content: e.content,
        memory_type: format!("{:?}", e.memory_type),
        tags: e.tags,
        priority: e.priority,
    }).collect())
}

// More commands will be added as development progresses:
// - File system operations (read, write, delete)
// - Terminal execution
// - LLM chat
// - Sandbox management
// - Module hot-reload

/// Chat with LLM
#[tauri::command]
pub async fn chat_with_llm(
    llm: tauri::State<'_, LLMEngine>,
    user_message: String,
    system_prompt: Option<String>,
) -> Result<LLMResponse, String> {
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: user_message,
        }
    ];

    let request = ChatRequest {
        messages,
        temperature: Some(0.7),
        max_tokens: Some(2048),
        system_prompt,
    };

    llm.chat(request)
        .await
        .map_err(|e| e.to_string())
}

/// Execute a tool
#[tauri::command]
pub async fn execute_tool(
    tool_module: tauri::State<'_, ToolModule>,
    tool_id: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    tool_module.execute_tool(&tool_id, args)
        .await
        .map_err(|e| e.to_string())
}

/// Manage memory operations
#[tauri::command]
pub async fn manage_memory(
    memory_module: tauri::State<'_, MemoryModule>,
    command: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    memory_module.execute(&command, args)
        .await
        .map_err(|e| e.to_string())
}

/// Manage agent tasks
#[tauri::command]
pub async fn manage_agent_tasks(
    agent_module: tauri::State<'_, AgentModule>,
    command: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    agent_module.execute(&command, args)
        .await
        .map_err(|e| e.to_string())
}
