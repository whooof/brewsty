use crate::domain::entities::{Service, ServiceStatus};
use egui::{Color32, RichText, ScrollArea};

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
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("service_grid")
                    .striped(true)
                    .spacing([10.0, 8.0])
                    .min_col_width(ui.available_width() / 5.0)
                    .show(ui, |ui| {
                        ui.heading("Name");
                        ui.heading("Status");
                        ui.heading("User");
                        ui.heading("File");
                        ui.heading("Actions");
                        ui.end_row();

                        for service in &self.services {
                            let is_selected = self
                                .selected_service
                                .as_ref()
                                .map_or(false, |s| s == &service.name);

                            if ui.selectable_label(is_selected, &service.name).clicked() {
                                self.selected_service = Some(service.name.clone());
                            }

                            let is_operating = services_loading.contains(&service.name);

                            let status_text = match &service.status {
                                ServiceStatus::Started => {
                                    RichText::new("Running").color(Color32::from_rgb(0, 255, 0))
                                }
                                ServiceStatus::Stopped => {
                                    RichText::new("Stopped").color(Color32::GRAY)
                                }
                                ServiceStatus::Error => {
                                    RichText::new("Error").color(Color32::from_rgb(255, 0, 0))
                                }
                                ServiceStatus::Unknown => {
                                    RichText::new("Unknown").color(Color32::YELLOW)
                                }
                            };

                            if is_operating {
                                ui.spinner();
                            } else {
                                ui.label(status_text);
                            }

                            ui.label(service.user.as_deref().unwrap_or("N/A"));

                            ui.label(service.file.as_deref().unwrap_or("N/A"));

                            ui.add_enabled_ui(!is_operating, |ui| {
                                ui.horizontal(|ui| {
                                    match &service.status {
                                        ServiceStatus::Started => {
                                            if ui.button("Stop").clicked() {
                                                *on_stop = Some(service.name.clone());
                                            }
                                            if ui.button("Restart").clicked() {
                                                *on_restart = Some(service.name.clone());
                                            }
                                        }
                                        ServiceStatus::Stopped | ServiceStatus::Error | ServiceStatus::Unknown => {
                                            if ui.button("Start").clicked() {
                                                *on_start = Some(service.name.clone());
                                            }
                                        }
                                    }
                                });
                            });

                            ui.end_row();
                        }
                    });
            });
    }
}
