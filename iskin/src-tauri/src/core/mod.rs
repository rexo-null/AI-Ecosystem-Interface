// ISKIN Core - Lifecycle Manager & Module System
// This is the immutable core that manages dynamic modules

pub mod lifecycle;
pub mod security;
pub mod resources;

pub use lifecycle::LifecycleManager;
pub use security::PolicyEngine;
pub use resources::ResourceManager;
