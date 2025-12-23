use crate::presentation::components::ServiceList;
use eframe::egui;
use std::collections::HashSet;

pub enum ServiceAction {
    Refresh,
    Start(String),
    Stop(String),
    Restart(String),
}

pub struct ServicesTab;

impl ServicesTab {
    pub fn show(
        ui: &mut egui::Ui,
        service_list: &mut ServiceList,
        services_in_operation: &HashSet<String>,
        loading_services: bool,
    ) -> Vec<ServiceAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            ui.label("Brew Services");
            ui.separator();
            if ui.button("Refresh").clicked() {
                actions.push(ServiceAction::Refresh);
            }
        });

        ui.separator();

        if loading_services {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading services...");
            });
        } else {
            let mut start_action = None;
            let mut stop_action = None;
            let mut restart_action = None;

            service_list.show(
                ui,
                &mut start_action,
                &mut stop_action,
                &mut restart_action,
                services_in_operation,
            );

            if let Some(service_name) = start_action {
                actions.push(ServiceAction::Start(service_name));
            }
            if let Some(service_name) = stop_action {
                actions.push(ServiceAction::Stop(service_name));
            }
            if let Some(service_name) = restart_action {
                actions.push(ServiceAction::Restart(service_name));
            }
        }

        actions
    }
}
