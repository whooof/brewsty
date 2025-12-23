use crate::presentation::components::LogManager;
use eframe::egui;

pub enum LogAction {
    CopyAll,
    Clear,
}

pub struct LogTab;

impl LogTab {
    pub fn show(ui: &mut egui::Ui, log_manager: &LogManager) -> Vec<LogAction> {
        let mut actions = Vec::new();

        ui.heading("Command Log");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("📋 Copy All").clicked() {
                actions.push(LogAction::CopyAll);
            }
            if ui.button("🗑 Clear").clicked() {
                actions.push(LogAction::Clear);
            }
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(0, 255, 0));
                let bg_frame = egui::Frame::default()
                    .fill(egui::Color32::BLACK)
                    .inner_margin(8.0);
                bg_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_style({
                        let mut style = (*ui.ctx().style()).clone();
                        style.override_font_id = Some(egui::FontId::monospace(12.0));
                        style
                    });

                    for entry in log_manager.filtered_logs_reversed() {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("[{}]", entry.format_timestamp()))
                                    .color(egui::Color32::GRAY)
                                    .monospace(),
                            );
                            ui.monospace(&entry.message);
                        });
                    }
                });
            });

        actions
    }
}
