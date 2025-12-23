#![allow(dead_code)]
use crate::domain::entities::Package;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDto {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub package_type: String,
    pub installed: bool,
    pub outdated: bool,
}

impl From<Package> for PackageDto {
    fn from(package: Package) -> Self {
        Self {
            name: package.name,
            version: package.version,
            description: package.description,
            package_type: package.package_type.to_string(),
            installed: package.installed,
            outdated: package.outdated,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfoDto {
    pub total_size: u64,
    pub package_count: usize,
}
