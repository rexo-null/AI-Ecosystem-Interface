use anyhow::{Result, Context};
use libloading::Library;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use log::{info, warn, error};

/// Represents a dynamically loaded module
pub struct DynamicModule {
    pub id: String,
    pub name: String,
    pub version: String,
    pub library: Option<Library>,
    pub path: PathBuf,
    pub is_active: bool,
}

/// Lifecycle Manager - handles hot-reloading of modules without restarting the core
pub struct LifecycleManager {
    modules: Arc<RwLock<HashMap<String, DynamicModule>>>,
    modules_dir: PathBuf,
}

impl LifecycleManager {
    pub fn new(modules_dir: PathBuf) -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            modules_dir,
        }
    }

    /// Load a module dynamically (Dylib or WASM)
    pub async fn load_module(&self, path: &Path) -> Result<String> {
        let module_id = Uuid::new_v4().to_string();
        let file_name = path.file_stem()
            .context("Invalid module path")?
            .to_string_lossy()
            .to_string();

        info!("Loading module: {} from {:?}", file_name, path);

        // Determine module type and load accordingly
        let library = if path.extension().map_or(false, |ext| ext == "so" || ext == "dll" || ext == "dylib") {
            // Dynamic library
            unsafe { Library::new(path) }.context("Failed to load dynamic library")?
        } else if path.extension().map_or(false, |ext| ext == "wasm") {
            // WASM module - handled separately by Wasmtime
            // For now, we'll skip actual loading and just register
            info!("WASM module detected, registration only");
            return self.register_wasm_module(&module_id, &file_name, path).await;
        } else {
            anyhow::bail!("Unsupported module format");
        };

        let mut modules = self.modules.write().await;
        modules.insert(module_id.clone(), DynamicModule {
            id: module_id.clone(),
            name: file_name,
            version: "0.1.0".to_string(),
            library: Some(library),
            path: path.to_path_buf(),
            is_active: true,
        });

        info!("Module {} loaded successfully", module_id);
        Ok(module_id)
    }

    /// Unload a module safely
    pub async fn unload_module(&self, module_id: &str) -> Result<()> {
        let mut modules = self.modules.write().await;
        
        if let Some(mut module) = modules.remove(module_id) {
            info!("Unloading module: {}", module.name);
            module.is_active = false;
            // Note: libloading doesn't support unloading on all platforms
            // On Unix, we can use dlclose, but it's unsafe
            drop(module.library);
            Ok(())
        } else {
            anyhow::bail!("Module not found: {}", module_id)
        }
    }

    /// Reload a module (unload + load) - key for self-improvement
    pub async fn reload_module(&self, module_id: &str, new_path: &Path) -> Result<String> {
        info!("Reloading module {} with new path {:?}", module_id, new_path);
        
        // Unload old version
        self.unload_module(module_id).await?;
        
        // Load new version
        let new_id = self.load_module(new_path).await?;
        
        info!("Module reloaded successfully: {} -> {}", module_id, new_id);
        Ok(new_id)
    }

    /// Register a WASM module (placeholder for Wasmtime integration)
    async fn register_wasm_module(&self, module_id: &str, name: &str, path: &Path) -> Result<String> {
        let mut modules = self.modules.write().await;
        
        modules.insert(module_id.to_string(), DynamicModule {
            id: module_id.to_string(),
            name: name.to_string(),
            version: "0.1.0".to_string(),
            library: None, // WASM handled by Wasmtime
            path: path.to_path_buf(),
            is_active: true,
        });

        Ok(module_id.to_string())
    }

    /// List all active modules
    pub async fn list_modules(&self) -> Vec<DynamicModuleInfo> {
        let modules = self.modules.read().await;
        modules.values()
            .filter(|m| m.is_active)
            .map(|m| DynamicModuleInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                version: m.version.clone(),
                path: m.path.clone(),
                module_type: if m.path.extension().map_or(false, |ext| ext == "wasm") { 
                    "wasm" 
                } else { 
                    "dylib" 
                }.to_string(),
            })
            .collect()
    }

    /// Check if a module exists
    pub async fn module_exists(&self, module_id: &str) -> bool {
        let modules = self.modules.read().await;
        modules.contains_key(module_id)
    }
}

#[derive(Clone, Debug)]
pub struct DynamicModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub module_type: String,
}

// Example usage in Tauri commands would be in api/commands.rs
