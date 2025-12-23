use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::{FilterState, InfoModal, MergedPackageList};
use eframe::egui;
use std::collections::HashSet;

pub enum InstalledAction {
    Refresh,
    Install(Package),
    Uninstall(Package),
    Update(Package),
    UpdateSelected(Vec<String>),
    Pin(Package),
    Unpin(Package),
    LoadInfo(String, PackageType),
}

pub struct InstalledTab;

impl InstalledTab {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut egui::Ui,
        merged_packages: &mut MergedPackageList,
        filter_state: &mut FilterState,
        packages_in_operation: &HashSet<String>,
        loading_installed: bool,
        loading_outdated: bool,
        info_modal: &mut InfoModal,
    ) -> Vec<InstalledAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(filter_state.installed_search_query_mut());
            ui.separator();
            let mut show_formulae = filter_state.show_formulae();
            let mut show_casks = filter_state.show_casks();
            ui.checkbox(&mut show_formulae, "Show Formulae");
            ui.checkbox(&mut show_casks, "Show Casks");
            filter_state.set_show_formulae(show_formulae);
            filter_state.set_show_casks(show_casks);
            ui.separator();
            if ui.button("Refresh").clicked() {
                actions.push(InstalledAction::Refresh);
            }
        });

        ui.separator();

        if loading_installed || loading_outdated {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading packages...");
            });
        } else {
            let mut install_action = None;
            let mut uninstall_action = None;
            let mut update_action = None;
            let mut update_selected_action = None;
            let mut pin_action = None;
            let mut unpin_action = None;
            let mut load_info_action = None;

            merged_packages.show_merged_with_search_and_pin(
                ui,
                &mut install_action,
                &mut uninstall_action,
                &mut update_action,
                &mut update_selected_action,
                filter_state.show_formulae(),
                filter_state.show_casks(),
                filter_state.installed_search_query(),
                &mut load_info_action,
                packages_in_operation,
                &mut pin_action,
                &mut unpin_action,
            );

            if let Some(package) = install_action {
                actions.push(InstalledAction::Install(package));
            }
            if let Some(package) = uninstall_action {
                actions.push(InstalledAction::Uninstall(package));
            }
            if let Some(package) = update_action {
                actions.push(InstalledAction::Update(package));
            }
            if let Some(package_names) = update_selected_action {
                actions.push(InstalledAction::UpdateSelected(package_names));
            }
            if let Some(package) = pin_action {
                actions.push(InstalledAction::Pin(package));
            }
            if let Some(package) = unpin_action {
                actions.push(InstalledAction::Unpin(package));
            }
            if let Some(package) = load_info_action {
                actions.push(InstalledAction::LoadInfo(
                    package.name,
                    package.package_type,
                ));
            }
            if let Some(package) = merged_packages.get_show_info_action() {
                info_modal.show(package);
            }
        }

        actions
    }
}
