use crate::core::LifecycleManager;
use crate::memory::{KnowledgeBase, SemanticIndexer, RulesEngine, VectorStore};
use crate::memory::knowledge_base::{MemoryType, MemoryEntry, SearchOptions, KnowledgeBaseStats};
use crate::memory::indexer::{IndexEntry, IndexStats, CodeSymbol, Language, SymbolKind};
use crate::memory::rules_engine::{Rule, RuleUpdate, EvaluationResult, RulePriority, ConditionType, RuleAction};
use crate::memory::vector_store::SearchResult as VectorSearchResult;
use crate::llm::{LLMEngine, ChatRequest, Message, LLMResponse, LLMStatus};
use crate::terminal::{TerminalManager, TerminalInfo};
use crate::modules::{ToolModule, MemoryModule, AgentModule};
use crate::sandbox::container::{
    ContainerManager, ContainerConfig as SandboxContainerConfig,
    ContainerStatus, ManagedContainer, DockerStatus, ExecResult,
};
use crate::sandbox::vnc::{VncManager, VncConfig, VncSession};
use crate::sandbox::browser::{BrowserAutomation, BrowserConfig, BrowserAction, ActionResult, ScreenshotResult, PageInfo};
use crate::sandbox::self_healing::{SelfHealingLoop, HealingStats, HealthCheckResult, HealingEvent};
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

    let filtered = results.into_iter()
        .filter(|entry| {
            match &params.language {
                Some(lang) if !lang.is_empty() => {
                    entry.language.as_str().eq_ignore_ascii_case(lang)
                }
                _ => true,
            }
        })
        .map(|entry| {
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
        })
        .collect();

    Ok(filtered)
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

/// Send a chat message (non-streaming, returns full response)
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
        temperature: None,
        max_tokens: None,
        system_prompt,
    };

    llm.chat(request)
        .await
        .map_err(|e| e.to_string())
}

/// Send a chat message with SSE streaming (emits llm-token, llm-done, llm-error events)
#[tauri::command]
pub async fn chat_with_llm_stream(
    llm: tauri::State<'_, LLMEngine>,
    app_handle: tauri::AppHandle,
    user_message: String,
    system_prompt: Option<String>,
    message_id: String,
) -> Result<(), String> {
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: user_message,
        }
    ];

    let request = ChatRequest {
        messages,
        temperature: None,
        max_tokens: None,
        system_prompt,
    };

    llm.chat_stream(request, message_id, app_handle)
        .await
        .map_err(|e| e.to_string())
}

/// Check LLM server status (online/offline, model info)
#[tauri::command]
pub async fn llm_status(
    llm: tauri::State<'_, LLMEngine>,
) -> Result<LLMStatus, String> {
    Ok(llm.status().await)
}

/// Stop the current LLM generation
#[tauri::command]
pub async fn llm_stop_generation(
    llm: tauri::State<'_, LLMEngine>,
) -> Result<(), String> {
    llm.stop_generation();
    Ok(())
}

