//! Brewfile Import/Export Modal

use egui::{RichText, ScrollArea, Ui};

// Legacy types for backward compatibility
#[derive(Default)]
pub struct BrewfileSyncModal {
    pub open: bool,
}

impl BrewfileSyncModal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show_preview(&mut self, _preview: crate::domain::entities::BrewfileSyncPreview) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn render(&mut self, _ctx: &egui::Context) -> Option<BrewfileSyncAction> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum BrewfileSyncAction {
    Apply { install: bool, cleanup: bool },
    Cancel,
}

#[derive(Default)]
pub struct BrewfileModal {
    pub open: bool,
    pub mode: BrewfileModalMode,
    pub preview: Option<String>,
    pub file_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BrewfileModalMode {
    #[default]
    None,
    Export,
    Import,
    Preview,
}

impl BrewfileModal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ctx: &egui::Context) -> BrewfileModalAction {
        let mut action = BrewfileModalAction::None;

        if !self.open || self.mode == BrewfileModalMode::None {
            return action;
        }

        let mut close = false;
        let title = match self.mode {
            BrewfileModalMode::Export => "📤 Export Brewfile",
            BrewfileModalMode::Import => "📥 Import Brewfile",
            BrewfileModalMode::Preview => "👁️ Brewfile Preview",
            _ => "🍺 Brewfile",
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 500.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("✕ Close").clicked() {
                        close = true;
                    }
                });
                ui.separator();

                match self.mode {
                    BrewfileModalMode::Export => self.render_export(ui, &mut action),
                    BrewfileModalMode::Import => self.render_import(ui, &mut action),
                    BrewfileModalMode::Preview => self.render_preview(ui),
                    _ => {}
                }

                if let Some(error) = &self.error {
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::RED, RichText::new(error).strong());
                }
            });

        if close {
            self.close();
        }

        action
    }

    fn render_export(&self, ui: &mut Ui, action: &mut BrewfileModalAction) {
        ui.label("Export your installed packages to a Brewfile.");
        ui.add_space(16.0);

        if let Some(preview) = &self.preview {
            ui.label(RichText::new("Preview:").strong());
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut preview.clone())
                                .desired_rows(10)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
                });

            ui.add_space(16.0);
        }

        ui.horizontal(|ui| {
            if ui.button("💾 Save to File").clicked() {
                *action = BrewfileModalAction::SaveToFile;
            }

            if ui.button("📋 Copy to Clipboard").clicked() {
                *action = BrewfileModalAction::CopyToClipboard;
            }
        });
    }

    fn render_import(&self, ui: &mut Ui, action: &mut BrewfileModalAction) {
        ui.label("Import packages from a Brewfile.");
        ui.add_space(16.0);

        ui.label(
            RichText::new("⚠️ Warning: This will install all packages listed in the Brewfile.")
                .small(),
        );
        ui.add_space(8.0);

        if let Some(path) = &self.file_path {
            ui.label(format!("Selected file: {}", path));
        }

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if ui.button("📂 Select File").clicked() {
                *action = BrewfileModalAction::SelectFile;
            }

            if self.file_path.is_some() && ui.button("✅ Import").clicked() {
                *action = BrewfileModalAction::Import;
            }
        });

        if let Some(preview) = &self.preview {
            ui.add_space(16.0);
            ui.label(RichText::new("Preview:").strong());
            ui.add_space(8.0);

            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                ui.label(preview);
            });
        }
    }

    fn render_preview(&self, ui: &mut Ui) {
        if let Some(preview) = &self.preview {
            ScrollArea::vertical().show(ui, |ui| {
                egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut preview.clone())
                            .desired_rows(20)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
            });
        } else {
            ui.label("No preview available");
        }
    }

    pub fn open_export(&mut self, preview: String) {
        self.mode = BrewfileModalMode::Export;
        self.preview = Some(preview);
        self.open = true;
        self.error = None;
    }

    pub fn open_import(&mut self) {
        self.mode = BrewfileModalMode::Import;
        self.preview = None;
        self.file_path = None;
        self.open = true;
        self.error = None;
    }

    pub fn open_preview(&mut self, preview: String) {
        self.mode = BrewfileModalMode::Preview;
        self.preview = Some(preview);
        self.open = true;
        self.error = None;
    }

    pub fn set_file_path(&mut self, path: String) {
        self.file_path = Some(path);
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn close(&mut self) {
        self.open = false;
        self.mode = BrewfileModalMode::None;
        self.preview = None;
        self.file_path = None;
        self.error = None;
    }
}

#[derive(Debug, Clone)]
pub enum BrewfileModalAction {
    None,
    SaveToFile,
    CopyToClipboard,
    SelectFile,
    Import,
}
