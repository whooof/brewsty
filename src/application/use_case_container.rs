use crate::application::use_cases::*;
use crate::domain::repositories::PackageRepository;
use std::sync::Arc;

pub struct UseCaseContainer {
    pub list_installed: Arc<ListInstalledPackages>,
    pub list_outdated: Arc<ListOutdatedPackages>,
    pub install: Arc<InstallPackage>,
    pub uninstall: Arc<UninstallPackage>,
    pub update: Arc<UpdatePackage>,
    pub update_all: Arc<UpdateAllPackages>,
    pub clean_cache: Arc<CleanCache>,
    pub cleanup_old_versions: Arc<CleanupOldVersions>,
    pub search: Arc<SearchPackages>,
    pub get_package_info: Arc<GetPackageInfo>,
    pub pin: Arc<PinPackage>,
    pub unpin: Arc<UnpinPackage>,
}

impl UseCaseContainer {
    pub fn new(repository: Arc<dyn PackageRepository>) -> Self {
        Self {
            list_installed: Arc::new(ListInstalledPackages::new(Arc::clone(&repository))),
            list_outdated: Arc::new(ListOutdatedPackages::new(Arc::clone(&repository))),
            install: Arc::new(InstallPackage::new(Arc::clone(&repository))),
            uninstall: Arc::new(UninstallPackage::new(Arc::clone(&repository))),
            update: Arc::new(UpdatePackage::new(Arc::clone(&repository))),
            update_all: Arc::new(UpdateAllPackages::new(Arc::clone(&repository))),
            clean_cache: Arc::new(CleanCache::new(Arc::clone(&repository))),
            cleanup_old_versions: Arc::new(CleanupOldVersions::new(Arc::clone(&repository))),
            search: Arc::new(SearchPackages::new(Arc::clone(&repository))),
            get_package_info: Arc::new(GetPackageInfo::new(Arc::clone(&repository))),
            pin: Arc::new(PinPackage::new(Arc::clone(&repository))),
            unpin: Arc::new(UnpinPackage::new(Arc::clone(&repository))),
        }
    }
}
