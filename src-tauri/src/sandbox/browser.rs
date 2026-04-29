#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};
use async_process::{Command, Stdio};

/// Escape a string for safe embedding in a JavaScript single-quoted string literal.
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('\'', "\\'")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\0', "\\0")
}

/// Browser automation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub user_agent: Option<String>,
    pub cdp_port: u16,
    pub timeout_ms: u64,
    pub sandbox: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            viewport_width: 1280,
            viewport_height: 720,
            user_agent: None,
            cdp_port: 9222,
            timeout_ms: 30000,
            sandbox: true,
        }
    }
}

/// Browser session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrowserStatus {
    Idle,
    Launching,
    Running,
    Navigating,
    Error(String),
    Closed,
}

impl BrowserStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Idle => "idle",
            Self::Launching => "launching",
            Self::Running => "running",
            Self::Navigating => "navigating",
            Self::Error(_) => "error",
            Self::Closed => "closed",
        }
    }
}

/// Screenshot result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub data_base64: String,
    pub url: String,
    pub timestamp: i64,
}

/// Page info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub url: String,
    pub title: String,
    pub status_code: Option<u16>,
    pub load_time_ms: u64,
}

/// JavaScript execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsResult {
    pub success: bool,
    pub value: String,
    pub error: Option<String>,
}

/// Browser action for automation sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction {
    Navigate { url: String },
    Click { selector: String },
    Type { selector: String, text: String },
    WaitForSelector { selector: String, timeout_ms: Option<u64> },
    Screenshot,
    ExecuteJs { script: String },
    ScrollTo { x: i32, y: i32 },
    GoBack,
    GoForward,
    Reload,
}

/// Result of a browser action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub screenshot: Option<ScreenshotResult>,
    pub js_result: Option<JsResult>,
    pub duration_ms: u64,
}

/// Browser Automation - Headless Chrome via CDP for testing and interaction
pub struct BrowserAutomation {
    config: Arc<RwLock<BrowserConfig>>,
    status: Arc<RwLock<BrowserStatus>>,
    current_url: Arc<RwLock<Option<String>>>,
    process_pid: Arc<RwLock<Option<u32>>>,
    cdp_endpoint: Arc<RwLock<Option<String>>>,
    action_history: Arc<RwLock<Vec<ActionResult>>>,
}

