use crate::domain::entities::{PackageList, brewfile::BrewfileSyncPreview};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PackageListRepository: Send + Sync {
    async fn export_package_list(&self) -> Result<PackageList>;
    async fn import_packages(&self, package_list: &PackageList) -> Result<Vec<String>>;

    // Brewfile operations
    async fn bundle_dump(&self, path: &str) -> Result<String>;
    async fn bundle_check_preview(&self, path: &str) -> Result<BrewfileSyncPreview>;
    async fn bundle_apply(&self, path: &str, install: bool, cleanup: bool) -> Result<()>;
}
