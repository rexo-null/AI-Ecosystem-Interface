//! ISKIN Core Library
//! 
//! Main library entry point exposing all public modules and SDK.

pub mod core;
pub mod agent;
pub mod memory;
pub mod sandbox;
pub mod tools;
pub mod modules;
pub mod llm;
pub mod terminal;
pub mod security;
pub mod self_improvement;
pub mod updater;
pub mod api;
pub mod vision;

// Public SDK for module developers
pub mod sdk;

// Re-export main types for convenience
pub use sdk::{ModuleManifest, ModuleType, Permission, ToolDefinition, IskinModule, ModuleError, BaseModule};
pub use vision::{QwenVLConfig, QwenVLEngine, VisionAnalysis, VisionError};
