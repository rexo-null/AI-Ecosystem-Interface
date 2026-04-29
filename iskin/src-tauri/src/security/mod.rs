// ISKIN Security Hardening - Audit Logger, Rate Limiter, Filesystem Whitelist
// Phase 8: Security Components

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use anyhow::Result;
use log::{info, warn, error};

/// Unique audit entry identifier
pub type AuditId = uuid::Uuid;

/// Action types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditAction {
    FileRead(String),
    FileWrite(String),
    FileDelete(String),
    CommandExec(String),
    NetworkRequest(String),
    ModuleReload(String),
    AgentAction(String),
    ConfigChange(String),
    AuthAttempt(String),
    SandboxEscape(String),
}

/// Action result status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionResult {
    Success,
    Denied(String),
    Error(String),
    RequiresConfirmation,
}

/// Single audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: AuditId,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub actor: String,
    pub target: Option<String>,
    pub result: ActionResult,
    pub rationale: Option<String>,
    pub metadata: HashMap<String, String>,
    pub hash: String,
}

impl AuditEntry {
    pub fn new(actor: String, action: AuditAction) -> Self {
        let mut entry = Self {
            id: AuditId::new_v4(),
            timestamp: Utc::now(),
            action,
            actor,
            target: None,
            result: ActionResult::Success,
            rationale: None,
            metadata: HashMap::new(),
            hash: String::new(),
        };
        entry.hash = entry.compute_hash();
        entry
    }

    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self.hash = self.compute_hash();
        self
    }

    pub fn with_result(mut self, result: ActionResult) -> Self {
        self.result = result;
        self.hash = self.compute_hash();
        self
    }

    pub fn with_rationale(mut self, rationale: String) -> Self {
        self.rationale = Some(rationale);
        self.hash = self.compute_hash();
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self.hash = self.compute_hash();
        self
    }

    fn compute_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.timestamp.timestamp().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Audit log storage and query interface
pub struct AuditLogger {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
    storage_path: PathBuf,
    max_entries: usize,
    auto_save: bool,
}

