use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};

use super::container::{ContainerManager, ContainerStatus};

/// Error pattern for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub severity: ErrorSeverity,
    pub recovery_strategy: RecoveryStrategy,
    pub occurrences: u32,
    pub last_seen: i64,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Recovery strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecoveryStrategy {
    Restart,
    Recreate,
    ScaleUp,
    Notify,
    Ignore,
    Custom(String),
}

impl RecoveryStrategy {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Restart => "restart",
            Self::Recreate => "recreate",
            Self::ScaleUp => "scale_up",
            Self::Notify => "notify",
            Self::Ignore => "ignore",
            Self::Custom(_) => "custom",
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub container_id: String,
    pub healthy: bool,
    pub status: String,
    pub message: String,
    pub checked_at: i64,
    pub recovery_action: Option<String>,
}

/// Self-healing event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingEvent {
    pub id: String,
    pub container_id: String,
    pub error_pattern: Option<String>,
    pub action_taken: String,
    pub success: bool,
    pub message: String,
    pub timestamp: i64,
}

/// Self-healing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingStats {
    pub total_checks: u64,
    pub total_recoveries: u64,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub active_monitors: usize,
    pub error_patterns_count: usize,
    pub uptime_percentage: f64,
}

/// Self-Healing Loop - monitors containers and auto-recovers from failures
pub struct SelfHealingLoop {
    error_patterns: Arc<RwLock<Vec<ErrorPattern>>>,
    events: Arc<RwLock<Vec<HealingEvent>>>,
    monitored_containers: Arc<RwLock<Vec<String>>>,
    is_running: Arc<RwLock<bool>>,
    max_restart_count: u32,
    check_interval_secs: u64,
    stats: Arc<RwLock<HealingStats>>,
}

impl SelfHealingLoop {
    pub fn new() -> Self {
        let mut default_patterns = Vec::new();

        // Default error patterns
        default_patterns.push(ErrorPattern {
            id: "oom_killer".to_string(),
            name: "Out of Memory".to_string(),
            pattern: "OOMKilled|out of memory|Cannot allocate memory".to_string(),
            severity: ErrorSeverity::Critical,
            recovery_strategy: RecoveryStrategy::Recreate,
            occurrences: 0,
            last_seen: 0,
        });

        default_patterns.push(ErrorPattern {
            id: "crash_loop".to_string(),
            name: "Crash Loop".to_string(),
            pattern: "CrashLoopBackOff|restarting|exit code 137".to_string(),
            severity: ErrorSeverity::High,
            recovery_strategy: RecoveryStrategy::Restart,
            occurrences: 0,
            last_seen: 0,
        });

        default_patterns.push(ErrorPattern {
            id: "network_error".to_string(),
            name: "Network Error".to_string(),
            pattern: "connection refused|ECONNREFUSED|timeout".to_string(),
            severity: ErrorSeverity::Medium,
            recovery_strategy: RecoveryStrategy::Restart,
            occurrences: 0,
            last_seen: 0,
        });

        default_patterns.push(ErrorPattern {
            id: "disk_full".to_string(),
            name: "Disk Full".to_string(),
            pattern: "No space left on device|ENOSPC".to_string(),
            severity: ErrorSeverity::High,
            recovery_strategy: RecoveryStrategy::Notify,
            occurrences: 0,
            last_seen: 0,
        });

        Self {
            error_patterns: Arc::new(RwLock::new(default_patterns)),
            events: Arc::new(RwLock::new(Vec::new())),
            monitored_containers: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
            max_restart_count: 3,
            check_interval_secs: 30,
            stats: Arc::new(RwLock::new(HealingStats {
                total_checks: 0,
                total_recoveries: 0,
                successful_recoveries: 0,
                failed_recoveries: 0,
                active_monitors: 0,
                error_patterns_count: 4,
                uptime_percentage: 100.0,
            })),
        }
    }

    /// Start monitoring a container
    pub async fn monitor(&self, container_id: &str) -> Result<()> {
        let mut monitored = self.monitored_containers.write().await;
        if !monitored.contains(&container_id.to_string()) {
            monitored.push(container_id.to_string());
            self.stats.write().await.active_monitors = monitored.len();
            info!("Self-healing: now monitoring container {}", container_id);
        }
        Ok(())
    }

    /// Stop monitoring a container
    pub async fn unmonitor(&self, container_id: &str) -> Result<()> {
        let mut monitored = self.monitored_containers.write().await;
        monitored.retain(|id| id != container_id);
        self.stats.write().await.active_monitors = monitored.len();
        info!("Self-healing: stopped monitoring container {}", container_id);
        Ok(())
    }

