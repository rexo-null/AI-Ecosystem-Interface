use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};

/// Rule priority levels (higher value = higher priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RulePriority {
    Default = 1,
    UserRule = 5,
    Protocol = 7,
    Constitution = 10,
}

/// Condition matching strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    Exact(String),
    Contains(String),
    Regex(String),
    Glob(String),
    FileExtension(String),
    Always,
}

impl ConditionType {
    /// Evaluate the condition against a given context string
    pub fn matches(&self, context: &str) -> bool {
        match self {
            ConditionType::Exact(pattern) => context == pattern,
            ConditionType::Contains(pattern) => {
                context.to_lowercase().contains(&pattern.to_lowercase())
            }
            ConditionType::Regex(pattern) => {
                match regex::Regex::new(pattern) {
                    Ok(re) => re.is_match(context),
                    Err(_) => {
                        warn!("Invalid regex pattern: {}", pattern);
                        false
                    }
                }
            }
            ConditionType::Glob(pattern) => {
                Self::glob_match(pattern, context)
            }
            ConditionType::FileExtension(ext) => {
                context.ends_with(&format!(".{}", ext))
            }
            ConditionType::Always => true,
        }
    }

    /// Simple glob matching (supports * and ?)
    fn glob_match(pattern: &str, text: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        Self::glob_match_recursive(&pattern_chars, &text_chars, 0, 0)
    }

    fn glob_match_recursive(
        pattern: &[char],
        text: &[char],
        pi: usize,
        ti: usize,
    ) -> bool {
        if pi == pattern.len() && ti == text.len() {
            return true;
        }
        if pi == pattern.len() {
            return false;
        }

        match pattern[pi] {
            '*' => {
                // Try matching zero or more characters
                for i in ti..=text.len() {
                    if Self::glob_match_recursive(pattern, text, pi + 1, i) {
                        return true;
                    }
                }
                false
            }
            '?' => {
                if ti < text.len() {
                    Self::glob_match_recursive(pattern, text, pi + 1, ti + 1)
                } else {
                    false
                }
            }
            c => {
                if ti < text.len() && text[ti] == c {
                    Self::glob_match_recursive(pattern, text, pi + 1, ti + 1)
                } else {
                    false
                }
            }
        }
    }
}

/// Action to execute when a rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Deny(String),
    Transform(String),
    Notify(String),
    Log(String),
    Custom(String),
}

/// A single rule in the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: ConditionType,
    pub action: RuleAction,
    pub priority: RulePriority,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Result of rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub rule_id: String,
    pub rule_name: String,
    pub matched: bool,
    pub action: RuleAction,
    pub priority: RulePriority,
}

/// Rules Engine - evaluates and enforces rules from the knowledge base
pub struct RulesEngine {
    rules: Arc<RwLock<HashMap<String, Rule>>>,
    storage_path: PathBuf,
}

