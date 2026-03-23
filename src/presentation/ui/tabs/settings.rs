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
    ExportBrewfile,
    SyncBrewfile,
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
        loading_bundle_dump: bool,
        loading_bundle_check: bool,
        doctor_output: &Option<crate::domain::entities::DoctorOutput>,
        taps: &[String],
        new_tap_name: &mut String,
    ) -> Vec<SettingsAction> {
        let mut actions = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Preferences, Diagnostics, and Maintenance");
            ui.separator();

            ui.columns(3, |columns| {
                // Column 1: Preferences & Logs
                columns[0].vertical(|ui| {
                    ui.group(|ui| {
                        ui.heading("Preferences");

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

                        ui.separator();

                        if ui
                            .checkbox(
                                &mut config.search_debounce_enabled,
                                "Debounce Search (As-You-Type)",
                            )
                            .changed()
                        {
                            actions.push(SettingsAction::SaveConfig);
                        }

                        if config.search_debounce_enabled {
                            ui.horizontal(|ui| {
                                ui.label("Delay (ms):");
                                let mut delay = config.search_debounce_delay as f64;
                                if ui
                                    .add(
                                        egui::Slider::new(&mut delay, 500.0..=5000.0)
                                            .step_by(100.0),
                                    )
                                    .changed()
                                {
                                    config.search_debounce_delay = delay as u64;
                                    actions.push(SettingsAction::SaveConfig);
                                }
                            });
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

                // Column 2: Maintenance & Diagnostics
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

                            if ui.button("Cleanup Orphans").clicked() {
                                actions
                                    .push(SettingsAction::ShowCleanupPreview(CleanupType::Orphans));
                            }
                            ui.label("Remove unneeded dependencies");

                            ui.add_space(10.0);

                            let danger_button = egui::Button::new(
                                egui::RichText::new("Update All Packages")
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(183, 28, 28));
                            if ui.add(danger_button).clicked() {
                                actions.push(SettingsAction::UpdateAll);
                            }
                            ui.label("Update all installed packages after confirmation");
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.heading("Brew Doctor");
                        if ui.button("Run Diagnostics").clicked() {
                            actions.push(SettingsAction::RunDoctor);
                        }
                        if let Some(output) = doctor_output {
                            ui.separator();
                            if output.is_ready && output.warnings.is_empty() {
                                ui.label(
                                    egui::RichText::new("✅ Your system is ready to brew.")
                                        .color(egui::Color32::GREEN),
                                );
                            } else {
                                egui::ScrollArea::vertical()
                                    .id_salt("doctor_output")
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        for (i, warning) in output.warnings.iter().enumerate() {
                                            ui.group(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("⚠")
                                                            .color(egui::Color32::YELLOW),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(&warning.title)
                                                            .strong(),
                                                    );
                                                });
                                                if !warning.body.is_empty() {
                                                    egui::CollapsingHeader::new("Details")
                                                        .id_salt(format!("doctor_warning_{}", i))
                                                        .show(ui, |ui| {
                                                            ui.monospace(&warning.body);
                                                        });
                                                }
                                            });
                                        }
                                        if !output.is_ready && output.warnings.is_empty() {
                                            ui.monospace(&output.raw_output);
                                        }
                                    });
                            }
                        }
                    });
                });

                // Column 3: Import/Export, Brewfile, Taps
                columns[2].vertical(|ui| {
                    ui.group(|ui| {
                        ui.heading("Import / Export");
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

                            ui.add_space(15.0);
                            ui.separator();
                            ui.heading("Brewfile");
                            ui.add_space(5.0);

                            if ui
                                .add_enabled(
                                    !loading_bundle_dump,
                                    egui::Button::new("Export to Brewfile"),
                                )
                                .clicked()
                            {
                                actions.push(SettingsAction::ExportBrewfile);
                            }
                            ui.label("Dump current state to Brewfile");

                            ui.add_space(10.0);

                            if ui
                                .add_enabled(
                                    !loading_bundle_check,
                                    egui::Button::new("Sync with Brewfile"),
                                )
                                .clicked()
                            {
                                actions.push(SettingsAction::SyncBrewfile);
                            }
                            ui.label("Compare & sync with a Brewfile");
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.heading("Taps");
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
                                            let untap = egui::Button::new(
                                                egui::RichText::new("Untap")
                                                    .color(egui::Color32::WHITE),
                                            )
                                            .fill(egui::Color32::from_rgb(198, 40, 40));
                                            if ui.add(untap).clicked() {
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
