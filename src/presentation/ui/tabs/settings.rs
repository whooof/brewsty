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
    RunDoctor,
    LoadTaps,
    Tap(String),
    Untap(String),
}

pub struct SettingsTab;

impl SettingsTab {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        log_manager: &mut LogManager,
        loading_export: bool,
        loading_import: bool,
        doctor_output: &Option<String>,
        taps: &[String],
        new_tap_name: &mut String,
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
                                    if ui
                                        .selectable_value(
                                            &mut config.theme,
                                            ThemeMode::System,
                                            "System",
                                        )
                                        .clicked()
                                    {
                                        actions.push(SettingsAction::SaveConfig);
                                        actions.push(SettingsAction::ApplyTheme);
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut config.theme,
                                            ThemeMode::Light,
                                            "Light",
                                        )
                                        .clicked()
                                    {
                                        actions.push(SettingsAction::SaveConfig);
                                        actions.push(SettingsAction::ApplyTheme);
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut config.theme,
                                            ThemeMode::Dark,
                                            "Dark",
                                        )
                                        .clicked()
                                    {
                                        actions.push(SettingsAction::SaveConfig);
                                        actions.push(SettingsAction::ApplyTheme);
                                    }
                                });
                        });

                        if ui
                            .checkbox(&mut config.auto_update_check, "Check updates on startup")
                            .changed()
                        {
                            actions.push(SettingsAction::SaveConfig);
                        }

                        if ui
                            .checkbox(&mut config.confirm_before_actions, "Confirm danger actions")
                            .changed()
                        {
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

                            if debug != log_manager.is_level_visible(LogLevel::Debug) {
                                log_manager.set_level_visible(LogLevel::Debug, debug);
                            }
                            if info != log_manager.is_level_visible(LogLevel::Info) {
                                log_manager.set_level_visible(LogLevel::Info, info);
                            }
                            if warn != log_manager.is_level_visible(LogLevel::Warn) {
                                log_manager.set_level_visible(LogLevel::Warn, warn);
                            }
                            if error != log_manager.is_level_visible(LogLevel::Error) {
                                log_manager.set_level_visible(LogLevel::Error, error);
                            }
                        });
                    });
                });

                // Column 2: Maintenance & Doctor
                columns[1].vertical(|ui| {
                    ui.group(|ui| {
                        ui.heading("Maintenance");
                        ui.vertical_centered(|ui| {
                            if ui.button("Clean Cache").clicked() {
                                actions
                                    .push(SettingsAction::ShowCleanupPreview(CleanupType::Cache));
                            }
                            ui.label("Remove old downloads");

                            ui.add_space(10.0);

                            if ui.button("Cleanup Old Versions").clicked() {
                                actions.push(SettingsAction::ShowCleanupPreview(
                                    CleanupType::OldVersions,
                                ));
                            }
                            ui.label("Remove old versions");

                            ui.add_space(10.0);

                            if ui.button("Update All Packages").clicked() {
                                actions.push(SettingsAction::UpdateAll);
                            }
                            ui.label("Update all installed");
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.heading("🩺 Brew Doctor");
                        if ui.button("Run Diagnostics").clicked() {
                            actions.push(SettingsAction::RunDoctor);
                        }
                        if let Some(output) = doctor_output {
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .id_salt("doctor_output")
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.monospace(output);
                                });
                        }
                    });
                });

                // Column 3: Package Mgmt & Taps
                columns[2].vertical(|ui| {
                    ui.group(|ui| {
                        ui.heading("Management");
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

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.heading("🔌 Taps");
                        ui.horizontal(|ui| {
                            if ui.button("Refresh Taps").clicked() {
                                actions.push(SettingsAction::LoadTaps);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(new_tap_name);
                            if ui.button("Add Tap").clicked() && !new_tap_name.is_empty() {
                                actions.push(SettingsAction::Tap(new_tap_name.clone()));
                                new_tap_name.clear();
                            }
                        });
                        if !taps.is_empty() {
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .id_salt("taps_list")
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    for tap in taps {
                                        ui.horizontal(|ui| {
                                            ui.label(tap);
                                            if ui.small_button("✕").clicked() {
                                                actions.push(SettingsAction::Untap(tap.clone()));
                                            }
                                        });
                                    }
                                });
                        }
                    });
                });
            });
        });

        actions
    }
}