impl AuditLogger {
    pub fn new(storage_path: PathBuf, max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            storage_path,
            max_entries,
            auto_save: true,
        }
    }

    /// Log an action
    pub async fn log(&self, entry: AuditEntry) {
        let mut entries = self.entries.write().await;
        entries.push(entry);
        
        // Trim old entries if exceeding limit
        while entries.len() > self.max_entries {
            entries.remove(0);
        }
        
        info!("Audit entry logged: {}", entries.last().unwrap().id);
        
        if self.auto_save {
            drop(entries);
            let _ = self.save_to_disk().await;
        }
    }

    /// Log a file read action
    pub async fn log_file_read(&self, actor: &str, path: &str) {
        let entry = AuditEntry::new(
            actor.to_string(),
            AuditAction::FileRead(path.to_string()),
        ).with_target(path.to_string());
        self.log(entry).await;
    }

    /// Log a file write action
    pub async fn log_file_write(&self, actor: &str, path: &str, rationale: Option<&str>) {
        let mut entry = AuditEntry::new(
            actor.to_string(),
            AuditAction::FileWrite(path.to_string()),
        ).with_target(path.to_string());
        
        if let Some(r) = rationale {
            entry = entry.with_rationale(r.to_string());
        }
        
        self.log(entry).await;
    }

    /// Log a command execution
    pub async fn log_command(&self, actor: &str, command: &str, result: ActionResult) {
        let entry = AuditEntry::new(
            actor.to_string(),
            AuditAction::CommandExec(command.to_string()),
        ).with_result(result);
        self.log(entry).await;
    }

    /// Get all entries
    pub async fn get_all(&self) -> Vec<AuditEntry> {
        self.entries.read().await.clone()
    }

    /// Get entries by actor
    pub async fn get_by_actor(&self, actor: &str) -> Vec<AuditEntry> {
        self.entries.read().await.iter()
            .filter(|e| e.actor == actor)
            .cloned()
            .collect()
    }

    /// Get entries by action type
    pub async fn get_by_action(&self, action_type: &str) -> Vec<AuditEntry> {
        self.entries.read().await.iter()
            .filter(|e| format!("{:?}", e.action).contains(action_type))
            .cloned()
            .collect()
    }

    /// Get denied actions
    pub async fn get_denied_actions(&self) -> Vec<AuditEntry> {
        self.entries.read().await.iter()
            .filter(|e| matches!(e.result, ActionResult::Denied(_)))
            .cloned()
            .collect()
    }

    /// Get entries in time range
    pub async fn get_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<AuditEntry> {
        self.entries.read().await.iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Search entries by keyword
    pub async fn search(&self, keyword: &str) -> Vec<AuditEntry> {
        let keyword_lower = keyword.to_lowercase();
        self.entries.read().await.iter()
            .filter(|e| {
                e.actor.to_lowercase().contains(&keyword_lower) ||
                format!("{:?}", e.action).to_lowercase().contains(&keyword_lower) ||
                e.target.as_ref().map_or(false, |t| t.to_lowercase().contains(&keyword_lower)) ||
                e.rationale.as_ref().map_or(false, |r| r.to_lowercase().contains(&keyword_lower))
            })
            .cloned()
            .collect()
    }

    /// Save to disk
    pub async fn save_to_disk(&self) -> Result<()> {
        std::fs::create_dir_all(&self.storage_path)?;
        let path = self.storage_path.join("audit_log.json");
        let entries = self.entries.read().await;
        let json = serde_json::to_string_pretty(&*entries)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load from disk
    pub async fn load_from_disk(&self) -> Result<()> {
        let path = self.storage_path.join("audit_log.json");
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let entries: Vec<AuditEntry> = serde_json::from_str(&content)?;
            *self.entries.write().await = entries;
        }
        Ok(())
    }

    /// Clear all entries
    pub async fn clear(&self) {
        *self.entries.write().await = Vec::new();
        info!("Audit log cleared");
    }

    /// Get statistics
    pub async fn get_stats(&self) -> AuditStats {
        let entries = self.entries.read().await;
        let total = entries.len();
        let denied = entries.iter().filter(|e| matches!(e.result, ActionResult::Denied(_))).count();
        let errors = entries.iter().filter(|e| matches!(e.result, ActionResult::Error(_))).count();
        
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        for entry in entries.iter() {
            let action_type = format!("{:?}", entry.action).split('(').next().unwrap_or("Unknown").to_string();
            *action_counts.entry(action_type).or_insert(0) += 1;
        }
        
        AuditStats {
            total_entries: total,
            denied_count: denied,
            error_count: errors,
            action_breakdown: action_counts,
        }
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: usize,
    pub denied_count: usize,
    pub error_count: usize,
    pub action_breakdown: HashMap<String, usize>,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u64,
    pub max_requests_per_hour: u64,
    pub burst_size: u64,
    pub cooldown_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
            burst_size: 10,
            cooldown_seconds: 60,
        }
    }
}

/// Rate limiter state per entity
#[derive(Debug, Clone)]
struct RateLimitState {
    requests_last_minute: Vec<DateTime<Utc>>,
    requests_last_hour: Vec<DateTime<Utc>>,
    consecutive_violations: u32,
    last_violation: Option<DateTime<Utc>>,
    is_cooldown: bool,
    cooldown_until: Option<DateTime<Utc>>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            requests_last_minute: Vec::new(),
            requests_last_hour: Vec::new(),
            consecutive_violations: 0,
            last_violation: None,
            is_cooldown: false,
            cooldown_until: None,
        }
    }
}

/// Rate Limiter - controls request frequency per actor/entity
pub struct RateLimiter {
    config: RateLimitConfig,
    states: Arc<RwLock<HashMap<String, RateLimitState>>>,
    violations: Arc<RwLock<HashMap<String, u32>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            states: Arc::new(RwLock::new(HashMap::new())),
            violations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request is allowed
    pub async fn check_rate_limit(&self, entity_id: &str) -> RateLimitResult {
        let now = Utc::now();
        let mut states = self.states.write().await;
        let state = states.entry(entity_id.to_string()).or_default();
        
        // Check if in cooldown
        if state.is_cooldown {
            if let Some(until) = state.cooldown_until {
                if now < until {
                    return RateLimitResult::Denied(format!("In cooldown until {:?}", until));
                } else {
                    // Cooldown expired
                    state.is_cooldown = false;
                    state.cooldown_until = None;
                    state.consecutive_violations = 0;
                }
            }
        }
        
        // Clean old entries
        let minute_ago = now - chrono::Duration::minutes(1);
        let hour_ago = now - chrono::Duration::hours(1);
        
        state.requests_last_minute.retain(|&t| t > minute_ago);
        state.requests_last_hour.retain(|&t| t > hour_ago);
        
        // Check limits
        if state.requests_last_minute.len() as u64 >= self.config.max_requests_per_minute {
            self.record_violation(entity_id, state).await;
            return RateLimitResult::Denied("Rate limit exceeded (per minute)".to_string());
        }
        
        if state.requests_last_hour.len() as u64 >= self.config.max_requests_per_hour {
            self.record_violation(entity_id, state).await;
            return RateLimitResult::Denied("Rate limit exceeded (per hour)".to_string());
        }
        
        // Allow request
        state.requests_last_minute.push(now);
        state.requests_last_hour.push(now);
        
        RateLimitResult::Allowed {
            remaining_per_minute: self.config.max_requests_per_minute - state.requests_last_minute.len() as u64,
            remaining_per_hour: self.config.max_requests_per_hour - state.requests_last_hour.len() as u64,
        }
    }

