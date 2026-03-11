use crate::domain::entities::PackageType;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The type of operation that was performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Install,
    Uninstall,
    Update,
    UpdateAll,
    Pin,
    Unpin,
    CleanCache,
    CleanupOldVersions,
    CleanOrphans,
    BundleApply,
    ServiceStart,
    ServiceStop,
    ServiceRestart,
}

impl OperationType {
    /// Returns the reverse operation, if one exists (for undo).
    pub fn reverse(&self) -> Option<OperationType> {
        match self {
            OperationType::Install => Some(OperationType::Uninstall),
            OperationType::Uninstall => Some(OperationType::Install),
            OperationType::Pin => Some(OperationType::Unpin),
            OperationType::Unpin => Some(OperationType::Pin),
            _ => None,
        }
    }

    /// Whether this operation can be undone.
    pub fn is_reversible(&self) -> bool {
        self.reverse().is_some()
    }
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::Install => write!(f, "Install"),
            OperationType::Uninstall => write!(f, "Uninstall"),
            OperationType::Update => write!(f, "Update"),
            OperationType::UpdateAll => write!(f, "Update All"),
            OperationType::Pin => write!(f, "Pin"),
            OperationType::Unpin => write!(f, "Unpin"),
            OperationType::CleanCache => write!(f, "Clean Cache"),
            OperationType::CleanupOldVersions => write!(f, "Cleanup Old Versions"),
            OperationType::CleanOrphans => write!(f, "Clean Orphans"),
            OperationType::BundleApply => write!(f, "Bundle Apply"),
            OperationType::ServiceStart => write!(f, "Service Start"),
            OperationType::ServiceStop => write!(f, "Service Stop"),
            OperationType::ServiceRestart => write!(f, "Service Restart"),
        }
    }
}

/// A single record in the operation history log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    /// Unique identifier for this record.
    pub id: u64,
    /// When the operation completed.
    pub timestamp: DateTime<Local>,
    /// The type of operation.
    pub operation: OperationType,
    /// The target package/service name (if applicable).
    pub target: Option<String>,
    /// The package type (Formula/Cask), if applicable.
    pub package_type: Option<PackageType>,
    /// Whether the operation succeeded.
    pub success: bool,
    /// Optional detail message (error message on failure, version info on success, etc.).
    pub detail: Option<String>,
}

impl OperationRecord {
    pub fn new(
        id: u64,
        operation: OperationType,
        target: Option<String>,
        package_type: Option<PackageType>,
        success: bool,
        detail: Option<String>,
    ) -> Self {
        Self {
            id,
            timestamp: Local::now(),
            operation,
            target,
            package_type,
            success,
            detail,
        }
    }

    /// Whether this operation can be undone.
    pub fn is_undoable(&self) -> bool {
        self.success && self.operation.is_reversible() && self.target.is_some()
    }

    /// Icon for the operation type.
    pub fn icon(&self) -> &str {
        match self.operation {
            OperationType::Install => "\u{2795}",             // +
            OperationType::Uninstall => "\u{2796}",           // -
            OperationType::Update => "\u{2B06}",              // up arrow
            OperationType::UpdateAll => "\u{2B06}",           // up arrow
            OperationType::Pin => "\u{1F4CC}",                // pin
            OperationType::Unpin => "\u{1F513}",              // unlock
            OperationType::CleanCache => "\u{1F9F9}",         // broom
            OperationType::CleanupOldVersions => "\u{1F9F9}", // broom
            OperationType::CleanOrphans => "\u{1F9F9}",       // broom
            OperationType::BundleApply => "\u{1F4E6}",        // package
            OperationType::ServiceStart => "\u{25B6}",        // play
            OperationType::ServiceStop => "\u{23F9}",         // stop
            OperationType::ServiceRestart => "\u{1F504}",     // counterclockwise arrows
        }
    }

    /// Status icon (success/failure).
    pub fn status_icon(&self) -> &str {
        if self.success {
            "\u{2705}" // green check
        } else {
            "\u{274C}" // red cross
        }
    }
}

/// The full operation history (persisted as JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationHistory {
    /// Auto-incrementing ID counter.
    pub next_id: u64,
    /// All recorded operations, newest first.
    pub records: Vec<OperationRecord>,
}

