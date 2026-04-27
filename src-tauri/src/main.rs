// ISKIN - Intelligent Self-Improving Knowledge Interface Network
// Main entry point for Tauri application

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod memory;
mod modules;
mod sandbox;
mod tools;
mod api;

use core::{LifecycleManager, PolicyEngine, ResourceManager, security::PolicyLevel};
use memory::KnowledgeBase;
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
    let rules_dir = app_dir.join("rules");
    let models_dir = app_dir.join("models");
    
    // Create necessary directories
    std::fs::create_dir_all(&modules_dir)?;
    std::fs::create_dir_all(&rules_dir)?;
    std::fs::create_dir_all(&models_dir)?;
    
    // Initialize core components
    let lifecycle_manager = LifecycleManager::new(modules_dir.clone());
    let policy_engine = PolicyEngine::new(PolicyLevel::Balanced);
    let resource_manager = ResourceManager::new();
    let knowledge_base = KnowledgeBase::new(rules_dir.clone());
    
    // Initialize knowledge base
    knowledge_base.initialize().await?;
    
    info!("ISKIN core initialized successfully");
    info!("Modules directory: {:?}", modules_dir);
    info!("Rules directory: {:?}", rules_dir);
    info!("Models directory: {:?}", models_dir);
    
    // Run Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(lifecycle_manager)
        .manage(policy_engine)
        .manage(resource_manager)
        .manage(knowledge_base)
        .invoke_handler(tauri::generate_handler![
            api::commands::ping,
            api::commands::list_modules,
            api::commands::get_knowledge_entries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ISKIN");
    
    Ok(())
}
