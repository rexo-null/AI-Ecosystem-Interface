#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};
use bollard::Docker;
use bollard::container::{
    Config, CreateContainerOptions, StartContainerOptions,
    StopContainerOptions, RemoveContainerOptions, LogsOptions,
    ListContainersOptions, InspectContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::models::{HostConfig, PortBinding};
use futures_util::StreamExt;

/// Container status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Paused,
    Restarting,
    Removing,
    Exited(i64),
    Error(String),
    Unknown,
}

impl ContainerStatus {
    pub fn from_docker_state(state: &str, exit_code: Option<i64>) -> Self {
        match state {
            "created" => Self::Created,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "restarting" => Self::Restarting,
            "removing" => Self::Removing,
            "exited" | "dead" => Self::Exited(exit_code.unwrap_or(-1)),
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Created => "created",
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Paused => "paused",
            Self::Restarting => "restarting",
            Self::Removing => "removing",
            Self::Exited(_) => "exited",
            Self::Error(_) => "error",
            Self::Unknown => "unknown",
        }
    }
}

/// Container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub image: String,
    pub name: Option<String>,
    pub memory_limit_mb: Option<u64>,
    pub cpu_limit: Option<f64>,
    pub env_vars: HashMap<String, String>,
    pub ports: Vec<(u16, u16)>,
    pub volumes: Vec<(String, String)>,
    pub working_dir: Option<String>,
    pub command: Option<Vec<String>>,
    pub auto_remove: bool,
    pub network_mode: Option<String>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            image: "ubuntu:22.04".to_string(),
            name: None,
            memory_limit_mb: Some(512),
            cpu_limit: Some(1.0),
            env_vars: HashMap::new(),
            ports: Vec::new(),
            volumes: Vec::new(),
            working_dir: None,
            command: None,
            auto_remove: false,
            network_mode: None,
        }
    }
}

/// Managed container instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedContainer {
    pub id: String,
    pub docker_id: Option<String>,
    pub config: ContainerConfig,
    pub status: ContainerStatus,
    pub created_at: i64,
    pub logs: Vec<String>,
    pub health_check_failures: u32,
}

/// Exec result from running a command inside a container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
}

/// Docker daemon connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerStatus {
    pub connected: bool,
    pub version: Option<String>,
    pub api_version: Option<String>,
    pub containers_running: usize,
    pub images_count: usize,
}

/// Container Manager - Docker/Podman integration for sandboxed execution
pub struct ContainerManager {
    containers: Arc<RwLock<HashMap<String, ManagedContainer>>>,
    docker: Arc<RwLock<Option<Docker>>>,
    is_connected: Arc<RwLock<bool>>,
}

impl ContainerManager {
    pub fn new() -> Self {
        Self {
            containers: Arc::new(RwLock::new(HashMap::new())),
            docker: Arc::new(RwLock::new(None)),
            is_connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize Docker connection
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Docker connection...");

        match Docker::connect_with_local_defaults() {
            Ok(docker) => {
                match docker.version().await {
                    Ok(version) => {
                        info!(
                            "Docker connected: version={}, API={}",
                            version.version.as_deref().unwrap_or("unknown"),
                            version.api_version.as_deref().unwrap_or("unknown")
                        );
                        *self.docker.write().await = Some(docker);
                        *self.is_connected.write().await = true;
                    }
                    Err(e) => {
                        warn!("Docker daemon not responding: {}. Running in simulation mode.", e);
                        *self.is_connected.write().await = false;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to connect to Docker: {}. Running in simulation mode.", e);
                *self.is_connected.write().await = false;
            }
        }

        Ok(())
    }

    /// Get Docker daemon status
    pub async fn docker_status(&self) -> DockerStatus {
        let docker = self.docker.read().await;
        let connected = *self.is_connected.read().await;

        if !connected || docker.is_none() {
            return DockerStatus {
                connected: false,
                version: None,
                api_version: None,
                containers_running: 0,
                images_count: 0,
            };
        }

        let docker = docker.as_ref().unwrap();

        let (version_info, containers_count, images_count) = tokio::join!(
            docker.version(),
            async {
                let mut filters = HashMap::new();
                filters.insert("status", vec!["running"]);
                docker.list_containers(Some(ListContainersOptions {
                    filters,
                    ..Default::default()
                })).await.map(|c| c.len()).unwrap_or(0)
            },
            async {
                docker.list_images::<String>(None).await.map(|i| i.len()).unwrap_or(0)
            }
        );

        let (version, api_version) = match version_info {
            Ok(v) => (v.version, v.api_version),
            Err(_) => (None, None),
        };

        DockerStatus {
            connected,
            version,
            api_version,
            containers_running: containers_count,
            images_count,
        }
    }

    /// Pull a Docker image
    pub async fn pull_image(&self, image: &str) -> Result<()> {
        let docker = self.docker.read().await;
        let docker = docker.as_ref().ok_or_else(|| anyhow::anyhow!("Docker not connected"))?;

        let (repo, tag) = if image.contains(':') {
            let parts: Vec<&str> = image.splitn(2, ':').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (image.to_string(), "latest".to_string())
        };

        info!("Pulling image: {}:{}", repo, tag);

        let options = CreateImageOptions {
            from_image: repo.clone(),
            tag: tag.clone(),
            ..Default::default()
        };

        let mut stream = docker.create_image(Some(options), None, None);
        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        info!("Pull progress: {}", status);
                    }
                }
                Err(e) => {
                    error!("Pull error: {}", e);
                    return Err(anyhow::anyhow!("Failed to pull image: {}", e));
                }
            }
        }

