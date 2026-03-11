use crate::domain::entities::PackageType;
use crate::domain::entities::history::{OperationHistory, OperationRecord, OperationType};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub enum HistoryAction {
    Undo(UndoRequest),
    ClearHistory,
}

#[derive(Debug, Clone)]
pub struct UndoRequest {
    pub reverse_operation: OperationType,
    pub target: String,
    pub package_type: Option<PackageType>,
}

pub struct HistoryTab;

impl HistoryTab {
    pub fn show(ui: &mut egui::Ui, history: &OperationHistory) -> Vec<HistoryAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            ui.heading("Operation History");
            ui.separator();
            ui.label(format!("{} records", history.records.len()));
            ui.separator();
            if ui.button("\u{1F5D1} Clear History").clicked() {
                actions.push(HistoryAction::ClearHistory);
            }
        });
        ui.add_space(4.0);

        if history.records.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(
                    egui::RichText::new("No operations recorded yet")
                        .size(16.0)
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(8.0);
                ui.label("Install, uninstall, update, or manage packages to see history here.");
            });
            return actions;
        }

        let text_height = ui.text_style_height(&egui::TextStyle::Body);
        let row_height = text_height + 8.0;

        TableBuilder::new(ui)
            .id_salt("history_table")
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(30.0)) // Status icon
            .column(Column::exact(30.0)) // Operation icon
            .column(Column::initial(140.0).at_least(100.0)) // Timestamp
            .column(Column::initial(130.0).at_least(80.0)) // Operation
            .column(Column::initial(200.0).at_least(120.0)) // Target
            .column(Column::initial(70.0).at_least(60.0)) // Type
            .column(Column::remainder().at_least(50.0)) // Undo
            .header(row_height, |mut header| {
                header.col(|ui| {
                    ui.strong("");
                });
                header.col(|ui| {
                    ui.strong("");
                });
                header.col(|ui| {
                    ui.strong("Time");
                });
                header.col(|ui| {
                    ui.strong("Operation");
                });
                header.col(|ui| {
                    ui.strong("Target");
                });
                header.col(|ui| {
                    ui.strong("Type");
                });
                header.col(|ui| {
                    ui.strong("Undo");
                });
            })
            .body(|body| {
                body.rows(row_height, history.records.len(), |mut row| {
                    let idx = row.index();
                    let record = &history.records[idx];

                    row.col(|ui| {
                        ui.label(record.status_icon());
                    });
                    row.col(|ui| {
                        ui.label(record.icon());
                    });
                    row.col(|ui| {
                        ui.label(record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                    });
                    row.col(|ui| {
                        let color = operation_color(record);
                        ui.label(egui::RichText::new(record.operation.to_string()).color(color));
                    });
                    row.col(|ui| {
                        let target_text = record.target.as_deref().unwrap_or("-");
                        let label = if let Some(detail) = &record.detail {
                            format!("{target_text} ({detail})")
                        } else {
                            target_text.to_string()
                        };
                        ui.label(&label).on_hover_text(&label);
                    });
                    row.col(|ui| {
                        if let Some(pt) = &record.package_type {
                            ui.label(pt.to_string());
                        } else {
                            ui.label("-");
                        }
                    });
                    row.col(|ui| {
                        if record.is_undoable() {
                            if ui
                                .button("\u{21A9}")
                                .on_hover_text("Undo this operation")
                                .clicked()
                            {
                                if let Some(reverse) = record.operation.reverse() {
                                    if let Some(target) = &record.target {
                                        actions.push(HistoryAction::Undo(UndoRequest {
                                            reverse_operation: reverse,
                                            target: target.clone(),
                                            package_type: record.package_type,
                                        }));
                                    }
                                }
                            }
                        }
                    });
                });
            });

        actions
    }
}

fn operation_color(record: &OperationRecord) -> egui::Color32 {
    if !record.success {
        return egui::Color32::from_rgb(220, 80, 80); // red for failures
    }
    match record.operation {
        OperationType::Install => egui::Color32::from_rgb(80, 200, 80), // green
        OperationType::Uninstall => egui::Color32::from_rgb(220, 140, 60), // orange
        OperationType::Update | OperationType::UpdateAll => egui::Color32::from_rgb(80, 160, 220), // blue
        OperationType::Pin | OperationType::Unpin => egui::Color32::from_rgb(180, 160, 220), // purple
        OperationType::CleanCache
        | OperationType::CleanupOldVersions
        | OperationType::CleanOrphans => egui::Color32::from_rgb(200, 200, 100), // yellow
        OperationType::BundleApply => egui::Color32::from_rgb(140, 200, 200),                // teal
        OperationType::ServiceStart
        | OperationType::ServiceStop
        | OperationType::ServiceRestart => egui::Color32::from_rgb(180, 180, 180), // gray
    }
}
