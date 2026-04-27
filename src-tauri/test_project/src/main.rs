use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub trait ISKINModule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn capabilities(&self) -> Vec<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

pub struct ModuleRegistry {
    modules: HashMap<String, Box<dyn ISKINModule>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register(&mut self, module: Box<dyn ISKINModule>) -> Result<(), String> {
        let id = module.id().to_string();
        if self.modules.contains_key(&id) {
            return Err(format!("Module with id '{}' already registered", id));
        }
        self.modules.insert(id, module);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Box<dyn ISKINModule>> {
        self.modules.get(id)
    }

    pub fn list(&self) -> Vec<ModuleInfo> {
        self.modules.values().map(|m| ModuleInfo {
            id: m.id().to_string(),
            name: m.name().to_string(),
            version: m.version().to_string(),
            capabilities: m.capabilities(),
        }).collect()
    }
}

// Simple test module
pub struct TestModule {
    id: String,
    name: String,
    version: String,
}

impl TestModule {
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

impl ISKINModule for TestModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initializing module: {}", self.name);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Shutting down module: {}", self.name);
        Ok(())
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["test".to_string()]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing ISKIN Module System...");

    let mut registry = ModuleRegistry::new();

    // Create and register test modules
    let tool_module = Box::new(TestModule::new("tool", "Tool Module", "1.0.0"));
    let memory_module = Box::new(TestModule::new("memory", "Memory Module", "1.0.0"));
    let agent_module = Box::new(TestModule::new("agent", "Agent Module", "1.0.0"));

    registry.register(tool_module)?;
    registry.register(memory_module)?;
    registry.register(agent_module)?;

    println!("Registered modules:");
    for module_info in registry.list() {
        println!("  - {} ({}) - v{}", module_info.name, module_info.id, module_info.version);
    }

    // Test initialization
    if let Some(module) = registry.get("tool") {
        let mut cloned = TestModule::new(module.id(), module.name(), module.version());
        cloned.initialize()?;
        cloned.shutdown()?;
    }

    println!("Module system test completed successfully!");
    Ok(())
}use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait ISKINModule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn capabilities(&self) -> Vec<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

pub struct ModuleRegistry {
    modules: HashMap<String, Box<dyn ISKINModule>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register(&mut self, module: Box<dyn ISKINModule>) -> Result<(), String> {
        let id = module.id().to_string();
        if self.modules.contains_key(&id) {
            return Err(format!("Module with id '{}' already registered", id));
        }
        self.modules.insert(id, module);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Box<dyn ISKINModule>> {
        self.modules.get(id)
    }

    pub fn list(&self) -> Vec<ModuleInfo> {
        self.modules.values().map(|m| ModuleInfo {
            id: m.id().to_string(),
            name: m.name().to_string(),
            version: m.version().to_string(),
            capabilities: m.capabilities(),
        }).collect()
    }
}

// Simple test module
pub struct TestModule {
    id: String,
    name: String,
    version: String,
}

impl TestModule {
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

#[async_trait]
impl ISKINModule for TestModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initializing module: {}", self.name);
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Shutting down module: {}", self.name);
        Ok(())
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["test".to_string()]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing ISKIN Module System...");

    let mut registry = ModuleRegistry::new();

    // Create and register test modules
    let tool_module = Box::new(TestModule::new("tool", "Tool Module", "1.0.0"));
    let memory_module = Box::new(TestModule::new("memory", "Memory Module", "1.0.0"));
    let agent_module = Box::new(TestModule::new("agent", "Agent Module", "1.0.0"));

    registry.register(tool_module)?;
    registry.register(memory_module)?;
    registry.register(agent_module)?;

    println!("Registered modules:");
    for module_info in registry.list() {
        println!("  - {} ({}) - v{}", module_info.name, module_info.id, module_info.version);
    }

    // Test initialization
    if let Some(module) = registry.get("tool") {
        let mut cloned = TestModule::new(module.id(), module.name(), module.version());
        cloned.initialize().await?;
        cloned.shutdown().await?;
    }

    println!("Module system test completed successfully!");
    Ok(())
}use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait ISKINModule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn capabilities(&self) -> Vec<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

pub struct ModuleRegistry {
    modules: HashMap<String, Box<dyn ISKINModule>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register(&mut self, module: Box<dyn ISKINModule>) -> Result<(), String> {
        let id = module.id().to_string();
        if self.modules.contains_key(&id) {
            return Err(format!("Module with id '{}' already registered", id));
        }
        self.modules.insert(id, module);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Box<dyn ISKINModule>> {
        self.modules.get(id)
    }

    pub fn list(&self) -> Vec<ModuleInfo> {
        self.modules.values().map(|m| ModuleInfo {
            id: m.id().to_string(),
            name: m.name().to_string(),
            version: m.version().to_string(),
            capabilities: m.capabilities(),
        }).collect()
    }
}

// Simple test module
pub struct TestModule {
    id: String,
    name: String,
    version: String,
}

impl TestModule {
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

#[async_trait]
impl ISKINModule for TestModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initializing module: {}", self.name);
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Shutting down module: {}", self.name);
        Ok(())
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["test".to_string()]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing ISKIN Module System...");

    let mut registry = ModuleRegistry::new();

    // Create and register test modules
    let tool_module = Box::new(TestModule::new("tool", "Tool Module", "1.0.0"));
    let memory_module = Box::new(TestModule::new("memory", "Memory Module", "1.0.0"));
    let agent_module = Box::new(TestModule::new("agent", "Agent Module", "1.0.0"));

    registry.register(tool_module)?;
    registry.register(memory_module)?;
    registry.register(agent_module)?;

    println!("Registered modules:");
    for module_info in registry.list() {
        println!("  - {} ({}) - v{}", module_info.name, module_info.id, module_info.version);
    }

    // Test initialization
    if let Some(module) = registry.get("tool") {
        let mut cloned = TestModule::new(module.id(), module.name(), module.version());
        cloned.initialize().await?;
        cloned.shutdown().await?;
    }

    println!("Module system test completed successfully!");
    Ok(())
}// ISKIN - Intelligent Self-Improving Knowledge Interface Network
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
use memory::KnowledgeBase;
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
        .manage(llm_engine)
        .manage(tool_module)
        .manage(memory_module)
        .manage(agent_module)
        .invoke_handler(tauri::generate_handler![
            api::commands::ping,
            api::commands::list_modules,
            api::commands::get_knowledge_entries,
            api::commands::chat_with_llm,
            api::commands::execute_tool,
            api::commands::manage_memory,
            api::commands::manage_agent_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ISKIN");
    
    Ok(())
}