    async fn record_violation(&self, entity_id: &str, state: &mut RateLimitState) {
        state.consecutive_violations += 1;
        state.last_violation = Some(Utc::now());
        
        let mut violations = self.violations.write().await;
        *violations.entry(entity_id.to_string()).or_insert(0) += 1;
        
        // Enter cooldown after multiple violations
        if state.consecutive_violations >= 3 {
            state.is_cooldown = true;
            state.cooldown_until = Some(Utc::now() + chrono::Duration::seconds(self.config.cooldown_seconds as i64));
            warn!("Entity {} entered cooldown due to repeated violations", entity_id);
        }
    }

    /// Get current rate limit status for an entity
    pub async fn get_status(&self, entity_id: &str) -> RateLimitStatus {
        let states = self.states.read().await;
        let state = states.get(entity_id).unwrap_or(&RateLimitState::default());
        let violations = self.violations.read().await;
        
        RateLimitStatus {
            entity_id: entity_id.to_string(),
            requests_last_minute: state.requests_last_minute.len(),
            requests_last_hour: state.requests_last_hour.len(),
            consecutive_violations: state.consecutive_violations,
            total_violations: *violations.get(entity_id).unwrap_or(&0),
            is_cooldown: state.is_cooldown,
            cooldown_remaining: state.cooldown_until.map(|until| {
                (until - Utc::now()).num_seconds().max(0) as u64
            }),
        }
    }

    /// Reset rate limit for an entity
    pub async fn reset(&self, entity_id: &str) {
        let mut states = self.states.write().await;
        states.remove(entity_id);
        let mut violations = self.violations.write().await;
        violations.remove(entity_id);
        info!("Rate limit reset for entity: {}", entity_id);
    }

    /// Get all violations
    pub async fn get_all_violations(&self) -> HashMap<String, u32> {
        self.violations.read().await.clone()
    }
}

/// Rate limit check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitResult {
    Allowed {
        remaining_per_minute: u64,
        remaining_per_hour: u64,
    },
    Denied(String),
}

/// Rate limit status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub entity_id: String,
    pub requests_last_minute: usize,
    pub requests_last_hour: usize,
    pub consecutive_violations: u32,
    pub total_violations: u32,
    pub is_cooldown: bool,
    pub cooldown_remaining: Option<u64>,
}

/// Filesystem access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPolicy {
    ReadOnly,
    ReadWrite,
    Deny,
}

/// Filesystem whitelist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub path: PathBuf,
    pub policy: AccessPolicy,
    pub recursive: bool,
    pub allowed_extensions: Option<Vec<String>>,
    pub max_file_size_mb: Option<u64>,
    pub description: String,
}

