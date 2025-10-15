use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageType {
    Formula,
    Cask,
}

impl fmt::Display for PackageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageType::Formula => write!(f, "Formula"),
            PackageType::Cask => write!(f, "Cask"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: Option<String>,
    pub available_version: Option<String>,
    pub description: Option<String>,
    pub package_type: PackageType,
    pub installed: bool,
    pub outdated: bool,
    pub version_load_failed: bool,
}

impl Package {
    pub fn new(
        name: String,
        package_type: PackageType,
    ) -> Self {
        Self {
            name,
            version: None,
            available_version: None,
            description: None,
            package_type,
            installed: false,
            outdated: false,
            version_load_failed: false,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_available_version(mut self, version: String) -> Self {
        self.available_version = Some(version);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_installed(mut self, installed: bool) -> Self {
        self.installed = installed;
        self
    }

    pub fn set_outdated(mut self, outdated: bool) -> Self {
        self.outdated = outdated;
        self
    }

    pub fn set_version_load_failed(mut self, failed: bool) -> Self {
        self.version_load_failed = failed;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub total_size: u64,
    pub package_count: usize,
}

#[derive(Debug, Clone)]
pub struct CleanupItem {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct CleanupPreview {
    pub items: Vec<CleanupItem>,
    pub total_size: u64,
}
