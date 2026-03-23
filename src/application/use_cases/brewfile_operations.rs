//! Brewfile support - parse, generate, and sync Homebrew Bundles

use crate::domain::entities::PackageType;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Brewfile structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Brewfile {
    pub taps: Vec<String>,
    pub formulae: Vec<BrewfileEntry>,
    pub casks: Vec<BrewfileEntry>,
}

/// Brewfile entry with optional attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewfileEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Brewfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tap to the Brewfile
    pub fn add_tap(&mut self, tap: String) {
        if !self.taps.contains(&tap) {
            self.taps.push(tap);
        }
    }

    /// Add a formula to the Brewfile
    pub fn add_formula(&mut self, name: String, version: Option<String>) {
        if !self.formulae.iter().any(|f| f.name == name) {
            self.formulae.push(BrewfileEntry { name, version });
        }
    }

    /// Add a cask to the Brewfile
    pub fn add_cask(&mut self, name: String, version: Option<String>) {
        if !self.casks.iter().any(|c| c.name == name) {
            self.casks.push(BrewfileEntry { name, version });
        }
    }
}

/// Generate Brewfile from installed packages
pub fn generate_brewfile(packages: &[crate::domain::entities::Package]) -> Brewfile {
    let mut brewfile = Brewfile::new();

    for pkg in packages {
        match pkg.package_type {
            PackageType::Formula => {
                brewfile.add_formula(pkg.name.clone(), pkg.version.clone());
            }
            PackageType::Cask => {
                brewfile.add_cask(pkg.name.clone(), pkg.version.clone());
            }
        }
    }

    brewfile
}

/// Export Brewfile to Ruby DSL format (standard Homebrew Bundle format)
pub fn export_brewfile_ruby(brewfile: &Brewfile) -> String {
    let mut output = String::new();

    // Add taps
    for tap in &brewfile.taps {
        output.push_str(&format!("tap \"{}\"\n", tap));
    }
    if !brewfile.taps.is_empty() {
        output.push('\n');
    }

    // Add formulae
    for formula in &brewfile.formulae {
        if let Some(version) = &formula.version {
            output.push_str(&format!(
                "brew \"{}\", args: [\"--version\", \"{}\"]\n",
                formula.name, version
            ));
        } else {
            output.push_str(&format!("brew \"{}\"\n", formula.name));
        }
    }
    if !brewfile.formulae.is_empty() {
        output.push('\n');
    }

    // Add casks
    for cask in &brewfile.casks {
        if let Some(version) = &cask.version {
            output.push_str(&format!(
                "cask \"{}\", version: \"{}\"\n",
                cask.name, version
            ));
        } else {
            output.push_str(&format!("cask \"{}\"\n", cask.name));
        }
    }

    output
}

/// Parse Brewfile from Ruby DSL format
pub fn parse_brewfile_ruby(content: &str) -> Result<Brewfile> {
    let mut brewfile = Brewfile::new();

    for line in content.lines() {
        let line = line.trim();

        // Parse tap
        if line.starts_with("tap ") {
            if let Some(tap) = extract_quoted_string(line) {
                brewfile.add_tap(tap);
            }
        }
        // Parse brew (formula)
        else if line.starts_with("brew ") {
            if let Some(name) = extract_quoted_string(line) {
                let version = extract_version_arg(line);
                brewfile.add_formula(name, version);
            }
        }
        // Parse cask
        else if line.starts_with("cask ") {
            if let Some(name) = extract_quoted_string(line) {
                let version = extract_cask_version(line);
                brewfile.add_cask(name, version);
            }
        }
    }

    Ok(brewfile)
}

/// Extract quoted string from a line (e.g., tap "homebrew/core" -> homebrew/core)
fn extract_quoted_string(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line[start + 1..].find('"')?;
    Some(line[start + 1..start + 1 + end].to_string())
}

/// Extract version from brew line with args
fn extract_version_arg(line: &str) -> Option<String> {
    if let Some(pos) = line.find("--version") {
        let rest = &line[pos..];
        extract_quoted_string(rest)
    } else {
        None
    }
}

/// Extract version from cask line
fn extract_cask_version(line: &str) -> Option<String> {
    if let Some(pos) = line.find("version:") {
        let rest = &line[pos..];
        extract_quoted_string(rest)
    } else {
        None
    }
}

/// Save Brewfile to file
pub fn save_brewfile(brewfile: &Brewfile, path: &Path) -> Result<()> {
    let content = export_brewfile_ruby(brewfile);
    fs::write(path, content)
        .with_context(|| format!("Failed to write Brewfile to {}", path.display()))
}

/// Load Brewfile from file
pub fn load_brewfile(path: &Path) -> Result<Brewfile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read Brewfile from {}", path.display()))?;
    parse_brewfile_ruby(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_brewfile() {
        use crate::domain::entities::{Package, PackageCategory};

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

        let brewfile = generate_brewfile(&packages);
        assert_eq!(brewfile.formulae.len(), 1);
        assert_eq!(brewfile.casks.len(), 1);
        assert_eq!(brewfile.formulae[0].name, "git");
        assert_eq!(brewfile.casks[0].name, "visual-studio-code");
    }

    #[test]
    fn test_export_import_roundtrip() {
        let mut brewfile = Brewfile::new();
        brewfile.add_tap("homebrew/core".to_string());
        brewfile.add_formula("git".to_string(), Some("2.42.0".to_string()));
        brewfile.add_cask("visual-studio-code".to_string(), None);

        let ruby = export_brewfile_ruby(&brewfile);
        let parsed = parse_brewfile_ruby(&ruby).unwrap();

        assert_eq!(parsed.taps, brewfile.taps);
        assert_eq!(parsed.formulae.len(), brewfile.formulae.len());
        assert_eq!(parsed.casks.len(), brewfile.casks.len());
    }

    #[test]
    fn test_extract_quoted_string() {
        assert_eq!(
            extract_quoted_string("tap \"homebrew/core\""),
            Some("homebrew/core".to_string())
        );
        assert_eq!(
            extract_quoted_string("brew \"git\""),
            Some("git".to_string())
        );
        assert_eq!(extract_quoted_string("no quotes here"), None);
    }
}
