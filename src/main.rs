mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::UseCaseContainer;
use domain::repositories::{PackageListRepository, PackageRepository, ServiceRepository};
use infrastructure::brew::{BrewPackageListRepository, BrewPackageRepository, BrewServiceRepository};
use presentation::services::log_capture;
use presentation::ui::BrewstyApp;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let log_rx = log_capture::init_log_capture();

    let package_repository: Arc<dyn PackageRepository> = Arc::new(BrewPackageRepository::new());
    let service_repository: Arc<dyn ServiceRepository> = Arc::new(BrewServiceRepository::new());
    let package_list_repository: Arc<dyn PackageListRepository> =
        Arc::new(BrewPackageListRepository::new());

    let use_cases = Arc::new(UseCaseContainer::new(
        package_repository,
        service_repository,
        package_list_repository,
    ));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Brewsty - Homebrew Package Manager",
        options,
        Box::new(|_cc| Ok(Box::new(BrewstyApp::new(use_cases, log_rx)))),
    )
}