        info!("Image pulled successfully: {}:{}", repo, tag);
        Ok(())
    }

    /// Create a new container
    pub async fn create(&self, config: ContainerConfig) -> Result<String> {
        let id = format!("iskin_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0000"));
        let now = chrono::Utc::now().timestamp();

        let docker_id = if *self.is_connected.read().await {
            let docker = self.docker.read().await;
            let docker = docker.as_ref().unwrap();

            // Build env vars
            let env: Vec<String> = config.env_vars.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();

            // Build port bindings
            let mut exposed_ports = HashMap::new();
            let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
            for (host_port, container_port) in &config.ports {
                let key = format!("{}/tcp", container_port);
                exposed_ports.insert(key.clone(), HashMap::new());
                port_bindings.insert(key, Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.to_string()),
                }]));
            }

            // Build volume binds
            let binds: Vec<String> = config.volumes.iter()
                .map(|(host, container)| format!("{}:{}", host, container))
                .collect();

            let host_config = HostConfig {
                memory: config.memory_limit_mb.map(|m| (m * 1024 * 1024) as i64),
                nano_cpus: config.cpu_limit.map(|c| (c * 1_000_000_000.0) as i64),
                binds: if binds.is_empty() { None } else { Some(binds) },
                port_bindings: if port_bindings.is_empty() { None } else { Some(port_bindings) },
                auto_remove: Some(config.auto_remove),
                network_mode: config.network_mode.clone(),
                ..Default::default()
            };

            let container_config = Config {
                image: Some(config.image.clone()),
                env: if env.is_empty() { None } else { Some(env) },
                exposed_ports: if exposed_ports.is_empty() { None } else { Some(exposed_ports) },
                working_dir: config.working_dir.clone(),
                cmd: config.command.clone(),
                host_config: Some(host_config),
                ..Default::default()
            };

            let options = config.name.as_ref().map(|n| CreateContainerOptions { name: n.as_str(), platform: None });

            match docker.create_container(options, container_config).await {
                Ok(response) => {
                    info!("Docker container created: {}", response.id);
                    Some(response.id)
                }
                Err(e) => {
                    error!("Failed to create Docker container: {}", e);
                    return Err(anyhow::anyhow!("Docker create failed: {}", e));
                }
            }
        } else {
            info!("Simulation mode: container {} created (no Docker)", id);
            None
        };

        let container = ManagedContainer {
            id: id.clone(),
            docker_id,
            config,
            status: ContainerStatus::Created,
            created_at: now,
            logs: Vec::new(),
            health_check_failures: 0,
        };

        self.containers.write().await.insert(id.clone(), container);
        info!("Container registered: {}", id);
        Ok(id)
    }

    /// Start a container
    pub async fn start(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get_mut(container_id)
            .ok_or_else(|| anyhow::anyhow!("Container not found: {}", container_id))?;

        if let Some(docker_id) = &container.docker_id {
            let docker = self.docker.read().await;
            if let Some(docker) = docker.as_ref() {
                docker.start_container(docker_id, None::<StartContainerOptions<String>>).await
                    .map_err(|e| anyhow::anyhow!("Failed to start container: {}", e))?;
            }
        }

        container.status = ContainerStatus::Running;
        container.health_check_failures = 0;
        info!("Container started: {}", container_id);
        Ok(())
    }

    /// Stop a container
    pub async fn stop(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get_mut(container_id)
            .ok_or_else(|| anyhow::anyhow!("Container not found: {}", container_id))?;

        if let Some(docker_id) = &container.docker_id {
            let docker = self.docker.read().await;
            if let Some(docker) = docker.as_ref() {
                docker.stop_container(docker_id, Some(StopContainerOptions { t: 10 })).await
                    .map_err(|e| anyhow::anyhow!("Failed to stop container: {}", e))?;
            }
        }

        container.status = ContainerStatus::Stopped;
        info!("Container stopped: {}", container_id);
        Ok(())
    }

    /// Remove a container
    pub async fn remove(&self, container_id: &str) -> Result<()> {
        let docker_id = {
            let containers = self.containers.read().await;
            let container = containers
                .get(container_id)
                .ok_or_else(|| anyhow::anyhow!("Container not found: {}", container_id))?;
            container.docker_id.clone()
        };

        if let Some(docker_id) = docker_id {
            let docker = self.docker.read().await;
            if let Some(docker) = docker.as_ref() {
                docker.remove_container(&docker_id, Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                })).await
                    .map_err(|e| anyhow::anyhow!("Failed to remove container: {}", e))?;
            }
        }

        self.containers.write().await.remove(container_id)
            .ok_or_else(|| anyhow::anyhow!("Container not found: {}", container_id))?;

        info!("Container removed: {}", container_id);
        Ok(())
    }

    /// Execute a command inside a running container
    pub async fn exec(&self, container_id: &str, command: Vec<String>) -> Result<ExecResult> {
        let containers = self.containers.read().await;
        let container = containers
            .get(container_id)
            .ok_or_else(|| anyhow::anyhow!("Container not found: {}", container_id))?;

        if container.status != ContainerStatus::Running {
            return Err(anyhow::anyhow!("Container is not running"));
        }

        if let Some(docker_id) = &container.docker_id {
            let docker = self.docker.read().await;
            if let Some(docker) = docker.as_ref() {
                let exec = docker.create_exec(docker_id, CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(command.iter().map(|s| s.as_str()).collect()),
                    ..Default::default()
                }).await.map_err(|e| anyhow::anyhow!("Failed to create exec: {}", e))?;

                let mut stdout = String::new();
                let mut stderr = String::new();

                if let StartExecResults::Attached { mut output, .. } = docker.start_exec(&exec.id, None).await
                    .map_err(|e| anyhow::anyhow!("Failed to start exec: {}", e))?
                {
                    while let Some(Ok(msg)) = output.next().await {
                        match msg {
                            bollard::container::LogOutput::StdOut { message } => {
                                stdout.push_str(&String::from_utf8_lossy(&message));
                            }
                            bollard::container::LogOutput::StdErr { message } => {
                                stderr.push_str(&String::from_utf8_lossy(&message));
                            }
                            _ => {}
                        }
                    }
                }

                let inspect = docker.inspect_exec(&exec.id).await
                    .map_err(|e| anyhow::anyhow!("Failed to inspect exec: {}", e))?;

                return Ok(ExecResult {
                    exit_code: inspect.exit_code.unwrap_or(-1),
                    stdout,
                    stderr,
                });
            }
        }

        // Simulation mode
        Ok(ExecResult {
            exit_code: 0,
            stdout: format!("[simulation] Executed: {}", command.join(" ")),
            stderr: String::new(),
        })
    }

    /// Get container logs
    pub async fn get_logs(&self, container_id: &str, tail: usize) -> Result<Vec<String>> {
        let containers = self.containers.read().await;
        let container = containers
            .get(container_id)
            .ok_or_else(|| anyhow::anyhow!("Container not found: {}", container_id))?;

        if let Some(docker_id) = &container.docker_id {
            let docker = self.docker.read().await;
            if let Some(docker) = docker.as_ref() {
                let options = LogsOptions::<String> {
                    stdout: true,
                    stderr: true,
                    tail: tail.to_string(),
                    ..Default::default()
                };

                let mut logs = Vec::new();
                let mut stream = docker.logs(docker_id, Some(options));
                while let Some(Ok(msg)) = stream.next().await {
                    logs.push(msg.to_string());
                }
                return Ok(logs);
            }
        }

        Ok(container.logs.iter().rev().take(tail).rev().cloned().collect())
    }

    /// Check container health
    pub async fn health_check(&self, container_id: &str) -> Result<bool> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get_mut(container_id)
            .ok_or_else(|| anyhow::anyhow!("Container not found: {}", container_id))?;

        if let Some(docker_id) = &container.docker_id {
            let docker = self.docker.read().await;
            if let Some(docker) = docker.as_ref() {
                match docker.inspect_container(docker_id, None::<InspectContainerOptions>).await {
                    Ok(info) => {
                        if let Some(state) = info.state {
                            let running = state.running.unwrap_or(false);
                            if !running && container.status == ContainerStatus::Running {
                                container.health_check_failures += 1;
                                let exit_code = state.exit_code.unwrap_or(-1);
                                container.status = ContainerStatus::Exited(exit_code);
                                return Ok(false);
                            }
                            container.health_check_failures = 0;
                            return Ok(running);
                        }
                    }
                    Err(e) => {
                        warn!("Health check failed for {}: {}", container_id, e);
                        container.health_check_failures += 1;
                        return Ok(false);
                    }
                }
            }
        }

        Ok(container.status == ContainerStatus::Running)
    }

    /// Restart a container
    pub async fn restart(&self, container_id: &str) -> Result<()> {
        self.stop(container_id).await?;
        self.start(container_id).await?;
        info!("Container restarted: {}", container_id);
        Ok(())
    }

    /// List all managed containers
    pub async fn list(&self) -> Vec<ManagedContainer> {
        self.containers.read().await.values().cloned().collect()
    }

    /// Get a specific container
    pub async fn get(&self, container_id: &str) -> Option<ManagedContainer> {
        self.containers.read().await.get(container_id).cloned()
    }

    /// Check if Docker is connected
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }
}
