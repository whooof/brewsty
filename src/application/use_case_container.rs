use crate::application::use_cases::*;
use crate::domain::repositories::{
    HistoryRepository, PackageListRepository, PackageRepository, ServiceRepository,
};
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
    pub clean_orphans: Arc<CleanOrphans>,
    pub search: Arc<SearchPackages>,
    pub get_package_info: Arc<GetPackageInfo>,
    pub pin: Arc<PinPackage>,
    pub unpin: Arc<UnpinPackage>,
    pub list_services: Arc<ListServices>,
    pub start_service: Arc<StartService>,
    pub stop_service: Arc<StopService>,
    pub restart_service: Arc<RestartService>,
    pub get_service_info: Arc<GetServiceInfo>,
    pub get_service_log: Arc<GetServiceLog>,
    pub export_packages: Arc<ExportPackages>,
    pub import_packages: Arc<ImportPackages>,
    pub bundle_dump: Arc<BundleDump>,
    pub bundle_check_preview: Arc<BundleCheckPreview>,
    pub bundle_apply: Arc<BundleApply>,
    pub record_operation: Arc<RecordOperation>,
    pub load_history: Arc<LoadHistory>,
    pub clear_history: Arc<ClearHistory>,
}

impl UseCaseContainer {
    pub fn new(
        package_repository: Arc<dyn PackageRepository>,
        service_repository: Arc<dyn ServiceRepository>,
        package_list_repository: Arc<dyn PackageListRepository>,
        history_repository: Arc<dyn HistoryRepository>,
    ) -> Self {
        Self {
            list_installed: Arc::new(ListInstalledPackages::new(Arc::clone(&package_repository))),
            list_outdated: Arc::new(ListOutdatedPackages::new(Arc::clone(&package_repository))),
            install: Arc::new(InstallPackage::new(Arc::clone(&package_repository))),
            uninstall: Arc::new(UninstallPackage::new(Arc::clone(&package_repository))),
            update: Arc::new(UpdatePackage::new(Arc::clone(&package_repository))),
            update_all: Arc::new(UpdateAllPackages::new(Arc::clone(&package_repository))),
            clean_cache: Arc::new(CleanCache::new(Arc::clone(&package_repository))),
            cleanup_old_versions: Arc::new(CleanupOldVersions::new(Arc::clone(
                &package_repository,
            ))),
            clean_orphans: Arc::new(CleanOrphans::new(Arc::clone(&package_repository))),
            search: Arc::new(SearchPackages::new(Arc::clone(&package_repository))),
            get_package_info: Arc::new(GetPackageInfo::new(Arc::clone(&package_repository))),
            pin: Arc::new(PinPackage::new(Arc::clone(&package_repository))),
            unpin: Arc::new(UnpinPackage::new(Arc::clone(&package_repository))),
            list_services: Arc::new(ListServices::new(Arc::clone(&service_repository))),
            start_service: Arc::new(StartService::new(Arc::clone(&service_repository))),
            stop_service: Arc::new(StopService::new(Arc::clone(&service_repository))),
            restart_service: Arc::new(RestartService::new(Arc::clone(&service_repository))),
            get_service_info: Arc::new(GetServiceInfo::new(Arc::clone(&service_repository))),
            get_service_log: Arc::new(GetServiceLog::new(Arc::clone(&service_repository))),
            export_packages: Arc::new(ExportPackages::new(Arc::clone(&package_list_repository))),
            import_packages: Arc::new(ImportPackages::new(Arc::clone(&package_list_repository))),
            bundle_dump: Arc::new(BundleDump::new(Arc::clone(&package_list_repository))),
            bundle_check_preview: Arc::new(BundleCheckPreview::new(Arc::clone(
                &package_list_repository,
            ))),
            bundle_apply: Arc::new(BundleApply::new(Arc::clone(&package_list_repository))),
            record_operation: Arc::new(RecordOperation::new(Arc::clone(&history_repository))),
            load_history: Arc::new(LoadHistory::new(Arc::clone(&history_repository))),
            clear_history: Arc::new(ClearHistory::new(Arc::clone(&history_repository))),
        }
    }
}
