//! Batch Operations Queue - queue and execute multiple operations

use crate::domain::entities::Package;
use std::collections::VecDeque;

/// Types of batch operations
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Install { package: Package },
    Uninstall { package: Package },
    Update { package: Package },
    UpdateAll,
}

impl BatchOperation {
    pub fn description(&self) -> String {
        match self {
            BatchOperation::Install { package } => {
                format!(
                    "📥 Install {} {}",
                    package.name,
                    package
                        .version
                        .as_ref()
                        .map(|v| format!("({})", v))
                        .unwrap_or_default()
                )
            }
            BatchOperation::Uninstall { package } => {
                format!(
                    "🗑️ Uninstall {} {}",
                    package.name,
                    package
                        .version
                        .as_ref()
                        .map(|v| format!("({})", v))
                        .unwrap_or_default()
                )
            }
            BatchOperation::Update { package } => {
                format!(
                    "🔄 Update {} {}",
                    package.name,
                    package
                        .version
                        .as_ref()
                        .map(|v| format!("({})", v))
                        .unwrap_or_default()
                )
            }
            BatchOperation::UpdateAll => "🔄 Update all packages".to_string(),
        }
    }

    pub fn package_name(&self) -> Option<&str> {
        match self {
            BatchOperation::Install { package }
            | BatchOperation::Uninstall { package }
            | BatchOperation::Update { package } => Some(&package.name),
            BatchOperation::UpdateAll => None,
        }
    }
}

/// Status of a batch operation
#[derive(Debug, Clone, PartialEq)]
pub enum BatchOperationStatus {
    Pending,
    InProgress,
    Completed { success: bool, message: String },
    Failed { error: String },
}

/// A queued operation with its status
#[derive(Debug, Clone)]
pub struct QueuedOperation {
    pub operation: BatchOperation,
    pub status: BatchOperationStatus,
}

/// Batch operations queue
#[derive(Debug, Clone, Default)]
pub struct BatchQueue {
    pub queue: VecDeque<QueuedOperation>,
    pub completed_count: usize,
    pub failed_count: usize,
}

impl BatchQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, operation: BatchOperation) {
        self.queue.push_back(QueuedOperation {
            operation,
            status: BatchOperationStatus::Pending,
        });
    }

    pub fn remove(&mut self, index: usize) -> Option<QueuedOperation> {
        if index < self.queue.len() {
            self.queue.remove(index)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.completed_count = 0;
        self.failed_count = 0;
    }

    pub fn clear_completed(&mut self) {
        self.queue
            .retain(|op| !matches!(op.status, BatchOperationStatus::Completed { .. }));
    }

    pub fn total_operations(&self) -> usize {
        self.queue.len()
    }

    pub fn pending_operations(&self) -> usize {
        self.queue
            .iter()
            .filter(|op| matches!(op.status, BatchOperationStatus::Pending))
            .count()
    }

    pub fn in_progress_operations(&self) -> usize {
        self.queue
            .iter()
            .filter(|op| matches!(op.status, BatchOperationStatus::InProgress))
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn has_pending(&self) -> bool {
        self.pending_operations() > 0
    }
}

/// Render the batch operations queue
pub fn render_batch_queue(
    ui: &mut egui::Ui,
    queue: &BatchQueue,
    on_execute: &mut dyn FnMut(),
    on_clear: &mut dyn FnMut(),
    on_remove: &mut dyn FnMut(usize),
) {
    ui.heading("📋 Batch Operations Queue");
    ui.separator();

    if queue.is_empty() {
        ui.label("Queue is empty. Add operations from package list.");
        return;
    }

    // Summary
    ui.horizontal(|ui| {
        ui.label(format!(
            "Total: {} | Pending: {} | In Progress: {} | Completed: {} | Failed: {}",
            queue.total_operations(),
            queue.pending_operations(),
            queue.in_progress_operations(),
            queue.completed_count,
            queue.failed_count
        ));
    });
    ui.add_space(10.0);

    // Operations list
    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            for (idx, queued_op) in queue.queue.iter().enumerate() {
                ui.horizontal(|ui| {
                    // Status icon
                    let status_icon = match &queued_op.status {
                        BatchOperationStatus::Pending => "⏳",
                        BatchOperationStatus::InProgress => "🔄",
                        BatchOperationStatus::Completed { success: true, .. } => "✅",
                        BatchOperationStatus::Completed { success: false, .. } => "❌",
                        BatchOperationStatus::Failed { .. } => "❌",
                    };
                    ui.label(status_icon);

                    // Operation description
                    ui.label(queued_op.operation.description());

                    // Remove button (only for pending)
                    if matches!(queued_op.status, BatchOperationStatus::Pending)
                        && ui.small_button("✕").clicked()
                    {
                        on_remove(idx);
                    }
                });
            }
        });

    ui.add_space(10.0);

    // Action buttons
    ui.horizontal(|ui| {
        if ui.button("▶️ Execute All").clicked() && queue.has_pending() {
            on_execute();
        }

        if ui.button("🗑️ Clear Completed").clicked() {
            on_clear();
        }

        if ui.button("🗑️ Clear All").clicked() {
            on_clear();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{PackageCategory, PackageType};

    fn create_test_package(name: &str) -> Package {
        Package {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            available_version: None,
            description: None,
            package_type: PackageType::Formula,
            installed: true,
            outdated: false,
            version_load_failed: false,
            pinned: false,
            installed_size: None,
            category: PackageCategory::Development,
        }
    }

    #[test]
    fn test_batch_queue_add() {
        let mut queue = BatchQueue::new();
        let pkg = create_test_package("git");

        queue.add(BatchOperation::Install { package: pkg });

        assert_eq!(queue.total_operations(), 1);
        assert_eq!(queue.pending_operations(), 1);
    }

    #[test]
    fn test_batch_queue_remove() {
        let mut queue = BatchQueue::new();
        let pkg = create_test_package("git");

        queue.add(BatchOperation::Install {
            package: pkg.clone(),
        });
        queue.add(BatchOperation::Uninstall { package: pkg });

        assert_eq!(queue.total_operations(), 2);

        queue.remove(0);

        assert_eq!(queue.total_operations(), 1);
    }

    #[test]
    fn test_batch_queue_clear() {
        let mut queue = BatchQueue::new();
        let pkg = create_test_package("git");

        queue.add(BatchOperation::Install { package: pkg });
        queue.clear();

        assert!(queue.is_empty());
        assert_eq!(queue.total_operations(), 0);
    }

    #[test]
    fn test_batch_operation_description() {
        let pkg = create_test_package("git");

        let install = BatchOperation::Install {
            package: pkg.clone(),
        };
        assert!(install.description().contains("Install"));

        let uninstall = BatchOperation::Uninstall {
            package: pkg.clone(),
        };
        assert!(uninstall.description().contains("Uninstall"));

        let update = BatchOperation::Update { package: pkg };
        assert!(update.description().contains("Update"));

        let update_all = BatchOperation::UpdateAll;
        assert!(update_all.description().contains("Update all"));
    }
}
