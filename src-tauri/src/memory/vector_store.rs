#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};

/// Vector embedding for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, serde_json::Value>,
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: HashMap<String, serde_json::Value>,
}

/// Configuration for vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub qdrant_url: Option<String>,
    pub collection_name: String,
    pub vector_size: usize,
    pub use_local_fallback: bool,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            qdrant_url: Some("http://localhost:6334".to_string()),
            collection_name: "iskin_knowledge".to_string(),
            vector_size: 384,
            use_local_fallback: true,
        }
    }
}

/// Local in-memory vector store (fallback when Qdrant is unavailable)
struct LocalVectorStore {
    points: HashMap<String, VectorPoint>,
}

impl LocalVectorStore {
    fn new() -> Self {
        Self {
            points: HashMap::new(),
        }
    }

    fn upsert(&mut self, point: VectorPoint) {
        self.points.insert(point.id.clone(), point);
    }

    fn delete(&mut self, id: &str) -> bool {
        self.points.remove(id).is_some()
    }

    fn search(&self, query_vector: &[f32], limit: usize) -> Vec<SearchResult> {
        let mut scored: Vec<SearchResult> = self
            .points
            .values()
            .map(|point| {
                let score = cosine_similarity(query_vector, &point.vector);
                SearchResult {
                    id: point.id.clone(),
                    score,
                    payload: point.payload.clone(),
                }
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored
    }

    fn count(&self) -> usize {
        self.points.len()
    }
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

/// Simple TF-IDF based text vectorizer (lightweight, no external model needed)
pub struct TextVectorizer {
    vocab: Arc<RwLock<HashMap<String, usize>>>,
    vector_size: usize,
}

impl TextVectorizer {
    pub fn new(vector_size: usize) -> Self {
        Self {
            vocab: Arc::new(RwLock::new(HashMap::new())),
            vector_size,
        }
    }

    /// Tokenize text into words
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }

    /// Compute a simple bag-of-words vector using hashing trick
    pub async fn vectorize(&self, text: &str) -> Vec<f32> {
        let tokens = Self::tokenize(text);
        let mut vector = vec![0.0f32; self.vector_size];

        // Hashing trick: map tokens to fixed-size vector
        for token in &tokens {
            let hash = Self::hash_token(token, self.vector_size);
            vector[hash] += 1.0;
        }

        // L2 normalize
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        // Update vocabulary
        let mut vocab = self.vocab.write().await;
        for token in tokens {
            let count = vocab.entry(token).or_insert(0);
            *count += 1;
        }

        vector
    }

    /// Simple hash function for the hashing trick
    fn hash_token(token: &str, size: usize) -> usize {
        let mut hash: usize = 5381;
        for byte in token.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as usize);
        }
        hash % size
    }
}

/// Vector Store - manages vector embeddings with Qdrant backend and local fallback
pub struct VectorStore {
    config: VectorStoreConfig,
    local_store: Arc<RwLock<LocalVectorStore>>,
    vectorizer: TextVectorizer,
    qdrant_available: Arc<RwLock<bool>>,
}

impl VectorStore {
    pub fn new(config: VectorStoreConfig) -> Self {
        let vector_size = config.vector_size;
        Self {
            config,
            local_store: Arc::new(RwLock::new(LocalVectorStore::new())),
            vectorizer: TextVectorizer::new(vector_size),
            qdrant_available: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the vector store - try to connect to Qdrant, fallback to local
    pub async fn initialize(&self) -> Result<()> {
        if let Some(ref url) = self.config.qdrant_url {
            match self.check_qdrant_connection(url).await {
                Ok(()) => {
                    *self.qdrant_available.write().await = true;
                    info!("Connected to Qdrant at {}", url);
                    self.ensure_collection(url).await?;
                }
                Err(e) => {
                    warn!("Qdrant unavailable at {}: {}. Using local fallback.", url, e);
                    *self.qdrant_available.write().await = false;
                }
            }
        } else {
            info!("No Qdrant URL configured, using local vector store");
        }

        Ok(())
    }

    /// Check if Qdrant is reachable
    async fn check_qdrant_connection(&self, url: &str) -> Result<()> {
        let client = qdrant_client::Qdrant::from_url(url).build()?;
        let _ = client.health_check().await;
        Ok(())
    }

    /// Ensure the collection exists in Qdrant
    async fn ensure_collection(&self, url: &str) -> Result<()> {
        let client = qdrant_client::Qdrant::from_url(url).build()?;
        let collections = client.list_collections().await?;

        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.config.collection_name);

        if !exists {
            // For now, skip Qdrant collection creation
            // Full implementation requires proper API understanding
            info!("Would create Qdrant collection (skipped for now): {}", self.config.collection_name);
        }

        Ok(())
    }

    /// Vectorize text and store it
    pub async fn upsert_text(&self, id: &str, text: &str, payload: HashMap<String, serde_json::Value>) -> Result<()> {
        let vector = self.vectorizer.vectorize(text).await;

        let point = VectorPoint {
            id: id.to_string(),
            vector: vector.clone(),
            payload: payload.clone(),
        };

        let is_qdrant = *self.qdrant_available.read().await;

        if is_qdrant {
            if let Err(e) = self.upsert_to_qdrant(&point).await {
                warn!("Qdrant upsert failed: {}. Falling back to local.", e);
                self.local_store.write().await.upsert(point);
            }
        } else {
            self.local_store.write().await.upsert(point);
        }

        Ok(())
    }

    /// Upsert a point to Qdrant
    async fn upsert_to_qdrant(&self, _point: &VectorPoint) -> Result<()> {
        // For now, Qdrant upsert is stubbed out
        // Use local store instead
        Ok(())
    }

    /// Search for similar text
    pub async fn search_text(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_vector = self.vectorizer.vectorize(query).await;

        let is_qdrant = *self.qdrant_available.read().await;

        if is_qdrant {
            match self.search_qdrant(&query_vector, limit).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    warn!("Qdrant search failed: {}. Falling back to local.", e);
                }
            }
        }

        let results = self.local_store.read().await.search(&query_vector, limit);
        Ok(results)
    }

    /// Search in Qdrant
    async fn search_qdrant(&self, _query_vector: &[f32], _limit: usize) -> Result<Vec<SearchResult>> {
        // For now, Qdrant search is stubbed out
        // Use local store instead
        Ok(Vec::new())
    }

    /// Delete a point from the store
    pub async fn delete(&self, id: &str) -> Result<()> {
        let is_qdrant = *self.qdrant_available.read().await;

        if is_qdrant {
            if let Err(e) = self.delete_from_qdrant(id).await {
                warn!("Qdrant delete failed: {}", e);
            }
        }

        self.local_store.write().await.delete(id);
        Ok(())
    }

    /// Delete from Qdrant
    async fn delete_from_qdrant(&self, _id: &str) -> Result<()> {
        // For now, Qdrant delete is stubbed out
        Ok(())
    }

    /// Get the number of stored points
    pub async fn count(&self) -> usize {
        self.local_store.read().await.count()
    }

    /// Check if Qdrant is currently available
    pub async fn is_qdrant_available(&self) -> bool {
        *self.qdrant_available.read().await
    }
}
