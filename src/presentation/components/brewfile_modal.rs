use crate::domain::entities::brewfile::BrewfileSyncPreview;
use eframe::egui;

pub enum BrewfileSyncAction {
    Apply { install: bool, cleanup: bool },
    Cancel,
}

pub struct BrewfileSyncModal {
    preview: Option<BrewfileSyncPreview>,
    show: bool,
    install_missing: bool,
    cleanup_extra: bool,
}

impl BrewfileSyncModal {
    pub fn new() -> Self {
        Self {
            preview: None,
            show: false,
            install_missing: true,
            cleanup_extra: false,
        }
    }

    pub fn show_preview(&mut self, preview: BrewfileSyncPreview) {
        self.preview = Some(preview);
        self.show = true;
        self.install_missing = true;
        self.cleanup_extra = false;
    }

    pub fn close(&mut self) {
        self.show = false;
        self.preview = None;
    }

    pub fn render(&mut self, ctx: &egui::Context) -> Option<BrewfileSyncAction> {
        if !self.show {
            return None;
        }

        let preview = self.preview.as_ref()?;
        let mut action = None;

        let mut is_open = true;
        egui::Window::new("🔄 Sync with Brewfile")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if !preview.has_changes() {
                    ui.label("✅ Your system perfectly matches the Brewfile! No changes needed.");
                    ui.add_space(10.0);
                    if ui.button("Close").clicked() {
                        action = Some(BrewfileSyncAction::Cancel);
                    }
                    return;
                }

                ui.label(format!("Brewfile: {}", preview.brewfile_path));
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        if !preview.missing_dependencies.is_empty() {
                            ui.heading(
                                egui::RichText::new("📦 Missing from System")
                                    .color(egui::Color32::YELLOW),
                            );
                            ui.label(
                                "These packages are in the Brewfile but not currently installed:",
                            );
                            ui.checkbox(&mut self.install_missing, "Install missing dependencies");
                            ui.add_space(5.0);
                            for dep in &preview.missing_dependencies {
                                ui.label(format!("• {}", dep));
                            }
                            ui.add_space(10.0);
                        }

                        if !preview.extra_dependencies.is_empty() {
                            ui.heading(
                                egui::RichText::new("Extra on System")
                                    .color(egui::Color32::LIGHT_RED),
                            );
                            ui.label(
                                "These packages are installed but not listed in the Brewfile:",
                            );
                            ui.checkbox(
                                &mut self.cleanup_extra,
                                "Uninstall extra dependencies (WARNING: Destructive)",
                            );
                            ui.add_space(5.0);
                            for dep in &preview.extra_dependencies {
                                ui.label(format!("• {}", dep));
                            }
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Apply Selected Changes").clicked() {
                        action = Some(BrewfileSyncAction::Apply {
                            install: self.install_missing
                                && !preview.missing_dependencies.is_empty(),
                            cleanup: self.cleanup_extra && !preview.extra_dependencies.is_empty(),
                        });
                    }
                    if ui.button("Cancel").clicked() {
                        action = Some(BrewfileSyncAction::Cancel);
                    }
                });
            });

        if !is_open {
            action = Some(BrewfileSyncAction::Cancel);
        }

        action
    }
}
