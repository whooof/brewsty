use crate::domain::entities::{Service, ServiceInfo, ServiceStatus};
use egui::{Color32, RichText};
use egui_extras::{Column, TableBuilder};

pub struct ServiceList {
    services: Vec<Service>,
    selected_service: Option<String>,
    service_detail_modal: Option<ServiceDetailModal>,
}

/// Modal showing detailed service info and/or log content.
pub struct ServiceDetailModal {
    pub service_name: String,
    pub info: Option<ServiceInfo>,
    pub log_content: Option<String>,
    pub loading_info: bool,
    pub loading_log: bool,
    pub error: Option<String>,
}

impl ServiceDetailModal {
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            info: None,
            log_content: None,
            loading_info: false,
            loading_log: false,
            error: None,
        }
    }
}

impl ServiceList {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            selected_service: None,
            service_detail_modal: None,
        }
    }

    pub fn update_services(&mut self, services: Vec<Service>) {
        self.services = services;
    }

    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    pub fn show_info_modal(&mut self, service_name: String) {
        let mut modal = ServiceDetailModal::new(service_name);
        modal.loading_info = true;
        self.service_detail_modal = Some(modal);
    }

    pub fn show_log_modal(&mut self, service_name: String) {
        let mut modal = if let Some(existing) = self.service_detail_modal.take() {
            if existing.service_name == service_name {
                existing
            } else {
                ServiceDetailModal::new(service_name)
            }
        } else {
            ServiceDetailModal::new(service_name)
        };
        modal.error = None;
        modal.loading_log = true;
        self.service_detail_modal = Some(modal);
    }

    pub fn set_service_info(&mut self, name: &str, info: ServiceInfo) {
        if let Some(modal) = &mut self.service_detail_modal {
            if modal.service_name == name {
                modal.info = Some(info);
                modal.loading_info = false;
            }
        }
    }

    pub fn set_service_info_error(&mut self, name: &str, error: String) {
        if let Some(modal) = &mut self.service_detail_modal {
            if modal.service_name == name {
                modal.error = Some(error);
                modal.loading_info = false;
            }
        }
    }

    pub fn set_service_log(&mut self, name: &str, log_content: String) {
        if let Some(modal) = &mut self.service_detail_modal {
            if modal.service_name == name {
                modal.log_content = Some(log_content);
                modal.loading_log = false;
            }
        }
    }

    pub fn set_service_log_error(&mut self, name: &str, error: String) {
        if let Some(modal) = &mut self.service_detail_modal {
            if modal.service_name == name {
                modal.error = Some(error);
                modal.loading_log = false;
            }
        }
    }

    /// Render the detail modal if open. Returns true if it was closed.
    pub fn render_detail_modal(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut log_request = None;

        let should_close = if let Some(modal) = &self.service_detail_modal {
            let mut close = false;
            let title = format!("Service: {}", modal.service_name);

            egui::Window::new(title)
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .default_height(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(error) = &modal.error {
                        ui.colored_label(Color32::from_rgb(255, 100, 100), error);
                        ui.separator();
                    }

                    if modal.loading_info {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading service info...");
                        });
                    } else if let Some(info) = &modal.info {
                        egui::Grid::new("service_info_grid")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.strong("Service Name:");
                                ui.label(&info.service_name);
                                ui.end_row();

                                ui.strong("Status:");
                                let status_text =
                                    match &info.status {
                                        ServiceStatus::Started => RichText::new("Running")
                                            .color(Color32::from_rgb(0, 255, 0)),
                                        ServiceStatus::Stopped => {
                                            RichText::new("Stopped").color(Color32::GRAY)
                                        }
                                        ServiceStatus::Error => RichText::new("Error")
                                            .color(Color32::from_rgb(255, 0, 0)),
                                        ServiceStatus::Unknown => {
                                            RichText::new("Unknown").color(Color32::YELLOW)
                                        }
                                    };
                                ui.label(status_text);
                                ui.end_row();

                                ui.strong("PID:");
                                ui.label(
                                    info.pid
                                        .map(|p| p.to_string())
                                        .unwrap_or_else(|| "N/A".to_string()),
                                );
                                ui.end_row();

                                ui.strong("Exit Code:");
                                ui.label(
                                    info.exit_code
                                        .map(|c| c.to_string())
                                        .unwrap_or_else(|| "N/A".to_string()),
                                );
                                ui.end_row();

                                ui.strong("User:");
                                ui.label(info.user.as_deref().unwrap_or("N/A"));
                                ui.end_row();

                                ui.strong("Boot:");
                                let boot_label = if info.registered {
                                    "Login (auto-start)"
                                } else if info.running {
                                    "Manual (run)"
                                } else {
                                    "None"
                                };
                                ui.label(boot_label);
                                ui.end_row();

                                ui.strong("Loaded:");
                                ui.label(if info.loaded { "Yes" } else { "No" });
                                ui.end_row();

                                if let Some(cmd) = &info.command {
                                    ui.strong("Command:");
                                    ui.label(cmd);
                                    ui.end_row();
                                }

                                if let Some(file) = &info.file {
                                    ui.strong("Plist File:");
                                    ui.label(file);
                                    ui.end_row();
                                }

                                if let Some(log) = &info.log_path {
                                    ui.strong("Log Path:");
                                    ui.label(log);
                                    ui.end_row();
                                }

                                if let Some(err_log) = &info.error_log_path {
                                    ui.strong("Error Log:");
                                    ui.label(err_log);
                                    ui.end_row();
                                }
                            });
                    }

                    ui.separator();

                    // Log section
                    if modal.loading_log {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading log...");
                        });
                    } else if let Some(log_text) = &modal.log_content {
                        ui.strong("Log Output (last 100 lines):");
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.monospace(log_text);
                            });
                    } else if modal.info.is_some() {
                        // Info loaded but no log yet - offer to load it
                        let has_log = modal
                            .info
                            .as_ref()
                            .map(|i| i.log_path.is_some() || i.error_log_path.is_some())
                            .unwrap_or(false);
                        if has_log {
                            if ui.button("Load Log").clicked() {
                                log_request = Some(modal.service_name.clone());
                            }
                        }
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            close = true;
                        }
                    });
                });
            close
        } else {
            false
        };

        if should_close {
            self.service_detail_modal = None;
        }

        log_request
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        on_start: &mut Option<String>,
        on_stop: &mut Option<String>,
        on_restart: &mut Option<String>,
        on_info: &mut Option<String>,
        on_log: &mut Option<String>,
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
            .column(Column::initial(70.0).resizable(true)) // Boot
            .column(Column::initial(80.0).resizable(true)) // User
            .column(Column::remainder().at_least(120.0)) // File
            .column(Column::initial(190.0)) // Actions
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Status");
                });
                header.col(|ui| {
                    ui.strong("Boot");
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
                        });

                        row.col(|ui| {
                            let label = service.boot_label();
                            let text = match label {
                                "Login" => {
                                    RichText::new("Login").color(Color32::from_rgb(100, 180, 255))
                                }
                                "Manual" => RichText::new("Manual").color(Color32::LIGHT_GRAY),
                                _ => RichText::new("None").color(Color32::DARK_GRAY),
                            };
                            ui.label(text);
                        });

                        row.col(|ui| {
                            ui.label(service.user.as_deref().unwrap_or("N/A"));
                        });

                        row.col(|ui| {
                            ui.label(service.file.as_deref().unwrap_or("N/A"));
                        });

                        row.col(|ui| {
                            ui.add_enabled_ui(!is_operating, |ui| {
                                ui.horizontal(|ui| {
                                    match &service.status {
                                        ServiceStatus::Started => {
                                            if ui
                                                .button("Stop")
                                                .on_hover_text("Stop service")
                                                .clicked()
                                            {
                                                *on_stop = Some(service.name.clone());
                                            }
                                            if ui
                                                .button("Restart")
                                                .on_hover_text("Restart service")
                                                .clicked()
                                            {
                                                *on_restart = Some(service.name.clone());
                                            }
                                        }
                                        ServiceStatus::Stopped
                                        | ServiceStatus::Error
                                        | ServiceStatus::Unknown => {
                                            if ui
                                                .button("Start")
                                                .on_hover_text("Start service")
                                                .clicked()
                                            {
                                                *on_start = Some(service.name.clone());
                                            }
                                        }
                                    }
                                    if ui
                                        .button("Info")
                                        .on_hover_text("View service details")
                                        .clicked()
                                    {
                                        *on_info = Some(service.name.clone());
                                    }
                                    if ui.button("Log").on_hover_text("View service log").clicked()
                                    {
                                        *on_log = Some(service.name.clone());
                                    }
                                });
                            });
                        });
                    });
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_log_modal_keeps_existing_modal_for_same_service() {
        let mut list = ServiceList::new();
        list.show_info_modal("postgresql".to_string());
        list.set_service_info(
            "postgresql",
            ServiceInfo {
                name: "postgresql".to_string(),
                service_name: "postgresql".to_string(),
                status: ServiceStatus::Started,
                pid: Some(123),
                exit_code: None,
                user: Some("me".to_string()),
                file: None,
                loaded: true,
                registered: true,
                running: true,
                command: None,
                log_path: Some("/tmp/postgres.log".to_string()),
                error_log_path: None,
            },
        );

        list.show_log_modal("postgresql".to_string());

        let modal = list.service_detail_modal.as_ref().unwrap();
        assert_eq!(modal.service_name, "postgresql");
        assert!(modal.info.is_some());
        assert!(modal.loading_log);
    }

    #[test]
    fn show_log_modal_replaces_modal_for_different_service() {
        let mut list = ServiceList::new();
        list.show_info_modal("redis".to_string());
        list.show_log_modal("nginx".to_string());

        let modal = list.service_detail_modal.as_ref().unwrap();
        assert_eq!(modal.service_name, "nginx");
        assert!(modal.info.is_none());
        assert!(modal.loading_log);
    }
}
