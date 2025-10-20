use crate::domain::entities::{CleanupPreview, Package, PackageType};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PackageRepository: Send + Sync {
    async fn get_installed_packages(&self, package_type: PackageType) -> Result<Vec<Package>>;
    async fn get_outdated_packages(&self, package_type: PackageType) -> Result<Vec<Package>>;
    async fn install_package(&self, package: &Package) -> Result<()>;
    async fn uninstall_package(&self, package: &Package) -> Result<()>;
    async fn update_package(&self, package: &Package) -> Result<()>;
    async fn update_all(&self) -> Result<()>;
    async fn get_cleanup_preview(&self) -> Result<CleanupPreview>;
    async fn get_cleanup_old_versions_preview(&self) -> Result<CleanupPreview>;
    async fn clean_cache(&self) -> Result<()>;
    async fn cleanup_old_versions(&self) -> Result<()>;
    async fn search_packages(&self, query: &str, package_type: PackageType)
    -> Result<Vec<Package>>;
    async fn get_package_info(&self, name: &str, package_type: PackageType) -> Result<Package>;
    async fn pin_package(&self, package: &Package) -> Result<()>;
    async fn unpin_package(&self, package: &Package) -> Result<()>;
}
