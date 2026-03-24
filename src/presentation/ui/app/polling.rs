use crate::domain::entities::OperationType;
use crate::domain::entities::{AppError, LoadState, MessageSeverity, UserMessage};
use crate::presentation::components::Tab;
use crate::presentation::services::desktop_notifications;

use super::BrewstyApp;
use super::format_size;

impl BrewstyApp {
    pub(super) fn poll_async_tasks(&mut self) {
        tracing::trace!("poll_async_tasks called, checking for active task");
        let result = self.task_manager.poll();
        let mut installed_refresh_finished = false;
        let mut installed_refresh_succeeded = false;
        let mut services_refresh_finished = false;
        let mut services_refresh_succeeded = false;

        if let Some(state) = result.installed_packages {
            self.loading_installed = false;
            installed_refresh_finished = true;
            self.installed_state = state.clone();

            match state {
                LoadState::Ready(packages) => {
                    tracing::info!("Got {} installed packages from poll", packages.len());
                    self.merged_packages.update_packages(packages);
                    self.installed_message = None;
                    installed_refresh_succeeded = true;
                }
                LoadState::Partial { data, warning } => {
                    self.merged_packages.update_packages(data);
                    self.installed_message = Some(load_warning_message(
                        "Installed packages loaded with warnings",
                        &warning,
                    ));
                    installed_refresh_succeeded = true;
                }
                LoadState::Error(error) => {
                    self.installed_message =
                        Some(error.to_user_message("Failed to load installed packages"));
                    self.set_operation_failure(error);
                }
                LoadState::Idle | LoadState::Loading => {}
            }
        }

        if let Some(state) = result.outdated_packages {
            self.loading_outdated = false;
            installed_refresh_finished = true;
            self.outdated_state = state.clone();

            match state {
                LoadState::Ready(packages) => {
                    tracing::info!("Got {} outdated packages from poll", packages.len());
                    self.merged_packages.update_outdated_packages(packages);
                    installed_refresh_succeeded = true;
                }
                LoadState::Partial { data, warning } => {
                    self.merged_packages.update_outdated_packages(data);
                    self.installed_message = Some(load_warning_message(
                        "Outdated packages loaded with warnings",
                        &warning,
                    ));
                    installed_refresh_succeeded = true;
                }
                LoadState::Error(error) => {
                    self.installed_message =
                        Some(error.to_user_message("Failed to load outdated packages"));
                    self.set_operation_failure(error);
                }
                LoadState::Idle | LoadState::Loading => {}
            }
        }

        if installed_refresh_finished
            && installed_refresh_succeeded
            && !self.loading_installed
            && !self.loading_outdated
        {
            self.tab_manager.mark_loaded(Tab::Installed);
            self.set_operation_success("Packages loaded");
        }

        if let Some(state) = result.search_results {
            self.loading_search = false;
            self.search_state = state.clone();
            match state {
                LoadState::Ready(packages) => {
                    self.search_results.update_packages(packages.clone());
                    self.search_message = None;
                    self.set_operation_success("Search completed");

                    if self.auto_load_version_info {
                        tracing::info!("Auto-loading version info for {} packages", packages.len());
                        for package in packages.iter() {
                            if package.version.is_none() && !package.version_load_failed {
                                tracing::debug!("Auto-loading info for {}", package.name);
                                self.load_package_info(package.name.clone(), package.package_type);
                            }
                        }
                    }
                }
                LoadState::Partial { data, warning } => {
                    self.search_results.update_packages(data);
                    self.search_message = Some(load_warning_message(
                        "Search completed with warnings",
                        &warning,
                    ));
                    self.set_operation_failure(warning);
                }
                LoadState::Error(error) => {
                    self.search_message = Some(error.to_user_message("Search failed"));
                    self.set_operation_failure(error);
                }
                LoadState::Idle | LoadState::Loading => {}
            }
        }

        if let Some((_name, package)) = result.package_info {
            self.search_results.update_package(package.clone());
            self.merged_packages.update_package(package);
        }

        if let Some(result) = result.install_completed {
            self.loading_install = false;
            self.loading = false;
            let installed_pkg_name = self.current_install_package.clone();
            if let Some(pkg) = &installed_pkg_name {
                self.packages_in_operation.remove(pkg);
            }

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
                    result.is_ok(),
                    result.as_ref().err().map(|error| error.short_message()),
                );
            }

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message.clone());

                // Send desktop notification
                if self.config.notifications.enabled && self.config.notifications.show_on_install {
                    desktop_notifications::notify_success(
                        "Package Installed",
                        &format!(
                            "{} has been successfully installed",
                            installed_pkg_name.as_ref().unwrap_or(&String::new())
                        ),
                    );
                }

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
            } else if matches!(result, Err(AppError::AuthCancelled)) {
                self.toast_manager.info("Password prompt cancelled");
                self.current_install_package = None;
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Install failed".to_string()));
                self.set_operation_failure(error.clone());
                self.installed_message = Some(error.to_user_message("Install failed"));

                // Send error notification
                if self.config.notifications.enabled && self.config.notifications.show_on_error {
                    desktop_notifications::notify_error(
                        "Install Failed",
                        &format!(
                            "Failed to install {}: {}",
                            installed_pkg_name.as_ref().unwrap_or(&String::new()),
                            error.short_message()
                        ),
                    );
                }

                self.current_install_package = None;
            }
        }

        if let Some(result) = result.uninstall_completed {
            self.loading_uninstall = false;
            self.loading = false;
            let uninstall_pkg_name = self.current_uninstall_package.clone();
            if let Some(pkg) = &uninstall_pkg_name {
                self.packages_in_operation.remove(pkg);
            }
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
                    result.is_ok(),
                    result.as_ref().err().map(|error| error.short_message()),
                );
            }

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message.clone());

                // Send desktop notification
                if self.config.notifications.enabled && self.config.notifications.show_on_install {
                    desktop_notifications::notify_success(
                        "Package Uninstalled",
                        &format!(
                            "{} has been successfully uninstalled",
                            uninstall_pkg_name.as_ref().unwrap_or(&String::new())
                        ),
                    );
                }

                if let Some(pkg) = self.current_uninstall_package.as_ref() {
                    self.merged_packages.remove_installed_package(pkg);
                }
                self.current_uninstall_package = None;
            } else if matches!(result, Err(AppError::AuthCancelled)) {
                self.toast_manager.info("Password prompt cancelled");
                self.current_uninstall_package = None;
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Uninstall failed".to_string()));
                self.set_operation_failure(error.clone());
                self.installed_message = Some(error.to_user_message("Uninstall failed"));

                // Send error notification
                if self.config.notifications.enabled && self.config.notifications.show_on_error {
                    desktop_notifications::notify_error(
                        "Uninstall Failed",
                        &format!(
                            "Failed to uninstall {}: {}",
                            uninstall_pkg_name.as_ref().unwrap_or(&String::new()),
                            error.short_message()
                        ),
                    );
                }

                self.current_uninstall_package = None;
            }
        }

        if let Some(result) = result.update_completed {
            self.loading_update = false;
            self.loading = false;
            let pkg = self.current_update_package.take();
            if let Some(ref pkg_name) = pkg {
                self.packages_in_operation.remove(pkg_name);
            }
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
                    result.is_ok(),
                    result.as_ref().err().map(|error| error.short_message()),
                );
            }

            match &result {
                Ok(message) => {
                    self.set_operation_success(message.clone());
                    self.toast_manager.success(message.clone());

                    // Send desktop notification
                    if self.config.notifications.enabled && self.config.notifications.show_on_update
                    {
                        desktop_notifications::notify_success(
                            "Package Updated",
                            &format!(
                                "{} has been successfully updated",
                                pkg.as_ref().unwrap_or(&String::new())
                            ),
                        );
                    }
                }
                Err(error) => {
                    self.set_operation_failure(error.clone());
                    self.installed_message = Some(error.to_user_message("Update failed"));

                    // Send error notification
                    if self.config.notifications.enabled && self.config.notifications.show_on_error
                    {
                        desktop_notifications::notify_error(
                            "Update Failed",
                            &format!(
                                "Failed to update {}: {}",
                                pkg.as_ref().unwrap_or(&String::new()),
                                error.short_message()
                            ),
                        );
                    }
                }
            }

            if result.is_ok()
                && let Some(pkg_name) = pkg
            {
                self.merged_packages.mark_package_updated(&pkg_name);
                self.merged_packages
                    .remove_from_outdated_selection_by_name(&pkg_name);
            }

            if self.loading_update_all && !self.pending_updates.is_empty() {
                self.process_next_pending_update();
                self.loading_update = true;
            } else if self.loading_update_all && self.pending_updates.is_empty() {
                self.loading_update_all = false;
                self.set_operation_success("Finished updating all packages");
                self.toast_manager.success("Finished updating all packages");
                self.log_manager
                    .push("Finished updating all packages".to_string());
                tracing::info!("Finished updating all packages");
                self.merged_packages.clear_outdated_selection();
            }
        }

        if let Some(result) = result.update_all_completed {
            self.loading_update_all = false;
            self.loading = false;

            // Record to operation history
            self.record_operation(
                OperationType::UpdateAll,
                None,
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                for pkg_name in self.packages_in_operation.iter() {
                    self.merged_packages.mark_package_updated(pkg_name);
                    self.merged_packages
                        .remove_from_outdated_selection_by_name(pkg_name);
                }
                self.packages_in_operation.clear();
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Update all failed".to_string()));
                self.set_operation_failure(error.clone());
                self.installed_message = Some(error.to_user_message("Update all failed"));
            }

            self.merged_packages.clear_outdated_selection();
        }

        if let Some(result) = result.clean_cache_completed {
            self.loading_clean_cache = false;
            self.loading = false;
            self.record_operation(
                OperationType::CleanCache,
                None,
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Cache cleanup failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Cache cleanup failed"));
            }
            self.cleanup_modal.close();
        }

        if let Some(result) = result.cleanup_old_versions_completed {
            self.loading_cleanup_old_versions = false;
            self.loading = false;
            self.record_operation(
                OperationType::CleanupOldVersions,
                None,
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Old version cleanup failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Old version cleanup failed"));
            }
            self.cleanup_modal.close();
        }

        if let Some(result) = result.clean_orphans_completed {
            self.loading_clean_orphans = false;
            self.loading = false;
            self.record_operation(
                OperationType::CleanOrphans,
                None,
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Orphan cleanup failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Orphan cleanup failed"));
            }
            self.cleanup_modal.close();
        }

        if let Some((package_name, result)) = result.pin_completed {
            self.packages_in_operation.remove(&package_name);
            self.record_operation(
                OperationType::Pin,
                Some(package_name),
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Pin failed".to_string()));
                self.set_operation_failure(error.clone());
                self.installed_message = Some(error.to_user_message("Pin failed"));
            }
        }

        if let Some((package_name, result)) = result.unpin_completed {
            self.packages_in_operation.remove(&package_name);
            self.record_operation(
                OperationType::Unpin,
                Some(package_name),
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Unpin failed".to_string()));
                self.set_operation_failure(error.clone());
                self.installed_message = Some(error.to_user_message("Unpin failed"));
            }
        }

        if let Some(state) = result.services {
            self.loading_services = false;
            services_refresh_finished = true;
            self.services_state = state.clone();

            match state {
                LoadState::Ready(services) => {
                    tracing::info!("Got {} services from poll", services.len());
                    self.service_list.update_services(services);
                    self.services_message = None;
                    services_refresh_succeeded = true;
                    self.set_operation_success("Services loaded");
                }
                LoadState::Partial { data, warning } => {
                    self.service_list.update_services(data);
                    self.services_message = Some(load_warning_message(
                        "Services loaded with warnings",
                        &warning,
                    ));
                    services_refresh_succeeded = true;
                }
                LoadState::Error(error) => {
                    self.services_message = Some(error.to_user_message("Failed to load services"));
                    self.set_operation_failure(error);
                }
                LoadState::Idle | LoadState::Loading => {}
            }
        }

        if services_refresh_finished && services_refresh_succeeded && !self.loading_services {
            self.tab_manager.mark_loaded(Tab::Services);
        }

        if let Some((service_name, result)) = result.start_service_completed {
            self.services_in_operation.remove(&service_name);
            self.record_operation(
                OperationType::ServiceStart,
                Some(service_name),
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_services();
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Service start failed".to_string()));
                self.set_operation_failure(error.clone());
                self.services_message = Some(error.to_user_message("Service start failed"));
            }
        }

        if let Some((service_name, result)) = result.stop_service_completed {
            self.services_in_operation.remove(&service_name);
            self.record_operation(
                OperationType::ServiceStop,
                Some(service_name),
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_services();
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Service stop failed".to_string()));
                self.set_operation_failure(error.clone());
                self.services_message = Some(error.to_user_message("Service stop failed"));
            }
        }

        if let Some((service_name, result)) = result.restart_service_completed {
            self.services_in_operation.remove(&service_name);
            self.record_operation(
                OperationType::ServiceRestart,
                Some(service_name),
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_services();
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Service restart failed".to_string()));
                self.set_operation_failure(error.clone());
                self.services_message = Some(error.to_user_message("Service restart failed"));
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
                    self.log_manager
                        .push(format!("Error: {}", e.short_message()));
                    self.service_list
                        .set_service_info_error(&service_name, e.short_message());
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
                    self.log_manager
                        .push(format!("Error: {}", e.short_message()));
                    self.service_list
                        .set_service_log_error(&service_name, e.short_message());
                }
            }
        }

        if let Some(result) = result.export_packages_completed {
            self.loading_export = false;
            self.loading = false;
            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Export failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Export failed"));
            }
        }

        if let Some(result) = result.import_packages_completed {
            self.loading_import = false;
            self.loading = false;
            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Import failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Import failed"));
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
                    self.log_manager.push(e.short_message());
                    self.set_operation_failure(e.clone());
                    self.settings_message =
                        Some(e.to_user_message("Failed to load cleanup preview"));
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
                    self.settings_message = None;
                }
                Err(e) => {
                    let msg = format!("Brew doctor error: {}", e.short_message());
                    self.log_manager.push(msg.clone());
                    self.settings_message = Some(e.to_user_message("Brew doctor failed"));
                    self.set_operation_failure(e.clone());
                    self.doctor_output = Some(crate::domain::entities::DoctorOutput {
                        is_ready: false,
                        warnings: vec![crate::domain::entities::DoctorWarning {
                            title: "Error running brew doctor".to_string(),
                            body: msg,
                        }],
                        raw_output: e.short_message(),
                    });
                }
            }
        }

        if let Some(state) = result.taps {
            self.loading = false;
            self.taps_state = state.clone();
            match state {
                LoadState::Ready(taps) => {
                    self.taps = taps;
                    self.settings_message = None;
                    self.set_operation_success("Taps loaded");
                }
                LoadState::Partial { data, warning } => {
                    self.taps = data;
                    self.settings_message =
                        Some(load_warning_message("Taps loaded with warnings", &warning));
                }
                LoadState::Error(error) => {
                    self.settings_message = Some(error.to_user_message("Failed to load taps"));
                    self.set_operation_failure(error);
                }
                LoadState::Idle | LoadState::Loading => {}
            }
        }

        if let Some(result) = result.tap_completed {
            self.loading = false;
            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_taps();
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Tap failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Tap failed"));
            }
        }

        if let Some(result) = result.untap_completed {
            self.loading = false;
            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_taps();
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Untap failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Untap failed"));
            }
        }

        if let Some(result) = result.bundle_dump_completed {
            self.loading_bundle_dump = false;
            self.loading = false;
            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Brewfile export failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Brewfile export failed"));
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
                    self.log_manager.push(e.short_message());
                    self.settings_message = Some(e.to_user_message("Failed to analyze Brewfile"));
                    self.set_operation_failure(e);
                }
            }
        }

        if let Some(result) = result.bundle_apply_completed {
            self.loading_bundle_apply = false;
            self.loading = false;
            self.brewfile_sync_modal.close();

            // Record to operation history
            self.record_operation(
                OperationType::BundleApply,
                self.current_brewfile_path.clone(),
                None,
                result.is_ok(),
                result.as_ref().err().map(|error| error.short_message()),
            );

            if let Ok(message) = result {
                self.set_operation_success(message.clone());
                self.toast_manager.success(message);
                self.load_installed_packages(true);
            } else {
                let error = result
                    .err()
                    .unwrap_or_else(|| AppError::Unknown("Brewfile apply failed".to_string()));
                self.set_operation_failure(error.clone());
                self.settings_message = Some(error.to_user_message("Brewfile apply failed"));
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

fn load_warning_message(title: &str, error: &AppError) -> UserMessage {
    UserMessage::new(title, error.short_message(), MessageSeverity::Warning)
        .with_details(error.details().unwrap_or_else(|| error.short_message()))
        .with_recovery_action("Retry")
}
