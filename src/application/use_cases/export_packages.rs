//! Export installed packages to JSON

use crate::domain::entities::Package;
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Package data for export
#[derive(Serialize)]
pub struct ExportPackage {
    pub name: String,
    pub version: String,
    pub package_type: String, // "formula" or "cask"
    pub installed_date: Option<String>,
}

impl From<&Package> for ExportPackage {
    fn from(pkg: &Package) -> Self {
        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone().unwrap_or_default(),
            package_type: match pkg.package_type {
                crate::domain::entities::PackageType::Formula => "formula".to_string(),
                crate::domain::entities::PackageType::Cask => "cask".to_string(),
            },
            installed_date: None, // Can be enhanced with actual install date tracking
        }
    }
}

/// Export packages to JSON file
pub fn export_packages_to_json(packages: &[Package], path: &Path) -> Result<()> {
    let export_data: Vec<ExportPackage> = packages.iter().map(ExportPackage::from).collect();

    let json = serde_json::to_string_pretty(&export_data)
        .context("Failed to serialize packages to JSON")?;

    fs::write(path, json).context("Failed to write JSON file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{PackageCategory, PackageType};
    use tempfile::tempdir;

    #[test]
    fn test_export_single_package() {
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
                name: "visual-studio-code".to_string(),
                version: Some("1.85.0".to_string()),
                available_version: None,
                description: None,
                package_type: PackageType::Cask,
                installed: true,
                outdated: false,
                version_load_failed: false,
                pinned: false,
                installed_size: None,
                category: PackageCategory::Development,
            },
        ];

        let dir = tempdir().unwrap();
        let path = dir.path().join("export.json");

        export_packages_to_json(&packages, &path).unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("git"));
        assert!(content.contains("visual-studio-code"));
    }
}
