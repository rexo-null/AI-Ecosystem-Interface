// ISKIN Self-Improvement System - Experience Log, Failure Analyzer, Self-Improver, Meta-Learner
// Phase 7: Self-Improvement Components

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use anyhow::Result;
use log::{info, warn, error};

/// Unique experience identifier
pub type ExperienceId = uuid::Uuid;

/// Task outcome for experience logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskOutcome {
    Success,
    PartialSuccess,
    Failure,
    Timeout,
    Cancelled,
}

/// Single experience record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceLog {
    pub id: ExperienceId,
    pub task_description: String,
    pub steps_taken: Vec<String>,
    pub outcome: TaskOutcome,
    pub duration_seconds: f64,
    pub tokens_used: u64,
    pub errors_encountered: Vec<String>,
    pub successful_patterns: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, String>,
}

impl ExperienceLog {
    pub fn new(task_description: String) -> Self {
        Self {
            id: ExperienceId::new_v4(),
            task_description,
            steps_taken: Vec::new(),
            outcome: TaskOutcome::Cancelled,
            duration_seconds: 0.0,
            tokens_used: 0,
            errors_encountered: Vec::new(),
            successful_patterns: Vec::new(),
            timestamp: Utc::now(),
            context: HashMap::new(),
        }
    }

    pub fn add_step(&mut self, step: String) {
        self.steps_taken.push(step);
    }

    pub fn record_error(&mut self, error: String) {
        self.errors_encountered.push(error);
    }

    pub fn record_success_pattern(&mut self, pattern: String) {
        self.successful_patterns.push(pattern);
    }

    pub fn complete(&mut self, outcome: TaskOutcome, duration: f64, tokens: u64) {
        self.outcome = outcome;
        self.duration_seconds = duration;
        self.tokens_used = tokens;
    }
}

/// Experience storage and retrieval
pub struct ExperienceStore {
    logs: Vec<ExperienceLog>,
    storage_path: PathBuf,
    max_logs: usize,
}

impl ExperienceStore {
    pub fn new(storage_path: PathBuf, max_logs: usize) -> Self {
        Self {
            logs: Vec::new(),
            storage_path,
            max_logs,
        }
    }

    pub fn add_experience(&mut self, log: ExperienceLog) {
        self.logs.push(log);
        
        // Trim old logs if exceeding limit
        while self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
        
        info!("Added experience log: {}", self.logs.last().unwrap().id);
    }

    pub fn get_all(&self) -> &[ExperienceLog] {
        &self.logs
    }

    pub fn get_by_outcome(&self, outcome: TaskOutcome) -> Vec<&ExperienceLog> {
        self.logs.iter().filter(|l| l.outcome == outcome).collect()
    }

    pub fn get_failed_experiences(&self) -> Vec<&ExperienceLog> {
        self.get_by_outcome(TaskOutcome::Failure)
    }

    pub fn get_successful_experiences(&self) -> Vec<&ExperienceLog> {
        self.get_by_outcome(TaskOutcome::Success)
    }

    pub fn find_similar_tasks(&self, query: &str, limit: usize) -> Vec<&ExperienceLog> {
        let query_lower = query.to_lowercase();
        let mut scored: Vec<(usize, &ExperienceLog)> = self.logs.iter()
            .map(|log| {
                let score = if log.task_description.to_lowercase().contains(&query_lower) {
                    2
                } else {
                    0
                } + log.steps_taken.iter()
                    .filter(|s| s.to_lowercase().contains(&query_lower))
                    .count();
                (score, log)
            })
            .filter(|(score, _)| *score > 0)
            .collect();
        
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().take(limit).map(|(_, log)| log).collect()
    }

    pub fn save_to_disk(&self) -> Result<()> {
        std::fs::create_dir_all(&self.storage_path)?;
        let path = self.storage_path.join("experiences.json");
        let json = serde_json::to_string_pretty(&self.logs)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_disk(&mut self) -> Result<()> {
        let path = self.storage_path.join("experiences.json");
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            self.logs = serde_json::from_str(&content)?;
        }
        Ok(())
    }
}

/// Failure analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAnalysis {
    pub experience_id: ExperienceId,
    pub failure_type: String,
    pub root_cause: String,
    pub contributing_factors: Vec<String>,
    pub suggested_fixes: Vec<String>,
    pub prevention_strategies: Vec<String>,
    pub confidence_score: f32,
}

