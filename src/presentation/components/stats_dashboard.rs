//! Statistics and analytics for Brewsty

use crate::domain::entities::{OperationHistory, OperationType, Package};
use egui_plot::{Line, Plot, PlotPoints};
use std::collections::HashMap;

/// Package statistics
#[derive(Debug, Clone, Default)]
pub struct PackageStats {
    pub total_installed: usize,
    pub total_formulae: usize,
    pub total_casks: usize,
    pub total_outdated: usize,
    pub total_pinned: usize,
    pub total_size_bytes: u64,
}

impl PackageStats {
    pub fn from_packages(packages: &[Package]) -> Self {
        let mut stats = Self::default();

        for pkg in packages {
            stats.total_installed += 1;

            match pkg.package_type {
                crate::domain::entities::PackageType::Formula => {
                    stats.total_formulae += 1;
                }
                crate::domain::entities::PackageType::Cask => {
                    stats.total_casks += 1;
                }
            }

            if pkg.outdated {
                stats.total_outdated += 1;
            }

            if pkg.pinned {
                stats.total_pinned += 1;
            }

            if let Some(size) = pkg.installed_size {
                stats.total_size_bytes += size;
            }
        }

        stats
    }

    pub fn total_size_formatted(&self) -> String {
        self.format_bytes(self.total_size_bytes)
    }

    fn format_bytes(&self, bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

/// Operation history statistics
#[derive(Debug, Clone, Default)]
pub struct OperationStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub installs: usize,
    pub uninstalls: usize,
    pub updates: usize,
}

impl OperationStats {
    pub fn from_history(history: &OperationHistory) -> Self {
        let total_operations = history.records.len();
        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut installs = 0;
        let mut uninstalls = 0;
        let mut updates = 0;

        for record in &history.records {
            if record.success {
                successful_operations += 1;
            } else {
                failed_operations += 1;
            }

            match record.operation {
                OperationType::Install => installs += 1,
                OperationType::Uninstall => uninstalls += 1,
                OperationType::Update | OperationType::UpdateAll => updates += 1,
                _ => {}
            }
        }

        Self {
            total_operations,
            successful_operations,
            failed_operations,
            installs,
            uninstalls,
            updates,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            100.0
        } else {
            (self.successful_operations as f64 / self.total_operations as f64) * 100.0
        }
    }
}

/// Category distribution for packages
#[derive(Debug, Clone, Default)]
pub struct CategoryDistribution {
    pub categories: HashMap<String, usize>,
}

impl CategoryDistribution {
    pub fn from_packages(packages: &[Package]) -> Self {
        let mut dist = Self::default();

        for pkg in packages {
            let category_name = pkg.category.as_str().to_string();
            *dist.categories.entry(category_name).or_insert(0) += 1;
        }

        dist
    }

    pub fn to_plot_points(&self) -> PlotPoints<'_> {
        let mut points = Vec::new();
        for (i, (_category, count)) in self.categories.iter().enumerate() {
            points.push([i as f64, *count as f64]);
        }
        PlotPoints::from(points)
    }
}

