mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::use_cases::*;
use domain::repositories::PackageRepository;
use infrastructure::brew::BrewPackageRepository;
use presentation::ui::BrustyApp;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let repository: Arc<dyn PackageRepository> = Arc::new(BrewPackageRepository::new());

    let list_installed_use_case = Arc::new(ListInstalledPackages::new(Arc::clone(&repository)));
    let list_outdated_use_case = Arc::new(ListOutdatedPackages::new(Arc::clone(&repository)));
    let install_use_case = Arc::new(InstallPackage::new(Arc::clone(&repository)));
    let uninstall_use_case = Arc::new(UninstallPackage::new(Arc::clone(&repository)));
    let update_use_case = Arc::new(UpdatePackage::new(Arc::clone(&repository)));
    let update_all_use_case = Arc::new(UpdateAllPackages::new(Arc::clone(&repository)));
    let clean_cache_use_case = Arc::new(CleanCache::new(Arc::clone(&repository)));
    let cleanup_old_versions_use_case = Arc::new(CleanupOldVersions::new(Arc::clone(&repository)));
    let search_use_case = Arc::new(SearchPackages::new(Arc::clone(&repository)));
    let get_package_info_use_case = Arc::new(GetPackageInfo::new(Arc::clone(&repository)));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Brusty - Homebrew Package Manager",
        options,
        Box::new(|_cc| {
            Ok(Box::new(BrustyApp::new(
                list_installed_use_case,
                list_outdated_use_case,
                install_use_case,
                uninstall_use_case,
                update_use_case,
                update_all_use_case,
                clean_cache_use_case,
                cleanup_old_versions_use_case,
                search_use_case,
                get_package_info_use_case,
            )))
        }),
    )
}
