use crate::domain::entities::history::OperationHistory;
use anyhow::Result;

/// Repository for persisting operation history.
pub trait HistoryRepository: Send + Sync {
    fn load(&self) -> Result<OperationHistory>;
    fn save(&self, history: &OperationHistory) -> Result<()>;
}
