use crate::domain::entities::{Service, ServiceStatus};
use egui::{Color32, RichText};
use egui_extras::{Column, TableBuilder};

pub struct ServiceList {
    services: Vec<Service>,
    selected_service: Option<String>,
}

#[allow(dead_code)]
impl ServiceList {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            selected_service: None,
        }
    }

    pub fn update_services(&mut self, services: Vec<Service>) {
        self.services = services;
    }

    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    pub fn update_service(&mut self, service: Service) {
        if let Some(existing) = self.services.iter_mut().find(|s| s.name == service.name) {
            *existing = service;
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        on_start: &mut Option<String>,
        on_stop: &mut Option<String>,
        on_restart: &mut Option<String>,
        services_loading: &std::collections::HashSet<String>,
    ) {
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(100.0).resizable(true)) // Name
            .column(Column::initial(100.0).resizable(true)) // Status
            .column(Column::initial(100.0).resizable(true)) // User
            .column(Column::remainder().at_least(150.0)) // File
            .column(Column::initial(100.0)) // Actions
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Status");
                });
                header.col(|ui| {
                    ui.strong("User");
                });
                header.col(|ui| {
                    ui.strong("File");
                });
                header.col(|ui| {
                    ui.strong("Actions");
                });
            })
            .body(|mut body| {
                for service in &self.services {
                    let is_operating = services_loading.contains(&service.name);
                    let row_height = text_height + 8.0;

                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            let is_selected = self.selected_service.as_ref() == Some(&service.name);
                            if ui.selectable_label(is_selected, &service.name).clicked() {
                                self.selected_service = Some(service.name.clone());
                            }
                        });

                        row.col(|ui| {
                            let status_text =
                                match &service.status {
                                    ServiceStatus::Started => RichText::new("🟢 Running")
                                        .color(Color32::from_rgb(0, 255, 0)),
                                    ServiceStatus::Stopped => {
                                        RichText::new("⚪ Stopped").color(Color32::GRAY)
                                    }
                                    ServiceStatus::Error => RichText::new("🔴 Error")
                                        .color(Color32::from_rgb(255, 0, 0)),
                                    ServiceStatus::Unknown => {
                                        RichText::new("🟡 Unknown").color(Color32::YELLOW)
                                    }
                                };

                            if is_operating {
                                ui.spinner();
                            } else {
                                ui.label(status_text);
                            }
                        });

                        row.col(|ui| {
                            ui.label(service.user.as_deref().unwrap_or("N/A"));
                        });

                        row.col(|ui| {
                            ui.label(service.file.as_deref().unwrap_or("N/A"));
                        });

                        row.col(|ui| {
                            ui.add_enabled_ui(!is_operating, |ui| {
                                ui.horizontal(|ui| match &service.status {
                                    ServiceStatus::Started => {
                                        if ui.button("⏹️").on_hover_text("Stop").clicked() {
                                            *on_stop = Some(service.name.clone());
                                        }
                                        if ui.button("🔄").on_hover_text("Restart").clicked() {
                                            *on_restart = Some(service.name.clone());
                                        }
                                    }
                                    ServiceStatus::Stopped
                                    | ServiceStatus::Error
                                    | ServiceStatus::Unknown => {
                                        if ui.button("▶️").on_hover_text("Start").clicked() {
                                            *on_start = Some(service.name.clone());
                                        }
                                    }
                                });
                            });
                        });
                    });
                }
            });
    }
}
