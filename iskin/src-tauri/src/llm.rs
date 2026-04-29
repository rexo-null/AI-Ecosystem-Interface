#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use futures_util::StreamExt;
use log::{info, warn, error};
use tauri::Emitter;

// ============================================================
// Configuration
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub endpoint: String,
    pub context_length: usize,
    pub temperature: f32,
    pub max_tokens: usize,
    pub gpu_layers: i32,
    pub threads: usize,
    pub model: ModelConfig,
    pub model_profile: ModelProfile,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub chat_template: String,
    pub tool_call_format: String,
    pub stop_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub batch_size: usize,
    pub ubatch_size: usize,
    pub flash_attn: bool,
    pub cache_type_k: String,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:8080".to_string(),
            context_length: 8192,
            temperature: 0.2,
            max_tokens: 2048,
            gpu_layers: 45,
            threads: 8,
            model: ModelConfig {
                path: "models/qwen2.5-coder-14b-instruct-Q4_K_M.gguf".to_string(),
                name: "Qwen2.5-Coder-14B".to_string(),
            },
            model_profile: ModelProfile {
                chat_template: "chatml".to_string(),
                tool_call_format: "json_block".to_string(),
                stop_tokens: vec![
                    "<|im_end|>".to_string(),
                    "<|endoftext|>".to_string(),
                ],
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                batch_size: 512,
                ubatch_size: 64,
                flash_attn: true,
                cache_type_k: "q4_0".to_string(),
            },
        }
    }
}

impl LLMConfig {
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let doc = content.parse::<toml_edit::DocumentMut>()
            .map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))?;

        let mut config = Self::default();

        if let Some(llm) = doc.get("llm") {
            if let Some(v) = llm.get("endpoint").and_then(|v| v.as_str()) {
                config.endpoint = v.to_string();
            }
            if let Some(v) = llm.get("context_length").and_then(|v| v.as_integer()) {
                config.context_length = v as usize;
            }
            if let Some(v) = llm.get("temperature").and_then(|v| v.as_float()) {
                config.temperature = v as f32;
            }
            if let Some(v) = llm.get("max_tokens").and_then(|v| v.as_integer()) {
                config.max_tokens = v as usize;
            }
            if let Some(v) = llm.get("gpu_layers").and_then(|v| v.as_integer()) {
                config.gpu_layers = v as i32;
            }
            if let Some(v) = llm.get("threads").and_then(|v| v.as_integer()) {
                config.threads = v as usize;
            }
        }

        if let Some(model) = doc.get("model") {
            if let Some(v) = model.get("path").and_then(|v| v.as_str()) {
                config.model.path = v.to_string();
            }
            if let Some(v) = model.get("name").and_then(|v| v.as_str()) {
                config.model.name = v.to_string();
            }
        }

        if let Some(profile) = doc.get("model_profile") {
            if let Some(v) = profile.get("chat_template").and_then(|v| v.as_str()) {
                config.model_profile.chat_template = v.to_string();
            }
            if let Some(v) = profile.get("tool_call_format").and_then(|v| v.as_str()) {
                config.model_profile.tool_call_format = v.to_string();
            }
            if let Some(arr) = profile.get("stop_tokens").and_then(|v| v.as_array()) {
                config.model_profile.stop_tokens = arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }

        if let Some(server) = doc.get("server") {
            if let Some(v) = server.get("host").and_then(|v| v.as_str()) {
                config.server.host = v.to_string();
            }
            if let Some(v) = server.get("port").and_then(|v| v.as_integer()) {
                config.server.port = v as u16;
            }
            if let Some(v) = server.get("batch_size").and_then(|v| v.as_integer()) {
                config.server.batch_size = v as usize;
            }
            if let Some(v) = server.get("ubatch_size").and_then(|v| v.as_integer()) {
                config.server.ubatch_size = v as usize;
            }
            if let Some(v) = server.get("flash_attn").and_then(|v| v.as_bool()) {
                config.server.flash_attn = v;
            }
            if let Some(v) = server.get("cache_type_k").and_then(|v| v.as_str()) {
                config.server.cache_type_k = v.to_string();
            }
        }

        Ok(config)
    }
}

// ============================================================
// Data Types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMStatus {
    pub online: bool,
    pub model: String,
    pub endpoint: String,
}