/// Clear LLM conversation history
#[tauri::command]
pub async fn llm_clear_history(
    llm: tauri::State<'_, LLMEngine>,
) -> Result<(), String> {
    llm.clear_history().await;
    Ok(())
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

// ============================================================
// Sandbox Commands — Container Management
// ============================================================

#[derive(Debug, Serialize)]
pub struct ContainerResponse {
    pub id: String,
    pub docker_id: Option<String>,
    pub image: String,
    pub name: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub health_check_failures: u32,
}

impl From<ManagedContainer> for ContainerResponse {
    fn from(c: ManagedContainer) -> Self {
        Self {
            id: c.id,
            docker_id: c.docker_id,
            image: c.config.image,
            name: c.config.name,
            status: c.status.as_str().to_string(),
            created_at: c.created_at,
            health_check_failures: c.health_check_failures,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateContainerParams {
    pub image: String,
    pub name: Option<String>,
    pub memory_limit_mb: Option<u64>,
    pub cpu_limit: Option<f64>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub ports: Vec<(u16, u16)>,
    #[serde(default)]
    pub volumes: Vec<(String, String)>,
    pub working_dir: Option<String>,
    pub command: Option<Vec<String>>,
}

/// Get Docker daemon status
#[tauri::command]
pub async fn get_docker_status(
    cm: tauri::State<'_, ContainerManager>,
) -> Result<DockerStatus, String> {
    Ok(cm.docker_status().await)
}

/// Create a new sandbox container
#[tauri::command]
pub async fn create_container(
    cm: tauri::State<'_, ContainerManager>,
    params: CreateContainerParams,
) -> Result<String, String> {
    let config = SandboxContainerConfig {
        image: params.image,
        name: params.name,
        memory_limit_mb: params.memory_limit_mb,
        cpu_limit: params.cpu_limit,
        env_vars: params.env_vars,
        ports: params.ports,
        volumes: params.volumes,
        working_dir: params.working_dir,
        command: params.command,
        auto_remove: false,
        network_mode: None,
    };

    cm.create(config).await.map_err(|e| e.to_string())
}

/// Start a container
#[tauri::command]
pub async fn start_container(
    cm: tauri::State<'_, ContainerManager>,
    container_id: String,
) -> Result<(), String> {
    cm.start(&container_id).await.map_err(|e| e.to_string())
}

/// Stop a container
#[tauri::command]
pub async fn stop_container(
    cm: tauri::State<'_, ContainerManager>,
    container_id: String,
) -> Result<(), String> {
    cm.stop(&container_id).await.map_err(|e| e.to_string())
}

/// Remove a container
#[tauri::command]
pub async fn remove_container(
    cm: tauri::State<'_, ContainerManager>,
    container_id: String,
) -> Result<(), String> {
    cm.remove(&container_id).await.map_err(|e| e.to_string())
}

/// List all containers
#[tauri::command]
pub async fn list_containers(
    cm: tauri::State<'_, ContainerManager>,
) -> Result<Vec<ContainerResponse>, String> {
    let containers = cm.list().await;
    Ok(containers.into_iter().map(ContainerResponse::from).collect())
}

/// Execute a command in a container
#[tauri::command]
pub async fn exec_in_container(
    cm: tauri::State<'_, ContainerManager>,
    container_id: String,
    command: Vec<String>,
) -> Result<ExecResult, String> {
    cm.exec(&container_id, command).await.map_err(|e| e.to_string())
}

/// Get container logs
#[tauri::command]
pub async fn get_container_logs(
    cm: tauri::State<'_, ContainerManager>,
    container_id: String,
    tail: Option<usize>,
) -> Result<Vec<String>, String> {
    cm.get_logs(&container_id, tail.unwrap_or(100)).await.map_err(|e| e.to_string())
}

// ============================================================
// Sandbox Commands — VNC
// ============================================================

#[derive(Debug, Serialize)]
pub struct VncSessionResponse {
    pub id: String,
    pub container_id: Option<String>,
    pub status: String,
    pub websocket_url: String,
    pub resolution: (u32, u32),
    pub created_at: i64,
}

impl From<VncSession> for VncSessionResponse {
    fn from(s: VncSession) -> Self {
        Self {
            id: s.id,
            container_id: s.container_id,
            status: s.status.as_str().to_string(),
            websocket_url: s.websocket_url,
            resolution: s.config.resolution,
            created_at: s.created_at,
        }
    }
}

/// Create a VNC session
#[tauri::command]
pub async fn create_vnc_session(
    vnc: tauri::State<'_, VncManager>,
    container_id: Option<String>,
    port: Option<u16>,
    websocket_port: Option<u16>,
) -> Result<String, String> {
    let config = VncConfig {
        port: port.unwrap_or(5900),
        websocket_port: websocket_port.unwrap_or(6080),
        ..Default::default()
    };
    vnc.create_session(config, container_id).await.map_err(|e| e.to_string())
}

/// Connect to a VNC session
#[tauri::command]
pub async fn connect_vnc(
    vnc: tauri::State<'_, VncManager>,
    session_id: String,
) -> Result<String, String> {
    vnc.connect(&session_id).await.map_err(|e| e.to_string())
}

/// Disconnect from a VNC session
#[tauri::command]
pub async fn disconnect_vnc(
    vnc: tauri::State<'_, VncManager>,
    session_id: String,
) -> Result<(), String> {
    vnc.disconnect(&session_id).await.map_err(|e| e.to_string())
}

/// List VNC sessions
#[tauri::command]
pub async fn list_vnc_sessions(
    vnc: tauri::State<'_, VncManager>,
) -> Result<Vec<VncSessionResponse>, String> {
    let sessions = vnc.list_sessions().await;
    Ok(sessions.into_iter().map(VncSessionResponse::from).collect())
}

// ============================================================
// Sandbox Commands — Browser Automation
// ============================================================

/// Launch headless browser
#[tauri::command]
pub async fn launch_browser(
    browser: tauri::State<'_, BrowserAutomation>,
) -> Result<String, String> {
    browser.launch().await.map_err(|e| e.to_string())?;
    Ok("Browser launched".to_string())
}

/// Navigate browser to URL
#[tauri::command]
pub async fn browser_navigate(
    browser: tauri::State<'_, BrowserAutomation>,
    url: String,
) -> Result<PageInfo, String> {
    browser.navigate(&url).await.map_err(|e| e.to_string())
}

/// Take browser screenshot
#[tauri::command]
pub async fn browser_screenshot(
    browser: tauri::State<'_, BrowserAutomation>,
) -> Result<ScreenshotResult, String> {
    browser.screenshot().await.map_err(|e| e.to_string())
}

/// Close browser
#[tauri::command]
pub async fn close_browser(
    browser: tauri::State<'_, BrowserAutomation>,
) -> Result<(), String> {
    browser.close().await.map_err(|e| e.to_string())
}

/// Get browser status
#[tauri::command]
pub async fn get_browser_status(
    browser: tauri::State<'_, BrowserAutomation>,
) -> Result<String, String> {
    Ok(browser.get_status().await.as_str().to_string())
}

// ============================================================
// Sandbox Commands — Self-Healing
// ============================================================

/// Get self-healing statistics
#[tauri::command]
pub async fn get_healing_stats(
    healing: tauri::State<'_, SelfHealingLoop>,
) -> Result<HealingStats, String> {
    Ok(healing.get_stats().await)
}

/// Run health check on all monitored containers
#[tauri::command]
pub async fn run_health_check(
    healing: tauri::State<'_, SelfHealingLoop>,
    cm: tauri::State<'_, ContainerManager>,
) -> Result<Vec<HealthCheckResult>, String> {
    Ok(healing.check_health(&cm).await)
}

/// Get healing event history
#[tauri::command]
pub async fn get_healing_events(
    healing: tauri::State<'_, SelfHealingLoop>,
    limit: Option<usize>,
) -> Result<Vec<HealingEvent>, String> {
    Ok(healing.get_events(limit.unwrap_or(50)).await)
}

// ============================================================
// Sandbox Status — Combined overview
// ============================================================

#[derive(Debug, Serialize)]
pub struct SandboxStatus {
    pub docker: DockerStatus,
    pub containers_count: usize,
    pub vnc_sessions_count: usize,
    pub browser_status: String,
    pub healing_stats: HealingStats,
}

/// Get overall sandbox status
#[tauri::command]
pub async fn get_sandbox_status(
    cm: tauri::State<'_, ContainerManager>,
    vnc: tauri::State<'_, VncManager>,
    browser: tauri::State<'_, BrowserAutomation>,
    healing: tauri::State<'_, SelfHealingLoop>,
) -> Result<SandboxStatus, String> {
    let docker = cm.docker_status().await;
    let containers_count = cm.list().await.len();
    let vnc_sessions_count = vnc.list_sessions().await.len();
    let browser_status = browser.get_status().await.as_str().to_string();
    let healing_stats = healing.get_stats().await;

    Ok(SandboxStatus {
        docker,
        containers_count,
        vnc_sessions_count,
        browser_status,
        healing_stats,
    })
}

// ============================================================
// Terminal Commands
// ============================================================

#[tauri::command]
pub async fn terminal_create(
    terminal_id: String,
    cols: Option<u16>,
    rows: Option<u16>,
    manager: tauri::State<'_, TerminalManager>,
    app_handle: tauri::AppHandle,
) -> Result<TerminalInfo, String> {
    let cols = cols.unwrap_or(80);
    let rows = rows.unwrap_or(24);
    manager.create(terminal_id, cols, rows, app_handle)
        .await
        .map_err(|e| format!("Failed to create terminal: {}", e))
}

#[tauri::command]
pub async fn terminal_write(
    terminal_id: String,
    data: String,
    manager: tauri::State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.write(&terminal_id, &data)
        .await
        .map_err(|e| format!("Failed to write to terminal: {}", e))
}

#[tauri::command]
pub async fn terminal_resize(
    terminal_id: String,
    cols: u16,
    rows: u16,
    manager: tauri::State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.resize(&terminal_id, cols, rows)
        .await
        .map_err(|e| format!("Failed to resize terminal: {}", e))
}

#[tauri::command]
pub async fn terminal_close(
    terminal_id: String,
    manager: tauri::State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.close(&terminal_id)
        .await
        .map_err(|e| format!("Failed to close terminal: {}", e))
}

#[tauri::command]
pub async fn terminal_list(
    manager: tauri::State<'_, TerminalManager>,
) -> Result<Vec<TerminalInfo>, String> {
    Ok(manager.list().await)
}
