use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

/// Resource limits for sandboxed operations
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub max_disk_mb: u64,
    pub max_network_connections: u32,
    pub timeout_seconds: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 2048,
            max_cpu_percent: 50.0,
            max_disk_mb: 10240,
            max_network_connections: 10,
            timeout_seconds: 300,
        }
    }
}

/// Resource Manager - tracks and limits resource usage
pub struct ResourceManager {
    limits: Arc<RwLock<ResourceLimits>>,
    current_usage: Arc<RwLock<ResourceUsage>>,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_mb: u64,
    pub cpu_percent: f32,
    pub disk_mb: u64,
    pub active_connections: u32,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            limits: Arc::new(RwLock::new(ResourceLimits::default())),
            current_usage: Arc::new(RwLock::new(ResourceUsage::default())),
        }
    }

    /// Set resource limits
    pub async fn set_limits(&self, limits: ResourceLimits) {
        let mut current = self.limits.write().await;
        *current = limits;
        info!("Resource limits updated: {:?}", limits);
    }

    /// Get current limits
    pub async fn get_limits(&self) -> ResourceLimits {
        self.limits.read().await.clone()
    }

    /// Update current usage
    pub async fn update_usage(&self, usage: ResourceUsage) {
        let mut current = self.current_usage.write().await;
        *current = usage;
    }

    /// Check if operation would exceed limits
    pub async fn check_limits(&self, required: &ResourceUsage) -> Result<bool> {
        let limits = self.limits.read().await;
        let current = self.current_usage.read().await;

        if current.memory_mb + required.memory_mb > limits.max_memory_mb {
            return Ok(false);
        }
        if current.cpu_percent + required.cpu_percent > limits.max_cpu_percent {
            return Ok(false);
        }
        if current.disk_mb + required.disk_mb > limits.max_disk_mb {
            return Ok(false);
        }
        if current.active_connections + required.active_connections > limits.max_network_connections {
            return Ok(false);
        }

        Ok(true)
    }

    /// Reserve resources for an operation
    pub async fn reserve_resources(&self, required: &ResourceUsage) -> Result<bool> {
        if !self.check_limits(required).await? {
            return Ok(false);
        }

        let mut current = self.current_usage.write().await;
        current.memory_mb += required.memory_mb;
        current.cpu_percent += required.cpu_percent;
        current.disk_mb += required.disk_mb;
        current.active_connections += required.active_connections;

        Ok(true)
    }

    /// Release reserved resources
    pub async fn release_resources(&self, used: &ResourceUsage) {
        let mut current = self.current_usage.write().await;
        current.memory_mb = current.memory_mb.saturating_sub(used.memory_mb);
        current.cpu_percent = current.cpu_percent.saturating_sub(used.cpu_percent);
        current.disk_mb = current.disk_mb.saturating_sub(used.disk_mb);
        current.active_connections = current.active_connections.saturating_sub(used.active_connections);
    }
}
