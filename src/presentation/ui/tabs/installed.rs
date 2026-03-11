use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::{
    FilterState, InfoModal, MergedPackageList, SortField, SortOrder,
};
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
            ui.separator();

            let sort_label = |field: SortField, current: SortField, order: SortOrder| -> String {
                let arrow = if field == current {
                    match order {
                        SortOrder::Ascending => " (asc)",
                        SortOrder::Descending => " (desc)",
                    }
                } else {
                    ""
                };
                match field {
                    SortField::Name => format!("Name{}", arrow),
                    SortField::Type => format!("Type{}", arrow),
                    SortField::Size => format!("Size{}", arrow),
                }
            };

            let current_field = filter_state.sort_field();
            let current_order = filter_state.sort_order();
            if ui
                .button(sort_label(SortField::Name, current_field, current_order))
                .clicked()
            {
                filter_state.toggle_sort(SortField::Name);
            }
            if ui
                .button(sort_label(SortField::Type, current_field, current_order))
                .clicked()
            {
                filter_state.toggle_sort(SortField::Type);
            }
            if ui
                .button(sort_label(SortField::Size, current_field, current_order))
                .clicked()
            {
                filter_state.toggle_sort(SortField::Size);
            }
        });

        ui.separator();

        if loading_installed || loading_outdated {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading packages...");
            });
        } else {
            merged_packages.apply_sort(filter_state.sort_field(), filter_state.sort_order());

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
