use crate::domain::entities::CleanupPreview;
use crate::presentation::utils::format_size;

#[derive(PartialEq, Clone, Copy)]
pub enum CleanupType {
    Cache,
    OldVersions,
    Orphans,
}

pub enum CleanupAction {
    Confirm(CleanupType),
    Cancel,
}

pub struct CleanupModal {
    show: bool,
    cleanup_type: Option<CleanupType>,
    preview: Option<CleanupPreview>,
}

impl CleanupModal {
    pub fn new() -> Self {
        Self {
            show: false,
            cleanup_type: None,
            preview: None,
        }
    }

    pub fn show_preview(&mut self, cleanup_type: CleanupType, preview: CleanupPreview) {
        self.cleanup_type = Some(cleanup_type);
        self.preview = Some(preview);
        self.show = true;
    }

    pub fn close(&mut self) {
        self.show = false;
        self.cleanup_type = None;
        self.preview = None;
    }

    pub fn render(&mut self, ctx: &egui::Context) -> Option<CleanupAction> {
        if !self.show {
            return None;
        }

        let mut action = None;

        egui::Window::new("Cleanup Preview")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(preview) = &self.preview {
                    ui.heading(format!(
                        "Total size to free: {}",
                        format_size(preview.total_size)
                    ));
                    ui.separator();

                    ui.label(format!(
                        "Files and folders to be removed ({} items):",
                        preview.items.len()
                    ));

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for item in &preview.items {
                                ui.horizontal(|ui| {
                                    ui.label(&item.path);
                                    ui.label(format!("({})", format_size(item.size)));
                                });
                            }
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Confirm").clicked() {
                            if let Some(cleanup_type) = &self.cleanup_type {
                                action = Some(CleanupAction::Confirm(*cleanup_type));
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            action = Some(CleanupAction::Cancel);
                        }
                    });
                }
            });

        action
    }
}

impl Default for CleanupModal {
    fn default() -> Self {
        Self::new()
    }
}