    /// Run a single health check cycle on all monitored containers
    pub async fn check_health(&self, container_manager: &ContainerManager) -> Vec<HealthCheckResult> {
        let monitored = self.monitored_containers.read().await.clone();
        let mut results = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for container_id in &monitored {
            let mut stats = self.stats.write().await;
            stats.total_checks += 1;
            drop(stats);

            let healthy = match container_manager.health_check(container_id).await {
                Ok(is_healthy) => is_healthy,
                Err(e) => {
                    warn!("Health check failed for {}: {}", container_id, e);
                    false
                }
            };

            let (status, message, recovery_action) = if healthy {
                ("healthy".to_string(), "Container is running".to_string(), None)
            } else {
                let container = container_manager.get(container_id).await;
                let (status, msg) = match &container {
                    Some(c) => (c.status.as_str().to_string(), format!("Container status: {}", c.status.as_str())),
                    None => ("not_found".to_string(), "Container not found".to_string()),
                };

                let recovery = if let Some(c) = &container {
                    if c.health_check_failures >= self.max_restart_count {
                        Some("notify".to_string())
                    } else {
                        Some("restart".to_string())
                    }
                } else {
                    None
                };

                (status, msg, recovery)
            };

            // Attempt recovery if needed
            if let Some(action) = &recovery_action {
                let event = self.attempt_recovery(container_id, action, container_manager).await;
                self.events.write().await.push(event);
            }

            results.push(HealthCheckResult {
                container_id: container_id.clone(),
                healthy,
                status,
                message,
                checked_at: now,
                recovery_action,
            });
        }

        results
    }

    /// Attempt automatic recovery for a container
    async fn attempt_recovery(
        &self,
        container_id: &str,
        action: &str,
        container_manager: &ContainerManager,
    ) -> HealingEvent {
        let now = chrono::Utc::now().timestamp();
        let event_id = format!("heal_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0000"));

        info!("Self-healing: attempting {} for container {}", action, container_id);

        let (success, message) = match action {
            "restart" => {
                match container_manager.restart(container_id).await {
                    Ok(_) => {
                        let mut stats = self.stats.write().await;
                        stats.total_recoveries += 1;
                        stats.successful_recoveries += 1;
                        (true, "Container restarted successfully".to_string())
                    }
                    Err(e) => {
                        let mut stats = self.stats.write().await;
                        stats.total_recoveries += 1;
                        stats.failed_recoveries += 1;
                        (false, format!("Restart failed: {}", e))
                    }
                }
            }
            "notify" => {
                warn!("Self-healing: container {} exceeded max restarts, notifying", container_id);
                (true, "Max restarts exceeded, notification sent".to_string())
            }
            _ => {
                (false, format!("Unknown recovery action: {}", action))
            }
        };

        HealingEvent {
            id: event_id,
            container_id: container_id.to_string(),
            error_pattern: None,
            action_taken: action.to_string(),
            success,
            message,
            timestamp: now,
        }
    }

    /// Analyze container logs for known error patterns
    pub async fn analyze_logs(&self, logs: &[String]) -> Vec<ErrorPattern> {
        let patterns = self.error_patterns.read().await;
        let mut matched = Vec::new();
        let log_text = logs.join("\n");

        for pattern in patterns.iter() {
            let regex = match regex::Regex::new(&pattern.pattern) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if regex.is_match(&log_text) {
                let mut p = pattern.clone();
                p.occurrences += 1;
                p.last_seen = chrono::Utc::now().timestamp();
                matched.push(p);
            }
        }

        matched
    }

    /// Add a custom error pattern
    pub async fn add_pattern(&self, pattern: ErrorPattern) -> Result<()> {
        // Validate regex
        regex::Regex::new(&pattern.pattern)
            .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

        self.error_patterns.write().await.push(pattern.clone());
        self.stats.write().await.error_patterns_count += 1;
        info!("Self-healing: added error pattern '{}'", pattern.name);
        Ok(())
    }

    /// Remove an error pattern
    pub async fn remove_pattern(&self, pattern_id: &str) -> Result<()> {
        let mut patterns = self.error_patterns.write().await;
        let initial_len = patterns.len();
        patterns.retain(|p| p.id != pattern_id);

        if patterns.len() == initial_len {
            return Err(anyhow::anyhow!("Pattern not found: {}", pattern_id));
        }

        self.stats.write().await.error_patterns_count = patterns.len();
        info!("Self-healing: removed error pattern {}", pattern_id);
        Ok(())
    }

    /// Get all error patterns
    pub async fn list_patterns(&self) -> Vec<ErrorPattern> {
        self.error_patterns.read().await.clone()
    }

    /// Get healing event history
    pub async fn get_events(&self, limit: usize) -> Vec<HealingEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get healing statistics
    pub async fn get_stats(&self) -> HealingStats {
        self.stats.read().await.clone()
    }

    /// Get list of monitored container IDs
    pub async fn list_monitored(&self) -> Vec<String> {
        self.monitored_containers.read().await.clone()
    }

    /// Check if monitoring loop is active
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}
