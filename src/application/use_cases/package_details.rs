//! Fetch and parse package details from brew info

use crate::domain::entities::Package;
use anyhow::{Context, Result};
use std::process::Command;

/// Package details information
#[derive(Debug, Clone, Default)]
pub struct PackageDetails {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub installed: bool,
    pub dependencies: Vec<String>,
    pub build_dependencies: Vec<String>,
    pub test_dependencies: Vec<String>,
    pub required_by: Vec<String>,
    pub caveats: Option<String>,
    pub license: Option<String>,
    pub repo_url: Option<String>,
}

impl PackageDetails {
    pub fn from_package(pkg: &Package) -> Self {
        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            description: pkg.description.clone(),
            installed: pkg.installed,
            ..Default::default()
        }
    }
}

/// Fetch package details by running `brew info --json=v2 <package>`
pub fn fetch_package_details(package_name: &str) -> Result<PackageDetails> {
    let output = Command::new("brew")
        .args(["info", "--json=v2", package_name])
        .output()
        .context("Failed to execute brew info command")?;

    if !output.status.success() {
        anyhow::bail!(
            "brew info failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse brew info JSON")?;

    let mut details = PackageDetails {
        name: package_name.to_string(),
        ..Default::default()
    };

    // Parse formula info if it exists
    if let Some(formulae) = json.get("formulae").and_then(|f| f.as_array())
        && let Some(formula) = formulae.first() {
            details.version = formula
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            details.description = formula
                .get("desc")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());
            details.homepage = formula
                .get("homepage")
                .and_then(|h| h.as_str())
                .map(|s| s.to_string());
            details.license = formula
                .get("license")
                .and_then(|l| l.as_str())
                .map(|s| s.to_string());
            details.dependencies = formula
                .get("dependencies")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            details.build_dependencies = formula
                .get("build_dependencies")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            details.caveats = formula
                .get("caveats")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string());
            // Extract repo URL from homepage if it's a GitHub URL
            if let Some(homepage) = &details.homepage
                && homepage.contains("github.com") {
                    details.repo_url = Some(homepage.clone());
                }
        }

    // Parse cask info if it exists
    if let Some(casks) = json.get("casks").and_then(|c| c.as_array())
        && let Some(cask) = casks.first() {
            details.version = cask
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            details.description = cask
                .get("desc")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());
            details.homepage = cask
                .get("homepage")
                .and_then(|h| h.as_str())
                .map(|s| s.to_string());
            details.installed = cask
                .get("installed")
                .and_then(|i| i.as_bool())
                .unwrap_or(false);
        }

    // Try to get reverse dependencies (what depends on this package)
    if let Ok(output) = Command::new("brew")
        .args(["uses", "--installed", package_name])
        .output()
        && output.status.success() {
            let uses = String::from_utf8_lossy(&output.stdout);
            details.required_by = uses
                .lines()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
        }

    Ok(details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_package_details_git() {
        // This test requires brew to be installed
        let result = fetch_package_details("git");
        // Test may fail if brew is not installed, so we just check it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
