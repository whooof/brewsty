use crate::domain::{
    entities::{CleanupPreview, Package, PackageType},
    repositories::PackageRepository,
};
use anyhow::Result;
use std::sync::Arc;

pub struct ListInstalledPackages {
    repository: Arc<dyn PackageRepository>,
}

impl ListInstalledPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, package_type: PackageType) -> Result<Vec<Package>> {
        self.repository.get_installed_packages(package_type).await
    }
}

pub struct ListOutdatedPackages {
    repository: Arc<dyn PackageRepository>,
}

impl ListOutdatedPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, package_type: PackageType) -> Result<Vec<Package>> {
        self.repository.get_outdated_packages(package_type).await
    }
}

pub struct InstallPackage {
    repository: Arc<dyn PackageRepository>,
}

impl InstallPackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.repository.install_package(&package).await
    }
}

pub struct UninstallPackage {
    repository: Arc<dyn PackageRepository>,
}

impl UninstallPackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.repository.uninstall_package(&package).await
    }
}

pub struct UpdatePackage {
    repository: Arc<dyn PackageRepository>,
}

impl UpdatePackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.repository.update_package(&package).await
    }
}

pub struct UpdateAllPackages {
    repository: Arc<dyn PackageRepository>,
}

impl UpdateAllPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<()> {
        self.repository.update_all().await
    }
}

pub struct CleanCache {
    repository: Arc<dyn PackageRepository>,
}

impl CleanCache {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn preview(&self) -> Result<CleanupPreview> {
        self.repository.get_cleanup_preview().await
    }

    pub async fn execute(&self) -> Result<()> {
        self.repository.clean_cache().await
    }
}

pub struct CleanupOldVersions {
    repository: Arc<dyn PackageRepository>,
}

impl CleanupOldVersions {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn preview(&self) -> Result<CleanupPreview> {
        self.repository.get_cleanup_old_versions_preview().await
    }

    pub async fn execute(&self) -> Result<()> {
        self.repository.cleanup_old_versions().await
    }
}

pub struct SearchPackages {
    repository: Arc<dyn PackageRepository>,
}

impl SearchPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, query: &str, package_type: PackageType) -> Result<Vec<Package>> {
        self.repository.search_packages(query, package_type).await
    }
}

pub struct GetPackageInfo {
    repository: Arc<dyn PackageRepository>,
}

impl GetPackageInfo {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: &str, package_type: PackageType) -> Result<Package> {
        self.repository.get_package_info(name, package_type).await
    }
}
