use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use log::{info, warn};

/// Memory entry types in the hierarchical knowledge base
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Constitution,      // Core unchangeable rules
    Protocol,          // Operational procedures
    Pattern,           // Code patterns and best practices
    ProjectContext,    // Project-specific knowledge
    UserRule,          // Custom user-defined rules
    ToolDefinition,    // MCP-like tool definitions
}

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub priority: u8, // 1-10, higher = more important
    pub is_active: bool,
}

/// Hierarchical Knowledge Base
pub struct KnowledgeBase {
    entries: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    storage_path: PathBuf,
}

impl KnowledgeBase {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        }
    }

    /// Initialize the knowledge base from disk
    pub async fn initialize(&self) -> Result<()> {
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path).await?;
            // Create default structure
            self.create_default_entries().await?;
        }
        
        // Load existing entries (implementation in next iteration)
        info!("Knowledge base initialized at {:?}", self.storage_path);
        Ok(())
    }

    /// Create default constitution and protocols
    async fn create_default_entries(&self) -> Result<()> {
        let defaults = vec![
            MemoryEntry {
                id: "constitution_1".to_string(),
                title: "Core Safety Principle".to_string(),
                content: "Never execute commands that could harm the host system or delete critical files without explicit user confirmation.".to_string(),
                memory_type: MemoryType::Constitution,
                tags: vec!["safety".to_string(), "core".to_string()],
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                priority: 10,
                is_active: true,
            },
            MemoryEntry {
                id: "protocol_1".to_string(),
                title: "Code Review Protocol".to_string(),
                content: "1. Analyze the code change\n2. Check for potential bugs\n3. Verify test coverage\n4. Ensure coding standards\n5. Request user confirmation before merging".to_string(),
                memory_type: MemoryType::Protocol,
                tags: vec!["review".to_string(), "quality".to_string()],
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                priority: 8,
                is_active: true,
            },
            MemoryEntry {
                id: "pattern_1".to_string(),
                title: "Rust Error Handling Pattern".to_string(),
                content: "Use Result<T, E> for fallible operations. Prefer ? operator for propagation. Use anyhow for application-level errors.".to_string(),
                memory_type: MemoryType::Pattern,
                tags: vec!["rust".to_string(), "error-handling".to_string()],
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                priority: 7,
                is_active: true,
            },
        ];

        let mut entries = self.entries.write().await;
        for entry in defaults {
            entries.insert(entry.id.clone(), entry);
        }

        Ok(())
    }

    /// Add a new memory entry
    pub async fn add_entry(&self, mut entry: MemoryEntry) -> Result<String> {
        entry.created_at = chrono::Utc::now().timestamp();
        entry.updated_at = entry.created_at;
        
        let mut entries = self.entries.write().await;
        entries.insert(entry.id.clone(), entry);
        
        info!("Added memory entry: {}", entry.id);
        Ok(entry.id)
    }

    /// Update an existing memory entry
    pub async fn update_entry(&self, id: &str, content: Option<String>, title: Option<String>) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(id) {
            if let Some(new_content) = content {
                entry.content = new_content;
            }
            if let Some(new_title) = title {
                entry.title = new_title;
            }
            entry.updated_at = chrono::Utc::now().timestamp();
            
            info!("Updated memory entry: {}", id);
            Ok(())
        } else {
            anyhow::bail!("Memory entry not found: {}", id)
        }
    }

    /// Delete a memory entry
    pub async fn delete_entry(&self, id: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if entries.remove(id).is_some() {
            info!("Deleted memory entry: {}", id);
            Ok(())
        } else {
            anyhow::bail!("Memory entry not found: {}", id)
        }
    }

    /// Get a specific memory entry
    pub async fn get_entry(&self, id: &str) -> Option<MemoryEntry> {
        let entries = self.entries.read().await;
        entries.get(id).cloned()
    }

    /// Search entries by type and tags
    pub async fn search_entries(&self, memory_type: Option<MemoryType>, tags: Option<Vec<String>>) -> Vec<MemoryEntry> {
        let entries = self.entries.read().await;
        
        entries.values()
            .filter(|e| {
                if !e.is_active {
                    return false;
                }
                if let Some(ref mtype) = memory_type {
                    if &e.memory_type != mtype {
                        return false;
                    }
                }
                if let Some(ref search_tags) = tags {
                    if !search_tags.iter().any(|t| e.tags.contains(t)) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Get all entries of a specific type (for indexing)
    pub async fn get_by_type(&self, memory_type: MemoryType) -> Vec<MemoryEntry> {
        self.search_entries(Some(memory_type), None).await
    }

    /// Export all entries to JSON (for backup or transfer)
    pub async fn export_all(&self) -> Result<String> {
        let entries = self.entries.read().await;
        serde_json::to_string_pretty(&*entries)
            .context("Failed to serialize knowledge base")
    }
}