// llama.cpp /v1/chat/completions request body
#[derive(Debug, Serialize)]
struct LlamaCppRequest {
    messages: Vec<LlamaCppMessage>,
    stream: bool,
    temperature: f32,
    max_tokens: usize,
    stop: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LlamaCppMessage {
    role: String,
    content: String,
}

// llama.cpp /v1/chat/completions response (non-streaming)
#[derive(Debug, Deserialize)]
struct LlamaCppResponse {
    choices: Vec<LlamaCppChoice>,
    #[serde(default)]
    usage: Option<LlamaCppUsage>,
}

#[derive(Debug, Deserialize)]
struct LlamaCppChoice {
    message: LlamaCppMessageResponse,
}

#[derive(Debug, Deserialize)]
struct LlamaCppMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct LlamaCppUsage {
    #[serde(default)]
    total_tokens: usize,
}

// SSE streaming chunk
#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

// Tauri event payloads
#[derive(Debug, Clone, Serialize)]
pub struct LLMTokenEvent {
    pub token: String,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LLMDoneEvent {
    pub message_id: String,
    pub full_content: String,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LLMErrorEvent {
    pub message_id: String,
    pub error: String,
}

// ============================================================
// LLM Engine
// ============================================================

pub struct LLMEngine {
    config: LLMConfig,
    client: reqwest::Client,
    conversation_history: Arc<Mutex<Vec<Message>>>,
    is_generating: Arc<AtomicBool>,
}

impl LLMEngine {
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        info!("LLM Engine initialized: model={}, endpoint={}", config.model.name, config.endpoint);

        Self {
            config,
            client,
            conversation_history: Arc::new(Mutex::new(Vec::new())),
            is_generating: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        info!("LLM Engine ready (model: {}, endpoint: {})", self.config.model.name, self.config.endpoint);
        Ok(())
    }

    pub fn config(&self) -> &LLMConfig {
        &self.config
    }

    /// Check if llama-server is available
    pub async fn status(&self) -> LLMStatus {
        let online = self.client
            .get(format!("{}/v1/models", self.config.endpoint))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        LLMStatus {
            online,
            model: self.config.model.name.clone(),
            endpoint: self.config.endpoint.clone(),
        }
    }

    /// Send a chat request and get the full response (non-streaming)
    pub async fn chat(&self, request: ChatRequest) -> anyhow::Result<LLMResponse> {
        // Snapshot history WITHOUT pushing user message yet (defer until success)
        let user_message = request.messages.last().cloned();
        let history_snapshot = {
            let history = self.conversation_history.lock().await;
            history.clone()
        };

        // Build messages: system prompt + history + current user message
        let mut llama_messages = Vec::new();

        if let Some(ref system) = request.system_prompt {
            llama_messages.push(LlamaCppMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        for msg in history_snapshot.iter() {
            llama_messages.push(LlamaCppMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        if let Some(ref msg) = user_message {
            llama_messages.push(LlamaCppMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        let llama_request = LlamaCppRequest {
            messages: llama_messages,
            stream: false,
            temperature: request.temperature.unwrap_or(self.config.temperature),
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens),
            stop: self.config.model_profile.stop_tokens.clone(),
        };

        let url = format!("{}/v1/chat/completions", self.config.endpoint);

        let response = self.client
            .post(&url)
            .json(&llama_request)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let llama_resp: LlamaCppResponse = resp.json().await?;
                let content = llama_resp.choices
                    .first()
                    .map(|c| c.message.content.clone())
                    .unwrap_or_default();

                let tokens_used = llama_resp.usage
                    .map(|u| u.total_tokens)
                    .unwrap_or(0);

                // Push both user + assistant to history atomically on success
                let mut history = self.conversation_history.lock().await;
                if let Some(ref msg) = user_message {
                    history.push(msg.clone());
                }
                history.push(Message {
                    role: "assistant".to_string(),
                    content: content.clone(),
                });

                Ok(LLMResponse {
                    content,
                    model: self.config.model.name.clone(),
                    tokens_used,
                })
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                warn!("llama-server returned {}: {}", status, body);
                Err(anyhow::anyhow!(
                    "llama-server error ({}): {}",
                    status,
                    body
                ))
            }
            Err(e) if e.is_connect() => {
                warn!("llama-server not available at {}", url);
                let content = format!(
                    "LLM сервер не запущен ({}). Запустите llama-server командой:\n\n\
                    ```bash\n./scripts/run.sh\n```\n\n\
                    Или вручную:\n\
                    ```bash\nllama-server \\\n  \
                    --model {} \\\n  \
                    --ctx-size {} \\\n  \
                    --n-gpu-layers {} \\\n  \
                    --host {} --port {}\n```",
                    self.config.endpoint,
                    self.config.model.path,
                    self.config.context_length,
                    self.config.gpu_layers,
                    self.config.server.host,
                    self.config.server.port,
                );

                let mut history = self.conversation_history.lock().await;
                if let Some(ref msg) = user_message {
                    history.push(msg.clone());
                }
                history.push(Message {
                    role: "assistant".to_string(),
                    content: content.clone(),
                });

                Ok(LLMResponse {
                    content,
                    model: self.config.model.name.clone(),
                    tokens_used: 0,
                })
            }
            Err(e) => Err(anyhow::anyhow!("HTTP request failed: {}", e)),
        }
    }

    /// Send a chat request with SSE streaming, emitting Tauri events for each token
    pub async fn chat_stream(
        &self,
        request: ChatRequest,
        message_id: String,
        app_handle: tauri::AppHandle,
    ) -> anyhow::Result<()> {
        self.is_generating.store(true, Ordering::SeqCst);

        // Snapshot history WITHOUT pushing user message (defer until success)
        let user_message = request.messages.last().cloned();
        let history_snapshot = {
            let history = self.conversation_history.lock().await;
            history.clone()
        };

        // Build messages: system prompt + history + current user message
        let mut llama_messages = Vec::new();

        if let Some(ref system) = request.system_prompt {
            llama_messages.push(LlamaCppMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        for msg in history_snapshot.iter() {
            llama_messages.push(LlamaCppMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        if let Some(ref msg) = user_message {
            llama_messages.push(LlamaCppMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        let llama_request = LlamaCppRequest {
            messages: llama_messages,
            stream: true,
            temperature: request.temperature.unwrap_or(self.config.temperature),
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens),
            stop: self.config.model_profile.stop_tokens.clone(),
        };

        let url = format!("{}/v1/chat/completions", self.config.endpoint);

        let response = self.client
            .post(&url)
            .json(&llama_request)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let mut full_content = String::new();
                let mut stream = resp.bytes_stream();

                let mut buffer = String::new();
                let mut stream_done = false;

                while let Some(chunk_result) = stream.next().await {
                    if !self.is_generating.load(Ordering::SeqCst) {
                        info!("Generation stopped by user for message {}", message_id);
                        break;
                    }

                    match chunk_result {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes);
                            buffer.push_str(&text);

                            // Process complete SSE lines
                            while let Some(line_end) = buffer.find('\n') {
                                let line = buffer[..line_end].trim().to_string();
                                buffer = buffer[line_end + 1..].to_string();

                                if line.is_empty() || line.starts_with(':') {
                                    continue;
                                }

                                if let Some(data) = line.strip_prefix("data: ") {
                                    if data.trim() == "[DONE]" {
                                        stream_done = true;
                                        break;
                                    }

                                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                                        if let Some(choice) = chunk.choices.first() {
                                            if let Some(ref token) = choice.delta.content {
                                                if !token.is_empty() {
                                                    full_content.push_str(token);
                                                    let _ = app_handle.emit("llm-token", LLMTokenEvent {
                                                        token: token.clone(),
                                                        message_id: message_id.clone(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if stream_done { break; }
                        }
                        Err(e) => {
                            error!("Stream error: {}", e);
                            let _ = app_handle.emit("llm-error", LLMErrorEvent {
                                message_id: message_id.clone(),
                                error: format!("Stream error: {}", e),
                            });
                            self.is_generating.store(false, Ordering::SeqCst);
                            return Err(anyhow::anyhow!("Stream error: {}", e));
                        }
                    }
                }

                // Push both user + assistant to history atomically on success
                let mut history = self.conversation_history.lock().await;
                if let Some(ref msg) = user_message {
                    history.push(msg.clone());
                }
                history.push(Message {
                    role: "assistant".to_string(),
                    content: full_content.clone(),
                });

                let _ = app_handle.emit("llm-done", LLMDoneEvent {
                    message_id,
                    full_content,
                    tokens_used: 0,
                });

                self.is_generating.store(false, Ordering::SeqCst);
                Ok(())
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                self.is_generating.store(false, Ordering::SeqCst);

                let _ = app_handle.emit("llm-error", LLMErrorEvent {
                    message_id,
                    error: format!("llama-server error ({}): {}", status, body),
                });

                Err(anyhow::anyhow!("llama-server error ({}): {}", status, body))
            }
            Err(e) if e.is_connect() => {
                self.is_generating.store(false, Ordering::SeqCst);

                let fallback = format!(
                    "LLM сервер не запущен ({}). Запустите: ./scripts/run.sh",
                    self.config.endpoint
                );

                let _ = app_handle.emit("llm-error", LLMErrorEvent {
                    message_id,
                    error: fallback.clone(),
                });

                Err(anyhow::anyhow!("{}", fallback))
            }
            Err(e) => {
                self.is_generating.store(false, Ordering::SeqCst);
                let _ = app_handle.emit("llm-error", LLMErrorEvent {
                    message_id,
                    error: format!("HTTP error: {}", e),
                });
                Err(anyhow::anyhow!("HTTP request failed: {}", e))
            }
        }
    }

    /// Stop the current generation
    pub fn stop_generation(&self) {
        self.is_generating.store(false, Ordering::SeqCst);
        info!("Generation stop requested");
    }

    /// Check if currently generating
    pub fn is_generating(&self) -> bool {
        self.is_generating.load(Ordering::SeqCst)
    }

    pub async fn clear_history(&self) {
        let mut history = self.conversation_history.lock().await;
        history.clear();
    }

    pub async fn get_history(&self) -> Vec<Message> {
        self.conversation_history.lock().await.clone()
    }
}