/// Filesystem Whitelist - controls which paths can be accessed
pub struct FilesystemWhitelist {
    entries: Arc<RwLock<Vec<WhitelistEntry>>>,
    default_policy: AccessPolicy,
    log_violations: bool,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl FilesystemWhitelist {
    pub fn new(default_policy: AccessPolicy) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            default_policy,
            log_violations: true,
            audit_logger: None,
        }
    }

    pub fn with_audit_logger(mut self, logger: Arc<AuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    /// Add a path to whitelist
    pub async fn add_entry(&self, entry: WhitelistEntry) {
        let mut entries = self.entries.write().await;
        entries.push(entry);
        info!("Added whitelist entry");
    }

    /// Remove entry by path
    pub async fn remove_entry(&self, path: &Path) {
        let mut entries = self.entries.write().await;
        entries.retain(|e| e.path != path);
        info!("Removed whitelist entry: {:?}", path);
    }

    /// Check if path access is allowed
    pub async fn check_access(&self, path: &Path, write: bool) -> AccessCheckResult {
        let path_str = path.to_string_lossy();
        let entries = self.entries.read().await;
        
        // Find matching entry
        let mut best_match: Option<&WhitelistEntry> = None;
        let mut best_match_len = 0;
        
        for entry in entries.iter() {
            let entry_path = entry.path.to_string_lossy();
            
            if entry.recursive {
                if path_str.starts_with(&entry_path.to_string()) {
                    if entry_path.len() > best_match_len {
                        best_match = Some(entry);
                        best_match_len = entry_path.len();
                    }
                }
            } else {
                if path_str == entry_path {
                    best_match = Some(entry);
                    break;
                }
            }
        }
        
        // Determine policy
        let policy = best_match.map(|e| &e.policy).unwrap_or(&self.default_policy);
        
        match policy {
            AccessPolicy::Deny => {
                let result = AccessCheckResult::Denied("Path not in whitelist".to_string());
                if self.log_violations {
                    if let Some(logger) = &self.audit_logger {
                        logger.log_file_write("system", &path_str).await;
                    }
                }
                result
            },
            AccessPolicy::ReadOnly if write => {
                AccessCheckResult::Denied("Path is read-only".to_string())
            },
            AccessPolicy::ReadOnly | AccessPolicy::ReadWrite => {
                // Check extension restrictions
                if let Some(entry) = best_match {
                    if let Some(allowed_exts) = &entry.allowed_extensions {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if !allowed_exts.contains(&ext.to_lowercase()) {
                                return AccessCheckResult::Denied(
                                    format!("Extension '{}' not allowed", ext)
                                );
                            }
                        }
                    }
                    
                    // Check file size for writes
                    if write {
                        if let Some(max_size) = entry.max_file_size_mb {
                            if let Ok(metadata) = std::fs::metadata(path) {
                                let size_mb = metadata.len() / (1024 * 1024);
                                if size_mb > max_size {
                                    return AccessCheckResult::Denied(
                                        format!("File size {}MB exceeds limit {}MB", size_mb, max_size)
                                    );
                                }
                            }
                        }
                    }
                }
                
                AccessCheckResult::Allowed
            },
        }
    }

    /// Check read access
    pub async fn check_read(&self, path: &Path) -> AccessCheckResult {
        self.check_access(path, false).await
    }

    /// Check write access
    pub async fn check_write(&self, path: &Path) -> AccessCheckResult {
        self.check_access(path, true).await
    }

    /// Get all whitelist entries
    pub async fn get_entries(&self) -> Vec<WhitelistEntry> {
        self.entries.read().await.clone()
    }

    /// Enable/disable violation logging
    pub fn set_log_violations(&mut self, enabled: bool) {
        self.log_violations = enabled;
    }
}

/// Access check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessCheckResult {
    Allowed,
    Denied(String),
}

impl AccessCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, AccessCheckResult::Allowed)
    }
}

/// Integrated Security System
pub struct SecuritySystem {
    pub audit_logger: Arc<AuditLogger>,
    pub rate_limiter: RateLimiter,
    pub filesystem_whitelist: FilesystemWhitelist,
}

impl SecuritySystem {
    pub fn new(storage_path: PathBuf) -> Self {
        let audit_logger = Arc::new(AuditLogger::new(storage_path.join("audit"), 10000));
        
        Self {
            audit_logger: audit_logger.clone(),
            rate_limiter: RateLimiter::new(RateLimitConfig::default()),
            filesystem_whitelist: FilesystemWhitelist::new(AccessPolicy::Deny)
                .with_audit_logger(audit_logger),
        }
    }

    /// Initialize with default safe paths
    pub async fn initialize_defaults(&self, workspace_root: &Path) -> Result<()> {
        // Add workspace as read-write
        self.filesystem_whitelist.add_entry(WhitelistEntry {
            path: workspace_root.to_path_buf(),
            policy: AccessPolicy::ReadWrite,
            recursive: true,
            allowed_extensions: None,
            max_file_size_mb: Some(100),
            description: "Workspace root - full access".to_string(),
        }).await;

        // Add temp directory
        if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
            self.filesystem_whitelist.add_entry(WhitelistEntry {
                path: temp_dir,
                policy: AccessPolicy::ReadWrite,
                recursive: true,
                allowed_extensions: None,
                max_file_size_mb: Some(50),
                description: "Temporary directory".to_string(),
            }).await;
        }

