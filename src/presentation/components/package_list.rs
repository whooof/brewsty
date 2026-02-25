use crate::domain::entities::{Package, PackageType};
use egui::{Color32, RichText};
use egui_extras::{Column, TableBuilder};

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

    #[allow(clippy::too_many_arguments)]
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
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(150.0).resizable(true)) // Name
            .column(Column::initial(150.0).resizable(true)) // Version
            .column(Column::initial(80.0).resizable(true)) // Type
            .column(Column::initial(100.0).resizable(true)) // Status
            .column(Column::remainder().at_least(150.0)) // Actions
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Version");
                });
                header.col(|ui| {
                    ui.strong("Type");
                });
                header.col(|ui| {
                    ui.strong("Status");
                });
                header.col(|ui| {
                    ui.strong("Actions");
                });
            })
            .body(|mut body| {
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

                    let row_height = text_height + 8.0;

                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            let is_selected = self.selected_package.as_ref() == Some(&package.name);

                            if ui.selectable_label(is_selected, &package.name).clicked() {
                                self.selected_package = Some(package.name.clone());
                            }
                        });

                        row.col(|ui| {
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
                        });

                        row.col(|ui| {
                            let type_str = match package.package_type {
                                PackageType::Formula => "📦 Formula",
                                PackageType::Cask => "🖥️ Cask",
                            };
                            ui.label(type_str);
                        });

                        row.col(|ui| {
                            let is_operating = packages_loading_info.contains(&package.name);
                            let status_text = if package.pinned {
                                RichText::new("📌 Pinned").color(Color32::from_rgb(255, 200, 0))
                            } else if package.outdated {
                                RichText::new("⚠️ Outdated").color(Color32::from_rgb(255, 165, 0))
                            } else if package.installed {
                                RichText::new("✅ Installed").color(Color32::from_rgb(0, 255, 0))
                            } else {
                                RichText::new("Available").color(Color32::GRAY)
                            };

                            if is_operating {
                                ui.spinner();
                            } else {
                                ui.label(status_text);
                            }
                        });

                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if package.installed {
                                    if ui.button("🗑️").on_hover_text("Uninstall").clicked() {
                                        *on_uninstall = Some(package.clone());
                                    }
                                    if package.outdated
                                        && !package.pinned
                                        && ui.button("🔄").on_hover_text("Update").clicked()
                                    {
                                        *on_update = Some(package.clone());
                                    }
                                    // Only show pin/unpin for formulae
                                    if matches!(package.package_type, PackageType::Formula) {
                                        if package.pinned {
                                            if ui.button("🔓").on_hover_text("Unpin").clicked() {
                                                *on_unpin = Some(package.clone());
                                            }
                                        } else if ui.button("📌").on_hover_text("Pin").clicked() {
                                            *on_pin = Some(package.clone());
                                        }
                                    }
                                } else if ui.button("⬇️").on_hover_text("Install").clicked() {
                                    *on_install = Some(package.clone());
                                }

                                if package.version.is_none()
                                    && !package.version_load_failed
                                    && !packages_loading_info.contains(&package.name)
                                {
                                    if ui.button("⏳").on_hover_text("Load Info").clicked() {
                                        *on_load_info = Some(package.clone());
                                    }
                                } else if package.description.is_some()
                                    && ui.button("ℹ️").on_hover_text("Info").clicked()
                                {
                                    self.show_info_action = Some(package.clone());
                                }
                            });
                        });
                    });
                }
            });
    }
}