impl Default for OperationHistory {
    fn default() -> Self {
        Self {
            next_id: 1,
            records: Vec::new(),
        }
    }
}

impl OperationHistory {
    /// Maximum number of records to keep.
    const MAX_RECORDS: usize = 500;

    /// Add a new record. Returns the assigned ID.
    pub fn add(
        &mut self,
        operation: OperationType,
        target: Option<String>,
        package_type: Option<PackageType>,
        success: bool,
        detail: Option<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let record = OperationRecord::new(id, operation, target, package_type, success, detail);
        self.records.insert(0, record); // newest first

        // Trim to max size
        if self.records.len() > Self::MAX_RECORDS {
            self.records.truncate(Self::MAX_RECORDS);
        }

        id
    }

    /// Clear all records.
    pub fn clear(&mut self) {
        self.records.clear();
        // Don't reset next_id — IDs should remain unique across clears.
    }

    /// Get a record by its unique ID.
    pub fn get(&self, id: u64) -> Option<&OperationRecord> {
        self.records.iter().find(|record| record.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut history = OperationHistory::default();
        let id = history.add(
            OperationType::Install,
            Some("wget".to_string()),
            Some(PackageType::Formula),
            true,
            None,
        );
        assert_eq!(id, 1);
        assert_eq!(history.records.len(), 1);
        assert_eq!(history.records[0].target.as_deref(), Some("wget"));
        assert!(history.records[0].success);
    }

    #[test]
    fn test_newest_first() {
        let mut history = OperationHistory::default();
        history.add(OperationType::Install, Some("a".into()), None, true, None);
        history.add(OperationType::Install, Some("b".into()), None, true, None);
        assert_eq!(history.records[0].target.as_deref(), Some("b"));
        assert_eq!(history.records[1].target.as_deref(), Some("a"));
    }

    #[test]
    fn test_max_records_trim() {
        let mut history = OperationHistory::default();
        for i in 0..600 {
            history.add(
                OperationType::Install,
                Some(format!("pkg-{i}")),
                None,
                true,
                None,
            );
        }
        assert_eq!(history.records.len(), OperationHistory::MAX_RECORDS);
    }

    #[test]
    fn test_clear() {
        let mut history = OperationHistory::default();
        history.add(
            OperationType::Install,
            Some("wget".into()),
            None,
            true,
            None,
        );
        history.add(
            OperationType::Uninstall,
            Some("wget".into()),
            None,
            true,
            None,
        );
        assert_eq!(history.next_id, 3);
        history.clear();
        assert_eq!(history.records.len(), 0);
        assert_eq!(history.next_id, 3); // not reset
    }

    #[test]
    fn test_reversible() {
        assert!(OperationType::Install.is_reversible());
        assert!(OperationType::Uninstall.is_reversible());
        assert!(OperationType::Pin.is_reversible());
        assert!(OperationType::Unpin.is_reversible());
        assert!(!OperationType::Update.is_reversible());
        assert!(!OperationType::CleanCache.is_reversible());
    }

    #[test]
    fn test_undoable() {
        let mut history = OperationHistory::default();

        // Successful install with target — undoable
        history.add(
            OperationType::Install,
            Some("wget".into()),
            None,
            true,
            None,
        );
        assert!(history.records[0].is_undoable());

        // Failed install — not undoable
        history.add(
            OperationType::Install,
            Some("wget".into()),
            None,
            false,
            None,
        );
        assert!(!history.records[0].is_undoable());

        // Update (not reversible) — not undoable
        history.add(OperationType::Update, Some("wget".into()), None, true, None);
        assert!(!history.records[0].is_undoable());

        // No target — not undoable
        history.add(OperationType::Install, None, None, true, None);
        assert!(!history.records[0].is_undoable());
    }

    #[test]
    fn test_get_by_id() {
        let mut history = OperationHistory::default();
        let id1 = history.add(OperationType::Install, Some("a".into()), None, true, None);
        let id2 = history.add(OperationType::Uninstall, Some("b".into()), None, true, None);

        let r1 = history.get(id1).unwrap();
        assert_eq!(r1.target.as_deref(), Some("a"));

        let r2 = history.get(id2).unwrap();
        assert_eq!(r2.target.as_deref(), Some("b"));

        assert!(history.get(999).is_none());
    }
}