        // Block sensitive system paths explicitly
        for path in ["/etc", "/root", "/home"] {
            if Path::new(path).exists() {
                self.filesystem_whitelist.add_entry(WhitelistEntry {
                    path: PathBuf::from(path),
                    policy: AccessPolicy::Deny,
                    recursive: true,
                    allowed_extensions: None,
                    max_file_size_mb: None,
                    description: "System protected path".to_string(),
                }).await;
            }
        }

        info!("Security system initialized with default paths");
        Ok(())
    }

    /// Validate and log a file operation
    pub async fn validate_file_operation(
        &self,
        actor: &str,
        path: &Path,
        write: bool,
        rationale: Option<&str>,
    ) -> Result<bool> {
        // Check whitelist
        let access_result = if write {
            self.filesystem_whitelist.check_write(path).await
        } else {
            self.filesystem_whitelist.check_read(path).await
        };

        // Log the attempt
        if write {
            self.audit_logger.log_file_write(actor, &path.to_string_lossy(), rationale).await;
        } else {
            self.audit_logger.log_file_read(actor, &path.to_string_lossy()).await;
        }

        match access_result {
            AccessCheckResult::Allowed => Ok(true),
            AccessCheckResult::Denied(reason) => {
                warn!("File access denied for {}: {} - {}", actor, path.display(), reason);
                Err(anyhow::anyhow!("Access denied: {}", reason))
            }
        }
    }

    /// Validate and log a command execution
    pub async fn validate_command(
        &self,
        actor: &str,
        command: &str,
    ) -> Result<bool> {
        // Check rate limit
        let rate_result = self.rate_limiter.check_rate_limit(actor).await;
        
        match rate_result {
            RateLimitResult::Allowed { .. } => {
                self.audit_logger.log_command(actor, command, ActionResult::Success).await;
                Ok(true)
            },
            RateLimitResult::Denied(reason) => {
                self.audit_logger.log_command(actor, command, ActionResult::Denied(reason.clone())).await;
                Err(anyhow::anyhow!("Rate limited: {}", reason))
            }
        }
    }

    /// Get comprehensive security stats
    pub async fn get_stats(&self) -> SecurityStats {
        let audit_stats = self.audit_logger.get_stats().await;
        let violations = self.rate_limiter.get_all_violations().await;
        
        SecurityStats {
            audit: audit_stats,
            total_rate_violations: violations.values().sum(),
            entities_with_violations: violations.len(),
        }
    }
}

/// Comprehensive security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub audit: AuditStats,
    pub total_rate_violations: u32,
    pub entities_with_violations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_logger() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf(), 100);
        
        logger.log_file_read("user1", "/test/file.txt").await;
        logger.log_file_write("user1", "/test/output.rs", Some("Creating new file")).await;
        
        let entries = logger.get_all().await;
        assert_eq!(entries.len(), 2);
        
        let stats = logger.get_stats().await;
        assert_eq!(stats.total_entries, 2);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut config = RateLimitConfig::default();
        config.max_requests_per_minute = 5;
        let limiter = RateLimiter::new(config);
        
        // First requests should succeed
        for _ in 0..5 {
            let result = limiter.check_rate_limit("test_entity").await;
            assert!(matches!(result, RateLimitResult::Allowed { .. }));
        }
        
        // Sixth request should fail
        let result = limiter.check_rate_limit("test_entity").await;
        assert!(matches!(result, RateLimitResult::Denied(_)));
    }

    #[tokio::test]
    async fn test_filesystem_whitelist() {
        let whitelist = FilesystemWhitelist::new(AccessPolicy::Deny);
        
        // Add a test path
        let temp_dir = TempDir::new().unwrap();
        whitelist.add_entry(WhitelistEntry {
            path: temp_dir.path().to_path_buf(),
            policy: AccessPolicy::ReadWrite,
            recursive: true,
            allowed_extensions: None,
            max_file_size_mb: None,
            description: "Test directory".to_string(),
        }).await;
        
        // Should allow access to temp dir
        let result = whitelist.check_read(temp_dir.path()).await;
        assert!(result.is_allowed());
        
        // Should deny access to other paths
        let result = whitelist.check_read(Path::new("/etc/passwd")).await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_security_system() {
        let temp_dir = TempDir::new().unwrap();
        let system = SecuritySystem::new(temp_dir.path().to_path_buf());
        system.initialize_defaults(temp_dir.path()).await.unwrap();
        
        // Test file validation
        let result = system.validate_file_operation(
            "test_agent",
            temp_dir.path().join("test.txt").as_path(),
            true,
            Some("Test write"),
        ).await;
        
        assert!(result.is_ok());
    }
}
