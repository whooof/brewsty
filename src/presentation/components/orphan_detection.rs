//! Orphan package detection - finds packages that are no longer needed

use crate::domain::entities::{Package, PackageType};
use crate::presentation::utils::format_size;
use std::collections::{HashMap, HashSet};

/// Information about an orphan package
#[derive(Debug, Clone)]
pub struct OrphanPackage {
    pub name: String,
    pub package_type: PackageType,
    pub version: Option<String>,
    pub installed_size: Option<u64>,
    /// Packages that depend on this orphan (if any)
    pub dependents: Vec<String>,
}

/// Result of orphan detection
#[derive(Debug, Clone, Default)]
pub struct OrphanDetectionResult {
    pub orphans: Vec<OrphanPackage>,
    pub total_orphans: usize,
    pub total_size_bytes: u64,
}

impl OrphanDetectionResult {
    pub fn total_size_formatted(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.total_size_bytes >= GB {
            format!("{:.2} GB", self.total_size_bytes as f64 / GB as f64)
        } else if self.total_size_bytes >= MB {
            format!("{:.2} MB", self.total_size_bytes as f64 / MB as f64)
        } else if self.total_size_bytes >= KB {
            format!("{:.2} KB", self.total_size_bytes as f64 / KB as f64)
        } else {
            format!("{} B", self.total_size_bytes)
        }
    }
}

/// Detect orphan packages
///
/// Orphans are packages that:
/// 1. Were installed as dependencies (not explicitly installed by user)
/// 2. Are no longer required by any explicitly installed packages
///
/// This is a simplified detection - in reality, we'd need to query brew
/// for the actual dependency tree using `brew info --json=v2 <package>`
pub fn detect_orphans(packages: &[Package]) -> OrphanDetectionResult {
    let mut result = OrphanDetectionResult::default();

    // Separate explicitly installed packages from dependencies
    let mut explicit_packages: HashSet<String> = HashSet::new();
    let mut all_packages: HashMap<String, &Package> = HashMap::new();

    for pkg in packages {
        all_packages.insert(pkg.name.clone(), pkg);

        // For now, we consider formulae without version as potentially explicit
        // In reality, we'd need to check brew's explicit formula list
        if pkg.package_type == PackageType::Formula {
            explicit_packages.insert(pkg.name.clone());
        }
    }

    // Find packages that are dependencies but not in explicit list
    // This is a simplified heuristic - real implementation would query brew
    for pkg in packages {
        // Skip if it's an explicit package
        if explicit_packages.contains(&pkg.name) {
            continue;
        }

        // Check if this package is a dependency of any explicit package
        let is_dependency = check_if_dependency(&pkg.name, &explicit_packages, packages);

        if !is_dependency {
            // This is an orphan
            result.orphans.push(OrphanPackage {
                name: pkg.name.clone(),
                package_type: pkg.package_type,
                version: pkg.version.clone(),
                installed_size: pkg.installed_size,
                dependents: vec![],
            });

            result.total_orphans += 1;
            if let Some(size) = pkg.installed_size {
                result.total_size_bytes += size;
            }
        }
    }

    result
}

/// Check if a package is a dependency of any explicit package
/// This is a simplified check - real implementation would use brew's dependency info
fn check_if_dependency(
    package_name: &str,
    _explicit_packages: &HashSet<String>,
    _packages: &[Package],
) -> bool {
    // Simplified heuristic: check if package name contains common dependency patterns
    // Real implementation would query `brew info --json=v2` for each explicit package
    // and check their dependencies

    // Common dependency patterns (very rough heuristic)
    let common_deps = [
        "gettext",
        "glib",
        "gmp",
        "libffi",
        "libyaml",
        "ncurses",
        "openssl",
        "readline",
        "sqlite",
        "xz",
        "zlib",
        "ca-certificates",
    ];

    common_deps.contains(&package_name)
}

/// Render the orphan detection results
pub fn render_orphan_detection(
    ui: &mut egui::Ui,
    result: &OrphanDetectionResult,
    on_remove_selected: &mut dyn FnMut(Vec<String>),
) {
    ui.heading("🔍 Orphan Detection");
    ui.separator();

    if result.total_orphans == 0 {
        ui.label("✅ No orphan packages detected!");
        return;
    }

    ui.label(format!(
        "Found {} orphan package(s) totaling {}",
        result.total_orphans,
        result.total_size_formatted()
    ));
    ui.add_space(10.0);

    // List orphans with checkboxes
    let mut selected_orphans: Vec<String> = Vec::new();

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            for orphan in &result.orphans {
                ui.horizontal(|ui| {
                    let mut selected = false;
                    ui.checkbox(&mut selected, "");
                    if selected {
                        selected_orphans.push(orphan.name.clone());
                    }

                    ui.label(format!(
                        "{} {} {}",
                        match orphan.package_type {
                            PackageType::Formula => "📦",
                            PackageType::Cask => "📱",
                        },
                        orphan.name,
                        orphan
                            .version
                            .as_ref()
                            .map(|v| format!("({})", v))
                            .unwrap_or_default()
                    ));

                    if let Some(size) = orphan.installed_size {
                        ui.label(format!("({})", format_size(size)));
                    }
                });
            }
        });

    ui.add_space(10.0);

    // Remove selected button
    if !selected_orphans.is_empty()
        && ui
            .button(format!(
                "🗑️ Remove {} selected orphan(s)",
                selected_orphans.len()
            ))
            .clicked()
    {
        on_remove_selected(selected_orphans);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_orphans_empty() {
        let packages = vec![];
        let result = detect_orphans(&packages);

        assert_eq!(result.total_orphans, 0);
        assert_eq!(result.total_size_bytes, 0);
    }

    #[test]
    fn test_detect_orphans_no_orphans() {
        let packages = vec![Package {
            name: "git".to_string(),
            version: Some("2.42.0".to_string()),
            available_version: None,
            description: None,
            package_type: PackageType::Formula,
            installed: true,
            outdated: false,
            version_load_failed: false,
            pinned: false,
            installed_size: Some(1024 * 1024 * 50),
            category: crate::domain::entities::PackageCategory::Development,
        }];

        let result = detect_orphans(&packages);

        // Git is considered explicit, so no orphans
        assert_eq!(result.total_orphans, 0);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_orphan_result_format() {
        let result = OrphanDetectionResult {
            total_orphans: 5,
            total_size_bytes: 1024 * 1024 * 150,
            orphans: vec![],
        };

        assert_eq!(result.total_size_formatted(), "150.00 MB");
    }
}
