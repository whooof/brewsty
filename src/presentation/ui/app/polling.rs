use crate::domain::entities::OperationType;
use crate::presentation::components::Tab;

use super::format_size;
use super::{BrewstyApp, PendingOperation};

impl BrewstyApp {
    pub(super) fn poll_async_tasks(&mut self) {
        tracing::trace!("poll_async_tasks called, checking for active task");
        let result = self.task_manager.poll();
        let mut installed_refresh_finished = false;
        let mut installed_refresh_succeeded = false;
        let mut services_refresh_finished = false;
        let mut services_refresh_succeeded = false;

        if let Some((packages, logs)) = result.installed_packages {
            tracing::info!("Got {} installed packages from poll", packages.len());
            self.loading_installed = false;
            installed_refresh_finished = true;

            if is_terminal_load_error(&packages, &logs, "Error loading installed") {
                self.status_message = "Failed to load installed packages".to_string();
                self.toast_manager
                    .error("Failed to load installed packages");
            } else {
                self.merged_packages.update_packages(packages);
                installed_refresh_succeeded = true;
            }
        }

        if let Some((packages, logs)) = result.outdated_packages {
            tracing::info!("Got {} outdated packages from poll", packages.len());
            self.loading_outdated = false;
            installed_refresh_finished = true;

            if is_terminal_load_error(&packages, &logs, "Error loading outdated") {
                self.status_message = "Failed to load outdated packages".to_string();
                self.toast_manager.error("Failed to load outdated packages");
            } else {
                self.merged_packages.update_outdated_packages(packages);
                installed_refresh_succeeded = true;
            }
        }

        if installed_refresh_finished
            && installed_refresh_succeeded
            && !self.loading_installed
            && !self.loading_outdated
        {
            self.tab_manager.mark_loaded(Tab::Installed);
            self.status_message = "Packages loaded".to_string();
        }

        if let Some((packages, logs)) = result.search_results {
            let search_failed = is_terminal_load_error(&packages, &logs, "Error searching");
            self.loading_search = false;
            if search_failed {
                self.status_message = "Search failed".to_string();
                self.toast_manager.error("Search failed");
            } else {
                self.search_results.update_packages(packages.clone());
                self.status_message = "Search completed".to_string();
            }

            if self.auto_load_version_info && !search_failed {
                tracing::info!("Auto-loading version info for {} packages", packages.len());
                for package in packages.iter() {
                    if package.version.is_none() && !package.version_load_failed {
                        tracing::debug!("Auto-loading info for {}", package.name);
                        self.load_package_info(package.name.clone(), package.package_type);
                    }
                }
            }
        }

        if let Some((_name, package)) = result.package_info {
            self.search_results.update_package(package.clone());
            self.merged_packages.update_package(package);
        }

        if let Some((success, message)) = result.install_completed {
            self.loading_install = false;
            self.loading = false;
            let installed_pkg_name = self.current_install_package.clone();
            if let Some(pkg) = &installed_pkg_name {
                self.packages_in_operation.remove(pkg);
            }
            self.status_message = message.clone();

            // Record to operation history
            {
                let pkg_type = installed_pkg_name.as_ref().and_then(|name| {
                    self.search_results
                        .get_package(name)
                        .or_else(|| self.merged_packages.get_package(name))
                        .map(|p| p.package_type)
                });
                self.record_operation(
                    OperationType::Install,
                    installed_pkg_name.clone(),
                    pkg_type,
                    success,
                    if success { None } else { Some(message.clone()) },
                );
            }

            if success {
                self.toast_manager.success(message.clone());
                if let Some(pkg_name) = installed_pkg_name {
                    if let Some(mut pkg) = self.search_results.get_package(&pkg_name) {
                        pkg.installed = true;
                        self.search_results.update_package(pkg);
                    }

                    self.merged_packages.mark_package_updated(&pkg_name);
                    self.merged_packages
                        .remove_from_outdated_selection_by_name(&pkg_name);
                }
                self.current_install_package = None;
            } else if self.is_password_error(&message) {
                if let Some(pkg_name) = &installed_pkg_name
                    && let Some(pkg) = self.search_results.get_package(pkg_name)
                {
                    self.pending_operation = Some(PendingOperation::Install(pkg));
                    self.password_modal.show(format!("Install {}", pkg_name));
                }
            } else {
                self.toast_manager.error(message.clone());
                self.current_install_package = None;
            }
        }

        if let Some((success, message)) = result.uninstall_completed {
            self.loading_uninstall = false;
            self.loading = false;
            let uninstall_pkg_name = self.current_uninstall_package.clone();
            if let Some(pkg) = &uninstall_pkg_name {
                self.packages_in_operation.remove(pkg);
            }
            self.status_message = message.clone();

            // Record to operation history
            {
                let pkg_type = uninstall_pkg_name.as_ref().and_then(|name| {
                    self.merged_packages
                        .get_package(name)
                        .or_else(|| self.search_results.get_package(name))
                        .map(|p| p.package_type)
                });
                self.record_operation(
                    OperationType::Uninstall,
                    uninstall_pkg_name.clone(),
                    pkg_type,
                    success,
                    if success { None } else { Some(message.clone()) },
                );
            }

            if success {
                self.toast_manager.success(message.clone());
                if let Some(pkg) = self.current_uninstall_package.as_ref() {
                    self.merged_packages.remove_installed_package(pkg);
                }
                self.current_uninstall_package = None;
            } else if self.is_password_error(&message) {
                if let Some(pkg_name) = &uninstall_pkg_name
                    && let Some(pkg) = self.merged_packages.get_package(pkg_name)
                {
                    self.pending_operation = Some(PendingOperation::Uninstall(pkg));
                    self.password_modal.show(format!("Uninstall {}", pkg_name));
                }
            } else {
                self.toast_manager.error(message.clone());
                self.current_uninstall_package = None;
            }
        }

        if let Some((success, message)) = result.update_completed {
            self.loading_update = false;
            self.loading = false;
            let pkg = self.current_update_package.take();
            if let Some(ref pkg_name) = pkg {
                self.packages_in_operation.remove(pkg_name);
            }
            self.status_message = message.clone();

            // Record to operation history
            {
                let pkg_type = pkg.as_ref().and_then(|name| {
                    self.merged_packages
                        .get_package(name)
                        .map(|p| p.package_type)
                });
                self.record_operation(
                    OperationType::Update,
                    pkg.clone(),
                    pkg_type,
                    success,
                    if success { None } else { Some(message.clone()) },
                );
            }

            if success {
                self.toast_manager.success(message.clone());
            } else {
                self.toast_manager.error(message.clone());
            }

            if success && let Some(pkg_name) = pkg {
                self.merged_packages.mark_package_updated(&pkg_name);
                self.merged_packages
                    .remove_from_outdated_selection_by_name(&pkg_name);
            }

            if self.loading_update_all && !self.pending_updates.is_empty() {
                self.process_next_pending_update();
                self.loading_update = true;
            } else if self.loading_update_all && self.pending_updates.is_empty() {
                self.loading_update_all = false;
                self.status_message = "Finished updating all packages".to_string();
                self.toast_manager.success("Finished updating all packages");
                self.log_manager
                    .push("Finished updating all packages".to_string());
                tracing::info!("Finished updating all packages");
                self.merged_packages.clear_outdated_selection();
            }
        }

        if let Some((success, message)) = result.update_all_completed {
            self.loading_update_all = false;
            self.loading = false;
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::UpdateAll,
                None,
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
                for pkg_name in self.packages_in_operation.iter() {
                    self.merged_packages.mark_package_updated(pkg_name);
                    self.merged_packages
                        .remove_from_outdated_selection_by_name(pkg_name);
                }
                self.packages_in_operation.clear();
            } else {
                self.toast_manager.error(message);
            }

            self.merged_packages.clear_outdated_selection();
        }

