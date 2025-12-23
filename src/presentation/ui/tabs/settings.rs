use crate::domain::entities::{AppConfig, ThemeMode};
use crate::presentation::components::{CleanupType, LogLevel, LogManager};
use eframe::egui;

pub enum SettingsAction {
    SaveConfig,
    ApplyTheme,
    ShowCleanupPreview(CleanupType),
    UpdateAll,
    ExportPackages,
    ImportPackages,
}

pub struct SettingsTab;

impl SettingsTab {
    pub fn show(
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        log_manager: &mut LogManager,
        loading_export: bool,
        loading_import: bool,
    ) -> Vec<SettingsAction> {
        let mut actions = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Settings & Maintenance");
            ui.separator();

            ui.columns(3, |columns| {
                // Column 1: General & Logs
                columns[0].vertical(|ui| {
                    ui.group(|ui| {
                        ui.heading("General");
                        
                        ui.horizontal(|ui| {
                            ui.label("Theme:");
                            egui::ComboBox::new("theme_combo", "")
                                .selected_text(format!("{:?}", config.theme))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut config.theme, ThemeMode::System, "System").clicked() {
                                        actions.push(SettingsAction::SaveConfig);
                                        actions.push(SettingsAction::ApplyTheme);
                                    }
                                    if ui.selectable_value(&mut config.theme, ThemeMode::Light, "Light").clicked() {
                                        actions.push(SettingsAction::SaveConfig);
                                        actions.push(SettingsAction::ApplyTheme);
                                    }
                                    if ui.selectable_value(&mut config.theme, ThemeMode::Dark, "Dark").clicked() {
                                        actions.push(SettingsAction::SaveConfig);
                                        actions.push(SettingsAction::ApplyTheme);
                                    }
                                });
                        });

                        if ui.checkbox(&mut config.auto_update_check, "Check updates on startup").changed() {
                            actions.push(SettingsAction::SaveConfig);
                        }

                        if ui.checkbox(&mut config.confirm_before_actions, "Confirm danger actions").changed() {
                            actions.push(SettingsAction::SaveConfig);
                        }
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.heading("Log Levels");
                        ui.vertical(|ui| {
                            let mut debug = log_manager.is_level_visible(LogLevel::Debug);
                            let mut info = log_manager.is_level_visible(LogLevel::Info);
                            let mut warn = log_manager.is_level_visible(LogLevel::Warn);
                            let mut error = log_manager.is_level_visible(LogLevel::Error);

                            ui.checkbox(&mut debug, "Debug");
                             ui.checkbox(&mut info, "Info");
                            ui.checkbox(&mut warn, "Warn");
                            ui.checkbox(&mut error, "Error");

                            if debug != log_manager.is_level_visible(LogLevel::Debug) { log_manager.set_level_visible(LogLevel::Debug, debug); }
                            if info != log_manager.is_level_visible(LogLevel::Info) { log_manager.set_level_visible(LogLevel::Info, info); }
                            if warn != log_manager.is_level_visible(LogLevel::Warn) { log_manager.set_level_visible(LogLevel::Warn, warn); }
                            if error != log_manager.is_level_visible(LogLevel::Error) { log_manager.set_level_visible(LogLevel::Error, error); }
                        });
                    });
                });

                // Column 2: Maintenance
                columns[1].vertical(|ui| {
                    ui.heading("Maintenance");
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        if ui.button("Clean Cache").clicked() {
                            actions.push(SettingsAction::ShowCleanupPreview(CleanupType::Cache));
                        }
                        ui.label("Remove old downloads");

                        ui.add_space(10.0);

                        if ui.button("Cleanup Old Versions").clicked() {
                            actions.push(SettingsAction::ShowCleanupPreview(CleanupType::OldVersions));
                        }
                        ui.label("Remove old versions");

                        ui.add_space(10.0);

                        if ui.button("Update All Packages").clicked() {
                            actions.push(SettingsAction::UpdateAll);
                        }
                        ui.label("Update all installed");
                    });
                });

                // Column 3: Package Mgmt
                columns[2].vertical(|ui| {
                    ui.heading("Management");
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        if ui
                            .add_enabled(!loading_export, egui::Button::new("Export Packages"))
                            .clicked()
                        {
                            actions.push(SettingsAction::ExportPackages);
                        }
                        ui.label("Export to JSON");

                        ui.add_space(10.0);

                        if ui
                            .add_enabled(!loading_import, egui::Button::new("Import Packages"))
                            .clicked()
                        {
                            actions.push(SettingsAction::ImportPackages);
                        }
                        ui.label("Import from JSON");
                    });
                });
            });
        });

        actions
    }
}
