// ISKIN Modules - Dynamic Module Definitions for Self-Improvement

pub mod tool_module;
pub mod memory_module;
pub mod agent_module;

// Module trait that all dynamic modules must implement
#[async_trait::async_trait]
pub trait ISKINModule: Send + Sync {
    /// Module identifier
    fn id(&self) -> &str;
    
    /// Module name (human-readable)
    fn name(&self) -> &str;
    
    /// Module version
    fn version(&self) -> &str;
    
    /// Initialize the module
    async fn initialize(&self) -> anyhow::Result<()>;
    
    /// Shutdown the module gracefully
    async fn shutdown(&self) -> anyhow::Result<()>;
    
    /// Execute a module command (for RPC-style calls)
    async fn execute(&self, command: &str, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}
