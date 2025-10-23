use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::SelectionState;
use egui::{Color32, RichText, ScrollArea};

pub struct MergedPackageList {
    packages: Vec<Package>,
    outdated_packages: Vec<Package>,
    selected_package: Option<String>,
    show_info_action: Option<Package>,
    outdated_selection: SelectionState,
}

impl MergedPackageList {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            outdated_packages: Vec::new(),
            selected_package: None,
            show_info_action: None,
            outdated_selection: SelectionState::new(),
        }
    }

    pub fn update_packages(&mut self, packages: Vec<Package>) {
        self.packages = packages;
    }

    pub fn update_outdated_packages(&mut self, packages: Vec<Package>) {
        self.outdated_packages = packages;
    }

    pub fn update_package(&mut self, package: Package) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.name == package.name) {
            *existing = package.clone();
        }
        if let Some(existing) = self
            .outdated_packages
            .iter_mut()
            .find(|p| p.name == package.name)
        {
            *existing = package;
        }
    }

    pub fn get_package(&self, name: &str) -> Option<Package> {
        self.packages
            .iter()
            .chain(self.outdated_packages.iter())
            .find(|p| p.name == name)
            .cloned()
    }

    pub fn mark_package_updated(&mut self, package_name: &str, new_version: String) {
        // Remove from outdated packages
        if let Some(pos) = self
            .outdated_packages
            .iter()
            .position(|p| p.name == package_name)
        {
            let mut package = self.outdated_packages.remove(pos);
            // Update version and clear available_version since it's now current
            package.version = Some(new_version);
            package.available_version = None;
            // Add to installed packages
            if !self.packages.iter().any(|p| p.name == package.name) {
                self.packages.push(package);
            } else if let Some(existing) = self.packages.iter_mut().find(|p| p.name == package.name)
            {
                existing.version = Some(new_version);
                existing.available_version = None;
            }
        }
    }

    pub fn remove_from_outdated_selection_by_name(&mut self, package_name: &str) {
        self.outdated_selection.deselect(package_name);
    }

    pub fn remove_installed_package(&mut self, package_name: &str) {
        if let Some(pos) = self.packages.iter().position(|p| p.name == package_name) {
            self.packages.remove(pos);
        }
    }

    pub fn add_installed_package(&mut self, package: Package) {
        if !self.packages.iter().any(|p| p.name == package.name) {
            self.packages.push(package);
        } else if let Some(existing) = self.packages.iter_mut().find(|p| p.name == package.name) {
            *existing = package;
        }
    }

    pub fn get_show_info_action(&mut self) -> Option<Package> {
        self.show_info_action.take()
    }

    pub fn get_outdated_selection(&self) -> SelectionState {
        self.outdated_selection.clone()
    }

    pub fn set_outdated_selection(&mut self, selection: SelectionState) {
        self.outdated_selection = selection;
    }

    pub fn clear_outdated_selection(&mut self) {
        self.outdated_selection.clear();
    }

    pub fn select_all_outdated(&mut self) {
        for package in &self.outdated_packages {
            self.outdated_selection.select(package.name.clone());
        }
    }

    pub fn deselect_all_outdated(&mut self) {
        self.outdated_selection.clear();
    }

    pub fn has_selected_outdated(&self) -> bool {
        self.outdated_selection.has_selection()
    }

    pub fn get_selected_outdated(&self) -> Vec<String> {
        self.outdated_selection.get_selected()
    }

    pub fn show_merged_with_search_and_pin(
        &mut self,
        ui: &mut egui::Ui,
        on_install: &mut Option<Package>,
        on_uninstall: &mut Option<Package>,
        on_update: &mut Option<Package>,
        on_update_selected: &mut Option<Vec<String>>,
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
                // Outdated Packages Section
                if !self.outdated_packages.is_empty() {
                    ui.heading("⚠️  Outdated Packages");
                    ui.separator();

                    egui::Grid::new("outdated_grid")
                        .striped(true)
                        .spacing([10.0, 8.0])
                        .min_col_width(ui.available_width() / 6.0)
                        .show(ui, |ui| {
                            ui.heading("");
                            ui.heading("Name");
                            ui.heading("Version");
                            ui.heading("Type");
                            ui.heading("Status");
                            ui.heading("Actions");
                            ui.end_row();

                            for package in &self.outdated_packages {
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

                                let mut is_selected =
                                    self.outdated_selection.is_selected(&package.name);
                                if ui.checkbox(&mut is_selected, "").changed() {
                                    if is_selected {
                                        self.outdated_selection.select(package.name.clone());
                                    } else {
                                        self.outdated_selection.deselect(&package.name);
                                    }
                                }

                                ui.label(&package.name);

                                let version_text = if package.version_load_failed {
                                    "Failed".to_string()
                                } else if let Some(av) = &package.available_version {
                                    format!(
                                        "{} -> {}",
                                        package.version.as_deref().unwrap_or("N/A"),
                                        av
                                    )
                                } else {
                                    package.version.as_deref().unwrap_or("N/A").to_string()
                                };

                                if packages_loading_info.contains(&package.name) {
                                    ui.spinner();
                                } else if package.version_load_failed {
                                    ui.label(
                                        RichText::new(version_text)
                                            .color(Color32::from_rgb(255, 0, 0)),
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
                                } else {
                                    RichText::new("Outdated").color(Color32::from_rgb(255, 165, 0))
                                };

                                if is_operating {
                                    ui.spinner();
                                } else {
                                    ui.label(status_text);
                                }

                                ui.horizontal(|ui| {
                                    if !package.pinned && ui.button("Update").clicked() {
                                        *on_update = Some(package.clone());
                                    }
                                    if package.pinned {
                                        if ui.button("Unpin").clicked() {
                                            *on_unpin = Some(package.clone());
                                        }
                                    } else if ui.button("Pin").clicked() {
                                        *on_pin = Some(package.clone());
                                    }

                                    if package.description.is_some() {
                                        if ui.button("Info").clicked() {
                                            self.show_info_action = Some(package.clone());
                                        }
                                    }
                                });

                                ui.end_row();
                            }
                        });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Select All").clicked() {
                            self.select_all_outdated();
                        }
                        if ui.button("Deselect All").clicked() {
                            self.deselect_all_outdated();
                        }
                        if ui
                            .add_enabled(
                                self.outdated_selection.has_selection(),
                                egui::Button::new("Update Selected"),
                            )
                            .clicked()
                        {
                            *on_update_selected = Some(self.outdated_selection.get_selected());
                        }
                    });
                    ui.separator();
                    ui.add_space(16.0);
                }

                // Installed Packages Section
                if !self.packages.is_empty() {
                    ui.heading("📦 Installed Packages");
                    ui.separator();

                    egui::Grid::new("installed_grid")
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

                                let version_text = package.version.as_deref().unwrap_or("N/A");

                                if packages_loading_info.contains(&package.name) {
                                    ui.spinner();
                                } else if package.version_load_failed {
                                    ui.label(
                                        RichText::new(version_text)
                                            .color(Color32::from_rgb(255, 0, 0)),
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
                                } else {
                                    RichText::new("Installed").color(Color32::from_rgb(0, 255, 0))
                                };

                                if is_operating {
                                    ui.spinner();
                                } else {
                                    ui.label(status_text);
                                }

                                ui.horizontal(|ui| {
                                    if ui.button("Uninstall").clicked() {
                                        *on_uninstall = Some(package.clone());
                                    }
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

                                    if package.version.is_none() {
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
                }
            });
    }
}