/// Failure Analyzer - analyzes failed experiences to identify patterns
pub struct FailureAnalyzer {
    experience_store: std::sync::Arc<std::sync::Mutex<ExperienceStore>>,
    failure_patterns: HashMap<String, usize>,
}

impl FailureAnalyzer {
    pub fn new(experience_store: std::sync::Arc<std::sync::Mutex<ExperienceStore>>) -> Self {
        Self {
            experience_store,
            failure_patterns: HashMap::new(),
        }
    }

    pub fn analyze_failure(&mut self, experience: &ExperienceLog) -> FailureAnalysis {
        let mut contributing_factors = Vec::new();
        let mut failure_type = "Unknown".to_string();
        let mut root_cause = "Unable to determine root cause".to_string();
        
        // Analyze error messages
        for error in &experience.errors_encountered {
            let error_lower = error.to_lowercase();
            
            if error_lower.contains("timeout") {
                failure_type = "Timeout".to_string();
                contributing_factors.push("Operation exceeded time limit".to_string());
            }
            if error_lower.contains("permission") || error_lower.contains("access denied") {
                failure_type = "PermissionError".to_string();
                contributing_factors.push("Insufficient permissions".to_string());
            }
            if error_lower.contains("syntax") || error_lower.contains("parse") {
                failure_type = "SyntaxError".to_string();
                contributing_factors.push("Invalid syntax in generated code".to_string());
            }
            if error_lower.contains("memory") || error_lower.contains("oom") {
                failure_type = "MemoryError".to_string();
                contributing_factors.push("Memory limit exceeded".to_string());
            }
            if error_lower.contains("network") || error_lower.contains("connection") {
                failure_type = "NetworkError".to_string();
                contributing_factors.push("Network connectivity issue".to_string());
            }
            
            // Track failure patterns
            *self.failure_patterns.entry(failure_type.clone()).or_insert(0) += 1;
        }
        
        // Generate suggested fixes based on failure type
        let suggested_fixes = self.generate_suggested_fixes(&failure_type);
        
        // Generate prevention strategies
        let prevention_strategies = self.generate_prevention_strategies(&failure_type, &contributing_factors);
        
        // Calculate confidence based on available data
        let confidence_score = if experience.errors_encountered.is_empty() {
            0.3
        } else {
            0.5 + (experience.errors_encountered.len() as f32 * 0.1).min(0.5)
        };
        
        FailureAnalysis {
            experience_id: experience.id,
            failure_type,
            root_cause,
            contributing_factors,
            suggested_fixes,
            prevention_strategies,
            confidence_score,
        }
    }

    fn generate_suggested_fixes(&self, failure_type: &str) -> Vec<String> {
        match failure_type {
            "Timeout" => vec![
                "Increase timeout threshold".to_string(),
                "Break task into smaller subtasks".to_string(),
                "Optimize algorithm complexity".to_string(),
            ],
            "PermissionError" => vec![
                "Request elevated permissions".to_string(),
                "Use sandboxed execution".to_string(),
                "Check file/directory ownership".to_string(),
            ],
            "SyntaxError" => vec![
                "Add syntax validation before execution".to_string(),
                "Use language-specific parser".to_string(),
                "Implement incremental code generation".to_string(),
            ],
            "MemoryError" => vec![
                "Reduce memory footprint".to_string(),
                "Process data in chunks".to_string(),
                "Increase container memory limit".to_string(),
            ],
            "NetworkError" => vec![
                "Add retry logic with exponential backoff".to_string(),
                "Implement offline fallback".to_string(),
                "Check network configuration".to_string(),
            ],
            _ => vec!["Review error logs manually".to_string()],
        }
    }

    fn generate_prevention_strategies(&self, failure_type: &str, factors: &[String]) -> Vec<String> {
        let mut strategies = Vec::new();
        
        match failure_type {
            "Timeout" => {
                strategies.push("Implement progress monitoring".to_string());
                strategies.push("Set intermediate deadlines".to_string());
            },
            "SyntaxError" => {
                strategies.push("Add pre-execution validation".to_string());
                strategies.push("Use tree-sitter for syntax checking".to_string());
            },
            _ => {}
        }
        
        // Add factor-specific strategies
        for factor in factors {
            if factor.contains("permissions") {
                strategies.push("Implement principle of least privilege".to_string());
            }
        }
        
        strategies
    }

