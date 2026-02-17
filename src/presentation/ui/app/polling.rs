use crate::presentation::components::Tab;

use super::format_size;
use super::{BrewstyApp, PendingOperation};

impl BrewstyApp {
    pub(super) fn poll_async_tasks(&mut self) {
        tracing::trace!("poll_async_tasks called, checking for active task");
        let result = self.task_manager.poll();

        if let Some(packages) = result.installed_packages {
            tracing::info!("Got {} installed packages from poll", packages.len());
            self.merged_packages.update_packages(packages);
            self.loading_installed = false;
        }

        if let Some(packages) = result.outdated_packages {
            tracing::info!("Got {} outdated packages from poll", packages.len());
            self.merged_packages.update_outdated_packages(packages);
            self.loading_outdated = false;
        }

        if !self.loading_installed && !self.loading_outdated {
            self.tab_manager.mark_loaded(Tab::Installed);
            self.status_message = "Packages loaded".to_string();
        }

        if let Some(packages) = result.search_results {
            self.search_results.update_packages(packages.clone());
            self.loading_search = false;
            self.status_message = "Search completed".to_string();

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

            if success {
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
                    && let Some(pkg) = self.search_results.get_package(pkg_name) {
                        self.pending_operation = Some(PendingOperation::Install(pkg));
                        self.password_modal.show(format!("Install {}", pkg_name));
                    }
            } else {
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

            if success {
                if let Some(pkg) = self.current_uninstall_package.as_ref() {
                    self.merged_packages.remove_installed_package(pkg);
                }
                self.current_uninstall_package = None;
            } else if self.is_password_error(&message) {
                if let Some(pkg_name) = &uninstall_pkg_name
                    && let Some(pkg) = self.merged_packages.get_package(pkg_name) {
                        self.pending_operation = Some(PendingOperation::Uninstall(pkg));
                        self.password_modal.show(format!("Uninstall {}", pkg_name));
                    }
            } else {
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
            self.status_message = message;

            if success
                && let Some(pkg_name) = pkg {
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
                self.log_manager
                    .push("Finished updating all packages".to_string());
                tracing::info!("Finished updating all packages");
                self.merged_packages.clear_outdated_selection();
            }
        }

        if let Some((success, message)) = result.update_all_completed {
            self.loading_update_all = false;
            self.loading = false;
            self.status_message = message;

            if success {
                for pkg_name in self.packages_in_operation.iter() {
                    self.merged_packages.mark_package_updated(pkg_name);
                    self.merged_packages
                        .remove_from_outdated_selection_by_name(pkg_name);
                }
                self.packages_in_operation.clear();
            }

            self.merged_packages.clear_outdated_selection();
        }

        if let Some((_success, message)) = result.clean_cache_completed {
            self.loading_clean_cache = false;
            self.loading = false;
            self.status_message = message;
            self.cleanup_modal.close();
        }

        if let Some((_success, message)) = result.cleanup_old_versions_completed {
            self.loading_cleanup_old_versions = false;
            self.loading = false;
            self.status_message = message;
            self.cleanup_modal.close();
        }

        if let Some((package_name, _success, message)) = result.pin_completed {
            self.packages_in_operation.remove(&package_name);
            self.status_message = message;
            self.load_installed_packages(true);
        }

        if let Some((package_name, _success, message)) = result.unpin_completed {
            self.packages_in_operation.remove(&package_name);
            self.status_message = message;
            self.load_installed_packages(true);
        }

        if let Some(services) = result.services {
            tracing::info!("Got {} services from poll", services.len());
            self.service_list.update_services(services);
            self.loading_services = false;
            self.tab_manager.mark_loaded(Tab::Services);
            self.status_message = "Services loaded".to_string();
        }

        if let Some((service_name, success, message)) = result.start_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message;
            if success {
                self.load_services();
            }
        }

        if let Some((service_name, success, message)) = result.stop_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message;
            if success {
                self.load_services();
            }
        }

        if let Some((service_name, success, message)) = result.restart_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message;
            if success {
                self.load_services();
            }
        }

        if let Some((_success, message)) = result.export_packages_completed {
            self.loading_export = false;
            self.loading = false;
            self.status_message = message;
        }

        if let Some((success, message)) = result.import_packages_completed {
            self.loading_import = false;
            self.loading = false;
            self.status_message = message;
            if success {
                self.load_installed_packages(true);
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
                    self.doctor_output = Some(output);
                }
                Err(e) => {
                    self.log_manager.push(format!("Brew doctor error: {}", e));
                    self.doctor_output = Some(format!("Error: {}", e));
                }
            }
        }

        if let Some(taps) = result.taps {
            self.taps = taps;
            self.loading = false;
            self.status_message = "Taps loaded".to_string();
        }

        if let Some((_success, message)) = result.tap_completed {
            self.loading = false;
            self.status_message = message;
            self.load_taps();
        }

        if let Some((_success, message)) = result.untap_completed {
            self.loading = false;
            self.status_message = message;
            self.load_taps();
        }

        self.log_manager.extend(result.logs);

        // Poll pending deps load for info modal
        let deps_ready = self.pending_deps_load.as_ref().and_then(|arc| {
            arc.try_lock().ok().and_then(|guard| guard.clone())
        });
        if let Some((deps, used_by)) = deps_ready {
            self.info_modal.set_deps(deps, used_by);
            self.pending_deps_load = None;
        }

        if self.task_manager.can_load_more_package_info()
            && self.task_manager.pending_loads_count() > 0
        {
            let to_load = 15 - self.task_manager.pending_loads_count();
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
