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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_package_list_is_empty() {
        let list = PackageList::new();
        assert!(list.formulae.is_empty());
        assert!(list.casks.is_empty());
        assert!(list.export_date.is_none());
        assert_eq!(list.total_count(), 0);
    }

    #[test]
    fn add_items() {
        let mut list = PackageList::new();
        list.add_formula(PackageListItem::new("wget".into(), PackageType::Formula));
        list.add_cask(PackageListItem::new("firefox".into(), PackageType::Cask));
        assert_eq!(list.total_count(), 2);
        assert_eq!(list.formulae.len(), 1);
        assert_eq!(list.casks.len(), 1);
    }

    #[test]
    fn package_list_item_with_version() {
        let item = PackageListItem::new("curl".into(), PackageType::Formula)
            .with_version("8.4.0".into());
        assert_eq!(item.name, "curl");
        assert_eq!(item.version.as_deref(), Some("8.4.0"));
    }

    #[test]
    fn serialize_deserialize() {
        let mut list = PackageList::new().with_export_date("2024-01-01".into());
        list.add_formula(
            PackageListItem::new("wget".into(), PackageType::Formula)
                .with_version("1.0".into()),
        );
        let json = serde_json::to_string(&list).unwrap();
        let deserialized: PackageList = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_count(), 1);
        assert_eq!(deserialized.export_date.as_deref(), Some("2024-01-01"));
    }
}
