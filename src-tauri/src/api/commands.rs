use crate::core::LifecycleManager;
use crate::memory::KnowledgeBase;
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
