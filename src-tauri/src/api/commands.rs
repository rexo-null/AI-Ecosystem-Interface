use crate::core::LifecycleManager;
use crate::memory::{KnowledgeBase, SemanticIndexer, RulesEngine, VectorStore};
use crate::memory::knowledge_base::{MemoryType, MemoryEntry, SearchOptions, KnowledgeBaseStats};
use crate::memory::indexer::{IndexEntry, IndexStats, CodeSymbol, Language, SymbolKind};
use crate::memory::rules_engine::{Rule, RuleUpdate, EvaluationResult, RulePriority, ConditionType, RuleAction};
use crate::memory::vector_store::SearchResult as VectorSearchResult;
use crate::llm::{LLMEngine, ChatRequest, Message, LLMResponse};
use crate::modules::{ToolModule, MemoryModule, AgentModule};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// Core Commands
// ============================================================

/// Simple ping command to test connectivity
#[tauri::command]
pub fn ping() -> String {
    "ISKIN is alive!".to_string()
}

/// List all active dynamic modules
#[tauri::command]
pub async fn list_modules(
    lifecycle: tauri::State<'_, LifecycleManager>,
) -> Result<Vec<crate::core::lifecycle::DynamicModuleInfo>, String> {
    Ok(lifecycle.list_modules().await)
}

