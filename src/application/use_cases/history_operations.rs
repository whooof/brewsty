use crate::domain::entities::PackageType;
use crate::domain::entities::history::{OperationHistory, OperationType};
use crate::domain::repositories::HistoryRepository;
use anyhow::Result;
use std::sync::Arc;

/// Use case: Record an operation to the history log.
pub struct RecordOperation {
    repository: Arc<dyn HistoryRepository>,
}

impl RecordOperation {
    pub fn new(repository: Arc<dyn HistoryRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        history: &mut OperationHistory,
        operation: OperationType,
        target: Option<String>,
        package_type: Option<PackageType>,
        success: bool,
        detail: Option<String>,
    ) -> Result<u64> {
        let id = history.add(operation, target, package_type, success, detail);
        self.repository.save(history)?;
        Ok(id)
    }
}

/// Use case: Load the full operation history from disk.
pub struct LoadHistory {
    repository: Arc<dyn HistoryRepository>,
}

impl LoadHistory {
    pub fn new(repository: Arc<dyn HistoryRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<OperationHistory> {
        self.repository.load()
    }
}

/// Use case: Clear all operation history records.
pub struct ClearHistory {
    repository: Arc<dyn HistoryRepository>,
}

impl ClearHistory {
    pub fn new(repository: Arc<dyn HistoryRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, history: &mut OperationHistory) -> Result<()> {
        history.clear();
        self.repository.save(history)?;
        Ok(())
    }
}
