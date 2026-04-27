use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

/// Rule priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RulePriority {
    Constitution = 10,
    Protocol = 7,
    UserRule = 5,
    Default = 1,
}

/// A single rule in the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: String,
    pub action: String,
    pub priority: RulePriority,
    pub is_active: bool,
}

/// Rules Engine - evaluates and enforces rules from the knowledge base
pub struct RulesEngine {
    rules: Arc<RwLock<HashMap<String, Rule>>>,
}

impl RulesEngine {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load rules from storage
    pub async fn initialize(&self) -> Result<()> {
        info!("Rules engine initialized");
        Ok(())
    }

    /// Add a rule
    pub async fn add_rule(&self, rule: Rule) -> Result<String> {
        let id = rule.id.clone();
        self.rules.write().await.insert(id.clone(), rule);
        info!("Rule added: {}", id);
        Ok(id)
    }

    /// Remove a rule
    pub async fn remove_rule(&self, id: &str) -> Result<()> {
        self.rules
            .write()
            .await
            .remove(id)
            .ok_or_else(|| anyhow::anyhow!("Rule not found: {}", id))?;
        info!("Rule removed: {}", id);
        Ok(())
    }

    /// Get all active rules sorted by priority
    pub async fn get_active_rules(&self) -> Vec<Rule> {
        let rules = self.rules.read().await;
        let mut active: Vec<Rule> = rules.values().filter(|r| r.is_active).cloned().collect();
        active.sort_by(|a, b| b.priority.cmp(&a.priority));
        active
    }

    /// Evaluate rules against a given context (placeholder for future implementation)
    pub async fn evaluate(&self, context: &str) -> Vec<Rule> {
        let rules = self.get_active_rules().await;
        rules
            .into_iter()
            .filter(|rule| context.contains(&rule.condition))
            .collect()
    }
}
