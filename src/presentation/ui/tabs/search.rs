use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::{FilterState, InfoModal, PackageList};
use eframe::egui;
use std::collections::HashSet;

pub enum SearchAction {
    Search,
    Install(Package),
    Uninstall(Package),
    Update(Package),
    LoadInfo(String, PackageType),
    Pin(Package),
    Unpin(Package),
}

pub struct SearchTab;

impl SearchTab {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut egui::Ui,
        search_results: &mut PackageList,
        filter_state: &mut FilterState,
        packages_in_operation: &HashSet<String>,
        loading_search: bool,
        auto_load_version_info: &mut bool,
        info_modal: &mut InfoModal,
    ) -> Vec<SearchAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            ui.label("Search:");
            let response =
                ui.text_edit_singleline(filter_state.search_query_mut());
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                actions.push(SearchAction::Search);
            }
            if ui.button("Search").clicked() {
                actions.push(SearchAction::Search);
            }
        });

        ui.horizontal(|ui| {
            let mut show_formulae = filter_state.show_formulae();
            let mut show_casks = filter_state.show_casks();
            ui.checkbox(&mut show_formulae, "Show Formulae");
            ui.checkbox(&mut show_casks, "Show Casks");
            filter_state.set_show_formulae(show_formulae);
            filter_state.set_show_casks(show_casks);
            ui.separator();
            ui.checkbox(auto_load_version_info, "Auto-load version info");
        });

        ui.separator();

        if loading_search {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Searching...");
            });
        } else {
            let mut install_action = None;
            let mut uninstall_action = None;
            let mut update_action = None;
            let mut load_info_action = None;
            let mut pin_action = None;
            let mut unpin_action = None;

            search_results.show_filtered_with_search_and_pin(
                ui,
                &mut install_action,
                &mut uninstall_action,
                &mut update_action,
                filter_state.show_formulae(),
                filter_state.show_casks(),
                "", // Filter string is empty here as we filter by query logic
                &mut load_info_action,
                packages_in_operation,
                &mut pin_action,
                &mut unpin_action,
            );

            if let Some(package) = install_action {
                actions.push(SearchAction::Install(package));
            }
            if let Some(package) = uninstall_action {
                actions.push(SearchAction::Uninstall(package));
            }
            if let Some(package) = update_action {
                actions.push(SearchAction::Update(package));
            }
            if let Some(package) = load_info_action {
                actions.push(SearchAction::LoadInfo(package.name, package.package_type));
            }
            if let Some(package) = pin_action {
                actions.push(SearchAction::Pin(package));
            }
            if let Some(package) = unpin_action {
                actions.push(SearchAction::Unpin(package));
            }
            if let Some(package) = search_results.get_show_info_action() {
                info_modal.show(package);
            }
        }

        actions
    }
}
