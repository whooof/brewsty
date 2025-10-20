use crate::domain::{
    entities::{CleanupPreview, Package, PackageType},
    repositories::PackageRepository,
};
use anyhow::Result;
use std::sync::Arc;

pub struct RepositoryUseCase {
    repository: Arc<dyn PackageRepository>,
}

impl RepositoryUseCase {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self { repository }
    }

    pub fn repository(&self) -> Arc<dyn PackageRepository> {
        Arc::clone(&self.repository)
    }
}

pub struct ListInstalledPackages {
    use_case: RepositoryUseCase,
}

impl ListInstalledPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, package_type: PackageType) -> Result<Vec<Package>> {
        self.use_case
            .repository()
            .get_installed_packages(package_type)
            .await
    }
}

pub struct ListOutdatedPackages {
    use_case: RepositoryUseCase,
}

impl ListOutdatedPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, package_type: PackageType) -> Result<Vec<Package>> {
        self.use_case
            .repository()
            .get_outdated_packages(package_type)
            .await
    }
}

pub struct InstallPackage {
    use_case: RepositoryUseCase,
}

impl InstallPackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.use_case.repository().install_package(&package).await
    }
}

pub struct UninstallPackage {
    use_case: RepositoryUseCase,
}

impl UninstallPackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.use_case.repository().uninstall_package(&package).await
    }
}

pub struct UpdatePackage {
    use_case: RepositoryUseCase,
}

impl UpdatePackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.use_case.repository().update_package(&package).await
    }
}

pub struct UpdateAllPackages {
    use_case: RepositoryUseCase,
}

impl UpdateAllPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self) -> Result<()> {
        self.use_case.repository().update_all().await
    }
}

pub struct CleanCache {
    use_case: RepositoryUseCase,
}

impl CleanCache {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn preview(&self) -> Result<CleanupPreview> {
        self.use_case.repository().get_cleanup_preview().await
    }

    pub async fn execute(&self) -> Result<()> {
        self.use_case.repository().clean_cache().await
    }
}

pub struct CleanupOldVersions {
    use_case: RepositoryUseCase,
}

impl CleanupOldVersions {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn preview(&self) -> Result<CleanupPreview> {
        self.use_case
            .repository()
            .get_cleanup_old_versions_preview()
            .await
    }

    pub async fn execute(&self) -> Result<()> {
        self.use_case.repository().cleanup_old_versions().await
    }
}

pub struct SearchPackages {
    use_case: RepositoryUseCase,
}

impl SearchPackages {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, query: &str, package_type: PackageType) -> Result<Vec<Package>> {
        self.use_case
            .repository()
            .search_packages(query, package_type)
            .await
    }
}

pub struct GetPackageInfo {
    use_case: RepositoryUseCase,
}

impl GetPackageInfo {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, name: &str, package_type: PackageType) -> Result<Package> {
        self.use_case
            .repository()
            .get_package_info(name, package_type)
            .await
    }
}

pub struct PinPackage {
    use_case: RepositoryUseCase,
}

impl PinPackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.use_case.repository().pin_package(&package).await
    }
}

pub struct UnpinPackage {
    use_case: RepositoryUseCase,
}

impl UnpinPackage {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            use_case: RepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, package: Package) -> Result<()> {
        self.use_case.repository().unpin_package(&package).await
    }
}