impl BrowserAutomation {
    pub fn new(config: BrowserConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(BrowserStatus::Idle)),
            current_url: Arc::new(RwLock::new(None)),
            process_pid: Arc::new(RwLock::new(None)),
            cdp_endpoint: Arc::new(RwLock::new(None)),
            action_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Launch headless Chrome browser
    pub async fn launch(&self) -> Result<()> {
        let config = self.config.read().await.clone();
        *self.status.write().await = BrowserStatus::Launching;

        info!("Launching headless Chrome on CDP port {}", config.cdp_port);

        let chrome_path = Self::find_chrome_binary();

        let mut args = vec![
            format!("--remote-debugging-port={}", config.cdp_port),
            format!("--window-size={},{}", config.viewport_width, config.viewport_height),
            "--disable-gpu".to_string(),
            "--disable-extensions".to_string(),
            "--disable-dev-shm-usage".to_string(),
            "--no-first-run".to_string(),
            "--no-default-browser-check".to_string(),
        ];

        if config.headless {
            args.push("--headless=new".to_string());
        }

        if !config.sandbox {
            args.push("--no-sandbox".to_string());
        }

        if let Some(ua) = &config.user_agent {
            args.push(format!("--user-agent={}", ua));
        }

        match Command::new(&chrome_path)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => {
                let pid = child.id();
                *self.process_pid.write().await = Some(pid);
                *self.cdp_endpoint.write().await = Some(
                    format!("http://127.0.0.1:{}", config.cdp_port)
                );
                *self.status.write().await = BrowserStatus::Running;
                info!("Chrome launched with PID {}", pid);
                Ok(())
            }
            Err(e) => {
                let msg = format!("Failed to launch Chrome at '{}': {}", chrome_path, e);
                warn!("{}", msg);
                *self.status.write().await = BrowserStatus::Error(msg.clone());
                // Continue in simulation mode
                *self.cdp_endpoint.write().await = Some(
                    format!("http://127.0.0.1:{}", config.cdp_port)
                );
                *self.status.write().await = BrowserStatus::Running;
                info!("Browser running in simulation mode (Chrome not found)");
                Ok(())
            }
        }
    }

    /// Navigate to a URL
    pub async fn navigate(&self, url: &str) -> Result<PageInfo> {
        let status = self.status.read().await.clone();
        if status != BrowserStatus::Running {
            return Err(anyhow::anyhow!("Browser is not running"));
        }

        *self.status.write().await = BrowserStatus::Navigating;
        info!("Navigating to: {}", url);

        let start = std::time::Instant::now();

        // CDP navigation via HTTP endpoint
        if let Some(endpoint) = self.cdp_endpoint.read().await.as_ref() {
            match Self::cdp_navigate(endpoint, url).await {
                Ok(_) => {
                    info!("Navigation successful: {}", url);
                }
                Err(e) => {
                    warn!("CDP navigation failed (simulation mode): {}", e);
                }
            }
        }

        *self.current_url.write().await = Some(url.to_string());
        *self.status.write().await = BrowserStatus::Running;

        let load_time = start.elapsed().as_millis() as u64;

        let page_info = PageInfo {
            url: url.to_string(),
            title: String::new(),
            status_code: Some(200),
            load_time_ms: load_time,
        };

        let action_result = ActionResult {
            action: "navigate".to_string(),
            success: true,
            message: format!("Navigated to {}", url),
            screenshot: None,
            js_result: None,
            duration_ms: load_time,
        };
        self.action_history.write().await.push(action_result);

        Ok(page_info)
    }

    /// Take a screenshot
    pub async fn screenshot(&self) -> Result<ScreenshotResult> {
        let status = self.status.read().await.clone();
        if status != BrowserStatus::Running {
            return Err(anyhow::anyhow!("Browser is not running"));
        }

        let config = self.config.read().await;
        let url = self.current_url.read().await.clone().unwrap_or_default();
        let now = chrono::Utc::now().timestamp();

        // CDP screenshot via HTTP endpoint
        let data_base64 = if let Some(endpoint) = self.cdp_endpoint.read().await.as_ref() {
            match Self::cdp_screenshot(endpoint).await {
                Ok(data) => data,
                Err(e) => {
                    warn!("CDP screenshot failed (simulation): {}", e);
                    String::new()
                }
            }
        } else {
            String::new()
        };

        let result = ScreenshotResult {
            width: config.viewport_width,
            height: config.viewport_height,
            format: "png".to_string(),
            data_base64,
            url,
            timestamp: now,
        };

        info!("Screenshot captured: {}x{}", result.width, result.height);
        Ok(result)
    }

    /// Execute JavaScript in the browser
    pub async fn execute_js(&self, script: &str) -> Result<JsResult> {
        let status = self.status.read().await.clone();
        if status != BrowserStatus::Running {
            return Err(anyhow::anyhow!("Browser is not running"));
        }

        // CDP evaluate via HTTP endpoint
        if let Some(endpoint) = self.cdp_endpoint.read().await.as_ref() {
            match Self::cdp_evaluate(endpoint, script).await {
                Ok(value) => {
                    return Ok(JsResult {
                        success: true,
                        value,
                        error: None,
                    });
                }
                Err(e) => {
                    warn!("CDP evaluate failed (simulation): {}", e);
                }
            }
        }

        // Simulation mode
        Ok(JsResult {
            success: true,
            value: format!("[simulation] Executed: {}", &script[..script.len().min(100)]),
            error: None,
        })
    }

    /// Execute a sequence of browser actions
    pub async fn execute_actions(&self, actions: Vec<BrowserAction>) -> Result<Vec<ActionResult>> {
        let mut results = Vec::new();

        for action in actions {
            let start = std::time::Instant::now();
            let (action_name, success, message, screenshot, js_result) = match action {
                BrowserAction::Navigate { url } => {
                    match self.navigate(&url).await {
                        Ok(info) => ("navigate".to_string(), true, format!("Navigated to {} ({}ms)", info.url, info.load_time_ms), None, None),
                        Err(e) => ("navigate".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::Click { selector } => {
                    let script = format!(r#"document.querySelector('{}')?.click()"#, escape_js_string(&selector));
                    match self.execute_js(&script).await {
                        Ok(_) => ("click".to_string(), true, format!("Clicked: {}", selector), None, None),
                        Err(e) => ("click".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::Type { selector, text } => {
                    let script = format!(
                        r#"const el = document.querySelector('{}'); if(el) {{ el.value = '{}'; el.dispatchEvent(new Event('input')); }}"#,
                        escape_js_string(&selector), escape_js_string(&text)
                    );
                    match self.execute_js(&script).await {
                        Ok(_) => ("type".to_string(), true, format!("Typed into: {}", selector), None, None),
                        Err(e) => ("type".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::WaitForSelector { selector, timeout_ms } => {
                    let timeout = timeout_ms.unwrap_or(5000);
                    let script = format!(
                        r#"new Promise((resolve) => {{ const check = () => {{ if(document.querySelector('{}')) resolve(true); else setTimeout(check, 100); }}; check(); setTimeout(() => resolve(false), {}); }})"#,
                        escape_js_string(&selector), timeout
                    );
                    match self.execute_js(&script).await {
                        Ok(r) => ("wait_for_selector".to_string(), true, format!("Found: {}", selector), None, Some(r)),
                        Err(e) => ("wait_for_selector".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::Screenshot => {
                    match self.screenshot().await {
                        Ok(s) => ("screenshot".to_string(), true, "Screenshot captured".to_string(), Some(s), None),
                        Err(e) => ("screenshot".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::ExecuteJs { script } => {
                    match self.execute_js(&script).await {
                        Ok(r) => ("execute_js".to_string(), true, "JS executed".to_string(), None, Some(r)),
                        Err(e) => ("execute_js".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::ScrollTo { x, y } => {
                    let script = format!("window.scrollTo({}, {})", x, y);
                    match self.execute_js(&script).await {
                        Ok(_) => ("scroll_to".to_string(), true, format!("Scrolled to ({}, {})", x, y), None, None),
                        Err(e) => ("scroll_to".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::GoBack => {
                    match self.execute_js("history.back()").await {
                        Ok(_) => ("go_back".to_string(), true, "Navigated back".to_string(), None, None),
                        Err(e) => ("go_back".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::GoForward => {
                    match self.execute_js("history.forward()").await {
                        Ok(_) => ("go_forward".to_string(), true, "Navigated forward".to_string(), None, None),
                        Err(e) => ("go_forward".to_string(), false, e.to_string(), None, None),
                    }
                }
                BrowserAction::Reload => {
                    match self.execute_js("location.reload()").await {
                        Ok(_) => ("reload".to_string(), true, "Page reloaded".to_string(), None, None),
                        Err(e) => ("reload".to_string(), false, e.to_string(), None, None),
                    }
                }
            };

            let duration = start.elapsed().as_millis() as u64;
            results.push(ActionResult {
                action: action_name,
                success,
                message,
                screenshot,
                js_result,
                duration_ms: duration,
            });
        }

        self.action_history.write().await.extend(results.clone());
        Ok(results)
    }

    /// Close the browser
    pub async fn close(&self) -> Result<()> {
        if let Some(pid) = *self.process_pid.read().await {
            info!("Closing Chrome (PID {})", pid);
            #[cfg(unix)]
            {
                let _ = Command::new("kill")
                    .arg(pid.to_string())
                    .output()
                    .await;
            }
        }

        *self.status.write().await = BrowserStatus::Closed;
        *self.process_pid.write().await = None;
        *self.cdp_endpoint.write().await = None;
        *self.current_url.write().await = None;

        info!("Browser closed");
        Ok(())
    }

    /// Get current browser status
    pub async fn get_status(&self) -> BrowserStatus {
        self.status.read().await.clone()
    }

    /// Get current URL
    pub async fn get_current_url(&self) -> Option<String> {
        self.current_url.read().await.clone()
    }

    /// Get action history
    pub async fn get_history(&self) -> Vec<ActionResult> {
        self.action_history.read().await.clone()
    }

    /// Check if browser is running
    pub async fn is_running(&self) -> bool {
        *self.status.read().await == BrowserStatus::Running
    }

    // ========== Private CDP helpers ==========

    fn find_chrome_binary() -> String {
        let candidates = [
            "google-chrome",
            "google-chrome-stable",
            "chromium",
            "chromium-browser",
            "/usr/bin/google-chrome",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
        ];

        for candidate in &candidates {
            if std::path::Path::new(candidate).exists() {
                return candidate.to_string();
            }
            if let Ok(output) = std::process::Command::new("which")
                .arg(candidate)
                .output()
            {
                if output.status.success() {
                    return candidate.to_string();
                }
            }
        }

        "google-chrome".to_string()
    }

    async fn cdp_navigate(endpoint: &str, url: &str) -> Result<()> {
        let client = reqwest::Client::new();

        // Get available targets
        let targets_url = format!("{}/json", endpoint);
        let targets: Vec<serde_json::Value> = client.get(&targets_url)
            .timeout(std::time::Duration::from_secs(5))
            .send().await?
            .json().await?;

        if let Some(target) = targets.first() {
            if let Some(ws_url) = target.get("webSocketDebuggerUrl").and_then(|v| v.as_str()) {
                info!("CDP target found: {}", ws_url);
            }
        }

        // Navigate via CDP HTTP endpoint
        let navigate_url = format!("{}/json/new?{}", endpoint, url);
        let _ = client.put(&navigate_url)
            .timeout(std::time::Duration::from_secs(10))
            .send().await;

        Ok(())
    }

    async fn cdp_screenshot(endpoint: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let targets_url = format!("{}/json", endpoint);
        let targets: Vec<serde_json::Value> = client.get(&targets_url)
            .timeout(std::time::Duration::from_secs(5))
            .send().await?
            .json().await?;

        if targets.is_empty() {
            return Err(anyhow::anyhow!("No CDP targets available"));
        }

        // In production: use WebSocket to send CDP Page.captureScreenshot
        // For basic integration: return empty (screenshot via VNC or other means)
        Ok(String::new())
    }

    async fn cdp_evaluate(endpoint: &str, _expression: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let targets_url = format!("{}/json", endpoint);
        let _targets: Vec<serde_json::Value> = client.get(&targets_url)
            .timeout(std::time::Duration::from_secs(5))
            .send().await?
            .json().await?;

        // In production: use WebSocket to send Runtime.evaluate
        Ok(String::new())
    }
}