        if let Some((success, message)) = result.clean_cache_completed {
            self.loading_clean_cache = false;
            self.loading = false;
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::CleanCache,
                None,
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
            } else {
                self.toast_manager.error(message);
            }
            self.cleanup_modal.close();
        }

        if let Some((success, message)) = result.cleanup_old_versions_completed {
            self.loading_cleanup_old_versions = false;
            self.loading = false;
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::CleanupOldVersions,
                None,
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
            } else {
                self.toast_manager.error(message);
            }
            self.cleanup_modal.close();
        }

        if let Some((success, message)) = result.clean_orphans_completed {
            self.loading_clean_orphans = false;
            self.loading = false;
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::CleanOrphans,
                None,
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
            } else {
                self.toast_manager.error(message);
            }
            self.cleanup_modal.close();
        }

        if let Some((package_name, success, message)) = result.pin_completed {
            self.packages_in_operation.remove(&package_name);
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::Pin,
                Some(package_name),
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((package_name, success, message)) = result.unpin_completed {
            self.packages_in_operation.remove(&package_name);
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::Unpin,
                Some(package_name),
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((services, logs)) = result.services {
            tracing::info!("Got {} services from poll", services.len());
            self.loading_services = false;
            services_refresh_finished = true;

            if is_terminal_load_error(&services, &logs, "Error loading services") {
                self.status_message = "Failed to load services".to_string();
                self.toast_manager.error("Failed to load services");
            } else {
                self.service_list.update_services(services);
                self.status_message = "Services loaded".to_string();
                services_refresh_succeeded = true;
            }
        }

        if services_refresh_finished && services_refresh_succeeded && !self.loading_services {
            self.tab_manager.mark_loaded(Tab::Services);
        }

        if let Some((service_name, success, message)) = result.start_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::ServiceStart,
                Some(service_name),
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
                self.load_services();
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((service_name, success, message)) = result.stop_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::ServiceStop,
                Some(service_name),
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
                self.load_services();
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((service_name, success, message)) = result.restart_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message.clone();

            // Record to operation history
            self.record_operation(
                OperationType::ServiceRestart,
                Some(service_name),
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
                self.load_services();
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((service_name, info_result)) = result.service_info_result {
            match info_result {
                Ok(info) => {
                    self.log_manager
                        .push(format!("Loaded info for service: {}", service_name));
                    self.service_list.set_service_info(&service_name, info);
                }
                Err(e) => {
                    self.log_manager.push(format!("Error: {}", e));
                    self.service_list.set_service_info_error(&service_name, e);
                }
            }
        }

        if let Some((service_name, log_result)) = result.service_log_result {
            match log_result {
                Ok(log_text) => {
                    self.log_manager
                        .push(format!("Loaded log for service: {}", service_name));
                    self.service_list.set_service_log(&service_name, log_text);
                }
                Err(e) => {
                    self.log_manager.push(format!("Error: {}", e));
                    self.service_list.set_service_log_error(&service_name, e);
                }
            }
        }

        if let Some((success, message)) = result.export_packages_completed {
            self.loading_export = false;
            self.loading = false;
            self.status_message = message.clone();
            if success {
                self.toast_manager.success(message);
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((success, message)) = result.import_packages_completed {
            self.loading_import = false;
            self.loading = false;
            self.status_message = message.clone();
            if success {
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((cleanup_type, preview_result)) = result.cleanup_preview_result {
            self.loading = false;
            match preview_result {
                Ok(preview) => {
                    let msg = format!(
                        "Found {} items to clean ({})",
                        preview.items.len(),
                        format_size(preview.total_size)
                    );
                    self.log_manager.push(msg);
                    self.cleanup_modal.show_preview(cleanup_type, preview);
                }
                Err(e) => {
                    self.log_manager.push(e.clone());
                    self.status_message = e;
                }
            }
        }

        if let Some(doctor_result) = result.doctor_result {
            self.loading = false;
            match doctor_result {
                Ok(output) => {
                    self.log_manager.push("Brew doctor completed".to_string());
                    self.toast_manager
                        .success("Brew doctor completed successfully");
                    self.doctor_output = Some(output);
                }
                Err(e) => {
                    let msg = format!("Brew doctor error: {}", e);
                    self.log_manager.push(msg.clone());
                    self.toast_manager.error(msg.clone());
                    self.doctor_output = Some(crate::domain::entities::DoctorOutput {
                        is_ready: false,
                        warnings: vec![crate::domain::entities::DoctorWarning {
                            title: "Error running brew doctor".to_string(),
                            body: e,
                        }],
                        raw_output: msg,
                    });
                }
            }
        }

        if let Some((taps, logs)) = result.taps {
            self.loading = false;
            if is_terminal_load_error(&taps, &logs, "Error loading taps") {
                self.status_message = "Failed to load taps".to_string();
                self.toast_manager.error("Failed to load taps");
            } else {
                self.taps = taps;
                self.status_message = "Taps loaded".to_string();
            }
        }

        if let Some((success, message)) = result.tap_completed {
            self.loading = false;
            self.status_message = message.clone();
            if success {
                self.toast_manager.success(message);
                self.load_taps();
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((success, message)) = result.untap_completed {
            self.loading = false;
            self.status_message = message.clone();
            if success {
                self.toast_manager.success(message);
                self.load_taps();
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some((success, message)) = result.bundle_dump_completed {
            self.loading_bundle_dump = false;
            self.loading = false;
            self.status_message = message.clone();
            if success {
                self.toast_manager.success(message);
            } else {
                self.toast_manager.error(message);
            }
        }

        if let Some(preview_result) = result.bundle_check_preview_result {
            self.loading_bundle_check = false;
            self.loading = false;
            match preview_result {
                Ok(preview) => {
                    let msg = format!(
                        "Brewfile check: {} missing, {} extra",
                        preview.missing_dependencies.len(),
                        preview.extra_dependencies.len()
                    );
                    self.log_manager.push(msg);
                    self.brewfile_sync_modal.show_preview(preview);
                }
                Err(e) => {
                    self.log_manager.push(e.clone());
                    self.toast_manager.error(e.clone());
                    self.status_message = e;
                }
            }
        }

        if let Some((success, message)) = result.bundle_apply_completed {
            self.loading_bundle_apply = false;
            self.loading = false;
            self.status_message = message.clone();
            self.brewfile_sync_modal.close();

            // Record to operation history
            self.record_operation(
                OperationType::BundleApply,
                self.current_brewfile_path.clone(),
                None,
                success,
                if success { None } else { Some(message.clone()) },
            );

            if success {
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                self.toast_manager.error(message);
            }
        }

        self.log_manager.extend(result.logs);

        // Poll pending deps load for info modal
        let deps_ready = self
            .pending_deps_load
            .as_ref()
            .and_then(|arc| arc.try_lock().ok().and_then(|guard| guard.clone()));
        if let Some((deps, used_by)) = deps_ready {
            self.info_modal.set_deps(deps, used_by);
            self.pending_deps_load = None;
        }

        if self.task_manager.can_load_more_package_info()
            && self.task_manager.pending_loads_count() > 0
        {
            let to_load = self.task_manager.available_package_info_slots();
            let batch = self.task_manager.drain_pending_loads(to_load);

            if !batch.is_empty() {
                tracing::info!(
                    "Starting batch load of {} packages ({} remaining in queue)",
                    batch.len(),
                    self.task_manager.pending_loads_count()
                );

                for (name, pkg_type) in batch {
                    self.load_package_info_immediate(name, pkg_type);
                }
            }
        }
    }

    pub(super) fn poll_logs(&mut self) {
        while let Ok(log_entry) = self.log_rx.try_recv() {
            self.log_manager.push(log_entry);
        }
    }
}

fn is_terminal_load_error<T>(data: &[T], logs: &[String], error_prefix: &str) -> bool {
    data.is_empty()
        && !logs.is_empty()
        && logs.iter().all(|message| message.starts_with(error_prefix))
}
