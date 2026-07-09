//! History Timeline visualization component

use crate::domain::entities::{OperationHistory, OperationType};
use chrono::{DateTime, Local};
use egui::{Color32, RichText, ScrollArea, Ui};

/// Timeline entry for display
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub timestamp: DateTime<Local>,
    pub operation: OperationType,
    pub package_name: String,
    pub success: bool,
}

impl TimelineEntry {
    pub fn icon(&self) -> &'static str {
        match self.operation {
            OperationType::Install => "📥",
            OperationType::Uninstall => "🗑️",
            OperationType::Update | OperationType::UpdateAll => "🔄",
            OperationType::Pin => "📌",
            OperationType::Unpin => "📍",
            OperationType::CleanCache => "🧹",
            OperationType::CleanupOldVersions => "🗑️",
            OperationType::CleanOrphans => "✨",
            OperationType::BundleApply => "📦",
            OperationType::ServiceStart => "▶️",
            OperationType::ServiceStop => "⏹️",
            OperationType::ServiceRestart => "🔄",
        }
    }

    pub fn color(&self) -> Color32 {
        if !self.success {
            Color32::RED
        } else {
            match self.operation {
                OperationType::Install => Color32::from_rgb(46, 204, 113),
                OperationType::Uninstall => Color32::from_rgb(231, 76, 60),
                OperationType::Update | OperationType::UpdateAll => Color32::from_rgb(52, 152, 219),
                OperationType::Pin | OperationType::Unpin => Color32::from_rgb(241, 196, 15),
                OperationType::CleanCache
                | OperationType::CleanupOldVersions
                | OperationType::CleanOrphans => Color32::from_rgb(155, 89, 182),
                OperationType::BundleApply => Color32::from_rgb(230, 126, 34),
                OperationType::ServiceStart
                | OperationType::ServiceStop
                | OperationType::ServiceRestart => Color32::from_rgb(52, 73, 94),
            }
        }
    }

    pub fn format_time(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M").to_string()
    }

    pub fn format_relative_time(&self) -> String {
        let now = Local::now();
        let duration = now.signed_duration_since(self.timestamp);

        if duration.num_seconds() < 60 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{}d ago", duration.num_days())
        } else {
            self.format_time()
        }
    }
}

/// Convert OperationHistory to TimelineEntry list
pub fn history_to_timeline(history: &OperationHistory) -> Vec<TimelineEntry> {
    history
        .records
        .iter()
        .map(|op| TimelineEntry {
            timestamp: op.timestamp,
            operation: op.operation,
            package_name: op.target.clone().unwrap_or_default(),
            success: op.success,
        })
        .collect()
}

/// Render the history timeline
pub fn render_timeline(ui: &mut Ui, entries: &[TimelineEntry]) {
    ScrollArea::vertical().show(ui, |ui| {
        if entries.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No operations in history yet");
            });
            return;
        }

        for (idx, entry) in entries.iter().enumerate() {
            ui.horizontal(|ui| {
                // Timeline line
                if idx > 0 {
                    ui.vertical(|ui| {
                        ui.add_space(20.0);
                        ui.separator();
                    });
                } else {
                    ui.add_space(24.0);
                }

                // Icon
                ui.label(RichText::new(entry.icon()).size(20.0));

                // Content
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&entry.package_name)
                                .color(entry.color())
                                .strong(),
                        );
                        ui.label(
                            RichText::new(entry.format_relative_time())
                                .color(Color32::GRAY)
                                .small(),
                        );
                    });

                    ui.label(
                        RichText::new(format!("{:?}", entry.operation))
                            .color(Color32::GRAY)
                            .small(),
                    );
                });

                // Success indicator
                if entry.success {
                    ui.label(RichText::new("✅").small());
                } else {
                    ui.label(RichText::new("❌").small());
                }
            });

            if idx < entries.len() - 1 {
                ui.add_space(8.0);
            }
        }
    });
}

