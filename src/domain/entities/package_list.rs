use super::PackageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageListItem {
    pub name: String,
    pub package_type: PackageType,
    pub version: Option<String>,
}

impl PackageListItem {
    pub fn new(name: String, package_type: PackageType) -> Self {
        Self {
            name,
            package_type,
            version: None,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageList {
    pub formulae: Vec<PackageListItem>,
    pub casks: Vec<PackageListItem>,
    pub export_date: Option<String>,
}

impl PackageList {
    pub fn new() -> Self {
        Self {
            formulae: Vec::new(),
            casks: Vec::new(),
            export_date: None,
        }
    }

    pub fn with_export_date(mut self, date: String) -> Self {
        self.export_date = Some(date);
        self
    }

    pub fn add_formula(&mut self, item: PackageListItem) {
        self.formulae.push(item);
    }

    pub fn add_cask(&mut self, item: PackageListItem) {
        self.casks.push(item);
    }

    pub fn total_count(&self) -> usize {
        self.formulae.len() + self.casks.len()
    }
}

impl Default for PackageList {
    fn default() -> Self {
        Self::new()
    }
}
