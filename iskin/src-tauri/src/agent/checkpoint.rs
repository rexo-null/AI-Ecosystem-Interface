use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use anyhow::Result;
use uuid::Uuid;

use super::planner::TaskId;
use super::validator::ValidationResult;

/// Checkpoint for task execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub task_id: Uuid,
    pub step_id: TaskId,
    pub step_type: String,
    pub input: String,
    pub output: Option<String>,
    pub validation_result: Option<ValidationResult>,
    pub artifacts_created: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub error: Option<String>,
}

/// Checkpoint manager for persistence
pub struct CheckpointManager {
    storage_path: PathBuf,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    /// Save a checkpoint
    pub fn save(&self, checkpoint: &Checkpoint) -> Result<()> {
        // Create directory if needed
        std::fs::create_dir_all(&self.storage_path)?;

        // Save as JSON
        let filename = format!("{}.json", checkpoint.step_id);
        let path = self.storage_path.join(&filename);

        let json = serde_json::to_string_pretty(checkpoint)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Load task history by task_id
    pub fn load_task_history(&self, task_id: Uuid) -> Result<Vec<Checkpoint>> {
        let mut checkpoints = Vec::new();

        if !self.storage_path.exists() {
            return Ok(checkpoints);
        }

        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(checkpoint) = serde_json::from_str::<Checkpoint>(&content) {
                    if checkpoint.task_id == task_id {
                        checkpoints.push(checkpoint);
                    }
                }
            }
        }

        // Sort by timestamp
        checkpoints.sort_by_key(|c| c.timestamp);
        Ok(checkpoints)
    }

    /// Get the latest checkpoint for a task
    pub fn get_latest(&self, task_id: Uuid) -> Result<Option<Checkpoint>> {
        let checkpoints = self.load_task_history(task_id)?;
        Ok(checkpoints.last().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_save_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mgr = CheckpointManager::new(temp_dir.path().to_path_buf());

        let checkpoint = Checkpoint {
            task_id: Uuid::new_v4(),
            step_id: Uuid::new_v4(),
            step_type: "GenerateCode".to_string(),
            input: "test input".to_string(),
            output: Some("test output".to_string()),
            validation_result: None,
            artifacts_created: vec!["test.rs".to_string()],
            timestamp: Utc::now(),
            error: None,
        };

        mgr.save(&checkpoint)?;

        let loaded = mgr.load_task_history(checkpoint.task_id)?;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].step_id, checkpoint.step_id);

        Ok(())
    }

    #[test]
    fn test_checkpoint_latest() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mgr = CheckpointManager::new(temp_dir.path().to_path_buf());
        let task_id = Uuid::new_v4();

        let cp1 = Checkpoint {
            task_id,
            step_id: Uuid::new_v4(),
            step_type: "Step1".to_string(),
            input: "input1".to_string(),
            output: None,
            validation_result: None,
            artifacts_created: vec![],
            timestamp: Utc::now(),
            error: None,
        };

        mgr.save(&cp1)?;

        let latest = mgr.get_latest(task_id)?;
        assert!(latest.is_some());

        Ok(())
    }
}