/// Group timeline entries by date
pub fn group_by_date(entries: &[TimelineEntry]) -> Vec<(String, Vec<&TimelineEntry>)> {
    let mut groups: std::collections::HashMap<String, Vec<&TimelineEntry>> =
        std::collections::HashMap::new();

    for entry in entries {
        let date_key = entry.timestamp.format("%Y-%m-%d").to_string();
        groups.entry(date_key).or_default().push(entry);
    }

    let mut sorted_groups: Vec<_> = groups.into_iter().collect();
    sorted_groups.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by date descending

    sorted_groups
}

/// Render grouped timeline
pub fn render_grouped_timeline(ui: &mut Ui, entries: &[TimelineEntry]) {
    let groups = group_by_date(entries);

    ScrollArea::vertical().show(ui, |ui| {
        if groups.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No operations in history yet");
            });
            return;
        }

        for (date, day_entries) in groups {
            // Date header
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new(date).strong().size(16.0));
            });
            ui.add_space(4.0);

            // Entries for this day
            for entry in day_entries {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new(entry.icon()).size(18.0));

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&entry.package_name)
                                    .color(entry.color())
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(entry.format_time())
                                    .color(Color32::GRAY)
                                    .small(),
                            );
                        });
                        ui.label(
                            RichText::new(format!("{:?}", entry.operation))
                                .color(Color32::GRAY)
                                .small(),
                        );
                    });

                    if entry.success {
                        ui.label(RichText::new("✅").small());
                    } else {
                        ui.label(RichText::new("❌").small());
                    }
                });
                ui.add_space(4.0);
            }

            ui.separator();
            ui.add_space(8.0);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_format_relative_time() {
        let now = Local::now();

        let entry_now = TimelineEntry {
            timestamp: now,
            operation: OperationType::Install,
            package_name: "test".to_string(),
            success: true,
        };
        assert_eq!(entry_now.format_relative_time(), "Just now");

        let entry_5min = TimelineEntry {
            timestamp: now - Duration::minutes(5),
            operation: OperationType::Install,
            package_name: "test".to_string(),
            success: true,
        };
        assert!(entry_5min.format_relative_time().contains("5m ago"));

        let entry_2h = TimelineEntry {
            timestamp: now - Duration::hours(2),
            operation: OperationType::Install,
            package_name: "test".to_string(),
            success: true,
        };
        assert!(entry_2h.format_relative_time().contains("2h ago"));

        let entry_3d = TimelineEntry {
            timestamp: now - Duration::days(3),
            operation: OperationType::Install,
            package_name: "test".to_string(),
            success: true,
        };
        assert!(entry_3d.format_relative_time().contains("3d ago"));
    }

    #[test]
    fn test_operation_icons() {
        let install = TimelineEntry {
            timestamp: Local::now(),
            operation: OperationType::Install,
            package_name: "test".to_string(),
            success: true,
        };
        assert_eq!(install.icon(), "📥");

        let uninstall = TimelineEntry {
            timestamp: Local::now(),
            operation: OperationType::Uninstall,
            package_name: "test".to_string(),
            success: true,
        };
        assert_eq!(uninstall.icon(), "🗑️");

        let update = TimelineEntry {
            timestamp: Local::now(),
            operation: OperationType::Update,
            package_name: "test".to_string(),
            success: true,
        };
        assert_eq!(update.icon(), "🔄");
    }

    #[test]
    fn test_group_by_date() {
        let now = Local::now();
        let entries = vec![
            TimelineEntry {
                timestamp: now,
                operation: OperationType::Install,
                package_name: "pkg1".to_string(),
                success: true,
            },
            TimelineEntry {
                timestamp: now - Duration::days(1),
                operation: OperationType::Update,
                package_name: "pkg2".to_string(),
                success: true,
            },
            TimelineEntry {
                timestamp: now - Duration::days(1),
                operation: OperationType::Uninstall,
                package_name: "pkg3".to_string(),
                success: true,
            },
        ];

        let groups = group_by_date(&entries);
        assert_eq!(groups.len(), 2); // Today and yesterday
        assert_eq!(groups[0].1.len(), 1); // Today has 1 entry
        assert_eq!(groups[1].1.len(), 2); // Yesterday has 2 entries
    }
}
