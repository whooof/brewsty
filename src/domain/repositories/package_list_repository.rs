use crate::domain::entities::PackageList;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PackageListRepository: Send + Sync {
    async fn export_package_list(&self) -> Result<PackageList>;
    async fn import_packages(&self, package_list: &PackageList) -> Result<Vec<String>>;
}
