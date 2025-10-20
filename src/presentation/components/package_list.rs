use crate::domain::entities::{Package, PackageType};
use egui::{Color32, RichText, ScrollArea};

pub struct PackageList {
    packages: Vec<Package>,
    selected_package: Option<String>,
    show_info_action: Option<Package>,
}

impl PackageList {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            selected_package: None,
            show_info_action: None,
        }
    }

    pub fn update_packages(&mut self, packages: Vec<Package>) {
        self.packages = packages;
    }

    pub fn update_package(&mut self, package: Package) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.name == package.name) {
            *existing = package;
        }
    }

    pub fn get_package(&self, name: &str) -> Option<Package> {
        self.packages.iter().find(|p| p.name == name).cloned()
    }

    pub fn get_show_info_action(&mut self) -> Option<Package> {
        self.show_info_action.take()
    }

    pub fn show_filtered_with_search_and_pin(
        &mut self,
        ui: &mut egui::Ui,
        on_install: &mut Option<Package>,
        on_uninstall: &mut Option<Package>,
        on_update: &mut Option<Package>,
        show_formulae: bool,
        show_casks: bool,
        search_query: &str,
        on_load_info: &mut Option<Package>,
        packages_loading_info: &std::collections::HashSet<String>,
        on_pin: &mut Option<Package>,
        on_unpin: &mut Option<Package>,
    ) {
        let search_lower = search_query.to_lowercase();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("package_grid")
                    .striped(true)
                    .spacing([10.0, 8.0])
                    .min_col_width(ui.available_width() / 5.0)
                    .show(ui, |ui| {
                        ui.heading("Name");
                        ui.heading("Version");
                        ui.heading("Type");
                        ui.heading("Status");
                        ui.heading("Actions");
                        ui.end_row();

                        for package in &self.packages {
                            let should_show = match package.package_type {
                                PackageType::Formula => show_formulae,
                                PackageType::Cask => show_casks,
                            };

                            if !should_show {
                                continue;
                            }

                            if !search_query.is_empty()
                                && !package.name.to_lowercase().contains(&search_lower)
                            {
                                continue;
                            }

                            let is_selected = self
                                .selected_package
                                .as_ref()
                                .map_or(false, |s| s == &package.name);

                            if ui.selectable_label(is_selected, &package.name).clicked() {
                                self.selected_package = Some(package.name.clone());
                            }

                            let version_text = if package.version_load_failed {
                                "Failed".to_string()
                            } else if package.outdated {
                                if let Some(av) = &package.available_version {
                                    format!(
                                        "{} -> {}",
                                        package.version.as_deref().unwrap_or("N/A"),
                                        av
                                    )
                                } else {
                                    package.version.as_deref().unwrap_or("N/A").to_string()
                                }
                            } else {
                                package.version.as_deref().unwrap_or("N/A").to_string()
                            };

                            if packages_loading_info.contains(&package.name) {
                                ui.spinner();
                            } else if package.version_load_failed {
                                ui.label(
                                    RichText::new(version_text).color(Color32::from_rgb(255, 0, 0)),
                                );
                            } else if package.pinned {
                                ui.label(
                                    RichText::new(version_text)
                                        .color(Color32::from_rgb(255, 200, 0)),
                                );
                            } else {
                                ui.label(version_text);
                            }

                            ui.label(package.package_type.to_string());

                            let is_operating = packages_loading_info.contains(&package.name);
                            let status_text = if package.pinned {
                                RichText::new("Pinned").color(Color32::from_rgb(255, 200, 0))
                            } else if package.outdated {
                                RichText::new("Outdated").color(Color32::from_rgb(255, 165, 0))
                            } else if package.installed {
                                RichText::new("Installed").color(Color32::from_rgb(0, 255, 0))
                            } else {
                                RichText::new("Available").color(Color32::GRAY)
                            };

                            if is_operating {
                                ui.spinner();
                            } else {
                                ui.label(status_text);
                            }

                            ui.horizontal(|ui| {
                                if package.installed {
                                    if ui.button("Uninstall").clicked() {
                                        *on_uninstall = Some(package.clone());
                                    }
                                    if package.outdated
                                        && !package.pinned
                                        && ui.button("Update").clicked()
                                    {
                                        *on_update = Some(package.clone());
                                    }
                                    // Only show pin/unpin for formulae (casks don't support pinning in Homebrew)
                                    if matches!(package.package_type, PackageType::Formula) {
                                        if package.pinned {
                                            if ui.button("Unpin").clicked() {
                                                *on_unpin = Some(package.clone());
                                            }
                                        } else {
                                            if ui.button("Pin").clicked() {
                                                *on_pin = Some(package.clone());
                                            }
                                        }
                                    }
                                } else {
                                    if ui.button("Install").clicked() {
                                        *on_install = Some(package.clone());
                                    }
                                }

                                if package.version.is_none()
                                    && !package.version_load_failed
                                    && !packages_loading_info.contains(&package.name)
                                {
                                    if ui.button("Load Info").clicked() {
                                        *on_load_info = Some(package.clone());
                                    }
                                } else if package.description.is_some() {
                                    if ui.button("Info").clicked() {
                                        self.show_info_action = Some(package.clone());
                                    }
                                }
                            });

                            ui.end_row();
                        }
                    });
            });
    }
}
