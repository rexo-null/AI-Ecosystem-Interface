// ISKIN - Intelligent Self-Improving Knowledge Interface Network
// Main entry point for Tauri application

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod memory;
mod modules;
mod sandbox;
mod tools;
mod api;
mod llm;
mod terminal;
mod agent;

use core::{LifecycleManager, PolicyEngine, ResourceManager, security::PolicyLevel};
use memory::{KnowledgeBase, SemanticIndexer, RulesEngine, VectorStore};
use memory::vector_store::VectorStoreConfig;
use llm::{LLMEngine, LLMConfig};
use modules::{ToolModule, MemoryModule, AgentModule, ISKINModule};
use sandbox::container::ContainerManager;
use sandbox::vnc::VncManager;
use sandbox::browser::{BrowserAutomation, BrowserConfig};
use sandbox::self_healing::SelfHealingLoop;
use terminal::TerminalManager;
use std::sync::Arc;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("Starting ISKIN v0.1.0-alpha");
    
    // Get application directories
    let app_dir = std::env::current_dir()?;
    let modules_dir = app_dir.join("modules");
    let data_dir = app_dir.join("data");
    let rules_dir = data_dir.join("rules");
    let knowledge_dir = data_dir.join("knowledge");
    let models_dir = app_dir.join("models");
    
    // Create necessary directories
    std::fs::create_dir_all(&modules_dir)?;
    std::fs::create_dir_all(&rules_dir)?;
    std::fs::create_dir_all(&knowledge_dir)?;
    std::fs::create_dir_all(&models_dir)?;
    
    // Initialize core components
    let lifecycle_manager = LifecycleManager::new(modules_dir.clone());
    let policy_engine = PolicyEngine::new(PolicyLevel::Balanced);
    let resource_manager = ResourceManager::new();

    // Initialize Knowledge Base with disk persistence
    let knowledge_base = KnowledgeBase::new(knowledge_dir.clone());
    knowledge_base.initialize().await?;

    // Initialize Semantic Indexer (Tree-sitter)
    let semantic_indexer = SemanticIndexer::new()
        .map_err(|e| anyhow::anyhow!("Failed to create SemanticIndexer: {}", e))?;

    // Initialize Rules Engine with disk persistence
    let rules_engine = RulesEngine::new(rules_dir.clone());
    rules_engine.initialize().await?;

    // Initialize Vector Store (Qdrant with local fallback)
    let vector_store = VectorStore::new(VectorStoreConfig::default());
    vector_store.initialize().await?;

    // Load LLM configuration
    let config_dir = app_dir.join("config");
    std::fs::create_dir_all(&config_dir)?;
    let llm_config_path = config_dir.join("llm.toml");
    let llm_config = if llm_config_path.exists() {
        match LLMConfig::load_from_file(&llm_config_path) {
            Ok(config) => {
                info!("LLM config loaded from {:?}", llm_config_path);
                config
            }
            Err(e) => {
                log::warn!("Failed to load LLM config: {}, using defaults", e);
                LLMConfig::default()
            }
        }
    } else {
        info!("No LLM config found at {:?}, using defaults", llm_config_path);
        LLMConfig::default()
    };

    // Initialize LLM engine with config
    let llm_engine = LLMEngine::new(llm_config);
    llm_engine.initialize().await?;

    // Initialize built-in modules
    let tool_module = Arc::new(ToolModule::new());
    let memory_module = Arc::new(MemoryModule::new());
    let agent_module = Arc::new(AgentModule::new());

    // Initialize modules
    tool_module.initialize().await?;
    memory_module.initialize().await?;
    agent_module.initialize().await?;

    // Scan and load dynamic modules
    let loaded_modules = lifecycle_manager.scan_and_load_modules().await?;
    info!("Loaded {} dynamic modules", loaded_modules.len());
    
    // Initialize Sandbox components
    let container_manager = ContainerManager::new();
    container_manager.initialize().await?;

    let vnc_manager = VncManager::new();
    let browser_automation = BrowserAutomation::new(BrowserConfig::default());
    let self_healing = SelfHealingLoop::new();

    // Initialize Terminal Manager
    let terminal_manager = TerminalManager::new();

    info!("ISKIN core initialized successfully");
    info!("Modules directory: {:?}", modules_dir);
    info!("Data directory: {:?}", data_dir);
    info!("Models directory: {:?}", models_dir);
    
    // Run Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(lifecycle_manager)
        .manage(policy_engine)
        .manage(resource_manager)
        .manage(knowledge_base)
        .manage(semantic_indexer)
        .manage(rules_engine)
        .manage(vector_store)
        .manage(llm_engine)
        .manage(tool_module)
        .manage(memory_module)
        .manage(agent_module)
        .manage(container_manager)
        .manage(vnc_manager)
        .manage(browser_automation)
        .manage(self_healing)
        .manage(terminal_manager)
        .invoke_handler(tauri::generate_handler![
            // Core
            api::commands::ping,
            api::commands::list_modules,
            // Knowledge Base
            api::commands::get_knowledge_entries,
            api::commands::add_knowledge_entry,
            api::commands::delete_knowledge_entry,
            api::commands::search_knowledge,
            api::commands::get_knowledge_stats,
            // Code Indexing (Tree-sitter)
            api::commands::index_project,
            api::commands::search_code,
            api::commands::get_file_symbols,
            api::commands::get_index_stats,
            // Rules Engine
            api::commands::list_rules,
            api::commands::add_rule,
            api::commands::delete_rule,
            api::commands::evaluate_rules,
            // LLM
            api::commands::chat_with_llm,
            api::commands::chat_with_llm_stream,
            api::commands::llm_status,
            api::commands::llm_stop_generation,
            api::commands::llm_clear_history,
            // Modules
            api::commands::execute_tool,
            api::commands::manage_memory,
            api::commands::manage_agent_tasks,
            // Sandbox — Containers
            api::commands::get_docker_status,
            api::commands::create_container,
            api::commands::start_container,
            api::commands::stop_container,
            api::commands::remove_container,
            api::commands::list_containers,
            api::commands::exec_in_container,
            api::commands::get_container_logs,
            // Sandbox — VNC
            api::commands::create_vnc_session,
            api::commands::connect_vnc,
            api::commands::disconnect_vnc,
            api::commands::list_vnc_sessions,
            // Sandbox — Browser
            api::commands::launch_browser,
            api::commands::browser_navigate,
            api::commands::browser_screenshot,
            api::commands::close_browser,
            api::commands::get_browser_status,
            // Sandbox — Self-Healing
            api::commands::get_healing_stats,
            api::commands::run_health_check,
            api::commands::get_healing_events,
            // Sandbox — Status
            api::commands::get_sandbox_status,
            // Terminal
            api::commands::terminal_create,
            api::commands::terminal_write,
            api::commands::terminal_resize,
            api::commands::terminal_close,
            api::commands::terminal_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ISKIN");
    
    Ok(())
}
