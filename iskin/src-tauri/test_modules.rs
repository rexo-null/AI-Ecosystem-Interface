use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Test our module system without Tauri dependencies
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

// Test compilation
fn main() {
    println!("Module system compiles successfully!");
    let registry = ModuleRegistry::new();
    println!("Registered modules: {}", registry.list().len());
}