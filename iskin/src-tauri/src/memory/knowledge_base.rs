#![allow(dead_code)]

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

/// Memory entry types in the hierarchical knowledge base
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    Constitution,
    Protocol,
    Pattern,
    ProjectContext,
    UserRule,
    ToolDefinition,
}

impl MemoryType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Constitution" => Some(MemoryType::Constitution),
            "Protocol" => Some(MemoryType::Protocol),
            "Pattern" => Some(MemoryType::Pattern),
            "ProjectContext" => Some(MemoryType::ProjectContext),
            "UserRule" => Some(MemoryType::UserRule),
            "ToolDefinition" => Some(MemoryType::ToolDefinition),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            MemoryType::Constitution => "Constitution",
            MemoryType::Protocol => "Protocol",
            MemoryType::Pattern => "Pattern",
            MemoryType::ProjectContext => "ProjectContext",
            MemoryType::UserRule => "UserRule",
            MemoryType::ToolDefinition => "ToolDefinition",
        }
    }
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
    pub priority: u8,
    pub is_active: bool,
    pub access_count: u64,
}

/// Search options for querying the knowledge base
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub min_priority: Option<u8>,
    pub limit: Option<usize>,
    pub sort_by_access: bool,
}

/// Hierarchical Knowledge Base with disk persistence
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
        tokio::fs::create_dir_all(&self.storage_path).await
            .context("Failed to create knowledge base directory")?;

        self.load_from_disk().await?;

        let entries = self.entries.read().await;
        if entries.is_empty() {
            drop(entries);
            self.create_default_entries().await?;
            self.save_to_disk().await?;
        }

        let count = self.entries.read().await.len();
        info!("Knowledge base initialized at {:?} with {} entries", self.storage_path, count);
        Ok(())
    }

    /// Load entries from JSON file on disk
    async fn load_from_disk(&self) -> Result<()> {
        let kb_file = self.storage_path.join("knowledge_base.json");

        if !kb_file.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&kb_file).await
            .context("Failed to read knowledge base file")?;

        let loaded: HashMap<String, MemoryEntry> = serde_json::from_str(&content)
            .context("Failed to parse knowledge base JSON")?;

        let mut entries = self.entries.write().await;
        *entries = loaded;

        info!("Loaded {} entries from disk", entries.len());
        Ok(())
    }

    /// Save all entries to disk as JSON
    pub async fn save_to_disk(&self) -> Result<()> {
        let kb_file = self.storage_path.join("knowledge_base.json");
        let entries = self.entries.read().await;

        let content = serde_json::to_string_pretty(&*entries)
            .context("Failed to serialize knowledge base")?;

        tokio::fs::write(&kb_file, content).await
            .context("Failed to write knowledge base file")?;

        Ok(())
    }

    /// Create default constitution and protocol entries
    async fn create_default_entries(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        let defaults = vec![
            MemoryEntry {
                id: "constitution_1".to_string(),
                title: "Принцип безопасности".to_string(),
                content: "Никогда не выполнять команды, которые могут повредить хост-систему или удалить критические файлы без явного подтверждения пользователя.".to_string(),
                memory_type: MemoryType::Constitution,
                tags: vec!["безопасность".to_string(), "основы".to_string()],
                created_at: now,
                updated_at: now,
                priority: 10,
                is_active: true,
                access_count: 0,
            },
            MemoryEntry {
                id: "protocol_1".to_string(),
                title: "Протокол проверки кода".to_string(),
                content: "1. Анализ изменений кода\n2. Проверка потенциальных ошибок\n3. Проверка тестового покрытия\n4. Соответствие стандартам кодирования\n5. Подтверждение пользователя перед мёржем".to_string(),
                memory_type: MemoryType::Protocol,
                tags: vec!["проверка".to_string(), "качество".to_string()],
                created_at: now,
                updated_at: now,
                priority: 8,
                is_active: true,
                access_count: 0,
            },
            MemoryEntry {
                id: "pattern_1".to_string(),
                title: "Обработка ошибок в Rust".to_string(),
                content: "Использовать Result<T, E> для fallible операций. Предпочитать оператор ? для проброса ошибок. Использовать anyhow для application-level ошибок, thiserror для library-level.".to_string(),
                memory_type: MemoryType::Pattern,
                tags: vec!["rust".to_string(), "ошибки".to_string()],
                created_at: now,
                updated_at: now,
                priority: 7,
                is_active: true,
                access_count: 0,
            },
        ];

        let mut entries = self.entries.write().await;
        for entry in defaults {
            entries.insert(entry.id.clone(), entry);
        }

        info!("Created {} default entries", entries.len());
        Ok(())
    }

    /// Add a new memory entry
    pub async fn add_entry(&self, mut entry: MemoryEntry) -> Result<String> {
        let now = chrono::Utc::now().timestamp();
        if entry.id.is_empty() {
            entry.id = format!("mem_{}", uuid::Uuid::new_v4());
        }
        entry.created_at = now;
        entry.updated_at = now;
        entry.access_count = 0;

        let id = entry.id.clone();
        self.entries.write().await.insert(id.clone(), entry);
        self.save_to_disk().await?;

        info!("Added knowledge entry: {}", id);
        Ok(id)
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

            info!("Updated knowledge entry: {}", id);
        } else {
            anyhow::bail!("Knowledge entry not found: {}", id)
        }

        drop(entries);
        self.save_to_disk().await?;
        Ok(())
    }

    /// Delete a memory entry
    pub async fn delete_entry(&self, id: &str) -> Result<()> {
        let mut entries = self.entries.write().await;

        if entries.remove(id).is_some() {
            info!("Deleted knowledge entry: {}", id);
            drop(entries);
            self.save_to_disk().await?;
            Ok(())
        } else {
            anyhow::bail!("Knowledge entry not found: {}", id)
        }
    }

    /// Get a specific memory entry (increments access_count)
    pub async fn get_entry(&self, id: &str) -> Option<MemoryEntry> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(id) {
            entry.access_count += 1;
            Some(entry.clone())
        } else {
            None
        }
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

    /// Full-text search across titles, content, and tags
    pub async fn full_text_search(&self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        let entries = self.entries.read().await;
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(f64, MemoryEntry)> = entries
            .values()
            .filter(|e| e.is_active)
            .filter_map(|entry| {
                let score = self.calculate_relevance_score(entry, &query_terms);
                if score > 0.0 {
                    Some((score, entry.clone()))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored.into_iter().map(|(_, entry)| entry).collect()
    }

    /// Calculate relevance score for full-text search
    fn calculate_relevance_score(&self, entry: &MemoryEntry, query_terms: &[&str]) -> f64 {
        let mut score = 0.0;
        let title_lower = entry.title.to_lowercase();
        let content_lower = entry.content.to_lowercase();
        let tags_lower: Vec<String> = entry.tags.iter().map(|t| t.to_lowercase()).collect();

        for term in query_terms {
            // Title match (weight: 3.0)
            if title_lower.contains(term) {
                score += 3.0;
            }
            // Tag match (weight: 2.0)
            if tags_lower.iter().any(|t| t.contains(term)) {
                score += 2.0;
            }
            // Content match (weight: 1.0)
            if content_lower.contains(term) {
                score += 1.0;
            }
        }

        // Priority boost
        score *= 1.0 + (entry.priority as f64 / 20.0);

        // Access frequency boost
        score *= 1.0 + (entry.access_count as f64).ln().max(0.0) / 10.0;

        score
    }

    /// Advanced search with multiple filters
    pub async fn search(&self, options: SearchOptions) -> Vec<MemoryEntry> {
        if let Some(ref query) = options.query {
            let limit = options.limit.unwrap_or(50);
            let mut results = self.full_text_search(query, limit).await;

            // Apply additional filters
            if let Some(ref mtype) = options.memory_type {
                results.retain(|e| &e.memory_type == mtype);
            }
            if let Some(ref tags) = options.tags {
                results.retain(|e| tags.iter().any(|t| e.tags.contains(t)));
            }
            if let Some(min_priority) = options.min_priority {
                results.retain(|e| e.priority >= min_priority);
            }

            results
        } else {
            self.search_entries(options.memory_type, options.tags).await
        }
    }

    /// Get all entries of a specific type
    pub async fn get_by_type(&self, memory_type: MemoryType) -> Vec<MemoryEntry> {
        self.search_entries(Some(memory_type), None).await
    }

    /// Get all entries sorted by priority
    pub async fn get_all_sorted(&self) -> Vec<MemoryEntry> {
        let entries = self.entries.read().await;
        let mut all: Vec<MemoryEntry> = entries.values().filter(|e| e.is_active).cloned().collect();
        all.sort_by(|a, b| b.priority.cmp(&a.priority));
        all
    }

    /// Get statistics about the knowledge base
    pub async fn stats(&self) -> KnowledgeBaseStats {
        let entries = self.entries.read().await;
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut total_access = 0u64;

        for entry in entries.values() {
            *type_counts.entry(entry.memory_type.as_str().to_string()).or_insert(0) += 1;
            total_access += entry.access_count;
        }

        KnowledgeBaseStats {
            total_entries: entries.len(),
            active_entries: entries.values().filter(|e| e.is_active).count(),
            type_counts,
            total_access_count: total_access,
        }
    }

    /// Export all entries to JSON string
    pub async fn export_all(&self) -> Result<String> {
        let entries = self.entries.read().await;
        serde_json::to_string_pretty(&*entries)
            .context("Failed to serialize knowledge base")
    }

    /// Import entries from JSON string (merge mode)
    pub async fn import_entries(&self, json: &str) -> Result<usize> {
        let imported: HashMap<String, MemoryEntry> = serde_json::from_str(json)
            .context("Failed to parse import JSON")?;

        let count = imported.len();
        let mut entries = self.entries.write().await;

        for (id, entry) in imported {
            entries.insert(id, entry);
        }

        drop(entries);
        self.save_to_disk().await?;

        info!("Imported {} entries", count);
        Ok(count)
    }
}

/// Knowledge base statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub type_counts: HashMap<String, usize>,
    pub total_access_count: u64,
}