impl RulesEngine {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        }
    }

    /// Initialize: load rules from disk and create defaults
    pub async fn initialize(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.storage_path).await
            .context("Failed to create rules directory")?;

        self.load_from_disk().await?;

        let rules = self.rules.read().await;
        if rules.is_empty() {
            drop(rules);
            self.create_default_rules().await?;
            self.save_to_disk().await?;
        }

        let count = self.rules.read().await.len();
        info!("Rules engine initialized with {} rules", count);
        Ok(())
    }

    /// Create default constitutional and protocol rules
    async fn create_default_rules(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        let defaults = vec![
            Rule {
                id: "rule_constitution_safety".to_string(),
                name: "Принцип безопасности".to_string(),
                description: "Запрет выполнения деструктивных команд без подтверждения пользователя".to_string(),
                condition: ConditionType::Regex(r"rm\s+-rf|format\s+c:|del\s+/[sS]|drop\s+database".to_string()),
                action: RuleAction::Deny("Деструктивная операция заблокирована. Требуется подтверждение пользователя.".to_string()),
                priority: RulePriority::Constitution,
                is_active: true,
                tags: vec!["безопасность".to_string(), "конституция".to_string()],
                created_at: now,
                updated_at: now,
            },
            Rule {
                id: "rule_constitution_no_secrets".to_string(),
                name: "Защита секретов".to_string(),
                description: "Запрет на вывод секретных ключей, паролей и токенов".to_string(),
                condition: ConditionType::Regex(r#"(?i)(api_key|secret|password|token|private_key)\s*=\s*['"]?\w+"#.to_string()),
                action: RuleAction::Deny("Обнаружен потенциальный секрет. Вывод заблокирован.".to_string()),
                priority: RulePriority::Constitution,
                is_active: true,
                tags: vec!["безопасность".to_string(), "секреты".to_string()],
                created_at: now,
                updated_at: now,
            },
            Rule {
                id: "rule_protocol_code_review".to_string(),
                name: "Протокол проверки кода".to_string(),
                description: "Обязательная проверка кода перед коммитом".to_string(),
                condition: ConditionType::Contains("git commit".to_string()),
                action: RuleAction::Notify("Рекомендация: выполнить проверку кода перед коммитом".to_string()),
                priority: RulePriority::Protocol,
                is_active: true,
                tags: vec!["протокол".to_string(), "код".to_string()],
                created_at: now,
                updated_at: now,
            },
            Rule {
                id: "rule_protocol_rust_errors".to_string(),
                name: "Обработка ошибок Rust".to_string(),
                description: "Использовать Result<T, E> и оператор ? вместо unwrap()".to_string(),
                condition: ConditionType::Regex(r#"\.unwrap\(\)"#.to_string()),
                action: RuleAction::Notify("Обнаружен unwrap(). Рекомендуется использовать оператор ? или обработку ошибок.".to_string()),
                priority: RulePriority::Protocol,
                is_active: true,
                tags: vec!["rust".to_string(), "качество".to_string()],
                created_at: now,
                updated_at: now,
            },
        ];

        let mut rules = self.rules.write().await;
        for rule in defaults {
            rules.insert(rule.id.clone(), rule);
        }

        info!("Created {} default rules", rules.len());
        Ok(())
    }

    /// Load rules from JSON files on disk
    async fn load_from_disk(&self) -> Result<()> {
        let rules_file = self.storage_path.join("rules.json");

        if !rules_file.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&rules_file).await
            .context("Failed to read rules file")?;

        let loaded: HashMap<String, Rule> = serde_json::from_str(&content)
            .context("Failed to parse rules JSON")?;

        let mut rules = self.rules.write().await;
        *rules = loaded;

        info!("Loaded {} rules from disk", rules.len());
        Ok(())
    }

    /// Save all rules to disk as JSON
    pub async fn save_to_disk(&self) -> Result<()> {
        let rules_file = self.storage_path.join("rules.json");
        let rules = self.rules.read().await;

        let content = serde_json::to_string_pretty(&*rules)
            .context("Failed to serialize rules")?;

        tokio::fs::write(&rules_file, content).await
            .context("Failed to write rules file")?;

        info!("Saved {} rules to disk", rules.len());
        Ok(())
    }

    /// Add a rule
    pub async fn add_rule(&self, rule: Rule) -> Result<String> {
        let id = rule.id.clone();
        self.rules.write().await.insert(id.clone(), rule);
        self.save_to_disk().await?;
        info!("Rule added: {}", id);
        Ok(id)
    }

    /// Update an existing rule
    pub async fn update_rule(&self, id: &str, updates: RuleUpdate) -> Result<()> {
        let mut rules = self.rules.write().await;
        let rule = rules.get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Rule not found: {}", id))?;

        if let Some(name) = updates.name {
            rule.name = name;
        }
        if let Some(description) = updates.description {
            rule.description = description;
        }
        if let Some(condition) = updates.condition {
            rule.condition = condition;
        }
        if let Some(action) = updates.action {
            rule.action = action;
        }
        if let Some(priority) = updates.priority {
            rule.priority = priority;
        }
        if let Some(is_active) = updates.is_active {
            rule.is_active = is_active;
        }
        if let Some(tags) = updates.tags {
            rule.tags = tags;
        }

        rule.updated_at = chrono::Utc::now().timestamp();
        drop(rules);

        self.save_to_disk().await?;
        info!("Rule updated: {}", id);
        Ok(())
    }

    /// Remove a rule (Constitution rules cannot be removed)
    pub async fn remove_rule(&self, id: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        let rule = rules.get(id)
            .ok_or_else(|| anyhow::anyhow!("Rule not found: {}", id))?;

        if rule.priority == RulePriority::Constitution {
            anyhow::bail!("Cannot remove Constitution rule: {}", id);
        }

        rules.remove(id);
        drop(rules);

        self.save_to_disk().await?;
        info!("Rule removed: {}", id);
        Ok(())
    }

    /// Get a rule by ID
    pub async fn get_rule(&self, id: &str) -> Option<Rule> {
        self.rules.read().await.get(id).cloned()
    }

    /// Get all rules
    pub async fn list_rules(&self) -> Vec<Rule> {
        self.rules.read().await.values().cloned().collect()
    }

    /// Get all active rules sorted by priority (highest first)
    pub async fn get_active_rules(&self) -> Vec<Rule> {
        let rules = self.rules.read().await;
        let mut active: Vec<Rule> = rules.values().filter(|r| r.is_active).cloned().collect();
        active.sort_by(|a, b| b.priority.cmp(&a.priority));
        active
    }

    /// Get rules filtered by tags
    pub async fn get_rules_by_tags(&self, tags: &[String]) -> Vec<Rule> {
        let rules = self.rules.read().await;
        rules
            .values()
            .filter(|r| tags.iter().any(|t| r.tags.contains(t)))
            .cloned()
            .collect()
    }

    /// Evaluate all active rules against a context string
    pub async fn evaluate(&self, context: &str) -> Vec<EvaluationResult> {
        let active_rules = self.get_active_rules().await;
        let mut results = Vec::new();

        for rule in active_rules {
            let matched = rule.condition.matches(context);

            if matched {
                results.push(EvaluationResult {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    matched: true,
                    action: rule.action.clone(),
                    priority: rule.priority,
                });
            }
        }

        results
    }

    /// Check if an action is allowed by the rules
    pub async fn is_allowed(&self, context: &str) -> (bool, Vec<EvaluationResult>) {
        let results = self.evaluate(context).await;

        let denied = results.iter().any(|r| matches!(r.action, RuleAction::Deny(_)));

        (!denied, results)
    }
}

/// Partial update for a rule
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub condition: Option<ConditionType>,
    pub action: Option<RuleAction>,
    pub priority: Option<RulePriority>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
}