// ============================================================
// Knowledge Base Commands
// ============================================================

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
    pub access_count: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<MemoryEntry> for KnowledgeEntryResponse {
    fn from(e: MemoryEntry) -> Self {
        Self {
            id: e.id,
            title: e.title,
            content: e.content,
            memory_type: e.memory_type.as_str().to_string(),
            tags: e.tags,
            priority: e.priority,
            access_count: e.access_count,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[tauri::command]
pub async fn get_knowledge_entries(
    kb: tauri::State<'_, KnowledgeBase>,
    params: Option<GetKnowledgeParams>,
) -> Result<Vec<KnowledgeEntryResponse>, String> {
    let (memory_type_str, tags) = match params {
        Some(p) => (p.memory_type, p.tags),
        None => (None, None),
    };

    let memory_type = memory_type_str.and_then(|t| MemoryType::from_str(&t));
    let entries = kb.search_entries(memory_type, tags).await;

    Ok(entries.into_iter().map(KnowledgeEntryResponse::from).collect())
}

/// Add a new knowledge entry
#[derive(Debug, Deserialize)]
pub struct AddKnowledgeParams {
    pub title: String,
    pub content: String,
    pub memory_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_priority")]
    pub priority: u8,
}

fn default_priority() -> u8 { 5 }

#[tauri::command]
pub async fn add_knowledge_entry(
    kb: tauri::State<'_, KnowledgeBase>,
    params: AddKnowledgeParams,
) -> Result<String, String> {
    let memory_type = MemoryType::from_str(&params.memory_type)
        .ok_or_else(|| format!("Invalid memory type: {}", params.memory_type))?;

    let entry = MemoryEntry {
        id: String::new(),
        title: params.title,
        content: params.content,
        memory_type,
        tags: params.tags,
        created_at: 0,
        updated_at: 0,
        priority: params.priority,
        is_active: true,
        access_count: 0,
    };

    kb.add_entry(entry).await.map_err(|e| e.to_string())
}

/// Delete a knowledge entry
#[tauri::command]
pub async fn delete_knowledge_entry(
    kb: tauri::State<'_, KnowledgeBase>,
    id: String,
) -> Result<(), String> {
    kb.delete_entry(&id).await.map_err(|e| e.to_string())
}

/// Search knowledge base
#[derive(Debug, Deserialize)]
pub struct SearchKnowledgeParams {
    pub query: String,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[tauri::command]
pub async fn search_knowledge(
    kb: tauri::State<'_, KnowledgeBase>,
    params: SearchKnowledgeParams,
) -> Result<Vec<KnowledgeEntryResponse>, String> {
    let options = SearchOptions {
        query: Some(params.query),
        memory_type: params.memory_type.and_then(|t| MemoryType::from_str(&t)),
        tags: params.tags,
        min_priority: None,
        limit: params.limit,
        sort_by_access: false,
    };

    let results = kb.search(options).await;
    Ok(results.into_iter().map(KnowledgeEntryResponse::from).collect())
}

/// Get knowledge base statistics
#[tauri::command]
pub async fn get_knowledge_stats(
    kb: tauri::State<'_, KnowledgeBase>,
) -> Result<KnowledgeBaseStats, String> {
    Ok(kb.stats().await)
}

// ============================================================
// Code Indexing Commands (Tree-sitter)
// ============================================================

#[derive(Debug, Serialize)]
pub struct IndexProjectResponse {
    pub files_indexed: usize,
    pub stats: IndexStats,
}

/// Index a project directory with Tree-sitter
#[tauri::command]
pub async fn index_project(
    indexer: tauri::State<'_, SemanticIndexer>,
    path: String,
) -> Result<IndexProjectResponse, String> {
    let project_path = std::path::Path::new(&path);

    if !project_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let ids = indexer.index_directory(project_path).await.map_err(|e| e.to_string())?;
    let stats = indexer.stats().await;

    Ok(IndexProjectResponse {
        files_indexed: ids.len(),
        stats,
    })
}

/// Search indexed code
#[derive(Debug, Deserialize)]
pub struct SearchCodeParams {
    pub query: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize { 20 }

#[derive(Debug, Serialize)]
pub struct SearchCodeResult {
    pub file_path: String,
    pub language: String,
    pub symbols: Vec<CodeSymbol>,
    pub line_count: usize,
    pub preview: String,
}

#[tauri::command]
pub async fn search_code(
    indexer: tauri::State<'_, SemanticIndexer>,
    params: SearchCodeParams,
) -> Result<Vec<SearchCodeResult>, String> {
    let results = indexer.search(&params.query, params.limit).await;

    Ok(results.into_iter().map(|entry| {
        let preview = entry.content.lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n");

        SearchCodeResult {
            file_path: entry.file_path,
            language: entry.language.as_str().to_string(),
            symbols: entry.symbols,
            line_count: entry.line_count,
            preview,
        }
    }).collect())
}

/// Get symbols from a specific file
#[tauri::command]
pub async fn get_file_symbols(
    indexer: tauri::State<'_, SemanticIndexer>,
    file_path: String,
) -> Result<Vec<CodeSymbol>, String> {
    Ok(indexer.get_file_symbols(&file_path).await)
}

/// Get index statistics
#[tauri::command]
pub async fn get_index_stats(
    indexer: tauri::State<'_, SemanticIndexer>,
) -> Result<IndexStats, String> {
    Ok(indexer.stats().await)
}

// ============================================================
// Rules Engine Commands
// ============================================================

#[derive(Debug, Deserialize)]
pub struct AddRuleParams {
    pub name: String,
    pub description: String,
    pub condition_type: String,
    pub condition_value: String,
    pub action_type: String,
    pub action_value: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: String,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Rule> for RuleResponse {
    fn from(r: Rule) -> Self {
        Self {
            id: r.id,
            name: r.name,
            description: r.description,
            priority: format!("{:?}", r.priority),
            is_active: r.is_active,
            tags: r.tags,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// List all rules
#[tauri::command]
pub async fn list_rules(
    rules_engine: tauri::State<'_, RulesEngine>,
) -> Result<Vec<RuleResponse>, String> {
    let rules = rules_engine.list_rules().await;
    Ok(rules.into_iter().map(RuleResponse::from).collect())
}

/// Add a new rule
#[tauri::command]
pub async fn add_rule(
    rules_engine: tauri::State<'_, RulesEngine>,
    params: AddRuleParams,
) -> Result<String, String> {
    let condition = match params.condition_type.as_str() {
        "exact" => ConditionType::Exact(params.condition_value),
        "contains" => ConditionType::Contains(params.condition_value),
        "regex" => ConditionType::Regex(params.condition_value),
        "glob" => ConditionType::Glob(params.condition_value),
        "extension" => ConditionType::FileExtension(params.condition_value),
        "always" => ConditionType::Always,
        _ => return Err(format!("Invalid condition type: {}", params.condition_type)),
    };

    let action = match params.action_type.as_str() {
        "allow" => RuleAction::Allow,
        "deny" => RuleAction::Deny(params.action_value.unwrap_or_default()),
        "transform" => RuleAction::Transform(params.action_value.unwrap_or_default()),
        "notify" => RuleAction::Notify(params.action_value.unwrap_or_default()),
        "log" => RuleAction::Log(params.action_value.unwrap_or_default()),
        "custom" => RuleAction::Custom(params.action_value.unwrap_or_default()),
        _ => return Err(format!("Invalid action type: {}", params.action_type)),
    };

    let priority = match params.priority.as_deref() {
        Some("Constitution") => RulePriority::Constitution,
        Some("Protocol") => RulePriority::Protocol,
        Some("UserRule") | Some("User") => RulePriority::UserRule,
        _ => RulePriority::Default,
    };

    let now = chrono::Utc::now().timestamp();

    let rule = Rule {
        id: format!("rule_{}", uuid::Uuid::new_v4()),
        name: params.name,
        description: params.description,
        condition,
        action,
        priority,
        is_active: true,
        tags: params.tags,
        created_at: now,
        updated_at: now,
    };

    rules_engine.add_rule(rule).await.map_err(|e| e.to_string())
}

/// Delete a rule
#[tauri::command]
pub async fn delete_rule(
    rules_engine: tauri::State<'_, RulesEngine>,
    id: String,
) -> Result<(), String> {
    rules_engine.remove_rule(&id).await.map_err(|e| e.to_string())
}

/// Evaluate rules against a context
#[tauri::command]
pub async fn evaluate_rules(
    rules_engine: tauri::State<'_, RulesEngine>,
    context: String,
) -> Result<Vec<EvaluationResult>, String> {
    Ok(rules_engine.evaluate(&context).await)
}

// ============================================================
// LLM Commands
// ============================================================

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

// ============================================================
// Tool / Memory / Agent Module Commands
// ============================================================

#[tauri::command]
pub async fn execute_tool(
    tool_module: tauri::State<'_, std::sync::Arc<ToolModule>>,
    tool_id: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let result = tool_module
        .execute_tool(&tool_id, args)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn manage_memory(
    memory_module: tauri::State<'_, std::sync::Arc<MemoryModule>>,
    command: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    memory_module
        .execute(&command, args)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn manage_agent_tasks(
    agent_module: tauri::State<'_, std::sync::Arc<AgentModule>>,
    command: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    agent_module
        .execute(&command, args)
        .await
        .map_err(|e| e.to_string())
}
