// Context Compressor: Auto-summarization when approaching context limit
// Sliding window: system prompt + last N messages + compressed summary of early history

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Compression summary result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSummary {
    pub original_token_count: usize,
    pub compressed_token_count: usize,
    pub preserved_facts: Vec<String>,
    pub summary_text: String,
}

/// Message with token count tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedMessage {
    pub role: String,
    pub content: String,
    pub token_count: usize,
    pub is_critical: bool,
}

/// Context compressor for managing LLM context budget
pub struct ContextCompressor {
    messages: VecDeque<TrackedMessage>,
    system_prompt: String,
    max_tokens: usize,
    compression_threshold: f32, // e.g., 0.8 = compress at 80% capacity
    critical_fact_keywords: Vec<String>,
}

impl ContextCompressor {
    /// Create a new context compressor
    pub fn new(max_tokens: usize, system_prompt: String) -> Self {
        Self {
            messages: VecDeque::new(),
            system_prompt,
            max_tokens,
            compression_threshold: 0.8,
            critical_fact_keywords: vec![
                "error".to_string(),
                "failed".to_string(),
                "critical".to_string(),
                "plan".to_string(),
                "task".to_string(),
                "rollback".to_string(),
            ],
        }
    }

    /// Add a message to the context
    pub fn add_message(&mut self, role: String, content: String) {
        let token_count = Self::estimate_tokens(&content);
        let is_critical = self.is_critical_message(&content);

        self.messages.push_back(TrackedMessage {
            role,
            content,
            token_count,
            is_critical,
        });

        // Check if compression is needed
        if self.should_compress() {
            self.compress();
        }
    }

    /// Check if compression is needed
    pub fn should_compress(&self) -> bool {
        let current_tokens = self.current_token_count();
        let threshold = (self.max_tokens as f32 * self.compression_threshold) as usize;
        current_tokens > threshold
    }

    /// Get current token count
    pub fn current_token_count(&self) -> usize {
        self.system_prompt.len() / 4 + self.messages.iter().map(|m| m.token_count).sum::<usize>()
    }

    /// Compress old messages while preserving critical facts
    pub fn compress(&mut self) -> CompressionSummary {
        let original_count = self.current_token_count();
        
        // Identify critical messages to preserve
        let mut preserved_facts = Vec::new();
        let mut messages_to_keep: VecDeque<TrackedMessage> = VecDeque::new();

        // Always keep last N messages (sliding window)
        let keep_last_n = 5;
        let total_messages = self.messages.len();

        for (i, msg) in self.messages.iter().enumerate() {
            // Keep last N messages
            if i >= total_messages.saturating_sub(keep_last_n) {
                messages_to_keep.push_back(msg.clone());
                continue;
            }

            // Preserve critical messages
            if msg.is_critical {
                preserved_facts.push(format!("[{}]: {}", msg.role, msg.content));
                messages_to_keep.push_back(msg.clone());
            }
        }

        // Generate summary of compressed messages
        let compressed_count = total_messages - messages_to_keep.len();
        let summary_text = if compressed_count > 0 {
            format!(
                "[Compressed {} earlier messages. Key facts: {}]",
                compressed_count,
                preserved_facts.join("; ")
            )
        } else {
            String::new()
        };

        // Replace messages with compressed version
        self.messages = messages_to_keep;

        // Add summary as a system message if there was compression
        if !summary_text.is_empty() {
            let summary_tokens = Self::estimate_tokens(&summary_text);
            self.messages.push_front(TrackedMessage {
                role: "system".to_string(),
                content: summary_text.clone(),
                token_count: summary_tokens,
                is_critical: true,
            });
        }

        let compressed_count = self.current_token_count();

        CompressionSummary {
            original_token_count: original_count,
            compressed_token_count: compressed_count,
            preserved_facts,
            summary_text,
        }
    }

    /// Check if a message contains critical information
    fn is_critical_message(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();
        self.critical_fact_keywords.iter().any(|kw| content_lower.contains(kw))
    }

    /// Estimate token count (rough: 1 token ≈ 4 characters)
    fn estimate_tokens(text: &str) -> usize {
        (text.len() + 3) / 4
    }

    /// Get all messages for LLM context
    pub fn get_context(&self) -> Vec<&TrackedMessage> {
        self.messages.iter().collect()
    }

    /// Get system prompt
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// Clear all messages (reset to system prompt only)
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Set compression threshold
    pub fn set_compression_threshold(&mut self, threshold: f32) {
        self.compression_threshold = threshold.clamp(0.5, 0.95);
    }

    /// Add critical keyword
    pub fn add_critical_keyword(&mut self, keyword: String) {
        if !self.critical_fact_keywords.contains(&keyword) {
            self.critical_fact_keywords.push(keyword);
        }
    }
}

impl Default for ContextCompressor {
    fn default() -> Self {
        Self::new(
            6000,
            "You are ISKIN, an autonomous code agent.".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_compressor_creation() {
        let compressor = ContextCompressor::new(4000, "Test prompt".to_string());
        assert_eq!(compressor.max_tokens, 4000);
        assert_eq!(compressor.message_count(), 0);
    }

    #[test]
    fn test_add_message() {
        let mut compressor = ContextCompressor::default();
        compressor.add_message("user".to_string(), "Hello".to_string());
        assert_eq!(compressor.message_count(), 1);
    }

    #[test]
    fn test_token_estimation() {
        let tokens = ContextCompressor::estimate_tokens("hello world");
        assert!(tokens > 0 && tokens <= 10);
    }

    #[test]
    fn test_critical_message_detection() {
        let mut compressor = ContextCompressor::default();
        
        assert!(compressor.is_critical_message("An error occurred"));
        assert!(compressor.is_critical_message("Task failed"));
        assert!(!compressor.is_critical_message("Everything is fine"));
    }

    #[test]
    fn test_should_compress() {
        let mut compressor = ContextCompressor::new(100, "System".to_string());
        compressor.set_compression_threshold(0.5);
        
        // Add messages until we exceed threshold
        for i in 0..20 {
            compressor.add_message("user".to_string(), format!("Message {}", i));
        }
        
        assert!(compressor.should_compress());
    }

    #[test]
    fn test_compress_preserves_critical() {
        let mut compressor = ContextCompressor::new(200, "System".to_string());
        compressor.set_compression_threshold(0.5);
        
        // Add normal messages
        for i in 0..10 {
            compressor.add_message("user".to_string(), format!("Normal message {}", i));
        }
        
        // Add critical message
        compressor.add_message("assistant".to_string(), "Error: Critical failure detected".to_string());
        
        // Force compression
        let summary = compressor.compress();
        
        assert!(!summary.preserved_facts.is_empty());
        assert!(summary.summary_text.contains("Compressed"));
    }

    #[test]
    fn test_get_context() {
        let mut compressor = ContextCompressor::default();
        compressor.add_message("user".to_string(), "Hello".to_string());
        compressor.add_message("assistant".to_string(), "Hi there".to_string());
        
        let context = compressor.get_context();
        assert_eq!(context.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut compressor = ContextCompressor::default();
        compressor.add_message("user".to_string(), "Hello".to_string());
        assert_eq!(compressor.message_count(), 1);
        
        compressor.clear();
        assert_eq!(compressor.message_count(), 0);
    }

    #[test]
    fn test_current_token_count() {
        let mut compressor = ContextCompressor::new(1000, "System prompt".to_string());
        let initial_count = compressor.current_token_count();
        
        compressor.add_message("user".to_string(), "Hello world".to_string());
        let new_count = compressor.current_token_count();
        
        assert!(new_count > initial_count);
    }
}
