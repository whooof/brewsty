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

macro_rules! package_use_case {
    // No-arg use case: fn execute(&self) -> Result<R>
    ($name:ident, $method:ident -> $ret:ty) => {
        pub struct $name {
            use_case: RepositoryUseCase,
        }
        impl $name {
            pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
                Self {
                    use_case: RepositoryUseCase::new(repository),
                }
            }
            pub async fn execute(&self) -> Result<$ret> {
                self.use_case.repository().$method().await
            }
        }
    };
    // PackageType arg: fn execute(&self, pt: PackageType) -> Result<R>
    // Must come before generic ($arg:ty) arm since PackageType matches $arg:ty
    ($name:ident, $method:ident(PackageType) -> $ret:ty) => {
        pub struct $name {
            use_case: RepositoryUseCase,
        }
        impl $name {
            pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
                Self {
                    use_case: RepositoryUseCase::new(repository),
                }
            }
            pub async fn execute(&self, package_type: PackageType) -> Result<$ret> {
                self.use_case.repository().$method(package_type).await
            }
        }
    };
    // Single owned-arg use case (passed as ref): fn execute(&self, arg: T) -> Result<R>
    ($name:ident, $method:ident($arg:ty) -> $ret:ty) => {
        pub struct $name {
            use_case: RepositoryUseCase,
        }
        impl $name {
            pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
                Self {
                    use_case: RepositoryUseCase::new(repository),
                }
            }
            pub async fn execute(&self, arg: $arg) -> Result<$ret> {
                self.use_case.repository().$method(&arg).await
            }
        }
    };
    // Single ref-arg use case: fn execute(&self, arg: &T) -> Result<R>
    ($name:ident, $method:ident(ref $arg:ty) -> $ret:ty) => {
        pub struct $name {
            use_case: RepositoryUseCase,
        }
        impl $name {
            pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
                Self {
                    use_case: RepositoryUseCase::new(repository),
                }
            }
            pub async fn execute(&self, arg: &$arg) -> Result<$ret> {
                self.use_case.repository().$method(arg).await
            }
        }
    };
}

package_use_case!(ListInstalledPackages, get_installed_packages(PackageType) -> Vec<Package>);
package_use_case!(ListOutdatedPackages, get_outdated_packages(PackageType) -> Vec<Package>);
package_use_case!(InstallPackage, install_package(Package) -> ());
package_use_case!(UninstallPackage, uninstall_package(Package) -> ());
package_use_case!(UpdatePackage, update_package(ref Package) -> ());
package_use_case!(UpdateAllPackages, update_all -> ());
package_use_case!(PinPackage, pin_package(Package) -> ());
package_use_case!(UnpinPackage, unpin_package(Package) -> ());

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
