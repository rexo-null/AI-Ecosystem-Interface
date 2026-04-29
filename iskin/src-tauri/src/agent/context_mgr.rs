use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use anyhow::Result;

use super::planner::TaskId;

/// Context configuration
#[derive(Debug, Clone)]
pub struct ContextConfig {
    pub base_prompt: String,
    pub max_tokens_per_step: usize,
    pub include_file_content: bool,
    pub max_history_messages: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            base_prompt: "You are ISKIN, an autonomous code agent.".to_string(),
            max_tokens_per_step: 4096,
            include_file_content: true,
            max_history_messages: 10,
        }
    }
}

/// Reference to an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactReference {
    pub path: String,
    pub hash: String,
    pub relevance_score: f32,
}

/// Artifact data
#[derive(Debug, Clone)]
pub struct ArtifactData {
    pub content: String,
    pub hash: String,
    pub last_modified: DateTime<Utc>,
}

/// Artifact storage
pub struct ArtifactStore {
    artifacts: HashMap<String, ArtifactData>,
}

impl ArtifactStore {
    /// Create a new artifact store
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
        }
    }

    /// Store artifact by path
    pub fn store(&mut self, path: String, content: String) -> Result<()> {
        let hash = format!("{:x}", md5::compute(content.as_bytes()));
        self.artifacts.insert(
            path,
            ArtifactData {
                content,
                hash,
                last_modified: Utc::now(),
            },
        );
        Ok(())
    }

    /// Get artifact by path
    pub fn get(&self, path: &str) -> Option<&ArtifactData> {
        self.artifacts.get(path)
    }

    /// List all artifacts
    pub fn list_all(&self) -> Vec<String> {
        self.artifacts.keys().cloned().collect()
    }
}

impl Default for ArtifactStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Context manager for task execution
pub struct ContextManager {
    config: ContextConfig,
    current_context: String,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(config: ContextConfig) -> Self {
        Self {
            current_context: config.base_prompt.clone(),
            config,
        }
    }

    /// Build context for a task step
    pub fn build_for_step(
        &mut self,
        _step_description: &str,
        _artifacts: &ArtifactStore,
    ) -> Result<String> {
        // Temporary implementation: return base prompt + step
        Ok(self.current_context.clone())
    }

    /// Clear context (reset to base prompt)
    pub fn clear(&mut self) {
        self.current_context = self.config.base_prompt.clone();
    }

    /// Count tokens (approximate)
    pub fn count_tokens(text: &str) -> usize {
        // Rough estimate: 1 token ≈ 4 characters
        (text.len() + 3) / 4
    }

    /// Get current context
    pub fn current(&self) -> &str {
        &self.current_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_store() {
        let mut store = ArtifactStore::new();
        store.store("test.txt".to_string(), "content".to_string()).unwrap();
        
        let artifact = store.get("test.txt");
        assert!(artifact.is_some());
    }

    #[test]
    fn test_context_manager() {
        let config = ContextConfig::default();
        let mut mgr = ContextManager::new(config);
        
        assert!(!mgr.current().is_empty());
        mgr.clear();
        assert!(!mgr.current().is_empty());
    }

    #[test]
    fn test_count_tokens() {
        let tokens = ContextManager::count_tokens("hello world");
        assert!(tokens > 0);
    }
}
