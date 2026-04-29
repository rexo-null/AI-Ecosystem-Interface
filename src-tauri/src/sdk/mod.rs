//! ISKIN Module SDK
//! 
//! Stable API для создания кастомных модулей с горячей перезагрузкой.
//! Поддерживает Dylib (native) и WASM (sandboxed) модули.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Manifest модуля - метаданные для регистрации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Уникальное имя модуля
    pub name: String,
    /// Версия в формате semver
    pub version: String,
    /// Описание назначения модуля
    pub description: String,
    /// Автор модуля
    pub author: Option<String>,
    /// Список предоставляемых инструментов
    pub tools: Vec<ToolDefinition>,
    /// Зависимости от других модулей
    pub dependencies: Vec<String>,
    /// Требуемые разрешения безопасности
    pub required_permissions: Vec<Permission>,
    /// Тип модуля (dylib/wasm)
    pub module_type: ModuleType,
}

/// Тип модуля
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleType {
    /// Native динамическая библиотека (.so/.dll/.dylib)
    Dylib,
    /// WebAssembly модуль (.wasm)
    Wasm,
}

/// Разрешения безопасности
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    /// Чтение файлов
    FileRead,
    /// Запись файлов
    FileWrite,
    /// Выполнение команд
    CommandExec,
    /// Сетевой доступ
    NetworkAccess,
    /// Доступ к Docker
    DockerAccess,
    /// Горячая перезагрузка
    HotReload,
}

/// Определение инструмента (tool)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Название инструмента
    pub name: String,
    /// Описание
    pub description: String,
    /// JSON Schema для параметров
    pub input_schema: serde_json::Value,
    /// JSON Schema для результата
    pub output_schema: Option<serde_json::Value>,
}

/// Trait для всех модулей
pub trait IskinModule: Send + Sync {
    /// Инициализация модуля
    fn init(&mut self) -> Result<(), ModuleError>;
    
    /// Получение манифеста
    fn manifest(&self) -> &ModuleManifest;
    
    /// Выполнение инструмента
    fn execute_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ModuleError>;
    
    /// Очистка ресурсов перед выгрузкой
    fn shutdown(&mut self) -> Result<(), ModuleError>;
}

/// Ошибки модуля
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleError {
    /// Ошибка инициализации
    InitError(String),
    /// Инструмент не найден
    ToolNotFound(String),
    /// Неверный формат параметров
    InvalidParams(String),
    /// Ошибка выполнения
    ExecutionError(String),
    /// Нарушение безопасности
    SecurityViolation(String),
    /// Ошибка совместимости версий
    VersionMismatch(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::InitError(msg) => write!(f, "Init error: {}", msg),
            ModuleError::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            ModuleError::InvalidParams(msg) => write!(f, "Invalid params: {}", msg),
            ModuleError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            ModuleError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            ModuleError::VersionMismatch(msg) => write!(f, "Version mismatch: {}", msg),
        }
    }
}

impl std::error::Error for ModuleError {}

/// Базовая структура модуля для наследования
pub struct BaseModule {
    manifest: ModuleManifest,
    tools: HashMap<String, ToolHandler>,
}

type ToolHandler = Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, ModuleError> + Send + Sync>;

impl BaseModule {
    pub fn new(manifest: ModuleManifest) -> Self {
        Self {
            manifest,
            tools: HashMap::new(),
        }
    }
    
    /// Регистрация инструмента
    pub fn register_tool<F>(&mut self, name: &str, handler: F) -> Result<(), ModuleError>
    where
        F: Fn(serde_json::Value) -> Result<serde_json::Value, ModuleError> + Send + Sync + 'static,
    {
        // Проверка наличия в манифесте
        if !self.manifest.tools.iter().any(|t| t.name == name) {
            return Err(ModuleError::ToolNotFound(name.to_string()));
        }
        
        self.tools.insert(name.to_string(), Box::new(handler));
        Ok(())
    }
}

impl IskinModule for BaseModule {
    fn init(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    
    fn manifest(&self) -> &ModuleManifest {
        &self.manifest
    }
    
    fn execute_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ModuleError> {
        let handler = self.tools.get(tool_name)
            .ok_or_else(|| ModuleError::ToolNotFound(tool_name.to_string()))?;
        
        handler(params)
    }
    
    fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.tools.clear();
        Ok(())
    }
}

