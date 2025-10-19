mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::UseCaseContainer;
use domain::repositories::PackageRepository;
use infrastructure::brew::BrewPackageRepository;
use presentation::ui::BrewstyApp;
use presentation::services::log_capture;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let log_rx = log_capture::init_log_capture();

    let repository: Arc<dyn PackageRepository> = Arc::new(BrewPackageRepository::new());
    let use_cases = Arc::new(UseCaseContainer::new(repository));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Brewsty - Homebrew Package Manager",
        options,
        Box::new(|_cc| {
            Ok(Box::new(BrewstyApp::new(use_cases, log_rx)))
        }),
    )
}
