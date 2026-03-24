//! Package Details Modal Component

use crate::application::use_cases::PackageDetails;
use egui::{RichText, ScrollArea, Ui};

#[derive(Default)]
pub struct PackageDetailsModal {
    pub open: bool,
    pub details: Option<PackageDetails>,
    pub loading: bool,
    pub pending_package: Option<String>,
}

impl PackageDetailsModal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ctx: &egui::Context) -> PackageDetailsAction {
        let mut action = PackageDetailsAction::None;

        if !self.open {
            return action;
        }

        let mut close = false;

        egui::Window::new("📦 Package Details")
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 600.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("✕ Close").clicked() {
                        close = true;
                    }
                });
                ui.separator();

                if self.loading {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("Loading package details...");
                    });
                    return;
                }

                if let Some(details) = &self.details {
                    self.render_details(ui, details, &mut action);
                } else {
                    ui.label("No package details available");
                }
            });

        if close {
            self.open = false;
            action = PackageDetailsAction::Close;
        }

        action
    }

    fn render_details(
        &self,
        ui: &mut Ui,
        details: &PackageDetails,
        action: &mut PackageDetailsAction,
    ) {
        ScrollArea::vertical().show(ui, |ui| {
            // Package name and version
            ui.heading(RichText::new(&details.name).strong());
            if let Some(version) = &details.version {
                ui.label(format!("Version: {}", version));
            }
            ui.separator();

            // Description
            if let Some(desc) = &details.description {
                ui.label(RichText::new("Description").strong());
                ui.label(desc);
                ui.add_space(8.0);
            }

            // Homepage and repo links
            ui.horizontal(|ui| {
                if let Some(homepage) = &details.homepage {
                    if ui.link("🌐 Homepage").clicked() {
                        if let Err(e) = open::that(homepage) {
                            log::warn!("Failed to open homepage: {}", e);
                        }
                    }
                }

                if let Some(repo) = &details.repo_url {
                    if ui.link("📁 Repository").clicked() {
                        if let Err(e) = open::that(repo) {
                            log::warn!("Failed to open repo: {}", e);
                        }
                    }
                }
            });
            ui.add_space(8.0);

            // License
            if let Some(license) = &details.license {
                ui.label(format!("📄 License: {}", license));
                ui.add_space(8.0);
            }

            // Dependencies
            if !details.dependencies.is_empty() {
                ui.label(RichText::new("Dependencies").strong());
                ui.horizontal_wrapped(|ui| {
                    for dep in &details.dependencies {
                        ui.label(format!("• {}", dep));
                    }
                });
                ui.add_space(8.0);
            }

            // Build dependencies
            if !details.build_dependencies.is_empty() {
                ui.label(RichText::new("Build Dependencies").strong());
                ui.horizontal_wrapped(|ui| {
                    for dep in &details.build_dependencies {
                        ui.label(format!("• {}", dep));
                    }
                });
                ui.add_space(8.0);
            }

            // Required by (reverse dependencies)
            if !details.required_by.is_empty() {
                ui.label(RichText::new("Required by").strong());
                ui.horizontal_wrapped(|ui| {
                    for pkg in &details.required_by {
                        ui.label(format!("• {}", pkg));
                    }
                });
                ui.add_space(8.0);
            }

            // Caveats
            if let Some(caveats) = &details.caveats {
                if !caveats.trim().is_empty() {
                    ui.label(RichText::new("⚠️ Caveats").strong());
                    ui.group(|ui| {
                        ui.label(caveats);
                    });
                }
            }

            ui.add_space(16.0);

            // Action buttons
            ui.horizontal(|ui| {
                if details.installed {
                    if ui.button("🗑️ Uninstall").clicked() {
                        *action = PackageDetailsAction::Uninstall(details.name.clone());
                    }
                } else {
                    if ui.button("📥 Install").clicked() {
                        *action = PackageDetailsAction::Install(details.name.clone());
                    }
                }

                if ui.button("Close").clicked() {
                    *action = PackageDetailsAction::Close;
                }
            });
        });
    }

    pub fn set_details(&mut self, details: PackageDetails) {
        self.details = Some(details);
        self.loading = false;
        self.open = true;
    }

    pub fn set_loading(&mut self) {
        self.loading = true;
        self.details = None;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.details = None;
        self.loading = false;
    }

    pub fn open_for_package(&mut self, package: &crate::domain::entities::Package) {
        self.set_loading();
        self.pending_package = Some(package.name.clone());
    }

    pub fn open_for_package_name(&mut self, package_name: String) {
        self.set_loading();
        self.pending_package = Some(package_name);
    }

    pub fn set_pending_package(&mut self, name: String) {
        self.pending_package = Some(name);
    }

    pub fn take_pending_package(&mut self) -> Option<String> {
        self.pending_package.take()
    }
}

#[derive(Debug, Clone)]
pub enum PackageDetailsAction {
    None,
    Close,
    Install(String),
    Uninstall(String),
}