    pub fn get_failure_statistics(&self) -> HashMap<String, usize> {
        self.failure_patterns.clone()
    }

    pub fn get_most_common_failures(&self, limit: usize) -> Vec<(String, usize)> {
        let mut patterns: Vec<_> = self.failure_patterns.iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(a.1));
        patterns.into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

/// Self-improvement action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementAction {
    UpdatePrompt(String),
    AddValidationRule(String),
    OptimizeAlgorithm(String),
    AddRetryLogic(String),
    AdjustTimeout(u64),
    ModifyResourceLimits { memory_mb: Option<u64>, cpu_limit: Option<f64> },
    CreateNewTool(String),
    DeprecatePattern(String),
}

/// Self-Improvement Plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPlan {
    pub id: ExperienceId,
    pub priority: u8,
    pub actions: Vec<ImprovementAction>,
    pub expected_impact: String,
    pub risk_level: String,
    pub rollback_plan: Option<String>,
}

/// Self-Improver - generates improvement plans from failure analysis
pub struct SelfImprover {
    improvement_history: Vec<ImprovementPlan>,
    pending_improvements: Vec<ImprovementPlan>,
}

impl SelfImprover {
    pub fn new() -> Self {
        Self {
            improvement_history: Vec::new(),
            pending_improvements: Vec::new(),
        }
    }

    pub fn generate_improvement_plan(
        &mut self,
        analysis: &FailureAnalysis,
        experience: &ExperienceLog,
    ) -> ImprovementPlan {
        let mut actions = Vec::new();
        
        // Generate actions based on suggested fixes
        for fix in &analysis.suggested_fixes {
            if fix.contains("validation") {
                actions.push(ImprovementAction::AddValidationRule(fix.clone()));
            }
            if fix.contains("timeout") || fix.contains("Threshold") {
                actions.push(ImprovementAction::AdjustTimeout(30));
            }
            if fix.contains("retry") {
                actions.push(ImprovementAction::AddRetryLogic(fix.clone()));
            }
        }
        
        // Add prompt updates for recurring issues
        if analysis.confidence_score > 0.7 {
            actions.push(ImprovementAction::UpdatePrompt(
                format!("Avoid patterns that led to: {}", analysis.failure_type)
            ));
        }
        
        let plan = ImprovementPlan {
            id: ExperienceId::new_v4(),
            priority: self.calculate_priority(&analysis),
            actions,
            expected_impact: format!("Reduce {} failures by 50%", analysis.failure_type),
            risk_level: if actions.len() > 3 { "Medium" } else { "Low" }.to_string(),
            rollback_plan: Some("Revert to previous configuration".to_string()),
        };
        
        self.pending_improvements.push(plan.clone());
        plan
    }

    fn calculate_priority(&self, analysis: &FailureAnalysis) -> u8 {
        let base_priority = 5u8;
        
        // Higher confidence = higher priority
        let confidence_bonus = (analysis.confidence_score * 3.0) as u8;
        
        // More errors = higher priority
        let error_bonus = (analysis.contributing_factors.len() / 2) as u8;
        
        (base_priority + confidence_bonus + error_bonus).min(10)
    }

    pub fn execute_improvement(&mut self, plan_id: ExperienceId) -> Result<bool> {
        let plan_idx = self.pending_improvements.iter()
            .position(|p| p.id == plan_id);
        
        if let Some(idx) = plan_idx {
            let plan = self.pending_improvements.remove(idx);
            info!("Executing improvement plan: {:?}", plan.actions);
            
            // In production, this would actually apply the improvements
            // For now, just mark as executed
            self.improvement_history.push(plan);
            return Ok(true);
        }
        
        Ok(false)
    }

    pub fn get_pending_improvements(&self) -> &[ImprovementPlan] {
        &self.pending_improvements
    }

    pub fn get_improvement_history(&self) -> &[ImprovementPlan] {
        &self.improvement_history
    }
}

impl Default for SelfImprover {
    fn default() -> Self {
        Self::new()
    }
}

