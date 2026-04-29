use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};
use chrono::{DateTime, Utc};

/// Memory entry types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Constitution,
    Protocol,
    Pattern,
    UserRule,
    ToolDefinition,
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub tags: Vec<String>,
    pub priority: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_count: u32,
}

/// Search query for memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub limit: Option<usize>,
}

/// Memory Module - manages knowledge base and semantic search
#[allow(dead_code)]
pub struct MemoryModule {
    entries: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    module_id: String,
    // In future: qdrant_client: Arc<QdrantClient>
}

#[allow(dead_code)]
impl MemoryModule {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            module_id: "memory_module".to_string(),
        }
    }

    /// Add a new memory entry
    pub async fn add_entry(&self, entry: MemoryEntry) -> anyhow::Result<String> {
        let mut entries = self.entries.write().await;
        let id = entry.id.clone();
        entries.insert(id.clone(), entry);
        info!("Added memory entry: {}", id);
        Ok(id)
    }

    /// Update an existing entry
    pub async fn update_entry(&self, id: &str, updates: MemoryEntryUpdate) -> anyhow::Result<()> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(id) {
            if let Some(title) = updates.title {
                entry.title = title;
            }
            if let Some(content) = updates.content {
                entry.content = content;
            }
            if let Some(memory_type) = updates.memory_type {
                entry.memory_type = memory_type;
            }
            if let Some(tags) = updates.tags {
                entry.tags = tags;
            }
            if let Some(priority) = updates.priority {
                entry.priority = priority;
            }
            entry.updated_at = Utc::now();
            entry.access_count += 1;

            info!("Updated memory entry: {}", id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Memory entry not found: {}", id))
        }
    }

    /// Delete an entry
    pub async fn delete_entry(&self, id: &str) -> anyhow::Result<()> {
        let mut entries = self.entries.write().await;
        if entries.remove(id).is_some() {
            info!("Deleted memory entry: {}", id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Memory entry not found: {}", id))
        }
    }

    /// Search entries
    pub async fn search_entries(&self, query: MemoryQuery) -> Vec<MemoryEntry> {
        let entries = self.entries.read().await;
        let mut results: Vec<MemoryEntry> = entries.values()
            .filter(|entry| {
                // Filter by memory type
                if let Some(ref mem_type) = query.memory_type {
                    if !std::mem::discriminant(&entry.memory_type).eq(&std::mem::discriminant(mem_type)) {
                        return false;
                    }
                }

                // Filter by tags
                if let Some(ref query_tags) = query.tags {
                    if !query_tags.iter().any(|tag| entry.tags.contains(tag)) {
                        return false;
                    }
                }

                // Filter by keywords (simple text search)
                if let Some(ref keywords) = query.keywords {
                    let content_lower = entry.content.to_lowercase();
                    let title_lower = entry.title.to_lowercase();
                    if !keywords.iter().any(|kw| {
                        content_lower.contains(&kw.to_lowercase()) ||
                        title_lower.contains(&kw.to_lowercase())
                    }) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by priority (descending) then by access count
        results.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| b.access_count.cmp(&a.access_count))
        });

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        // Update access counts
        for entry in &results {
            if let Some(_e) = entries.get(&entry.id) {
                // Note: In real implementation, we'd update access_count here
                // but we can't modify while holding read lock
            }
        }

        results
    }

    /// Get entry by ID
    pub async fn get_entry(&self, id: &str) -> Option<MemoryEntry> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(id) {
            entry.access_count += 1;
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Get all entries
    pub async fn get_all_entries(&self) -> Vec<MemoryEntry> {
        let entries = self.entries.read().await;
        entries.values().cloned().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let entries = self.entries.read().await;
        let total_entries = entries.len();
        let mut type_counts = HashMap::new();

        for entry in entries.values() {
            *type_counts.entry(format!("{:?}", entry.memory_type)).or_insert(0) += 1;
        }

        MemoryStats {
            total_entries,
            type_counts,
        }
    }

    /// Semantic search (placeholder for future Qdrant integration)
    pub async fn semantic_search(&self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        warn!("Semantic search not implemented yet, falling back to keyword search");

        let query_obj = MemoryQuery {
            memory_type: None,
            tags: None,
            keywords: Some(query.split_whitespace().map(|s| s.to_string()).collect()),
            limit: Some(limit),
        };

        self.search_entries(query_obj).await
    }
}

/// Update structure for memory entries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryEntryUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub priority: Option<u8>,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub type_counts: HashMap<String, usize>,
}

#[async_trait::async_trait]
impl super::ISKINModule for MemoryModule {
    fn id(&self) -> &str {
        &self.module_id
    }

    fn name(&self) -> &str {
        "Memory & Knowledge Base"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing Memory Module");

        // Add default constitution entries
        let constitution_entry = MemoryEntry {
            id: "constitution_1".to_string(),
            title: "Core Safety Principle".to_string(),
            content: "The AI must prioritize user safety and never execute harmful commands without explicit confirmation.".to_string(),
            memory_type: MemoryType::Constitution,
            tags: vec!["safety".to_string(), "core".to_string()],
            priority: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };

        let protocol_entry = MemoryEntry {
            id: "protocol_1".to_string(),
            title: "Code Review Protocol".to_string(),
            content: "Always perform security analysis before executing code changes. Check for injection vulnerabilities and unsafe operations.".to_string(),
            memory_type: MemoryType::Protocol,
            tags: vec!["review".to_string(), "security".to_string()],
            priority: 9,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };

        self.add_entry(constitution_entry).await?;
        self.add_entry(protocol_entry).await?;

        info!("Memory Module initialized with {} entries", self.entries.read().await.len());
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        info!("Shutting down Memory Module");
        // In future: persist to disk/Qdrant
        Ok(())
    }

    async fn execute(&self, command: &str, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match command {
            "add_entry" => {
                let entry: MemoryEntry = serde_json::from_value(args)?;
                let id = self.add_entry(entry).await?;
                Ok(serde_json::json!({ "id": id }))
            }
            "get_entry" => {
                let id = args.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing id"))?;

                if let Some(entry) = self.get_entry(id).await {
                    Ok(serde_json::to_value(entry)?)
                } else {
                    Err(anyhow::anyhow!("Entry not found"))
                }
            }
            "search_entries" => {
                let query: MemoryQuery = serde_json::from_value(args)?;
                let results = self.search_entries(query).await;
                Ok(serde_json::to_value(results)?)
            }
            "update_entry" => {
                let id = args.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing id"))?;
                let updates: MemoryEntryUpdate = serde_json::from_value(
                    args.get("updates").cloned().unwrap_or(serde_json::Value::Null)
                )?;

                self.update_entry(id, updates).await?;
                Ok(serde_json::json!({ "success": true }))
            }
            "delete_entry" => {
                let id = args.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing id"))?;

                self.delete_entry(id).await?;
                Ok(serde_json::json!({ "success": true }))
            }
            "get_stats" => {
                let stats = self.get_stats().await;
                Ok(serde_json::to_value(stats)?)
            }
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }
}