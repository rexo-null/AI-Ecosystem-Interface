# ISKIN Module SDK Guide

## Overview

The ISKIN Module SDK provides a stable API for creating custom modules with hot-reload capabilities. Modules can extend ISKIN's functionality with new tools, integrations, and features.

## Module Types

### Dylib Modules (Native)
- Compiled as `.so` (Linux), `.dll` (Windows), or `.dylib` (macOS)
- Full performance, direct system access
- Requires PolicyEngine approval for dangerous operations

### WASM Modules (Sandboxed)
- Compiled as `.wasm` with WASI support
- Isolated execution in wasmtime sandbox
- Safer for untrusted code

## Creating Your First Module

### Step 1: Define the Manifest

```rust
use iskin::sdk::*;

let manifest = ModuleManifest {
    name: "my_module".to_string(),
    version: "1.0.0".to_string(),
    description: "My custom module".to_string(),
    author: Some("Your Name".to_string()),
    tools: vec![
        ToolDefinition {
            name: "greet".to_string(),
            description: "Greet a user".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                },
                "required": ["name"]
            }),
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                }
            })),
        }
    ],
    dependencies: vec![],
    required_permissions: vec![],
    module_type: ModuleType::Dylib,
};
```

### Step 2: Implement the Module

```rust
use iskin::sdk::*;

pub struct MyModule {
    base: BaseModule,
}

impl MyModule {
    pub fn new() -> Self {
        let manifest = /* define manifest */;
        let mut base = BaseModule::new(manifest);
        
        // Register tool handlers
        base.register_tool("greet", |params| {
            let name = params["name"].as_str()
                .ok_or_else(|| ModuleError::InvalidParams("name required".to_string()))?;
            
            Ok(serde_json::json!({
                "message": format!("Hello, {}!", name)
            }))
        }).unwrap();
        
        Self { base }
    }
}

impl IskinModule for MyModule {
    fn init(&mut self) -> Result<(), ModuleError> {
        // Initialization logic
        Ok(())
    }
    
    fn manifest(&self) -> &ModuleManifest {
        self.base.manifest()
    }
    
    fn execute_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ModuleError> {
        self.base.execute_tool(tool_name, params)
    }
    
    fn shutdown(&mut self) -> Result<(), ModuleError> {
        // Cleanup logic
        Ok(())
    }
}
```

### Step 3: Use the Macro (Alternative)

```rust
use iskin::create_module;

// Quick module creation
let mut module = create_module!(
    "quick_module",
    "1.0.0",
    "Quick example",
    ["tool1", "tool2"]
);

// Register handlers
module.register_tool("tool1", |params| {
    Ok(serde_json::json!({"result": "done"}))
}).unwrap();
```

## Building Modules

### Dylib Module

Create `Cargo.toml`:

```toml
[package]
name = "my_module"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
name = "my_module"

[dependencies]
iskin = { path = "../iskin" }
serde_json = "1.0"
```

Build:

```bash
cargo build --release
# Output: target/release/libmy_module.so (Linux)
#         target/release/my_module.dll (Windows)
#         target/release/libmy_module.dylib (macOS)
```

### WASM Module

Add target:

```bash
rustup target add wasm32-wasi
```

Build:

```bash
cargo build --release --target wasm32-wasi
# Output: target/wasm32-wasi/release/my_module.wasm
```

## Permissions

Modules must declare required permissions:

```rust
required_permissions: vec![
    Permission::FileRead,
    Permission::FileWrite,
    Permission::CommandExec,
    Permission::NetworkAccess,
    Permission::DockerAccess,
    Permission::HotReload,
]
```

The PolicyEngine will enforce these permissions at runtime.

## Hot Reload

Modules can be reloaded without restarting ISKIN:

```rust
// From ISKIN core
lifecycle_manager.reload_module("my_module", "/path/to/module.so")?;
```

The old module is unloaded, new one loaded, and state transferred if compatible.

## Testing Modules

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_greet_tool() {
        let module = MyModule::new();
        let result = module.execute_tool(
            "greet",
            serde_json::json!({"name": "Alice"})
        ).unwrap();
        
        assert_eq!(result["message"], "Hello, Alice!");
    }
    
    #[test]
    fn test_invalid_params() {
        let module = MyModule::new();
        let result = module.execute_tool(
            "greet",
            serde_json::json!({})
        );
        
        assert!(matches!(result, Err(ModuleError::InvalidParams(_))));
    }
}
```

Run tests:

```bash
cargo test
```

## Example Modules

See `src-tauri/src/sdk/mod.rs` for examples:
- `file_utils`: File read/write operations
- `code_analyzer`: Code search and analysis

## Best Practices

1. **Validate Input**: Always validate parameters before processing
2. **Handle Errors**: Return descriptive `ModuleError` variants
3. **Document Tools**: Provide clear descriptions and schemas
4. **Minimize Permissions**: Request only necessary permissions
5. **Test Thoroughly**: Write unit tests for all tools
6. **Version Properly**: Follow semver for compatibility

## API Reference

### ModuleManifest
- `name`: Unique module identifier
- `version`: Semver version string
- `description`: Human-readable description
- `author`: Optional author name
- `tools`: List of provided tools
- `dependencies`: Required module dependencies
- `required_permissions`: Security permissions needed
- `module_type`: Dylib or Wasm

### ToolDefinition
- `name`: Tool identifier
- `description`: What the tool does
- `input_schema`: JSON Schema for parameters
- `output_schema`: Optional JSON Schema for result

### IskinModule Trait
- `init()`: Initialize module resources
- `manifest()`: Get module metadata
- `execute_tool()`: Run a tool with parameters
- `shutdown()`: Cleanup before unload

### ModuleError
- `InitError`: Initialization failed
- `ToolNotFound`: Requested tool doesn't exist
- `InvalidParams`: Parameter validation failed
- `ExecutionError`: Runtime error occurred
- `SecurityViolation`: Permission denied
- `VersionMismatch`: Incompatible version

## Support

For questions and issues:
- GitHub Issues: https://github.com/rexo-null/AI-Ecosystem-Interface/issues
- Documentation: USER_GUIDE.md
