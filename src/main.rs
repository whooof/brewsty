mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::UseCaseContainer;
use domain::repositories::PackageRepository;
use infrastructure::brew::BrewPackageRepository;
use presentation::ui::BrustyApp;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let repository: Arc<dyn PackageRepository> = Arc::new(BrewPackageRepository::new());
    let use_cases = Arc::new(UseCaseContainer::new(repository));

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
            Ok(Box::new(BrustyApp::new(use_cases)))
        }),
    )
}