/// Meta-learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearningStats {
    pub total_experiences: usize,
    pub success_rate: f64,
    pub avg_improvement_effectiveness: f64,
    pub learning_velocity: f64,
    pub recommended_focus_areas: Vec<String>,
}

/// Meta-Learner - learns from improvement outcomes to optimize learning strategy
pub struct MetaLearner {
    improvement_outcomes: HashMap<ExperienceId, bool>,
    learning_rate: f64,
    exploration_factor: f64,
}

impl MetaLearner {
    pub fn new() -> Self {
        Self {
            improvement_outcomes: HashMap::new(),
            learning_rate: 0.1,
            exploration_factor: 0.2,
        }
    }

    pub fn record_improvement_outcome(&mut self, plan_id: ExperienceId, successful: bool) {
        self.improvement_outcomes.insert(plan_id, successful);
        
        // Adjust learning rate based on success
        if successful {
            self.learning_rate = (self.learning_rate * 1.1).min(0.5);
        } else {
            self.learning_rate = (self.learning_rate * 0.9).max(0.01);
        }
    }

    pub fn get_stats(&self, experience_store: &ExperienceStore) -> MetaLearningStats {
        let total = experience_store.get_all().len();
        let successes = experience_store.get_successful_experiences().len();
        let success_rate = if total > 0 { successes as f64 / total as f64 } else { 0.0 };
        
        let total_improvements = self.improvement_outcomes.len();
        let successful_improvements = self.improvement_outcomes.values()
            .filter(|&&s| s)
            .count();
        let avg_effectiveness = if total_improvements > 0 {
            successful_improvements as f64 / total_improvements as f64
        } else {
            0.0
        };
        
        // Learning velocity: rate of improvement over time
        let learning_velocity = self.learning_rate * avg_effectiveness;
        
        // Recommend focus areas based on failure patterns
        let recommended_focus_areas = self.recommend_focus_areas(experience_store);
        
        MetaLearningStats {
            total_experiences: total,
            success_rate,
            avg_improvement_effectiveness: avg_effectiveness,
            learning_velocity,
            recommended_focus_areas,
        }
    }

    fn recommend_focus_areas(&self, experience_store: &ExperienceStore) -> Vec<String> {
        let mut areas = Vec::new();
        
        let failed = experience_store.get_failed_experiences();
        if failed.len() > 5 {
            areas.push("Error handling and recovery".to_string());
        }
        
        // Check for specific patterns
        let timeout_count = failed.iter()
            .filter(|e| e.errors_encountered.iter().any(|err| err.to_lowercase().contains("timeout")))
            .count();
        if timeout_count > 3 {
            areas.push("Timeout management".to_string());
        }
        
        let syntax_count = failed.iter()
            .filter(|e| e.errors_encountered.iter().any(|err| err.to_lowercase().contains("syntax")))
            .count();
        if syntax_count > 3 {
            areas.push("Code validation".to_string());
        }
        
        areas
    }

    pub fn get_learning_rate(&self) -> f64 {
        self.learning_rate
    }

    pub fn get_exploration_factor(&self) -> f64 {
        self.exploration_factor
    }

    pub fn adjust_exploration(&mut self, delta: f64) {
        self.exploration_factor = (self.exploration_factor + delta).clamp(0.0, 1.0);
    }
}

impl Default for MetaLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// Integrated Self-Improvement System
pub struct SelfImprovementSystem {
    pub experience_store: ExperienceStore,
    pub failure_analyzer: FailureAnalyzer,
    pub self_improver: SelfImprover,
    pub meta_learner: MetaLearner,
}

impl SelfImprovementSystem {
    pub fn new(storage_path: PathBuf) -> Self {
        let store = ExperienceStore::new(storage_path.clone(), 1000);
        let store_arc = std::sync::Arc::new(std::sync::Mutex::new(store));
        
        Self {
            experience_store: ExperienceStore::new(storage_path, 1000),
            failure_analyzer: FailureAnalyzer::new(store_arc.clone()),
            self_improver: SelfImprover::new(),
            meta_learner: MetaLearner::new(),
        }
    }

    pub fn log_experience(&mut self, mut log: ExperienceLog) {
        // Record in store
        self.experience_store.add_experience(log.clone());
        
        // If failed, analyze and create improvement plan
        if log.outcome == TaskOutcome::Failure {
            let analysis = self.failure_analyzer.analyze_failure(&log);
            let plan = self.self_improver.generate_improvement_plan(&analysis, &log);
            info!("Generated improvement plan: {:?} with priority {}", plan.id, plan.priority);
        }
    }