/// Render the stats dashboard
pub fn render_stats_dashboard(
    ui: &mut egui::Ui,
    package_stats: &PackageStats,
    operation_stats: &OperationStats,
    category_dist: &CategoryDistribution,
) {
    ui.heading("📊 Package Statistics");
    ui.separator();

    // Package overview
    egui::Grid::new("package_stats_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label("📦 Total Installed:");
            ui.label(format!("{}", package_stats.total_installed));
            ui.end_row();

            ui.label("🍺 Formulae:");
            ui.label(format!("{}", package_stats.total_formulae));
            ui.end_row();

            ui.label("📱 Casks:");
            ui.label(format!("{}", package_stats.total_casks));
            ui.end_row();

            ui.label("🔄 Outdated:");
            ui.label(format!("{}", package_stats.total_outdated));
            ui.end_row();

            ui.label("📌 Pinned:");
            ui.label(format!("{}", package_stats.total_pinned));
            ui.end_row();

            ui.label("💾 Total Size:");
            ui.label(package_stats.total_size_formatted());
            ui.end_row();
        });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(10.0);

    // Operation statistics
    ui.heading("📈 Operation Statistics");
    ui.separator();

    egui::Grid::new("operation_stats_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            ui.label("Total Operations:");
            ui.label(format!("{}", operation_stats.total_operations));
            ui.end_row();

            ui.label("✅ Successful:");
            ui.label(format!("{}", operation_stats.successful_operations));
            ui.end_row();

            ui.label("❌ Failed:");
            ui.label(format!("{}", operation_stats.failed_operations));
            ui.end_row();

            ui.label("Success Rate:");
            ui.label(format!("{:.1}%", operation_stats.success_rate()));
            ui.end_row();

            ui.label("📥 Installs:");
            ui.label(format!("{}", operation_stats.installs));
            ui.end_row();

            ui.label("🗑️ Uninstalls:");
            ui.label(format!("{}", operation_stats.uninstalls));
            ui.end_row();

            ui.label("🔄 Updates:");
            ui.label(format!("{}", operation_stats.updates));
            ui.end_row();
        });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(10.0);

    // Category distribution chart
    ui.heading("📊 Package Categories");
    ui.separator();

    if !category_dist.categories.is_empty() {
        let points = category_dist.to_plot_points();
        let line = Line::new("Packages", points);

        Plot::new("category_plot")
            .legend(egui_plot::Legend::default().position(egui_plot::Corner::RightTop))
            .height(200.0)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    } else {
        ui.label("No package data available");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{OperationRecord, Package, PackageCategory, PackageType};

    #[test]
    fn test_package_stats_from_packages() {
        let packages = vec![
            Package {
                name: "git".to_string(),
                version: Some("2.42.0".to_string()),
                available_version: None,
                description: None,
                package_type: PackageType::Formula,
                installed: true,
                outdated: false,
                version_load_failed: false,
                pinned: false,
                installed_size: Some(1024 * 1024 * 50), // 50MB
                category: PackageCategory::Development,
            },
            Package {
                name: "visual-studio-code".to_string(),
                version: Some("1.85.0".to_string()),
                available_version: None,
                description: None,
                package_type: PackageType::Cask,
                installed: true,
                outdated: true,
                version_load_failed: false,
                pinned: true,
                installed_size: Some(1024 * 1024 * 300), // 300MB
                category: PackageCategory::Development,
            },
        ];

        let stats = PackageStats::from_packages(&packages);

        assert_eq!(stats.total_installed, 2);
        assert_eq!(stats.total_formulae, 1);
        assert_eq!(stats.total_casks, 1);
        assert_eq!(stats.total_outdated, 1);
        assert_eq!(stats.total_pinned, 1);
        assert_eq!(stats.total_size_bytes, 1024 * 1024 * 350);
    }

    #[test]
    fn test_format_bytes() {
        let stats = PackageStats::default();

        assert_eq!(stats.format_bytes(500), "500 B");
        assert_eq!(stats.format_bytes(1024), "1.00 KB");
        assert_eq!(stats.format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(stats.format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_operation_stats_from_history() {
        let mut history = OperationHistory::default();

        // Add some test records
        history.records.push(OperationRecord {
            id: 1,
            timestamp: chrono::Local::now(),
            operation: OperationType::Install,
            target: Some("git".to_string()),
            package_type: Some(PackageType::Formula),
            success: true,
            detail: None,
        });

        history.records.push(OperationRecord {
            id: 2,
            timestamp: chrono::Local::now(),
            operation: OperationType::Update,
            target: Some("git".to_string()),
            package_type: Some(PackageType::Formula),
            success: true,
            detail: None,
        });

        history.records.push(OperationRecord {
            id: 3,
            timestamp: chrono::Local::now(),
            operation: OperationType::Uninstall,
            target: Some("vim".to_string()),
            package_type: Some(PackageType::Formula),
            success: false,
            detail: Some("Failed".to_string()),
        });

        let stats = OperationStats::from_history(&history);

        assert_eq!(stats.total_operations, 3);
        assert_eq!(stats.successful_operations, 2);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.installs, 1);
        assert_eq!(stats.updates, 1);
        assert_eq!(stats.uninstalls, 1);
        assert!((stats.success_rate() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_category_distribution() {
        let packages = vec![
            Package {
                name: "git".to_string(),
                version: Some("2.42.0".to_string()),
                available_version: None,
                description: None,
                package_type: PackageType::Formula,
                installed: true,
                outdated: false,
                version_load_failed: false,
                pinned: false,
                installed_size: None,
                category: PackageCategory::Development,
            },
            Package {
                name: "python".to_string(),
                version: Some("3.11.0".to_string()),
                available_version: None,
                description: None,
                package_type: PackageType::Formula,
                installed: true,
                outdated: false,
                version_load_failed: false,
                pinned: false,
                installed_size: None,
                category: PackageCategory::Languages,
            },
            Package {
                name: "node".to_string(),
                version: Some("20.0.0".to_string()),
                available_version: None,
                description: None,
                package_type: PackageType::Formula,
                installed: true,
                outdated: false,
                version_load_failed: false,
                pinned: false,
                installed_size: None,
                category: PackageCategory::Languages,
            },
        ];

        let dist = CategoryDistribution::from_packages(&packages);

        assert_eq!(dist.categories.get("Development"), Some(&1));
        assert_eq!(dist.categories.get("Languages"), Some(&2));
    }
}
