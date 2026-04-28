use anyhow::{Result, Context};
use libloading::Library;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use log::{info, warn, error};
use wasmtime::{Engine, Module, Store, Instance, Linker};

/// Represents a dynamically loaded module
#[derive(Debug)]
pub struct DynamicModule {
    pub id: String,
    pub name: String,
    pub version: String,
    pub library: Option<Library>,
    pub wasm_instance: Option<WasmModule>,
    pub path: PathBuf,
    pub is_active: bool,
    pub module_type: ModuleType,
}

#[derive(Debug, Clone)]
pub enum ModuleType {
    Dylib,
    Wasm,
}

#[derive(Debug)]
pub struct WasmModule {
    pub module: Module,
    pub store: Arc<RwLock<Store<()>>>,
    pub instance: Instance,
}

/// Information about a loaded module for API responses
#[derive(Debug, Clone, serde::Serialize)]
pub struct DynamicModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub is_active: bool,
    pub module_type: String,
}

/// Lifecycle Manager - handles hot-reloading of modules without restarting the core
pub struct LifecycleManager {
    modules: Arc<RwLock<HashMap<String, DynamicModule>>>,
    modules_dir: PathBuf,
    wasm_engine: Engine,
}

impl LifecycleManager {
    pub fn new(modules_dir: PathBuf) -> Self {
        let wasm_engine = Engine::default();
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            modules_dir,
            wasm_engine,
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

        let (module_type, library, wasm_instance) = if path.extension().map_or(false, |ext| ext == "so" || ext == "dll" || ext == "dylib") {
            // Dynamic library
            let lib = unsafe { Library::new(path) }.context("Failed to load dynamic library")?;
            (ModuleType::Dylib, Some(lib), None)
        } else if path.extension().map_or(false, |ext| ext == "wasm") {
            // WASM module
            let wasm_module = self.load_wasm_module(path).await?;
            (ModuleType::Wasm, None, Some(wasm_module))
        } else {
            anyhow::bail!("Unsupported module format");
        };

        let mut modules = self.modules.write().await;
        modules.insert(module_id.clone(), DynamicModule {
            id: module_id.clone(),
            name: file_name,
            version: "0.1.0".to_string(),
            library,
            wasm_instance,
            path: path.to_path_buf(),
            is_active: true,
            module_type,
        });

        info!("Module {} loaded successfully", module_id);
        Ok(module_id)
    }

    /// Load WASM module using Wasmtime
    async fn load_wasm_module(&self, path: &Path) -> Result<WasmModule> {
        let module = Module::from_file(&self.wasm_engine, path)
            .context("Failed to load WASM module")?;

        let mut store = Store::new(&self.wasm_engine, ());
        let mut linker = Linker::new(&self.wasm_engine);

        // For now, skip WASI setup - it requires proper Component API integration
        // This is simplified WASM support without full WASI
        // TODO: Implement proper WASI support with wasmtime::component::Linker

        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        Ok(WasmModule {
            module,
            store: Arc::new(RwLock::new(store)),
            instance,
        })
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

    /// Get list of all modules
    pub async fn list_modules(&self) -> Vec<DynamicModuleInfo> {
        let modules = self.modules.read().await;
        modules.values().map(|module| {
            DynamicModuleInfo {
                id: module.id.clone(),
                name: module.name.clone(),
                version: module.version.clone(),
                path: module.path.to_string_lossy().to_string(),
                is_active: module.is_active,
                module_type: format!("{:?}", module.module_type),
            }
        }).collect()
    }

    /// Get module by ID
    pub async fn get_module(&self, module_id: &str) -> Option<DynamicModuleInfo> {
        let modules = self.modules.read().await;
        modules.get(module_id).map(|module| {
            DynamicModuleInfo {
                id: module.id.clone(),
                name: module.name.clone(),
                version: module.version.clone(),
                path: module.path.to_string_lossy().to_string(),
                is_active: module.is_active,
                module_type: format!("{:?}", module.module_type),
            }
        })
    }

    /// Scan modules directory and load all available modules
    pub async fn scan_and_load_modules(&self) -> Result<Vec<String>> {
        let mut loaded_modules = Vec::new();

        if !self.modules_dir.exists() {
            info!("Modules directory does not exist: {:?}", self.modules_dir);
            return Ok(loaded_modules);
        }

        let entries = std::fs::read_dir(&self.modules_dir)
            .context("Failed to read modules directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "so" || ext == "dll" || ext == "dylib" || ext == "wasm" {
                        match self.load_module(&path).await {
                            Ok(module_id) => {
                                loaded_modules.push(module_id);
                            }
                            Err(e) => {
                                error!("Failed to load module {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        info!("Loaded {} modules from {:?}", loaded_modules.len(), self.modules_dir);
        Ok(loaded_modules)
    }

    /// Execute a function from a loaded module
    pub async fn execute_module_function(&self, module_id: &str, function_name: &str, args: &[u8]) -> Result<Vec<u8>> {
        let modules = self.modules.read().await;
        let module = modules.get(module_id)
            .context("Module not found")?;

        if !module.is_active {
            anyhow::bail!("Module is not active");
        }

        match &module.module_type {
            ModuleType::Dylib => {
                if let Some(lib) = &module.library {
                    // For Dylib, we'd need to get function pointers
                    // This is a simplified example
                    anyhow::bail!("Dylib function execution not implemented");
                } else {
                    anyhow::bail!("No library loaded for module");
                }
            }
            ModuleType::Wasm => {
                if let Some(wasm) = &module.wasm_instance {
                    // Execute WASM function
                    let mut store_mut = wasm.store.write().await;
                    let mut store_ref = &mut *store_mut;
                    let func = wasm.instance.get_typed_func::<(), ()>(&mut store_ref, function_name)
                        .context("Function not found in WASM module")?;

                    func.call(&mut store_ref, ())
                        .context("Failed to execute WASM function")?;

                    // For now, return empty result
                    Ok(Vec::new())
                } else {
                    anyhow::bail!("No WASM instance loaded for module");
                }
            }
        }
    }
}