    pub fn get_meta_stats(&self) -> MetaLearningStats {
        self.meta_learner.get_stats(&self.experience_store)
    }

    pub fn save(&self) -> Result<()> {
        self.experience_store.save_to_disk()
    }

    pub fn load(&mut self) -> Result<()> {
        self.experience_store.load_from_disk()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_experience_log_creation() {
        let mut log = ExperienceLog::new("Test task".to_string());
        log.add_step("Step 1".to_string());
        log.record_error("Test error".to_string());
        log.complete(TaskOutcome::Failure, 10.0, 100);
        
        assert_eq!(log.outcome, TaskOutcome::Failure);
        assert_eq!(log.steps_taken.len(), 1);
        assert_eq!(log.errors_encountered.len(), 1);
    }

    #[test]
    fn test_experience_store() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = ExperienceStore::new(temp_dir.path().to_path_buf(), 10);
        
        let mut log = ExperienceLog::new("Task 1".to_string());
        log.complete(TaskOutcome::Success, 5.0, 50);
        store.add_experience(log);
        
        assert_eq!(store.get_all().len(), 1);
        assert_eq!(store.get_successful_experiences().len(), 1);
    }

    #[test]
    fn test_failure_analyzer() {
        let temp_dir = TempDir::new().unwrap();
        let store = ExperienceStore::new(temp_dir.path().to_path_buf(), 10);
        let store_arc = std::sync::Arc::new(std::sync::Mutex::new(store));
        let mut analyzer = FailureAnalyzer::new(store_arc);
        
        let mut log = ExperienceLog::new("Failing task".to_string());
        log.record_error("Timeout exceeded".to_string());
        log.complete(TaskOutcome::Failure, 30.0, 200);
        
        let analysis = analyzer.analyze_failure(&log);
        assert_eq!(analysis.failure_type, "Timeout");
        assert!(!analysis.suggested_fixes.is_empty());
    }

    #[test]
    fn test_self_improver() {
        let mut improver = SelfImprover::new();
        
        let analysis = FailureAnalysis {
            experience_id: ExperienceId::new_v4(),
            failure_type: "Timeout".to_string(),
            root_cause: "Operation took too long".to_string(),
            contributing_factors: vec!["Large input size".to_string()],
            suggested_fixes: vec!["Increase timeout".to_string()],
            prevention_strategies: vec![],
            confidence_score: 0.8,
        };
        
        let log = ExperienceLog::new("Test".to_string());
        let plan = improver.generate_improvement_plan(&analysis, &log);
        
        assert!(!plan.actions.is_empty());
        assert!(plan.priority > 0);
    }

    #[test]
    fn test_meta_learner() {
        let mut learner = MetaLearner::new();
        let temp_dir = TempDir::new().unwrap();
        let mut store = ExperienceStore::new(temp_dir.path().to_path_buf(), 10);
        
        // Add some experiences
        for i in 0..5 {
            let mut log = ExperienceLog::new(format!("Task {}", i));
            log.complete(if i < 3 { TaskOutcome::Success } else { TaskOutcome::Failure }, 10.0, 100);
            store.add_experience(log);
        }
        
        let stats = learner.get_stats(&store);
        assert_eq!(stats.total_experiences, 5);
        assert!((stats.success_rate - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_full_system() {
        let temp_dir = TempDir::new().unwrap();
        let mut system = SelfImprovementSystem::new(temp_dir.path().to_path_buf());
        
        // Log a successful experience
        let mut success_log = ExperienceLog::new("Successful task".to_string());
        success_log.complete(TaskOutcome::Success, 5.0, 50);
        system.log_experience(success_log);
        
        // Log a failed experience
        let mut fail_log = ExperienceLog::new("Failed task".to_string());
        fail_log.record_error("Timeout".to_string());
        fail_log.complete(TaskOutcome::Failure, 30.0, 200);
        system.log_experience(fail_log);
        
        // Check stats
        let stats = system.get_meta_stats();
        assert_eq!(stats.total_experiences, 2);
        
        // Save and load
        system.save().unwrap();
        system.load().unwrap();
    }
}
