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

use core::{LifecycleManager, PolicyEngine, ResourceManager, security::PolicyLevel};
use memory::{KnowledgeBase, SemanticIndexer, RulesEngine, VectorStore};
use memory::vector_store::VectorStoreConfig;
use llm::LLMEngine;
use modules::{ToolModule, MemoryModule, AgentModule, ISKINModule};
use std::sync::Arc;
use std::path::PathBuf;
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

    // Initialize LLM engine
    let llm_engine = LLMEngine::new("Qwen-2.5-Coder-14B".to_string());
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
            // Modules
            api::commands::execute_tool,
            api::commands::manage_memory,
            api::commands::manage_agent_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ISKIN");
    
    Ok(())
}
