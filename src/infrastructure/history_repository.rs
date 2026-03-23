use crate::domain::entities::history::OperationHistory;
use crate::domain::repositories::HistoryRepository;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct FileHistoryRepository {
    history_path: PathBuf,
}

impl Default for FileHistoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl FileHistoryRepository {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("brewsty");

        Self {
            history_path: config_dir.join("history.json"),
        }
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self { history_path: path }
    }
}

impl HistoryRepository for FileHistoryRepository {
    fn load(&self) -> Result<OperationHistory> {
        if !self.history_path.exists() {
            return Ok(OperationHistory::default());
        }

        let content =
            fs::read_to_string(&self.history_path).context("Failed to read history file")?;

        let history = serde_json::from_str(&content).context("Failed to parse history file")?;

        Ok(history)
    }

    fn save(&self, history: &OperationHistory) -> Result<()> {
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content =
            serde_json::to_string_pretty(history).context("Failed to serialize history")?;

        fs::write(&self.history_path, content).context("Failed to write history file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::PackageType;
    use crate::domain::entities::history::OperationType;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_nonexistent_returns_default() {
        let repo = FileHistoryRepository::with_path(PathBuf::from(
            "/tmp/nonexistent_brewsty_test_history.json",
        ));
        let history = repo.load().unwrap();
        assert_eq!(history.records.len(), 0);
        assert_eq!(history.next_id, 1);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let repo = FileHistoryRepository::with_path(path);

        let mut history = OperationHistory::default();
        history.add(
            OperationType::Install,
            Some("wget".to_string()),
            Some(PackageType::Formula),
            true,
            Some("Installed wget 1.21".to_string()),
        );
        history.add(
            OperationType::Uninstall,
            Some("curl".to_string()),
            Some(PackageType::Formula),
            false,
            Some("Permission denied".to_string()),
        );

        repo.save(&history).unwrap();

        let loaded = repo.load().unwrap();
        assert_eq!(loaded.records.len(), 2);
        assert_eq!(loaded.next_id, 3);

        // Newest first
        assert_eq!(loaded.records[0].target.as_deref(), Some("curl"));
        assert!(!loaded.records[0].success);
        assert_eq!(loaded.records[1].target.as_deref(), Some("wget"));
        assert!(loaded.records[1].success);
    }

    #[test]
    fn test_corrupted_file_returns_error() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        fs::write(&path, "not valid json!!!").unwrap();

        let repo = FileHistoryRepository::with_path(path);
        assert!(repo.load().is_err());
    }
}