/// Макрос для быстрого создания модулей
#[macro_export]
macro_rules! create_module {
    ($name:expr, $version:expr, $description:expr, [$($tool_name:expr),*]) => {
        {
            let manifest = $crate::sdk::ModuleManifest {
                name: $name.to_string(),
                version: $version.to_string(),
                description: $description.to_string(),
                author: None,
                tools: vec![$(
                    $crate::sdk::ToolDefinition {
                        name: $tool_name.to_string(),
                        description: format!("Tool {}", $tool_name),
                        input_schema: serde_json::json!({"type": "object"}),
                        output_schema: None,
                    }
                ),*],
                dependencies: vec![],
                required_permissions: vec![],
                module_type: $crate::sdk::ModuleType::Dylib,
            };
            $crate::sdk::BaseModule::new(manifest)
        }
    };
}

/// Пример модуля для тестирования
pub mod examples {
    use super::*;
    
    /// Простой файловый модуль
    pub fn create_file_module() -> BaseModule {
        let manifest = ModuleManifest {
            name: "file_utils".to_string(),
            version: "1.0.0".to_string(),
            description: "Базовые файловые операции".to_string(),
            author: Some("ISKIN Team".to_string()),
            tools: vec![
                ToolDefinition {
                    name: "read_file".to_string(),
                    description: "Чтение файла".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"}
                        },
                        "required": ["path"]
                    }),
                    output_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "content": {"type": "string"}
                        }
                    })),
                },
                ToolDefinition {
                    name: "write_file".to_string(),
                    description: "Запись файла".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"},
                            "content": {"type": "string"}
                        },
                        "required": ["path", "content"]
                    }),
                    output_schema: None,
                },
            ],
            dependencies: vec![],
            required_permissions: vec![Permission::FileRead, Permission::FileWrite],
            module_type: ModuleType::Dylib,
        };
        
        let mut module = BaseModule::new(manifest);
        
        // Регистрация обработчиков
        module.register_tool("read_file", |params| {
            let path = params["path"].as_str()
                .ok_or_else(|| ModuleError::InvalidParams("path must be string".to_string()))?;
            
            // В реальном модуле здесь будет чтение файла
            Ok(serde_json::json!({
                "content": format!("Content of {}", path)
            }))
        }).unwrap();
        
        module.register_tool("write_file", |params| {
            let path = params["path"].as_str()
                .ok_or_else(|| ModuleError::InvalidParams("path must be string".to_string()))?;
            let content = params["content"].as_str()
                .ok_or_else(|| ModuleError::InvalidParams("content must be string".to_string()))?;
            
            // В реальном модуле здесь будет запись файла
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Written {} bytes to {}", content.len(), path)
            }))
        }).unwrap();
        
        module
    }
    
    /// Модуль для работы с кодом
    pub fn create_code_module() -> BaseModule {
        let manifest = ModuleManifest {
            name: "code_analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "Анализ и поиск кода".to_string(),
            author: Some("ISKIN Team".to_string()),
            tools: vec![
                ToolDefinition {
                    name: "search_code".to_string(),
                    description: "Поиск паттернов в коде".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "pattern": {"type": "string"},
                            "language": {"type": "string"}
                        }
                    }),
                    output_schema: None,
                },
            ],
            dependencies: vec![],
            required_permissions: vec![Permission::FileRead],
            module_type: ModuleType::Dylib,
        };
        
        BaseModule::new(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::examples::*;
    
    #[test]
    fn test_module_creation() {
        let module = create_file_module();
        assert_eq!(module.manifest().name, "file_utils");
        assert_eq!(module.manifest().version, "1.0.0");
    }
    
    #[test]
    fn test_tool_execution() {
        let module = create_file_module();
        let result = module.execute_tool(
            "read_file",
            serde_json::json!({"path": "/test.txt"})
        ).unwrap();
        
        assert!(result["content"].is_string());
    }
    
    #[test]
    fn test_tool_not_found() {
        let module = create_file_module();
        let result = module.execute_tool(
            "nonexistent",
            serde_json::json!({})
        );
        
        assert!(matches!(result, Err(ModuleError::ToolNotFound(_))));
    }
    
    #[test]
    fn test_manifest_serialization() {
        let manifest = ModuleManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test module".to_string(),
            author: None,
            tools: vec![],
            dependencies: vec![],
            required_permissions: vec![],
            module_type: ModuleType::Wasm,
        };
        
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: ModuleManifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(manifest.name, deserialized.name);
        assert_eq!(manifest.module_type, deserialized.module_type);
    }
}
