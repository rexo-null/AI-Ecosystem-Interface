use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

/// Semantic indexing entry for code/document search
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexEntry {
    pub id: String,
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub symbols: Vec<String>,
}

/// Semantic Indexer - code and document indexing via Tree-sitter and vector embeddings
pub struct SemanticIndexer {
    index: Arc<RwLock<HashMap<String, IndexEntry>>>,
    root_path: Option<PathBuf>,
}

impl SemanticIndexer {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
            root_path: None,
        }
    }

    pub fn with_root(root_path: PathBuf) -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
            root_path: Some(root_path),
        }
    }

    /// Index a single file
    pub async fn index_file(&self, path: &str, content: &str, language: &str) -> Result<String> {
        let id = format!("idx_{}", uuid::Uuid::new_v4());
        let entry = IndexEntry {
            id: id.clone(),
            file_path: path.to_string(),
            content: content.to_string(),
            language: language.to_string(),
            symbols: Vec::new(), // TODO: extract symbols via Tree-sitter
        };

        self.index.write().await.insert(id.clone(), entry);
        info!("Indexed file: {}", path);
        Ok(id)
    }

    /// Search the index by keyword
    pub async fn search(&self, query: &str, limit: usize) -> Vec<IndexEntry> {
        let index = self.index.read().await;
        let query_lower = query.to_lowercase();

        let mut results: Vec<IndexEntry> = index
            .values()
            .filter(|entry| {
                entry.content.to_lowercase().contains(&query_lower)
                    || entry.file_path.to_lowercase().contains(&query_lower)
                    || entry.symbols.iter().any(|s| s.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        results.truncate(limit);
        results
    }

    /// Remove a file from the index
    pub async fn remove_file(&self, path: &str) -> Result<()> {
        let mut index = self.index.write().await;
        index.retain(|_, entry| entry.file_path != path);
        info!("Removed from index: {}", path);
        Ok(())
    }

    /// Clear the entire index
    pub async fn clear(&self) {
        self.index.write().await.clear();
        info!("Index cleared");
    }

    /// Get the total number of indexed entries
    pub async fn count(&self) -> usize {
        self.index.read().await.len()
    }
}
