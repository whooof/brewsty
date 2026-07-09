//! Import packages from JSON/YAML export file

use crate::domain::entities::PackageType;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Package data for import (matches ExportPackage structure)
#[derive(Debug, Deserialize)]
pub struct ImportPackage {
    pub name: String,
    pub version: String,
    pub package_type: String,
    #[serde(default)]
    pub installed_date: Option<String>,
}

/// Import result summary
#[derive(Debug, Default)]
pub struct ImportResult {
    pub total: usize,
    pub installed: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Import packages from JSON file
pub fn import_packages_from_json(path: &Path) -> Result<Vec<ImportPackage>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read import file: {}", path.display()))?;

    let packages: Vec<ImportPackage> =
        serde_json::from_str(&content).context("Failed to parse JSON import file")?;

    Ok(packages)
}

/// Import packages from YAML file
pub fn import_packages_from_yaml(path: &Path) -> Result<Vec<ImportPackage>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read import file: {}", path.display()))?;

    let packages: Vec<ImportPackage> =
        serde_yaml::from_str(&content).context("Failed to parse YAML import file")?;

    Ok(packages)
}

/// Determine file type and import accordingly
pub fn import_packages(path: &Path) -> Result<Vec<ImportPackage>> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "json" => import_packages_from_json(path),
        "yaml" | "yml" => import_packages_from_yaml(path),
        _ => anyhow::bail!("Unsupported file format: {}. Use .json or .yaml", extension),
    }
}

/// Validate package type string
pub fn parse_package_type(type_str: &str) -> Option<PackageType> {
    match type_str.to_lowercase().as_str() {
        "formula" => Some(PackageType::Formula),
        "cask" => Some(PackageType::Cask),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_import_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("import.json");

        let json = r#"[
            {"name": "git", "version": "2.42.0", "package_type": "formula"},
            {"name": "visual-studio-code", "version": "1.85.0", "package_type": "cask"}
        ]"#;

        fs::write(&path, json).unwrap();

        let packages = import_packages_from_json(&path).unwrap();
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "git");
        assert_eq!(packages[1].name, "visual-studio-code");
    }

    #[test]
    fn test_import_yaml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("import.yaml");

        let yaml = r#"
- name: git
  version: 2.42.0
  package_type: formula
- name: visual-studio-code
  version: 1.85.0
  package_type: cask
"#;

        fs::write(&path, yaml).unwrap();

        let packages = import_packages_from_yaml(&path).unwrap();
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "git");
    }

    #[test]
    fn test_parse_package_type() {
        assert_eq!(parse_package_type("formula"), Some(PackageType::Formula));
        assert_eq!(parse_package_type("Formula"), Some(PackageType::Formula));
        assert_eq!(parse_package_type("cask"), Some(PackageType::Cask));
        assert_eq!(parse_package_type("invalid"), None);
    }
}
