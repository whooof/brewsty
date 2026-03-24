use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::SelectionState;
use crate::presentation::components::filter_state::{SortField, SortOrder};
use egui::{Color32, RichText, ScrollArea};
use egui_extras::{Column, TableBuilder};

const TABLE_CELL_H_PADDING: f32 = 8.0;
const TABLE_ROW_PADDING: f32 = 14.0;
const TABLE_HEADER_HEIGHT: f32 = 28.0;

fn padded_cell(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(TABLE_CELL_H_PADDING);
    add_contents(ui);
    ui.add_space(TABLE_CELL_H_PADDING);
}

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

    pub fn get_all_packages(&self) -> &[Package] {
        &self.packages
    }

    pub fn mark_package_updated(&mut self, package_name: &str) {
        if let Some(pos) = self
            .outdated_packages
            .iter()
            .position(|p| p.name == package_name)
        {
            self.outdated_packages.remove(pos);
        }

        if let Some(installed) = self.packages.iter_mut().find(|p| p.name == package_name) {
            installed.available_version = None;
        }
    }

    pub fn remove_from_outdated_selection_by_name(&mut self, package_name: &str) {
        self.outdated_selection.deselect(package_name);
    }

    pub fn remove_installed_package(&mut self, package_name: &str) {
        if let Some(pos) = self.packages.iter().position(|p| p.name == package_name) {
            self.packages.remove(pos);
        }
        if let Some(pos) = self
            .outdated_packages
            .iter()
            .position(|p| p.name == package_name)
        {
            self.outdated_packages.remove(pos);
        }
    }

    pub fn get_show_info_action(&mut self) -> Option<Package> {
        self.show_info_action.take()
    }

    pub fn clear_outdated_selection(&mut self) {
        self.outdated_selection.clear();
    }

    pub fn installed_count(&self) -> usize {
        self.packages.len()
    }

    pub fn apply_sort(&mut self, field: SortField, order: SortOrder) {
        let cmp = |a: &Package, b: &Package| -> std::cmp::Ordering {
            let result = match field {
                SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortField::Type => a.package_type.to_string().cmp(&b.package_type.to_string()),
                SortField::Size => a.installed_size.cmp(&b.installed_size),
            };
            match order {
                SortOrder::Ascending => result,
                SortOrder::Descending => result.reverse(),
            }
        };
        self.packages.sort_by(cmp);
        self.outdated_packages.sort_by(cmp);
    }

    pub fn select_all_outdated(&mut self) {
        for package in &self.outdated_packages {
            self.outdated_selection.select(package.name.clone());
        }
    }

    pub fn deselect_all_outdated(&mut self) {
        self.outdated_selection.clear();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show_merged_with_search_and_pin(
        &mut self,
        ui: &mut egui::Ui,
        _on_install: &mut Option<Package>,
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
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut visible_outdated = 0usize;
                let mut visible_installed = 0usize;
                // Outdated Packages Section
                if !self.outdated_packages.is_empty() {
                    ui.heading("⚠ Outdated Packages");
                    ui.separator();

                    TableBuilder::new(ui)
                        .id_salt("outdated_packages_table")
                        .striped(true)
                        .resizable(true)
                        .vscroll(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(180.0).resizable(true)) // Select + Name
                        .column(Column::initial(150.0).resizable(true)) // Version
                        .column(Column::initial(80.0).resizable(true)) // Type
                        .column(Column::initial(80.0).resizable(true)) // Size
                        .column(Column::initial(100.0).resizable(true)) // Status
                        .column(Column::remainder().at_least(120.0)) // Actions
                        .header(TABLE_HEADER_HEIGHT, |mut header| {
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Name");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Version");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Type");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Size");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Status");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Actions");
                                })
                            });
                        })
                        .body(|mut body| {
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

                                visible_outdated += 1;

                                let row_height = text_height + TABLE_ROW_PADDING;

                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let was_selected =
                                                self.outdated_selection.is_selected(&package.name);
                                            let mut is_selected = was_selected;

                                            let mut should_toggle_from_label = false;
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 4.0;
                                                ui.checkbox(&mut is_selected, "");
                                                if ui
                                                    .selectable_label(is_selected, &package.name)
                                                    .clicked()
                                                {
                                                    should_toggle_from_label = true;
                                                }
                                            });

                                            if should_toggle_from_label {
                                                is_selected = !was_selected;
                                            }

                                            if is_selected != was_selected {
                                                if is_selected {
                                                    self.outdated_selection
                                                        .select(package.name.clone());
                                                } else {
                                                    self.outdated_selection.deselect(&package.name);
                                                }
                                            }
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let version_text = if package.version_load_failed {
                                                "Failed".to_string()
                                            } else if let Some(av) = &package.available_version {
                                                format!(
                                                    "{} -> {}",
                                                    package.version.as_deref().unwrap_or("N/A"),
                                                    av
                                                )
                                            } else {
                                                package
                                                    .version
                                                    .as_deref()
                                                    .unwrap_or("N/A")
                                                    .to_string()
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
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let type_str = match package.package_type {
                                                PackageType::Formula => "📦 Formula",
                                                PackageType::Cask => "Cask",
                                            };
                                            ui.label(type_str);
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            if let Some(size) = package.installed_size {
                                                ui.label(
                                                    crate::presentation::ui::app::format_size(size),
                                                );
                                            } else {
                                                ui.label("-");
                                            }
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let is_operating =
                                                packages_loading_info.contains(&package.name);
                                            let status_text = if package.pinned {
                                                RichText::new("📌 Pinned")
                                                    .color(Color32::from_rgb(255, 200, 0))
                                            } else {
                                                RichText::new("⚠ Outdated")
                                                    .color(Color32::from_rgb(255, 165, 0))
                                            };

                                            if is_operating {
                                                ui.spinner();
                                            } else {
                                                ui.label(status_text);
                                            }
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            ui.menu_button("Actions", |ui| {
                                                if !package.pinned && ui.button("Update").clicked()
                                                {
                                                    *on_update = Some(package.clone());
                                                    ui.close();
                                                }
                                                if package.pinned {
                                                    if ui.button("Unpin").clicked() {
                                                        *on_unpin = Some(package.clone());
                                                        ui.close();
                                                    }
                                                } else if ui.button("Pin").clicked() {
                                                    *on_pin = Some(package.clone());
                                                    ui.close();
                                                }

                                                if package.description.is_some()
                                                    && ui.button("Info").clicked()
                                                {
                                                    self.show_info_action = Some(package.clone());
                                                    ui.close();
                                                }
                                            });
                                        })
                                    });
                                });
                            }
                        });

                    if visible_outdated == 0 {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("No outdated packages match the current filters.")
                                .italics()
                                .color(Color32::GRAY),
                        );
                    }

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
                                egui::Button::new("🚀 Update Selected"),
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
                    ui.heading("✅ Installed Packages");
                    ui.separator();

                    TableBuilder::new(ui)
                        .id_salt("installed_packages_table")
                        .striped(true)
                        .resizable(true)
                        .vscroll(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(150.0).resizable(true)) // Name
                        .column(Column::initial(150.0).resizable(true)) // Version
                        .column(Column::initial(80.0).resizable(true)) // Type
                        .column(Column::initial(80.0).resizable(true)) // Size
                        .column(Column::initial(100.0).resizable(true)) // Status
                        .column(Column::remainder().at_least(120.0)) // Actions
                        .header(TABLE_HEADER_HEIGHT, |mut header| {
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Name");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Version");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Type");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Size");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Status");
                                })
                            });
                            header.col(|ui| {
                                padded_cell(ui, |ui| {
                                    ui.strong("Actions");
                                })
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

                                visible_installed += 1;

                                let row_height = text_height + TABLE_ROW_PADDING;

                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let is_selected = self.selected_package.as_ref()
                                                == Some(&package.name);

                                            if ui
                                                .selectable_label(is_selected, &package.name)
                                                .clicked()
                                            {
                                                self.selected_package = Some(package.name.clone());
                                            }
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let version_text =
                                                package.version.as_deref().unwrap_or("N/A");

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
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let type_str = match package.package_type {
                                                PackageType::Formula => "📦 Formula",
                                                PackageType::Cask => "Cask",
                                            };
                                            ui.label(type_str);
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            if let Some(size) = package.installed_size {
                                                ui.label(
                                                    crate::presentation::ui::app::format_size(size),
                                                );
                                            } else {
                                                ui.label("-");
                                            }
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            let is_operating =
                                                packages_loading_info.contains(&package.name);
                                            let status_text = if package.pinned {
                                                RichText::new("📌 Pinned")
                                                    .color(Color32::from_rgb(255, 200, 0))
                                            } else {
                                                RichText::new("Installed")
                                                    .color(Color32::from_rgb(0, 255, 0))
                                            };

                                            if is_operating {
                                                ui.spinner();
                                            } else {
                                                ui.label(status_text);
                                            }
                                        })
                                    });

                                    row.col(|ui| {
                                        padded_cell(ui, |ui| {
                                            ui.menu_button("Actions", |ui| {
                                                if ui.button("Uninstall").clicked() {
                                                    *on_uninstall = Some(package.clone());
                                                    ui.close();
                                                }
                                                if matches!(
                                                    package.package_type,
                                                    PackageType::Formula
                                                ) {
                                                    if package.pinned {
                                                        if ui.button("Unpin").clicked() {
                                                            *on_unpin = Some(package.clone());
                                                            ui.close();
                                                        }
                                                    } else if ui.button("Pin").clicked() {
                                                        *on_pin = Some(package.clone());
                                                        ui.close();
                                                    }
                                                }

                                                if package.version.is_none() {
                                                    if ui.button("Load Info").clicked() {
                                                        *on_load_info = Some(package.clone());
                                                        ui.close();
                                                    }
                                                } else if package.description.is_some()
                                                    && ui.button("Info").clicked()
                                                {
                                                    self.show_info_action = Some(package.clone());
                                                    ui.close();
                                                }
                                            });
                                        })
                                    });
                                });
                            }
                        });

                    if visible_installed == 0 {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("No installed packages match the current filters.")
                                .italics()
                                .color(Color32::GRAY),
                        );
                    }
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_sort_sorts_outdated_packages_too() {
        let mut list = MergedPackageList::new();
        list.update_packages(vec![
            Package::new("zeta".to_string(), PackageType::Formula),
            Package::new("alpha".to_string(), PackageType::Formula),
        ]);
        list.update_outdated_packages(vec![
            Package::new("zulu".to_string(), PackageType::Formula),
            Package::new("beta".to_string(), PackageType::Formula),
        ]);

        list.apply_sort(SortField::Name, SortOrder::Ascending);

        assert_eq!(list.packages[0].name, "alpha");
        assert_eq!(list.outdated_packages[0].name, "beta");
    }
}
