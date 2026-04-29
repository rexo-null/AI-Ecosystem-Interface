use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};

/// Action types that require user confirmation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum DangerousAction {
    FileDelete(String),
    FileOverwrite(String),
    SystemCommand(String),
    PackageInstall(String),
    NetworkRequest(String),
    ModuleReload(String),
}

/// Security policy level
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyLevel {
    Strict,    // Always confirm dangerous actions
    Balanced,  // Confirm only critical actions
    Permissive // Allow most actions with logging
}

/// Policy Engine - controls what the agent can do autonomously
#[allow(dead_code)]
pub struct PolicyEngine {
    level: Arc<RwLock<PolicyLevel>>,
    blocked_actions: Arc<RwLock<HashSet<DangerousAction>>>,
    allowed_patterns: Arc<RwLock<HashSet<String>>>,
}

#[allow(dead_code)]
impl PolicyEngine {
    pub fn new(default_level: PolicyLevel) -> Self {
        Self {
            level: Arc::new(RwLock::new(default_level)),
            blocked_actions: Arc::new(RwLock::new(HashSet::new())),
            allowed_patterns: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Set security policy level
    pub async fn set_policy_level(&self, level: PolicyLevel) {
        let mut current_level = self.level.write().await;
        *current_level = level;
        info!("Security policy level set to: {:?}", level);
    }

    /// Get current policy level
    pub async fn get_policy_level(&self) -> PolicyLevel {
        *self.level.read().await
    }

    /// Check if an action requires user confirmation
    pub async fn requires_confirmation(&self, action: &DangerousAction) -> bool {
        let level = *self.level.read().await;
        
        // Always check blocked list first
        if self.blocked_actions.read().await.contains(action) {
            return true;
        }

        match level {
            PolicyLevel::Strict => {
                matches!(action, 
                    DangerousAction::FileDelete(_) |
                    DangerousAction::FileOverwrite(_) |
                    DangerousAction::SystemCommand(_) |
                    DangerousAction::PackageInstall(_) |
                    DangerousAction::ModuleReload(_)
                )
            }
            PolicyLevel::Balanced => match action {
                DangerousAction::FileDelete(_) => true,
                DangerousAction::SystemCommand(cmd) if cmd.contains("rm -rf") || cmd.contains("sudo") || cmd.contains("format") => true,
                _ => false,
            },
            PolicyLevel::Permissive => false,
        }
    }

    /// Add an action to the blocked list
    pub async fn block_action(&self, action: DangerousAction) {
        info!("Action blocked: {:?}", &action);
        self.blocked_actions.write().await.insert(action);
    }

    /// Remove an action from the blocked list
    pub async fn unblock_action(&self, action: &DangerousAction) {
        self.blocked_actions.write().await.remove(action);
        info!("Action unblocked: {:?}", action);
    }

    /// Add an allowed pattern (e.g., "npm install *", "cargo build")
    pub async fn add_allowed_pattern(&self, pattern: String) {
        self.allowed_patterns.write().await.insert(pattern);
    }

    /// Check if action matches an allowed pattern
    pub async fn is_allowed_by_pattern(&self, action: &str) -> bool {
        let patterns = self.allowed_patterns.read().await;
        patterns.iter().any(|p| {
            // Simple glob matching - could be enhanced with proper glob crate
            action.contains(p.trim_end_matches('*'))
        })
    }

    /// Validate a command before execution
    pub async fn validate_command(&self, cmd: &str) -> Result<bool> {
        let action = DangerousAction::SystemCommand(cmd.to_string());
        
        if self.requires_confirmation(&action).await {
            return Ok(false); // Requires user confirmation
        }

        if self.is_allowed_by_pattern(cmd).await {
            return Ok(true);
        }

        // Additional validation logic here
        // Check for dangerous patterns
        let dangerous_patterns = ["rm -rf /", "mkfs", "dd if=", ":(){:|:&};:"];
        for pattern in dangerous_patterns {
            if cmd.contains(pattern) {
                warn!("Blocked dangerous command pattern: {}", pattern);
                return Ok(false);
            }
        }

        Ok(true)
    }
}

// Example usage in Tauri commands
// Before executing any file system or system command:
// if !policy_engine.validate_command(&cmd).await? {
//     return Err(anyhow::anyhow!("Action requires user confirmation"));
// }
